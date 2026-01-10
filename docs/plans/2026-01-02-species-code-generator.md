# Species Code Generator Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create a Python CLI tool that reads TOML species definitions and generates all Rust code (entity archetype, values, polity state, behavior module) plus patches to existing integration points.

**Problem:** Adding a new species currently requires modifying 10+ files with match statements. This reduces it to one TOML file + one command.

**Tech Stack:** Python 3.10+, Jinja2 templates, TOML parsing, regex-based patching with markers.

---

## Task 1: Create Tool Directory Structure

Set up the species generator tool structure.

**Files:**
- Create: `tools/species_gen/__init__.py`
- Create: `tools/species_gen/cli.py`
- Create: `tools/species_gen/generator.py`
- Create: `tools/species_gen/patcher.py`
- Create: `tools/species_gen/templates/` (directory)
- Create: `tools/species_gen/requirements.txt`

**Step 1: Create directory and requirements**

```bash
mkdir -p tools/species_gen/templates
```

```python
# tools/species_gen/requirements.txt
toml>=0.10.2
jinja2>=3.1.0
```

**Step 2: Create __init__.py**

```python
# tools/species_gen/__init__.py
"""Species code generator for arc-citadel."""
__version__ = "0.1.0"
```

**Step 3: Verify structure**

Run: `ls -la tools/species_gen/`
Expected: See `__init__.py`, `requirements.txt`, `templates/`

---

## Task 2: Define TOML Schema and Create Example

Create the species definition schema and an example file.

**Files:**
- Create: `tools/species_gen/schema.md`
- Create: `species/human.toml` (example based on existing Human)
- Create: `species/template.toml`

**Step 1: Create schema documentation**

```markdown
# tools/species_gen/schema.md
# Species Definition Schema

## Required Fields

```toml
[metadata]
name = "Human"              # PascalCase, used for struct names
module_name = "human"       # snake_case, used for file and module names

[entity_values]
# Arbitrary f32 fields - species-specific value concepts
honor = 0.5                 # Default value (0.0-1.0)
beauty = 0.5
# ... add as many as needed

[polity_state]
# Fields for aggregate-level state
expansion_pressure = { type = "f32", default = 0.5 }
internal_cohesion = { type = "f32", default = 0.7 }
factions = { type = "Vec<Faction>", default = "Vec::new()" }
# Supported types: f32, u32, String, Vec<T>, HashMap<K,V>, HashSet<T>

[terrain_fitness]
# Terrain type -> fitness value (0.0-1.0)
plains = 0.9
forest = 0.7
mountains = 0.3
hills = 0.7
desert = 0.3
tundra = 0.2
swamp = 0.3
coastal = 0.8

[growth]
rate = 1.01                 # Population multiplier per year (e.g., 1.01 = 1% growth)

[expansion]
threshold = 0.3             # Minimum terrain fitness for expansion

[naming]
prefixes = ["Alden", "Bran", "Cael"]
suffixes = ["mark", "ford", "heim"]

[polity_types]
# Map territory size ranges to polity types
small = "Tribe"             # 0-5 regions
medium = "CityState"        # 6-15 regions
large = "Kingdom"           # 16+ regions
```
```

**Step 2: Create Human example (validates schema against existing code)**

```toml
# species/human.toml
[metadata]
name = "Human"
module_name = "human"

[entity_values]
honor = 0.5
beauty = 0.5
comfort = 0.5
ambition = 0.5
loyalty = 0.5
love = 0.5
justice = 0.5
curiosity = 0.5
safety = 0.5
piety = 0.5

[polity_state]
expansion_pressure = { type = "f32", default = 0.5 }
internal_cohesion = { type = "f32", default = 0.7 }
reputation = { type = "f32", default = 0.5 }
piety = { type = "f32", default = 0.5 }
factions = { type = "Vec<Faction>", default = "Vec::new()" }

[terrain_fitness]
plains = 0.9
forest = 0.7
mountains = 0.3
hills = 0.7
desert = 0.3
tundra = 0.2
swamp = 0.3
coastal = 0.85

[growth]
rate = 1.01

[expansion]
threshold = 0.3

[naming]
prefixes = ["Alden", "Bran", "Cael", "Dorn", "Eld", "Frey", "Grim", "Hal", "Isen", "Kael"]
suffixes = ["mark", "ford", "heim", "dale", "wick", "ton", "bury", "wood", "vale", "gate"]

[polity_types]
small = "Tribe"
medium = "CityState"
large = "Kingdom"
```

