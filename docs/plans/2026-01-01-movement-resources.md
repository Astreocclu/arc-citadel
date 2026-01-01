# Movement & Resources Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add purposeful movement toward goals and static food zones with mixed scarcity.

**Architecture:** Entities perceive nearby food zones during perception phase. When hungry, they move toward food using MoveTo action. Eat action only works when at a food zone. Scarce zones deplete and regenerate.

**Tech Stack:** Rust, existing SoA patterns, SparseHashGrid for entities (zones use linear search)

---

## Task 1: Add FoodZone struct and storage to World

**Files:**
- Modify: `src/ecs/world.rs`

**Step 1: Write the failing test**

```rust
// In src/ecs/world.rs, add to tests module
#[test]
fn test_food_zones_exist() {
    let mut world = World::new();
    assert!(world.food_zones.is_empty());
    
    world.add_food_zone(Vec2::new(100.0, 100.0), 20.0, Abundance::Unlimited);
    assert_eq!(world.food_zones.len(), 1);
    
    world.add_food_zone(Vec2::new(500.0, 500.0), 15.0, Abundance::Scarce { current: 100.0, max: 100.0, regen: 0.1 });
    assert_eq!(world.food_zones.len(), 2);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_food_zones_exist -- --nocapture`
Expected: FAIL with "food_zones not found" or "Abundance not found"

**Step 3: Write minimal implementation**

Add to `src/ecs/world.rs`:

```rust
use crate::core::types::Vec2;

/// Abundance level of a food zone
#[derive(Debug, Clone)]
pub enum Abundance {
    /// Infinite food - never depletes
    Unlimited,
    /// Limited food - depletes when eaten, regenerates over time
    Scarce { current: f32, max: f32, regen: f32 },
}

/// A static zone where entities can find food
#[derive(Debug, Clone)]
pub struct FoodZone {
    pub id: u32,
    pub position: Vec2,
    pub radius: f32,
    pub abundance: Abundance,
}

impl FoodZone {
    /// Check if entity at given position can eat from this zone
    pub fn contains(&self, pos: Vec2) -> bool {
        self.position.distance(&pos) <= self.radius
    }
    
    /// Try to consume food from this zone. Returns amount actually consumed.
    pub fn consume(&mut self, amount: f32) -> f32 {
        match &mut self.abundance {
            Abundance::Unlimited => amount,
            Abundance::Scarce { current, .. } => {
                let consumed = amount.min(*current);
                *current -= consumed;
                consumed
            }
        }
    }
    
    /// Regenerate food for scarce zones
    pub fn regenerate(&mut self) {
        if let Abundance::Scarce { current, max, regen } = &mut self.abundance {
            *current = (*current + *regen).min(*max);
        }
    }
}
```

Add to `World` struct:
```rust
pub struct World {
    // ... existing fields
    pub food_zones: Vec<FoodZone>,
    next_food_zone_id: u32,
}
```

Add to `World::new()`:
```rust
food_zones: Vec::new(),
next_food_zone_id: 0,
```

Add method:
```rust
impl World {
    pub fn add_food_zone(&mut self, position: Vec2, radius: f32, abundance: Abundance) -> u32 {
        let id = self.next_food_zone_id;
        self.next_food_zone_id += 1;
        self.food_zones.push(FoodZone { id, position, radius, abundance });
        id
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_food_zones_exist -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ecs/world.rs
git commit -m "feat: add FoodZone struct with abundance levels"
```

---

## Task 2: Add nearest_food_zone to Perception

**Files:**
- Modify: `src/simulation/perception.rs`
- Modify: `src/simulation/tick.rs`

**Step 1: Write the failing test**

```rust
// In src/simulation/perception.rs tests
#[test]
fn test_perception_finds_food_zone() {
    use crate::core::types::Vec2;
    use crate::ecs::world::{Abundance, FoodZone};
    
    let zones = vec![
        FoodZone { id: 0, position: Vec2::new(50.0, 50.0), radius: 10.0, abundance: Abundance::Unlimited },
        FoodZone { id: 1, position: Vec2::new(200.0, 200.0), radius: 20.0, abundance: Abundance::Unlimited },
    ];
    
    let observer_pos = Vec2::new(60.0, 60.0);
    let perception_range = 100.0;
    
    let nearest = find_nearest_food_zone(observer_pos, perception_range, &zones);
    
    assert!(nearest.is_some());
    let (zone_id, zone_pos, distance) = nearest.unwrap();
    assert_eq!(zone_id, 0);  // Closer zone
    assert!(distance < 20.0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_perception_finds_food_zone -- --nocapture`
Expected: FAIL with "find_nearest_food_zone not found"

**Step 3: Write minimal implementation**

