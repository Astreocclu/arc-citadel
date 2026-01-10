"""Connector generation with elastic pathfinding and full schema support."""

import random
from dataclasses import dataclass, field
from typing import Optional

from worldgen.schemas.connector import (
    ConnectorCollection, ConnectorType, ConnectorHex,
    EntryPoint, MinorAnchorSlot, ElasticSegment
)
from worldgen.schemas.base import Terrain, SpeciesFitness
from worldgen.hex_coords import get_neighbor, distance


@dataclass
class PathConstraint:
    """Constraints for pathfinding."""
    start: tuple[int, int]
    end: tuple[int, int]
    avoid_positions: set[tuple[int, int]] = field(default_factory=set)
    prefer_terrain: list[str] = field(default_factory=list)
    max_length: int = 100


class ConnectorGenerator:
    """Generates connectors between clusters with full schema support."""

    def __init__(
        self,
        elasticity: float = 0.3,
        max_deviation: int = 3,
    ):
        self.elasticity = elasticity
        self.max_deviation = max_deviation

    def generate(
        self,
        connector_type: ConnectorType,
        start_pos: tuple[int, int],
        end_pos: tuple[int, int],
        placed_hexes: set[tuple[int, int]],
        elastic_segments: Optional[list[ElasticSegment]] = None,
    ) -> ConnectorCollection:
        """Generate a connector between two points.

        Args:
            connector_type: Type of connector (road, river, etc.)
            start_pos: Starting hex coordinates (q, r)
            end_pos: Ending hex coordinates (q, r)
            placed_hexes: Set of already-occupied hex positions to avoid
            elastic_segments: Optional elastic segment definitions

        Returns:
            ConnectorCollection with hexes, graph, and anchor slots
        """
        # 1. Find path using A* with elasticity
        constraint = PathConstraint(
            start=start_pos,
            end=end_pos,
            avoid_positions=placed_hexes,
            prefer_terrain=self._get_preferred_terrain(connector_type),
        )
        path_coords = self._find_elastic_path(constraint)

        # 2. Generate ConnectorHex for each position
        connector_hexes = self._generate_connector_hexes(path_coords, connector_type)

        # 3. Apply elastic segment adjustments if provided
        if elastic_segments:
            connector_hexes = self._apply_elastic_segments(connector_hexes, elastic_segments)

        # 4. Build internal graph
        internal_graph = self._build_internal_graph(connector_hexes)

        # 5. Generate anchor slots
        anchor_slots = self._generate_anchor_slots(connector_hexes, connector_type)

        # 6. Create entry points
        entry_points = self._create_entry_points(connector_hexes, connector_type)

        # 7. Assemble collection
        collection = ConnectorCollection(
            id=f"conn_{connector_type.value}_{start_pos[0]}_{start_pos[1]}",
            type=connector_type,
            hexes=connector_hexes,
            base_length=len(connector_hexes),
            entry_points=entry_points,
            internal_graph=internal_graph,
            elastic_segments=elastic_segments or [],
            minor_slots=anchor_slots,
        )

        return collection

    def _find_elastic_path(self, constraint: PathConstraint) -> list[tuple[int, int]]:
        """A* pathfinding with elasticity allowing scenic detours."""
        start = constraint.start
        end = constraint.end

        # Priority queue: (f_score, g_score, position, path)
        open_set = [(0, 0, start, [start])]
        visited = set()

        while open_set:
            open_set.sort(key=lambda x: x[0])
            f, g, current, path = open_set.pop(0)

            if current == end:
                return path

            if current in visited:
                continue
            visited.add(current)

            # Explore neighbors
            for edge in range(6):
                nq, nr = get_neighbor(current[0], current[1], edge)
                neighbor = (nq, nr)

                if neighbor in visited:
                    continue
                if neighbor in constraint.avoid_positions:
                    continue

                # Cost calculation with elasticity
                new_g = g + 1
                h = distance(nq, nr, end[0], end[1])

                # Elasticity bonus: allow slightly longer paths for variety
                elasticity_bonus = random.random() * self.elasticity
                new_f = new_g + h - elasticity_bonus

                # Check deviation from straight line
                straight_dist = distance(start[0], start[1], end[0], end[1])
                current_deviation = (new_g + h) - straight_dist
                if current_deviation > self.max_deviation:
                    continue

                open_set.append((new_f, new_g, neighbor, path + [neighbor]))

        # Fallback: straight line if A* fails
        return self._straight_line_path(start, end)

    def _straight_line_path(
        self, start: tuple[int, int], end: tuple[int, int]
    ) -> list[tuple[int, int]]:
        """Generate straight-line hex path as fallback."""
        path = [start]
        current = start

        while current != end:
            best_neighbor = None
            best_dist = float('inf')

            for edge in range(6):
                nq, nr = get_neighbor(current[0], current[1], edge)
                d = distance(nq, nr, end[0], end[1])
                if d < best_dist:
                    best_dist = d
                    best_neighbor = (nq, nr)

            if best_neighbor is None or best_neighbor == current:
                break

            current = best_neighbor
            path.append(current)

        return path

    def _generate_connector_hexes(
        self,
        path: list[tuple[int, int]],
        connector_type: ConnectorType,
    ) -> list[ConnectorHex]:
        """Generate ConnectorHex objects for path coordinates."""
        hexes = []
        terrain = self._get_base_terrain(connector_type)

        for i, (q, r) in enumerate(path):
            # Determine connectivity
            connects_to = []
            if i > 0:
                connects_to.append(i - 1)
            if i < len(path) - 1:
                connects_to.append(i + 1)

            chex = ConnectorHex(
                index=i,
                terrain=terrain,
                elevation=self._sample_elevation(connector_type, i, len(path)),
                moisture=self._sample_moisture(connector_type),
                connects_to=connects_to,
                is_entry_point=(i == 0 or i == len(path) - 1),
                species_fitness=SpeciesFitness(human=0.5, dwarf=0.5, elf=0.5),
            )
            hexes.append(chex)

        return hexes

    def _apply_elastic_segments(
        self,
        hexes: list[ConnectorHex],
        segments: list[ElasticSegment],
    ) -> list[ConnectorHex]:
        """Apply elastic segment stretch/compress."""
        result = list(hexes)

        for segment in segments:
            start_idx = segment.start_hex
            end_idx = segment.end_hex

            if start_idx >= len(result) or end_idx >= len(result):
                continue

            current_length = end_idx - start_idx + 1

            if current_length < segment.min_length:
                # Stretch: insert hexes with stretch_terrain
                stretch_count = segment.min_length - current_length
                insert_idx = start_idx + current_length // 2

                for j in range(stretch_count):
                    stretch_hex = ConnectorHex(
                        index=insert_idx + j,
                        terrain=segment.stretch_terrain,
                        elevation=result[start_idx].elevation,
                        moisture=result[start_idx].moisture,
                        connects_to=[],
                        species_fitness=SpeciesFitness(human=0.5, dwarf=0.5, elf=0.5),
                    )
                    result.insert(insert_idx + j, stretch_hex)

            elif current_length > segment.max_length:
                # Compress: remove middle hexes
                remove_count = current_length - segment.max_length
                mid_idx = start_idx + current_length // 2

                for _ in range(remove_count):
                    if mid_idx < len(result):
                        result.pop(mid_idx)

        # Reindex after modifications
        for i, chex in enumerate(result):
            chex.index = i
            chex.connects_to = []
            if i > 0:
                chex.connects_to.append(i - 1)
            if i < len(result) - 1:
                chex.connects_to.append(i + 1)

        return result

    def _build_internal_graph(
        self, hexes: list[ConnectorHex]
    ) -> dict[int, list[int]]:
        """Build hex-to-hex connectivity graph."""
        graph = {}
        for chex in hexes:
            graph[chex.index] = chex.connects_to.copy()
        return graph

    def _generate_anchor_slots(
        self,
        hexes: list[ConnectorHex],
        connector_type: ConnectorType,
    ) -> list[MinorAnchorSlot]:
        """Generate anchor slots based on connector type and features."""
        slots = []

        if not hexes:
            return slots

        # Endpoints always get slots
        slots.append(MinorAnchorSlot(
            slot_id="slot_start",
            hex_index=0,
            compatible_categories=["waystation", "toll_gate", "border_post"],
            required=False,
            narrative_context="Connector entrance",
        ))

        slots.append(MinorAnchorSlot(
            slot_id="slot_end",
            hex_index=len(hexes) - 1,
            compatible_categories=["waystation", "toll_gate", "border_post"],
            required=False,
            narrative_context="Connector terminus",
        ))

        # Interval-based slots
        interval = self._get_anchor_interval(connector_type)
        for i in range(interval, len(hexes) - 1, interval):
            compatible = self._get_compatible_anchors(connector_type, hexes[i])
            slots.append(MinorAnchorSlot(
                slot_id=f"slot_{i}",
                hex_index=i,
                compatible_categories=compatible,
                required=False,
                narrative_context="Regular stopping point",
            ))

        # Feature-based slots (river crossings, elevation changes)
        for i, chex in enumerate(hexes):
            if i == 0 or i == len(hexes) - 1:
                continue

            # River crossing detection
            if chex.terrain == Terrain.SHALLOW_WATER:
                slots.append(MinorAnchorSlot(
                    slot_id=f"slot_crossing_{i}",
                    hex_index=i,
                    compatible_categories=["bridge_wood", "bridge_stone", "ford_improved", "ferry"],
                    required=True,
                    narrative_context="River crossing",
                ))

            # Elevation change detection
            if i > 0 and abs(chex.elevation - hexes[i-1].elevation) > 100:
                slots.append(MinorAnchorSlot(
                    slot_id=f"slot_pass_{i}",
                    hex_index=i,
                    compatible_categories=["watchtower", "shrine", "camp"],
                    required=False,
                    narrative_context="Mountain pass or significant elevation",
                ))

        return slots

    def _create_entry_points(
        self,
        hexes: list[ConnectorHex],
        connector_type: ConnectorType,
    ) -> list[EntryPoint]:
        """Create entry points at connector ends."""
        if not hexes:
            return []

        return [
            EntryPoint(
                hex_index=0,
                direction="start",
                elevation=hexes[0].elevation,
                terrain=hexes[0].terrain,
                is_terminus=True,
            ),
            EntryPoint(
                hex_index=len(hexes) - 1,
                direction="end",
                elevation=hexes[-1].elevation,
                terrain=hexes[-1].terrain,
                is_terminus=True,
            ),
        ]

    def _get_preferred_terrain(self, connector_type: ConnectorType) -> list[str]:
        """Get preferred terrain for pathfinding."""
        preferences = {
            ConnectorType.TRADE_ROUTE_MAJOR: ["plains", "grassland"],
            ConnectorType.TRADE_ROUTE_MINOR: ["plains", "forest"],
            ConnectorType.MILITARY_ROAD: ["plains", "hills"],
            ConnectorType.RIVER_MIDDLE: ["plains", "forest"],
            ConnectorType.MOUNTAIN_PASS: ["mountains", "hills"],
        }
        return preferences.get(connector_type, ["plains"])

    def _get_base_terrain(self, connector_type: ConnectorType) -> Terrain:
        """Get base terrain for connector type."""
        terrain_map = {
            ConnectorType.RIVER_HEADWATERS: Terrain.SHALLOW_WATER,
            ConnectorType.RIVER_UPPER: Terrain.SHALLOW_WATER,
            ConnectorType.RIVER_MIDDLE: Terrain.SHALLOW_WATER,
            ConnectorType.RIVER_LOWER: Terrain.DEEP_WATER,
            ConnectorType.TRADE_ROUTE_MAJOR: Terrain.PLAINS,
            ConnectorType.TRADE_ROUTE_MINOR: Terrain.PLAINS,
            ConnectorType.MILITARY_ROAD: Terrain.PLAINS,
            ConnectorType.MOUNTAIN_PASS: Terrain.MOUNTAINS,
        }
        return terrain_map.get(connector_type, Terrain.PLAINS)

    def _sample_elevation(
        self, connector_type: ConnectorType, index: int, total: int
    ) -> float:
        """Sample elevation along connector."""
        # Rivers flow downhill
        if "river" in connector_type.value.lower():
            return 500.0 - (index / max(1, total)) * 300.0
        # Roads follow terrain
        return 200.0 + random.random() * 100.0

    def _sample_moisture(self, connector_type: ConnectorType) -> float:
        """Sample moisture for connector type."""
        if "river" in connector_type.value.lower():
            return 0.9
        return 0.4 + random.random() * 0.2

    def _get_anchor_interval(self, connector_type: ConnectorType) -> int:
        """Get anchor placement interval for connector type."""
        intervals = {
            ConnectorType.TRADE_ROUTE_MAJOR: 8,
            ConnectorType.TRADE_ROUTE_MINOR: 12,
            ConnectorType.MILITARY_ROAD: 10,
            ConnectorType.PILGRIM_PATH: 6,
        }
        return intervals.get(connector_type, 10)

    def _get_compatible_anchors(
        self, connector_type: ConnectorType, chex: ConnectorHex
    ) -> list[str]:
        """Get compatible anchor categories for position."""
        base = ["camp", "shrine"]

        if connector_type in (ConnectorType.TRADE_ROUTE_MAJOR, ConnectorType.TRADE_ROUTE_MINOR):
            base.extend(["inn", "tavern", "market_small", "caravanserai"])
        elif connector_type == ConnectorType.MILITARY_ROAD:
            base.extend(["watchtower", "signal_tower", "border_post"])
        elif connector_type == ConnectorType.PILGRIM_PATH:
            base.extend(["shrine", "temple_small", "sacred_spring"])

        return base
