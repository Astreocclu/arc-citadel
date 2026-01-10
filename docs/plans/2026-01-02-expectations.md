# Expectation-Based Social Dynamics Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement expectation formation, violation detection, and emergent role derivation. Entities form predictions about others based on observed behavior; violations generate negative thoughts; roles emerge from clustered expectations.

**Architecture:** Extend RelationshipSlot with expectations field. Add Calendar for time-of-day. Add per-entity EventBuffer. Integrate into tick.rs perception phase.

**Tech Stack:** Rust, existing SoA layout, extends social memory system.

---

## Task 1: Add Calendar System

Create calendar with time-of-day periods for LocationDuring patterns.

**Files:**
- Create: `src/core/calendar.rs`
- Modify: `src/core/mod.rs`
- Modify: `src/ecs/world.rs`

**Step 1: Write the failing test**

```rust
// In src/core/calendar.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_period_from_hour() {
        assert_eq!(TimePeriod::from_hour(6), TimePeriod::Morning);
        assert_eq!(TimePeriod::from_hour(11), TimePeriod::Morning);
        assert_eq!(TimePeriod::from_hour(12), TimePeriod::Afternoon);
        assert_eq!(TimePeriod::from_hour(17), TimePeriod::Afternoon);
        assert_eq!(TimePeriod::from_hour(18), TimePeriod::Evening);
        assert_eq!(TimePeriod::from_hour(21), TimePeriod::Evening);
        assert_eq!(TimePeriod::from_hour(22), TimePeriod::Night);
        assert_eq!(TimePeriod::from_hour(5), TimePeriod::Night);
    }

    #[test]
    fn test_calendar_advances() {
        let mut cal = Calendar::new(1000); // ticks per day
        assert_eq!(cal.current_tick(), 0);
        assert_eq!(cal.current_day(), 0);

        cal.advance();
        assert_eq!(cal.current_tick(), 1);

        // Advance to next day
        for _ in 0..999 {
            cal.advance();
        }
        assert_eq!(cal.current_tick(), 1000);
        assert_eq!(cal.current_day(), 1);
    }

    #[test]
    fn test_calendar_time_period() {
        let mut cal = Calendar::new(1000); // 1000 ticks per day

        // At tick 0, hour 0 = Night
        assert_eq!(cal.current_time_period(), TimePeriod::Night);

        // Advance to morning (6am = 250 ticks at 1000/day)
        for _ in 0..250 {
            cal.advance();
        }
        assert_eq!(cal.current_time_period(), TimePeriod::Morning);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib core::calendar`
Expected: FAIL with "can't find crate"

**Step 3: Write implementation**

```rust
// src/core/calendar.rs
use serde::{Deserialize, Serialize};

/// Time of day periods for LocationDuring expectations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimePeriod {
    Morning,    // 06:00-12:00
    Afternoon,  // 12:00-18:00
    Evening,    // 18:00-22:00
    Night,      // 22:00-06:00
}

impl TimePeriod {
    pub fn from_hour(hour: u32) -> Self {
        match hour {
            6..=11 => TimePeriod::Morning,
            12..=17 => TimePeriod::Afternoon,
            18..=21 => TimePeriod::Evening,
            _ => TimePeriod::Night, // 22-23, 0-5
        }
    }
}

/// Calendar tracks simulation time with day/hour granularity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Calendar {
    tick: u64,
    ticks_per_day: u64,
}

impl Calendar {
    pub fn new(ticks_per_day: u64) -> Self {
        Self {
            tick: 0,
            ticks_per_day,
        }
    }

    pub fn advance(&mut self) {
        self.tick += 1;
    }

    pub fn current_tick(&self) -> u64 {
        self.tick
    }

    pub fn current_day(&self) -> u64 {
        self.tick / self.ticks_per_day
    }

    pub fn current_hour(&self) -> u32 {
        let tick_in_day = self.tick % self.ticks_per_day;
        let hours_per_day = 24;
        ((tick_in_day * hours_per_day) / self.ticks_per_day) as u32
    }

    pub fn current_time_period(&self) -> TimePeriod {
        TimePeriod::from_hour(self.current_hour())
    }

    pub fn ticks_per_day(&self) -> u64 {
        self.ticks_per_day
    }
}

impl Default for Calendar {
    fn default() -> Self {
        Self::new(1000) // Match existing TICKS_PER_DAY
    }
}
```

**Step 4: Add to mod.rs and world.rs**

```rust
// In src/core/mod.rs, add:
pub mod calendar;
pub use calendar::{Calendar, TimePeriod};

// In src/ecs/world.rs, add field to World:
pub calendar: Calendar,

// In World::new(), initialize:
calendar: Calendar::default(),
```

**Step 5: Run tests**

Run: `cargo test --lib core::calendar`
Expected: PASS

**Step 6: Commit**

```bash
git add src/core/calendar.rs src/core/mod.rs src/ecs/world.rs
git commit -m "feat: add Calendar with time-of-day periods"
```

---

## Task 2: Add ServiceType and TraitIndicator Enums

Create enums for expectation pattern types.

**Files:**
- Create: `src/entity/social/service_types.rs`
- Modify: `src/entity/social/mod.rs`

**Step 1: Write the failing test**

```rust
// In src/entity/social/service_types.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::catalog::ActionId;

    #[test]
    fn test_service_from_action() {
        assert_eq!(ServiceType::from_action(ActionId::Craft), Some(ServiceType::Crafting));
        assert_eq!(ServiceType::from_action(ActionId::Trade), Some(ServiceType::Trading));
        assert_eq!(ServiceType::from_action(ActionId::Help), Some(ServiceType::Helping));
        assert_eq!(ServiceType::from_action(ActionId::Gather), Some(ServiceType::Labor));
        assert_eq!(ServiceType::from_action(ActionId::Build), Some(ServiceType::Labor));
        assert_eq!(ServiceType::from_action(ActionId::MoveTo), None); // Not a service
    }

    #[test]
    fn test_trait_from_action() {
        assert_eq!(TraitIndicator::from_action(ActionId::Help), Some(TraitIndicator::Generous));
        assert_eq!(TraitIndicator::from_action(ActionId::Flee), Some(TraitIndicator::Peaceful));
        assert_eq!(TraitIndicator::from_action(ActionId::Attack), Some(TraitIndicator::Aggressive));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib entity::social::service_types`
