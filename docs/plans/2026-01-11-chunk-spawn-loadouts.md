# Chunk Spawn Loadouts Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Every entity spawns with appropriate chunks based on archetype and age, eliminating empty ChunkLibraries.

**Architecture:** Create `spawn_loadouts.rs` module with archetype-specific chunk generation. Wire to HumanArchetype::spawn(). Remove the "instinctive work" fallback that bypasses chunking for entities without chunks.

**Tech Stack:** Rust, rand crate for variance

---

## Context

### Current State
- `HumanArchetype::spawn()` at `src/entity/species/human.rs:117` uses `ChunkLibrary::with_basics()` which is EMPTY
- `src/skills/integration.rs:121-127` has fallback: entities without chunks work at 0.3x skill with 0 attention cost
- `src/simulation/tick.rs` has additional bypass: if `attention_cost == 0`, use `effective_skill = 1.0`

### Target State
- Every entity spawns with chunks appropriate to their archetype and age
- No empty ChunkLibraries ever exist
- The 0.3x instinctive fallback becomes unreachable (all entities have chunks)
- Skill differentiation emerges naturally from encoding depths

### Existing Infrastructure
- `ChunkId` enum at `src/skills/chunk_id.rs` has 137 chunks across 7 domains (already complete)
- `ChunkLibrary` at `src/skills/library.rs` has `set_chunk()`, preset constructors
- `PersonalChunkState` has `encoding_depth`, `repetition_count`, `last_used_tick`, `formation_tick`

---

## Task 1: Create EntityArchetype and Related Enums

**Files:**
- Create: `src/entity/archetype.rs`
- Modify: `src/entity/mod.rs`

**Step 1: Write the failing test**

Create test in `src/entity/archetype.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archetype_variants_exist() {
        let _peasant = EntityArchetype::Peasant;
        let _laborer = EntityArchetype::Laborer;
        let _craftsman = EntityArchetype::Craftsman { specialty: CraftSpecialty::Smithing };
        let _soldier = EntityArchetype::Soldier { training: TrainingLevel::Levy };
        let _noble = EntityArchetype::Noble;
        let _merchant = EntityArchetype::Merchant;
        let _scholar = EntityArchetype::Scholar;
        let _child = EntityArchetype::Child;
    }

    #[test]
    fn test_training_levels() {
        assert!(TrainingLevel::Levy.base_skill() < TrainingLevel::Elite.base_skill());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test archetype_variants_exist`
Expected: FAIL with "cannot find type `EntityArchetype`"

**Step 3: Write the implementation**

```rust
//! Entity archetype definitions for spawn loadout generation

use serde::{Deserialize, Serialize};

/// High-level entity role determining spawn chunk loadout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityArchetype {
    /// Rural farmer/villager - basic physical labor, no specialization
    Peasant,
    /// Construction/hauling worker - strong physical skills
    Laborer,
    /// Skilled tradesperson with specialty
    Craftsman { specialty: CraftSpecialty },
    /// Military personnel with training level
    Soldier { training: TrainingLevel },
    /// Aristocrat - social, leadership, some combat
    Noble,
    /// Trader - social, assessment skills
    Merchant,
    /// Educated person - knowledge, teaching
    Scholar,
    /// Young person (age < 16) - only universal chunks
    Child,
}

/// Craft specialization for craftsmen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CraftSpecialty {
    Smithing,
    Carpentry,
    Masonry,
    Cooking,
    Tailoring,
    Leatherwork,
}

/// Military training level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrainingLevel {
    /// Farmers with spears
    Levy,
    /// Part-time trained
    Militia,
    /// Professional soldiers
    Regular,
    /// Battle-hardened
    Veteran,
    /// Best of the best
    Elite,
}

impl TrainingLevel {
    /// Base skill level for this training (0.0 to 1.0)
    pub fn base_skill(&self) -> f32 {
        match self {
            Self::Levy => 0.2,
            Self::Militia => 0.35,
            Self::Regular => 0.5,
            Self::Veteran => 0.7,
            Self::Elite => 0.85,
        }
    }
}

impl Default for EntityArchetype {
    fn default() -> Self {
        Self::Peasant
    }
}
```

**Step 4: Export from mod.rs**

Add to `src/entity/mod.rs`:
```rust
pub mod archetype;
pub use archetype::{EntityArchetype, CraftSpecialty, TrainingLevel};
```

**Step 5: Run test to verify it passes**

Run: `cargo test archetype_variants_exist`
Expected: PASS

**Step 6: Commit**

```bash
git add src/entity/archetype.rs src/entity/mod.rs
git commit -m "feat(entity): add EntityArchetype, CraftSpecialty, TrainingLevel enums"
```

---

## Task 2: Create spawn_loadouts.rs with Universal Chunks

