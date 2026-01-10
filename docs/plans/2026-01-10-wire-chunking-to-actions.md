# Wire Chunking System to Gameplay Actions

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Connect the hierarchical chunking skill system to action execution so skill level affects outcome quality, attention limits how much entities can do, and practice improves skills.

**Architecture:** Every action checks entity's ChunkLibrary before execution. Skill modifier (from encoding_depth) affects outcome variance. Attention budget limits actions per tick. Experience is recorded for learning.

**Tech Stack:** Rust, no new dependencies. Extends existing `src/skills/` module.

---

## Overview

| Task | Component | Files |
|------|-----------|-------|
| 1 | Physical Domain Chunks | `src/skills/chunk_id.rs`, `src/skills/definitions.rs` |
| 2 | Action-to-Chunk Mapping | `src/skills/action_mapping.rs` |
| 3 | Generic Action Resolution | `src/skills/integration.rs` |
| 4 | Attention Refresh in Tick | `src/simulation/tick.rs` |
| 5 | Wire Combat Actions | `src/simulation/tick.rs` |
| 6 | Wire Work Actions | `src/simulation/tick.rs` |
| 7 | Wire Movement Actions | `src/simulation/tick.rs` |
| 8 | Wire Social Actions | `src/simulation/tick.rs` |
| 9 | Integration Tests | `tests/skill_action_integration.rs` |

---

## Task 1: Add Physical Domain Chunks

**Files:**
- Modify: `src/skills/chunk_id.rs`
- Modify: `src/skills/definitions.rs`
- Modify: `src/skills/mod.rs`

Physical domain exists in `ChunkDomain` but has no chunks yet.

**Step 1: Add Physical chunk variants to ChunkId**

Add after line 106 in `src/skills/chunk_id.rs` (after SocialCultOfPersonality):

```rust
    // === PHYSICAL DOMAIN ===
    // Level 1 - Micro-chunks
    PhysicalEfficientGait,     // Weight shift + footfall control + breath
    PhysicalQuietMovement,     // Footfall control + weight shift
    PhysicalPowerStance,       // Balance + muscle engage + grip
    PhysicalClimbGrip,         // Grip adjust + weight shift

    // Level 2 - Technique chunks
    PhysicalDistanceRunning,   // Efficient gait + breath control
    PhysicalHeavyLifting,      // Power stance + breath control
    PhysicalSilentApproach,    // Quiet movement + timing
    PhysicalRockClimbing,      // Climb grip + balance

    // Level 3 - Activity chunks
    PhysicalSustainedLabor,    // Heavy lifting + pace regulation
    PhysicalRoughTerrainTravel,// Efficient gait + obstacle handling
    PhysicalSwimming,          // Breath control + stroke technique
```

**Step 2: Update domain() method in ChunkId**

Add to the domain() match in `src/skills/chunk_id.rs` (after Social domain section, around line 179):

```rust
            // Physical domain - all physical activity chunks
            Self::PhysicalEfficientGait
            | Self::PhysicalQuietMovement
            | Self::PhysicalPowerStance
            | Self::PhysicalClimbGrip
            | Self::PhysicalDistanceRunning
            | Self::PhysicalHeavyLifting
            | Self::PhysicalSilentApproach
            | Self::PhysicalRockClimbing
            | Self::PhysicalSustainedLabor
            | Self::PhysicalRoughTerrainTravel
            | Self::PhysicalSwimming => ChunkDomain::Physical,
```

**Step 3: Update level() method in ChunkId**

Add to the level() match (after Social Level 5 section, around line 261):

```rust
            // Physical Level 1
            Self::PhysicalEfficientGait
            | Self::PhysicalQuietMovement
            | Self::PhysicalPowerStance
            | Self::PhysicalClimbGrip => 1,

            // Physical Level 2
            Self::PhysicalDistanceRunning
            | Self::PhysicalHeavyLifting
            | Self::PhysicalSilentApproach
            | Self::PhysicalRockClimbing => 2,

            // Physical Level 3
            Self::PhysicalSustainedLabor
            | Self::PhysicalRoughTerrainTravel
            | Self::PhysicalSwimming => 3,
```

**Step 4: Update name() method in ChunkId**

Add to the name() match (after Social names section, around line 341):

```rust
            // Physical Level 1
            Self::PhysicalEfficientGait => "Efficient Gait",
            Self::PhysicalQuietMovement => "Quiet Movement",
            Self::PhysicalPowerStance => "Power Stance",
            Self::PhysicalClimbGrip => "Climb Grip",
            // Physical Level 2
            Self::PhysicalDistanceRunning => "Distance Running",
            Self::PhysicalHeavyLifting => "Heavy Lifting",
            Self::PhysicalSilentApproach => "Silent Approach",
            Self::PhysicalRockClimbing => "Rock Climbing",
            // Physical Level 3
            Self::PhysicalSustainedLabor => "Sustained Labor",
            Self::PhysicalRoughTerrainTravel => "Rough Terrain Travel",
            Self::PhysicalSwimming => "Swimming",
```

**Step 5: Add Physical chunk definitions**

Add to CHUNK_LIBRARY in `src/skills/definitions.rs` (after Social definitions, before the closing `];`):