Expected: FAIL

**Step 3: Write implementation**

```rust
// src/entity/social/service_types.rs
use serde::{Deserialize, Serialize};
use crate::actions::catalog::ActionId;

/// Types of services an entity might provide
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceType {
    Crafting,       // Making things
    Trading,        // Buying/selling
    Helping,        // Assisting others
    Labor,          // General work (gather, build)
    Protection,     // Guarding, defending
    Teaching,       // Instruction
    Healing,        // Medical care
}

impl ServiceType {
    pub fn from_action(action: ActionId) -> Option<Self> {
        match action {
            ActionId::Craft => Some(ServiceType::Crafting),
            ActionId::Trade => Some(ServiceType::Trading),
            ActionId::Help => Some(ServiceType::Helping),
            ActionId::Gather | ActionId::Build | ActionId::Repair => Some(ServiceType::Labor),
            ActionId::Defend | ActionId::HoldPosition => Some(ServiceType::Protection),
            // Add more mappings as actions are added
            _ => None,
        }
    }
}

/// Observable behavioral traits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TraitIndicator {
    Reliable,       // Does what they say
    Unreliable,     // Frequently fails commitments
    Generous,       // Gives beyond obligation
    Stingy,         // Minimal compliance
    Aggressive,     // Quick to violence
    Peaceful,       // Avoids conflict
    Punctual,       // On time
    Late,           // Frequently tardy
}

impl TraitIndicator {
    pub fn from_action(action: ActionId) -> Option<Self> {
        match action {
            ActionId::Help => Some(TraitIndicator::Generous),
            ActionId::Flee => Some(TraitIndicator::Peaceful),
            ActionId::Attack | ActionId::Charge => Some(TraitIndicator::Aggressive),
            ActionId::Defend | ActionId::HoldPosition => Some(TraitIndicator::Reliable),
            _ => None,
        }
    }

    /// Returns the opposite trait (for violation detection)
    pub fn opposite(&self) -> Self {
        match self {
            TraitIndicator::Reliable => TraitIndicator::Unreliable,
            TraitIndicator::Unreliable => TraitIndicator::Reliable,
            TraitIndicator::Generous => TraitIndicator::Stingy,
            TraitIndicator::Stingy => TraitIndicator::Generous,
            TraitIndicator::Aggressive => TraitIndicator::Peaceful,
            TraitIndicator::Peaceful => TraitIndicator::Aggressive,
            TraitIndicator::Punctual => TraitIndicator::Late,
            TraitIndicator::Late => TraitIndicator::Punctual,
        }
    }
}
```

**Step 4: Add to mod.rs**

```rust
// In src/entity/social/mod.rs, add:
pub mod service_types;
pub use service_types::{ServiceType, TraitIndicator};
```

**Step 5: Run tests**

Run: `cargo test --lib entity::social::service_types`
Expected: PASS

**Step 6: Commit**

```bash
git add src/entity/social/service_types.rs src/entity/social/mod.rs
git commit -m "feat: add ServiceType and TraitIndicator enums"
```

---

## Task 3: Add BehaviorPattern and PatternType

Create core expectation data structures.

**Files:**
- Create: `src/entity/social/expectations.rs`
- Modify: `src/entity/social/mod.rs`

**Step 1: Write the failing test**

```rust
// In src/entity/social/expectations.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::EntityId;

    #[test]
    fn test_pattern_creation() {
        let pattern = BehaviorPattern::new(
            PatternType::ProvidesWhenAsked { service_type: ServiceType::Crafting },
            100,
        );
        assert_eq!(pattern.observation_count, 1);
        assert_eq!(pattern.violation_count, 0);
        assert!(pattern.confidence > 0.0);
        assert!((pattern.salience - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_confidence_calculation() {
        let mut pattern = BehaviorPattern::new(
            PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Reliable },
            0,
        );

        // Initial confidence with 1 observation
        let initial = pattern.confidence;

        // Add more observations
        pattern.record_observation(100);
        pattern.record_observation(200);

        // Confidence should increase
        assert!(pattern.confidence > initial);
        assert_eq!(pattern.observation_count, 3);
    }

    #[test]
    fn test_violation_reduces_confidence() {
        let mut pattern = BehaviorPattern::new(
            PatternType::ProvidesWhenAsked { service_type: ServiceType::Trading },
            0,
        );

        // Build up confidence
        for i in 1..=5 {
            pattern.record_observation(i * 100);
        }
        let high_confidence = pattern.confidence;

        // Record violation
        pattern.record_violation(600);

        // Confidence should decrease
        assert!(pattern.confidence < high_confidence);
        assert_eq!(pattern.violation_count, 1);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib entity::social::expectations`
Expected: FAIL

**Step 3: Write implementation**

