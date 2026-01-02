"""Species code generator - creates Rust files from TOML definitions."""

import os
from pathlib import Path
from typing import Dict, Any
import toml
from jinja2 import Environment, FileSystemLoader


class SpeciesGenerator:
    """Generates Rust code from species TOML definitions."""

    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.templates_dir = project_root / "tools" / "species_gen" / "templates"
        self.env = Environment(
            loader=FileSystemLoader(str(self.templates_dir)),
            trim_blocks=True,
            lstrip_blocks=True,
        )

    def load_spec(self, spec_path: Path) -> Dict[str, Any]:
        """Load and validate a species specification."""
        with open(spec_path) as f:
            spec = toml.load(f)

        # Validate required fields
        required = ["metadata", "entity_values", "polity_state", "terrain_fitness", "growth", "expansion", "naming"]
        for field in required:
            if field not in spec:
                raise ValueError(f"Missing required field: {field}")

        if "name" not in spec["metadata"] or "module_name" not in spec["metadata"]:
            raise ValueError("metadata must contain 'name' and 'module_name'")

        return spec

    def generate_entity_archetype(self, spec: Dict[str, Any]) -> str:
        """Generate entity archetype + values Rust code."""
        template = self.env.get_template("entity_archetype.rs.j2")
        return template.render(
            name=spec["metadata"]["name"],
            module_name=spec["metadata"]["module_name"],
            entity_values=spec["entity_values"],
        )

    def generate_behavior_module(self, spec: Dict[str, Any]) -> str:
        """Generate behavior module Rust code."""
        template = self.env.get_template("behavior_module.rs.j2")
        return template.render(
            name=spec["metadata"]["name"],
            module_name=spec["metadata"]["module_name"],
            polity_types=spec.get("polity_types", {"large": "Kingdom"}),
            behavior_rules=spec.get("behavior_rules", []),
            state_update_rules=spec.get("state_update_rules", []),
        )

    def generate_all(self, spec_path: Path, dry_run: bool = False, force: bool = False) -> Dict[str, str]:
        """Generate all files for a species.

        Args:
            spec_path: Path to the TOML spec file
            dry_run: If True, don't write files
            force: If True, overwrite existing files

        Returns dict of {filepath: content}.
        """
        spec = self.load_spec(spec_path)
        module_name = spec["metadata"]["module_name"]

        files = {}

        # Entity archetype
        entity_path = self.project_root / "src" / "entity" / "species" / f"{module_name}.rs"
        if force or not entity_path.exists():
            files[str(entity_path)] = self.generate_entity_archetype(spec)

        # Behavior module
        behavior_path = self.project_root / "src" / "aggregate" / "species" / f"{module_name}.rs"
        if force or not behavior_path.exists():
            files[str(behavior_path)] = self.generate_behavior_module(spec)

        if not dry_run:
            for filepath, content in files.items():
                path = Path(filepath)
                path.parent.mkdir(parents=True, exist_ok=True)
                with open(path, "w") as f:
                    f.write(content)

        return files


def main():
    """Test the generator."""
    import sys
    if len(sys.argv) < 2:
        print("Usage: python generator.py <species.toml>")
        sys.exit(1)

    project_root = Path(__file__).parent.parent.parent
    generator = SpeciesGenerator(project_root)

    spec_path = Path(sys.argv[1])
    dry_run = "--dry-run" in sys.argv
    force = "--force" in sys.argv or True  # Default to force for new species
    files = generator.generate_all(spec_path, dry_run=dry_run, force=force)

    for filepath, content in files.items():
        print(f"\n=== {filepath} ===")
        print(content[:500] + "..." if len(content) > 500 else content)


if __name__ == "__main__":
    main()