**Files:**
- Create: `src/skills/spawn_loadouts.rs`
- Modify: `src/skills/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::EntityArchetype;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn test_child_has_walking_chunks() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let library = generate_spawn_chunks(EntityArchetype::Child, 8, 0, &mut rng);

        // Children 5+ have basic walking chunks
        assert!(library.has_chunk(ChunkId::PhysEfficientGait));
        assert!(!library.chunks().is_empty());
    }

    #[test]
    fn test_adult_has_more_chunks_than_child() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let child = generate_spawn_chunks(EntityArchetype::Child, 8, 0, &mut rng);
        let adult = generate_spawn_chunks(EntityArchetype::Peasant, 25, 0, &mut rng);

        assert!(adult.chunks().len() > child.chunks().len());
    }

    #[test]
    fn test_no_empty_libraries() {
        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let archetypes = [
            EntityArchetype::Peasant,
            EntityArchetype::Child,
            EntityArchetype::Laborer,
            EntityArchetype::Noble,
            EntityArchetype::Merchant,
            EntityArchetype::Scholar,
        ];

        for archetype in archetypes {
            let library = generate_spawn_chunks(archetype, 20, 0, &mut rng);
            assert!(!library.chunks().is_empty(), "{:?} should have chunks", archetype);
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test child_has_walking_chunks`
Expected: FAIL with "cannot find function `generate_spawn_chunks`"

**Step 3: Write the implementation**

```rust
//! Spawn loadout generation for entity chunk libraries
//!
//! Every entity spawns with chunks appropriate to their archetype and age.
//! No entity should ever have an empty ChunkLibrary.

use crate::entity::{CraftSpecialty, EntityArchetype, TrainingLevel};
use crate::skills::{ChunkId, ChunkLibrary, PersonalChunkState};
use rand::Rng;
use std::collections::HashMap;

/// Generate starting chunks based on entity archetype and age.
///
/// # Arguments
/// * `archetype` - The entity's role/profession
/// * `age` - Age in years (affects skill depth)
/// * `tick` - Current simulation tick (for formation_tick)
/// * `rng` - Random number generator for variance
///
/// # Returns
/// A ChunkLibrary with appropriate chunks. Never empty.
pub fn generate_spawn_chunks(
    archetype: EntityArchetype,
    age: u32,
    tick: u64,
    rng: &mut impl Rng,
) -> ChunkLibrary {
    let mut library = ChunkLibrary::new();

    // === UNIVERSAL CHUNKS (everyone who can walk) ===
    add_universal_chunks(&mut library, age, tick, rng);

    // === ARCHETYPE-SPECIFIC CHUNKS ===
    match archetype {
        EntityArchetype::Peasant => add_peasant_chunks(&mut library, age, tick, rng),
        EntityArchetype::Laborer => add_laborer_chunks(&mut library, age, tick, rng),
        EntityArchetype::Craftsman { specialty } => {
            add_craftsman_chunks(&mut library, specialty, age, tick, rng)
        }
        EntityArchetype::Soldier { training } => {
            add_soldier_chunks(&mut library, training, age, tick, rng)
        }
        EntityArchetype::Noble => add_noble_chunks(&mut library, age, tick, rng),
        EntityArchetype::Merchant => add_merchant_chunks(&mut library, age, tick, rng),
        EntityArchetype::Scholar => add_scholar_chunks(&mut library, age, tick, rng),
        EntityArchetype::Child => {
            // Children only have universal chunks (already added above)
        }
    }

    library
}

/// Add universal physical chunks everyone has from years of living
fn add_universal_chunks(library: &mut ChunkLibrary, age: u32, tick: u64, rng: &mut impl Rng) {
    if age >= 5 {
        // Walking - even young children have this
        set_chunk_with_variance(library, ChunkId::PhysEfficientGait, 0.5, tick, rng);
    }

    if age >= 12 {
        // Adolescent+ has more physical automation
        set_chunk_with_variance(library, ChunkId::PhysEfficientGait, 0.6, tick, rng);
        set_chunk_with_variance(library, ChunkId::PhysQuietMovement, 0.3, tick, rng);
    }

    if age >= 16 {
        // Adult baseline - deeper encoding from more years
        set_chunk_with_variance(library, ChunkId::PhysEfficientGait, 0.7, tick, rng);
        set_chunk_with_variance(library, ChunkId::PhysPowerStance, 0.4, tick, rng);
    }
}

/// Set a chunk with random variance around base depth
fn set_chunk_with_variance(
    library: &mut ChunkLibrary,
    chunk_id: ChunkId,
    base_depth: f32,
    tick: u64,
    rng: &mut impl Rng,
) {
    let variance = rng.gen_range(-0.05..0.05);
    let depth = (base_depth + variance).clamp(0.05, 0.95);
    let reps = (depth * 500.0) as u32;

    library.set_chunk(
        chunk_id,
        PersonalChunkState {
            encoding_depth: depth,
            repetition_count: reps,
            last_used_tick: tick,
            formation_tick: tick.saturating_sub((depth * 10000.0) as u64),
        },
    );
}

// Placeholder functions - will be implemented in subsequent tasks
fn add_peasant_chunks(_library: &mut ChunkLibrary, _age: u32, _tick: u64, _rng: &mut impl Rng) {}
fn add_laborer_chunks(_library: &mut ChunkLibrary, _age: u32, _tick: u64, _rng: &mut impl Rng) {}
fn add_craftsman_chunks(_library: &mut ChunkLibrary, _specialty: CraftSpecialty, _age: u32, _tick: u64, _rng: &mut impl Rng) {}
fn add_soldier_chunks(_library: &mut ChunkLibrary, _training: TrainingLevel, _age: u32, _tick: u64, _rng: &mut impl Rng) {}
fn add_noble_chunks(_library: &mut ChunkLibrary, _age: u32, _tick: u64, _rng: &mut impl Rng) {}
fn add_merchant_chunks(_library: &mut ChunkLibrary, _age: u32, _tick: u64, _rng: &mut impl Rng) {}
fn add_scholar_chunks(_library: &mut ChunkLibrary, _age: u32, _tick: u64, _rng: &mut impl Rng) {}
```