```rust
// src/entity/social/expectations.rs
use serde::{Deserialize, Serialize};
use crate::core::types::EntityId;
use crate::core::calendar::TimePeriod;
use crate::actions::catalog::ActionCategory;
use super::event_types::EventType;
use super::service_types::{ServiceType, TraitIndicator};

/// What kind of behavior we expect
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternType {
    /// "They provide this service when asked"
    ProvidesWhenAsked { service_type: ServiceType },

    /// "They behave with this trait"
    BehavesWithTrait { trait_indicator: TraitIndicator },

    /// "They're at this location during this time"
    LocationDuring { location_id: EntityId, time_period: TimePeriod },

    /// "They respond to this event with this action type"
    RespondsToEvent { event_type: EventType, typical_response: ActionCategory },
}

/// A behavioral expectation about an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorPattern {
    pub pattern_type: PatternType,

    // Confidence tracking
    pub observation_count: u32,
    pub violation_count: u32,
    pub confidence: f32,

    // Recency
    pub last_confirmed: u64,
    pub last_violated: u64,

    // Decay (like memories)
    pub salience: f32,
}

// Constants
const PRIOR_WEIGHT: f32 = 2.0;
const INITIAL_SALIENCE: f32 = 0.5;
const OBSERVATION_BOOST: f32 = 0.15;
const SALIENCE_BOOST: f32 = 0.1;

impl BehaviorPattern {
    pub fn new(pattern_type: PatternType, tick: u64) -> Self {
        Self {
            pattern_type,
            observation_count: 1,
            violation_count: 0,
            confidence: Self::calculate_confidence(1, 0),
            last_confirmed: tick,
            last_violated: 0,
            salience: INITIAL_SALIENCE,
        }
    }

    fn calculate_confidence(observations: u32, violations: u32) -> f32 {
        let total = observations as f32 + violations as f32 + PRIOR_WEIGHT;
        observations as f32 / total
    }

    pub fn record_observation(&mut self, tick: u64) {
        self.observation_count += 1;
        self.last_confirmed = tick;
        self.salience = (self.salience + SALIENCE_BOOST).min(1.0);
        self.confidence = Self::calculate_confidence(self.observation_count, self.violation_count);
    }

    pub fn record_violation(&mut self, tick: u64) {
        self.violation_count += 1;
        self.last_violated = tick;
        self.confidence = Self::calculate_confidence(self.observation_count, self.violation_count);
    }

    pub fn apply_decay(&mut self, decay_rate: f32) {
        self.salience *= 1.0 - decay_rate;
    }

    pub fn is_stale(&self, salience_floor: f32) -> bool {
        self.salience < salience_floor
    }
}

/// Maximum patterns per relationship slot
pub const MAX_PATTERNS_PER_SLOT: usize = 8;

/// Threshold for pattern salience to be checked
pub const SALIENCE_THRESHOLD: f32 = 0.1;

/// Floor below which patterns are removed
pub const SALIENCE_FLOOR: f32 = 0.05;
```

**Step 4: Add to mod.rs**

```rust
// In src/entity/social/mod.rs, add:
pub mod expectations;
pub use expectations::{BehaviorPattern, PatternType, MAX_PATTERNS_PER_SLOT, SALIENCE_THRESHOLD, SALIENCE_FLOOR};
```

**Step 5: Run tests**

Run: `cargo test --lib entity::social::expectations`
Expected: PASS

**Step 6: Commit**

```bash
git add src/entity/social/expectations.rs src/entity/social/mod.rs
git commit -m "feat: add BehaviorPattern and PatternType for expectations"
```

---

## Task 4: Add EventBuffer for Recent Events

Per-entity ring buffer for RespondsToEvent pattern detection.

**Files:**
- Create: `src/entity/social/event_buffer.rs`
- Modify: `src/entity/social/mod.rs`
- Modify: `src/entity/species/human.rs`

**Step 1: Write the failing test**

```rust
// In src/entity/social/event_buffer.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_buffer_ring() {
        let mut buffer = EventBuffer::new(3); // Small for testing

        buffer.push(RecentEvent { event_type: EventType::Observation, actor: EntityId::new(), tick: 1 });
        buffer.push(RecentEvent { event_type: EventType::Transaction, actor: EntityId::new(), tick: 2 });
        buffer.push(RecentEvent { event_type: EventType::AidReceived, actor: EntityId::new(), tick: 3 });

        assert_eq!(buffer.len(), 3);

        // Push fourth - oldest should be evicted
        buffer.push(RecentEvent { event_type: EventType::Betrayal, actor: EntityId::new(), tick: 4 });

        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.events[0].tick, 2); // tick 1 evicted
    }

    #[test]
    fn test_find_recent_event() {
        let mut buffer = EventBuffer::new(10);
        let actor = EntityId::new();

        buffer.push(RecentEvent { event_type: EventType::AidReceived, actor, tick: 100 });

        let found = buffer.find_recent(actor, EventType::AidReceived, 50);
        assert!(found.is_some());

        let not_found = buffer.find_recent(actor, EventType::Betrayal, 50);
        assert!(not_found.is_none());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib entity::social::event_buffer`
Expected: FAIL

**Step 3: Write implementation**

```rust
// src/entity/social/event_buffer.rs
use serde::{Deserialize, Serialize};
use crate::core::types::EntityId;
use super::event_types::EventType;

/// A recent event witnessed or performed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentEvent {
    pub event_type: EventType,
    pub actor: EntityId,
    pub tick: u64,
}

/// Ring buffer of recent events for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBuffer {
    pub events: Vec<RecentEvent>,
    capacity: usize,
}

impl EventBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, event: RecentEvent) {
        if self.events.len() >= self.capacity {
            self.events.remove(0); // Remove oldest
        }
        self.events.push(event);
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Find a recent event by actor and type within last N ticks
    pub fn find_recent(&self, actor: EntityId, event_type: EventType, within_ticks: u64) -> Option<&RecentEvent> {
        let min_tick = self.events.last().map(|e| e.tick.saturating_sub(within_ticks)).unwrap_or(0);

        self.events.iter().rev().find(|e| {
            e.actor == actor && e.event_type == event_type && e.tick >= min_tick
        })
    }

    /// Get events involving a specific actor
    pub fn events_by_actor(&self, actor: EntityId) -> impl Iterator<Item = &RecentEvent> {
        self.events.iter().filter(move |e| e.actor == actor)
    }

    /// Clear old events beyond a tick threshold
    pub fn clear_before(&mut self, tick: u64) {
        self.events.retain(|e| e.tick >= tick);
    }
}

impl Default for EventBuffer {
    fn default() -> Self {
        Self::new(10) // Default capacity
    }
}
```

**Step 4: Add to mod.rs and HumanArchetype**

