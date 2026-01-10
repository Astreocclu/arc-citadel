# Action Execution Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement real mechanics for 14 non-combat actions (Movement, Survival, Social, Work, Idle).

**Architecture:** Extend `execute_tasks()` in tick.rs with action-specific logic. Add ResourceZone for Gather. Modify perception system for IdleObserve boost. Social actions use proximity + mutual engagement.

**Tech Stack:** Rust, existing ECS with SoA layout, rand crate for wandering.

---

## Task 1: Add ResourceZone to World

Add a new zone type for gatherable resources (wood, stone, etc.) following the FoodZone pattern.

**Files:**
- Create: `src/simulation/resource_zone.rs`
- Modify: `src/simulation/mod.rs`
- Modify: `src/ecs/world.rs`

**Step 1: Write the failing test**

```rust
// In src/simulation/resource_zone.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Vec2;

    #[test]
    fn test_resource_zone_creation() {
        let zone = ResourceZone::new(
            Vec2::new(10.0, 20.0),
            ResourceType::Wood,
            5.0, // radius
        );
        assert_eq!(zone.resource_type, ResourceType::Wood);
        assert!((zone.current - 1.0).abs() < 0.01); // Starts full
        assert!(zone.contains(Vec2::new(12.0, 22.0))); // Inside
        assert!(!zone.contains(Vec2::new(20.0, 20.0))); // Outside
    }

    #[test]
    fn test_resource_zone_depletion() {
        let mut zone = ResourceZone::new(
            Vec2::new(0.0, 0.0),
            ResourceType::Stone,
            5.0,
        );

        let gathered = zone.gather(0.3);
        assert!((gathered - 0.3).abs() < 0.01);
        assert!((zone.current - 0.7).abs() < 0.01);

        // Can't gather more than available
        let gathered = zone.gather(1.0);
        assert!((gathered - 0.7).abs() < 0.01);
        assert!(zone.current < 0.01);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib simulation::resource_zone`
Expected: FAIL with "can't find crate for `resource_zone`"

**Step 3: Write minimal implementation**

```rust
// src/simulation/resource_zone.rs
use serde::{Deserialize, Serialize};
use crate::core::types::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Wood,
    Stone,
    Ore,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceZone {
    pub position: Vec2,
    pub resource_type: ResourceType,
    pub radius: f32,
    pub current: f32,    // 0.0 to 1.0
    pub max: f32,        // Usually 1.0
    pub regen_rate: f32, // Per tick
}

impl ResourceZone {
    pub fn new(position: Vec2, resource_type: ResourceType, radius: f32) -> Self {
        Self {
            position,
            resource_type,
            radius,
            current: 1.0,
            max: 1.0,
            regen_rate: 0.0001, // Very slow regen
        }
    }

    pub fn contains(&self, pos: Vec2) -> bool {
        self.position.distance(&pos) <= self.radius
    }

    /// Gather resources, returns amount actually gathered
    pub fn gather(&mut self, amount: f32) -> f32 {
        let gathered = amount.min(self.current);
        self.current -= gathered;
        gathered
    }

    /// Regenerate resources over time
    pub fn regenerate(&mut self) {
        self.current = (self.current + self.regen_rate).min(self.max);
    }
}
```

**Step 4: Add to mod.rs**

```rust
// In src/simulation/mod.rs, add:
pub mod resource_zone;
pub use resource_zone::{ResourceZone, ResourceType};
```

**Step 5: Add to World**

```rust
// In src/ecs/world.rs, add field:
pub resource_zones: Vec<ResourceZone>,

// In World::new(), initialize:
resource_zones: Vec::new(),
```

**Step 6: Run tests**

Run: `cargo test --lib simulation::resource_zone`
Expected: PASS

**Step 7: Commit**

```bash
git add src/simulation/resource_zone.rs src/simulation/mod.rs src/ecs/world.rs
git commit -m "feat: add ResourceZone for gatherable resources"
```

---

## Task 2: Implement Follow Action

Make Follow track a moving target entity each tick.

**Files:**
- Modify: `src/simulation/tick.rs`

**Step 1: Write the failing test**