**Step 4: Export from mod.rs**

Add to `src/skills/mod.rs`:
```rust
pub mod spawn_loadouts;
pub use spawn_loadouts::generate_spawn_chunks;
```

**Step 5: Run test to verify it passes**

Run: `cargo test child_has_walking_chunks`
Expected: PASS

**Step 6: Commit**

```bash
git add src/skills/spawn_loadouts.rs src/skills/mod.rs
git commit -m "feat(skills): add spawn_loadouts module with universal chunks"
```

---

## Task 3: Implement Peasant and Laborer Chunk Loadouts

**Files:**
- Modify: `src/skills/spawn_loadouts.rs`

**Step 1: Write the failing test**

Add to tests in `spawn_loadouts.rs`:

```rust
#[test]
fn test_peasant_has_labor_chunks() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let library = generate_spawn_chunks(EntityArchetype::Peasant, 30, 0, &mut rng);

    assert!(library.has_chunk(ChunkId::PhysSustainedLabor));
    assert!(library.has_chunk(ChunkId::SocialActiveListening));
}

#[test]
fn test_laborer_stronger_physical_than_peasant() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let peasant = generate_spawn_chunks(EntityArchetype::Peasant, 30, 0, &mut rng);
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let laborer = generate_spawn_chunks(EntityArchetype::Laborer, 30, 0, &mut rng);

    let peasant_labor = peasant.get_chunk(ChunkId::PhysSustainedLabor).unwrap();
    let laborer_labor = laborer.get_chunk(ChunkId::PhysSustainedLabor).unwrap();

    assert!(laborer_labor.encoding_depth > peasant_labor.encoding_depth);
}

#[test]
fn test_older_peasant_more_skilled() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let young = generate_spawn_chunks(EntityArchetype::Peasant, 18, 0, &mut rng);
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let old = generate_spawn_chunks(EntityArchetype::Peasant, 45, 0, &mut rng);

    let young_labor = young.get_chunk(ChunkId::PhysSustainedLabor).unwrap();
    let old_labor = old.get_chunk(ChunkId::PhysSustainedLabor).unwrap();

    assert!(old_labor.encoding_depth > young_labor.encoding_depth);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test peasant_has_labor_chunks`
Expected: FAIL (peasant function is a stub)

**Step 3: Write the implementation**

Replace the placeholder functions:

```rust
/// Peasants have physical labor skills but not specialized craft
fn add_peasant_chunks(library: &mut ChunkLibrary, age: u32, tick: u64, rng: &mut impl Rng) {
    let years_working = age.saturating_sub(10) as f32;
    let experience = (years_working / 30.0).min(1.0); // Caps at ~40 years old

    // Physical labor - moderate skill from field work
    set_chunk_with_variance(
        library,
        ChunkId::PhysSustainedLabor,
        0.3 + experience * 0.4,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::PhysHeavyLifting,
        0.2 + experience * 0.3,
        tick,
        rng,
    );

    // Rough terrain from rural life
    set_chunk_with_variance(
        library,
        ChunkId::PhysRoughTerrainTravel,
        0.2 + experience * 0.3,
        tick,
        rng,
    );

    // Basic social (village interactions)
    set_chunk_with_variance(
        library,
        ChunkId::SocialActiveListening,
        0.3 + experience * 0.2,
        tick,
        rng,
    );

    // Basic craft (everyone can do rough work)
    set_chunk_with_variance(
        library,
        ChunkId::CraftBasicCut,
        0.2 + experience * 0.2,
        tick,
        rng,
    );
}

/// Laborers have better physical skills than peasants
fn add_laborer_chunks(library: &mut ChunkLibrary, age: u32, tick: u64, rng: &mut impl Rng) {
    let years_working = age.saturating_sub(12) as f32;
    let experience = (years_working / 25.0).min(1.0);

    // Strong physical labor
    set_chunk_with_variance(
        library,
        ChunkId::PhysSustainedLabor,
        0.4 + experience * 0.4,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::PhysHeavyLifting,
        0.4 + experience * 0.4,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::PhysDistanceRunning,
        0.3 + experience * 0.3,
        tick,
        rng,
    );

    // Basic craft from construction work
    set_chunk_with_variance(
        library,
        ChunkId::CraftBasicMeasure,
        0.3 + experience * 0.3,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::CraftBasicCut,
        0.3 + experience * 0.3,
        tick,
        rng,
    );
    set_chunk_with_variance(
        library,
        ChunkId::CraftBasicJoin,
        0.2 + experience * 0.3,
        tick,
        rng,
    );
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test peasant_has_labor_chunks && cargo test laborer_stronger`
Expected: PASS

**Step 5: Commit**

