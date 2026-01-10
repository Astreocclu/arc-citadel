# Hierarchical Chunking Skill System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a cognitive chunking skill system where skill mastery frees attention bandwidth rather than adding numeric bonuses.

**Architecture:** Entities have a ChunkLibrary storing per-chunk encoding_depth (0.0-1.0). Higher depth = lower attention cost. Combat actions consume attention budget. Fresh conscripts spend all attention on basic swing; masters execute complex sequences "for free."

**Tech Stack:** Rust, no external dependencies. Follows existing SoA archetype pattern.

---

## Overview

| Task | Component | Files |
|------|-----------|-------|
| 1 | Core Types | `src/skills/mod.rs`, `src/skills/chunk_id.rs` |
| 2 | Context Tags | `src/skills/context.rs` |
| 3 | Chunk Definitions | `src/skills/definitions.rs` |
| 4 | Per-Entity State | `src/skills/library.rs` |
| 5 | Attention Budget | `src/skills/attention.rs` |
| 6 | Action Resolution | `src/skills/resolution.rs` |
| 7 | Learning System | `src/skills/learning.rs` |
| 8 | SkillLevel Bridge | `src/combat/skill.rs` (modify) |
| 9 | Archetype Integration | `src/entity/species/human.rs` (modify) |
| 10 | Integration Tests | `tests/skills_integration.rs` |

---

## Task 1: Core Types & Module Structure

**Files:**
- Create: `src/skills/mod.rs`
- Create: `src/skills/chunk_id.rs`
- Modify: `src/lib.rs:1-22`

**Step 1: Write the failing test**

Create `src/skills/chunk_id.rs`:

```rust
//! Chunk identifiers for the hierarchical skill system

use serde::{Deserialize, Serialize};

/// Unique identifier for a skill chunk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChunkId {
    // Level 1 - Micro-chunks (first learning)
    BasicSwing,
    BasicBlock,
    BasicStance,

    // Level 2 - Action chunks (competent soldier)
    AttackSequence,
    DefendSequence,
    Riposte,

    // Level 3 - Tactical chunks (veteran)
    EngageMelee,
    HandleFlanking,
}

impl ChunkId {
    /// Get the hierarchy level of this chunk (1-5)
    pub fn level(&self) -> u8 {
        match self {
            Self::BasicSwing | Self::BasicBlock | Self::BasicStance => 1,
            Self::AttackSequence | Self::DefendSequence | Self::Riposte => 2,
            Self::EngageMelee | Self::HandleFlanking => 3,
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            Self::BasicSwing => "Basic Swing",
            Self::BasicBlock => "Basic Block",
            Self::BasicStance => "Basic Stance",
            Self::AttackSequence => "Attack Sequence",
            Self::DefendSequence => "Defend Sequence",
            Self::Riposte => "Riposte",
            Self::EngageMelee => "Engage Melee",
            Self::HandleFlanking => "Handle Flanking",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_levels() {
        assert_eq!(ChunkId::BasicSwing.level(), 1);
        assert_eq!(ChunkId::AttackSequence.level(), 2);
        assert_eq!(ChunkId::EngageMelee.level(), 3);
    }

    #[test]
    fn test_chunk_names() {
        assert_eq!(ChunkId::BasicSwing.name(), "Basic Swing");
        assert_eq!(ChunkId::Riposte.name(), "Riposte");
    }
}
```

**Step 2: Create module file**

Create `src/skills/mod.rs`:

```rust
//! Hierarchical chunking skill system
//!
//! Skill mastery is modeled through cognitive chunking - practiced actions
//! combine into larger chunks that execute with lower attention cost.
//!
//! A novice conscript thinks: "Grip. Stance. Swing. Don't drop it."
//! A master thinks: "Handle this flank." - thousands of micro-actions automatic.

pub mod chunk_id;

pub use chunk_id::ChunkId;
```

**Step 3: Register module in lib.rs**

Modify `src/lib.rs` to add after line 11 (`pub mod combat;`):

```rust
pub mod skills;
```

**Step 4: Run test to verify it compiles and passes**

Run: `cargo test --lib skills::chunk_id::tests -v`
Expected: PASS (2 tests)

**Step 5: Commit**

```bash
git add src/skills/mod.rs src/skills/chunk_id.rs src/lib.rs
git commit -m "feat(skills): add ChunkId enum and module structure"
```

---

## Task 2: Context Tags

**Files:**
- Create: `src/skills/context.rs`
- Modify: `src/skills/mod.rs`

**Step 1: Write the failing test**

Create `src/skills/context.rs`:

```rust
//! Combat context tags for chunk applicability

use serde::{Deserialize, Serialize};

/// Context tags that determine which chunks are applicable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextTag {
    // Spatial
    InMelee,
    AtRange,
    Flanked,
    Flanking,

    // Equipment
    HasSword,
    HasShield,
    HasPolearm,
    Armored,

    // Opponent
    EnemyVisible,
    MultipleEnemies,

    // State
    Fresh,
    Fatigued,
}

/// A set of context tags for a combat situation
#[derive(Debug, Clone, Default)]
pub struct CombatContext {
    tags: Vec<ContextTag>,
}

impl CombatContext {
    pub fn new() -> Self {
        Self { tags: Vec::new() }
    }

    pub fn with_tag(mut self, tag: ContextTag) -> Self {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
        self
    }

    pub fn has(&self, tag: ContextTag) -> bool {
        self.tags.contains(&tag)
    }

    pub fn tags(&self) -> &[ContextTag] {
        &self.tags
    }

    /// Calculate match quality against requirements (0.0 to 1.0)
    pub fn match_quality(&self, requirements: &[ContextTag]) -> f32 {
        if requirements.is_empty() {
            return 1.0;
        }
        let matched = requirements.iter().filter(|r| self.has(**r)).count();
        matched as f32 / requirements.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder() {
        let ctx = CombatContext::new()
            .with_tag(ContextTag::InMelee)
            .with_tag(ContextTag::HasSword);

        assert!(ctx.has(ContextTag::InMelee));
        assert!(ctx.has(ContextTag::HasSword));
        assert!(!ctx.has(ContextTag::AtRange));
    }

    #[test]
    fn test_match_quality_full() {
        let ctx = CombatContext::new()
            .with_tag(ContextTag::InMelee)
            .with_tag(ContextTag::HasSword);

        let reqs = &[ContextTag::InMelee, ContextTag::HasSword];
        assert_eq!(ctx.match_quality(reqs), 1.0);
    }

    #[test]
    fn test_match_quality_partial() {
        let ctx = CombatContext::new()
            .with_tag(ContextTag::InMelee);

        let reqs = &[ContextTag::InMelee, ContextTag::HasSword];
        assert_eq!(ctx.match_quality(reqs), 0.5);
    }

    #[test]
    fn test_match_quality_empty_requirements() {
        let ctx = CombatContext::new();
        assert_eq!(ctx.match_quality(&[]), 1.0);
    }
}
```

**Step 2: Update module exports**

Modify `src/skills/mod.rs`:

```rust
//! Hierarchical chunking skill system

pub mod chunk_id;
pub mod context;

pub use chunk_id::ChunkId;
pub use context::{CombatContext, ContextTag};
```

