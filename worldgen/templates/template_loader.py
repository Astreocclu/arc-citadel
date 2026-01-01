"""Load and validate cluster templates from YAML."""

from pathlib import Path
from typing import Optional

import yaml

from worldgen import config
from worldgen.schemas import ClusterTemplate


class TemplateLoader:
    """Load cluster templates from YAML files."""

    def __init__(self, templates_dir: Optional[Path] = None):
        self.templates_dir = templates_dir or config.TEMPLATES_DIR

    def load_template(self, template_path: str) -> ClusterTemplate:
        """Load a single template by path (e.g., 'dwarf/hold_major')."""
        yaml_path = self.templates_dir / f"{template_path}.yaml"
        if not yaml_path.exists():
            raise FileNotFoundError(f"Template not found: {yaml_path}")

        with open(yaml_path) as f:
            data = yaml.safe_load(f)

        return ClusterTemplate.model_validate(data)

    def load_all(self) -> dict[str, ClusterTemplate]:
        """Load all templates from the templates directory."""
        templates = {}

        for yaml_file in self.templates_dir.rglob("*.yaml"):
            try:
                with open(yaml_file) as f:
                    data = yaml.safe_load(f)

                template = ClusterTemplate.model_validate(data)
                templates[template.id] = template
            except Exception as e:
                print(f"Failed to load {yaml_file}: {e}")

        return templates

    def validate_template(
        self, template: ClusterTemplate, component_categories: set[str]
    ) -> list[str]:
        """Validate a template against available component categories."""
        errors = []

        for slot in template.slots:
            if slot.component_category.value not in component_categories:
                errors.append(
                    f"Unknown component category: {slot.component_category.value}"
                )

        slot_ids = {s.slot_id for s in template.slots}
        for conn in template.internal_connections:
            if conn.from_slot not in slot_ids:
                errors.append(f"Unknown slot in connection: {conn.from_slot}")
            if conn.to_slot not in slot_ids:
                errors.append(f"Unknown slot in connection: {conn.to_slot}")

        return errors