```rust
    // === PHYSICAL DOMAIN ===
    // Level 1 - Micro-chunks (Atomic)
    ChunkDefinition {
        id: ChunkId::PhysicalEfficientGait,
        name: "Efficient Gait",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 50,
    },
    ChunkDefinition {
        id: ChunkId::PhysicalQuietMovement,
        name: "Quiet Movement",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 30,
    },
    ChunkDefinition {
        id: ChunkId::PhysicalPowerStance,
        name: "Power Stance",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 25,
    },
    ChunkDefinition {
        id: ChunkId::PhysicalClimbGrip,
        name: "Climb Grip",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 20,
    },

    // Level 2 - Technique chunks (Composite)
    ChunkDefinition {
        id: ChunkId::PhysicalDistanceRunning,
        name: "Distance Running",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysicalEfficientGait,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysicalEfficientGait],
        base_repetitions: 100,
    },
    ChunkDefinition {
        id: ChunkId::PhysicalHeavyLifting,
        name: "Heavy Lifting",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysicalPowerStance,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysicalPowerStance],
        base_repetitions: 80,
    },
    ChunkDefinition {
        id: ChunkId::PhysicalSilentApproach,
        name: "Silent Approach",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysicalQuietMovement,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysicalQuietMovement],
        base_repetitions: 60,
    },
    ChunkDefinition {
        id: ChunkId::PhysicalRockClimbing,
        name: "Rock Climbing",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysicalClimbGrip,
            ChunkId::PhysicalPowerStance,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysicalClimbGrip, ChunkId::PhysicalPowerStance],
        base_repetitions: 70,
    },

    // Level 3 - Activity chunks
    ChunkDefinition {
        id: ChunkId::PhysicalSustainedLabor,
        name: "Sustained Labor",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysicalHeavyLifting,
            ChunkId::PhysicalPowerStance,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysicalHeavyLifting, ChunkId::PhysicalPowerStance],
        base_repetitions: 200,
    },
    ChunkDefinition {
        id: ChunkId::PhysicalRoughTerrainTravel,
        name: "Rough Terrain Travel",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysicalEfficientGait,
            ChunkId::PhysicalClimbGrip,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysicalEfficientGait, ChunkId::PhysicalClimbGrip],
        base_repetitions: 150,
    },
    ChunkDefinition {
        id: ChunkId::PhysicalSwimming,
        name: "Swimming",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::PhysicalEfficientGait,
        ]),
        context_requirements: &[],
        prerequisite_chunks: &[ChunkId::PhysicalEfficientGait],
        base_repetitions: 120,
    },
```

**Step 6: Run tests**

Run: `cargo test --lib skills::chunk_id::tests -v && cargo test --lib skills::definitions::tests -v`
Expected: PASS (existing tests + new chunks work)

**Step 7: Commit**

```bash
git add src/skills/chunk_id.rs src/skills/definitions.rs
git commit -m "feat(skills): add Physical domain chunks"
```

---

## Task 2: Action-to-Chunk Mapping

**Files:**
- Create: `src/skills/action_mapping.rs`
- Modify: `src/skills/mod.rs`

**Step 1: Create action_mapping.rs**

Create `src/skills/action_mapping.rs`:

```rust
//! Maps ActionId to required skill chunks
//!
//! Each action requires certain chunks for skilled execution.
//! Actions without mappings execute without skill checks.

use crate::actions::catalog::ActionId;
use crate::skills::ChunkId;

/// Get the chunks required for an action
///
/// Returns empty slice if action has no skill requirements
pub fn get_chunks_for_action(action: ActionId) -> &'static [ChunkId] {
    match action {
        // === MOVEMENT ===
        ActionId::MoveTo => &[ChunkId::PhysicalEfficientGait],
        ActionId::Follow => &[ChunkId::PhysicalEfficientGait],
        ActionId::Flee => &[ChunkId::PhysicalDistanceRunning],

        // === SURVIVAL ===
        // Rest, Eat, SeekSafety require no skill - instinctive
        ActionId::Rest | ActionId::Eat | ActionId::SeekSafety => &[],

        // === WORK ===
        ActionId::Build => &[ChunkId::PhysicalSustainedLabor, ChunkId::CraftBasicMeasure],
        ActionId::Craft => &[ChunkId::CraftBasicMeasure, ChunkId::CraftBasicCut],
        ActionId::Gather => &[ChunkId::PhysicalSustainedLabor],
        ActionId::Repair => &[ChunkId::CraftBasicMeasure],

        // === SOCIAL ===
        ActionId::TalkTo => &[ChunkId::SocialActiveListening, ChunkId::SocialBuildRapport],
        ActionId::Help => &[ChunkId::SocialActiveListening],
        ActionId::Trade => &[ChunkId::SocialNegotiateTerms, ChunkId::SocialReadReaction],

        // === COMBAT ===
        ActionId::Attack => &[ChunkId::BasicSwing, ChunkId::BasicStance],
        ActionId::Defend => &[ChunkId::BasicBlock, ChunkId::BasicStance],
        ActionId::Charge => &[ChunkId::BasicSwing, ChunkId::PhysicalDistanceRunning],
        ActionId::HoldPosition => &[ChunkId::BasicStance],

        // === IDLE ===
        // Idle actions require no skill
        ActionId::IdleWander | ActionId::IdleObserve => &[],
    }
}

/// Check if an action requires skill (has chunk mappings)
pub fn action_requires_skill(action: ActionId) -> bool {
    !get_chunks_for_action(action).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_actions_have_chunks() {
        assert!(!get_chunks_for_action(ActionId::Attack).is_empty());
        assert!(!get_chunks_for_action(ActionId::Defend).is_empty());
    }

    #[test]
    fn test_idle_actions_have_no_chunks() {
        assert!(get_chunks_for_action(ActionId::IdleWander).is_empty());
        assert!(get_chunks_for_action(ActionId::IdleObserve).is_empty());
    }

    #[test]
    fn test_survival_actions_instinctive() {
        assert!(get_chunks_for_action(ActionId::Rest).is_empty());
        assert!(get_chunks_for_action(ActionId::Eat).is_empty());
    }

    #[test]
    fn test_work_actions_have_chunks() {
        assert!(!get_chunks_for_action(ActionId::Build).is_empty());
        assert!(!get_chunks_for_action(ActionId::Craft).is_empty());
    }

    #[test]
    fn test_action_requires_skill_helper() {
        assert!(action_requires_skill(ActionId::Attack));
        assert!(action_requires_skill(ActionId::Build));
        assert!(!action_requires_skill(ActionId::Rest));
        assert!(!action_requires_skill(ActionId::IdleWander));
    }
}
```