**Step 3: Run tests**

Run: `cargo test --lib skills::context::tests -v`
Expected: PASS (4 tests)

**Step 4: Commit**

```bash
git add src/skills/context.rs src/skills/mod.rs
git commit -m "feat(skills): add ContextTag enum and CombatContext"
```

---

## Task 3: Chunk Definitions (Global Library)

**Files:**
- Create: `src/skills/definitions.rs`
- Modify: `src/skills/mod.rs`

**Step 1: Write chunk definitions**

Create `src/skills/definitions.rs`:

```rust
//! Static chunk definitions - the global library all entities reference

use crate::skills::{ChunkId, ContextTag};

/// What a chunk contains
#[derive(Debug, Clone)]
pub enum ChunkComponents {
    /// Level 0: Cannot be decomposed further (internal)
    Atomic,
    /// Level 1+: Composed of other chunks
    Composite(&'static [ChunkId]),
}

/// Definition of a skill chunk
#[derive(Debug, Clone)]
pub struct ChunkDefinition {
    pub id: ChunkId,
    pub name: &'static str,
    pub level: u8,
    pub components: ChunkComponents,
    pub context_requirements: &'static [ContextTag],
    pub prerequisite_chunks: &'static [ChunkId],
    /// Base number of repetitions to form this chunk
    pub base_repetitions: u32,
}

/// Global chunk library - static definitions
pub static CHUNK_LIBRARY: &[ChunkDefinition] = &[
    // Level 1 - Micro-chunks
    ChunkDefinition {
        id: ChunkId::BasicSwing,
        name: "Basic Swing",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[ContextTag::InMelee],
        prerequisite_chunks: &[],
        base_repetitions: 20,
    },
    ChunkDefinition {
        id: ChunkId::BasicBlock,
        name: "Basic Block",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[ContextTag::InMelee, ContextTag::EnemyVisible],
        prerequisite_chunks: &[],
        base_repetitions: 20,
    },
    ChunkDefinition {
        id: ChunkId::BasicStance,
        name: "Basic Stance",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[],
        prerequisite_chunks: &[],
        base_repetitions: 10,
    },

    // Level 2 - Action chunks
    ChunkDefinition {
        id: ChunkId::AttackSequence,
        name: "Attack Sequence",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::BasicStance,
            ChunkId::BasicSwing,
        ]),
        context_requirements: &[ContextTag::InMelee],
        prerequisite_chunks: &[ChunkId::BasicStance, ChunkId::BasicSwing],
        base_repetitions: 50,
    },
    ChunkDefinition {
        id: ChunkId::DefendSequence,
        name: "Defend Sequence",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::BasicBlock,
            ChunkId::BasicStance,
        ]),
        context_requirements: &[ContextTag::InMelee, ContextTag::EnemyVisible],
        prerequisite_chunks: &[ChunkId::BasicBlock, ChunkId::BasicStance],
        base_repetitions: 50,
    },
    ChunkDefinition {
        id: ChunkId::Riposte,
        name: "Riposte",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::BasicBlock,
            ChunkId::BasicSwing,
        ]),
        context_requirements: &[ContextTag::InMelee, ContextTag::EnemyVisible],
        prerequisite_chunks: &[ChunkId::BasicBlock, ChunkId::BasicSwing],
        base_repetitions: 80,
    },

    // Level 3 - Tactical chunks
    ChunkDefinition {
        id: ChunkId::EngageMelee,
        name: "Engage Melee",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::AttackSequence,
            ChunkId::DefendSequence,
        ]),
        context_requirements: &[ContextTag::InMelee],
        prerequisite_chunks: &[ChunkId::AttackSequence, ChunkId::DefendSequence],
        base_repetitions: 200,
    },
    ChunkDefinition {
        id: ChunkId::HandleFlanking,
        name: "Handle Flanking",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::EngageMelee,
        ]),
        context_requirements: &[ContextTag::InMelee, ContextTag::MultipleEnemies],
        prerequisite_chunks: &[ChunkId::EngageMelee],
        base_repetitions: 300,
    },
];

/// Look up a chunk definition by ID
pub fn get_chunk_definition(id: ChunkId) -> Option<&'static ChunkDefinition> {
    CHUNK_LIBRARY.iter().find(|def| def.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_has_all_chunks() {
        // Every ChunkId variant should have a definition
        assert!(get_chunk_definition(ChunkId::BasicSwing).is_some());
        assert!(get_chunk_definition(ChunkId::BasicBlock).is_some());
        assert!(get_chunk_definition(ChunkId::BasicStance).is_some());
        assert!(get_chunk_definition(ChunkId::AttackSequence).is_some());
        assert!(get_chunk_definition(ChunkId::DefendSequence).is_some());
        assert!(get_chunk_definition(ChunkId::Riposte).is_some());
        assert!(get_chunk_definition(ChunkId::EngageMelee).is_some());
        assert!(get_chunk_definition(ChunkId::HandleFlanking).is_some());
    }

    #[test]
    fn test_levels_match_chunk_id() {
        for def in CHUNK_LIBRARY {
            assert_eq!(def.level, def.id.level(),
                "Definition level mismatch for {:?}", def.id);
        }
    }

    #[test]
    fn test_prerequisites_exist() {
        for def in CHUNK_LIBRARY {
            for prereq in def.prerequisite_chunks {
                assert!(get_chunk_definition(*prereq).is_some(),
                    "Missing prerequisite {:?} for {:?}", prereq, def.id);
            }
        }
    }

    #[test]
    fn test_composite_components_exist() {
        for def in CHUNK_LIBRARY {
            if let ChunkComponents::Composite(components) = &def.components {
                for comp in *components {
                    assert!(get_chunk_definition(*comp).is_some(),
                        "Missing component {:?} for {:?}", comp, def.id);
                }
            }
        }
    }
}
```

**Step 2: Update module exports**

Modify `src/skills/mod.rs`:

```rust
//! Hierarchical chunking skill system

pub mod chunk_id;
pub mod context;
pub mod definitions;

pub use chunk_id::ChunkId;
pub use context::{CombatContext, ContextTag};
pub use definitions::{get_chunk_definition, ChunkComponents, ChunkDefinition, CHUNK_LIBRARY};
```

**Step 3: Run tests**

Run: `cargo test --lib skills::definitions::tests -v`
Expected: PASS (4 tests)

**Step 4: Commit**

```bash
git add src/skills/definitions.rs src/skills/mod.rs
git commit -m "feat(skills): add static chunk definitions library"
```

---

## Task 4: Per-Entity Chunk State (ChunkLibrary)

**Files:**
- Create: `src/skills/library.rs`
- Modify: `src/skills/mod.rs`

**Step 1: Write chunk library**

Create `src/skills/library.rs`:

```rust
//! Per-entity chunk state storage

use crate::skills::ChunkId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Personal state for a chunk the entity has encountered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalChunkState {
    /// How automatic this chunk is (0.0 to 1.0)
    /// 0.0 = conscious effort, 1.0 = fully compiled
    pub encoding_depth: f32,
    /// Times this chunk has been executed
    pub repetition_count: u32,
    /// Tick when last used (for rust decay)
    pub last_used_tick: u64,
    /// Tick when first formed
    pub formation_tick: u64,
}

impl PersonalChunkState {
    pub fn new(tick: u64) -> Self {
        Self {
            encoding_depth: 0.1, // Just learned
            repetition_count: 1,
            last_used_tick: tick,
            formation_tick: tick,
        }
    }

    /// Attention cost to execute this chunk (1.0 - depth)
    pub fn attention_cost(&self) -> f32 {
        1.0 - self.encoding_depth
    }
}

/// Experience record for learning
#[derive(Debug, Clone)]
pub struct Experience {
    pub chunk_id: ChunkId,
    pub success: bool,
    pub tick: u64,
}

/// Per-entity skill chunk library
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChunkLibrary {
    /// Chunks this entity has formed
    #[serde(default)]
    chunks: HashMap<ChunkId, PersonalChunkState>,

    /// Current attention budget (refreshes each decision point)
    #[serde(skip)]
    pub attention_budget: f32,

    /// Attention spent this decision point
    #[serde(skip)]
    pub attention_spent: f32,

    /// Pending experiences (cleared during learning consolidation)
    #[serde(skip)]
    pending_experiences: Vec<Experience>,
}

impl ChunkLibrary {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            attention_budget: 1.0,
            attention_spent: 0.0,
            pending_experiences: Vec::new(),
        }
    }

    /// Create a library with basic universal chunks (walking, etc.)
    pub fn with_basics() -> Self {
        let mut lib = Self::new();
        // Fresh entity has no combat chunks - must learn everything
        lib
    }

    /// Create a library for a trained soldier
    pub fn trained_soldier(tick: u64) -> Self {
        let mut lib = Self::new();

        // Level 1 chunks - well practiced
        lib.chunks.insert(ChunkId::BasicSwing, PersonalChunkState {
            encoding_depth: 0.6,
            repetition_count: 100,
            last_used_tick: tick,
            formation_tick: tick.saturating_sub(1000),
        });
        lib.chunks.insert(ChunkId::BasicBlock, PersonalChunkState {
            encoding_depth: 0.5,
            repetition_count: 80,
            last_used_tick: tick,
            formation_tick: tick.saturating_sub(1000),
        });
        lib.chunks.insert(ChunkId::BasicStance, PersonalChunkState {
            encoding_depth: 0.7,
            repetition_count: 150,
            last_used_tick: tick,
            formation_tick: tick.saturating_sub(1000),
        });

        // Level 2 - forming
        lib.chunks.insert(ChunkId::AttackSequence, PersonalChunkState {
            encoding_depth: 0.3,
            repetition_count: 30,
            last_used_tick: tick,
            formation_tick: tick.saturating_sub(500),
        });

        lib
    }

    /// Create a library for a veteran
    pub fn veteran(tick: u64) -> Self {
        let mut lib = Self::trained_soldier(tick);

        // Deepen level 1 chunks
        if let Some(state) = lib.chunks.get_mut(&ChunkId::BasicSwing) {
            state.encoding_depth = 0.85;
            state.repetition_count = 500;
        }
        if let Some(state) = lib.chunks.get_mut(&ChunkId::BasicBlock) {
            state.encoding_depth = 0.85;
            state.repetition_count = 500;
        }
        if let Some(state) = lib.chunks.get_mut(&ChunkId::BasicStance) {
            state.encoding_depth = 0.9;
            state.repetition_count = 600;
        }

        // Level 2 chunks - practiced
        lib.chunks.insert(ChunkId::AttackSequence, PersonalChunkState {
            encoding_depth: 0.7,
            repetition_count: 200,
            last_used_tick: tick,
            formation_tick: tick.saturating_sub(2000),
        });
        lib.chunks.insert(ChunkId::DefendSequence, PersonalChunkState {
            encoding_depth: 0.65,
            repetition_count: 180,
            last_used_tick: tick,
            formation_tick: tick.saturating_sub(2000),
        });
        lib.chunks.insert(ChunkId::Riposte, PersonalChunkState {
            encoding_depth: 0.5,
            repetition_count: 100,
            last_used_tick: tick,
            formation_tick: tick.saturating_sub(1500),
        });

        // Level 3 - forming
        lib.chunks.insert(ChunkId::EngageMelee, PersonalChunkState {
            encoding_depth: 0.3,
            repetition_count: 50,
            last_used_tick: tick,
            formation_tick: tick.saturating_sub(500),
        });

        lib
    }

    /// Check if entity has a chunk
    pub fn has_chunk(&self, id: ChunkId) -> bool {
        self.chunks.contains_key(&id)
    }

    /// Get chunk state (if formed)
    pub fn get_chunk(&self, id: ChunkId) -> Option<&PersonalChunkState> {
        self.chunks.get(&id)
    }

    /// Get mutable chunk state
    pub fn get_chunk_mut(&mut self, id: ChunkId) -> Option<&mut PersonalChunkState> {
        self.chunks.get_mut(&id)
    }

    /// Insert or update a chunk
    pub fn set_chunk(&mut self, id: ChunkId, state: PersonalChunkState) {
        self.chunks.insert(id, state);
    }

    /// Get all formed chunks
    pub fn chunks(&self) -> &HashMap<ChunkId, PersonalChunkState> {
        &self.chunks
    }

    /// Get mutable access to chunks
    pub fn chunks_mut(&mut self) -> &mut HashMap<ChunkId, PersonalChunkState> {
        &mut self.chunks
    }

    /// Remaining attention this decision point
    pub fn attention_remaining(&self) -> f32 {
        (self.attention_budget - self.attention_spent).max(0.0)
    }

    /// Spend attention (returns false if insufficient)
    pub fn spend_attention(&mut self, cost: f32) -> bool {
        if cost > self.attention_remaining() {
            return false;
        }
        self.attention_spent += cost;
        true
    }

    /// Record an experience for later learning
    pub fn record_experience(&mut self, exp: Experience) {
        self.pending_experiences.push(exp);
    }

    /// Get pending experiences
    pub fn pending_experiences(&self) -> &[Experience] {
        &self.pending_experiences
    }

    /// Clear pending experiences (after consolidation)
    pub fn clear_experiences(&mut self) {
        self.pending_experiences.clear();
    }

    /// Get best combat chunk (highest level with good depth)
    pub fn best_combat_chunk(&self) -> Option<(ChunkId, f32)> {
        self.chunks
            .iter()
            .max_by(|a, b| {
                let score_a = a.0.level() as f32 * 10.0 + a.1.encoding_depth * 5.0;
                let score_b = b.0.level() as f32 * 10.0 + b.1.encoding_depth * 5.0;
                score_a.partial_cmp(&score_b).unwrap()
            })
            .map(|(id, state)| (*id, state.encoding_depth))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_library() {
        let lib = ChunkLibrary::new();
        assert!(!lib.has_chunk(ChunkId::BasicSwing));
        assert_eq!(lib.attention_budget, 1.0);
    }

    #[test]
    fn test_trained_soldier_has_chunks() {
        let lib = ChunkLibrary::trained_soldier(1000);
        assert!(lib.has_chunk(ChunkId::BasicSwing));
        assert!(lib.has_chunk(ChunkId::BasicStance));
        assert!(lib.has_chunk(ChunkId::AttackSequence));
    }

    #[test]
    fn test_attention_spending() {
        let mut lib = ChunkLibrary::new();
        lib.attention_budget = 1.0;

        assert!(lib.spend_attention(0.3));
        assert_eq!(lib.attention_remaining(), 0.7);

        assert!(lib.spend_attention(0.5));
        assert!(!lib.spend_attention(0.3)); // Not enough
    }

    #[test]
    fn test_attention_cost_from_depth() {
        let state = PersonalChunkState {
            encoding_depth: 0.8,
            repetition_count: 100,
            last_used_tick: 0,
            formation_tick: 0,
        };
        assert!((state.attention_cost() - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_experience_recording() {
        let mut lib = ChunkLibrary::new();
        lib.record_experience(Experience {
            chunk_id: ChunkId::BasicSwing,
            success: true,
            tick: 100,
        });

        assert_eq!(lib.pending_experiences().len(), 1);
        lib.clear_experiences();
        assert_eq!(lib.pending_experiences().len(), 0);
    }
}
```

