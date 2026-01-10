# Module Specification Schema for Arc Citadel

> **For Claude:** This document defines the schema format for domain module specifications. Use this schema when implementing new domain modules (species values, perception filters, combat variants, morale systems).

**Goal:** Enable one-shot LLM implementation of domain modules by providing a precise, unambiguous specification format.

**Scope:** This schema applies ONLY to rapidly-integrated domain modules:
- Species value calculations (adding new species like Orc)
- Perception filters (species-specific perception)
- Combat resolution variants (new weapon types)
- Morale/relationship systems

**NOT for:** Core architecture, history generation, game loop orchestration, UI rendering.

---

## The Schema Format

Copy this template for each new domain module:

```
MODULE: [PascalCase name]
CATEGORY: [Pure Transform | State Query | State Mutation]
PURPOSE: [Single sentence describing transformation]

=== FILE LOCATION ===
[Exact path, e.g., src/entity/species/orc.rs]

=== PATTERN TO FOLLOW ===
[Path to reference implementation, e.g., src/entity/species/human.rs]
Key differences from reference:
1. [Specific difference]
2. [Another difference]

=== INPUT CONTRACT ===
[TypeName]:
  - field_name: Type           // semantic meaning
  - field_name: Type           // semantic meaning
  ...

=== OUTPUT CONTRACT ===
[TypeName]:
  - field_name: Type           // semantic meaning
  ...

=== STATE ACCESS ===
READS: [None | list of world state types readable]
WRITES: [None | list of component types modified]

=== INVARIANTS ===
1. [Boolean condition that must ALWAYS hold]
2. [Another invariant]
...

=== VALIDATION SCENARIOS ===
SCENARIO: [Short name]
  GIVEN: [Setup conditions]
  INPUT: [Specific input values]
  EXPECTED: [What output should be]
  RATIONALE: [Why this validates correctness]

SCENARIO: [Second scenario]
  ...

=== INTEGRATION POINT ===
Callsite: [Where this module is called from]
```rust
// Wiring code showing imports and invocation
use crate::module::path;
// Show exact integration
```

=== TEST TEMPLATE ===
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_[scenario_name]() {
        // Arrange
        let input = ...;

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result.field, expected_value);
    }
}
```

