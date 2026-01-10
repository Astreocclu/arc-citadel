"""Main world assembly pipeline."""

import random
import time
from typing import Optional, Any

from worldgen.schemas import (
    WorldSeed, HexMap, AssembledCluster, AssembledWorld,
    ConnectorCollection, ConnectorType, MinorAnchor,
    WorldHex, HexCoord, Terrain, SpeciesFitness,
    TaggedHex, HexCluster,
)
from worldgen.hex_coords import coords_to_key
from .layout_solver import LayoutSolver


# Tag to terrain mapping
TAG_TO_TERRAIN = {
    "underground": Terrain.UNDERGROUND,
    "surface": Terrain.PLAINS,
    "underwater": Terrain.DEEP_WATER,
    "aerial": Terrain.PLAINS,  # No aerial terrain, use plains
    "deep": Terrain.CAVERN,
    "shallow_under": Terrain.UNDERGROUND,
    "elevated": Terrain.HILLS,
    "peak": Terrain.HIGH_MOUNTAINS,
    "wild": Terrain.FOREST,
    "corrupted": Terrain.MARSH,
}


class WorldAssembler:
    """Assemble a complete world from a seed.

    Pipeline order:
    1. Place clusters (existing)
    2. Generate connectors between clusters
    3. Place minor anchors along connectors
    4. Fill empty hexes with constraint propagation
    """

    def __init__(self, database: Optional[Any] = None):
        self.database = database
        self.layout_solver = LayoutSolver()

        # Lazy imports to avoid circular dependencies
        self._connector_generator = None
        self._anchor_placer = None
        self._filler_generator = None
        self._cluster_generator = None

    @property
    def cluster_generator(self):
        """LLM-based cluster generator for tagged hexes."""
        if self._cluster_generator is None:
            from worldgen.cluster_generator import ClusterGenerator
            self._cluster_generator = ClusterGenerator()
        return self._cluster_generator

    @property
    def connector_generator(self):
        if self._connector_generator is None:
            from worldgen.connector_generator import ConnectorGenerator
            self._connector_generator = ConnectorGenerator()
        return self._connector_generator

    @property
    def anchor_placer(self):
        if self._anchor_placer is None:
            from worldgen.anchor_placer import AnchorPlacer
            self._anchor_placer = AnchorPlacer()
        return self._anchor_placer

    @property
    def filler_generator(self):
        if self._filler_generator is None:
            from worldgen.filler_generator import FillerGenerator
            self._filler_generator = FillerGenerator()
        return self._filler_generator

    def assemble(self, seed: WorldSeed) -> AssembledWorld:
        """Assemble a complete world from a seed."""
        start_time = time.time()
        rng = random.Random(seed.seed_id)

        # 1. Create assembled clusters and solve layout
        assembled_clusters, cluster_positions, hex_map = self._place_clusters(seed, rng)

        # 2. Generate connectors between clusters
        connectors = self._generate_connectors(seed, cluster_positions, hex_map)

        # 3. Place minor anchors along connectors
        anchors = self._place_anchors(connectors)

        # 4. Generate filler hexes
        hex_map = self._generate_fillers(hex_map, seed.world_radius)

        # 5. Store data in hex_map
        hex_map.clusters = assembled_clusters
        hex_map.cluster_positions = cluster_positions

        elapsed_ms = int((time.time() - start_time) * 1000)

        return AssembledWorld(
            seed_id=seed.seed_id,
            name=seed.name,
            hex_map=hex_map,
            generation_time_ms=elapsed_ms,
            total_hexes=len(hex_map.hexes),
            total_clusters=len(assembled_clusters),
        )

    def _tagged_to_world_hex(
        self,
        tagged: TaggedHex,
        world_q: int,
        world_r: int,
        cluster_id: str,
    ) -> WorldHex:
        """Convert a TaggedHex to a WorldHex with world coordinates."""
        # Determine terrain from tags
        terrain = Terrain.PLAINS  # Default
        for tag in tagged.tags:
            if tag in TAG_TO_TERRAIN:
                terrain = TAG_TO_TERRAIN[tag]
                break

        # Determine elevation from tags
        elevation = 200.0  # Default surface
        if "deep" in tagged.tags:
            elevation = -500.0
        elif "shallow_under" in tagged.tags:
            elevation = -50.0
        elif "elevated" in tagged.tags:
            elevation = 400.0
        elif "peak" in tagged.tags:
            elevation = 800.0
        elif "underground" in tagged.tags:
            elevation = -100.0

        # Determine species fitness from culture tags
        human_fit = 0.5
        dwarf_fit = 0.5
        elf_fit = 0.5
        if "human" in tagged.tags:
            human_fit = 0.9
        if "dwarf" in tagged.tags:
            dwarf_fit = 0.9
        if "elf" in tagged.tags:
            elf_fit = 0.9
        if "wild" in tagged.tags:
            human_fit = 0.3
            elf_fit = 0.7

        return WorldHex(
            coord=HexCoord(q=world_q, r=world_r),
            terrain=terrain,
            elevation=elevation,
            moisture=0.5,
            temperature=15.0,
            species_fitness=SpeciesFitness(human=human_fit, dwarf=dwarf_fit, elf=elf_fit),
            cluster_id=cluster_id,
            # Tagged content
            name=tagged.name,
            description=tagged.description,
            tags=tagged.tags,
            edge_types=tagged.edge_types,
        )

    def _place_clusters(
        self, seed: WorldSeed, rng: random.Random
    ) -> tuple[dict[str, AssembledCluster], dict[str, tuple[int, int]], HexMap]:
        """Place clusters and convert to world hexes.

        For settlement templates, uses ClusterGenerator to create LLM-generated
        tagged hexes with names, descriptions, and cultural tags.
        """
        assembled_clusters: dict[str, AssembledCluster] = {}
        generated_clusters: dict[str, HexCluster] = {}

        for placement in seed.clusters:
            # Generate tagged cluster for settlements
            if placement.template_id == "settlement":
                print(f"Generating settlement cluster: {placement.instance_id}...")
                cluster = self.cluster_generator.generate(size=20)
                generated_clusters[placement.instance_id] = cluster
                # Build footprint from generated hexes
                footprint = [(h.q, h.r) for h in cluster.hexes]
            else:
                footprint = [(0, 0), (1, 0), (0, 1)]  # Minimal footprint

            assembled = AssembledCluster(
                template_id=placement.template_id,
                instance_id=placement.instance_id,
                components={},
                layout={},
                footprint=footprint,
            )
            assembled_clusters[placement.instance_id] = assembled

        # Solve global layout
        cluster_positions = self.layout_solver.solve(
            clusters=assembled_clusters,
            hints=seed.layout_hints,
            world_radius=seed.world_radius,
            rng=rng,
        )

        # Create hex map with cluster hexes
        hex_map = HexMap(
            seed_id=seed.seed_id,
            world_radius=seed.world_radius,
            hexes={},
            clusters={},
            cluster_positions={},
        )

        # Add cluster hexes to map
        for instance_id, cluster in assembled_clusters.items():
            base_pos = cluster_positions[instance_id]

            # Check if we have generated tagged hexes for this cluster
            if instance_id in generated_clusters:
                tagged_cluster = generated_clusters[instance_id]
                for tagged_hex in tagged_cluster.hexes:
                    world_q = base_pos[0] + tagged_hex.q
                    world_r = base_pos[1] + tagged_hex.r
                    key = coords_to_key(world_q, world_r)

                    hex_map.hexes[key] = self._tagged_to_world_hex(
                        tagged_hex, world_q, world_r, instance_id
                    )
            else:
                # Fallback: create placeholder hexes
                for offset in cluster.footprint:
                    world_q = base_pos[0] + offset[0]
                    world_r = base_pos[1] + offset[1]
                    key = coords_to_key(world_q, world_r)

                    hex_map.hexes[key] = WorldHex(
                        coord=HexCoord(q=world_q, r=world_r),
                        terrain=Terrain.PLAINS,
                        elevation=200.0,
                        moisture=0.5,
                        temperature=15.0,
                        species_fitness=SpeciesFitness(human=0.7, dwarf=0.5, elf=0.5),
                        cluster_id=instance_id,
                    )

        return assembled_clusters, cluster_positions, hex_map

    def _generate_connectors(
        self,
        seed: WorldSeed,
        cluster_positions: dict[str, tuple[int, int]],
        hex_map: HexMap,
    ) -> list[ConnectorCollection]:
        """Generate connectors based on seed assignments."""
        connectors = []
        placed_hexes = set(hex_map.hexes.keys())

        for assignment in seed.connectors:
            start_cluster = assignment.start_cluster
            end_cluster = assignment.end_cluster

            if not start_cluster or not end_cluster:
                continue
            if start_cluster not in cluster_positions:
                continue
            if end_cluster not in cluster_positions:
                continue

            start_pos = cluster_positions[start_cluster]
            end_pos = cluster_positions[end_cluster]

            # Determine connector type
            connector_type = ConnectorType.TRADE_ROUTE_MAJOR  # Default
            if "river" in assignment.collection_id.lower():
                connector_type = ConnectorType.RIVER_MIDDLE
            elif "military" in assignment.collection_id.lower():
                connector_type = ConnectorType.MILITARY_ROAD

            # Convert placed_hexes keys to tuples
            placed_tuples = set()
            for k in placed_hexes:
                parts = k.split(',')
                placed_tuples.add((int(parts[0]), int(parts[1])))

            # Generate connector
            connector = self.connector_generator.generate(
                connector_type=connector_type,
                start_pos=start_pos,
                end_pos=end_pos,
                placed_hexes=placed_tuples,
            )

            # Add connector hexes to map
            # Use actual path positions from A* result
            path = self._get_connector_path(start_pos, end_pos, len(connector.hexes))

            for i, (world_q, world_r) in enumerate(path):
                key = coords_to_key(world_q, world_r)

                if key not in hex_map.hexes and i < len(connector.hexes):
                    chex = connector.hexes[i]
                    hex_map.hexes[key] = WorldHex(
                        coord=HexCoord(q=world_q, r=world_r),
                        terrain=chex.terrain,
                        elevation=chex.elevation,
                        moisture=chex.moisture,
                        temperature=15.0,
                        species_fitness=SpeciesFitness(
                            human=chex.species_fitness.human,
                            dwarf=chex.species_fitness.dwarf,
                            elf=chex.species_fitness.elf,
                        ),
                    )
                    placed_hexes.add(key)

            connectors.append(connector)

        return connectors

    def _get_connector_path(
        self,
        start: tuple[int, int],
        end: tuple[int, int],
        length: int,
    ) -> list[tuple[int, int]]:
        """Get world coordinates for connector path."""
        from worldgen.hex_coords import get_neighbor, distance

        path = [start]
        current = start

        for _ in range(length - 1):
            if current == end:
                break

            best_neighbor = None
            best_dist = float('inf')

            for edge in range(6):
                nq, nr = get_neighbor(current[0], current[1], edge)
                d = distance(nq, nr, end[0], end[1])
                if d < best_dist:
                    best_dist = d
                    best_neighbor = (nq, nr)

            if best_neighbor and best_neighbor != current:
                current = best_neighbor
                path.append(current)
            else:
                break

        return path

    def _place_anchors(
        self, connectors: list[ConnectorCollection]
    ) -> list[MinorAnchor]:
        """Place anchors along connectors via LLM."""
        all_anchors = []

        for connector in connectors:
            try:
                anchors = self.anchor_placer.place_anchors(connector)
                all_anchors.extend(anchors)
            except Exception as e:
                print(f"Anchor placement failed for {connector.id}: {e}")

        return all_anchors

    def _generate_fillers(self, hex_map: HexMap, world_radius: int) -> HexMap:
        """Generate filler hexes."""
        return self.filler_generator.generate_fillers(hex_map, world_radius)