**Step 2: Update module exports**

Modify `src/skills/mod.rs`:

```rust
//! Hierarchical chunking skill system

pub mod chunk_id;
pub mod context;
pub mod definitions;
pub mod library;

pub use chunk_id::ChunkId;
pub use context::{CombatContext, ContextTag};
pub use definitions::{get_chunk_definition, ChunkComponents, ChunkDefinition, CHUNK_LIBRARY};
pub use library::{ChunkLibrary, Experience, PersonalChunkState};
```

**Step 3: Run tests**

Run: `cargo test --lib skills::library::tests -v`
Expected: PASS (5 tests)

**Step 4: Commit**

```bash
git add src/skills/library.rs src/skills/mod.rs
git commit -m "feat(skills): add ChunkLibrary per-entity state"
```

---

## Task 5: Attention Budget Management

**Files:**
- Create: `src/skills/attention.rs`
- Modify: `src/skills/mod.rs`

**Step 1: Write attention system**

Create `src/skills/attention.rs`:

```rust
//! Attention budget management
//!
//! Each entity has a base attention budget of 1.0 per decision point.
//! Fatigue, pain, and stress reduce available attention.

/// Refresh attention budget for a new decision point
///
/// # Arguments
/// * `fatigue` - 0.0 (fresh) to 1.0 (exhausted)
/// * `pain` - 0.0 (none) to 1.0 (incapacitating)
/// * `stress` - 0.0 (calm) to 1.0 (panicked)
///
/// # Returns
/// Available attention budget (minimum 0.2)
pub fn calculate_attention_budget(fatigue: f32, pain: f32, stress: f32) -> f32 {
    let base = 1.0;

    // Penalties stack but multiplicatively to prevent going negative
    let fatigue_mult = 1.0 - (fatigue * 0.3);  // Max -30%
    let pain_mult = 1.0 - (pain * 0.4);        // Max -40%
    let stress_mult = 1.0 - (stress * 0.2);    // Max -20%

    let budget = base * fatigue_mult * pain_mult * stress_mult;

    // Minimum viable attention
    budget.max(0.2)
}

/// Check if an action is affordable within current attention
pub fn can_afford_attention(remaining: f32, cost: f32) -> bool {
    cost <= remaining
}

/// Fumble threshold - below this attention, action risks fumbling
pub const FUMBLE_ATTENTION_THRESHOLD: f32 = 0.1;

/// Check if action execution risks fumble due to attention overload
pub fn risks_fumble(remaining_after: f32) -> bool {
    remaining_after < FUMBLE_ATTENTION_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_entity_full_attention() {
        let budget = calculate_attention_budget(0.0, 0.0, 0.0);
        assert_eq!(budget, 1.0);
    }

    #[test]
    fn test_exhausted_reduced_attention() {
        let budget = calculate_attention_budget(1.0, 0.0, 0.0);
        assert!((budget - 0.7).abs() < 0.01); // 30% reduction
    }

    #[test]
    fn test_combined_penalties() {
        // High fatigue + pain + stress
        let budget = calculate_attention_budget(0.8, 0.5, 0.6);

        // 1.0 * 0.76 * 0.8 * 0.88 = 0.535
        assert!(budget > 0.4 && budget < 0.6);
    }

    #[test]
    fn test_minimum_attention_floor() {
        // Even with max penalties, minimum 0.2
        let budget = calculate_attention_budget(1.0, 1.0, 1.0);
        assert_eq!(budget, 0.2);
    }

    #[test]
    fn test_can_afford() {
        assert!(can_afford_attention(0.5, 0.3));
        assert!(can_afford_attention(0.5, 0.5));
        assert!(!can_afford_attention(0.5, 0.6));
    }

    #[test]
    fn test_fumble_risk() {
        assert!(!risks_fumble(0.2));
        assert!(risks_fumble(0.05));
        assert!(risks_fumble(-0.1));
    }
}
```

**Step 2: Update module exports**

Modify `src/skills/mod.rs`:

```rust
//! Hierarchical chunking skill system

pub mod attention;
pub mod chunk_id;
pub mod context;
pub mod definitions;
pub mod library;

pub use attention::{calculate_attention_budget, can_afford_attention, risks_fumble, FUMBLE_ATTENTION_THRESHOLD};
pub use chunk_id::ChunkId;
pub use context::{CombatContext, ContextTag};
pub use definitions::{get_chunk_definition, ChunkComponents, ChunkDefinition, CHUNK_LIBRARY};
pub use library::{ChunkLibrary, Experience, PersonalChunkState};
```

**Step 3: Run tests**

Run: `cargo test --lib skills::attention::tests -v`
Expected: PASS (6 tests)

**Step 4: Commit**

```bash
git add src/skills/attention.rs src/skills/mod.rs
git commit -m "feat(skills): add attention budget calculation"
```

---

## Task 6: Action Resolution

**Files:**
- Create: `src/skills/resolution.rs`
- Modify: `src/skills/mod.rs`

**Step 1: Write resolution system**

Create `src/skills/resolution.rs`:

```rust
//! Action resolution using chunk system
//!
//! Resolves combat actions through chunking - finding the best matching chunk,
//! spending attention, and determining outcome variance.

use crate::skills::{
    can_afford_attention, get_chunk_definition, risks_fumble, ChunkId, ChunkLibrary,
    CombatContext, Experience, CHUNK_LIBRARY,
};

/// Result of attempting an action
#[derive(Debug, Clone)]
pub enum ActionResult {
    /// Action succeeded
    Success {
        /// Skill modifier (0.0 to 1.0) affecting outcome quality
        skill_modifier: f32,
        /// Chunk used (if any)
        chunk_used: Option<ChunkId>,
    },
    /// Action succeeded critically (master execution)
    Critical {
        skill_modifier: f32,
        chunk_used: Option<ChunkId>,
    },
    /// Action failed cleanly
    Failure,
    /// Action fumbled (negative outcome possible)
    Fumble,
    /// Not enough attention to attempt
    AttentionOverload,
}

impl ActionResult {
    pub fn is_success(&self) -> bool {
        matches!(self, ActionResult::Success { .. } | ActionResult::Critical { .. })
    }

    pub fn skill_modifier(&self) -> f32 {
        match self {
            ActionResult::Success { skill_modifier, .. } => *skill_modifier,
            ActionResult::Critical { skill_modifier, .. } => *skill_modifier,
            _ => 0.0,
        }
    }
}

/// Find the best chunk for an intended action in context
///
/// Returns (ChunkId, encoding_depth) of best match, or None if no applicable chunks
pub fn find_best_chunk(
    library: &ChunkLibrary,
    action_chunks: &[ChunkId],
    context: &CombatContext,
) -> Option<(ChunkId, f32)> {
    let mut best: Option<(ChunkId, f32, f32)> = None; // (id, depth, score)

    for chunk_id in action_chunks {
        if let Some(state) = library.get_chunk(*chunk_id) {
            if let Some(def) = get_chunk_definition(*chunk_id) {
                let context_quality = context.match_quality(def.context_requirements);

                // Skip if context doesn't match at all
                if context_quality < 0.5 {
                    continue;
                }

                // Score: higher level + better encoding + better context match
                let score = (def.level as f32) * 10.0
                    + state.encoding_depth * 5.0
                    + context_quality * 3.0;

                if best.map_or(true, |(_, _, s)| score > s) {
                    best = Some((*chunk_id, state.encoding_depth, score));
                }
            }
        }
    }

    best.map(|(id, depth, _)| (id, depth))
}

/// Chunks applicable for offensive melee actions
pub const ATTACK_CHUNKS: &[ChunkId] = &[
    ChunkId::BasicSwing,
    ChunkId::AttackSequence,
    ChunkId::EngageMelee,
    ChunkId::HandleFlanking,
];

/// Chunks applicable for defensive actions
pub const DEFENSE_CHUNKS: &[ChunkId] = &[
    ChunkId::BasicBlock,
    ChunkId::DefendSequence,
    ChunkId::EngageMelee,
    ChunkId::HandleFlanking,
];

/// Chunks applicable for riposte (counter-attack)
pub const RIPOSTE_CHUNKS: &[ChunkId] = &[
    ChunkId::Riposte,
    ChunkId::EngageMelee,
];

/// Resolve an attack action
///
/// Returns result and records experience
pub fn resolve_attack(
    library: &mut ChunkLibrary,
    context: &CombatContext,
    tick: u64,
) -> ActionResult {
    resolve_action(library, ATTACK_CHUNKS, context, tick)
}

/// Resolve a defense action
pub fn resolve_defense(
    library: &mut ChunkLibrary,
    context: &CombatContext,
    tick: u64,
) -> ActionResult {
    resolve_action(library, DEFENSE_CHUNKS, context, tick)
}

/// Resolve a riposte action
pub fn resolve_riposte(
    library: &mut ChunkLibrary,
    context: &CombatContext,
    tick: u64,
) -> ActionResult {
    resolve_action(library, RIPOSTE_CHUNKS, context, tick)
}

/// Core action resolution
fn resolve_action(
    library: &mut ChunkLibrary,
    applicable_chunks: &[ChunkId],
    context: &CombatContext,
    tick: u64,
) -> ActionResult {
    // Find best chunk
    let (chunk_id, encoding_depth, attention_cost) =
        if let Some((id, depth)) = find_best_chunk(library, applicable_chunks, context) {
            (Some(id), depth, 1.0 - depth)
        } else {
            // No chunk - use atomics (very expensive)
            (None, 0.1, 0.9)
        };

    // Check attention budget
    if !can_afford_attention(library.attention_remaining(), attention_cost) {
        return ActionResult::AttentionOverload;
    }

    // Spend attention
    library.spend_attention(attention_cost);

    // Record experience
    if let Some(id) = chunk_id {
        library.record_experience(Experience {
            chunk_id: id,
            success: true, // Will be updated by caller if action fails
            tick,
        });

        // Update last used tick
        if let Some(state) = library.get_chunk_mut(id) {
            state.last_used_tick = tick;
        }
    }

    // Check fumble risk
    if risks_fumble(library.attention_remaining()) && encoding_depth < 0.3 {
        return ActionResult::Fumble;
    }

    // Determine success level
    // Higher encoding = more likely critical, lower variance
    if encoding_depth > 0.9 {
        ActionResult::Critical {
            skill_modifier: encoding_depth,
            chunk_used: chunk_id,
        }
    } else {
        ActionResult::Success {
            skill_modifier: encoding_depth,
            chunk_used: chunk_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::ContextTag;

    #[test]
    fn test_no_chunks_high_cost() {
        let mut lib = ChunkLibrary::new();
        lib.attention_budget = 1.0;

        let ctx = CombatContext::new().with_tag(ContextTag::InMelee);
        let result = resolve_attack(&mut lib, &ctx, 0);

        // Should succeed but spend most attention
        assert!(result.is_success());
        assert!(lib.attention_remaining() < 0.2);
    }

    #[test]
    fn test_veteran_low_cost() {
        let mut lib = ChunkLibrary::veteran(0);
        lib.attention_budget = 1.0;

        let ctx = CombatContext::new()
            .with_tag(ContextTag::InMelee)
            .with_tag(ContextTag::EnemyVisible);

        let result = resolve_attack(&mut lib, &ctx, 100);

        assert!(result.is_success());
        // Should have lots of attention remaining
        assert!(lib.attention_remaining() > 0.6);
    }

    #[test]
    fn test_attention_overload() {
        let mut lib = ChunkLibrary::new();
        lib.attention_budget = 0.5;
        lib.attention_spent = 0.5;

        let ctx = CombatContext::new().with_tag(ContextTag::InMelee);
        let result = resolve_attack(&mut lib, &ctx, 0);

        assert!(matches!(result, ActionResult::AttentionOverload));
    }

    #[test]
    fn test_find_best_chunk_prefers_higher_level() {
        let lib = ChunkLibrary::veteran(0);
        let ctx = CombatContext::new()
            .with_tag(ContextTag::InMelee)
            .with_tag(ContextTag::EnemyVisible);

        let best = find_best_chunk(&lib, ATTACK_CHUNKS, &ctx);

        // Should prefer EngageMelee (level 3) over BasicSwing (level 1)
        assert!(best.is_some());
        let (id, _) = best.unwrap();
        assert!(id.level() >= 2);
    }

    #[test]
    fn test_context_affects_selection() {
        let lib = ChunkLibrary::veteran(0);

        // Without MultipleEnemies, shouldn't select HandleFlanking
        let ctx = CombatContext::new().with_tag(ContextTag::InMelee);
        let best = find_best_chunk(&lib, &[ChunkId::HandleFlanking, ChunkId::BasicSwing], &ctx);

        // HandleFlanking requires MultipleEnemies - should get BasicSwing
        if let Some((id, _)) = best {
            assert_ne!(id, ChunkId::HandleFlanking);
        }
    }

    #[test]
    fn test_experience_recorded() {
        let mut lib = ChunkLibrary::trained_soldier(0);
        lib.attention_budget = 1.0;

        let ctx = CombatContext::new().with_tag(ContextTag::InMelee);
        let _ = resolve_attack(&mut lib, &ctx, 100);

        assert!(!lib.pending_experiences().is_empty());
    }
}
```