**Step 2: Update mod.rs exports**

Add to `src/skills/mod.rs` after line 16 (`pub mod resolution;`):

```rust
pub mod action_mapping;
```

Add to exports after line 28 (after resolution exports):

```rust
pub use action_mapping::{get_chunks_for_action, action_requires_skill};
```

**Step 3: Run tests**

Run: `cargo test --lib skills::action_mapping::tests -v`
Expected: PASS (5 tests)

**Step 4: Commit**

```bash
git add src/skills/action_mapping.rs src/skills/mod.rs
git commit -m "feat(skills): add action-to-chunk mapping"
```

---

## Task 3: Generic Action Resolution

**Files:**
- Create: `src/skills/integration.rs`
- Modify: `src/skills/mod.rs`

**Step 1: Create integration.rs**

Create `src/skills/integration.rs`:

```rust
//! Integration layer connecting chunking system to action execution
//!
//! Provides functions to:
//! - Calculate skill modifier for any action
//! - Check attention budget before execution
//! - Record experience after execution

use crate::skills::{
    action_mapping::get_chunks_for_action, calculate_attention_budget, can_afford_attention,
    get_chunk_definition, risks_fumble, ChunkComponents, ChunkId, ChunkLibrary, Experience,
};
use crate::actions::catalog::ActionId;

/// Result of skill check before action execution
#[derive(Debug, Clone)]
pub struct SkillCheckResult {
    /// Skill modifier (0.1 to 1.0) affecting outcome quality
    /// Higher = better outcomes, lower variance
    pub skill_modifier: f32,
    /// Attention cost to execute this action
    pub attention_cost: f32,
    /// Chunks that will be used (for experience recording)
    pub chunks_used: Vec<ChunkId>,
    /// Whether execution can proceed
    pub can_execute: bool,
    /// If can't execute, why
    pub failure_reason: Option<SkillFailure>,
}

/// Reasons skill check might fail
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillFailure {
    /// Not enough attention remaining
    AttentionOverload,
    /// Barely enough attention - high fumble risk
    FumbleRisk,
}

/// Perform skill check for an action
///
/// Call this BEFORE executing an action to get skill_modifier.
/// If can_execute is false, handle failure appropriately.
pub fn skill_check(
    library: &ChunkLibrary,
    action: ActionId,
) -> SkillCheckResult {
    let required_chunks = get_chunks_for_action(action);

    // No skill requirements - execute with full competence
    if required_chunks.is_empty() {
        return SkillCheckResult {
            skill_modifier: 1.0,
            attention_cost: 0.0,
            chunks_used: Vec::new(),
            can_execute: true,
            failure_reason: None,
        };
    }

    // Calculate skill and cost from chunks
    let (attention_cost, skill_modifier, chunks_used) =
        calculate_action_skill(required_chunks, library);

    // Check attention budget
    if !can_afford_attention(library.attention_remaining(), attention_cost) {
        return SkillCheckResult {
            skill_modifier,
            attention_cost,
            chunks_used,
            can_execute: false,
            failure_reason: Some(SkillFailure::AttentionOverload),
        };
    }

    // Check fumble risk (low attention + low skill = dangerous)
    let remaining_after = library.attention_remaining() - attention_cost;
    if risks_fumble(remaining_after) && skill_modifier < 0.3 {
        return SkillCheckResult {
            skill_modifier,
            attention_cost,
            chunks_used,
            can_execute: false,
            failure_reason: Some(SkillFailure::FumbleRisk),
        };
    }

    SkillCheckResult {
        skill_modifier,
        attention_cost,
        chunks_used,
        can_execute: true,
        failure_reason: None,
    }
}

/// Calculate skill modifier and attention cost from required chunks
///
/// Returns (attention_cost, skill_modifier, chunks_actually_used)
fn calculate_action_skill(
    required_chunks: &[ChunkId],
    library: &ChunkLibrary,
) -> (f32, f32, Vec<ChunkId>) {
    let mut total_cost = 0.0;
    let mut total_modifier = 1.0;
    let mut chunks_used = Vec::new();

    for chunk_id in required_chunks {
        if let Some(state) = library.get_chunk(*chunk_id) {
            // Has this chunk - cost and skill based on encoding depth
            let cost = 1.0 - state.encoding_depth;
            // Skill ranges from 0.5 (just learned) to 1.0 (mastered)
            let modifier = 0.5 + (state.encoding_depth * 0.5);

            total_cost += cost;
            total_modifier *= modifier;
            chunks_used.push(*chunk_id);
        } else {
            // Doesn't have this chunk - check if we can decompose
            if let Some(def) = get_chunk_definition(*chunk_id) {
                match &def.components {
                    ChunkComponents::Atomic => {
                        // Must do atomically - very expensive, low skill
                        total_cost += 0.9;
                        total_modifier *= 0.3;
                    }
                    ChunkComponents::Composite(sub_chunks) => {
                        // Recursively check sub-chunks
                        let (sub_cost, sub_mod, sub_used) =
                            calculate_action_skill(sub_chunks, library);
                        total_cost += sub_cost;
                        total_modifier *= sub_mod;
                        chunks_used.extend(sub_used);
                    }
                }
            } else {
                // Unknown chunk - treat as atomic
                total_cost += 0.9;
                total_modifier *= 0.3;
            }
        }
    }

    // Normalize: average cost across chunks, multiply modifiers
    let num_chunks = required_chunks.len().max(1) as f32;
    let avg_cost = total_cost / num_chunks;

    (avg_cost, total_modifier.clamp(0.1, 1.0), chunks_used)
}

/// Spend attention for an action (call after skill_check succeeds)
pub fn spend_attention(library: &mut ChunkLibrary, cost: f32) {
    library.spend_attention(cost);
}

/// Record experience after action execution
///
/// Call this AFTER action executes with the outcome.
pub fn record_action_experience(
    library: &mut ChunkLibrary,
    chunks_used: &[ChunkId],
    success: bool,
    tick: u64,
) {
    for chunk_id in chunks_used {
        library.record_experience(Experience {
            chunk_id: *chunk_id,
            success,
            tick,
        });

        // Update last_used_tick immediately (even before learning consolidation)
        if let Some(state) = library.get_chunk_mut(*chunk_id) {
            state.last_used_tick = tick;
        }
    }
}

/// Refresh attention budget for a new decision period
///
/// Call at start of tick or other decision point.
pub fn refresh_attention(
    library: &mut ChunkLibrary,
    fatigue: f32,
    pain: f32,
    stress: f32,
) {
    library.attention_budget = calculate_attention_budget(fatigue, pain, stress);
    library.attention_spent = 0.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::PersonalChunkState;

    fn setup_library_with_chunks(encoding_depth: f32) -> ChunkLibrary {
        let mut lib = ChunkLibrary::new();
        lib.attention_budget = 1.0;

        // Add basic combat and physical chunks
        for chunk_id in &[
            ChunkId::BasicSwing,
            ChunkId::BasicStance,
            ChunkId::PhysicalEfficientGait,
        ] {
            lib.set_chunk(*chunk_id, PersonalChunkState {
                encoding_depth,
                repetition_count: (encoding_depth * 100.0) as u32,
                last_used_tick: 0,
                formation_tick: 0,
            });
        }

        lib
    }

    #[test]
    fn test_skill_check_no_chunks_required() {
        let lib = ChunkLibrary::new();
        let result = skill_check(&lib, ActionId::Rest);

        assert!(result.can_execute);
        assert_eq!(result.skill_modifier, 1.0);
        assert_eq!(result.attention_cost, 0.0);
    }

    #[test]
    fn test_skill_check_with_practiced_chunks() {
        let lib = setup_library_with_chunks(0.7);
        let result = skill_check(&lib, ActionId::Attack);

        assert!(result.can_execute);
        assert!(result.skill_modifier > 0.5);
        assert!(result.attention_cost < 0.5);
    }

    #[test]
    fn test_skill_check_novice_high_cost() {
        let lib = setup_library_with_chunks(0.1);
        let result = skill_check(&lib, ActionId::Attack);

        // Should still be able to execute but with high cost
        assert!(result.can_execute);
        assert!(result.attention_cost > 0.5);
    }

    #[test]
    fn test_skill_check_attention_overload() {
        let mut lib = ChunkLibrary::new();
        lib.attention_budget = 0.2;
        lib.attention_spent = 0.2;

        // No chunks = high cost = overload
        let result = skill_check(&lib, ActionId::Attack);

        assert!(!result.can_execute);
        assert_eq!(result.failure_reason, Some(SkillFailure::AttentionOverload));
    }

    #[test]
    fn test_refresh_attention_resets_budget() {
        let mut lib = ChunkLibrary::new();
        lib.attention_budget = 0.5;
        lib.attention_spent = 0.4;

        refresh_attention(&mut lib, 0.0, 0.0, 0.0);

        assert_eq!(lib.attention_budget, 1.0);
        assert_eq!(lib.attention_spent, 0.0);
    }

    #[test]
    fn test_refresh_attention_fatigue_penalty() {
        let mut lib = ChunkLibrary::new();

        refresh_attention(&mut lib, 0.5, 0.0, 0.0);

        // Fatigue should reduce budget
        assert!(lib.attention_budget < 1.0);
        assert!(lib.attention_budget > 0.5);
    }

    #[test]
    fn test_record_experience() {
        let mut lib = setup_library_with_chunks(0.5);

        record_action_experience(&mut lib, &[ChunkId::BasicSwing], true, 100);

        assert!(!lib.pending_experiences().is_empty());
        assert_eq!(lib.pending_experiences()[0].chunk_id, ChunkId::BasicSwing);
        assert!(lib.pending_experiences()[0].success);
    }
}
```