```bash
git add src/skills/spawn_loadouts.rs
git commit -m "feat(skills): implement peasant and laborer spawn loadouts"
```

---

## Task 4: Implement Craftsman Chunk Loadouts

**Files:**
- Modify: `src/skills/spawn_loadouts.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_smith_has_smithing_chunks() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let smith = generate_spawn_chunks(
        EntityArchetype::Craftsman { specialty: CraftSpecialty::Smithing },
        30,
        0,
        &mut rng,
    );

    assert!(smith.has_chunk(ChunkId::CraftBasicHeatCycle));
    assert!(smith.has_chunk(ChunkId::CraftBasicHammerWork));
}

#[test]
fn test_master_smith_has_advanced_chunks() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let master = generate_spawn_chunks(
        EntityArchetype::Craftsman { specialty: CraftSpecialty::Smithing },
        50,
        0,
        &mut rng,
    );

    // 36 years experience should unlock advanced chunks
    assert!(master.has_chunk(ChunkId::CraftForgeSword));
}

#[test]
fn test_carpenter_has_carpentry_chunks() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let carpenter = generate_spawn_chunks(
        EntityArchetype::Craftsman { specialty: CraftSpecialty::Carpentry },
        30,
        0,
        &mut rng,
    );

    assert!(carpenter.has_chunk(ChunkId::CraftShapeWood));
    assert!(!carpenter.has_chunk(ChunkId::CraftBasicHeatCycle)); // No smithing
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test smith_has_smithing_chunks`
Expected: FAIL

**Step 3: Write the implementation**

```rust
/// Craftsmen have deep specialty skills
fn add_craftsman_chunks(
    library: &mut ChunkLibrary,
    specialty: CraftSpecialty,
    age: u32,
    tick: u64,
    rng: &mut impl Rng,
) {
    let years_training = age.saturating_sub(14) as f32; // Apprentice at 14
    let experience = (years_training / 20.0).min(1.0); // Master at ~34

    // All craftsmen have basic craft chunks
    set_chunk_with_variance(library, ChunkId::CraftBasicMeasure, 0.4 + experience * 0.4, tick, rng);
    set_chunk_with_variance(library, ChunkId::CraftBasicCut, 0.4 + experience * 0.4, tick, rng);
    set_chunk_with_variance(library, ChunkId::CraftBasicJoin, 0.3 + experience * 0.4, tick, rng);

    // Physical from manual work
    set_chunk_with_variance(library, ChunkId::PhysSustainedLabor, 0.4 + experience * 0.2, tick, rng);

    // Specialty-specific chunks
    match specialty {
        CraftSpecialty::Smithing => {
            set_chunk_with_variance(library, ChunkId::CraftBasicHeatCycle, 0.3 + experience * 0.5, tick, rng);
            set_chunk_with_variance(library, ChunkId::CraftBasicHammerWork, 0.3 + experience * 0.5, tick, rng);
            set_chunk_with_variance(library, ChunkId::CraftDrawOutMetal, 0.2 + experience * 0.5, tick, rng);
            set_chunk_with_variance(library, ChunkId::CraftUpsetMetal, 0.2 + experience * 0.5, tick, rng);

            if experience > 0.3 {
                set_chunk_with_variance(library, ChunkId::CraftBasicWeld, experience * 0.6, tick, rng);
                set_chunk_with_variance(library, ChunkId::CraftForgeKnife, (experience - 0.3) * 0.8, tick, rng);
            }
            if experience > 0.5 {
                set_chunk_with_variance(library, ChunkId::CraftForgeToolHead, (experience - 0.5) * 1.0, tick, rng);
                set_chunk_with_variance(library, ChunkId::CraftForgeSword, (experience - 0.5) * 0.8, tick, rng);
            }
            if experience > 0.8 {
                set_chunk_with_variance(library, ChunkId::CraftPatternWeld, (experience - 0.8) * 1.5, tick, rng);
                set_chunk_with_variance(library, ChunkId::CraftForgeArmor, (experience - 0.8) * 1.2, tick, rng);
            }
        }
        CraftSpecialty::Carpentry => {
            set_chunk_with_variance(library, ChunkId::CraftShapeWood, 0.3 + experience * 0.5, tick, rng);
            set_chunk_with_variance(library, ChunkId::CraftFinishSurface, 0.2 + experience * 0.4, tick, rng);

            if experience > 0.3 {
                set_chunk_with_variance(library, ChunkId::CraftBuildFurniture, (experience - 0.3) * 0.9, tick, rng);
            }
            if experience > 0.6 {
                set_chunk_with_variance(library, ChunkId::CraftBuildStructure, (experience - 0.6) * 1.0, tick, rng);
            }
        }
        CraftSpecialty::Masonry => {
            // Stone work uses similar base chunks plus specialty
            set_chunk_with_variance(library, ChunkId::PhysHeavyLifting, 0.4 + experience * 0.4, tick, rng);
            if experience > 0.4 {
                set_chunk_with_variance(library, ChunkId::CraftBuildStructure, (experience - 0.4) * 0.9, tick, rng);
            }
        }
        CraftSpecialty::Cooking => {
            set_chunk_with_variance(library, ChunkId::CraftBasicHeatCycle, 0.3 + experience * 0.4, tick, rng);
            // Cooks use similar heat control as smiths but different application
        }
        CraftSpecialty::Tailoring => {
            set_chunk_with_variance(library, ChunkId::CraftSewGarment, 0.2 + experience * 0.6, tick, rng);
        }
        CraftSpecialty::Leatherwork => {
            // Similar to tailoring with different materials
            set_chunk_with_variance(library, ChunkId::CraftSewGarment, 0.2 + experience * 0.5, tick, rng);
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test smith_has && cargo test carpenter_has`
Expected: PASS

