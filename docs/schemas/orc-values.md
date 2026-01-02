# OrcValues Module Schema

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
