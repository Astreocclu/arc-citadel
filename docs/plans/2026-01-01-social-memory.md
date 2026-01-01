# Social Memory System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement episodic social memory where relationships emerge from accumulated memories, not stored scores.

**Architecture:** Each entity has bounded relationship slots (200 max), each slot holds bounded memories (5 max) about one other entity. Disposition computed on-demand from memories. Encounter buffer accumulates salience before promotion to slots.

**Tech Stack:** Rust, existing SoA HumanArchetype pattern, integration with tick.rs simulation loop.

**Core Principle:** "The relationship IS the memories."

---

## Task 1: Add EventType and Valence enums

**Files:**
- Create: `src/entity/social/mod.rs`
- Create: `src/entity/social/event_types.rs`
- Modify: `src/entity/mod.rs`

**Step 1: Create social module structure**

```rust
// src/entity/social/mod.rs
pub mod event_types;
pub mod memory;
pub mod social_memory;

pub use event_types::{EventType, Valence};
pub use memory::RelationshipMemory;
pub use social_memory::{SocialMemory, RelationshipSlot, PendingEncounter, Disposition};
```

**Step 2: Create EventType and Valence**

```rust
// src/entity/social/event_types.rs
use serde::{Deserialize, Serialize};

/// Valence of a memory - positive or negative
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Valence {
    Positive,
    Negative,
}

/// Types of social events that create memories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    // Positive events
    AidReceived,      // They helped me
    AidGiven,         // I helped them
    GiftReceived,
    GiftGiven,
    SharedExperience, // Survived danger together
    Compliment,
    PromiseKept,

    // Negative events
    HarmReceived,     // They hurt me
    HarmGiven,        // I hurt them
    Insult,
    Theft,
    Betrayal,
    PromiseBroken,

    // Neutral but formative
    FirstMeeting,
    Transaction,      // Trade, business
    Observation,      // Witnessed them do something notable
}

impl EventType {
    /// Default valence for this event type
    pub fn default_valence(&self) -> Valence {
        match self {
            EventType::AidReceived | EventType::AidGiven |
            EventType::GiftReceived | EventType::GiftGiven |
            EventType::SharedExperience | EventType::Compliment |
            EventType::PromiseKept => Valence::Positive,

            EventType::HarmReceived | EventType::HarmGiven |
            EventType::Insult | EventType::Theft |
            EventType::Betrayal | EventType::PromiseBroken => Valence::Negative,

            // Neutral events default to positive (slight familiarity bonus)
            EventType::FirstMeeting | EventType::Transaction |
            EventType::Observation => Valence::Positive,
        }
    }

    /// Base intensity for this event type (0.0 to 1.0)
    pub fn base_intensity(&self) -> f32 {
        match self {
            EventType::Betrayal | EventType::SharedExperience => 0.9,
            EventType::HarmReceived | EventType::AidReceived => 0.7,
            EventType::GiftReceived | EventType::PromiseKept | EventType::PromiseBroken => 0.6,
            EventType::HarmGiven | EventType::AidGiven | EventType::GiftGiven => 0.5,
            EventType::Theft | EventType::Insult | EventType::Compliment => 0.4,
            EventType::FirstMeeting => 0.3,
            EventType::Transaction | EventType::Observation => 0.2,
        }
    }
}
```

**Step 3: Update entity module**

```rust
// src/entity/mod.rs - add to existing
pub mod social;
```

**Step 4: Run test to verify compilation**

Run: `cargo build --lib`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add src/entity/social/
git commit -m "feat: add EventType and Valence for social memory"
```

---

## Task 2: Add RelationshipMemory struct

**Files:**
- Create: `src/entity/social/memory.rs`

**Step 1: Write the failing test**

```rust
// src/entity/social/memory.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = RelationshipMemory::new(
            EventType::AidReceived,
            Valence::Positive,
            0.8,
            100,
        );
        assert_eq!(memory.event_type, EventType::AidReceived);
        assert_eq!(memory.valence, Valence::Positive);
        assert!((memory.intensity - 0.8).abs() < 0.01);
        assert!((memory.salience - 1.0).abs() < 0.01); // Starts at full salience
        assert_eq!(memory.tick_created, 100);
    }

    #[test]
    fn test_salience_decay() {
        let mut memory = RelationshipMemory::new(
            EventType::AidReceived,
            Valence::Positive,
            0.8,
            0,
        );

        // After 1000 ticks (1 day), salience should decay by ~2%
        memory.apply_decay(1000, 0.02);
        assert!(memory.salience < 1.0);
        assert!(memory.salience > 0.9);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib test_memory_creation`
Expected: FAIL (struct not defined)

**Step 3: Implement RelationshipMemory**

```rust
// src/entity/social/memory.rs
use serde::{Deserialize, Serialize};
use super::event_types::{EventType, Valence};

/// A single memory about an interaction with another entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipMemory {
    /// What happened
    pub event_type: EventType,
    /// Positive or negative experience
    pub valence: Valence,
    /// How impactful (0.0 to 1.0)
    pub intensity: f32,
    /// Current importance, decays over time (0.0 to 1.0)
    pub salience: f32,
    /// When this memory was formed
    pub tick_created: u64,
}

