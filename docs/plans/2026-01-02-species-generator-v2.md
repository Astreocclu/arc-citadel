# Species Generator V2: Schema Completeness for Mass Production

**Created:** 2026-01-02
**Status:** Ready for Implementation
**Confidence:** 95%

## Overview

Enhance the species code generator to handle ALL integration points, enabling true one-file + one-command species creation. This fixes the gaps identified in the Orc implementation review.

## Problem Statement

The current generator misses several integration points:
1. **Region fitness** - No marker in `region.rs`, Orcs have 0.0 fitness everywhere
2. **Entity action selection** - Hardcoded for humans in `action_select.rs`
3. **Behavior modules** - Generated as empty stubs with no configurable behavior

## Solution Architecture

### Extended TOML Schema

```toml
[metadata]
name = "Orc"
module_name = "orc"

[entity_values]
rage = 0.5
strength = 0.5
dominance = 0.5
# ... species-specific values

[polity_state]
waaagh_level = { type = "f32", default = "0.0" }
raid_targets = { type = "Vec<u32>", default = "Vec::new()" }
# ... polity-level state fields

[terrain_fitness]
plains = 0.7
hills = 0.8
mountains = 0.6
forest = 0.5
marsh = 0.6
coast = 0.3
desert = 0.5
river = 0.4

[growth]
rate = 1.02

[naming]
prefixes = ["Grak", "Thok", "Zug"]
suffixes = ["gash", "gore", "skull"]

[polity_types]
small = "Warband"
large = "Horde"

# NEW: Behavior configuration
[behavior]
aggression = 0.8
expansion_threshold = 0.5
population_for_expansion = 3000

[behavior.war_triggers]
strength_ratio_required = 1.2
grudge_threshold = 0.8

[behavior.expansion]
terrain_filter = []  # empty = all terrain
fitness_minimum = 0.3

# NEW: Entity action priorities (weights, normalized to 1.0)
[action_priorities]
fight = 0.3
gather = 0.2
rest = 0.15
socialize = 0.1
wander = 0.25
```

### Files to Modify

| File | Change | Marker |
|------|--------|--------|
| `src/aggregate/region.rs` | Add fitness for each terrain | `species_terrain_fitness` (8 markers, one per terrain) |
| `src/simulation/action_select.rs` | Add match arm + function | `species_action_select_arms`, `species_action_select_functions` |
| `tools/species_gen/patcher.py` | Add new patch generators | N/A |
| `tools/species_gen/generator.py` | Enhanced behavior generation | N/A |
| `tools/species_gen/templates/behavior_module.rs.j2` | Add behavior logic | N/A |
| `tools/species_gen/schema.md` | Document new schema sections | N/A |

### Files to Create

| File | Purpose |
|------|---------|
| `tools/species_gen/templates/action_select_function.rs.j2` | Template for entity action selection |

---

## Implementation Tasks

### Task 1: Add Region Fitness Markers

**Goal:** Enable patching terrain fitness values for each species

**File:** `src/aggregate/region.rs`

**Current code (lines 56-103):**
```rust
impl Region {
    pub fn calculate_fitness(terrain: Terrain) -> HashMap<Species, f32> {
        let mut fitness = HashMap::new();

        match terrain {
            Terrain::Mountain => {
                fitness.insert(Species::Dwarf, 1.0);
                fitness.insert(Species::Elf, 0.1);
                fitness.insert(Species::Human, 0.2);
            }
            // ... 7 more terrain types
        }
        fitness
    }
}
```

**Change:** Add marker comment after each existing species insert:

```rust
Terrain::Mountain => {
    fitness.insert(Species::Dwarf, 1.0);
    fitness.insert(Species::Elf, 0.1);
    fitness.insert(Species::Human, 0.2);
    // CODEGEN: species_fitness_mountain
}
Terrain::Hills => {
    fitness.insert(Species::Dwarf, 0.8);
    fitness.insert(Species::Elf, 0.5);
    fitness.insert(Species::Human, 0.7);
    // CODEGEN: species_fitness_hills
}
// ... same for all 8 terrain types
```

**Verification:**
```bash
grep -c "CODEGEN: species_fitness" src/aggregate/region.rs
# Expected: 8
```

---

### Task 2: Add Orc Fitness Values (Manual Fix)

**Goal:** Fix the critical bug where Orcs have 0.0 fitness everywhere

**File:** `src/aggregate/region.rs`