```rust
// In src/entity/social/mod.rs, add:
pub mod event_buffer;
pub use event_buffer::{EventBuffer, RecentEvent};

// In src/entity/species/human.rs, add to HumanArchetype:
pub event_buffers: Vec<EventBuffer>,

// In HumanArchetype::new(), add:
event_buffers: Vec::new(),

// In spawn() method, add:
self.event_buffers.push(EventBuffer::default());
```

**Step 5: Run tests**

Run: `cargo test --lib entity::social::event_buffer`
Expected: PASS

**Step 6: Commit**

```bash
git add src/entity/social/event_buffer.rs src/entity/social/mod.rs src/entity/species/human.rs
git commit -m "feat: add EventBuffer for tracking recent events"
```

---

## Task 5: Extend RelationshipSlot with Expectations

Add expectations field to existing RelationshipSlot.

**Files:**
- Modify: `src/entity/social/social_memory.rs`

**Step 1: Write the failing test**

```rust
// Add to existing tests in social_memory.rs
#[test]
fn test_relationship_slot_expectations() {
    let target = EntityId::new();
    let mut slot = RelationshipSlot::new(target, 0);

    assert!(slot.expectations.is_empty());

    // Add expectation
    let pattern = BehaviorPattern::new(
        PatternType::ProvidesWhenAsked { service_type: ServiceType::Crafting },
        100,
    );
    slot.add_expectation(pattern);

    assert_eq!(slot.expectations.len(), 1);

    // Find expectation
    let found = slot.find_expectation(&PatternType::ProvidesWhenAsked {
        service_type: ServiceType::Crafting
    });
    assert!(found.is_some());
}

#[test]
fn test_expectations_bounded() {
    let target = EntityId::new();
    let mut slot = RelationshipSlot::new(target, 0);

    // Add more than MAX_PATTERNS_PER_SLOT
    for i in 0..12 {
        let pattern = BehaviorPattern::new(
            PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Reliable },
            i as u64 * 100,
        );
        slot.add_expectation(pattern);
    }

    // Should be bounded
    assert!(slot.expectations.len() <= MAX_PATTERNS_PER_SLOT);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib entity::social::social_memory::tests::test_relationship_slot_expectations`
Expected: FAIL

**Step 3: Modify RelationshipSlot**

```rust
// In src/entity/social/social_memory.rs

// Add imports at top:
use super::expectations::{BehaviorPattern, PatternType, MAX_PATTERNS_PER_SLOT, SALIENCE_FLOOR};
use super::service_types::{ServiceType, TraitIndicator};

// Modify RelationshipSlot struct:
pub struct RelationshipSlot {
    pub target_id: EntityId,
    pub memories: Vec<RelationshipMemory>,
    pub first_contact: u64,
    pub last_contact: u64,
    pub interaction_count: u32,
    // NEW: Behavioral expectations about this entity
    pub expectations: Vec<BehaviorPattern>,
}

// Update RelationshipSlot::new():
impl RelationshipSlot {
    pub fn new(target_id: EntityId, tick: u64) -> Self {
        Self {
            target_id,
            memories: Vec::new(),
            first_contact: tick,
            last_contact: tick,
            interaction_count: 0,
            expectations: Vec::new(), // NEW
        }
    }

    // NEW: Add expectation methods
    pub fn add_expectation(&mut self, pattern: BehaviorPattern) {
        // Check if similar pattern exists
        if let Some(existing) = self.find_expectation_mut(&pattern.pattern_type) {
            // Strengthen existing
            existing.record_observation(pattern.last_confirmed);
            return;
        }

        // Evict lowest salience if at capacity
        if self.expectations.len() >= MAX_PATTERNS_PER_SLOT {
            if let Some(min_idx) = self.expectations
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.salience.partial_cmp(&b.salience).unwrap())
                .map(|(i, _)| i)
            {
                self.expectations.remove(min_idx);
            }
        }

        self.expectations.push(pattern);
    }

    pub fn find_expectation(&self, pattern_type: &PatternType) -> Option<&BehaviorPattern> {
        self.expectations.iter().find(|p| Self::pattern_matches(&p.pattern_type, pattern_type))
    }

    pub fn find_expectation_mut(&mut self, pattern_type: &PatternType) -> Option<&mut BehaviorPattern> {
        self.expectations.iter_mut().find(|p| Self::pattern_matches(&p.pattern_type, pattern_type))
    }

    fn pattern_matches(a: &PatternType, b: &PatternType) -> bool {
        match (a, b) {
            (PatternType::ProvidesWhenAsked { service_type: s1 },
             PatternType::ProvidesWhenAsked { service_type: s2 }) => s1 == s2,
            (PatternType::BehavesWithTrait { trait_indicator: t1 },
             PatternType::BehavesWithTrait { trait_indicator: t2 }) => t1 == t2,
            (PatternType::LocationDuring { location_id: l1, time_period: t1 },
             PatternType::LocationDuring { location_id: l2, time_period: t2 }) => l1 == l2 && t1 == t2,
            (PatternType::RespondsToEvent { event_type: e1, .. },
             PatternType::RespondsToEvent { event_type: e2, .. }) => e1 == e2,
            _ => false,
        }
    }

    pub fn decay_expectations(&mut self, decay_rate: f32) {
        for pattern in &mut self.expectations {
            pattern.apply_decay(decay_rate);
        }
        self.expectations.retain(|p| !p.is_stale(SALIENCE_FLOOR));
    }
}
```

**Step 4: Run tests**

Run: `cargo test --lib entity::social::social_memory`
Expected: PASS

**Step 5: Commit**

```bash
git add src/entity/social/social_memory.rs
git commit -m "feat: extend RelationshipSlot with expectations field"
```

---

## Task 6: Implement Expectation Formation

Form expectations when observing actions during perception.

**Files:**
- Create: `src/simulation/expectation_formation.rs`
- Modify: `src/simulation/mod.rs`
- Modify: `src/simulation/tick.rs`

