"""Generate valid world seeds."""

import json
import random
from pathlib import Path
from typing import Optional

from worldgen.schemas import (
    WorldSeed,
    ClusterPlacement,
    ConnectorAssignment,
    LayoutHint,
)


class SeedGenerator:
    """Generate valid world seeds from templates."""

    def __init__(self, available_templates: list[str]):
        self.available_templates = available_templates

    def generate_seed(
        self,
        seed_id: int,
        num_dwarf_holds: int = 3,
        num_elf_groves: int = 3,
        num_human_cities: int = 5,
        world_radius: int = 150,
    ) -> Optional[WorldSeed]:
        """Generate a single valid seed."""
        rng = random.Random(seed_id)

        clusters = []

        # Dwarf holds
        for i in range(num_dwarf_holds):
            template_id = "dwarf_hold_major"  # Use available template
            if template_id in self.available_templates:
                clusters.append(
                    ClusterPlacement(
                        template_id=template_id,
                        instance_id=f"dwarf_{i}",
                        region_hint=rng.choice(["N", "NE", "NW"]),
                    )
                )

        # Generate layout hints
        layout_hints = []
        for c in clusters:
            if c.template_id.startswith("dwarf_"):
                layout_hints.append(
                    LayoutHint(
                        cluster_id=c.instance_id,
                        hints=["terrain:mountains", "region:north"],
                    )
                )

        return WorldSeed(
            seed_id=seed_id,
            clusters=clusters,
            connectors=[],
            layout_hints=layout_hints,
            world_radius=world_radius,
        )

    def save_seed(self, seed: WorldSeed, output_path: Path) -> None:
        """Save a seed to a JSON file."""
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, "w") as f:
            f.write(seed.model_dump_json(indent=2))

    def load_seed(self, seed_path: Path) -> WorldSeed:
        """Load a seed from a JSON file."""
        with open(seed_path) as f:
            data = json.load(f)
        return WorldSeed.model_validate(data)