**Step 2: Update mod.rs exports**

Add to `src/skills/mod.rs` after `pub mod action_mapping;`:

```rust
pub mod integration;
```

Add to exports:

```rust
pub use integration::{
    skill_check, spend_attention, record_action_experience, refresh_attention,
    SkillCheckResult, SkillFailure,
};
```

**Step 3: Run tests**

Run: `cargo test --lib skills::integration::tests -v`
Expected: PASS (7 tests)

**Step 4: Commit**

```bash
git add src/skills/integration.rs src/skills/mod.rs
git commit -m "feat(skills): add generic action skill integration"
```

---

## Task 4: Wire Attention Refresh to Tick

**Files:**
- Modify: `src/simulation/tick.rs`

**Step 1: Add import**

Add to imports at top of `src/simulation/tick.rs` (around line 10):

```rust
use crate::skills::refresh_attention;
```

**Step 2: Create refresh_all_attention function**

Add after `update_needs` function (around line 120):

```rust
/// Refresh attention budgets for all entities
///
/// Called at start of tick to reset attention for new decision period.
fn refresh_all_attention(world: &mut World) {
    // Process humans
    let living_indices: Vec<usize> = world.humans.iter_living().collect();
    for i in living_indices {
        let fatigue = world.humans.body_states[i].fatigue;
        let pain = world.humans.body_states[i].pain;
        // Use 0.0 for stress until stress system is added
        let stress = 0.0;

        refresh_attention(&mut world.humans.chunk_libraries[i], fatigue, pain, stress);
    }

    // Process orcs (if they have chunk_libraries)
    // TODO: Add chunk_libraries to OrcArchetype
}
```