**Step 1: Write the failing test**

```rust
// In src/simulation/expectation_formation.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::world::World;

    #[test]
    fn test_observation_forms_expectation() {
        let mut world = World::new();

        // Spawn observer and actor
        let observer_id = world.spawn_human("Observer", Vec2::new(0.0, 0.0));
        let actor_id = world.spawn_human("Actor", Vec2::new(5.0, 0.0));

        let observer_idx = world.humans.index_of(observer_id).unwrap();
        let actor_idx = world.humans.index_of(actor_id).unwrap();

        // Actor is doing Help action
        let task = Task::new(ActionId::Help, TaskPriority::Normal, 0).with_entity(observer_id);
        world.humans.task_queues[actor_idx].push(task);

        // Observer sees actor
        record_observation(&mut world, observer_idx, actor_idx, ActionId::Help, 100);

        // Observer should form expectation about actor
        let slot = world.humans.social_memories[observer_idx].find_slot(actor_id);
        assert!(slot.is_some());

        let slot = slot.unwrap();
        let expectation = slot.find_expectation(&PatternType::BehavesWithTrait {
            trait_indicator: TraitIndicator::Generous
        });
        assert!(expectation.is_some());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib simulation::expectation_formation`
Expected: FAIL

**Step 3: Write implementation**

```rust
// src/simulation/expectation_formation.rs
use crate::ecs::world::World;
use crate::core::types::EntityId;
use crate::actions::catalog::{ActionId, ActionCategory};
use crate::entity::social::{
    BehaviorPattern, PatternType, ServiceType, TraitIndicator,
    EventType, RecentEvent,
};

/// Record an observation and form expectations
pub fn record_observation(
    world: &mut World,
    observer_idx: usize,
    observed_idx: usize,
    action: ActionId,
    current_tick: u64,
) {
    let observed_id = world.humans.ids[observed_idx];

    // Infer patterns from action
    let patterns = infer_patterns_from_action(action, current_tick);

    if patterns.is_empty() {
        return;
    }

    // Get or create relationship slot
    world.humans.social_memories[observer_idx].record_encounter(
        observed_id,
        EventType::Observation,
        0.3, // Low intensity for just observing
        current_tick,
    );

    // Add expectations to the slot
    if let Some(slot) = world.humans.social_memories[observer_idx].find_slot_mut(observed_id) {
        for pattern in patterns {
            slot.add_expectation(pattern);
        }
    }

    // Also record in observer's event buffer
    world.humans.event_buffers[observer_idx].push(RecentEvent {
        event_type: EventType::Observation,
        actor: observed_id,
        tick: current_tick,
    });
}

/// Infer behavioral patterns from an observed action
fn infer_patterns_from_action(action: ActionId, tick: u64) -> Vec<BehaviorPattern> {
    let mut patterns = Vec::new();

    // Service patterns
    if let Some(service) = ServiceType::from_action(action) {
        patterns.push(BehaviorPattern::new(
            PatternType::ProvidesWhenAsked { service_type: service },
            tick,
        ));
    }

    // Trait patterns
    if let Some(trait_ind) = TraitIndicator::from_action(action) {
        patterns.push(BehaviorPattern::new(
            PatternType::BehavesWithTrait { trait_indicator: trait_ind },
            tick,
        ));
    }

    // RespondsToEvent patterns based on action category
    match action.category() {
        ActionCategory::Combat => {
            patterns.push(BehaviorPattern::new(
                PatternType::RespondsToEvent {
                    event_type: EventType::HarmReceived,
                    typical_response: ActionCategory::Combat,
                },
                tick,
            ));
        }
        ActionCategory::Movement if action == ActionId::Flee => {
            patterns.push(BehaviorPattern::new(
                PatternType::RespondsToEvent {
                    event_type: EventType::HarmReceived,
                    typical_response: ActionCategory::Movement,
                },
                tick,
            ));
        }
        _ => {}
    }

    patterns
}

/// Process all observations from perception data
pub fn process_observations(world: &mut World, perceptions: &[crate::simulation::perception::Perception]) {
    let current_tick = world.current_tick;

    for (observer_idx, perception) in perceptions.iter().enumerate() {
        for perceived in &perception.entities {
            // Get observed entity's current action
            if let Some(observed_idx) = world.humans.index_of(perceived.entity_id) {
                if let Some(task) = world.humans.task_queues[observed_idx].current() {
                    record_observation(world, observer_idx, observed_idx, task.action, current_tick);
                }
            }
        }
    }
}
```

**Step 4: Add to mod.rs and tick.rs**

```rust
// In src/simulation/mod.rs, add:
pub mod expectation_formation;
pub use expectation_formation::{record_observation, process_observations};

// In src/simulation/tick.rs run_simulation_tick(), after generate_thoughts():
process_observations(world, &perceptions);
```

**Step 5: Run tests**

Run: `cargo test --lib simulation::expectation_formation`
Expected: PASS

**Step 6: Commit**

```bash
git add src/simulation/expectation_formation.rs src/simulation/mod.rs src/simulation/tick.rs
git commit -m "feat: implement expectation formation from observations"
```

---

## Task 7: Implement Violation Detection

Detect when observed behavior violates expectations, generate thoughts.

**Files:**
- Create: `src/simulation/violation_detection.rs`
- Modify: `src/simulation/mod.rs`
- Modify: `src/simulation/tick.rs`

**Step 1: Write the failing test**