**Add after each marker** (before closing brace):
```rust
Terrain::Mountain => {
    // ... existing
    fitness.insert(Species::Orc, 0.6);
    // CODEGEN: species_fitness_mountain
}
Terrain::Hills => {
    // ... existing
    fitness.insert(Species::Orc, 0.8);
    // CODEGEN: species_fitness_hills
}
Terrain::Forest => {
    // ... existing
    fitness.insert(Species::Orc, 0.5);
    // CODEGEN: species_fitness_forest
}
Terrain::Plains => {
    // ... existing
    fitness.insert(Species::Orc, 0.7);
    // CODEGEN: species_fitness_plains
}
Terrain::Marsh => {
    // ... existing
    fitness.insert(Species::Orc, 0.6);
    // CODEGEN: species_fitness_marsh
}
Terrain::Coast => {
    // ... existing
    fitness.insert(Species::Orc, 0.3);
    // CODEGEN: species_fitness_coast
}
Terrain::Desert => {
    // ... existing
    fitness.insert(Species::Orc, 0.5);
    // CODEGEN: species_fitness_desert
}
Terrain::River => {
    // ... existing
    fitness.insert(Species::Orc, 0.4);
    // CODEGEN: species_fitness_river
}
```

**Verification:**
```bash
cargo check 2>&1 | grep -i error
# Expected: no errors
grep "Species::Orc" src/aggregate/region.rs | wc -l
# Expected: 8
```

---

### Task 3: Update Patcher for Region Fitness

**Goal:** Generate fitness patches from TOML `[terrain_fitness]` section

**File:** `tools/species_gen/patcher.py`

**Add to `generate_patches()` method:**

```python
def generate_patches(self, spec: Dict[str, Any]) -> List[Patch]:
    patches = []
    name = spec['metadata']['name']
    module_name = spec['metadata']['module_name']

    # ... existing patches ...

    # NEW: Terrain fitness patches
    if 'terrain_fitness' in spec:
        terrain_map = {
            'mountain': 'Mountain',
            'hills': 'Hills',
            'forest': 'Forest',
            'plains': 'Plains',
            'marsh': 'Marsh',
            'coast': 'Coast',
            'desert': 'Desert',
            'river': 'River',
        }

        for terrain_key, terrain_rust in terrain_map.items():
            if terrain_key in spec['terrain_fitness']:
                fitness = spec['terrain_fitness'][terrain_key]
                patches.append(Patch(
                    file='src/aggregate/region.rs',
                    marker=f'species_fitness_{terrain_key}',
                    content=f'fitness.insert(Species::{name}, {fitness});',
                ))

    return patches
```

**Verification:**
```bash
cd /home/astre/arc-citadel
source .venv/bin/activate
python -c "
from tools.species_gen.patcher import SpeciesPatcher
import toml
spec = toml.load('species/orc.toml')
p = SpeciesPatcher('.')
patches = p.generate_patches(spec)
terrain_patches = [p for p in patches if 'fitness' in p.marker]
print(f'Terrain patches: {len(terrain_patches)}')
for patch in terrain_patches[:3]:
    print(f'  {patch.marker}: {patch.content}')
"
# Expected: Terrain patches: 8
```

---

### Task 4: Add Entity Action Selection Markers

**Goal:** Enable patching species-specific action selection into simulation

**File:** `src/simulation/action_select.rs`

**Current structure:** The file has `select_action_human()` function but no generic dispatcher.

**Step 4a:** Add species dispatch function at end of file:

```rust
/// Select action for any species (dispatches to species-specific logic)
pub fn select_action_for_species(
    species: Species,
    ctx: &SelectionContext,
) -> Option<Task> {
    match species {
        Species::Human => select_action_human(ctx),
        // CODEGEN: species_action_select_arms
        _ => select_action_human(ctx), // fallback to human behavior
    }
}
```

**Step 4b:** Add marker for new action selection functions at end of file:

```rust
// CODEGEN: species_action_select_functions
```

**Verification:**
```bash
grep -c "CODEGEN: species_action" src/simulation/action_select.rs
# Expected: 2
cargo check 2>&1 | grep -i error
# Expected: no errors
```

---

### Task 5: Create Action Selection Template

**Goal:** Template for generating species-specific action selection functions

**File:** `tools/species_gen/templates/action_select_function.rs.j2`