**Step 3: Call refresh in run_simulation_tick**

Add after `update_needs(world);` call in `run_simulation_tick` (around line 54):

```rust
    refresh_all_attention(world);
```

**Step 4: Run tests**

Run: `cargo test --lib simulation:: -v`
Expected: PASS (existing tests should still pass)

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat(simulation): refresh attention budgets at tick start"
```

---

## Task 5: Wire Combat Actions

**Files:**
- Modify: `src/simulation/tick.rs`

Combat actions (Attack, Defend, Charge, HoldPosition) need skill checks.

**Step 1: Add skill imports**

Ensure these imports exist at top of `src/simulation/tick.rs`:

```rust
use crate::skills::{skill_check, spend_attention, record_action_experience, SkillFailure};
```

**Step 2: Find combat action execution**

In `execute_tasks`, locate the Combat category match arm (around line 700+ based on structure).
The combat actions are currently stubs. We'll add skill integration.

**Step 3: Modify Attack action execution**

Find the `ActionId::Attack` handling in execute_tasks. Modify to:

```rust
                    ActionId::Attack => {
                        // Skill check
                        let skill_result = skill_check(&world.humans.chunk_libraries[i], ActionId::Attack);

                        if !skill_result.can_execute {
                            // Handle failure - task fails
                            match skill_result.failure_reason {
                                Some(SkillFailure::AttentionOverload) => {
                                    // Too exhausted to attack - abort
                                    true // Mark complete (failed)
                                }
                                Some(SkillFailure::FumbleRisk) => {
                                    // Fumble - potentially hurt self or ally
                                    // For now, just abort
                                    true
                                }
                                None => true,
                            }
                        } else {
                            // Spend attention
                            spend_attention(&mut world.humans.chunk_libraries[i], skill_result.attention_cost);

                            // Execute attack (existing logic)
                            // skill_result.skill_modifier affects damage/accuracy
                            // For now, just mark complete
                            let success = true; // TODO: actual combat resolution

                            // Record experience
                            record_action_experience(
                                &mut world.humans.chunk_libraries[i],
                                &skill_result.chunks_used,
                                success,
                                world.current_tick,
                            );

                            true // Mark complete
                        }
                    }
```

**Step 4: Modify Defend action execution**

Similar pattern for `ActionId::Defend`:

```rust
                    ActionId::Defend => {
                        let skill_result = skill_check(&world.humans.chunk_libraries[i], ActionId::Defend);

                        if !skill_result.can_execute {
                            true // Failed to defend
                        } else {
                            spend_attention(&mut world.humans.chunk_libraries[i], skill_result.attention_cost);

                            // Execute defend - skill_modifier affects block chance
                            let success = true;

                            record_action_experience(
                                &mut world.humans.chunk_libraries[i],
                                &skill_result.chunks_used,
                                success,
                                world.current_tick,
                            );

                            true
                        }
                    }
```

**Step 5: Run tests**

Run: `cargo test --lib simulation:: -v`
Expected: PASS

**Step 6: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat(simulation): wire skill checks to combat actions"
```

---

## Task 6: Wire Work Actions

**Files:**
- Modify: `src/simulation/tick.rs`

Work actions (Build, Craft, Gather, Repair) should use skill_modifier for quality/speed.

**Step 1: Modify Build action**

Find `ActionId::Build` in execute_tasks. The existing Build logic involves `apply_construction_work`.
Modify to factor in skill_modifier:

```rust
                    ActionId::Build => {
                        // Skill check
                        let skill_result = skill_check(&world.humans.chunk_libraries[i], ActionId::Build);

                        if !skill_result.can_execute {
                            // Too tired to build effectively
                            false // Don't progress, but don't abort
                        } else {
                            spend_attention(&mut world.humans.chunk_libraries[i], skill_result.attention_cost);

                            // Existing build logic with skill modifier
                            if let Some(site_id) = task.target_building {
                                let building_skill = world.humans.building_skills[i];
                                // Combine static skill with chunk-based skill
                                let effective_skill = (building_skill + skill_result.skill_modifier) / 2.0;

                                let contribution_result = calculate_worker_contribution(
                                    &world.buildings,
                                    site_id,
                                    effective_skill,
                                    1.0,
                                );

                                let is_complete = match contribution_result {
                                    ContributionResult::Contributed { complete, .. } => {
                                        apply_construction_work(&mut world.buildings, site_id, effective_skill, 1.0);
                                        if complete {
                                            // Building finished
                                            true
                                        } else {
                                            task.progress += 1;
                                            task.progress >= task.duration
                                        }
                                    }
                                    ContributionResult::AlreadyComplete | ContributionResult::NotFound => true,
                                };

                                // Record experience
                                record_action_experience(
                                    &mut world.humans.chunk_libraries[i],
                                    &skill_result.chunks_used,
                                    true, // Building always teaches
                                    world.current_tick,
                                );

                                is_complete
                            } else {
                                true // No target
                            }
                        }
                    }
```

**Step 2: Modify Craft action**

```rust
                    ActionId::Craft => {
                        let skill_result = skill_check(&world.humans.chunk_libraries[i], ActionId::Craft);

                        if !skill_result.can_execute {
                            false // Wait, don't progress
                        } else {
                            spend_attention(&mut world.humans.chunk_libraries[i], skill_result.attention_cost);

                            // Progress craft with skill affecting speed
                            let progress_rate = 0.5 + (skill_result.skill_modifier * 0.5);
                            task.progress += (progress_rate * 2.0) as u32; // Base 1-2 per tick

                            let is_complete = task.progress >= task.duration;

                            if is_complete {
                                // Craft complete - quality based on skill
                                // skill_modifier 0.3 = Poor, 0.5 = Standard, 0.7 = Fine, 0.9 = Masterwork
                                // TODO: Actually produce item with quality
                            }

                            record_action_experience(
                                &mut world.humans.chunk_libraries[i],
                                &skill_result.chunks_used,
                                is_complete,
                                world.current_tick,
                            );

                            is_complete
                        }
                    }
```

**Step 3: Modify Gather action**

```rust
                    ActionId::Gather => {
                        let skill_result = skill_check(&world.humans.chunk_libraries[i], ActionId::Gather);

                        if !skill_result.can_execute {
                            false
                        } else {
                            spend_attention(&mut world.humans.chunk_libraries[i], skill_result.attention_cost);

                            // Efficiency affects how much gathered per tick
                            let efficiency = 0.5 + (skill_result.skill_modifier * 0.5);
                            task.progress += (efficiency * 2.0) as u32;

                            let is_complete = task.progress >= task.duration;

                            record_action_experience(
                                &mut world.humans.chunk_libraries[i],
                                &skill_result.chunks_used,
                                is_complete,
                                world.current_tick,
                            );

                            is_complete
                        }
                    }
```

**Step 4: Run tests**

Run: `cargo test --lib simulation:: -v`
Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat(simulation): wire skill checks to work actions"
```

---

## Task 7: Wire Movement Actions

**Files:**
- Modify: `src/simulation/tick.rs`

Movement actions (MoveTo, Follow, Flee) use skill for efficiency.

**Step 1: Modify MoveTo action**

Find existing MoveTo handling. It moves entity toward target. Skill affects speed.

```rust
                    ActionId::MoveTo => {
                        let skill_result = skill_check(&world.humans.chunk_libraries[i], ActionId::MoveTo);

                        // Movement always possible, but efficiency varies
                        if skill_result.can_execute && skill_result.attention_cost > 0.0 {
                            spend_attention(&mut world.humans.chunk_libraries[i], skill_result.attention_cost);
                        }

                        // Speed modified by skill (50% to 100%)
                        let speed_modifier = 0.5 + (skill_result.skill_modifier * 0.5);

                        if let Some(target) = target_pos {
                            let current = world.humans.positions[i];
                            let direction = (target - current).normalize_or_zero();
                            let base_speed = 2.0; // Base movement per tick
                            let actual_speed = base_speed * speed_modifier;

                            let distance = (target - current).length();
                            if distance < actual_speed {
                                world.humans.positions[i] = target;

                                // Record successful movement experience
                                if !skill_result.chunks_used.is_empty() {
                                    record_action_experience(
                                        &mut world.humans.chunk_libraries[i],
                                        &skill_result.chunks_used,
                                        true,
                                        world.current_tick,
                                    );
                                }

                                true // Arrived
                            } else {
                                world.humans.positions[i] = current + direction * actual_speed;
                                false // Still moving
                            }
                        } else {
                            true // No target
                        }
                    }