**Step 5: Commit**

```bash
git add src/skills/spawn_loadouts.rs
git commit -m "feat(skills): implement craftsman spawn loadouts with specialties"
```

---

## Task 5: Implement Soldier Chunk Loadouts

**Files:**
- Modify: `src/skills/spawn_loadouts.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_levy_has_basic_combat() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let levy = generate_spawn_chunks(
        EntityArchetype::Soldier { training: TrainingLevel::Levy },
        25,
        0,
        &mut rng,
    );

    assert!(levy.has_chunk(ChunkId::BasicStance));
    assert!(levy.has_chunk(ChunkId::BasicSwing));
}

#[test]
fn test_veteran_deeper_than_levy() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let levy = generate_spawn_chunks(
        EntityArchetype::Soldier { training: TrainingLevel::Levy },
        25,
        0,
        &mut rng,
    );
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let veteran = generate_spawn_chunks(
        EntityArchetype::Soldier { training: TrainingLevel::Veteran },
        25,
        0,
        &mut rng,
    );

    let levy_stance = levy.get_chunk(ChunkId::BasicStance).unwrap();
    let veteran_stance = veteran.get_chunk(ChunkId::BasicStance).unwrap();

    assert!(veteran_stance.encoding_depth > levy_stance.encoding_depth);
}

#[test]
fn test_elite_has_advanced_combat() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let elite = generate_spawn_chunks(
        EntityArchetype::Soldier { training: TrainingLevel::Elite },
        30,
        0,
        &mut rng,
    );

    assert!(elite.has_chunk(ChunkId::EngageMelee));
    assert!(elite.has_chunk(ChunkId::HandleFlanking));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test levy_has_basic_combat`
Expected: FAIL

**Step 3: Write the implementation**

```rust
/// Soldiers have combat skills based on training level
fn add_soldier_chunks(
    library: &mut ChunkLibrary,
    training: TrainingLevel,
    age: u32,
    tick: u64,
    rng: &mut impl Rng,
) {
    let base = training.base_skill();
    let variance = rng.gen_range(-0.1..0.1);

    // Combat fundamentals - all soldiers have these
    set_chunk_with_variance(library, ChunkId::BasicStance, base, tick, rng);
    set_chunk_with_variance(library, ChunkId::BasicSwing, base, tick, rng);
    set_chunk_with_variance(library, ChunkId::BasicBlock, base, tick, rng);

    // Physical fitness - soldiers are fit
    set_chunk_with_variance(library, ChunkId::PhysSustainedLabor, base + 0.1, tick, rng);
    set_chunk_with_variance(library, ChunkId::PhysDistanceRunning, base, tick, rng);

    // Militia+ has weapon coordination
    if base >= 0.35 {
        set_chunk_with_variance(library, ChunkId::AttackSequence, base - 0.1, tick, rng);
    }

    // Regular+ has defensive sequences
    if base >= 0.5 {
        set_chunk_with_variance(library, ChunkId::DefendSequence, base - 0.15, tick, rng);
        set_chunk_with_variance(library, ChunkId::Riposte, base - 0.2, tick, rng);
    }

    // Veteran+ has engagement skills
    if base >= 0.7 {
        set_chunk_with_variance(library, ChunkId::EngageMelee, base - 0.3, tick, rng);
    }

    // Elite has mastery
    if base >= 0.85 {
        set_chunk_with_variance(library, ChunkId::HandleFlanking, base - 0.35, tick, rng);
    }

    // Leadership chunks for higher ranks
    if base >= 0.5 {
        set_chunk_with_variance(library, ChunkId::LeadCommandPresence, base - 0.2, tick, rng);
    }
    if base >= 0.7 {
        set_chunk_with_variance(library, ChunkId::LeadIssueCommand, base - 0.3, tick, rng);
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test levy_has && cargo test veteran_deeper && cargo test elite_has`
Expected: PASS

**Step 5: Commit**

```bash
git add src/skills/spawn_loadouts.rs
git commit -m "feat(skills): implement soldier spawn loadouts with training levels"
```

---

## Task 6: Implement Noble, Merchant, Scholar Chunk Loadouts

**Files:**
- Modify: `src/skills/spawn_loadouts.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_noble_has_social_and_combat() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let noble = generate_spawn_chunks(EntityArchetype::Noble, 30, 0, &mut rng);

    assert!(noble.has_chunk(ChunkId::SocialProjectConfidence));
    assert!(noble.has_chunk(ChunkId::LeadCommandPresence));
    assert!(noble.has_chunk(ChunkId::BasicStance)); // Combat training
}

#[test]
fn test_merchant_has_social_and_assessment() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let merchant = generate_spawn_chunks(EntityArchetype::Merchant, 35, 0, &mut rng);

    assert!(merchant.has_chunk(ChunkId::SocialBuildRapport));
    assert!(merchant.has_chunk(ChunkId::SocialNegotiateTerms));
}

#[test]
fn test_scholar_has_knowledge_chunks() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let scholar = generate_spawn_chunks(EntityArchetype::Scholar, 40, 0, &mut rng);

    assert!(scholar.has_chunk(ChunkId::KnowFluentReading));
    assert!(scholar.has_chunk(ChunkId::KnowFluentWriting));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test noble_has_social`