**Content:**
```jinja2
/// Action selection for {{ name }} entities
///
/// Generated from {{ module_name }}.toml
/// Priorities: {% for k, v in action_priorities.items() %}{{ k }}={{ v }} {% endfor %}
pub fn select_action_{{ module_name }}(ctx: &SelectionContext) -> Option<Task> {
    // Critical needs always take priority (same as human)
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Check for disposition-based responses
    if let Some(task) = check_disposition_response(ctx) {
        return Some(task);
    }

    {% if action_priorities %}
    // Species-specific weighted action selection
    let roll: f32 = rand::random();
    let mut cumulative = 0.0;

    {% for action, weight in action_priorities.items() %}
    cumulative += {{ weight }};
    if roll < cumulative {
        {% if action == 'fight' %}
        if ctx.threat_nearby {
            return Some(Task {
                action: ActionId::Attack,
                target_position: None,
                target_entity: None,
                priority: TaskPriority::High,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Instinct,
            });
        }
        {% elif action == 'gather' %}
        if ctx.food_available {
            return Some(Task {
                action: ActionId::Gather,
                target_position: ctx.nearest_food_zone.map(|(_, pos, _)| pos),
                target_entity: None,
                priority: TaskPriority::Normal,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            });
        }
        {% elif action == 'rest' %}
        if ctx.needs.fatigue > 0.5 {
            return Some(Task {
                action: ActionId::Rest,
                target_position: None,
                target_entity: None,
                priority: TaskPriority::Normal,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            });
        }
        {% elif action == 'socialize' %}
        if ctx.entity_nearby {
            return Some(Task {
                action: ActionId::Talk,
                target_position: None,
                target_entity: None,
                priority: TaskPriority::Low,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            });
        }
        {% elif action == 'wander' %}
        // Default wandering behavior
        {% endif %}
    }
    {% endfor %}
    {% endif %}

    // Fall back to idle behavior
    Some(select_idle_action(ctx))
}
```

**Verification:**
```bash
test -f tools/species_gen/templates/action_select_function.rs.j2 && echo "Template exists"
```

---

### Task 6: Update Patcher for Action Selection

**Goal:** Generate action selection patches from TOML

**File:** `tools/species_gen/patcher.py`

**Add to `generate_patches()` method:**

```python
# Action selection match arm
patches.append(Patch(
    file='src/simulation/action_select.rs',
    marker='species_action_select_arms',
    content=f'Species::{name} => select_action_{module_name}(ctx),',
))
```

**File:** `tools/species_gen/generator.py`

**Add method to generate action selection function:**

```python
def generate_action_select(self, spec: Dict[str, Any]) -> str:
    """Generate the action selection function for a species."""
    template = self.env.get_template('action_select_function.rs.j2')

    return template.render(
        name=spec['metadata']['name'],
        module_name=spec['metadata']['module_name'],
        action_priorities=spec.get('action_priorities', {}),
    )
```

**Verification:**
```bash
python -c "
from tools.species_gen.generator import SpeciesGenerator
import toml
spec = toml.load('species/orc.toml')
# Add test action_priorities if not present
if 'action_priorities' not in spec:
    spec['action_priorities'] = {'fight': 0.3, 'gather': 0.2, 'rest': 0.15, 'wander': 0.35}
g = SpeciesGenerator('.')
code = g.generate_action_select(spec)
print(code[:500])
"
```

---

### Task 7: Enhance Behavior Module Template

**Goal:** Generate behavior modules with configurable thresholds from TOML

**File:** `tools/species_gen/templates/behavior_module.rs.j2`

**Replace current template with:**