**Step 2: Update module exports**

Modify `src/skills/mod.rs`:

```rust
//! Hierarchical chunking skill system

pub mod attention;
pub mod chunk_id;
pub mod context;
pub mod definitions;
pub mod library;
pub mod resolution;

pub use attention::{calculate_attention_budget, can_afford_attention, risks_fumble, FUMBLE_ATTENTION_THRESHOLD};
pub use chunk_id::ChunkId;
pub use context::{CombatContext, ContextTag};
pub use definitions::{get_chunk_definition, ChunkComponents, ChunkDefinition, CHUNK_LIBRARY};
pub use library::{ChunkLibrary, Experience, PersonalChunkState};
pub use resolution::{
    find_best_chunk, resolve_attack, resolve_defense, resolve_riposte, ActionResult,
    ATTACK_CHUNKS, DEFENSE_CHUNKS, RIPOSTE_CHUNKS,
};
```

**Step 3: Run tests**

Run: `cargo test --lib skills::resolution::tests -v`
Expected: PASS (6 tests)

**Step 4: Commit**

```bash
git add src/skills/resolution.rs src/skills/mod.rs
git commit -m "feat(skills): add action resolution with chunking"
```

---

## Task 7: Learning System

**Files:**
- Create: `src/skills/learning.rs`
- Modify: `src/skills/mod.rs`

**Step 1: Write learning system**

Create `src/skills/learning.rs`:

```rust
//! Learning and chunk formation
//!
//! Entities develop chunks through practice. Encoding depth increases
//! logarithmically with repetitions. Unused chunks rust over time.

use crate::skills::{get_chunk_definition, ChunkId, ChunkLibrary, PersonalChunkState, CHUNK_LIBRARY};

/// Learning rate constant (higher = faster learning)
const LEARNING_RATE: f32 = 0.01;

/// Ticks until rust starts (unused chunks decay)
const RUST_THRESHOLD: u64 = 10000;

/// Rust decay rate per tick past threshold
const RUST_RATE: f32 = 0.0001;

/// Minimum encoding depth (chunks never fully forgotten)
const MIN_ENCODING: f32 = 0.1;

/// Maximum encoding depth
const MAX_ENCODING: f32 = 0.99;

/// Calculate encoding depth from repetition count
///
/// Uses logarithmic curve: fast early gains, slow mastery
pub fn calculate_encoding_depth(repetitions: u32) -> f32 {
    // depth = 1.0 - (1.0 / (1.0 + count * rate))
    let depth = 1.0 - (1.0 / (1.0 + repetitions as f32 * LEARNING_RATE));
    depth.clamp(MIN_ENCODING, MAX_ENCODING)
}

/// Process learning for an entity
///
/// - Consolidates pending experiences into encoding depth
/// - Checks for new chunk formation
/// - Applies rust decay to unused chunks
pub fn process_learning(library: &mut ChunkLibrary, tick: u64) {
    // 1. Consolidate experiences
    for exp in library.pending_experiences().to_vec() {
        if let Some(state) = library.get_chunk_mut(exp.chunk_id) {
            // Increment repetitions
            if exp.success {
                state.repetition_count += 1;
            } else {
                // Failures teach less
                state.repetition_count = state.repetition_count.saturating_add(1).saturating_sub(1);
            }

            // Recalculate encoding depth
            state.encoding_depth = calculate_encoding_depth(state.repetition_count);
            state.last_used_tick = exp.tick;
        } else {
            // Experience with un-owned chunk - check if we can form it
            check_chunk_formation(library, exp.chunk_id, tick);
        }
    }

    library.clear_experiences();

    // 2. Check for new chunk formation from prerequisites
    check_all_formations(library, tick);

    // 3. Apply rust decay
    apply_rust_decay(library, tick);
}

/// Check if prerequisites are met to form a new chunk
fn check_chunk_formation(library: &mut ChunkLibrary, chunk_id: ChunkId, tick: u64) {
    if library.has_chunk(chunk_id) {
        return;
    }

    let Some(def) = get_chunk_definition(chunk_id) else {
        return;
    };

    // Check all prerequisites met with sufficient depth
    let prereqs_met = def.prerequisite_chunks.iter().all(|prereq| {
        library.get_chunk(*prereq).map_or(false, |s| s.encoding_depth > 0.3)
    });

    if prereqs_met {
        library.set_chunk(chunk_id, PersonalChunkState::new(tick));
    }
}

/// Check all potential chunk formations
fn check_all_formations(library: &mut ChunkLibrary, tick: u64) {
    // Get chunks we might be able to form
    let candidate_chunks: Vec<ChunkId> = CHUNK_LIBRARY
        .iter()
        .filter(|def| !library.has_chunk(def.id))
        .map(|def| def.id)
        .collect();

    for chunk_id in candidate_chunks {
        check_chunk_formation(library, chunk_id, tick);
    }
}

/// Apply rust decay to unused chunks
fn apply_rust_decay(library: &mut ChunkLibrary, tick: u64) {
    for state in library.chunks_mut().values_mut() {
        let ticks_since_use = tick.saturating_sub(state.last_used_tick);

        if ticks_since_use > RUST_THRESHOLD {
            let decay_ticks = ticks_since_use - RUST_THRESHOLD;
            let decay = decay_ticks as f32 * RUST_RATE;
            state.encoding_depth = (state.encoding_depth - decay).max(MIN_ENCODING);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::Experience;

    #[test]
    fn test_encoding_curve_starts_low() {
        assert!(calculate_encoding_depth(1) < 0.15);
    }

    #[test]
    fn test_encoding_curve_grows() {
        let depth_10 = calculate_encoding_depth(10);
        let depth_50 = calculate_encoding_depth(50);
        let depth_200 = calculate_encoding_depth(200);

        assert!(depth_10 < depth_50);
        assert!(depth_50 < depth_200);
    }

    #[test]
    fn test_encoding_curve_plateaus() {
        let depth_1000 = calculate_encoding_depth(1000);
        let depth_5000 = calculate_encoding_depth(5000);

        // Should be close to max
        assert!(depth_1000 > 0.9);
        // Marginal gains at high counts
        assert!((depth_5000 - depth_1000) < 0.05);
    }

    #[test]
    fn test_experience_increases_depth() {
        let mut lib = ChunkLibrary::new();
        lib.set_chunk(ChunkId::BasicSwing, PersonalChunkState {
            encoding_depth: 0.2,
            repetition_count: 10,
            last_used_tick: 0,
            formation_tick: 0,
        });

        lib.record_experience(Experience {
            chunk_id: ChunkId::BasicSwing,
            success: true,
            tick: 100,
        });

        let old_depth = lib.get_chunk(ChunkId::BasicSwing).unwrap().encoding_depth;
        process_learning(&mut lib, 100);
        let new_depth = lib.get_chunk(ChunkId::BasicSwing).unwrap().encoding_depth;

        assert!(new_depth > old_depth);
    }

    #[test]
    fn test_chunk_formation_from_prerequisites() {
        let mut lib = ChunkLibrary::new();

        // Add prerequisites with sufficient depth
        lib.set_chunk(ChunkId::BasicStance, PersonalChunkState {
            encoding_depth: 0.5,
            repetition_count: 50,
            last_used_tick: 0,
            formation_tick: 0,
        });
        lib.set_chunk(ChunkId::BasicSwing, PersonalChunkState {
            encoding_depth: 0.5,
            repetition_count: 50,
            last_used_tick: 0,
            formation_tick: 0,
        });

        // Should not have AttackSequence yet
        assert!(!lib.has_chunk(ChunkId::AttackSequence));

        process_learning(&mut lib, 1000);

        // Should now have formed AttackSequence
        assert!(lib.has_chunk(ChunkId::AttackSequence));
    }

    #[test]
    fn test_rust_decay() {
        let mut lib = ChunkLibrary::new();
        lib.set_chunk(ChunkId::BasicSwing, PersonalChunkState {
            encoding_depth: 0.8,
            repetition_count: 200,
            last_used_tick: 0,
            formation_tick: 0,
        });

        // Advance time past rust threshold
        process_learning(&mut lib, RUST_THRESHOLD + 5000);

        let depth = lib.get_chunk(ChunkId::BasicSwing).unwrap().encoding_depth;
        assert!(depth < 0.8);
        assert!(depth >= MIN_ENCODING);
    }
}
```