impl RelationshipMemory {
    pub fn new(event_type: EventType, valence: Valence, intensity: f32, tick: u64) -> Self {
        Self {
            event_type,
            valence,
            intensity: intensity.clamp(0.0, 1.0),
            salience: 1.0, // Starts at full salience
            tick_created: tick,
        }
    }

    /// Create memory with default valence and intensity from event type
    pub fn from_event(event_type: EventType, tick: u64) -> Self {
        Self::new(
            event_type,
            event_type.default_valence(),
            event_type.base_intensity(),
            tick,
        )
    }

    /// Apply decay based on days passed
    /// decay_rate is per-day rate (e.g., 0.02 = 2% per day)
    pub fn apply_decay(&mut self, current_tick: u64, decay_rate: f32) {
        let ticks_per_day = 1000; // TODO: Make configurable
        let days_passed = (current_tick.saturating_sub(self.tick_created)) as f32 / ticks_per_day as f32;
        self.salience = (1.0 - decay_rate).powf(days_passed).max(0.01);
    }

    /// Weighted importance: intensity * salience
    pub fn weighted_importance(&self) -> f32 {
        self.intensity * self.salience
    }
}

#[cfg(test)]
mod tests {
    // ... tests from above
}
```

**Step 4: Run tests**

Run: `cargo test --lib memory::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add src/entity/social/memory.rs
git commit -m "feat: add RelationshipMemory with salience decay"
```

---

## Task 3: Add RelationshipSlot and SocialMemory structs

**Files:**
- Create: `src/entity/social/social_memory.rs`

**Step 1: Write the failing test**

```rust
// In social_memory.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::EntityId;

    #[test]
    fn test_slot_keeps_top_5_by_salience() {
        let target = EntityId(42);
        let mut slot = RelationshipSlot::new(target, 0);

        // Add 6 memories with varying intensities
        for i in 0..6 {
            let memory = RelationshipMemory::new(
                EventType::Transaction,
                Valence::Positive,
                0.1 * (i as f32 + 1.0), // 0.1, 0.2, 0.3, 0.4, 0.5, 0.6
                i as u64 * 10,
            );
            slot.add_memory(memory, i as u64 * 10);
        }

        // Should only have 5 memories
        assert_eq!(slot.memories.len(), 5);

        // Lowest intensity (0.1) should have been evicted
        let min_intensity = slot.memories.iter()
            .map(|m| m.intensity)
            .fold(f32::MAX, f32::min);
        assert!(min_intensity > 0.15); // 0.1 was evicted
    }

    #[test]
    fn test_disposition_from_memories() {
        let target = EntityId(42);
        let mut slot = RelationshipSlot::new(target, 0);

        // Add positive memories
        slot.add_memory(RelationshipMemory::new(
            EventType::AidReceived, Valence::Positive, 0.8, 0
        ), 0);
        slot.add_memory(RelationshipMemory::new(
            EventType::GiftReceived, Valence::Positive, 0.6, 10
        ), 10);

        let disposition = slot.get_disposition();
        assert_eq!(disposition, Disposition::Favorable);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib test_slot_keeps_top_5`
Expected: FAIL (structs not defined)

**Step 3: Implement structs**

```rust
// src/entity/social/social_memory.rs
use serde::{Deserialize, Serialize};
use crate::core::types::EntityId;
use super::memory::RelationshipMemory;
use super::event_types::{EventType, Valence};

/// How an entity feels about another based on memories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Disposition {
    Hostile,     // net < -0.5
    Suspicious,  // -0.5 <= net < -0.1
    Neutral,     // -0.1 <= net <= 0.1
    Friendly,    // 0.1 < net <= 0.5
    Favorable,   // net > 0.5
    Unknown,     // No memories
}

/// A known entity with bounded memory buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipSlot {
    /// Who this relationship is about
    pub target_id: EntityId,
    /// Bounded memory buffer, sorted by salience (highest first)
    pub memories: Vec<RelationshipMemory>,
    /// When we first met
    pub first_contact: u64,
    /// Most recent interaction
    pub last_contact: u64,
    /// Total interactions ever (even if memories evicted)
    pub interaction_count: u32,
}

