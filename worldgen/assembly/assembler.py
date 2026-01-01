"""Main world assembly pipeline."""

import random

from worldgen.schemas import WorldSeed, HexMap, AssembledCluster
from worldgen.storage import Database
from .layout_solver import LayoutSolver


class WorldAssembler:
    """Assemble a complete world from a seed."""

    def __init__(self, database: Database):
        self.database = database
        self.layout_solver = LayoutSolver()

    def assemble(self, seed: WorldSeed) -> HexMap:
        """Assemble a complete world from a seed."""
        rng = random.Random(seed.seed_id)

        # 1. Create assembled clusters (stub - just empty footprints for now)
        assembled_clusters: dict[str, AssembledCluster] = {}
        for placement in seed.clusters:
            assembled = AssembledCluster(
                template_id=placement.template_id,
                instance_id=placement.instance_id,
                components={},
                layout={},
                footprint=[(0, 0), (1, 0), (0, 1)],  # Minimal footprint
            )
            assembled_clusters[placement.instance_id] = assembled

        # 2. Solve global layout
        cluster_positions = self.layout_solver.solve(
            clusters=assembled_clusters,
            hints=seed.layout_hints,
            world_radius=seed.world_radius,
            rng=rng,
        )

        return HexMap(
            seed_id=seed.seed_id,
            world_radius=seed.world_radius,
            hexes={},
            clusters=assembled_clusters,
            cluster_positions=cluster_positions,
        )