```rust
// In src/simulation/violation_detection.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::world::World;

    #[test]
    fn test_violation_generates_thought() {
        let mut world = World::new();

        // Setup: Observer expects Actor to be Generous
        let observer_id = world.spawn_human("Observer", Vec2::new(0.0, 0.0));
        let actor_id = world.spawn_human("Actor", Vec2::new(5.0, 0.0));

        let observer_idx = world.humans.index_of(observer_id).unwrap();
        let actor_idx = world.humans.index_of(actor_id).unwrap();

        // Build expectation (observe Help action 5 times)
        for i in 0..5 {
            record_observation(&mut world, observer_idx, actor_idx, ActionId::Help, i * 100);
        }

        // Verify expectation exists with decent confidence
        let slot = world.humans.social_memories[observer_idx].find_slot(actor_id).unwrap();
        let exp = slot.find_expectation(&PatternType::BehavesWithTrait {
            trait_indicator: TraitIndicator::Generous
        }).unwrap();
        assert!(exp.confidence > 0.5);

        // Now actor does something Stingy (Attack instead of Help)
        let task = Task::new(ActionId::Attack, TaskPriority::Normal, 500);
        world.humans.task_queues[actor_idx].push(task);

        let initial_thoughts = world.humans.thoughts[observer_idx].len();

        // Check for violations
        check_violations(&mut world, observer_idx, actor_idx, ActionId::Attack, 500);

        // Should have generated a negative thought
        assert!(world.humans.thoughts[observer_idx].len() > initial_thoughts);

        let thought = world.humans.thoughts[observer_idx].strongest();
        assert!(thought.is_some());
        assert_eq!(thought.unwrap().valence, Valence::Negative);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib simulation::violation_detection`
Expected: FAIL

**Step 3: Write implementation**

```rust
// src/simulation/violation_detection.rs
use crate::ecs::world::World;
use crate::core::types::EntityId;
use crate::actions::catalog::ActionId;
use crate::entity::social::{PatternType, TraitIndicator, SALIENCE_THRESHOLD};
use crate::entity::thoughts::{Thought, Valence, CauseType};

/// Check if an observed action violates any expectations
pub fn check_violations(
    world: &mut World,
    observer_idx: usize,
    observed_idx: usize,
    observed_action: ActionId,
    current_tick: u64,
) {
    let observed_id = world.humans.ids[observed_idx];

    // Get expectations about this entity
    let violations = {
        let slot = match world.humans.social_memories[observer_idx].find_slot(observed_id) {
            Some(s) => s,
            None => return,
        };

        let mut violations = Vec::new();

        for pattern in &slot.expectations {
            if pattern.salience < SALIENCE_THRESHOLD {
                continue;
            }

            if let Some(violation) = check_pattern_violation(pattern, observed_action) {
                violations.push((pattern.pattern_type.clone(), pattern.confidence, violation));
            }
        }

        violations
    };

    // Process violations
    for (pattern_type, confidence, violation_type) in violations {
        // Update pattern stats
        if let Some(slot) = world.humans.social_memories[observer_idx].find_slot_mut(observed_id) {
            if let Some(pattern) = slot.find_expectation_mut(&pattern_type) {
                pattern.record_violation(current_tick);
            }
        }

        // Generate violation thought
        let intensity = confidence * 0.8; // High confidence = high disappointment
        let concept = match violation_type {
            ViolationType::TraitContradiction => "BETRAYAL",
            ViolationType::ServiceRefused => "DISAPPOINTMENT",
            ViolationType::UnexpectedResponse => "SURPRISE",
        };

        let thought = Thought::new(
            Valence::Negative,
            intensity.min(1.0),
            concept.to_string(),
            CauseType::Entity,
            Some(observed_id),
        );

        world.humans.thoughts[observer_idx].add(thought);
    }
}

#[derive(Debug, Clone)]
enum ViolationType {
    TraitContradiction,
    ServiceRefused,
    UnexpectedResponse,
}

fn check_pattern_violation(pattern: &crate::entity::social::BehaviorPattern, action: ActionId) -> Option<ViolationType> {
    match &pattern.pattern_type {
        PatternType::BehavesWithTrait { trait_indicator } => {
            // Check if action contradicts expected trait
            if let Some(action_trait) = TraitIndicator::from_action(action) {
                if action_trait == trait_indicator.opposite() {
                    return Some(ViolationType::TraitContradiction);
                }
            }
            None
        }

        PatternType::ProvidesWhenAsked { service_type } => {
            // This would be checked when a service request is refused
            // For now, just return None
            None
        }

        PatternType::RespondsToEvent { typical_response, .. } => {
            // Check if action category matches expected response
            if action.category() != *typical_response {
                return Some(ViolationType::UnexpectedResponse);
            }
            None
        }

        PatternType::LocationDuring { .. } => {
            // Location checking requires more context
            None
        }
    }
}

/// Process violation checks for all perceptions
pub fn process_violations(world: &mut World, perceptions: &[crate::simulation::perception::Perception]) {
    let current_tick = world.current_tick;

    for (observer_idx, perception) in perceptions.iter().enumerate() {
        for perceived in &perception.entities {
            if let Some(observed_idx) = world.humans.index_of(perceived.entity_id) {
                if let Some(task) = world.humans.task_queues[observed_idx].current() {
                    check_violations(world, observer_idx, observed_idx, task.action, current_tick);
                }
            }
        }
    }
}
```

**Step 4: Add to mod.rs and tick.rs**

```rust
// In src/simulation/mod.rs, add:
pub mod violation_detection;
pub use violation_detection::{check_violations, process_violations};

// In src/simulation/tick.rs run_simulation_tick(), after process_observations():
process_violations(world, &perceptions);
```

**Step 5: Run tests**

Run: `cargo test --lib simulation::violation_detection`
Expected: PASS

**Step 6: Commit**

```bash
git add src/simulation/violation_detection.rs src/simulation/mod.rs src/simulation/tick.rs
git commit -m "feat: implement violation detection with thought generation"
```

---

## Task 8: Implement Expectation Decay

Decay expectations daily like memories.

**Files:**
- Modify: `src/simulation/tick.rs`

**Step 1: Write the failing test**