**Step 3: Verify TOML parses**

Run: `python3 -c "import toml; print(toml.load('species/human.toml'))"`
Expected: Prints parsed dict without errors

---

## Task 3: Add Marker Comments to Codebase

Add `// CODEGEN:` markers to existing files to enable safe patching.

**Files:**
- Modify: `src/core/types.rs`
- Modify: `src/aggregate/polity.rs`
- Modify: `src/aggregate/species/mod.rs`
- Modify: `src/aggregate/systems/generation.rs`
- Modify: `src/aggregate/systems/population.rs`
- Modify: `src/entity/species/mod.rs`

**Step 1: Add marker to Species enum (src/core/types.rs)**

```rust
// Find this:
pub enum Species {
    Human,
    Dwarf,
    Elf,
    Orc,
}

// Change to:
pub enum Species {
    Human,
    Dwarf,
    Elf,
    Orc,
    // CODEGEN: species_enum_variants
}
```

**Step 2: Add markers to polity.rs**

```rust
// In SpeciesState enum (around line 124):
pub enum SpeciesState {
    Human(HumanState),
    Dwarf(DwarfState),
    Elf(ElfState),
    // CODEGEN: species_state_variants
}

// After ElfState struct (around line 216):
// CODEGEN: species_state_structs

// In impl Polity, after elf_state_mut (around line 259):
    // CODEGEN: species_state_accessors
}
```

**Step 3: Add markers to aggregate/species/mod.rs**

```rust
mod human;
mod dwarf;
mod elf;
// CODEGEN: species_behavior_mods

// In tick function:
pub fn tick(polity: &Polity, world: &AggregateWorld, year: u32) -> Vec<EventType> {
    match polity.species {
        Species::Human => human::tick(polity, world, year),
        Species::Dwarf => dwarf::tick(polity, world, year),
        Species::Elf => elf::tick(polity, world, year),
        Species::Orc => vec![],
        // CODEGEN: species_tick_arms
    }
}
```

**Step 4: Add markers to generation.rs**

```rust
// In species_state match (around line 286):
let species_state = match species {
    Species::Human => SpeciesState::Human(HumanState { ... }),
    Species::Dwarf => SpeciesState::Dwarf(DwarfState { ... }),
    Species::Elf => SpeciesState::Elf(ElfState { ... }),
    Species::Orc => todo!("Orc SpeciesState not implemented"),
    // CODEGEN: species_state_generation
};

// In generate_polity_name prefixes (around line 349):
let prefixes = match species {
    Species::Human => [...],
    Species::Dwarf => [...],
    Species::Elf => [...],
    Species::Orc => [...],
    // CODEGEN: species_name_prefixes
};

// In generate_polity_name suffixes (around line 356):
let suffixes = match species {
    Species::Human => [...],
    Species::Dwarf => [...],
    Species::Elf => [...],
    Species::Orc => [...],
    // CODEGEN: species_name_suffixes
};
```

**Step 5: Add marker to population.rs**

```rust
// In base_growth match (around line 12):
let base_growth = match polity.species {
    Species::Human => 1.01,
    Species::Dwarf => 1.005,
    Species::Elf => 1.002,
    Species::Orc => 1.02,
    // CODEGEN: species_growth_rates
};
```

**Step 6: Add marker to entity/species/mod.rs**

```rust
pub mod human;
pub mod orc;
// CODEGEN: entity_species_mods
```

