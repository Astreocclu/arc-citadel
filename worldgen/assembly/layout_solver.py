"""Solve cluster placement on hex grid."""

import random
from typing import Optional

from worldgen.schemas import AssembledCluster, LayoutHint


class LayoutSolver:
    """Solve cluster placement via constraint satisfaction."""

    def __init__(self):
        self.max_attempts = 1000

    def solve(
        self,
        clusters: dict[str, AssembledCluster],
        hints: list[LayoutHint],
        world_radius: int,
        rng: random.Random,
    ) -> dict[str, tuple[int, int]]:
        """Find valid positions for all clusters."""
        positions = {}
        placed_footprints: set[tuple[int, int]] = set()

        for instance_id, cluster in clusters.items():
            hint = next((h for h in hints if h.cluster_id == instance_id), None)

            position = self._find_valid_position(
                cluster=cluster,
                hint=hint,
                placed=placed_footprints,
                world_radius=world_radius,
                rng=rng,
            )

            if position is None:
                raise ValueError(f"Could not place cluster {instance_id}")

            positions[instance_id] = position

            # Add footprint to placed set
            for offset in cluster.footprint:
                placed_footprints.add((position[0] + offset[0], position[1] + offset[1]))

        return positions

    def _find_valid_position(
        self,
        cluster: AssembledCluster,
        hint: Optional[LayoutHint],
        placed: set[tuple[int, int]],
        world_radius: int,
        rng: random.Random,
    ) -> Optional[tuple[int, int]]:
        """Find a valid position for a cluster."""
        for _ in range(self.max_attempts):
            q = rng.randint(-world_radius, world_radius)
            r = rng.randint(-world_radius, world_radius)
            if abs(q + r) <= world_radius:
                if self._is_valid_position(cluster, (q, r), placed, world_radius):
                    return (q, r)
        return None

    def _is_valid_position(
        self,
        cluster: AssembledCluster,
        position: tuple[int, int],
        placed: set[tuple[int, int]],
        world_radius: int,
    ) -> bool:
        """Check if a position is valid for a cluster."""
        for offset in cluster.footprint:
            world_q = position[0] + offset[0]
            world_r = position[1] + offset[1]

            if abs(world_q) + abs(world_r) > world_radius:
                return False

            if (world_q, world_r) in placed:
                return False

        return True