```jinja2
//! {{ name }}-specific polity behavior
//!
//! Generated from {{ module_name }}.toml
//! Aggression: {{ behavior.aggression | default(0.5) }}
//! Expansion threshold: {{ behavior.expansion_threshold | default(0.6) }}

use crate::aggregate::polity::Polity;
use crate::aggregate::world::AggregateWorld;
use crate::aggregate::events::EventType;
use crate::aggregate::systems::expansion::find_expansion_targets;

// ===== GENERATED CONSTANTS (from TOML) =====
const AGGRESSION: f32 = {{ behavior.aggression | default(0.5) }};
const EXPANSION_THRESHOLD: f32 = {{ behavior.expansion_threshold | default(0.6) }};
const POPULATION_FOR_EXPANSION: u32 = {{ behavior.population_for_expansion | default(5000) }};
{% if behavior.war_triggers %}
const WAR_STRENGTH_RATIO: f32 = {{ behavior.war_triggers.strength_ratio_required | default(1.2) }};
{% endif %}

/// Generate {{ name }}-specific events for a polity
pub fn tick(polity: &Polity, world: &AggregateWorld, _year: u32) -> Vec<EventType> {
    let mut events = Vec::new();

    // Access {{ module_name }}-specific state
    let state = match polity.{{ module_name }}_state() {
        Some(s) => s,
        None => return events,
    };

    // EXPANSION: Check if we should expand
    if polity.population > POPULATION_FOR_EXPANSION {
        let targets = find_expansion_targets(polity, world);

        if !targets.unclaimed.is_empty() {
            // Aggression affects expansion likelihood
            if rand::random::<f32>() < AGGRESSION {
                events.push(EventType::Expansion {
                    polity: polity.id.0,
                    region: targets.unclaimed[0],
                });
            }
        }
    }

    {% if behavior.war_triggers %}
    // WAR: Check for war opportunities (aggression-driven)
    if AGGRESSION > 0.5 {
        for (&target_id, rel) in &polity.relations {
            if rel.at_war || rel.alliance {
                continue;
            }

            if let Some(target) = world.get_polity(target_id) {
                let strength_ratio = polity.military_strength / target.military_strength.max(1.0);
                if strength_ratio > WAR_STRENGTH_RATIO && rand::random::<f32>() < AGGRESSION * 0.5 {
                    events.push(EventType::WarDeclared {
                        aggressor: polity.id.0,
                        defender: target_id,
                        cause: crate::aggregate::world::WarCause::Expansion,
                    });
                    break; // One war at a time
                }
            }
        }
    }
    {% endif %}

    // TODO: Add {{ name }}-specific mechanics using state fields:
    {% for field in polity_state_fields %}
    // - state.{{ field.name }}: {{ field.rust_type }}
    {% endfor %}

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
            polity_type: PolityType::{{ polity_types.large | default('Kingdom') }},
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
    fn test_{{ module_name }}_state_accessor() {
        let polity = create_test_polity();
        let state = polity.{{ module_name }}_state();
        assert!(state.is_some());
    }

    #[test]
    fn test_{{ module_name }}_constants() {
        assert!(AGGRESSION >= 0.0 && AGGRESSION <= 1.0);
        assert!(EXPANSION_THRESHOLD >= 0.0 && EXPANSION_THRESHOLD <= 1.0);
    }
}
```

**Verification:**
```bash
# After updating template, regenerate orc behavior module
source .venv/bin/activate
python tools/species_gen/cli.py species/orc.toml --force --dry-run 2>&1 | head -50
```

---

### Task 8: Update orc.toml with Full Schema

**Goal:** Add all new schema sections to orc.toml

**File:** `species/orc.toml`

**Add/update sections:**

```toml
[metadata]
name = "Orc"
module_name = "orc"

[entity_values]
rage = 0.5
strength = 0.5
dominance = 0.5
clan_loyalty = 0.5
blood_debt = 0.5
territory = 0.5
combat_prowess = 0.5

[polity_state]
waaagh_level = { type = "f32", default = "0.0" }
raid_targets = { type = "Vec<u32>", default = "Vec::new()" }
blood_feuds = { type = "Vec<u32>", default = "Vec::new()" }
tribal_strength = { type = "f32", default = "0.5" }

[terrain_fitness]
plains = 0.7
hills = 0.8
mountains = 0.6
forest = 0.5
marsh = 0.6
coast = 0.3
desert = 0.5
river = 0.4

[growth]
rate = 1.02

[naming]
prefixes = ["Grak", "Thok", "Zug", "Mog", "Gor", "Skul", "Nar", "Krag", "Urg", "Drak"]
suffixes = ["gash", "gore", "skull", "bone", "rot", "maw", "fang", "claw", "blood", "war"]

[polity_types]
small = "Warband"
large = "Horde"

[behavior]
aggression = 0.8
expansion_threshold = 0.5
population_for_expansion = 3000

[behavior.war_triggers]
strength_ratio_required = 1.2
grudge_threshold = 0.8

[behavior.expansion]
terrain_filter = []
fitness_minimum = 0.3

[action_priorities]
fight = 0.3
gather = 0.2
rest = 0.15
socialize = 0.05
wander = 0.3
```

**Verification:**
```bash
python -c "import toml; d = toml.load('species/orc.toml'); print('Sections:', list(d.keys()))"
# Expected: Sections: ['metadata', 'entity_values', 'polity_state', 'terrain_fitness', 'growth', 'naming', 'polity_types', 'behavior', 'action_priorities']
```

---

### Task 9: Update Schema Documentation

**Goal:** Document the new schema sections

**File:** `tools/species_gen/schema.md`

**Add sections:**