=== DEPENDENCIES ===
UPSTREAM: [List of modules that produce this module's inputs]
DOWNSTREAM: [List of modules that consume this module's outputs]

=== ANTI-PATTERNS ===
- NEVER: [Specific thing to avoid]
- NEVER: [Another antipattern]
```

---

## Category-Implicit Error Semantics

The `CATEGORY` field defines the module's error behavior:

| Category | Return Type | Error Behavior |
|----------|-------------|----------------|
| Pure Transform | `Output` directly | Infallible - always succeeds |
| State Query | `Option<Output>` or `Vec<Output>` | May return None/empty, not an error |
| State Mutation | `Result<Output, ModuleError>` | May fail validation |

Do NOT add explicit ERROR_HANDLING sections unless the module deviates from these defaults.

---

## Example: OrcValues Module

```
MODULE: OrcValues
CATEGORY: Pure Transform
PURPOSE: Define value weights for Orc action selection (Orcs prioritize strength and dominance over human concepts like honor and beauty)

=== FILE LOCATION ===
src/entity/species/orc.rs

=== PATTERN TO FOLLOW ===
src/entity/species/human.rs
Key differences from reference:
1. Replace human value concepts with Orc concepts (rage, dominance, etc.)
2. Add blood_rage mechanism that triggers at low HP
3. Perception filter: Orcs detect blood trails at extended range

=== INPUT CONTRACT ===
OrcValues:
  - rage: f32              // 0.0-1.0, propensity for uncontrolled violence
  - strength: f32          // 0.0-1.0, respect for physical power
  - dominance: f32         // 0.0-1.0, need to assert hierarchy position
  - clan_loyalty: f32      // 0.0-1.0, tribal vs individual priority
  - blood_debt: f32        // 0.0-1.0, vengeance obligations
  - territory: f32         // 0.0-1.0, land/resource ownership drive
  - combat_prowess: f32    // 0.0-1.0, skill in battle as virtue

=== OUTPUT CONTRACT ===
ActionWeight:
  - action: ActionId       // Which action to consider
  - weight: f32            // 0.0-1.0, how strongly this action is preferred

=== STATE ACCESS ===
READS: None (pure function of input values)
WRITES: None

=== INVARIANTS ===
1. All value fields are in range [0.0, 1.0]
2. dominant() returns the highest-weighted value name
3. If rage > 0.8 and HP < 30%, blood_rage modifier applies

=== VALIDATION SCENARIOS ===
SCENARIO: High-rage orc in combat
  GIVEN: Orc with rage=0.9, combat context
  INPUT: OrcValues { rage: 0.9, strength: 0.5, dominance: 0.4, ... }
  EXPECTED: Attack action weighted highest (>0.7)
  RATIONALE: Rage-dominant orcs prefer direct combat

SCENARIO: Clan-loyal orc with ally nearby
  GIVEN: Orc with clan_loyalty=0.8, ally in perception
  INPUT: OrcValues { rage: 0.3, clan_loyalty: 0.8, ... }
  EXPECTED: Help/Defend action weighted high (>0.5)
  RATIONALE: Clan loyalty overrides individual impulses

SCENARIO: Territorial orc at border
  GIVEN: Orc with territory=0.9, at clan boundary
  INPUT: OrcValues { territory: 0.9, dominance: 0.6, ... }
  EXPECTED: Patrol/Claim action weighted high
  RATIONALE: Territory drives defensive positioning

=== INTEGRATION POINT ===
Callsite: src/simulation/action_select.rs
```rust
// In action_select.rs - add species dispatch
use crate::entity::species::orc::OrcValues;

pub fn select_action_orc(ctx: &SelectionContext<OrcValues>) -> Option<Task> {
    // Same pattern as select_action_human
    // but uses OrcValues instead of HumanValues
}

// In tick.rs - add to species dispatch
match entity_species {
    Species::Human => select_action_human(&ctx),
    Species::Orc => select_action_orc(&ctx),
}
```

=== TEST TEMPLATE ===
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orc_default_values() {
        let values = OrcValues::default();
        // All values should be 0.0 by default
        assert_eq!(values.rage, 0.0);
        assert_eq!(values.strength, 0.0);
    }

    #[test]
    fn test_orc_dominant_value() {
        let mut values = OrcValues::default();
        values.rage = 0.9;
        values.dominance = 0.3;

        let (name, level) = values.dominant();
        assert_eq!(name, "rage");
        assert_eq!(level, 0.9);
    }

    #[test]
    fn test_blood_rage_trigger() {
        let mut values = OrcValues::default();
        values.rage = 0.85;
        // Test blood rage calculation at low HP
        // ... scenario-specific assertions
    }
}
```

=== DEPENDENCIES ===
UPSTREAM: Genetics (generates initial values), Perception (provides context)
DOWNSTREAM: ActionSelect (consumes weights), ThoughtGeneration (uses values for filtering)

=== ANTI-PATTERNS ===
- NEVER: Use human value concepts (honor, beauty, piety) for Orcs
- NEVER: Hard-code percentage modifiers like "50% attack bonus"
- NEVER: Reference global state; this is a pure function
- NEVER: Add value fields not listed in INPUT CONTRACT without updating spec
```

---

## Implementation Tasks

### Task 1: Create Schema Template File

**File:** `docs/schemas/TEMPLATE.md`

Create a blank template file that can be copied for each new module.

**Verification:** `cat docs/schemas/TEMPLATE.md` shows the complete template.

---

### Task 2: Create OrcValues Schema

**File:** `docs/schemas/orc-values.md`

Fill out the complete schema for the OrcValues module using the example above.

**Verification:** Schema is complete with all sections filled.

---

### Task 3: Implement OrcValues Module

**File:** `src/entity/species/orc.rs`

Following the schema specification:

```rust
//! Orc-specific archetype with SoA layout

use crate::core::types::{EntityId, Vec2, Tick};
use crate::entity::body::BodyState;
use crate::entity::needs::Needs;
use crate::entity::thoughts::ThoughtBuffer;
use crate::entity::tasks::TaskQueue;
use crate::entity::social::SocialMemory;

/// Orc-specific value vocabulary
#[derive(Debug, Clone, Default)]
pub struct OrcValues {
    pub rage: f32,
    pub strength: f32,
    pub dominance: f32,
    pub clan_loyalty: f32,
    pub blood_debt: f32,
    pub territory: f32,
    pub combat_prowess: f32,
}

impl OrcValues {
    pub fn dominant(&self) -> (&'static str, f32) {
        let values = [
            ("rage", self.rage),
            ("strength", self.strength),
            ("dominance", self.dominance),
            ("clan_loyalty", self.clan_loyalty),
            ("blood_debt", self.blood_debt),
            ("territory", self.territory),
            ("combat_prowess", self.combat_prowess),
        ];
        values.into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
    }
}

/// Structure of Arrays for orc entities
pub struct OrcArchetype {
    pub ids: Vec<EntityId>,
    pub names: Vec<String>,
    pub birth_ticks: Vec<Tick>,
    pub positions: Vec<Vec2>,
    pub velocities: Vec<Vec2>,
    pub body_states: Vec<BodyState>,
    pub needs: Vec<Needs>,
    pub thoughts: Vec<ThoughtBuffer>,
    pub values: Vec<OrcValues>,
    pub task_queues: Vec<TaskQueue>,
    pub alive: Vec<bool>,
    pub social_memories: Vec<SocialMemory>,
}

// ... (same pattern as HumanArchetype)
```

**Verification:** `cargo build` succeeds, `cargo test orc` passes.

---

### Task 4: Add Orc to World

**File:** `src/ecs/world.rs`

Add the OrcArchetype to the World struct:

```rust
pub struct World {
    pub humans: HumanArchetype,
    pub orcs: OrcArchetype,  // ADD THIS
    // ...
}
```

**Verification:** `cargo build` succeeds.

---

### Task 5: Add Orc Action Selection

**File:** `src/simulation/action_select.rs`

Add `select_action_orc` following the same pattern as `select_action_human` but using OrcValues:

```rust
pub fn select_action_orc(ctx: &SelectionContext) -> Option<Task> {
    // Similar to select_action_human but with Orc-specific behavior
    // e.g., blood_rage check when HP is low
}
```

**Verification:** `cargo test action_select` passes.

---

### Task 6: Update Tick System

**File:** `src/simulation/tick.rs`

Add Orc processing to all tick phases:

```rust
fn update_needs(world: &mut World) {
    // Existing human processing
    // ...

    // Add orc processing
    let orc_indices: Vec<usize> = world.orcs.iter_living().collect();
    for i in orc_indices {
        // Same pattern as humans
    }
}
```

**Verification:** `cargo test tick` passes with orcs.

---

## Verification Commands

After implementing all tasks:

```bash
# Build succeeds
cargo build

# All tests pass
cargo test

# Orc-specific tests pass
cargo test orc

# Action selection tests pass
cargo test action_select

# Tick tests pass with orcs
cargo test tick
```

---

## Schema Validation Checklist

When reviewing a schema before implementation, verify:

- [ ] MODULE name is PascalCase
- [ ] CATEGORY is one of: Pure Transform, State Query, State Mutation
- [ ] PURPOSE is a single sentence
- [ ] FILE LOCATION is an exact path
- [ ] PATTERN TO FOLLOW points to existing code
- [ ] INPUT CONTRACT lists all fields with types and semantics
- [ ] OUTPUT CONTRACT lists all fields with types and semantics
- [ ] INVARIANTS are boolean conditions, not prose
- [ ] VALIDATION SCENARIOS have GIVEN/INPUT/EXPECTED/RATIONALE
- [ ] INTEGRATION POINT shows actual Rust code
- [ ] TEST TEMPLATE shows Arrange/Act/Assert pattern
- [ ] ANTI-PATTERNS start with "NEVER:"