```

**Step 2: Modify Flee action**

Flee uses distance running - higher skill = faster escape:

```rust
                    ActionId::Flee => {
                        let skill_result = skill_check(&world.humans.chunk_libraries[i], ActionId::Flee);

                        if skill_result.can_execute && skill_result.attention_cost > 0.0 {
                            spend_attention(&mut world.humans.chunk_libraries[i], skill_result.attention_cost);
                        }

                        // Flee speed: 50% to 150% of base (adrenaline helps)
                        let speed_modifier = 0.5 + skill_result.skill_modifier;
                        let base_speed = 3.0; // Faster than walk
                        let actual_speed = base_speed * speed_modifier;

                        if let Some(target) = target_pos {
                            // Move AWAY from target
                            let current = world.humans.positions[i];
                            let away = (current - target).normalize_or_zero();

                            world.humans.positions[i] = current + away * actual_speed;

                            // Record experience
                            if !skill_result.chunks_used.is_empty() {
                                record_action_experience(
                                    &mut world.humans.chunk_libraries[i],
                                    &skill_result.chunks_used,
                                    true,
                                    world.current_tick,
                                );
                            }

                            task.progress += 1;
                            task.progress >= 10 // Flee for 10 ticks then reassess
                        } else {
                            true // Nothing to flee from
                        }
                    }
```

**Step 3: Run tests**

Run: `cargo test --lib simulation:: -v`
Expected: PASS

**Step 4: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat(simulation): wire skill checks to movement actions"
```

---

## Task 8: Wire Social Actions

**Files:**
- Modify: `src/simulation/tick.rs`

Social actions (TalkTo, Help, Trade) use social chunks.

**Step 1: Modify TalkTo action**

```rust
                    ActionId::TalkTo => {
                        let skill_result = skill_check(&world.humans.chunk_libraries[i], ActionId::TalkTo);

                        if skill_result.can_execute && skill_result.attention_cost > 0.0 {
                            spend_attention(&mut world.humans.chunk_libraries[i], skill_result.attention_cost);
                        }

                        // Social skill affects relationship change
                        // Higher skill = more positive interaction
                        let relationship_bonus = skill_result.skill_modifier * 0.1;

                        // Existing TalkTo logic (approach target, converse)
                        // ... (keep existing movement/interaction logic)

                        // On completion, record experience
                        let is_complete = /* existing completion check */;

                        if is_complete && !skill_result.chunks_used.is_empty() {
                            record_action_experience(
                                &mut world.humans.chunk_libraries[i],
                                &skill_result.chunks_used,
                                true,
                                world.current_tick,
                            );
                        }

                        is_complete
                    }
```

**Step 2: Modify Trade action**

```rust
                    ActionId::Trade => {
                        let skill_result = skill_check(&world.humans.chunk_libraries[i], ActionId::Trade);

                        if !skill_result.can_execute {
                            // Too distracted to negotiate well
                            // Could continue but at disadvantage
                        }

                        if skill_result.attention_cost > 0.0 {
                            spend_attention(&mut world.humans.chunk_libraries[i], skill_result.attention_cost);
                        }

                        // Trade skill affects deal quality
                        // skill_modifier 0.3 = bad deals, 0.7 = fair, 0.9 = advantageous
                        let deal_quality = skill_result.skill_modifier;

                        // Existing trade logic with deal_quality factored in
                        // ...

                        let is_complete = task.progress >= task.duration;

                        if is_complete {
                            record_action_experience(
                                &mut world.humans.chunk_libraries[i],
                                &skill_result.chunks_used,
                                true, // Completed trade teaches regardless of outcome
                                world.current_tick,
                            );
                        }

                        is_complete
                    }
```

**Step 3: Run tests**

Run: `cargo test --lib simulation:: -v`
Expected: PASS

**Step 4: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat(simulation): wire skill checks to social actions"
```

---

## Task 9: Integration Tests

**Files:**
- Create: `tests/skill_action_integration.rs`

**Step 1: Create integration test file**

Create `tests/skill_action_integration.rs`:

```rust
//! Integration tests for chunking system wired to actions

use arc_citadel::actions::catalog::ActionId;
use arc_citadel::skills::{
    skill_check, refresh_attention, record_action_experience, spend_attention,
    ChunkId, ChunkLibrary, PersonalChunkState, SkillFailure,
};

/// Helper to create entity with specific chunk state
fn create_entity_with_skill(chunk_ids: &[ChunkId], encoding_depth: f32) -> ChunkLibrary {
    let mut lib = ChunkLibrary::new();
    lib.attention_budget = 1.0;

    for chunk_id in chunk_ids {
        lib.set_chunk(*chunk_id, PersonalChunkState {
            encoding_depth,
            repetition_count: (encoding_depth * 100.0) as u32,
            last_used_tick: 0,
            formation_tick: 0,
        });
    }

    lib
}

#[test]
fn test_novice_vs_master_attack() {
    // Novice has no combat chunks
    let novice = ChunkLibrary::new();
    let novice_result = skill_check(&novice, ActionId::Attack);

    // Master has practiced chunks
    let master = create_entity_with_skill(
        &[ChunkId::BasicSwing, ChunkId::BasicStance, ChunkId::AttackSequence],
        0.8,
    );
    let master_result = skill_check(&master, ActionId::Attack);

    // Both can execute (have attention)
    assert!(novice_result.can_execute);
    assert!(master_result.can_execute);

    // Master has higher skill modifier
    assert!(master_result.skill_modifier > novice_result.skill_modifier + 0.3);

    // Master pays less attention
    assert!(master_result.attention_cost < novice_result.attention_cost);
}

#[test]
fn test_exhausted_entity_overloads() {
    let mut lib = create_entity_with_skill(&[ChunkId::BasicSwing], 0.3);
    lib.attention_budget = 0.1; // Very tired

    let result = skill_check(&lib, ActionId::Attack);

    assert!(!result.can_execute);
    assert_eq!(result.failure_reason, Some(SkillFailure::AttentionOverload));
}

