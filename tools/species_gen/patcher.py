"""Patch existing Rust files by inserting code at markers."""

import re
from pathlib import Path
from typing import Dict, Any, List, Tuple
from dataclasses import dataclass
from jinja2 import Environment, FileSystemLoader


@dataclass
class Patch:
    """A patch to apply to a file."""
    marker: str
    content: str
    file_path: Path


class SpeciesPatcher:
    """Patches existing Rust files to add new species support."""

    def __init__(self, project_root: Path):
        self.project_root = project_root
        self.templates_dir = project_root / "tools" / "species_gen" / "templates"
        self.env = Environment(
            loader=FileSystemLoader(str(self.templates_dir)),
            trim_blocks=True,
            lstrip_blocks=True,
        )

    def find_marker(self, content: str, marker: str) -> int:
        """Find the line index of a marker comment in content.

        Returns the line index, or -1 if not found.
        """
        pattern = rf"^\s*// CODEGEN: {re.escape(marker)}\s*$"
        for i, line in enumerate(content.split("\n")):
            if re.match(pattern, line):
                return i
        return -1

    def get_marker_indent(self, content: str, marker: str) -> str:
        """Get the indentation of a marker line."""
        pattern = rf"^(\s*)// CODEGEN: {re.escape(marker)}\s*$"
        for line in content.split("\n"):
            match = re.match(pattern, line)
            if match:
                return match.group(1)
        return ""

    def insert_at_marker(self, content: str, marker: str, insertion: str) -> str:
        """Insert content before a marker, preserving marker's indentation.

        Returns modified content, or original if marker not found.
        """
        lines = content.split("\n")
        marker_idx = self.find_marker(content, marker)

        if marker_idx == -1:
            return content

        # Get the indentation of the marker line
        indent = self.get_marker_indent(content, marker)

        # Prepare insertion with proper indentation
        # The insertion content should already have its internal indentation
        # We just need to ensure each line of insertion aligns with marker
        insertion_lines = insertion.rstrip().split("\n")

        # Insert before the marker
        new_lines = lines[:marker_idx] + insertion_lines + lines[marker_idx:]

        return "\n".join(new_lines)

    def generate_patches(self, spec: Dict[str, Any]) -> List[Patch]:
        """Generate all patches needed for a new species."""
        name = spec["metadata"]["name"]
        module_name = spec["metadata"]["module_name"]
        patches = []

        # 1. Species enum variant
        patches.append(Patch(
            marker="species_enum_variants",
            content=f"    {name},",
            file_path=self.project_root / "src" / "core" / "types.rs"
        ))

        # 2. SpeciesState enum variant
        patches.append(Patch(
            marker="species_state_variants",
            content=f"    {name}({name}State),",
            file_path=self.project_root / "src" / "aggregate" / "polity.rs"
        ))

        # 3. Species state struct
        state_fields = []
        for field_name, field_def in spec["polity_state"].items():
            state_fields.append(f"    pub {field_name}: {field_def['type']},")
        state_struct = f'''#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct {name}State {{
{chr(10).join(state_fields)}
}}

'''
        patches.append(Patch(
            marker="species_state_structs",
            content=state_struct,
            file_path=self.project_root / "src" / "aggregate" / "polity.rs"
        ))

        # 4. State accessor methods
        accessor = f'''
    pub fn {module_name}_state(&self) -> Option<&{name}State> {{
        match &self.species_state {{
            SpeciesState::{name}(s) => Some(s),
            _ => None,
        }}
    }}

    pub fn {module_name}_state_mut(&mut self) -> Option<&mut {name}State> {{
        match &mut self.species_state {{
            SpeciesState::{name}(s) => Some(s),
            _ => None,
        }}
    }}
'''
        patches.append(Patch(
            marker="species_state_accessors",
            content=accessor,
            file_path=self.project_root / "src" / "aggregate" / "polity.rs"
        ))

        # 5. Behavior module declaration
        patches.append(Patch(
            marker="species_behavior_mods",
            content=f"mod {module_name};",
            file_path=self.project_root / "src" / "aggregate" / "species" / "mod.rs"
        ))

        # 6. Tick dispatch arm
        patches.append(Patch(
            marker="species_tick_arms",
            content=f"        Species::{name} => {module_name}::tick(polity, world, year),",
            file_path=self.project_root / "src" / "aggregate" / "species" / "mod.rs"
        ))

        # 7. Species state generation
        state_init = []
        for field_name, field_def in spec["polity_state"].items():
            default = field_def.get("default", "Default::default()")
            state_init.append(f"                {field_name}: {default},")
        gen_arm = f'''            Species::{name} => SpeciesState::{name}({name}State {{
{chr(10).join(state_init)}
            }}),'''
        patches.append(Patch(
            marker="species_state_generation",
            content=gen_arm,
            file_path=self.project_root / "src" / "aggregate" / "systems" / "generation.rs"
        ))

        # 8. Name prefixes
        prefixes = spec["naming"]["prefixes"]
        prefix_str = ", ".join(f'"{p}"' for p in prefixes)
        patches.append(Patch(
            marker="species_name_prefixes",
            content=f'        Species::{name} => [{prefix_str}],',
            file_path=self.project_root / "src" / "aggregate" / "systems" / "generation.rs"
        ))

        # 9. Name suffixes
        suffixes = spec["naming"]["suffixes"]
        suffix_str = ", ".join(f'"{s}"' for s in suffixes)
        patches.append(Patch(
            marker="species_name_suffixes",
            content=f'        Species::{name} => [{suffix_str}],',
            file_path=self.project_root / "src" / "aggregate" / "systems" / "generation.rs"
        ))

        # 10. Growth rate
        rate = spec["growth"]["rate"]
        patches.append(Patch(
            marker="species_growth_rates",
            content=f"                Species::{name} => {rate},",
            file_path=self.project_root / "src" / "aggregate" / "systems" / "population.rs"
        ))

        # 11. Entity species mod
        patches.append(Patch(
            marker="entity_species_mods",
            content=f"pub mod {module_name};",
            file_path=self.project_root / "src" / "entity" / "species" / "mod.rs"
        ))

        # 12. Terrain fitness patches
        if 'terrain_fitness' in spec:
            terrain_map = {
                'mountain': 'Mountain', 'hills': 'Hills', 'forest': 'Forest',
                'plains': 'Plains', 'marsh': 'Marsh', 'coast': 'Coast',
                'desert': 'Desert', 'river': 'River',
            }
            for terrain_key, terrain_rust in terrain_map.items():
                if terrain_key in spec['terrain_fitness']:
                    fitness = spec['terrain_fitness'][terrain_key]
                    # 16 spaces to match indentation inside match arms
                    patches.append(Patch(
                        marker=f'species_fitness_{terrain_key}',
                        content=f'                fitness.insert(Species::{name}, {fitness});',
                        file_path=self.project_root / "src" / "aggregate" / "region.rs"
                    ))

        # 13. Action selection patches (if action_rules defined)
        if 'action_rules' in spec:
            action_select_content = self._render_action_selection(spec)
            if action_select_content:
                patches.append(Patch(
                    marker='species_select_action',
                    content=action_select_content,
                    file_path=self.project_root / "src" / "simulation" / "action_select.rs"
                ))

        return patches

    def _render_action_selection(self, spec: Dict[str, Any]) -> str:
        """Render action selection code from template."""
        try:
            template = self.env.get_template("action_select.rs.j2")
            return template.render(
                name=spec["metadata"]["name"],
                module_name=spec["metadata"]["module_name"],
                action_rules=spec.get("action_rules", []),
                idle_behaviors=spec.get("idle_behaviors", []),
            )
        except Exception:
            return ""

    def apply_patches(self, patches: List[Patch], dry_run: bool = False) -> Dict[str, Tuple[str, str]]:
        """Apply patches to files.

        Returns dict of {filepath: (original, modified)}.
        """
        results = {}

        # Group patches by file
        by_file: Dict[Path, List[Patch]] = {}
        for patch in patches:
            if patch.file_path not in by_file:
                by_file[patch.file_path] = []
            by_file[patch.file_path].append(patch)

        for file_path, file_patches in by_file.items():
            with open(file_path) as f:
                original = f.read()

            modified = original
            for patch in file_patches:
                modified = self.insert_at_marker(modified, patch.marker, patch.content)

            results[str(file_path)] = (original, modified)

            if not dry_run and modified != original:
                with open(file_path, "w") as f:
                    f.write(modified)

        return results

    def verify_markers_exist(self, patches: List[Patch]) -> List[str]:
        """Check that all required markers exist in the codebase.

        Returns list of missing markers.
        """
        missing = []
        checked_files: Dict[str, str] = {}

        for patch in patches:
            fp = str(patch.file_path)
            if fp not in checked_files:
                with open(patch.file_path) as f:
                    checked_files[fp] = f.read()

            content = checked_files[fp]
            if self.find_marker(content, patch.marker) == -1:
                missing.append(f"{patch.marker} in {patch.file_path}")

        return missing