Add to `src/simulation/perception.rs`:

```rust
use crate::ecs::world::FoodZone;

/// Find the nearest food zone within perception range
pub fn find_nearest_food_zone(
    observer_pos: Vec2,
    perception_range: f32,
    food_zones: &[FoodZone],
) -> Option<(u32, Vec2, f32)> {
    let mut nearest: Option<(u32, Vec2, f32)> = None;
    
    for zone in food_zones {
        let distance = observer_pos.distance(&zone.position);
        if distance <= perception_range {
            if nearest.is_none() || distance < nearest.as_ref().unwrap().2 {
                nearest = Some((zone.id, zone.position, distance));
            }
        }
    }
    
    nearest
}
```

Add field to `Perception` struct:
```rust
pub struct Perception {
    pub observer: EntityId,
    pub perceived_entities: Vec<PerceivedEntity>,
    pub perceived_objects: Vec<PerceivedObject>,
    pub perceived_events: Vec<PerceivedEvent>,
    /// Nearest food zone: (zone_id, position, distance)
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_perception_finds_food_zone -- --nocapture`
Expected: PASS

**Step 5: Integrate into tick.rs perception phase**

Modify `perception_system_parallel` in `tick.rs` to include food zone lookup (or pass zones to perception function).

**Step 6: Commit**

```bash
git add src/simulation/perception.rs src/simulation/tick.rs
git commit -m "feat: perception finds nearest food zone"
```

---

## Task 3: Add movement to execute_tasks

**Files:**
- Modify: `src/simulation/tick.rs`
- Modify: `src/entity/tasks.rs` (add target_position field if missing)

**Step 1: Write the failing test**

```rust
// In src/simulation/tick.rs tests
#[test]
fn test_moveto_changes_position() {
    let mut world = World::new();
    world.spawn_human("Mover".into());
    
    // Set initial position
    world.humans.positions[0] = Vec2::new(0.0, 0.0);
    
    // Assign MoveTo task
    let target = Vec2::new(100.0, 0.0);
    let task = Task {
        action: ActionId::MoveTo,
        target_position: Some(target),
        target_entity: None,
        priority: TaskPriority::Normal,
        created_tick: 0,
        progress: 0.0,
        source: TaskSource::Autonomous,
    };
    world.humans.task_queues[0].push(task);
    
    // Run tick
    run_simulation_tick(&mut world);
    
    // Position should have moved toward target
    assert!(world.humans.positions[0].x > 0.0);
    assert!(world.humans.positions[0].x < 100.0);  // Not teleported
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_moveto_changes_position -- --nocapture`
Expected: FAIL (position unchanged, still 0.0)

**Step 3: Write minimal implementation**

Modify `execute_tasks` in `src/simulation/tick.rs`:

```rust
fn execute_tasks(world: &mut World) {
    let living_indices: Vec<usize> = world.humans.iter_living().collect();
    
    for i in living_indices {
        let task_info = world.humans.task_queues[i].current_mut().map(|task| {
            let action = task.action;
            let target_pos = task.target_position;
            let target_entity = task.target_entity;
            
            // Movement actions
            let movement_complete = match action {
                ActionId::MoveTo => {
                    if let Some(target) = target_pos {
                        let current = world.humans.positions[i];
                        let direction = (target - current).normalize();
                        let speed = 2.0;  // units per tick
                        
                        // Move toward target
                        if direction.length() > 0.0 {
                            world.humans.positions[i] = current + direction * speed;
                        }
                        
                        // Check if arrived
                        world.humans.positions[i].distance(&target) < 2.0
                    } else {
                        true  // No target, complete immediately
                    }
                }
                ActionId::Flee => {
                    if let Some(threat_pos) = target_pos {
                        let current = world.humans.positions[i];
                        let away = (current - threat_pos).normalize();
                        let speed = 3.0;  // Flee faster
                        world.humans.positions[i] = current + away * speed;
                    }
                    false  // Flee continues until interrupted
                }
                _ => false,
            };
            
            // Progress for non-movement actions
            let progress_rate = match action.base_duration() {
                0 => 0.1,
                1..=60 => 0.05,
                _ => 0.02,
            };
            
            if !matches!(action, ActionId::MoveTo | ActionId::Flee) {
                task.progress += progress_rate;
            }
            
            let is_complete = movement_complete || task.progress >= 1.0;
            (action, is_complete)
        });
        
        // ... rest of need satisfaction logic
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_moveto_changes_position -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat: MoveTo and Flee actions move entities"
```

---

## Task 4: Update action selection to use food zones

**Files:**
- Modify: `src/simulation/action_select.rs`
- Modify: `src/simulation/tick.rs`

**Step 1: Write the failing test**

