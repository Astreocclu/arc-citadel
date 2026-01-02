# Species TOML Schema

> Complete schema for defining new species via the code generator.

## Overview

Species are defined in TOML files located in `species/`. The generator reads these files and produces:
1. Entity archetype with species-specific values (`src/entity/species/{module_name}.rs`)
2. Polity behavior module (`src/aggregate/species/{module_name}.rs`)
3. Patches to integrate the species across the codebase

## Required Sections

### [metadata]

```toml
[metadata]
name = "Orc"           # PascalCase, used in Rust types (Species::Orc)
module_name = "orc"    # snake_case, used in file names and function names
```

### [entity_values]

Entity-layer values that drive individual behavior. Each value is 0.0-1.0.

```toml
[entity_values]
rage = { type = "f32", default = 0.5, description = "Combat fury level" }
strength = { type = "f32", default = 0.5, description = "Physical power" }
```

Or simplified form (type defaults to f32, default to 0.5):
```toml
[entity_values]
rage = 0.5
strength = 0.5
```

### [polity_state]

Polity-layer state fields for aggregate simulation.

```toml
[polity_state]
waaagh_level = { type = "f32", default = "0.0" }
raid_targets = { type = "Vec<u32>", default = "Vec::new()" }
```

### [terrain_fitness]

Fitness values (0.0-1.0) for each terrain type. Keys must match `region.rs` Terrain enum:

```toml
[terrain_fitness]
mountain = 0.6
hills = 0.8
forest = 0.5
plains = 0.7
marsh = 0.6
coast = 0.3
desert = 0.5
river = 0.4
```

### [growth]

Population growth parameters.

```toml
[growth]
rate = 1.02  # Annual growth multiplier (1.02 = 2% growth)
```

### [expansion]

Territory expansion parameters.

```toml
[expansion]
threshold = 0.2  # Minimum population density before expansion
```

### [naming]

Name generation for polities.

```toml
[naming]
prefixes = ["Grak", "Thok", "Zug"]
suffixes = ["gash", "gore", "skull"]
```

### [polity_types]

Species-specific polity type names by size.

```toml
[polity_types]
small = "Warband"
medium = "Tribe"
large = "Horde"
```

## Optional Sections

### [[action_rules]]

Entity-layer action selection rules. Each rule triggers an action when a value exceeds a threshold.

```toml
[[action_rules]]
trigger_value = "rage"       # Entity value field to check
threshold = 0.7              # Trigger when value > threshold
action = "Defend"            # ActionId variant
priority = "High"            # TaskPriority: Critical, High, Normal, Low
requires_target = true       # Whether action needs entity_nearby
description = "High rage triggers aggressive action"
```

### [[idle_behaviors]]

Idle action priorities, checked in order.

```toml
[[idle_behaviors]]
value = "combat_prowess"     # Entity value field to check
threshold = 0.7              # Trigger when value > threshold
action = "IdleWander"        # ActionId variant
requires_target = false      # Whether action needs entity_nearby
description = "Patrol for combat opportunities"
```

### [[behavior_rules]]

Polity-layer behavior rules that generate events.

```toml
[[behavior_rules]]
state_field = "waaagh_level" # Polity state field to check
threshold = 0.8              # Trigger when field > threshold
event_type = "WarDeclared"   # EventType variant to emit
description = "High WAAAGH level triggers war"
```

### [[state_update_rules]]

Rules for updating polity state each tick.

```toml
[[state_update_rules]]
field = "waaagh_level"       # State field to update
delta = 0.01                 # Amount to change per tick
condition = "world.at_war"   # Optional Rust condition (raw)
description = "WAAAGH builds during war"
```

## Complete Example

See `species/orc.toml` for a complete working example.

## Generator Usage

```bash
# Generate all files for a species (dry run)
python tools/species_gen/cli.py generate species/orc.toml --dry-run

# Generate and write files
python tools/species_gen/cli.py generate species/orc.toml

# Apply patches to integrate species
python tools/species_gen/cli.py patch species/orc.toml

# Full generation (files + patches)
python tools/species_gen/cli.py full species/orc.toml
```

## Integration Points

The generator patches these files:
- `src/core/types.rs` - Species enum variant
- `src/aggregate/polity.rs` - SpeciesState enum, state struct, accessors
- `src/aggregate/species/mod.rs` - Module declaration, tick dispatch
- `src/aggregate/systems/generation.rs` - State generation, name generation
- `src/aggregate/systems/population.rs` - Growth rate
- `src/aggregate/region.rs` - Terrain fitness
- `src/entity/species/mod.rs` - Entity module declaration
- `src/simulation/action_select.rs` - Action selection (if action_rules defined)

Each integration point uses CODEGEN markers:
```rust
// CODEGEN: marker_name
```

The patcher inserts species-specific code before each marker.