**Step 2: Update module exports**

Modify `src/skills/mod.rs`:

```rust
//! Hierarchical chunking skill system
//!
//! Skill mastery is modeled through cognitive chunking - practiced actions
//! combine into larger chunks that execute with lower attention cost.
//!
//! A novice conscript thinks: "Grip. Stance. Swing. Don't drop it."
//! A master thinks: "Handle this flank." - thousands of micro-actions automatic.

pub mod attention;
pub mod chunk_id;
pub mod context;
pub mod definitions;
pub mod learning;
pub mod library;
pub mod resolution;

pub use attention::{calculate_attention_budget, can_afford_attention, risks_fumble, FUMBLE_ATTENTION_THRESHOLD};
pub use chunk_id::ChunkId;
pub use context::{CombatContext, ContextTag};
pub use definitions::{get_chunk_definition, ChunkComponents, ChunkDefinition, CHUNK_LIBRARY};
pub use learning::{calculate_encoding_depth, process_learning};
pub use library::{ChunkLibrary, Experience, PersonalChunkState};
pub use resolution::{
    find_best_chunk, resolve_attack, resolve_defense, resolve_riposte, ActionResult,
    ATTACK_CHUNKS, DEFENSE_CHUNKS, RIPOSTE_CHUNKS,
};
```

**Step 3: Run tests**

Run: `cargo test --lib skills::learning::tests -v`
Expected: PASS (6 tests)

**Step 4: Commit**

```bash
git add src/skills/learning.rs src/skills/mod.rs
git commit -m "feat(skills): add learning and chunk formation system"
```

---

## Task 8: SkillLevel Bridge

**Files:**
- Modify: `src/combat/skill.rs`

**Step 1: Read current implementation**

The current file has `SkillLevel` enum and `CombatSkill` struct. We need to add a method to derive SkillLevel from ChunkLibrary.

**Step 2: Add bridge implementation**

Add to `src/combat/skill.rs` after the existing `impl CombatSkill` block (around line 83):

```rust
impl CombatSkill {
    // ... existing methods ...

    /// Derive combat skill from chunk library
    ///
    /// Uses best combat chunk's encoding depth to determine skill level:
    /// - < 0.3 → Novice
    /// - < 0.6 → Trained
    /// - < 0.85 → Veteran
    /// - >= 0.85 → Master
    pub fn from_chunk_library(library: &crate::skills::ChunkLibrary) -> Self {
        let level = if let Some((_, depth)) = library.best_combat_chunk() {
            if depth >= 0.85 {
                SkillLevel::Master
            } else if depth >= 0.6 {
                SkillLevel::Veteran
            } else if depth >= 0.3 {
                SkillLevel::Trained
            } else {
                SkillLevel::Novice
            }
        } else {
            SkillLevel::Novice
        };

        Self { level }
    }
}
```

**Step 3: Add test**

Add to the tests module in `src/combat/skill.rs`:

```rust
    #[test]
    fn test_skill_from_chunk_library() {
        use crate::skills::ChunkLibrary;

        // Empty library = Novice
        let lib = ChunkLibrary::new();
        let skill = CombatSkill::from_chunk_library(&lib);
        assert_eq!(skill.level, SkillLevel::Novice);

        // Trained soldier
        let lib = ChunkLibrary::trained_soldier(0);
        let skill = CombatSkill::from_chunk_library(&lib);
        assert!(skill.level >= SkillLevel::Trained);

        // Veteran
        let lib = ChunkLibrary::veteran(0);
        let skill = CombatSkill::from_chunk_library(&lib);
        assert!(skill.level >= SkillLevel::Veteran);
    }
```

**Step 4: Run tests**

Run: `cargo test --lib combat::skill::tests -v`
Expected: PASS (6 tests including new one)

**Step 5: Commit**

```bash
git add src/combat/skill.rs
git commit -m "feat(combat): bridge SkillLevel from ChunkLibrary"
```

---

## Task 9: Archetype Integration

**Files:**
- Modify: `src/entity/species/human.rs`

**Step 1: Add chunk_libraries field**

Add to `HumanArchetype` struct (after `combat_states` field, around line 66):

```rust
    /// Skill chunk libraries for each entity
    pub chunk_libraries: Vec<crate::skills::ChunkLibrary>,
```

**Step 2: Update new() method**

Add to `HumanArchetype::new()`:

```rust
            chunk_libraries: Vec::new(),
```

**Step 3: Update spawn() method**

Add to `HumanArchetype::spawn()` (after `combat_states.push`):

```rust
        self.chunk_libraries.push(crate::skills::ChunkLibrary::with_basics());
```

**Step 4: Add test**

Add test to the tests module:

```rust
    #[test]
    fn test_human_has_chunk_library() {
        let mut archetype = HumanArchetype::new();
        let id = EntityId::new();
        archetype.spawn(id, "Conscript".into(), 0);

        assert_eq!(archetype.chunk_libraries.len(), 1);
        // Fresh spawn has no combat chunks
        assert!(!archetype.chunk_libraries[0].has_chunk(crate::skills::ChunkId::BasicSwing));
    }
```

**Step 5: Run tests**

Run: `cargo test --lib entity::species::human::tests -v`
Expected: PASS (including new test)

**Step 6: Commit**

```bash
git add src/entity/species/human.rs
git commit -m "feat(entity): add chunk_libraries to HumanArchetype"
```

---

## Task 10: Integration Tests

**Files:**
- Create: `tests/skills_integration.rs`

**Step 1: Write integration tests**

Create `tests/skills_integration.rs`:

```rust
//! Integration tests for hierarchical chunking skill system

use arc_citadel::skills::{
    calculate_attention_budget, process_learning, resolve_attack, ChunkId, ChunkLibrary,
    CombatContext, ContextTag,
};
use arc_citadel::combat::CombatSkill;

/// Test 1: Fresh conscript spends all attention on basic action
#[test]
fn test_conscript_attention_overload() {
    let mut lib = ChunkLibrary::new();
    lib.attention_budget = 1.0;

    let ctx = CombatContext::new().with_tag(ContextTag::InMelee);

    // First attack should succeed but spend most attention
    let result1 = resolve_attack(&mut lib, &ctx, 0);
    assert!(result1.is_success());

    // Should have very little attention left
    assert!(lib.attention_remaining() < 0.2);

    // Second attack should fail due to attention overload
    let result2 = resolve_attack(&mut lib, &ctx, 1);
    assert!(matches!(
        result2,
        arc_citadel::skills::ActionResult::AttentionOverload
    ));
}

/// Test 2: Veteran executes actions cheaply
#[test]
fn test_veteran_has_bandwidth() {
    let mut lib = ChunkLibrary::veteran(0);
    lib.attention_budget = 1.0;

    let ctx = CombatContext::new()
        .with_tag(ContextTag::InMelee)
        .with_tag(ContextTag::EnemyVisible);

    // Attack should be cheap
    let result1 = resolve_attack(&mut lib, &ctx, 100);
    assert!(result1.is_success());
    assert!(lib.attention_remaining() > 0.5);

    // Can afford multiple actions
    let result2 = resolve_attack(&mut lib, &ctx, 101);
    assert!(result2.is_success());
    assert!(lib.attention_remaining() > 0.0);
}

/// Test 3: SkillLevel reflects chunk mastery
#[test]
fn test_skill_level_progression() {
    // Conscript = Novice
    let lib = ChunkLibrary::new();
    let skill = CombatSkill::from_chunk_library(&lib);
    assert_eq!(skill.level, arc_citadel::combat::SkillLevel::Novice);

    // Trained soldier >= Trained
    let lib = ChunkLibrary::trained_soldier(0);
    let skill = CombatSkill::from_chunk_library(&lib);
    assert!(skill.level >= arc_citadel::combat::SkillLevel::Trained);

    // Veteran >= Veteran
    let lib = ChunkLibrary::veteran(0);
    let skill = CombatSkill::from_chunk_library(&lib);
    assert!(skill.level >= arc_citadel::combat::SkillLevel::Veteran);
}

/// Test 4: Learning increases encoding depth
#[test]
fn test_learning_progression() {
    let mut lib = ChunkLibrary::trained_soldier(0);

    let initial_depth = lib.get_chunk(ChunkId::BasicSwing).unwrap().encoding_depth;

    // Simulate 50 successful swings
    for tick in 1..=50 {
        lib.attention_budget = 1.0;
        lib.attention_spent = 0.0;

        let ctx = CombatContext::new().with_tag(ContextTag::InMelee);
        let _ = resolve_attack(&mut lib, &ctx, tick as u64);
        process_learning(&mut lib, tick as u64);
    }

    let final_depth = lib.get_chunk(ChunkId::BasicSwing).unwrap().encoding_depth;
    assert!(final_depth > initial_depth);
}

/// Test 5: Fatigue reduces attention budget
#[test]
fn test_fatigue_reduces_attention() {
    let fresh = calculate_attention_budget(0.0, 0.0, 0.0);
    let tired = calculate_attention_budget(0.5, 0.0, 0.0);
    let exhausted = calculate_attention_budget(1.0, 0.0, 0.0);

    assert_eq!(fresh, 1.0);
    assert!(tired < fresh);
    assert!(exhausted < tired);
    assert!(exhausted >= 0.2); // Minimum floor
}

/// Test 6: Chunk formation from prerequisites
#[test]
fn test_chunk_formation() {
    let mut lib = ChunkLibrary::new();

    // Give prerequisites at sufficient depth
    lib.set_chunk(
        ChunkId::BasicStance,
        arc_citadel::skills::PersonalChunkState {
            encoding_depth: 0.5,
            repetition_count: 50,
            last_used_tick: 0,
            formation_tick: 0,
        },
    );
    lib.set_chunk(
        ChunkId::BasicSwing,
        arc_citadel::skills::PersonalChunkState {
            encoding_depth: 0.5,
            repetition_count: 50,
            last_used_tick: 0,
            formation_tick: 0,
        },
    );

    assert!(!lib.has_chunk(ChunkId::AttackSequence));

    process_learning(&mut lib, 1000);

    assert!(lib.has_chunk(ChunkId::AttackSequence));
}

/// Test 7: Qualitative difference between novice and master
#[test]
fn test_qualitative_skill_difference() {
    let ctx = CombatContext::new()
        .with_tag(ContextTag::InMelee)
        .with_tag(ContextTag::EnemyVisible)
        .with_tag(ContextTag::MultipleEnemies);

    // Conscript: should struggle to execute even one action well
    let mut conscript = ChunkLibrary::new();
    conscript.attention_budget = 1.0;
    let result = resolve_attack(&mut conscript, &ctx, 0);
    let conscript_remaining = conscript.attention_remaining();
    let conscript_skill = result.skill_modifier();

    // Veteran: should execute easily with attention to spare
    let mut veteran = ChunkLibrary::veteran(0);
    veteran.attention_budget = 1.0;
    let result = resolve_attack(&mut veteran, &ctx, 0);
    let veteran_remaining = veteran.attention_remaining();
    let veteran_skill = result.skill_modifier();

    // Veteran should have:
    // 1. More attention remaining
    assert!(veteran_remaining > conscript_remaining + 0.3);

    // 2. Higher skill modifier (better execution quality)
    assert!(veteran_skill > conscript_skill + 0.3);
}
```

**Step 2: Run integration tests**

Run: `cargo test --test skills_integration -v`
Expected: PASS (7 tests)

**Step 3: Commit**

```bash
git add tests/skills_integration.rs
git commit -m "test: add integration tests for chunking skill system"
```

---

## Verification Commands

After completing all tasks, run:

```bash
# All unit tests
cargo test --lib skills:: -v

# Integration tests
cargo test --test skills_integration -v

# Full test suite
cargo test

# Check for warnings
cargo clippy -- -D warnings
```

Expected: All tests pass, no clippy warnings.

---

## Summary

| Task | Files Created/Modified | Tests |
|------|----------------------|-------|
| 1 | `src/skills/mod.rs`, `src/skills/chunk_id.rs`, `src/lib.rs` | 2 |
| 2 | `src/skills/context.rs` | 4 |
| 3 | `src/skills/definitions.rs` | 4 |
| 4 | `src/skills/library.rs` | 5 |
| 5 | `src/skills/attention.rs` | 6 |
| 6 | `src/skills/resolution.rs` | 6 |
| 7 | `src/skills/learning.rs` | 6 |
| 8 | `src/combat/skill.rs` | 1 |
| 9 | `src/entity/species/human.rs` | 1 |
| 10 | `tests/skills_integration.rs` | 7 |

**Total: 42 tests**

---

## Future Extensions (Not In Scope)

1. Other species archetypes (add `chunk_libraries` field)
2. Craft/Social/Movement domains (extend ChunkId, add definitions)
3. Species modifiers (elven deep time, dwarven craft bonuses)
4. Context transfer between weapons
5. TOML-based chunk definitions
6. UI for player-facing skill display