```markdown
## Terrain Fitness (NEW)

Defines species suitability for each terrain type (0.0 to 1.0):

```toml
[terrain_fitness]
plains = 0.7      # Good for open combat
hills = 0.8       # Excellent defensive terrain
mountains = 0.6   # Acceptable
forest = 0.5      # Average
marsh = 0.6       # Orcs don't mind swamps
coast = 0.3       # Poor - orcs don't like water
desert = 0.5      # Survivable
river = 0.4       # Tolerable
```

All 8 terrain types should be specified. Missing values default to 0.5.

## Behavior Configuration (NEW)

Controls aggregate-level polity behavior:

```toml
[behavior]
aggression = 0.8              # 0.0-1.0, affects war declaration likelihood
expansion_threshold = 0.5     # Population pressure threshold for expansion
population_for_expansion = 3000  # Minimum population before considering expansion

[behavior.war_triggers]
strength_ratio_required = 1.2  # Attack when this much stronger than target
grudge_threshold = 0.8         # Grudge severity to trigger war

[behavior.expansion]
terrain_filter = []            # Empty = all terrain, or ["mountain", "hills"]
fitness_minimum = 0.3          # Only expand to regions with this fitness
```

## Action Priorities (NEW)

Weights for entity-level action selection (should sum to ~1.0):

```toml
[action_priorities]
fight = 0.3       # Attack nearby threats
gather = 0.2      # Collect resources
rest = 0.15       # Recover fatigue
socialize = 0.05  # Talk to nearby entities
wander = 0.3      # Explore/patrol
```

These weights influence which action an idle entity chooses.
```

---

### Task 10: Integration Test

**Goal:** Verify full generation pipeline works

**Test script:** `tools/species_gen/test_full_generation.py`

```python
#!/usr/bin/env python3
"""Integration test for species generator v2."""

import subprocess
import sys
from pathlib import Path

def run(cmd):
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    return result.returncode == 0, result.stdout, result.stderr

def main():
    print("=== Species Generator V2 Integration Test ===\n")

    # Test 1: Verify markers exist
    print("1. Checking markers...")
    markers = [
        ("region.rs fitness", "grep -c 'CODEGEN: species_fitness' src/aggregate/region.rs"),
        ("action_select arms", "grep -c 'species_action_select_arms' src/simulation/action_select.rs"),
    ]

    for name, cmd in markers:
        ok, out, _ = run(cmd)
        count = int(out.strip()) if ok else 0
        status = "OK" if count > 0 else "MISSING"
        print(f"   {name}: {status} ({count})")

    # Test 2: Generate Orc (dry-run)
    print("\n2. Dry-run generation for Orc...")
    ok, out, err = run("python tools/species_gen/cli.py species/orc.toml --dry-run")
    print(f"   Status: {'OK' if ok else 'FAILED'}")
    if not ok:
        print(f"   Error: {err[:200]}")

    # Test 3: Full generation with force
    print("\n3. Full generation (--force)...")
    ok, out, err = run("python tools/species_gen/cli.py species/orc.toml --force")
    print(f"   Status: {'OK' if ok else 'FAILED'}")

    # Test 4: Compile check
    print("\n4. Cargo check...")
    ok, out, err = run("cargo check 2>&1")
    if ok:
        print("   Status: OK")
    else:
        print("   Status: FAILED")
        # Show first error
        for line in err.split('\n'):
            if 'error[' in line:
                print(f"   {line}")
                break

    # Test 5: Run tests
    print("\n5. Running tests...")
    ok, out, err = run("cargo test --lib 2>&1 | tail -5")
    print(out)

    print("\n=== Test Complete ===")
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
```

**Verification:**
```bash
chmod +x tools/species_gen/test_full_generation.py
python tools/species_gen/test_full_generation.py
```

---

## Task Summary

| # | Task | Priority | Est. Complexity |
|---|------|----------|-----------------|
| 1 | Add region fitness markers | Critical | Low |
| 2 | Add Orc fitness values (manual) | Critical | Low |
| 3 | Update patcher for region fitness | High | Medium |
| 4 | Add action selection markers | High | Low |
| 5 | Create action selection template | High | Medium |
| 6 | Update patcher for action selection | High | Low |
| 7 | Enhance behavior module template | Medium | Medium |
| 8 | Update orc.toml with full schema | Medium | Low |
| 9 | Update schema documentation | Low | Low |
| 10 | Integration test | High | Low |

## Success Criteria

1. `cargo check` passes after running generator
2. `cargo test --lib` passes (152+ tests)
3. Orc polities have non-zero fitness on all terrain types
4. New species can be added with single TOML + single command
5. Generated behavior modules have configurable thresholds

## Rollback Plan

If issues arise:
1. All changes are additive (markers, new functions)
2. Existing species behavior modules are NOT modified
3. Generator can be run with `--dry-run` to preview changes
4. Git revert to previous commit if needed