#[test]
fn test_attention_depletes_over_actions() {
    let mut lib = create_entity_with_skill(
        &[ChunkId::BasicSwing, ChunkId::BasicStance],
        0.5, // Moderate skill - moderate cost
    );
    lib.attention_budget = 1.0;

    // First action
    let result1 = skill_check(&lib, ActionId::Attack);
    assert!(result1.can_execute);
    spend_attention(&mut lib, result1.attention_cost);

    // Second action
    let result2 = skill_check(&lib, ActionId::Attack);
    assert!(result2.can_execute);
    spend_attention(&mut lib, result2.attention_cost);

    // Third action - may fail depending on cost
    let remaining = lib.attention_remaining();
    let result3 = skill_check(&lib, ActionId::Attack);

    // Either executes with low remaining, or overloads
    if result3.can_execute {
        assert!(remaining > result3.attention_cost);
    } else {
        assert!(remaining < result3.attention_cost);
    }
}

#[test]
fn test_instinctive_actions_no_skill_check() {
    let lib = ChunkLibrary::new(); // No chunks at all

    // Rest is instinctive
    let result = skill_check(&lib, ActionId::Rest);

    assert!(result.can_execute);
    assert_eq!(result.skill_modifier, 1.0);
    assert_eq!(result.attention_cost, 0.0);
}

#[test]
fn test_skill_improves_through_practice() {
    use arc_citadel::skills::process_learning;

    let mut lib = create_entity_with_skill(
        &[ChunkId::PhysicalEfficientGait],
        0.3, // Starting skill
    );
    lib.attention_budget = 1.0;

    let initial_depth = lib.get_chunk(ChunkId::PhysicalEfficientGait).unwrap().encoding_depth;

    // Simulate 20 successful movements
    for tick in 1..=20 {
        let result = skill_check(&lib, ActionId::MoveTo);
        if result.can_execute {
            spend_attention(&mut lib, result.attention_cost);
            record_action_experience(&mut lib, &result.chunks_used, true, tick);
        }
        // Refresh attention each "tick"
        refresh_attention(&mut lib, 0.0, 0.0, 0.0);
        // Process learning
        process_learning(&mut lib, tick);
    }

    let final_depth = lib.get_chunk(ChunkId::PhysicalEfficientGait).unwrap().encoding_depth;

    // Skill should have improved
    assert!(final_depth > initial_depth);
}

#[test]
fn test_fatigue_reduces_attention_budget() {
    let mut fresh = ChunkLibrary::new();
    let mut tired = ChunkLibrary::new();

    refresh_attention(&mut fresh, 0.0, 0.0, 0.0); // No fatigue
    refresh_attention(&mut tired, 0.8, 0.0, 0.0); // High fatigue

    assert!(fresh.attention_budget > tired.attention_budget);
    assert!(tired.attention_budget >= 0.2); // Minimum floor
}

#[test]
fn test_work_actions_use_physical_chunks() {
    let lib = create_entity_with_skill(
        &[ChunkId::PhysicalSustainedLabor, ChunkId::PhysicalPowerStance],
        0.6,
    );

    let gather_result = skill_check(&lib, ActionId::Gather);

    // Should use physical chunks
    assert!(gather_result.chunks_used.iter().any(|c| matches!(c,
        ChunkId::PhysicalSustainedLabor | ChunkId::PhysicalPowerStance
    )));
}

#[test]
fn test_social_actions_use_social_chunks() {
    let lib = create_entity_with_skill(
        &[ChunkId::SocialActiveListening, ChunkId::SocialBuildRapport, ChunkId::SocialNegotiateTerms, ChunkId::SocialReadReaction],
        0.5,
    );

    let trade_result = skill_check(&lib, ActionId::Trade);

    // Should use social chunks for trade
    assert!(!trade_result.chunks_used.is_empty());
    assert!(trade_result.skill_modifier > 0.3);
}
```

**Step 2: Run integration tests**

Run: `cargo test --test skill_action_integration -v`
Expected: PASS (9 tests)

**Step 3: Commit**

```bash
git add tests/skill_action_integration.rs
git commit -m "test: add skill-action integration tests"
```

---

## Verification Commands

After completing all tasks:

```bash
# All skill module tests
cargo test --lib skills:: -v

# All simulation tests
cargo test --lib simulation:: -v

# Integration tests
cargo test --test skill_action_integration -v
cargo test --test skills_integration -v

# Full test suite
cargo test

# Clippy
cargo clippy -- -D warnings
```

Expected: All tests pass, no clippy warnings.

---

## Summary

| Task | Files Created/Modified | Tests |
|------|----------------------|-------|
| 1 | `chunk_id.rs`, `definitions.rs` | Existing pass |
| 2 | `action_mapping.rs`, `mod.rs` | 5 |
| 3 | `integration.rs`, `mod.rs` | 7 |
| 4 | `tick.rs` | Existing pass |
| 5 | `tick.rs` | Existing pass |
| 6 | `tick.rs` | Existing pass |
| 7 | `tick.rs` | Existing pass |
| 8 | `tick.rs` | Existing pass |
| 9 | `skill_action_integration.rs` | 9 |

**Total new tests: 21**

---

## Future Extensions (Out of Scope)

1. Species modifiers for chunk learning rates
2. Phenotype ceilings on skill
3. Quality system for crafted items
4. Combat damage modified by skill
5. Social relationship changes from skill