```rust
// Add to tick.rs tests
#[test]
fn test_expectation_decay() {
    let mut world = World::new();

    let observer_id = world.spawn_human("Observer", Vec2::new(0.0, 0.0));
    let actor_id = world.spawn_human("Actor", Vec2::new(5.0, 0.0));

    let observer_idx = world.humans.index_of(observer_id).unwrap();
    let actor_idx = world.humans.index_of(actor_id).unwrap();

    // Build expectation
    record_observation(&mut world, observer_idx, actor_idx, ActionId::Help, 0);

    let initial_salience = {
        let slot = world.humans.social_memories[observer_idx].find_slot(actor_id).unwrap();
        slot.expectations[0].salience
    };

    // Advance 1000 ticks (one day)
    for _ in 0..1000 {
        world.tick();
    }

    // Decay expectations
    decay_expectations(world);

    let final_salience = {
        let slot = world.humans.social_memories[observer_idx].find_slot(actor_id).unwrap();
        slot.expectations[0].salience
    };

    assert!(final_salience < initial_salience, "Salience should decay");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib simulation::tick::tests::test_expectation_decay`
Expected: FAIL

**Step 3: Implement decay**

```rust
// In src/simulation/tick.rs, add function:

/// Decay expectation salience once per simulation day
fn decay_expectations(world: &mut World) {
    // Only decay once per day
    if world.current_tick % TICKS_PER_DAY != 0 {
        return;
    }

    const EXPECTATION_DECAY_RATE: f32 = 0.05;

    let living_indices: Vec<usize> = world.humans.iter_living().collect();
    for idx in living_indices {
        for slot in &mut world.humans.social_memories[idx].slots {
            slot.decay_expectations(EXPECTATION_DECAY_RATE);
        }
    }
}

// In run_simulation_tick(), after decay_social_memories():
decay_expectations(world);
```

**Step 4: Run tests**