Expected: FAIL

**Step 3: Write the implementation**

```rust
/// Nobles have social, leadership, and some combat skills
fn add_noble_chunks(library: &mut ChunkLibrary, age: u32, tick: u64, rng: &mut impl Rng) {
    let years_training = age.saturating_sub(8) as f32; // Nobles educated early
    let experience = (years_training / 25.0).min(1.0);

    // Social skills (primary noble domain)
    set_chunk_with_variance(library, ChunkId::SocialProjectConfidence, 0.4 + experience * 0.4, tick, rng);
    set_chunk_with_variance(library, ChunkId::SocialBuildRapport, 0.3 + experience * 0.4, tick, rng);
    set_chunk_with_variance(library, ChunkId::SocialReadReaction, 0.2 + experience * 0.5, tick, rng);

    // Leadership (nobles expected to lead)
    set_chunk_with_variance(library, ChunkId::LeadCommandPresence, 0.3 + experience * 0.4, tick, rng);
    set_chunk_with_variance(library, ChunkId::LeadClearOrder, 0.2 + experience * 0.4, tick, rng);

    // Combat training (nobles learn to fight)
    set_chunk_with_variance(library, ChunkId::BasicStance, 0.3 + experience * 0.3, tick, rng);
    set_chunk_with_variance(library, ChunkId::BasicSwing, 0.3 + experience * 0.3, tick, rng);

    // Riding (essential noble skill)
    set_chunk_with_variance(library, ChunkId::PhysHorseControl, 0.3 + experience * 0.4, tick, rng);

    // Advanced social for experienced nobles
    if experience > 0.5 {
        set_chunk_with_variance(library, ChunkId::SocialNegotiateTerms, (experience - 0.5) * 0.8, tick, rng);
        set_chunk_with_variance(library, ChunkId::SocialProjectAuthority, (experience - 0.5) * 0.8, tick, rng);
        set_chunk_with_variance(library, ChunkId::LeadIssueCommand, (experience - 0.5) * 0.7, tick, rng);
    }
}

/// Merchants have social and assessment skills
fn add_merchant_chunks(library: &mut ChunkLibrary, age: u32, tick: u64, rng: &mut impl Rng) {
    let years_trading = age.saturating_sub(14) as f32;
    let experience = (years_trading / 25.0).min(1.0);

    // Social skills (merchant bread and butter)
    set_chunk_with_variance(library, ChunkId::SocialBuildRapport, 0.4 + experience * 0.4, tick, rng);
    set_chunk_with_variance(library, ChunkId::SocialReadReaction, 0.3 + experience * 0.5, tick, rng);
    set_chunk_with_variance(library, ChunkId::SocialNegotiateTerms, 0.3 + experience * 0.5, tick, rng);

    // Travel (merchants move around)
    set_chunk_with_variance(library, ChunkId::PhysRoughTerrainTravel, 0.3 + experience * 0.3, tick, rng);

    // Basic literacy for contracts
    set_chunk_with_variance(library, ChunkId::KnowFluentReading, 0.3 + experience * 0.3, tick, rng);
    set_chunk_with_variance(library, ChunkId::KnowArithmetic, 0.4 + experience * 0.4, tick, rng);

    // Advanced social for experienced merchants
    if experience > 0.5 {
        set_chunk_with_variance(library, ChunkId::SocialPersuade, (experience - 0.5) * 0.8, tick, rng);
    }
    if experience > 0.7 {
        set_chunk_with_variance(library, ChunkId::SocialDeceive, (experience - 0.7) * 0.6, tick, rng);
    }
}

/// Scholars have knowledge and teaching skills
fn add_scholar_chunks(library: &mut ChunkLibrary, age: u32, tick: u64, rng: &mut impl Rng) {
    let years_study = age.saturating_sub(10) as f32;
    let experience = (years_study / 30.0).min(1.0);

    // Knowledge chunks (primary domain)
    set_chunk_with_variance(library, ChunkId::KnowFluentReading, 0.4 + experience * 0.4, tick, rng);
    set_chunk_with_variance(library, ChunkId::KnowFluentWriting, 0.3 + experience * 0.5, tick, rng);
    set_chunk_with_variance(library, ChunkId::KnowMemorization, 0.3 + experience * 0.4, tick, rng);
    set_chunk_with_variance(library, ChunkId::KnowArithmetic, 0.4 + experience * 0.4, tick, rng);

    // Research and teaching develop with experience
    if experience > 0.3 {
        set_chunk_with_variance(library, ChunkId::KnowResearchSource, (experience - 0.3) * 0.9, tick, rng);
        set_chunk_with_variance(library, ChunkId::KnowTeachConcept, (experience - 0.3) * 0.8, tick, rng);
    }

    if experience > 0.5 {
        set_chunk_with_variance(library, ChunkId::KnowComposeDocument, (experience - 0.5) * 0.8, tick, rng);
        set_chunk_with_variance(library, ChunkId::KnowAnalyzeText, (experience - 0.5) * 0.9, tick, rng);
    }

    if experience > 0.7 {
        set_chunk_with_variance(library, ChunkId::KnowSynthesizeSources, (experience - 0.7) * 1.0, tick, rng);
    }

    // Scholars are not physical - they keep only basic universal chunks
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test noble_has && cargo test merchant_has && cargo test scholar_has`
Expected: PASS