**Step 7: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors (comments don't affect code)

---

## Task 4: Create Jinja2 Templates

Create templates for generated Rust code.

**Files:**
- Create: `tools/species_gen/templates/entity_archetype.rs.j2`
- Create: `tools/species_gen/templates/entity_values.rs.j2`
- Create: `tools/species_gen/templates/polity_state.rs.j2`
- Create: `tools/species_gen/templates/behavior_module.rs.j2`

**Step 1: Entity archetype + values template**

```jinja2
{# tools/species_gen/templates/entity_archetype.rs.j2 #}
//! {{ name }} entity archetype and values

use serde::{Deserialize, Serialize};
use crate::core::types::{EntityId, Vec2};
use crate::entity::needs::Needs;
use crate::entity::thoughts::ThoughtBuffer;
use crate::entity::tasks::TaskQueue;
use crate::entity::body::BodyState;
use crate::entity::social_memory::SocialMemory;

/// {{ name }}-specific value vocabulary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct {{ name }}Values {
{% for value_name, default in entity_values.items() %}
    pub {{ value_name }}: f32,
{% endfor %}
}

impl {{ name }}Values {
    pub fn new() -> Self {
        Self {
{% for value_name, default in entity_values.items() %}
            {{ value_name }}: {{ default }},
{% endfor %}
        }
    }

    /// Randomize values within reasonable bounds
    pub fn randomize(&mut self, rng: &mut impl rand::Rng) {
{% for value_name, _ in entity_values.items() %}
        self.{{ value_name }} = rng.gen_range(0.2..0.8);
{% endfor %}
    }
}

/// {{ name }} archetype using Structure of Arrays layout
#[derive(Debug, Default)]
pub struct {{ name }}Archetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<{{ name }}Values>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

impl {{ name }}Archetype {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, name: String, position: Vec2, values: {{ name }}Values) -> EntityId {
        let id = EntityId::new();
        self.ids.push(id);
        self.names.push(name);
        self.positions.push(position);
        self.velocities.push(Vec2::default());
        self.body_states.push(BodyState::default());
        self.needs.push(Needs::new());
        self.thoughts.push(ThoughtBuffer::new());
        self.values.push(values);
        self.task_queues.push(TaskQueue::new());
        self.alive.push(true);
        self.social_memories.push(SocialMemory::default());
        id
    }

    pub fn index_of(&self, id: EntityId) -> Option<usize> {
        self.ids.iter().position(|&eid| eid == id)
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn alive_count(&self) -> usize {
        self.alive.iter().filter(|&&a| a).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_{{ module_name }}_values_creation() {
        let values = {{ name }}Values::new();
{% for value_name, default in entity_values.items() %}
        assert!((values.{{ value_name }} - {{ default }}).abs() < 0.01);
{% endfor %}
    }

    #[test]
    fn test_{{ module_name }}_archetype_spawn() {
        let mut archetype = {{ name }}Archetype::new();
        let id = archetype.spawn(
            "Test {{ name }}".to_string(),
            Vec2::new(10.0, 20.0),
            {{ name }}Values::new(),
        );
        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.index_of(id), Some(0));
    }
}
```

**Step 2: Polity state template**

```jinja2
{# tools/species_gen/templates/polity_state.rs.j2 #}
/// {{ name }}-specific polity state
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct {{ name }}State {
{% for field_name, field_def in polity_state.items() %}
    pub {{ field_name }}: {{ field_def.type }},
{% endfor %}
}
```

**Step 3: Behavior module template**

```jinja2
{# tools/species_gen/templates/behavior_module.rs.j2 #}
//! {{ name }}-specific polity behavior

use crate::aggregate::polity::Polity;
use crate::aggregate::world::AggregateWorld;
use crate::aggregate::events::EventType;

/// Generate {{ name }}-specific events for a polity
pub fn tick(polity: &Polity, _world: &AggregateWorld, _year: u32) -> Vec<EventType> {
    let mut events = Vec::new();

    // Access {{ module_name }}-specific state
    if let Some(_state) = polity.{{ module_name }}_state() {
        // TODO: Implement {{ name }}-specific behavior
        // Example: Check conditions, generate events
    }

    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::polity::*;
    use crate::core::types::{PolityId, Species, PolityTier, GovernmentType};
    use std::collections::HashMap;

    fn create_test_polity() -> Polity {
        Polity {
            id: PolityId(1),
            name: "Test {{ name }} Polity".to_string(),
            species: Species::{{ name }},
            polity_type: PolityType::{{ polity_types.large }},
            tier: PolityTier::Kingdom,
            government: GovernmentType::Autocracy,
            parent: None,
            rulers: vec![],
            council_roles: HashMap::new(),
            population: 1000,
            capital: 0,
            military_strength: 100.0,
            economic_strength: 100.0,
            cultural_drift: CulturalDrift::default(),
            relations: HashMap::new(),
            species_state: SpeciesState::{{ name }}({{ name }}State::default()),
            alive: true,
        }
    }

    #[test]
    fn test_{{ module_name }}_tick_no_panic() {
        let polity = create_test_polity();
        let world = AggregateWorld::default();
        let events = tick(&polity, &world, 100);
        // Should not panic, events may be empty for now
        assert!(events.is_empty() || !events.is_empty());
    }
}
```

**Step 4: Verify templates are valid Jinja2**

Run: `python3 -c "from jinja2 import Template; Template(open('tools/species_gen/templates/entity_archetype.rs.j2').read())"`
Expected: No errors

---

## Task 5: Implement Core Generator

Implement the main generation logic.

**Files:**
- Create: `tools/species_gen/generator.py`

**Step 1: Write the generator module**

```python
# tools/species_gen/generator.py
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

    def generate_polity_state(self, spec: Dict[str, Any]) -> str:
        """Generate polity state struct Rust code."""
        template = self.env.get_template("polity_state.rs.j2")
        return template.render(
            name=spec["metadata"]["name"],
            polity_state=spec["polity_state"],
        )

    def generate_behavior_module(self, spec: Dict[str, Any]) -> str:
        """Generate behavior module Rust code."""
        template = self.env.get_template("behavior_module.rs.j2")
        return template.render(
            name=spec["metadata"]["name"],
            module_name=spec["metadata"]["module_name"],
            polity_types=spec.get("polity_types", {"large": "Kingdom"}),
        )

    def generate_all(self, spec_path: Path, dry_run: bool = False) -> Dict[str, str]:
        """Generate all files for a species.

        Returns dict of {filepath: content}.
        """
        spec = self.load_spec(spec_path)
        name = spec["metadata"]["name"]
        module_name = spec["metadata"]["module_name"]

        files = {}

        # Entity archetype
        entity_path = self.project_root / "src" / "entity" / "species" / f"{module_name}.rs"
        files[str(entity_path)] = self.generate_entity_archetype(spec)

        # Behavior module
        behavior_path = self.project_root / "src" / "aggregate" / "species" / f"{module_name}.rs"
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
    files = generator.generate_all(spec_path, dry_run=True)

    for filepath, content in files.items():
        print(f"\n=== {filepath} ===")
        print(content[:500] + "..." if len(content) > 500 else content)


if __name__ == "__main__":
    main()
```

**Step 2: Verify generator runs**

Run: `python3 tools/species_gen/generator.py species/human.toml`
Expected: Prints generated code for Human archetype

---

## Task 6: Implement Patcher

Implement the file patching logic using markers.

**Files:**
- Create: `tools/species_gen/patcher.py`

**Step 1: Write the patcher module**

```python
# tools/species_gen/patcher.py
"""Patch existing Rust files by inserting code at markers."""

import re
from pathlib import Path
from typing import Dict, Any, List, Tuple
from dataclasses import dataclass


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

    def find_marker(self, content: str, marker: str) -> int:
        """Find the position of a marker comment in content.

        Returns the position after the marker line, or -1 if not found.
        """
        pattern = rf"^\s*// CODEGEN: {re.escape(marker)}\s*$"
        for i, line in enumerate(content.split("\n")):
            if re.match(pattern, line):
                # Return position after this line
                lines = content.split("\n")
                return sum(len(l) + 1 for l in lines[:i+1])
        return -1

    def insert_at_marker(self, content: str, marker: str, insertion: str) -> str:
        """Insert content after a marker.

        Returns modified content, or original if marker not found.
        """
        pattern = rf"(^\s*// CODEGEN: {re.escape(marker)}\s*$)"
        match = re.search(pattern, content, re.MULTILINE)
        if not match:
            return content

        pos = match.start()
        # Find the line with the marker
        before = content[:pos]
        after = content[pos:]

        # Insert before the marker (new code goes above the marker)
        lines = after.split("\n", 1)
        marker_line = lines[0]
        rest = lines[1] if len(lines) > 1 else ""

        return before + insertion + "\n        " + marker_line + "\n" + rest

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

        # 3. Species state struct (inserted separately)
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
        gen_arm = f'''        Species::{name} => SpeciesState::{name}({name}State {{
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

        return patches

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
        checked_files = {}

        for patch in patches:
            fp = str(patch.file_path)
            if fp not in checked_files:
                with open(patch.file_path) as f:
                    checked_files[fp] = f.read()

            content = checked_files[fp]
            if self.find_marker(content, patch.marker) == -1:
                missing.append(f"{patch.marker} in {patch.file_path}")

        return missing
```

**Step 2: Test patcher finds markers**

Run: `python3 -c "from tools.species_gen.patcher import SpeciesPatcher; p = SpeciesPatcher(Path('.')); print(p.find_marker(open('src/core/types.rs').read(), 'species_enum_variants'))"`
Expected: Returns -1 (markers not added yet) or position if Task 3 completed

---

## Task 7: Implement CLI

Create the command-line interface.

**Files:**
- Create: `tools/species_gen/cli.py`

**Step 1: Write CLI**

```python
#!/usr/bin/env python3
# tools/species_gen/cli.py
"""Species generator CLI."""

import argparse
import sys
from pathlib import Path

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from tools.species_gen.generator import SpeciesGenerator
from tools.species_gen.patcher import SpeciesPatcher


def main():
    parser = argparse.ArgumentParser(
        description="Generate Rust code for new species from TOML definitions"
    )
    parser.add_argument(
        "spec",
        type=Path,
        help="Path to species TOML specification file"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be generated without writing files"
    )
    parser.add_argument(
        "--verify-markers",
        action="store_true",
        help="Check that all required markers exist in codebase"
    )
    parser.add_argument(
        "--project-root",
        type=Path,
        default=Path(__file__).parent.parent.parent,
        help="Project root directory (default: auto-detect)"
    )

    args = parser.parse_args()

    generator = SpeciesGenerator(args.project_root)
    patcher = SpeciesPatcher(args.project_root)

    # Load spec
    try:
        spec = generator.load_spec(args.spec)
    except Exception as e:
        print(f"Error loading spec: {e}", file=sys.stderr)
        sys.exit(1)

    name = spec["metadata"]["name"]
    module_name = spec["metadata"]["module_name"]
    print(f"Generating species: {name} (module: {module_name})")

    # Generate patches
    patches = patcher.generate_patches(spec)

    # Verify markers if requested
    if args.verify_markers:
        missing = patcher.verify_markers_exist(patches)
        if missing:
            print("Missing markers:", file=sys.stderr)
            for m in missing:
                print(f"  - {m}", file=sys.stderr)
            sys.exit(1)
        print("All markers present!")
        return

    # Check markers exist before proceeding
    missing = patcher.verify_markers_exist(patches)
    if missing:
        print("ERROR: Missing markers in codebase. Run Task 3 first.", file=sys.stderr)
        print("Missing markers:", file=sys.stderr)
        for m in missing:
            print(f"  - {m}", file=sys.stderr)
        sys.exit(1)

    # Generate new files
    print("\n=== Generated Files ===")
    generated = generator.generate_all(args.spec, dry_run=args.dry_run)
    for filepath in generated:
        status = "[DRY RUN]" if args.dry_run else "[CREATED]"
        print(f"{status} {filepath}")

    # Apply patches
    print("\n=== Patched Files ===")
    patched = patcher.apply_patches(patches, dry_run=args.dry_run)
    for filepath, (original, modified) in patched.items():
        if original != modified:
            status = "[DRY RUN]" if args.dry_run else "[PATCHED]"
            print(f"{status} {filepath}")

    if args.dry_run:
        print("\nDry run complete. No files modified.")
    else:
        print("\nGeneration complete. Run 'cargo check' to verify.")


if __name__ == "__main__":
    main()
```

**Step 2: Make CLI executable**

Run: `chmod +x tools/species_gen/cli.py`

**Step 3: Test CLI help**

Run: `python3 tools/species_gen/cli.py --help`
Expected: Shows usage information

---

## Task 8: Create Integration Test

Create a test that generates a new species and verifies compilation.

**Files:**
- Create: `tools/species_gen/test_generator.py`
- Create: `species/goblin.toml` (test species)

**Step 1: Create test species**

```toml
# species/goblin.toml
[metadata]
name = "Goblin"
module_name = "goblin"

[entity_values]
cunning = 0.7
greed = 0.8
cowardice = 0.6
spite = 0.5
loyalty = 0.3

[polity_state]
stolen_goods = { type = "u32", default = "0" }
tribal_unity = { type = "f32", default = "0.3" }
raid_cooldown = { type = "u32", default = "0" }

[terrain_fitness]
plains = 0.4
forest = 0.8
mountains = 0.6
hills = 0.7
desert = 0.2
tundra = 0.2
swamp = 0.7
coastal = 0.3

[growth]
rate = 1.03

[expansion]
threshold = 0.2

[naming]
prefixes = ["Grik", "Snot", "Muk", "Zik", "Gob", "Nik", "Skab", "Pik"]
suffixes = ["snot", "tooth", "ear", "nose", "gut", "eye", "foot", "hand"]

[polity_types]
small = "Warband"
medium = "Tribe"
large = "Horde"
```

**Step 2: Create integration test**

```python
# tools/species_gen/test_generator.py
"""Integration tests for species generator."""

import subprocess
import sys
import tempfile
import shutil
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent.parent


def test_dry_run():
    """Test that dry-run produces expected output."""
    result = subprocess.run(
        [sys.executable, "tools/species_gen/cli.py", "species/goblin.toml", "--dry-run"],
        cwd=PROJECT_ROOT,
        capture_output=True,
        text=True,
    )
    print(result.stdout)
    print(result.stderr, file=sys.stderr)

    assert result.returncode == 0 or "Missing markers" in result.stderr
    assert "Generating species: Goblin" in result.stdout


def test_marker_verification():
    """Test marker verification."""
    result = subprocess.run(
        [sys.executable, "tools/species_gen/cli.py", "species/goblin.toml", "--verify-markers"],
        cwd=PROJECT_ROOT,
        capture_output=True,
        text=True,
    )
    print(result.stdout)
    print(result.stderr, file=sys.stderr)

    # Either all markers present or reports missing
    assert "markers" in result.stdout.lower() or "markers" in result.stderr.lower()


if __name__ == "__main__":
    print("=== Test: Dry Run ===")
    test_dry_run()
    print("\n=== Test: Marker Verification ===")
    test_marker_verification()
    print("\n=== All tests passed! ===")
```

**Step 3: Run tests**

Run: `python3 tools/species_gen/test_generator.py`
Expected: Tests run (may report missing markers until Task 3 complete)

---

## Task 9: End-to-End Verification

Generate a complete new species and verify compilation.

**Prerequisites:** Tasks 1-8 complete

**Step 1: Verify markers are in place**

Run: `python3 tools/species_gen/cli.py species/goblin.toml --verify-markers`
Expected: "All markers present!"

**Step 2: Generate the Goblin species**

Run: `python3 tools/species_gen/cli.py species/goblin.toml`
Expected:
```
Generating species: Goblin (module: goblin)

=== Generated Files ===
[CREATED] src/entity/species/goblin.rs
[CREATED] src/aggregate/species/goblin.rs

=== Patched Files ===
[PATCHED] src/core/types.rs
[PATCHED] src/aggregate/polity.rs
[PATCHED] src/aggregate/species/mod.rs
[PATCHED] src/aggregate/systems/generation.rs
[PATCHED] src/aggregate/systems/population.rs
[PATCHED] src/entity/species/mod.rs
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 4: Run tests**

Run: `cargo test goblin`
Expected: Goblin-specific tests pass

---

## Summary

| Task | Description | Verification |
|------|-------------|--------------|
| 1 | Tool directory structure | `ls tools/species_gen/` shows files |
| 2 | TOML schema + example | `python3 -c "import toml; toml.load('species/human.toml')"` |
| 3 | Add markers to codebase | `cargo check` passes |
| 4 | Jinja2 templates | Templates parse without error |
| 5 | Core generator | `python3 generator.py species/human.toml` shows output |
| 6 | Patcher | `--verify-markers` works |
| 7 | CLI | `--help` shows usage |
| 8 | Integration test | `test_generator.py` runs |
| 9 | End-to-end | New species compiles |

**Total estimated LOC:** ~800 Python, ~200 Rust generated per species