Run: `cargo test --lib simulation::tick::tests::test_expectation_decay`
Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat: implement daily expectation decay"
```

---

## Task 9: Implement Role Derivation

Query-time function to derive emergent roles from expectations.

**Files:**
- Create: `src/entity/social/role_derivation.rs`
- Modify: `src/entity/social/mod.rs`

**Step 1: Write the failing test**

```rust
// In src/entity/social/role_derivation.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::world::World;

    #[test]
    fn test_derive_roles_from_expectations() {
        let mut world = World::new();

        // Create "smith" - entity that multiple others expect to craft
        let smith_id = world.spawn_human("Smith", Vec2::new(0.0, 0.0));
        let smith_idx = world.humans.index_of(smith_id).unwrap();

        // Create 5 observers who expect crafting
        for i in 0..5 {
            let observer_id = world.spawn_human(&format!("Observer{}", i), Vec2::new((i+1) as f32 * 10.0, 0.0));
            let observer_idx = world.humans.index_of(observer_id).unwrap();

            // Each observer has high-confidence expectation
            let pattern = BehaviorPattern::new(
                PatternType::ProvidesWhenAsked { service_type: ServiceType::Crafting },
                0,
            );
            // Boost confidence
            let mut pattern = pattern;
            for _ in 0..10 {
                pattern.record_observation(100);
            }

            world.humans.social_memories[observer_idx].record_encounter(
                smith_id, EventType::Observation, 0.5, 0
            );
            if let Some(slot) = world.humans.social_memories[observer_idx].find_slot_mut(smith_id) {
                slot.add_expectation(pattern);
            }
        }

        // Derive roles
        let roles = derive_roles(smith_id, &world);

        assert!(!roles.is_empty());
        assert!(roles.iter().any(|r| r.role_type == ServiceType::Crafting));
        assert!(roles[0].holder_count >= 3); // At least 3 expect this
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib entity::social::role_derivation`
Expected: FAIL

**Step 3: Write implementation**

```rust
// src/entity/social/role_derivation.rs
use crate::ecs::world::World;
use crate::core::types::EntityId;
use super::expectations::PatternType;
use super::service_types::ServiceType;
use std::collections::HashMap;

/// An emergent role derived from what others expect
#[derive(Debug, Clone)]
pub struct EmergentRole {
    pub role_type: ServiceType,
    pub holder_count: u32,
    pub average_confidence: f32,
}

/// Minimum expectation holders to derive a role
const ROLE_THRESHOLD: u32 = 3;

/// Derive emergent roles for an entity based on others' expectations
pub fn derive_roles(entity_id: EntityId, world: &World) -> Vec<EmergentRole> {
    let mut service_expectations: HashMap<ServiceType, Vec<f32>> = HashMap::new();

    // Scan all entities for expectations about this entity
    for observer_idx in world.humans.iter_living() {
        if let Some(slot) = world.humans.social_memories[observer_idx].find_slot(entity_id) {
            for pattern in &slot.expectations {
                if let PatternType::ProvidesWhenAsked { service_type } = &pattern.pattern_type {
                    if pattern.confidence > 0.3 {
                        service_expectations
                            .entry(*service_type)
                            .or_default()
                            .push(pattern.confidence);
                    }
                }
            }
        }
    }

    // Build roles from clustered expectations
    let mut roles = Vec::new();

    for (service_type, confidences) in service_expectations {
        let holder_count = confidences.len() as u32;

        if holder_count >= ROLE_THRESHOLD {
            let average_confidence = confidences.iter().sum::<f32>() / confidences.len() as f32;

            roles.push(EmergentRole {
                role_type: service_type,
                holder_count,
                average_confidence,
            });
        }
    }

    // Sort by holder count (most established first)
    roles.sort_by(|a, b| b.holder_count.cmp(&a.holder_count));

    roles
}

/// Find entities who function in a specific role
pub fn query_entities_by_role(world: &World, role_type: ServiceType) -> Vec<(EntityId, f32)> {
    let mut results = Vec::new();

    for entity_idx in world.humans.iter_living() {
        let entity_id = world.humans.ids[entity_idx];
        let roles = derive_roles(entity_id, world);

        if let Some(role) = roles.iter().find(|r| r.role_type == role_type) {
            results.push((entity_id, role.average_confidence));
        }
    }

    // Sort by confidence
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    results
}
```

**Step 4: Add to mod.rs**

```rust
// In src/entity/social/mod.rs, add:
pub mod role_derivation;
pub use role_derivation::{EmergentRole, derive_roles, query_entities_by_role};
```

**Step 5: Run tests**

Run: `cargo test --lib entity::social::role_derivation`
Expected: PASS

**Step 6: Commit**

```bash
git add src/entity/social/role_derivation.rs src/entity/social/mod.rs
git commit -m "feat: implement query-time role derivation from expectations"
```

---

## Task 10: Integration Tests

Verify the full expectation → violation → thought → relationship cycle.

**Files:**
- Modify: `tests/emergence_tests.rs`

**Step 1: Write integration tests**

```rust
// Add to tests/emergence_tests.rs

#[test]
fn test_expectation_formation_from_observation() {
    let mut world = World::new();

    // Place entities near each other
    let observer_id = world.spawn_human("Observer", Vec2::new(0.0, 0.0));
    let crafter_id = world.spawn_human("Crafter", Vec2::new(5.0, 0.0));

    let observer_idx = world.humans.index_of(observer_id).unwrap();
    let crafter_idx = world.humans.index_of(crafter_id).unwrap();

    // Crafter does Craft action repeatedly
    for _ in 0..5 {
        let task = Task::new(ActionId::Craft, TaskPriority::Normal, world.current_tick);
        world.humans.task_queues[crafter_idx].push(task);

        // Run simulation tick (includes perception and expectation formation)
        run_simulation_tick(&mut world);
    }

    // Observer should have formed expectation about Crafter
    let slot = world.humans.social_memories[observer_idx].find_slot(crafter_id);
    assert!(slot.is_some(), "Observer should have relationship slot for Crafter");

    let has_craft_expectation = slot.unwrap().expectations.iter().any(|e| {
        matches!(e.pattern_type, PatternType::ProvidesWhenAsked { service_type: ServiceType::Crafting })
    });
    assert!(has_craft_expectation, "Observer should expect Crafter to craft");
}

#[test]
fn test_violation_creates_negative_thought() {
    let mut world = World::new();

    let observer_id = world.spawn_human("Observer", Vec2::new(0.0, 0.0));
    let helper_id = world.spawn_human("Helper", Vec2::new(5.0, 0.0));

    let observer_idx = world.humans.index_of(observer_id).unwrap();
    let helper_idx = world.humans.index_of(helper_id).unwrap();

    // Build expectation: Helper is Generous (observed helping 10 times)
    for _ in 0..10 {
        let task = Task::new(ActionId::Help, TaskPriority::Normal, world.current_tick)
            .with_entity(observer_id);
        world.humans.task_queues[helper_idx].push(task);
        run_simulation_tick(&mut world);
    }

    // Verify high-confidence expectation
    let slot = world.humans.social_memories[observer_idx].find_slot(helper_id).unwrap();
    let exp = slot.find_expectation(&PatternType::BehavesWithTrait {
        trait_indicator: TraitIndicator::Generous
    });
    assert!(exp.is_some() && exp.unwrap().confidence > 0.6);

    // Now Helper attacks (contradicts Generous trait)
    world.humans.thoughts[observer_idx].clear();
    let task = Task::new(ActionId::Attack, TaskPriority::Normal, world.current_tick);
    world.humans.task_queues[helper_idx].push(task);
    run_simulation_tick(&mut world);

    // Observer should have negative thought about Helper
    let thoughts: Vec<_> = world.humans.thoughts[observer_idx].iter().collect();
    let violation_thought = thoughts.iter().find(|t| {
        t.valence == Valence::Negative && t.cause_reference == Some(helper_id)
    });
    assert!(violation_thought.is_some(), "Should have negative thought about violator");
}

#[test]
fn test_role_emerges_from_consistent_behavior() {
    let mut world = World::new();

    // Create potential "healer"
    let healer_id = world.spawn_human("Healer", Vec2::new(50.0, 50.0));
    let healer_idx = world.humans.index_of(healer_id).unwrap();

    // Create 5 observers who watch healer
    let mut observers = Vec::new();
    for i in 0..5 {
        let obs_id = world.spawn_human(
            &format!("Patient{}", i),
            Vec2::new(50.0 + (i as f32 * 2.0), 50.0)
        );
        observers.push(obs_id);
    }

    // Healer does Help action many times (simulating healing)
    for _ in 0..20 {
        let patient = observers[world.current_tick as usize % observers.len()];
        let task = Task::new(ActionId::Help, TaskPriority::Normal, world.current_tick)
            .with_entity(patient);
        world.humans.task_queues[healer_idx].push(task);
        run_simulation_tick(&mut world);
    }

    // Query roles for healer
    let roles = derive_roles(healer_id, &world);

    // Should have emergent role based on helping behavior
    assert!(!roles.is_empty(), "Healer should have emergent roles");

    // Query entities by role
    let helpers = query_entities_by_role(&world, ServiceType::Helping);
    assert!(helpers.iter().any(|(id, _)| *id == healer_id), "Healer should be found by role query");
}
```

**Step 2: Run integration tests**

Run: `cargo test --test emergence_tests`
Expected: PASS

**Step 3: Commit**

```bash
git add tests/emergence_tests.rs
git commit -m "test: integration tests for expectation-based social dynamics"
```

---

## Summary

| Task | Component | Outcome |
|------|-----------|---------|
| 1 | Calendar | Time-of-day periods for LocationDuring |
| 2 | ServiceType/TraitIndicator | Enums for pattern classification |
| 3 | BehaviorPattern | Core expectation data structure |
| 4 | EventBuffer | Per-entity ring buffer for events |
| 5 | RelationshipSlot | Extended with expectations field |
| 6 | Formation | Expectations form from observations |
| 7 | Violations | Detection generates negative thoughts |
| 8 | Decay | Daily salience decay like memories |
| 9 | Role Derivation | Query-time role emergence |
| 10 | Integration | Full cycle verification |

**Total: 10 tasks**

**Phase 2 (future):** Proactive social cost in action selection