```rust
// Add to tests in tick.rs or create new test file
#[test]
fn test_follow_tracks_moving_target() {
    let mut world = World::new();

    // Spawn follower at origin
    let follower_id = world.spawn_human("Follower", Vec2::new(0.0, 0.0));

    // Spawn target at (10, 0)
    let target_id = world.spawn_human("Target", Vec2::new(10.0, 0.0));

    // Give follower a Follow task
    let follower_idx = world.humans.index_of(follower_id).unwrap();
    let task = Task::new(ActionId::Follow, TaskPriority::Normal, 0)
        .with_entity(target_id);
    world.humans.task_queues[follower_idx].push(task);

    // Run a tick
    run_simulation_tick(&mut world);

    // Follower should have moved toward target
    let follower_pos = world.humans.positions[follower_idx];
    assert!(follower_pos.x > 0.0, "Follower should move toward target");

    // Move target
    let target_idx = world.humans.index_of(target_id).unwrap();
    world.humans.positions[target_idx] = Vec2::new(10.0, 10.0);

    // Run another tick
    run_simulation_tick(&mut world);

    // Follower should now be moving toward new position
    let follower_pos = world.humans.positions[follower_idx];
    assert!(follower_pos.y > 0.0 || follower_pos.x > 2.0, "Follower should track moving target");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_follow_tracks_moving_target`
Expected: FAIL (Follow not implemented)

**Step 3: Implement Follow in execute_tasks**

In `src/simulation/tick.rs`, inside the `execute_tasks` function, add a case for `ActionId::Follow`:

```rust
ActionId::Follow => {
    if let Some(target_id) = target_entity {
        // Find target's current position
        if let Some(target_idx) = world.humans.index_of(target_id) {
            let target_pos = world.humans.positions[target_idx];
            let current = world.humans.positions[i];
            let direction = (target_pos - current).normalize();
            let speed = 2.0;

            if direction.length() > 0.0 {
                world.humans.positions[i] = current + direction * speed;
            }

            // Follow never auto-completes - continues until interrupted
            false
        } else {
            // Target doesn't exist, complete
            true
        }
    } else {
        true // No target, complete immediately
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_follow_tracks_moving_target`
Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat: Follow action tracks moving target entity"
```

---

## Task 3: Implement Rest Action

Rest reduces fatigue and keeps entity stationary.

**Files:**
- Modify: `src/simulation/tick.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_rest_reduces_fatigue() {
    let mut world = World::new();
    let id = world.spawn_human("Tired", Vec2::new(0.0, 0.0));
    let idx = world.humans.index_of(id).unwrap();

    // Set high fatigue
    world.humans.body_states[idx].fatigue = 0.8;

    // Give Rest task
    let task = Task::new(ActionId::Rest, TaskPriority::Normal, 0);
    world.humans.task_queues[idx].push(task);

    let initial_fatigue = world.humans.body_states[idx].fatigue;

    // Run 10 ticks
    for _ in 0..10 {
        run_simulation_tick(&mut world);
    }

    let final_fatigue = world.humans.body_states[idx].fatigue;
    assert!(final_fatigue < initial_fatigue, "Rest should reduce fatigue");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_rest_reduces_fatigue`
Expected: FAIL (Rest doesn't reduce fatigue)

**Step 3: Implement Rest in execute_tasks**

```rust
ActionId::Rest => {
    // Stay still
    // (velocity is not used in current implementation)

    // Reduce fatigue
    world.humans.body_states[i].fatigue =
        (world.humans.body_states[i].fatigue - 0.01).max(0.0);

    // Progress toward completion
    let duration = task.action.base_duration(); // 50 ticks
    task.progress += 1.0 / duration as f32;
    task.progress >= 1.0
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_rest_reduces_fatigue`
Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat: Rest action reduces fatigue over time"
```

---

## Task 4: Implement SeekSafety Action

SeekSafety moves entity away from perceived threats.

**Files:**
- Modify: `src/simulation/tick.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_seek_safety_moves_away_from_threats() {
    let mut world = World::new();

    // Entity at origin
    let id = world.spawn_human("Scared", Vec2::new(0.0, 0.0));
    let idx = world.humans.index_of(id).unwrap();

    // Threat nearby (another entity with hostile disposition)
    let threat_id = world.spawn_human("Threat", Vec2::new(5.0, 0.0));
    let threat_idx = world.humans.index_of(threat_id).unwrap();

    // Make threat hostile (record negative memory)
    world.humans.social_memories[idx].record_encounter(
        threat_id,
        EventType::HarmReceived,
        0.9,
        0,
    );

    // Give SeekSafety task with threat position
    let task = Task::new(ActionId::SeekSafety, TaskPriority::Critical, 0)
        .with_position(Vec2::new(5.0, 0.0)); // Threat location
    world.humans.task_queues[idx].push(task);

    // Run tick
    run_simulation_tick(&mut world);

    // Should have moved away (negative x direction)
    let pos = world.humans.positions[idx];
    assert!(pos.x < 0.0, "Should move away from threat");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_seek_safety_moves_away_from_threats`
Expected: FAIL

**Step 3: Implement SeekSafety in execute_tasks**

```rust
ActionId::SeekSafety => {
    // Move away from threat position (stored in target_position)
    if let Some(threat_pos) = target_pos {
        let current = world.humans.positions[i];
        let away = (current - threat_pos).normalize();
        let speed = 3.0; // Fast like Flee

        if away.length() > 0.0 {
            world.humans.positions[i] = current + away * speed;
        }

        // Check if far enough away (20 units = safe)
        let distance = world.humans.positions[i].distance(&threat_pos);
        distance > 20.0 // Complete when safe
    } else {
        true // No threat position, complete
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_seek_safety_moves_away_from_threats`
Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat: SeekSafety action moves entity away from threats"
```

---

## Task 5: Implement Social Actions (TalkTo, Help, Trade)

Social actions require proximity, provide mutual satisfaction, and trigger reciprocal engagement.

**Files:**
- Modify: `src/simulation/tick.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_social_action_requires_proximity() {
    let mut world = World::new();

    // Two entities far apart
    let a_id = world.spawn_human("Alice", Vec2::new(0.0, 0.0));
    let b_id = world.spawn_human("Bob", Vec2::new(20.0, 0.0));

    let a_idx = world.humans.index_of(a_id).unwrap();
    let b_idx = world.humans.index_of(b_id).unwrap();

    // Set high social need
    world.humans.needs[a_idx].social = 0.8;
    world.humans.needs[b_idx].social = 0.8;

    // Alice tries to TalkTo Bob
    let task = Task::new(ActionId::TalkTo, TaskPriority::Normal, 0)
        .with_entity(b_id);
    world.humans.task_queues[a_idx].push(task);

    // Run tick
    run_simulation_tick(&mut world);

    // Alice should have moved toward Bob (not satisfied yet)
    let alice_pos = world.humans.positions[a_idx];
    assert!(alice_pos.x > 0.0, "Should move toward target");

    // Social need should NOT be satisfied yet (too far)
    assert!(world.humans.needs[a_idx].social > 0.7, "Should not satisfy social need when far");
}

#[test]
fn test_social_action_mutual_satisfaction() {
    let mut world = World::new();

    // Two entities close together
    let a_id = world.spawn_human("Alice", Vec2::new(0.0, 0.0));
    let b_id = world.spawn_human("Bob", Vec2::new(3.0, 0.0)); // Within 5.0

    let a_idx = world.humans.index_of(a_id).unwrap();
    let b_idx = world.humans.index_of(b_id).unwrap();

    // Set high social need
    world.humans.needs[a_idx].social = 0.8;
    world.humans.needs[b_idx].social = 0.8;

    // Alice talks to Bob
    let task = Task::new(ActionId::TalkTo, TaskPriority::Normal, 0)
        .with_entity(b_id);
    world.humans.task_queues[a_idx].push(task);

    // Run several ticks
    for _ in 0..5 {
        run_simulation_tick(&mut world);
    }

    // Both should have reduced social need
    assert!(world.humans.needs[a_idx].social < 0.8, "Alice's social need should decrease");
    assert!(world.humans.needs[b_idx].social < 0.8, "Bob's social need should decrease too");

    // Bob should have a reciprocal task or have had one
    // Check memories instead (more reliable)
    assert!(world.humans.social_memories[a_idx].find_slot(b_id).is_some(),
            "Alice should remember Bob");
    assert!(world.humans.social_memories[b_idx].find_slot(a_id).is_some(),
            "Bob should remember Alice");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_social_action`
Expected: FAIL

**Step 3: Implement social actions in execute_tasks**

```rust
ActionId::TalkTo | ActionId::Help | ActionId::Trade => {
    if let Some(target_id) = target_entity {
        if let Some(target_idx) = world.humans.index_of(target_id) {
            let current = world.humans.positions[i];
            let target_pos = world.humans.positions[target_idx];
            let distance = current.distance(&target_pos);

            const SOCIAL_RANGE: f32 = 5.0;

            if distance > SOCIAL_RANGE {
                // Move toward target
                let direction = (target_pos - current).normalize();
                let speed = 2.0;
                if direction.length() > 0.0 {
                    world.humans.positions[i] = current + direction * speed;
                }
                false // Not complete yet
            } else {
                // In range - interact

                // Determine satisfaction amounts based on action
                let (social_amount, purpose_amount) = match action {
                    ActionId::TalkTo => (0.02, 0.0),
                    ActionId::Help => (0.03, 0.01),
                    ActionId::Trade => (0.02, 0.02),
                    _ => (0.0, 0.0),
                };

                // Satisfy needs for both parties
                world.humans.needs[i].satisfy(NeedType::Social, social_amount);
                world.humans.needs[target_idx].satisfy(NeedType::Social, social_amount);

                if purpose_amount > 0.0 {
                    world.humans.needs[i].satisfy(NeedType::Purpose, purpose_amount);
                    world.humans.needs[target_idx].satisfy(NeedType::Purpose, purpose_amount);
                }

                // Create memories for both (on first tick of interaction)
                if task.progress < 0.01 {
                    let event_type = match action {
                        ActionId::Help => EventType::AidGiven,
                        ActionId::Trade => EventType::Transaction,
                        _ => EventType::Observation,
                    };

                    world.humans.social_memories[i].record_encounter(
                        target_id, event_type, 0.5, world.current_tick
                    );

                    let target_event = match action {
                        ActionId::Help => EventType::AidReceived,
                        _ => event_type,
                    };
                    world.humans.social_memories[target_idx].record_encounter(
                        world.humans.ids[i], target_event, 0.5, world.current_tick
                    );

                    // Push reciprocal task if target is idle
                    let target_is_idle = world.humans.task_queues[target_idx]
                        .current()
                        .map(|t| matches!(t.action, ActionId::IdleWander | ActionId::IdleObserve))
                        .unwrap_or(true);

                    let target_doing_social = world.humans.task_queues[target_idx]
                        .current()
                        .map(|t| matches!(t.action, ActionId::TalkTo | ActionId::Help | ActionId::Trade))
                        .unwrap_or(false);

                    if target_is_idle && !target_doing_social {
                        let reciprocal = Task::new(action, TaskPriority::Normal, world.current_tick)
                            .with_entity(world.humans.ids[i]);
                        world.humans.task_queues[target_idx].push(reciprocal);
                    }
                }

                // Progress toward completion
                let duration = action.base_duration() as f32;
                let progress_rate = if duration > 0.0 { 1.0 / duration } else { 0.1 };
                task.progress += progress_rate;
                task.progress >= 1.0
            }
        } else {
            true // Target doesn't exist
        }
    } else {
        true // No target
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_social_action`
Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat: social actions require proximity and provide mutual engagement"
```

---

## Task 6: Implement Gather Action

Gather depletes ResourceZone and satisfies Purpose need.

**Files:**
- Modify: `src/simulation/tick.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_gather_depletes_resource_zone() {
    let mut world = World::new();

    // Create resource zone
    let zone = ResourceZone::new(
        Vec2::new(10.0, 0.0),
        ResourceType::Wood,
        5.0,
    );
    world.resource_zones.push(zone);

    // Spawn gatherer
    let id = world.spawn_human("Gatherer", Vec2::new(10.0, 0.0)); // At zone
    let idx = world.humans.index_of(id).unwrap();

    // Set purpose need
    world.humans.needs[idx].purpose = 0.7;

    // Give Gather task (target position is zone center)
    let task = Task::new(ActionId::Gather, TaskPriority::Normal, 0)
        .with_position(Vec2::new(10.0, 0.0));
    world.humans.task_queues[idx].push(task);

    let initial_resources = world.resource_zones[0].current;

    // Run 10 ticks
    for _ in 0..10 {
        run_simulation_tick(&mut world);
    }

    // Resources should be depleted
    assert!(world.resource_zones[0].current < initial_resources,
            "Resource zone should be depleted");

    // Purpose need should decrease
    assert!(world.humans.needs[idx].purpose < 0.7,
            "Purpose need should be satisfied");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_gather_depletes_resource_zone`
Expected: FAIL

**Step 3: Implement Gather in execute_tasks**

```rust
ActionId::Gather => {
    if let Some(zone_pos) = target_pos {
        let current = world.humans.positions[i];

        // Find resource zone at target position
        let zone_idx = world.resource_zones.iter().position(|z| z.contains(zone_pos));

        if let Some(zone_idx) = zone_idx {
            let distance = current.distance(&zone_pos);

            if distance > 2.0 {
                // Move to zone
                let direction = (zone_pos - current).normalize();
                let speed = 2.0;
                if direction.length() > 0.0 {
                    world.humans.positions[i] = current + direction * speed;
                }
                false
            } else {
                // At zone - gather
                let gathered = world.resource_zones[zone_idx].gather(0.02);

                if gathered > 0.0 {
                    // Satisfy purpose proportional to gathered amount
                    world.humans.needs[i].satisfy(NeedType::Purpose, gathered * 0.5);
                }

                // Complete when zone empty or task duration reached
                let duration = action.base_duration() as f32; // 40 ticks
                let progress_rate = if duration > 0.0 { 1.0 / duration } else { 0.1 };
                task.progress += progress_rate;

                task.progress >= 1.0 || world.resource_zones[zone_idx].current <= 0.0
            }
        } else {
            true // No zone at target
        }
    } else {
        true // No target position
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_gather_depletes_resource_zone`
Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat: Gather action depletes resource zones and satisfies purpose"
```

---

## Task 7: Implement IdleWander Action

IdleWander picks a random nearby point and moves slowly toward it.

**Files:**
- Modify: `src/simulation/tick.rs`
- Modify: `Cargo.toml` (add rand if not present)

**Step 1: Write the failing test**

```rust
#[test]
fn test_idle_wander_moves_slowly() {
    let mut world = World::new();

    let id = world.spawn_human("Wanderer", Vec2::new(0.0, 0.0));
    let idx = world.humans.index_of(id).unwrap();

    // Give IdleWander task
    let task = Task::new(ActionId::IdleWander, TaskPriority::Low, 0);
    world.humans.task_queues[idx].push(task);

    let initial_pos = world.humans.positions[idx];

    // Run several ticks
    for _ in 0..10 {
        run_simulation_tick(&mut world);
    }

    let final_pos = world.humans.positions[idx];
    let distance_moved = initial_pos.distance(&final_pos);

    // Should have moved (but slowly)
    assert!(distance_moved > 0.0, "Should move when wandering");
    assert!(distance_moved < 20.0, "Should move slowly"); // Not at full speed
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_idle_wander_moves_slowly`
Expected: FAIL

**Step 3: Implement IdleWander in execute_tasks**

```rust
ActionId::IdleWander => {
    // If no target or reached target, pick new random point
    let current = world.humans.positions[i];
    let needs_new_target = target_pos.map(|t| current.distance(&t) < 1.0).unwrap_or(true);

    if needs_new_target {
        // Pick random point within 10 units
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let angle = rng.gen::<f32>() * std::f32::consts::TAU;
        let distance = rng.gen::<f32>() * 10.0;
        let offset = Vec2::new(angle.cos() * distance, angle.sin() * distance);
        task.target_position = Some(current + offset);
    }

    if let Some(target) = task.target_position {
        let direction = (target - current).normalize();
        let speed = 1.0; // Slow wandering speed

        if direction.length() > 0.0 {
            world.humans.positions[i] = current + direction * speed;
        }
    }

    // IdleWander never auto-completes (duration 0)
    false
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_idle_wander_moves_slowly`
Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat: IdleWander picks random nearby point and moves slowly"
```

---

## Task 8: Implement IdleObserve Action

IdleObserve keeps entity stationary. Perception system gives 1.5x range.

**Files:**
- Modify: `src/simulation/tick.rs`
- Modify: `src/simulation/perception.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_idle_observe_stays_still() {
    let mut world = World::new();

    let id = world.spawn_human("Observer", Vec2::new(5.0, 5.0));
    let idx = world.humans.index_of(id).unwrap();

    let task = Task::new(ActionId::IdleObserve, TaskPriority::Low, 0);
    world.humans.task_queues[idx].push(task);

    let initial_pos = world.humans.positions[idx];

    for _ in 0..10 {
        run_simulation_tick(&mut world);
    }

    let final_pos = world.humans.positions[idx];
    assert!((initial_pos.x - final_pos.x).abs() < 0.01, "Should not move");
    assert!((initial_pos.y - final_pos.y).abs() < 0.01, "Should not move");
}

// Test for perception boost would go in perception.rs tests
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_idle_observe_stays_still`
Expected: FAIL

**Step 3: Implement IdleObserve in execute_tasks**

```rust
ActionId::IdleObserve => {
    // Stay still - don't update position
    // Perception boost is handled in perception system

    // IdleObserve never auto-completes (duration 0)
    false
}
```

**Step 4: Modify perception.rs for perception boost**

In `src/simulation/perception.rs`, where perception range is calculated:

```rust
// When calculating perception range, check current action
let base_range = 50.0;
let perception_range = if world.humans.task_queues[idx]
    .current()
    .map(|t| t.action == ActionId::IdleObserve)
    .unwrap_or(false)
{
    base_range * 1.5
} else {
    base_range
};
```

**Step 5: Run test to verify it passes**

Run: `cargo test test_idle_observe`
Expected: PASS

**Step 6: Commit**

```bash
git add src/simulation/tick.rs src/simulation/perception.rs
git commit -m "feat: IdleObserve keeps entity still with 1.5x perception range"
```

---

## Task 9: Refactor execute_tasks for Clarity

Current execute_tasks has grown. Refactor action handling into helper functions.

**Files:**
- Modify: `src/simulation/tick.rs`

**Step 1: Create helper functions for each action category**

```rust
/// Execute movement actions (MoveTo, Follow, Flee)
fn execute_movement_action(
    world: &mut World,
    idx: usize,
    task: &mut Task
) -> bool {
    // ... existing MoveTo, Follow, Flee logic
}

/// Execute survival actions (Rest, Eat, SeekSafety)
fn execute_survival_action(
    world: &mut World,
    idx: usize,
    task: &mut Task,
) -> bool {
    // ... existing Rest, Eat, SeekSafety logic
}

/// Execute social actions (TalkTo, Help, Trade)
fn execute_social_action(
    world: &mut World,
    idx: usize,
    task: &mut Task,
) -> bool {
    // ... existing social logic
}

/// Execute work actions (Gather, Build, Craft, Repair)
fn execute_work_action(
    world: &mut World,
    idx: usize,
    task: &mut Task,
) -> bool {
    // ... existing work logic
}

/// Execute idle actions (IdleWander, IdleObserve)
fn execute_idle_action(
    world: &mut World,
    idx: usize,
    task: &mut Task,
) -> bool {
    // ... existing idle logic
}
```

**Step 2: Update execute_tasks to dispatch to helpers**

```rust
fn execute_tasks(world: &mut World) {
    let living_indices: Vec<usize> = world.humans.iter_living().collect();

    for i in living_indices {
        let is_complete = {
            let task = match world.humans.task_queues[i].current_mut() {
                Some(t) => t,
                None => continue,
            };

            match task.action.category() {
                ActionCategory::Movement => execute_movement_action(world, i, task),
                ActionCategory::Survival => execute_survival_action(world, i, task),
                ActionCategory::Social => execute_social_action(world, i, task),
                ActionCategory::Work => execute_work_action(world, i, task),
                ActionCategory::Idle => execute_idle_action(world, i, task),
                ActionCategory::Combat => false, // Combat stubs - don't complete
            }
        };

        if is_complete {
            world.humans.task_queues[i].complete_current();
        }
    }
}
```

**Step 3: Run all tests**

Run: `cargo test`
Expected: All tests PASS

**Step 4: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "refactor: extract action execution into category helper functions"
```

---

## Task 10: Integration Tests

Verify all actions work together in realistic scenarios.

**Files:**
- Modify: `tests/emergence_tests.rs`

**Step 1: Write integration tests**

```rust
#[test]
fn test_survival_priority_chain() {
    // Entity with high hunger should Eat, then if tired should Rest
    let mut world = World::new();

    // Create food zone
    world.food_zones.push(FoodZone::new(Vec2::new(0.0, 0.0), 10.0));

    let id = world.spawn_human("Survivor", Vec2::new(0.0, 0.0));
    let idx = world.humans.index_of(id).unwrap();

    // Very hungry and tired
    world.humans.needs[idx].food = 0.9;
    world.humans.body_states[idx].fatigue = 0.8;

    // Run simulation
    for _ in 0..100 {
        run_simulation_tick(&mut world);
    }

    // Hunger should be reduced (ate)
    assert!(world.humans.needs[idx].food < 0.9);
    // Fatigue should be reduced (rested)
    assert!(world.humans.body_states[idx].fatigue < 0.8);
}

#[test]
fn test_social_clustering() {
    // Multiple entities should naturally cluster through social actions
    let mut world = World::new();

    // Spawn entities spread out with high social need
    for i in 0..5 {
        let id = world.spawn_human(&format!("Person{}", i), Vec2::new(i as f32 * 20.0, 0.0));
        let idx = world.humans.index_of(id).unwrap();
        world.humans.needs[idx].social = 0.8;
    }

    // Run simulation
    for _ in 0..200 {
        run_simulation_tick(&mut world);
    }

    // Calculate average distance between entities
    let positions: Vec<Vec2> = world.humans.positions.iter().cloned().collect();
    let mut total_distance = 0.0;
    let mut pairs = 0;
    for i in 0..positions.len() {
        for j in (i+1)..positions.len() {
            total_distance += positions[i].distance(&positions[j]);
            pairs += 1;
        }
    }
    let avg_distance = total_distance / pairs as f32;

    // Entities should cluster closer together
    assert!(avg_distance < 40.0, "Entities should cluster through social interaction");
}

#[test]
fn test_resource_gathering_workflow() {
    let mut world = World::new();

    // Create resource zone
    world.resource_zones.push(ResourceZone::new(
        Vec2::new(20.0, 0.0),
        ResourceType::Wood,
        5.0,
    ));

    let id = world.spawn_human("Worker", Vec2::new(0.0, 0.0));
    let idx = world.humans.index_of(id).unwrap();
    world.humans.needs[idx].purpose = 0.8;

    // Give gather task
    let task = Task::new(ActionId::Gather, TaskPriority::Normal, 0)
        .with_position(Vec2::new(20.0, 0.0));
    world.humans.task_queues[idx].push(task);

    // Run until task completes or 200 ticks
    for _ in 0..200 {
        run_simulation_tick(&mut world);
        if world.humans.task_queues[idx].is_idle() {
            break;
        }
    }

    // Should have gathered resources
    assert!(world.resource_zones[0].current < 1.0, "Resources should be depleted");
    assert!(world.humans.needs[idx].purpose < 0.8, "Purpose need should be satisfied");
}
```

**Step 2: Run integration tests**

Run: `cargo test --test emergence_tests`
Expected: PASS

**Step 3: Commit**

```bash
git add tests/emergence_tests.rs
git commit -m "test: integration tests for action execution workflows"
```

---

## Summary

| Task | Action | Outcome |
|------|--------|---------|
| 1 | ResourceZone | New zone type for gatherable resources |
| 2 | Follow | Tracks moving target entity |
| 3 | Rest | Reduces fatigue over time |
| 4 | SeekSafety | Moves away from threats |
| 5 | TalkTo/Help/Trade | Proximity + mutual engagement + memories |
| 6 | Gather | Depletes ResourceZone + Purpose satisfaction |
| 7 | IdleWander | Random nearby point, slow movement |
| 8 | IdleObserve | Stationary with 1.5x perception |
| 9 | Refactor | Clean helper function structure |
| 10 | Integration | Verify emergent behaviors |

**Total: 10 tasks**