impl RelationshipSlot {
    const MAX_MEMORIES: usize = 5;

    pub fn new(target_id: EntityId, first_contact: u64) -> Self {
        Self {
            target_id,
            memories: Vec::with_capacity(Self::MAX_MEMORIES),
            first_contact,
            last_contact: first_contact,
            interaction_count: 0,
        }
    }

    /// Add memory, keeping only top 5 by salience
    pub fn add_memory(&mut self, memory: RelationshipMemory, current_tick: u64) {
        self.last_contact = current_tick;
        self.interaction_count += 1;

        self.memories.push(memory);

        // Sort by weighted importance (descending)
        self.memories.sort_by(|a, b| {
            b.weighted_importance()
                .partial_cmp(&a.weighted_importance())
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Keep only top 5
        self.memories.truncate(Self::MAX_MEMORIES);
    }

    /// Apply decay to all memories and re-sort
    pub fn apply_decay(&mut self, current_tick: u64, decay_rate: f32) {
        for memory in &mut self.memories {
            memory.apply_decay(current_tick, decay_rate);
        }

        // Re-sort after decay
        self.memories.sort_by(|a, b| {
            b.weighted_importance()
                .partial_cmp(&a.weighted_importance())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Compute disposition from memories (no stored score)
    pub fn get_disposition(&self) -> Disposition {
        if self.memories.is_empty() {
            return Disposition::Unknown;
        }

        let positive: f32 = self.memories.iter()
            .filter(|m| m.valence == Valence::Positive)
            .map(|m| m.weighted_importance())
            .sum();

        let negative: f32 = self.memories.iter()
            .filter(|m| m.valence == Valence::Negative)
            .map(|m| m.weighted_importance())
            .sum();

        let net = positive - negative;

        match net {
            n if n > 0.5 => Disposition::Favorable,
            n if n > 0.1 => Disposition::Friendly,
            n if n < -0.5 => Disposition::Hostile,
            n if n < -0.1 => Disposition::Suspicious,
            _ => Disposition::Neutral,
        }
    }

    /// Calculate relationship strength for eviction decisions
    pub fn strength(&self, current_tick: u64, params: &SocialMemoryParams) -> f32 {
        let ticks_per_day = 1000;
        let days_since = (current_tick.saturating_sub(self.last_contact)) as f32 / ticks_per_day as f32;
        let recency_score = 1.0 / (1.0 + days_since * 0.1);

        let intensity_score: f32 = self.memories.iter()
            .map(|m| m.weighted_importance())
            .sum::<f32>() / Self::MAX_MEMORIES as f32;

        let depth_score = (self.interaction_count as f32 / 20.0).min(1.0);

        recency_score * params.recency_weight +
        intensity_score * params.intensity_weight +
        depth_score * params.interaction_count_weight
    }
}

/// Pre-threshold encounter tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingEncounter {
    pub target_id: EntityId,
    pub accumulated_salience: f32,
    pub encounter_count: u32,
    pub most_recent_tick: u64,
    pub most_significant: Option<(EventType, f32)>, // (event, intensity)
}

impl PendingEncounter {
    pub fn new(target_id: EntityId, tick: u64) -> Self {
        Self {
            target_id,
            accumulated_salience: 0.0,
            encounter_count: 0,
            most_recent_tick: tick,
            most_significant: None,
        }
    }

    pub fn add_encounter(&mut self, event_type: EventType, intensity: f32, tick: u64) {
        self.accumulated_salience += intensity;
        self.encounter_count += 1;
        self.most_recent_tick = tick;

        // Track most significant event
        if self.most_significant.map_or(true, |(_, i)| intensity > i) {
            self.most_significant = Some((event_type, intensity));
        }
    }
}

/// Species-specific social memory parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMemoryParams {
    pub max_relationship_slots: usize,
    pub memories_per_slot: usize,
    pub encounter_buffer_size: usize,
    pub slot_allocation_threshold: f32,
    pub memory_importance_floor: f32,
    pub memory_salience_decay: f32,
    pub recency_weight: f32,
    pub intensity_weight: f32,
    pub interaction_count_weight: f32,
}

impl Default for SocialMemoryParams {
    fn default() -> Self {
        // Human defaults from spec
        Self {
            max_relationship_slots: 200,
            memories_per_slot: 5,
            encounter_buffer_size: 50,
            slot_allocation_threshold: 0.3,
            memory_importance_floor: 0.2,
            memory_salience_decay: 0.02, // 2% per day
            recency_weight: 0.4,
            intensity_weight: 0.4,
            interaction_count_weight: 0.2,
        }
    }
}

/// Complete social memory for one entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialMemory {
    /// Known entities (bounded)
    pub slots: Vec<RelationshipSlot>,
    /// Pre-threshold encounters
    pub encounter_buffer: Vec<PendingEncounter>,
    /// Species parameters
    pub params: SocialMemoryParams,
}

impl SocialMemory {
    pub fn new() -> Self {
        Self::with_params(SocialMemoryParams::default())
    }

    pub fn with_params(params: SocialMemoryParams) -> Self {
        Self {
            slots: Vec::with_capacity(params.max_relationship_slots),
            encounter_buffer: Vec::with_capacity(params.encounter_buffer_size),
            params,
        }
    }

    /// Find slot for a known entity
    pub fn find_slot(&self, target: EntityId) -> Option<&RelationshipSlot> {
        self.slots.iter().find(|s| s.target_id == target)
    }

    pub fn find_slot_mut(&mut self, target: EntityId) -> Option<&mut RelationshipSlot> {
        self.slots.iter_mut().find(|s| s.target_id == target)
    }

    /// Get disposition toward entity (computed from memories)
    pub fn get_disposition(&self, target: EntityId) -> Disposition {
        self.find_slot(target)
            .map(|slot| slot.get_disposition())
            .unwrap_or(Disposition::Unknown)
    }

    /// Record an encounter (may go to buffer or directly to slot)
    pub fn record_encounter(
        &mut self,
        target: EntityId,
        event_type: EventType,
        intensity: f32,
        current_tick: u64,
    ) {
        // Below importance floor? Ignore
        if intensity < self.params.memory_importance_floor {
            return;
        }

        // Already known? Add memory directly
        if let Some(slot) = self.find_slot_mut(target) {
            let memory = RelationshipMemory::new(
                event_type,
                event_type.default_valence(),
                intensity,
                current_tick,
            );
            slot.add_memory(memory, current_tick);
            return;
        }

        // Update or create pending encounter
        if let Some(encounter) = self.encounter_buffer.iter_mut()
            .find(|e| e.target_id == target)
        {
            encounter.add_encounter(event_type, intensity, current_tick);

            // Check promotion threshold
            if encounter.accumulated_salience >= self.params.slot_allocation_threshold {
                self.promote_encounter(target, current_tick);
            }
        } else {
            // New encounter
            let mut encounter = PendingEncounter::new(target, current_tick);
            encounter.add_encounter(event_type, intensity, current_tick);

            // Check if immediately significant enough
            if encounter.accumulated_salience >= self.params.slot_allocation_threshold {
                self.promote_encounter(target, current_tick);
            } else {
                // Add to buffer (evict oldest if full)
                if self.encounter_buffer.len() >= self.params.encounter_buffer_size {
                    // Remove oldest by most_recent_tick
                    if let Some(oldest_idx) = self.encounter_buffer.iter()
                        .enumerate()
                        .min_by_key(|(_, e)| e.most_recent_tick)
                        .map(|(i, _)| i)
                    {
                        self.encounter_buffer.remove(oldest_idx);
                    }
                }
                self.encounter_buffer.push(encounter);
            }
        }
    }

    /// Promote encounter to relationship slot
    fn promote_encounter(&mut self, target: EntityId, current_tick: u64) {
        // Find and remove from buffer
        let encounter = self.encounter_buffer.iter()
            .position(|e| e.target_id == target)
            .map(|i| self.encounter_buffer.remove(i));

        // Ensure we have room (evict if needed)
        if self.slots.len() >= self.params.max_relationship_slots {
            self.evict_weakest_slot(current_tick);
        }

        // Create new slot
        let mut slot = RelationshipSlot::new(target, current_tick);

        // Seed with most significant event from encounter
        if let Some(enc) = encounter {
            if let Some((event_type, intensity)) = enc.most_significant {
                let memory = RelationshipMemory::new(
                    event_type,
                    event_type.default_valence(),
                    intensity,
                    current_tick,
                );
                slot.add_memory(memory, current_tick);
            }
        }

        self.slots.push(slot);
    }

    /// Evict weakest relationship to make room
    fn evict_weakest_slot(&mut self, current_tick: u64) {
        if self.slots.is_empty() {
            return;
        }

        let weakest_idx = self.slots.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                a.strength(current_tick, &self.params)
                    .partial_cmp(&b.strength(current_tick, &self.params))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
            .unwrap_or(0);

        self.slots.remove(weakest_idx);
    }

    /// Decay all memories (call once per day)
    pub fn apply_decay(&mut self, current_tick: u64) {
        for slot in &mut self.slots {
            slot.apply_decay(current_tick, self.params.memory_salience_decay);
        }

        // Decay encounter buffer salience
        for encounter in &mut self.encounter_buffer {
            encounter.accumulated_salience *= 1.0 - self.params.memory_salience_decay;
        }

        // Remove near-zero encounters
        self.encounter_buffer.retain(|e| e.accumulated_salience > 0.01);
    }
}

impl Default for SocialMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ... tests from Step 1
}
```

**Step 4: Run tests**

Run: `cargo test --lib social_memory::tests`
Expected: PASS

**Step 5: Commit**

```bash
git add src/entity/social/social_memory.rs
git commit -m "feat: add RelationshipSlot and SocialMemory with encounter buffer"
```

---

## Task 4: Add SocialMemory to HumanArchetype

**Files:**
- Modify: `src/entity/species/human.rs`
- Modify: `src/ecs/world.rs`

**Step 1: Write the failing test**

```rust
// In human.rs tests
#[test]
fn test_human_has_social_memory() {
    let mut archetype = HumanArchetype::new();
    archetype.spawn("Alice".into(), Vec2::new(0.0, 0.0));

    assert_eq!(archetype.social_memories.len(), 1);
    assert_eq!(archetype.social_memories[0].slots.len(), 0); // Empty initially
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib test_human_has_social_memory`
Expected: FAIL (field doesn't exist)

**Step 3: Add social_memories to HumanArchetype**

```rust
// In human.rs - add to struct
use crate::entity::social::SocialMemory;

pub struct HumanArchetype {
    // ... existing fields
    pub social_memories: Vec<SocialMemory>,
}

// In HumanArchetype::new()
social_memories: Vec::new(),

// In spawn() method - add after other fields
self.social_memories.push(SocialMemory::new());
```

**Step 4: Run tests**

Run: `cargo test --lib test_human_has_social_memory`
Expected: PASS

**Step 5: Update World spawn_human to match**

Verify `world.spawn_human()` still works.

Run: `cargo test --lib`
Expected: All tests pass

**Step 6: Commit**

```bash
git add src/entity/species/human.rs
git commit -m "feat: add social_memories to HumanArchetype"
```

---

## Task 5: Add memory creation from task execution

**Files:**
- Modify: `src/simulation/tick.rs`

**Step 1: Write the failing test**

```rust
// In tick.rs tests
#[test]
fn test_help_action_creates_memories() {
    let mut world = World::new();
    let alice = world.spawn_human("Alice".into());
    let bob = world.spawn_human("Bob".into());

    let alice_idx = world.humans.index_of(alice).unwrap();
    let bob_idx = world.humans.index_of(bob).unwrap();

    // Position them together
    world.humans.positions[alice_idx] = Vec2::new(0.0, 0.0);
    world.humans.positions[bob_idx] = Vec2::new(1.0, 0.0);

    // Alice helps Bob
    let task = Task {
        action: ActionId::Help,
        target_entity: Some(bob),
        target_position: None,
        priority: TaskPriority::Normal,
        created_tick: 0,
        progress: 0.0,
        source: TaskSource::Autonomous,
    };
    world.humans.task_queues[alice_idx].push(task);

    // Run enough ticks for Help to complete
    for _ in 0..35 {
        run_simulation_tick(&mut world);
    }

    // Bob should remember being helped by Alice
    let bob_memory = &world.humans.social_memories[bob_idx];
    let disposition = bob_memory.get_disposition(alice);
    assert!(disposition == Disposition::Friendly || disposition == Disposition::Favorable);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib test_help_action_creates_memories`
Expected: FAIL (no memory created)

**Step 3: Add memory creation in execute_tasks**

```rust
// In tick.rs, modify execute_tasks function
// After completing a social action, create memories for both parties

fn execute_tasks(world: &mut World) {
    let current_tick = world.tick;
    let living_indices: Vec<usize> = world.humans.iter_living().collect();

    for &idx in &living_indices {
        // ... existing task execution logic ...

        // When task completes (progress >= 1.0), check for social memory creation
        if let Some(task) = world.humans.task_queues[idx].current() {
            if task.progress >= 1.0 {
                create_social_memory_from_task(world, idx, task, current_tick);
            }
        }
    }
}

fn create_social_memory_from_task(world: &mut World, actor_idx: usize, task: &Task, tick: u64) {
    use crate::entity::social::{EventType, Valence};

    let actor_id = world.humans.ids[actor_idx];

    // Only social actions with targets create memories
    let target_id = match task.target_entity {
        Some(id) => id,
        None => return,
    };

    let target_idx = match world.humans.index_of(target_id) {
        Some(idx) => idx,
        None => return,
    };

    // Map action to event types for actor and target
    let (actor_event, target_event) = match task.action {
        ActionId::Help => (EventType::AidGiven, EventType::AidReceived),
        ActionId::TalkTo => (EventType::Transaction, EventType::Transaction),
        ActionId::Trade => (EventType::Transaction, EventType::Transaction),
        ActionId::Attack => (EventType::HarmGiven, EventType::HarmReceived),
        _ => return, // Non-social action
    };

    // Record for actor (remembers helping/attacking target)
    world.humans.social_memories[actor_idx].record_encounter(
        target_id,
        actor_event,
        actor_event.base_intensity(),
        tick,
    );

    // Record for target (remembers being helped/attacked by actor)
    world.humans.social_memories[target_idx].record_encounter(
        actor_id,
        target_event,
        target_event.base_intensity(),
        tick,
    );
}
```

**Step 4: Run tests**

Run: `cargo test --lib test_help_action_creates_memories`
Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat: social actions create memories for both parties"
```

---

## Task 6: Add memory creation from thoughts

**Files:**
- Modify: `src/simulation/tick.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_intense_thoughts_create_memories() {
    let mut world = World::new();
    let alice = world.spawn_human("Alice".into());
    let bob = world.spawn_human("Bob".into());

    let alice_idx = world.humans.index_of(alice).unwrap();

    // Manually add an intense thought about Bob
    use crate::entity::thoughts::{Thought, ThoughtType, CauseType};
    let thought = Thought {
        thought_type: ThoughtType::Observation,
        cause_type: CauseType::Entity,
        cause_entity: Some(bob),
        cause_description: "Bob did something amazing".into(),
        intensity: 0.9, // Very intense
        valence: 0.8,
        tick_created: 0,
    };
    world.humans.thoughts[alice_idx].add(thought);

    // Run tick to process thoughts
    run_simulation_tick(&mut world);

    // Alice should remember Bob
    let alice_memory = &world.humans.social_memories[alice_idx];
    assert!(alice_memory.find_slot(bob).is_some());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib test_intense_thoughts_create_memories`
Expected: FAIL (no memory created)

**Step 3: Add thought-to-memory conversion in tick**

```rust
// In tick.rs, add new function and call it after generate_thoughts

fn convert_thoughts_to_memories(world: &mut World) {
    use crate::entity::social::EventType;

    let current_tick = world.tick;
    const THOUGHT_MEMORY_THRESHOLD: f32 = 0.7;

    let living_indices: Vec<usize> = world.humans.iter_living().collect();

    for &idx in &living_indices {
        // Collect thoughts that should become memories
        let thoughts_to_convert: Vec<_> = world.humans.thoughts[idx]
            .thoughts
            .iter()
            .filter(|t| t.intensity >= THOUGHT_MEMORY_THRESHOLD && t.cause_entity.is_some())
            .map(|t| (t.cause_entity.unwrap(), t.intensity, t.valence))
            .collect();

        // Create memories
        for (target_id, intensity, valence) in thoughts_to_convert {
            let event_type = if valence > 0.0 {
                EventType::Observation // Positive observation
            } else {
                EventType::Observation // Can add more specific types later
            };

            world.humans.social_memories[idx].record_encounter(
                target_id,
                event_type,
                intensity,
                current_tick,
            );
        }
    }
}

// In run_simulation_tick, add after generate_thoughts:
// 3. Generate thoughts
generate_thoughts(&mut world);

// 4. Convert intense thoughts to memories
convert_thoughts_to_memories(&mut world);

// 5. Decay thoughts (was 4)
decay_thoughts(&mut world);
```

**Step 4: Run tests**

Run: `cargo test --lib test_intense_thoughts_create_memories`
Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat: intense thoughts about entities create memories"
```

---

## Task 7: Add memory decay to tick

**Files:**
- Modify: `src/simulation/tick.rs`
- Modify: `src/ecs/world.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_memory_decay_happens_daily() {
    let mut world = World::new();
    let alice = world.spawn_human("Alice".into());
    let bob = world.spawn_human("Bob".into());

    let alice_idx = world.humans.index_of(alice).unwrap();

    // Create a memory
    world.humans.social_memories[alice_idx].record_encounter(
        bob,
        EventType::AidReceived,
        0.8,
        0,
    );

    // Get initial salience
    let initial_salience = world.humans.social_memories[alice_idx]
        .find_slot(bob).unwrap()
        .memories[0].salience;

    // Run 1001 ticks (1 day + 1)
    for _ in 0..1001 {
        run_simulation_tick(&mut world);
    }

    // Salience should have decayed
    let final_salience = world.humans.social_memories[alice_idx]
        .find_slot(bob).unwrap()
        .memories[0].salience;

    assert!(final_salience < initial_salience);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib test_memory_decay_happens_daily`
Expected: FAIL (salience unchanged)

**Step 3: Add TICKS_PER_DAY constant and decay phase**

```rust
// In world.rs or config.rs
pub const TICKS_PER_DAY: u64 = 1000;

// In tick.rs, add decay function
fn decay_social_memories(world: &mut World) {
    let current_tick = world.tick;

    // Only decay once per day
    if current_tick % TICKS_PER_DAY != 0 {
        return;
    }

    let living_indices: Vec<usize> = world.humans.iter_living().collect();

    for &idx in &living_indices {
        world.humans.social_memories[idx].apply_decay(current_tick);
    }
}

// In run_simulation_tick, add at end:
// 9. Decay social memories (once per day)
decay_social_memories(&mut world);
```

**Step 4: Run tests**

Run: `cargo test --lib test_memory_decay_happens_daily`
Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs src/ecs/world.rs
git commit -m "feat: social memory decay runs once per simulation day"
```

---

## Task 8: Add disposition to Perception

**Files:**
- Modify: `src/simulation/perception.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_perception_includes_disposition() {
    let mut world = World::new();
    let alice = world.spawn_human("Alice".into());
    let bob = world.spawn_human("Bob".into());

    let alice_idx = world.humans.index_of(alice).unwrap();
    let bob_idx = world.humans.index_of(bob).unwrap();

    // Position them near each other
    world.humans.positions[alice_idx] = Vec2::new(0.0, 0.0);
    world.humans.positions[bob_idx] = Vec2::new(5.0, 0.0);

    // Alice has positive memories of Bob
    world.humans.social_memories[alice_idx].record_encounter(
        bob,
        EventType::AidReceived,
        0.8,
        0,
    );

    // Run perception
    run_simulation_tick(&mut world);

    // Check Alice's perception of Bob
    // Perception should include disposition info
    // (Implementation depends on PerceivedEntity structure)
}
```

**Step 2: Add disposition lookup in perception**

```rust
// In perception.rs, add to PerceivedEntity
#[derive(Debug, Clone)]
pub struct PerceivedEntity {
    pub entity_id: EntityId,
    pub position: Vec2,
    pub distance: f32,
    pub disposition: Disposition, // NEW
}

// In perception building logic, add disposition lookup:
fn build_perception(
    observer_idx: usize,
    world: &World,
    // ... other params
) -> Perception {
    // ... existing perception building ...

    for entity_id in nearby_entities {
        let disposition = world.humans.social_memories[observer_idx]
            .get_disposition(entity_id);

        perceived_entities.push(PerceivedEntity {
            entity_id,
            position,
            distance,
            disposition,
        });
    }

    // ...
}
```

**Step 3: Run tests**

Run: `cargo test --lib`
Expected: PASS

**Step 4: Commit**

```bash
git add src/simulation/perception.rs
git commit -m "feat: perception includes disposition from social memory"
```

---

## Task 9: Add disposition influence to action selection

**Files:**
- Modify: `src/simulation/action_select.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_hostile_disposition_influences_action() {
    let mut world = World::new();
    let alice = world.spawn_human("Alice".into());
    let bob = world.spawn_human("Bob".into());

    let alice_idx = world.humans.index_of(alice).unwrap();
    let bob_idx = world.humans.index_of(bob).unwrap();

    // Position them near each other
    world.humans.positions[alice_idx] = Vec2::new(0.0, 0.0);
    world.humans.positions[bob_idx] = Vec2::new(5.0, 0.0);

    // Alice has hostile memories of Bob
    world.humans.social_memories[alice_idx].record_encounter(
        bob,
        EventType::HarmReceived,
        0.9,
        0,
    );
    world.humans.social_memories[alice_idx].record_encounter(
        bob,
        EventType::Betrayal,
        0.9,
        0,
    );

    // Make Alice's safety need critical
    world.humans.needs[alice_idx].safety = 0.9;

    // Run perception and action selection
    run_simulation_tick(&mut world);

    // Alice should choose defensive/flee action, not help
    let task = world.humans.task_queues[alice_idx].current();
    assert!(task.is_some());
    // Hostile entity nearby should trigger safety response
}
```

**Step 2: Modify action selection to use disposition**

```rust
// In action_select.rs, add to SelectionContext
pub struct SelectionContext {
    // ... existing fields
    pub perceived_entities_with_disposition: Vec<(EntityId, Disposition)>,
}

// In select_actions logic
fn select_action(ctx: &SelectionContext) -> Option<ActionId> {
    // Check for hostile entities nearby
    let has_hostile = ctx.perceived_entities_with_disposition
        .iter()
        .any(|(_, d)| *d == Disposition::Hostile);

    if has_hostile && ctx.observer_needs.safety > 0.5 {
        return Some(ActionId::Flee);
    }

    // Check for friendly entities for social actions
    let friendly_target = ctx.perceived_entities_with_disposition
        .iter()
        .find(|(_, d)| *d == Disposition::Friendly || *d == Disposition::Favorable)
        .map(|(id, _)| *id);

    if let Some(target) = friendly_target {
        if ctx.observer_needs.social > 0.5 {
            return Some(ActionId::TalkTo); // with target
        }
    }

    // ... rest of action selection
}
```

**Step 3: Run tests**

Run: `cargo test --lib test_hostile_disposition_influences_action`
Expected: PASS

**Step 4: Commit**

```bash
git add src/simulation/action_select.rs
git commit -m "feat: disposition influences action selection"
```

---

## Task 10: Integration test - full social memory cycle

**Files:**
- Modify: `tests/emergence_tests.rs`

**Step 1: Write comprehensive integration test**

```rust
#[test]
fn test_social_memory_emergence() {
    let mut world = World::new();

    // Create a small community
    let alice = world.spawn_human("Alice".into());
    let bob = world.spawn_human("Bob".into());
    let charlie = world.spawn_human("Charlie".into());

    let alice_idx = world.humans.index_of(alice).unwrap();
    let bob_idx = world.humans.index_of(bob).unwrap();
    let charlie_idx = world.humans.index_of(charlie).unwrap();

    // Position them in a triangle
    world.humans.positions[alice_idx] = Vec2::new(0.0, 0.0);
    world.humans.positions[bob_idx] = Vec2::new(10.0, 0.0);
    world.humans.positions[charlie_idx] = Vec2::new(5.0, 8.0);

    // Add food zone so they can survive
    world.add_food_zone(Vec2::new(5.0, 4.0), 20.0, Abundance::Unlimited);

    // Run simulation for extended period
    for _ in 0..5000 {
        run_simulation_tick(&mut world);
    }

    // After extended interaction, entities should know each other
    let alice_memory = &world.humans.social_memories[alice_idx];

    // Alice should have formed relationships
    assert!(alice_memory.slots.len() >= 1, "Alice should know at least one other entity");

    // Check that dispositions are computed correctly
    for slot in &alice_memory.slots {
        let disposition = slot.get_disposition();
        // Should have some disposition, not Unknown
        assert_ne!(disposition, Disposition::Unknown);
    }
}

#[test]
fn test_relationship_eviction_when_full() {
    let mut world = World::new();

    // Create one observer
    let observer = world.spawn_human("Observer".into());
    let observer_idx = world.humans.index_of(observer).unwrap();

    // Create more entities than slot capacity (200+)
    let mut entity_ids = vec![];
    for i in 0..210 {
        let id = world.spawn_human(format!("Entity{}", i));
        entity_ids.push(id);

        // Position near observer
        let idx = world.humans.index_of(id).unwrap();
        world.humans.positions[idx] = Vec2::new(i as f32, 0.0);
    }

    // Manually create memories for all entities
    for (i, &entity_id) in entity_ids.iter().enumerate() {
        world.humans.social_memories[observer_idx].record_encounter(
            entity_id,
            EventType::FirstMeeting,
            0.5,
            i as u64,
        );
    }

    // Should not exceed max slots
    let slots = &world.humans.social_memories[observer_idx].slots;
    assert!(slots.len() <= 200, "Should not exceed 200 relationship slots");
}
```

**Step 2: Run tests**

Run: `cargo test --test emergence_tests test_social_memory`
Expected: PASS

**Step 3: Commit**

```bash
git add tests/emergence_tests.rs
git commit -m "test: social memory emergence and eviction integration tests"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | EventType and Valence enums | src/entity/social/event_types.rs |
| 2 | RelationshipMemory struct | src/entity/social/memory.rs |
| 3 | RelationshipSlot and SocialMemory | src/entity/social/social_memory.rs |
| 4 | Add to HumanArchetype | src/entity/species/human.rs |
| 5 | Memory from task execution | src/simulation/tick.rs |
| 6 | Memory from thoughts | src/simulation/tick.rs |
| 7 | Memory decay phase | src/simulation/tick.rs |
| 8 | Disposition in perception | src/simulation/perception.rs |
| 9 | Disposition in action selection | src/simulation/action_select.rs |
| 10 | Integration tests | tests/emergence_tests.rs |

**Core principle preserved:** "The relationship IS the memories."

---

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-01 | Initial plan |