```rust
// In src/simulation/action_select.rs tests
#[test]
fn test_hungry_entity_moves_to_food() {
    let body = BodyState::new();
    let mut needs = Needs::default();
    needs.food = 0.85;  // Critical hunger
    let thoughts = ThoughtBuffer::new();
    let values = HumanValues::default();
    
    let ctx = SelectionContext {
        body: &body,
        needs: &needs,
        thoughts: &thoughts,
        values: &values,
        has_current_task: false,
        threat_nearby: false,
        food_available: false,  // No food at current position
        safe_location: true,
        entity_nearby: false,
        current_tick: 0,
        nearest_food_zone: Some((0, Vec2::new(100.0, 100.0), 50.0)),  // Food zone nearby
    };
    
    let task = select_action_human(&ctx);
    assert!(task.is_some());
    let task = task.unwrap();
    assert_eq!(task.action, ActionId::MoveTo);
    assert_eq!(task.target_position, Some(Vec2::new(100.0, 100.0)));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_hungry_entity_moves_to_food -- --nocapture`
Expected: FAIL (nearest_food_zone field doesn't exist on SelectionContext)

**Step 3: Write minimal implementation**

Add to `SelectionContext`:
```rust
pub struct SelectionContext<'a> {
    // ... existing fields
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,  // (id, position, distance)
}
```

Modify `select_critical_response`:
```rust
fn select_critical_response(need: NeedType, ctx: &SelectionContext) -> Option<Task> {
    let action = match need {
        NeedType::Food => {
            if ctx.food_available {
                ActionId::Eat  // Already at food
            } else if let Some((_, food_pos, _)) = ctx.nearest_food_zone {
                // Move to food zone
                return Some(Task {
                    action: ActionId::MoveTo,
                    target_position: Some(food_pos),
                    target_entity: None,
                    priority: TaskPriority::Critical,
                    created_tick: ctx.current_tick,
                    progress: 0.0,
                    source: TaskSource::Autonomous,
                });
            } else {
                ActionId::IdleWander  // No food known, wander to find some
            }
        }
        // ... rest unchanged
    };
    // ...
}
```

**Step 4: Update tick.rs to pass food zone info to action selection**

In `select_actions`:
```rust
// Find food zone for this entity
let nearest_food = find_nearest_food_zone(
    world.humans.positions[i],
    50.0,  // perception range
    &world.food_zones,
);

// Check if entity is AT a food zone
let food_available = world.food_zones.iter()
    .any(|zone| zone.contains(world.humans.positions[i]));

let ctx = SelectionContext {
    // ... existing fields
    food_available,
    nearest_food_zone: nearest_food,
};
```

**Step 5: Run test to verify it passes**

Run: `cargo test test_hungry_entity_moves_to_food -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add src/simulation/action_select.rs src/simulation/tick.rs
git commit -m "feat: hungry entities move toward food zones"
```

---

## Task 5: Eat action consumes from food zone

**Files:**
- Modify: `src/simulation/tick.rs`
- Modify: `src/entity/tasks.rs` (add target_zone_id field)

**Step 1: Write the failing test**

```rust
#[test]
fn test_eating_depletes_scarce_zone() {
    let mut world = World::new();
    world.spawn_human("Eater".into());
    
    // Add scarce food zone
    let zone_id = world.add_food_zone(
        Vec2::new(0.0, 0.0),
        10.0,
        Abundance::Scarce { current: 10.0, max: 100.0, regen: 0.1 },
    );
    
    // Entity at the zone
    world.humans.positions[0] = Vec2::new(0.0, 0.0);
    world.humans.needs[0].food = 0.9;  // Very hungry
    
    // Run several ticks (entity should eat and deplete)
    for _ in 0..50 {
        run_simulation_tick(&mut world);
    }
    
    // Zone should be depleted
    if let Abundance::Scarce { current, .. } = &world.food_zones[0].abundance {
        assert!(*current < 10.0, "Zone should be partially depleted");
    }
    
    // Entity should be less hungry
    assert!(world.humans.needs[0].food < 0.9);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_eating_depletes_scarce_zone -- --nocapture`
Expected: FAIL (zone not depleted)

**Step 3: Write minimal implementation**

Modify `execute_tasks` to consume from zones:

```rust
// When executing Eat action
ActionId::Eat => {
    // Find zone entity is standing in
    let pos = world.humans.positions[i];
    for zone in &mut world.food_zones {
        if zone.contains(pos) {
            let consumed = zone.consume(0.1);  // Consume rate per tick
            if consumed > 0.0 {
                world.humans.needs[i].satisfy(NeedType::Food, consumed * 0.5);
            }
            break;
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_eating_depletes_scarce_zone -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat: Eat action consumes from food zones"
```

---

## Task 6: Food zone regeneration

**Files:**
- Modify: `src/simulation/tick.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_scarce_zone_regenerates() {
    let mut world = World::new();
    world.add_food_zone(
        Vec2::new(0.0, 0.0),
        10.0,
        Abundance::Scarce { current: 0.0, max: 100.0, regen: 1.0 },  // Empty but regens
    );
    
    // Run ticks
    for _ in 0..50 {
        run_simulation_tick(&mut world);
    }
    
    // Zone should have regenerated
    if let Abundance::Scarce { current, .. } = &world.food_zones[0].abundance {
        assert!(*current > 0.0, "Zone should have regenerated");
    }
}
```

**Step 2: Run test to verify it fails**

Expected: FAIL (zone stays at 0)

**Step 3: Write minimal implementation**

Add to `run_simulation_tick`:

```rust
pub fn run_simulation_tick(world: &mut World) {
    update_needs(world);
    let perceptions = run_perception(world);
    generate_thoughts(world, &perceptions);
    decay_thoughts(world);
    select_actions(world);
    execute_tasks(world);
    regenerate_food_zones(world);  // NEW
    world.tick();
}

fn regenerate_food_zones(world: &mut World) {
    for zone in &mut world.food_zones {
        zone.regenerate();
    }
}
```

**Step 4: Run test to verify it passes**

Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat: scarce food zones regenerate over time"
```

---

## Task 7: Spawn food zones in stress test and emergence sim

**Files:**
- Modify: `src/bin/stress_test.rs`
- Modify: `src/bin/emergence_sim.rs`

**Step 1: Add zones to emergence_sim.rs**

```rust
// After spawning entities
println!("Creating food zones...");

// Abundant zones (4 corners)
world.add_food_zone(Vec2::new(200.0, 200.0), 100.0, Abundance::Unlimited);
world.add_food_zone(Vec2::new(1800.0, 200.0), 100.0, Abundance::Unlimited);
world.add_food_zone(Vec2::new(200.0, 1800.0), 100.0, Abundance::Unlimited);
world.add_food_zone(Vec2::new(1800.0, 1800.0), 100.0, Abundance::Unlimited);

// Scarce zones (center area - creates competition)
for x in (600..1400).step_by(200) {
    for y in (600..1400).step_by(200) {
        world.add_food_zone(
            Vec2::new(x as f32, y as f32),
            50.0,
            Abundance::Scarce { current: 500.0, max: 500.0, regen: 5.0 },
        );
    }
}
```

**Step 2: Run emergence simulation**

```bash
cargo run --bin emergence_sim --release
```

Expected: Entities migrate toward food zones, scarce zones deplete and regenerate

**Step 3: Commit**

```bash
git add src/bin/emergence_sim.rs src/bin/stress_test.rs
git commit -m "feat: add food zones to simulation binaries"
```

---

## Task 8: Integration test - full movement to food cycle

**Files:**
- Create: `tests/movement_integration.rs`

**Step 1: Write integration test**

```rust
use arc_citadel::ecs::world::{World, Abundance};
use arc_citadel::core::types::Vec2;
use arc_citadel::simulation::tick::run_simulation_tick;

#[test]
fn test_entity_moves_to_food_and_eats() {
    let mut world = World::new();
    
    // Spawn entity far from food
    world.spawn_human("Hungry".into());
    world.humans.positions[0] = Vec2::new(0.0, 0.0);
    world.humans.needs[0].food = 0.9;  // Very hungry
    
    // Add food zone far away
    world.add_food_zone(Vec2::new(100.0, 0.0), 10.0, Abundance::Unlimited);
    
    // Record initial state
    let initial_food_need = world.humans.needs[0].food;
    let initial_x = world.humans.positions[0].x;
    
    // Run simulation
    for _ in 0..100 {
        run_simulation_tick(&mut world);
    }
    
    // Entity should have moved toward food
    assert!(world.humans.positions[0].x > initial_x + 50.0, 
        "Entity should have moved toward food zone");
    
    // Entity should have eaten (reduced hunger)
    assert!(world.humans.needs[0].food < initial_food_need,
        "Entity should have eaten and reduced hunger");
}
```

**Step 2: Run integration test**

```bash
cargo test test_entity_moves_to_food_and_eats -- --nocapture
```

Expected: PASS

**Step 3: Commit**

```bash
git add tests/movement_integration.rs
git commit -m "test: integration test for movement to food cycle"
```

---

## Verification

After all tasks complete, run:

```bash
cargo test
cargo run --bin emergence_sim --release
```

Expected:
- All tests pass
- Emergence simulation shows entities migrating to food zones
- Scarce zones show depletion/regeneration patterns
- No performance regression (300+ ticks/sec)