**Step 5: Commit**

```bash
git add src/skills/spawn_loadouts.rs
git commit -m "feat(skills): implement noble, merchant, scholar spawn loadouts"
```

---

## Task 7: Wire Spawn Loadouts to HumanArchetype::spawn()

**Files:**
- Modify: `src/entity/species/human.rs`

**Step 1: Write the failing test**

Add test in `src/entity/species/human.rs`:

```rust
#[test]
fn test_spawn_creates_chunks() {
    let mut archetype = HumanArchetype::new();
    let id = EntityId(1);

    archetype.spawn_with_archetype(
        id,
        "Test".to_string(),
        0,
        crate::entity::EntityArchetype::Peasant,
        25,
    );

    let idx = archetype.index_of(id).unwrap();
    assert!(!archetype.chunk_libraries[idx].chunks().is_empty());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test spawn_creates_chunks`
Expected: FAIL with "no method named `spawn_with_archetype`"

**Step 3: Write the implementation**

Add to `HumanArchetype`:

```rust
use crate::entity::{EntityArchetype, CraftSpecialty, TrainingLevel};
use crate::skills::generate_spawn_chunks;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

impl HumanArchetype {
    /// Spawn a new entity with chunks based on archetype and age
    pub fn spawn_with_archetype(
        &mut self,
        id: EntityId,
        name: String,
        tick: Tick,
        archetype: EntityArchetype,
        age: u32,
    ) {
        // Create RNG seeded from entity ID for reproducibility
        let mut rng = ChaCha8Rng::seed_from_u64(id.0 as u64);

        self.ids.push(id);
        self.names.push(name);
        self.birth_ticks.push(tick.saturating_sub((age as u64) * 365 * 24)); // Approximate birth
        self.positions.push(Vec2::default());
        self.velocities.push(Vec2::default());
        self.body_states.push(BodyState::default());
        self.needs.push(Needs::default());
        self.thoughts.push(ThoughtBuffer::new());
        self.values.push(HumanValues::default());
        self.task_queues.push(TaskQueue::new());
        self.alive.push(true);
        self.social_memories.push(SocialMemory::new());
        self.event_buffers.push(EventBuffer::default());
        self.building_skills.push(0.0); // Deprecated, use chunk_libraries
        self.combat_states.push(CombatState::default());
        self.assigned_houses.push(None);
        self.chunk_libraries.push(generate_spawn_chunks(archetype, age, tick, &mut rng));
    }
}
```

**Step 4: Update existing spawn() to use Peasant default**

```rust
/// Spawn a new entity with default (Peasant) archetype
/// Use spawn_with_archetype() for specific archetypes
pub fn spawn(&mut self, id: EntityId, name: String, tick: Tick) {
    self.spawn_with_archetype(id, name, tick, EntityArchetype::Peasant, 25);
}
```

**Step 5: Run test to verify it passes**

Run: `cargo test spawn_creates_chunks`
Expected: PASS

**Step 6: Commit**

```bash
git add src/entity/species/human.rs
git commit -m "feat(entity): wire spawn_with_archetype to generate chunks"
```

---

## Task 8: Remove the Empty Chunk Bypass in tick.rs

**Files:**
- Modify: `src/simulation/tick.rs`

**Step 1: Identify bypass patterns**

Search for patterns like `if attention_cost == 0.0 { effective_skill = 1.0 }` in tick.rs

**Step 2: Write the failing test**

Create `tests/skill_spawn_loadouts.rs`:

```rust
//! Integration tests for spawn loadouts

use arc_citadel::entity::{EntityArchetype, CraftSpecialty};
use arc_citadel::skills::{generate_spawn_chunks, skill_check, spend_attention};
use arc_citadel::actions::catalog::ActionId;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

#[test]
fn test_peasant_builds_with_skill_check() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut library = generate_spawn_chunks(EntityArchetype::Peasant, 25, 0, &mut rng);
    library.attention_budget = 1.0;

    let result = skill_check(&library, ActionId::Build);

    // Peasant has chunks, so skill_check should use them
    assert!(result.can_execute);
    assert!(result.attention_cost > 0.0, "Should use attention for chunks");
    assert!(result.skill_modifier > 0.1, "Should have skill from chunks");
    assert!(result.skill_modifier < 1.0, "Should not be perfect");
}

#[test]
fn test_carpenter_builds_better_than_peasant() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut peasant = generate_spawn_chunks(EntityArchetype::Peasant, 30, 0, &mut rng);
    peasant.attention_budget = 1.0;

    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut carpenter = generate_spawn_chunks(
        EntityArchetype::Craftsman { specialty: CraftSpecialty::Carpentry },
        30,
        0,
        &mut rng,
    );
    carpenter.attention_budget = 1.0;

    let peasant_result = skill_check(&peasant, ActionId::Build);
    let carpenter_result = skill_check(&carpenter, ActionId::Build);

    assert!(
        carpenter_result.skill_modifier > peasant_result.skill_modifier,
        "Carpenter {} should build better than peasant {}",
        carpenter_result.skill_modifier,
        peasant_result.skill_modifier
    );
}

#[test]
fn test_chunking_always_affects_actions() {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut library = generate_spawn_chunks(EntityArchetype::Peasant, 25, 0, &mut rng);
    library.attention_budget = 1.0;
    library.attention_spent = 0.0;

    let result = skill_check(&library, ActionId::Build);
    assert!(result.can_execute);

    // Spend attention
    spend_attention(&mut library, result.attention_cost);

    // Attention should have been spent
    assert!(
        library.attention_spent > 0.0,
        "Chunking system should have spent attention"
    );
}
```

**Step 3: Run test to verify it fails**

Run: `cargo test peasant_builds_with_skill_check`
Expected: May pass or fail depending on current action_mapping

**Step 4: Remove bypass patterns in tick.rs**

Find and modify patterns like:

```rust
// BEFORE (bypass):
let effective_skill = if skill_result.can_execute {
    if skill_result.attention_cost > 0.0 {
        spend_attention(...);
        skill_result.skill_modifier
    } else {
        // No attention cost = no chunks = work instinctively at normal rate
        1.0  // <-- THIS IS THE BYPASS
    }
} else {
    0.5
};

// AFTER (no bypass - always use skill_modifier):
let effective_skill = if skill_result.can_execute {
    spend_attention(..., skill_result.attention_cost);
    skill_result.skill_modifier
} else {
    // Exhausted: work at reduced efficiency
    0.5
};
```

Apply this change to all work actions: Build, Gather, Craft, Repair

**Step 5: Run test to verify it passes**

Run: `cargo test peasant_builds && cargo test carpenter_builds && cargo test chunking_always`
Expected: PASS

**Step 6: Commit**

```bash
git add src/simulation/tick.rs tests/skill_spawn_loadouts.rs
git commit -m "fix(simulation): remove empty chunk bypass, always use skill_modifier"
```

---

## Task 9: Update Action Mapping for Work Actions

**Files:**
- Modify: `src/skills/action_mapping.rs`

**Step 1: Review current mapping**

Read `src/skills/action_mapping.rs` to see what chunks are mapped to Build, Gather, Craft, Repair.

**Step 2: Ensure work actions map to chunks peasants have**

The mapping should include chunks that peasants spawn with:
- `PhysSustainedLabor` (peasants have this)
- `CraftBasicCut` (peasants have this)
- `CraftBasicMeasure` (laborers have this)

Update mappings if needed:

```rust
ActionId::Build => &[
    ChunkId::PhysSustainedLabor,
    ChunkId::CraftBasicMeasure,
    ChunkId::CraftBasicCut,
    ChunkId::CraftBasicJoin,
],
ActionId::Gather => &[
    ChunkId::PhysSustainedLabor,
],
ActionId::Craft => &[
    ChunkId::CraftBasicMeasure,
    ChunkId::CraftBasicCut,
    ChunkId::CraftBasicJoin,
],
```

**Step 3: Run all tests**

Run: `cargo test`
Expected: PASS

**Step 4: Commit**

```bash
git add src/skills/action_mapping.rs
git commit -m "fix(skills): update action mapping to include peasant chunks"
```

---

## Task 10: Run Full Test Suite and Fix Regressions

**Files:**
- Various (fix any regressions)

**Step 1: Run full test suite**

Run: `cargo test`

**Step 2: Fix any failing tests**

Tests that may need updating:
- `tests/city_integration.rs` - building times may change
- `tests/skill_action_integration.rs` - novice behavior changes
- `tests/skill_differentiation.rs` - no more empty libraries

**Step 3: Run tests again**

Run: `cargo test`
Expected: PASS

**Step 4: Commit**

```bash
git add -A
git commit -m "fix: update tests for spawn loadouts system"
```

---

## Validation Checklist

After completing all tasks, verify:

- [ ] `cargo test` passes
- [ ] No entity spawns with empty ChunkLibrary
- [ ] Peasants can build (have relevant chunks)
- [ ] Craftsmen build better than peasants
- [ ] Soldiers fight better than peasants
- [ ] Scholars have knowledge chunks
- [ ] Age affects chunk encoding depth
- [ ] All work actions use attention and skill_modifier

---

## Files Summary

| File | Action |
|------|--------|
| `src/entity/archetype.rs` | CREATE |
| `src/entity/mod.rs` | MODIFY |
| `src/skills/spawn_loadouts.rs` | CREATE |
| `src/skills/mod.rs` | MODIFY |
| `src/entity/species/human.rs` | MODIFY |
| `src/simulation/tick.rs` | MODIFY |
| `src/skills/action_mapping.rs` | MODIFY |
| `tests/skill_spawn_loadouts.rs` | CREATE |

---

*Every entity has a history. That history is encoded in their chunks.*
