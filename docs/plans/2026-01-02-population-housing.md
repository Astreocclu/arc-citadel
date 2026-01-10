# Population & Housing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement housing capacity that affects population, with homelessness penalties, population growth when needs are met, and food consumption.

**Architecture:** Housing is tracked via `assigned_house: Vec<Option<BuildingId>>` on HumanArchetype (SoA pattern). Homeless entities get 1.5x need decay. Daily systems handle housing assignment, food consumption, and population growth. Stockpile is added to World for resource management.

**Tech Stack:** Rust, existing SoA archetype pattern, existing Stockpile and BuildingArchetype

---

## Constants

```rust
// src/core/config.rs (or inline where used)
pub const TICKS_PER_DAY: u64 = 100;
pub const HOMELESS_DECAY_MULTIPLIER: f32 = 1.5;
pub const FOOD_PER_ENTITY_PER_DAY: u32 = 1;
pub const DAILY_GROWTH_CHANCE: f32 = 0.05;  // 5%
pub const GROWTH_FOOD_MULTIPLIER: u32 = 2;  // Need food > population * 2
```

---

## Task 1: Add housing_capacity() to BuildingType

**Files:**
- Modify: `src/city/building.rs:19-54` (BuildingType impl block)
- Test: `src/city/building.rs` (existing test module)

**Step 1: Write the failing test**

Add to the test module at the bottom of `src/city/building.rs`:

```rust
#[test]
fn test_building_housing_capacity() {
    assert_eq!(BuildingType::House.housing_capacity(), 4);
    assert_eq!(BuildingType::Farm.housing_capacity(), 0);
    assert_eq!(BuildingType::Workshop.housing_capacity(), 0);
    assert_eq!(BuildingType::Granary.housing_capacity(), 0);
    assert_eq!(BuildingType::Wall.housing_capacity(), 0);
    assert_eq!(BuildingType::Gate.housing_capacity(), 0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_building_housing_capacity --lib`
Expected: FAIL with "no method named `housing_capacity`"

**Step 3: Write minimal implementation**

Add to the `impl BuildingType` block (after `required_materials`):

```rust
/// Housing capacity for residential buildings
pub fn housing_capacity(&self) -> u32 {
    match self {
        BuildingType::House => 4,
        _ => 0,
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_building_housing_capacity --lib`
Expected: PASS

**Step 5: Commit**

```bash
git add src/city/building.rs
git commit -m "feat(building): add housing_capacity method to BuildingType"
```

---

## Task 2: Add assigned_house field to HumanArchetype

**Files:**
- Modify: `src/entity/species/human.rs:1-10` (imports)
- Modify: `src/entity/species/human.rs:47-65` (HumanArchetype struct)
- Modify: `src/entity/species/human.rs:68-108` (new() and spawn())
- Test: `src/entity/species/human.rs` (test module)

**Step 1: Write the failing test**

Add to test module:

```rust
#[test]
fn test_human_has_assigned_house() {
    use crate::city::building::BuildingId;

    let mut archetype = HumanArchetype::new();
    let id = EntityId::new();
    archetype.spawn(id, "Homeless Harry".into(), 0);

    assert_eq!(archetype.assigned_houses.len(), 1);
    assert_eq!(archetype.assigned_houses[0], None); // Starts homeless
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_human_has_assigned_house --lib`
Expected: FAIL with "no field `assigned_houses`"

**Step 3: Write minimal implementation**

Add import at top:
```rust
use crate::city::building::BuildingId;
```

Add field to HumanArchetype struct (after `combat_states`):
```rust
/// Assigned housing (None = homeless)
pub assigned_houses: Vec<Option<BuildingId>>,
```

Add to `new()`:
```rust
assigned_houses: Vec::new(),
```

Add to `spawn()` (after `self.combat_states.push(...)`):
```rust
self.assigned_houses.push(None);
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_human_has_assigned_house --lib`
Expected: PASS

**Step 5: Commit**

```bash
git add src/entity/species/human.rs
git commit -m "feat(human): add assigned_houses field to track housing"
```

---

## Task 3: Add Stockpile to World

**Files:**
- Modify: `src/ecs/world.rs:1-12` (imports)
- Modify: `src/ecs/world.rs:57-71` (World struct)
- Modify: `src/ecs/world.rs:73-102` (World::new)
- Test: `src/ecs/world.rs` (test module)

**Step 1: Write the failing test**

Add to test module:

```rust
#[test]
fn test_world_has_stockpile() {
    use crate::simulation::resource_zone::ResourceType;

    let mut world = World::new();

    // Stockpile should exist and be empty
    assert_eq!(world.stockpile.get(ResourceType::Food), 0);

    // Should be able to add resources
    world.stockpile.add(ResourceType::Food, 100);
    assert_eq!(world.stockpile.get(ResourceType::Food), 100);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_world_has_stockpile --lib`
Expected: FAIL with "no field `stockpile`"

**Step 3: Write minimal implementation**

Add import:
```rust
use crate::city::stockpile::Stockpile;
```

Add field to World struct:
```rust
/// Global stockpile for resources (MVP - later per-settlement)
pub stockpile: Stockpile,
```

Add to `World::new()`:
```rust
stockpile: Stockpile::new(),
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_world_has_stockpile --lib`
Expected: PASS

**Step 5: Commit**

```bash
git add src/ecs/world.rs
git commit -m "feat(world): add global stockpile for resource management"
```

---

## Task 4: Implement homeless need decay multiplier

**Files:**
- Modify: `src/simulation/tick.rs:67-98` (update_needs function)
- Test: `src/simulation/tick.rs` (add test)

**Step 1: Write the failing test**

Add to test module (or create one in tick.rs):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_homeless_decay_multiplier() {
        let mut world = World::new();

        // Spawn two humans
        let housed_id = world.spawn_human("Housed Helen".into());
        let homeless_id = world.spawn_human("Homeless Harry".into());

        // Assign house to one
        use crate::city::building::{BuildingId, BuildingType};
        let house_id = world.spawn_building(BuildingType::House, crate::core::types::Vec2::new(0.0, 0.0));
        let housed_idx = world.humans.index_of(housed_id).unwrap();
        world.humans.assigned_houses[housed_idx] = Some(house_id);

        // Set identical starting needs
        let homeless_idx = world.humans.index_of(homeless_id).unwrap();
        world.humans.needs[housed_idx].food = 0.5;
        world.humans.needs[homeless_idx].food = 0.5;

        // Run update_needs
        update_needs(&mut world);

        // Homeless should have higher food need (faster decay)
        let housed_food = world.humans.needs[housed_idx].food;
        let homeless_food = world.humans.needs[homeless_idx].food;

        assert!(homeless_food > housed_food,
            "Homeless should decay faster: homeless={} housed={}",
            homeless_food, housed_food);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_homeless_decay_multiplier --lib`
Expected: FAIL (both have same decay rate currently)

**Step 3: Write minimal implementation**

Modify `update_needs` function - replace the human processing loop:

```rust
// Process humans
let living_indices: Vec<usize> = world.humans.iter_living().collect();
for i in living_indices {
    let is_restful = world.humans.task_queues[i]
        .current()
        .map(|t| t.action.is_restful())
        .unwrap_or(true);
    let is_active = !is_restful;

    // Homeless entities have accelerated need decay
    let is_homeless = world.humans.assigned_houses[i].is_none();
    let homeless_mult = if is_homeless { 1.5 } else { 1.0 };
    let dt = 1.0 * homeless_mult;

    world.humans.needs[i].decay(dt, is_active);
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_homeless_decay_multiplier --lib`
Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat(tick): homeless entities decay needs 1.5x faster"
```

---

## Task 5: Create housing assignment system

**Files:**
- Create: `src/simulation/housing.rs`
- Modify: `src/simulation/mod.rs` (add module)
- Test: `src/simulation/housing.rs`

**Step 1: Write the failing test**

Create `src/simulation/housing.rs`:

```rust
//! Housing assignment system
//!
//! Assigns homeless entities to available houses.

use crate::ecs::world::World;
use crate::city::building::{BuildingId, BuildingType, BuildingState};

/// Assign homeless humans to available housing
pub fn assign_housing(world: &mut World) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Vec2;
    use crate::city::building::BuildingType;

    #[test]
    fn test_assign_housing_basic() {
        let mut world = World::new();

        // Spawn a house (capacity 4)
        let house_id = world.spawn_building(BuildingType::House, Vec2::new(0.0, 0.0));
        // Complete construction
        world.buildings.states[0] = BuildingState::Complete;

        // Spawn 2 homeless humans
        world.spawn_human("Alice".into());
        world.spawn_human("Bob".into());

        // Both should be homeless initially
        assert!(world.humans.assigned_houses[0].is_none());
        assert!(world.humans.assigned_houses[1].is_none());

        // Run housing assignment
        assign_housing(&mut world);

        // Both should now be housed
        assert_eq!(world.humans.assigned_houses[0], Some(house_id));
        assert_eq!(world.humans.assigned_houses[1], Some(house_id));
    }

    #[test]
    fn test_assign_housing_respects_capacity() {
        let mut world = World::new();

        // Spawn a house (capacity 4)
        world.spawn_building(BuildingType::House, Vec2::new(0.0, 0.0));
        world.buildings.states[0] = BuildingState::Complete;

        // Spawn 6 humans (exceeds capacity)
        for i in 0..6 {
            world.spawn_human(format!("Human {}", i));
        }

        assign_housing(&mut world);

        // Count housed vs homeless
        let housed = world.humans.assigned_houses.iter()
            .filter(|h| h.is_some())
            .count();
        let homeless = world.humans.assigned_houses.iter()
            .filter(|h| h.is_none())
            .count();

        assert_eq!(housed, 4, "Should house exactly 4 (capacity)");
        assert_eq!(homeless, 2, "Should have 2 homeless");
    }

    #[test]
    fn test_assign_housing_ignores_incomplete_buildings() {
        let mut world = World::new();

        // Spawn a house still under construction
        world.spawn_building(BuildingType::House, Vec2::new(0.0, 0.0));
        // Don't complete it - stays UnderConstruction

        world.spawn_human("Alice".into());

        assign_housing(&mut world);

        // Should still be homeless (building not complete)
        assert!(world.humans.assigned_houses[0].is_none());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_assign_housing --lib`
Expected: FAIL with "not yet implemented"

**Step 3: Write minimal implementation**

Replace the `todo!()` with:

```rust
/// Assign homeless humans to available housing
pub fn assign_housing(world: &mut World) {
    // Calculate available capacity per completed house
    let mut available: Vec<(BuildingId, u32)> = Vec::new();

    for idx in 0..world.buildings.count() {
        if world.buildings.states[idx] != BuildingState::Complete {
            continue;
        }

        let capacity = world.buildings.building_types[idx].housing_capacity();
        if capacity == 0 {
            continue;
        }

        let building_id = world.buildings.ids[idx];

        // Count current occupants
        let current_occupants = world.humans.assigned_houses.iter()
            .filter(|&&h| h == Some(building_id))
            .count() as u32;

        let remaining = capacity.saturating_sub(current_occupants);
        if remaining > 0 {
            available.push((building_id, remaining));
        }
    }

    // Assign homeless to available housing
    let mut available_iter = available.into_iter().peekable();
    let mut current_house = available_iter.peek().map(|(id, _)| *id);
    let mut remaining_capacity = available_iter.peek().map(|(_, cap)| *cap).unwrap_or(0);

    for idx in world.humans.iter_living() {
        if world.humans.assigned_houses[idx].is_some() {
            continue; // Already housed
        }

        if current_house.is_none() {
            break; // No more housing available
        }

        world.humans.assigned_houses[idx] = current_house;
        remaining_capacity -= 1;

        if remaining_capacity == 0 {
            available_iter.next();
            current_house = available_iter.peek().map(|(id, _)| *id);
            remaining_capacity = available_iter.peek().map(|(_, cap)| *cap).unwrap_or(0);
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_assign_housing --lib`
Expected: PASS

**Step 5: Add module to mod.rs**

Add to `src/simulation/mod.rs`:
```rust
pub mod housing;
```

**Step 6: Commit**

```bash
git add src/simulation/housing.rs src/simulation/mod.rs
git commit -m "feat(housing): add housing assignment system"
```

---

## Task 6: Create food consumption system

**Files:**
- Create: `src/simulation/consumption.rs`
- Modify: `src/simulation/mod.rs`
- Test: `src/simulation/consumption.rs`

**Step 1: Write the failing test**

Create `src/simulation/consumption.rs`:

```rust
//! Food consumption system
//!
//! Living entities consume food from the stockpile daily.

use crate::ecs::world::World;
use crate::simulation::resource_zone::ResourceType;

/// Consume food for all living entities
/// Returns number of entities that went hungry
pub fn consume_food(world: &mut World) -> u32 {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume_food_basic() {
        let mut world = World::new();

        // Add food to stockpile
        world.stockpile.add(ResourceType::Food, 100);

        // Spawn 3 humans
        world.spawn_human("Alice".into());
        world.spawn_human("Bob".into());
        world.spawn_human("Charlie".into());

        let hungry = consume_food(&mut world);

        assert_eq!(hungry, 0, "No one should be hungry");
        assert_eq!(world.stockpile.get(ResourceType::Food), 97, "Should consume 3 food");
    }

    #[test]
    fn test_consume_food_not_enough() {
        let mut world = World::new();

        // Only 1 food, 3 humans
        world.stockpile.add(ResourceType::Food, 1);

        world.spawn_human("Alice".into());
        world.spawn_human("Bob".into());
        world.spawn_human("Charlie".into());

        let hungry = consume_food(&mut world);

        assert_eq!(hungry, 2, "2 should go hungry");
        assert_eq!(world.stockpile.get(ResourceType::Food), 0, "Should consume all food");
    }

    #[test]
    fn test_consume_food_empty_stockpile() {
        let mut world = World::new();

        world.spawn_human("Alice".into());

        let hungry = consume_food(&mut world);

        assert_eq!(hungry, 1, "Should be hungry with no food");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_consume_food --lib`
Expected: FAIL with "not yet implemented"

**Step 3: Write minimal implementation**

Replace `todo!()`:

```rust
/// Consume food for all living entities
/// Returns number of entities that went hungry
pub fn consume_food(world: &mut World) -> u32 {
    let living_count = world.humans.iter_living().count() as u32;

    if living_count == 0 {
        return 0;
    }

    let food_needed = living_count; // 1 food per entity
    let food_consumed = world.stockpile.remove(ResourceType::Food, food_needed);

    // Return number who went hungry
    living_count - food_consumed
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_consume_food --lib`
Expected: PASS

**Step 5: Add module to mod.rs**

Add to `src/simulation/mod.rs`:
```rust
pub mod consumption;
```

**Step 6: Commit**

```bash
git add src/simulation/consumption.rs src/simulation/mod.rs
git commit -m "feat(consumption): add food consumption system"
```

---

## Task 7: Create population growth system

**Files:**
- Create: `src/simulation/population.rs`
- Modify: `src/simulation/mod.rs`
- Test: `src/simulation/population.rs`

**Step 1: Write the failing test**

Create `src/simulation/population.rs`:

```rust
//! Population growth system
//!
//! Entities reproduce when housing and food are available.

use crate::ecs::world::World;
use crate::simulation::resource_zone::ResourceType;
use crate::city::building::{BuildingType, BuildingState};

/// Try to grow population based on conditions
/// Returns true if new entity was spawned
pub fn try_population_growth(world: &mut World) -> bool {
    todo!()
}

/// Calculate current housing surplus (available - occupied)
pub fn housing_surplus(world: &World) -> i32 {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Vec2;

    #[test]
    fn test_housing_surplus_calculation() {
        let mut world = World::new();

        // No housing = 0 surplus
        assert_eq!(housing_surplus(&world), 0);

        // Add a house (capacity 4)
        world.spawn_building(BuildingType::House, Vec2::new(0.0, 0.0));
        world.buildings.states[0] = BuildingState::Complete;

        // 4 capacity, 0 people = 4 surplus
        assert_eq!(housing_surplus(&world), 4);

        // Add 2 people
        world.spawn_human("Alice".into());
        world.spawn_human("Bob".into());

        // Assign them housing
        let house_id = world.buildings.ids[0];
        world.humans.assigned_houses[0] = Some(house_id);
        world.humans.assigned_houses[1] = Some(house_id);

        // 4 capacity, 2 occupied = 2 surplus
        assert_eq!(housing_surplus(&world), 2);
    }

    #[test]
    fn test_population_growth_requires_housing() {
        let mut world = World::new();

        // Plenty of food, no housing
        world.stockpile.add(ResourceType::Food, 1000);
        world.spawn_human("Adam".into());

        let initial_count = world.humans.count();

        // Try growth many times - should never succeed
        for _ in 0..100 {
            try_population_growth(&mut world);
        }

        assert_eq!(world.humans.count(), initial_count,
            "Should not grow without housing surplus");
    }

    #[test]
    fn test_population_growth_requires_food() {
        let mut world = World::new();

        // Housing available, no food
        world.spawn_building(BuildingType::House, Vec2::new(0.0, 0.0));
        world.buildings.states[0] = BuildingState::Complete;
        world.spawn_human("Adam".into());

        let initial_count = world.humans.count();

        // Try growth many times - should never succeed
        for _ in 0..100 {
            try_population_growth(&mut world);
        }

        assert_eq!(world.humans.count(), initial_count,
            "Should not grow without sufficient food");
    }

    #[test]
    fn test_population_growth_when_conditions_met() {
        let mut world = World::new();

        // Setup: 1 person, plenty of housing, plenty of food
        world.spawn_building(BuildingType::House, Vec2::new(0.0, 0.0));
        world.buildings.states[0] = BuildingState::Complete;
        world.stockpile.add(ResourceType::Food, 1000);
        world.spawn_human("Adam".into());

        let initial_count = world.humans.count();

        // With 5% chance, trying 200 times should almost certainly succeed
        let mut grew = false;
        for _ in 0..200 {
            if try_population_growth(&mut world) {
                grew = true;
                break;
            }
        }

        assert!(grew, "Should have grown with conditions met");
        assert!(world.humans.count() > initial_count, "Population should have increased");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_population_growth --lib`
Expected: FAIL with "not yet implemented"

**Step 3: Write minimal implementation**

Replace the `todo!()` implementations:

```rust
/// Calculate current housing surplus (available - occupied)
pub fn housing_surplus(world: &World) -> i32 {
    let mut total_capacity: i32 = 0;

    for idx in 0..world.buildings.count() {
        if world.buildings.states[idx] != BuildingState::Complete {
            continue;
        }
        total_capacity += world.buildings.building_types[idx].housing_capacity() as i32;
    }

    let occupied = world.humans.assigned_houses.iter()
        .filter(|h| h.is_some())
        .count() as i32;

    total_capacity - occupied
}

/// Try to grow population based on conditions
/// Returns true if new entity was spawned
pub fn try_population_growth(world: &mut World) -> bool {
    // Check housing surplus
    if housing_surplus(world) <= 0 {
        return false;
    }

    // Check food surplus (need food > population * 2)
    let living_count = world.humans.iter_living().count() as u32;
    let food_available = world.stockpile.get(ResourceType::Food);
    let food_threshold = living_count * 2;

    if food_available <= food_threshold {
        return false;
    }

    // 5% chance per attempt
    let roll: f32 = rand::random();
    if roll >= 0.05 {
        return false;
    }

    // Spawn new human
    let name = format!("Newborn {}", world.current_tick);
    world.spawn_human(name);

    true
}
```

**Step 4: Add rand dependency if not present**

Check `Cargo.toml` - if `rand` is not a dependency, add it. (It likely is already for other systems.)

**Step 5: Run test to verify it passes**

Run: `cargo test test_population_growth --lib`
Expected: PASS

**Step 6: Add module to mod.rs**

Add to `src/simulation/mod.rs`:
```rust
pub mod population;
```

**Step 7: Commit**

```bash
git add src/simulation/population.rs src/simulation/mod.rs
git commit -m "feat(population): add population growth system"
```

---

## Task 8: Integrate daily systems into tick

**Files:**
- Modify: `src/simulation/tick.rs` (run_simulation_tick)
- Test: Integration test

**Step 1: Write the failing test**

Add to tick.rs tests (or create integration test):

```rust
#[test]
fn test_daily_systems_run_on_tick_100() {
    use crate::simulation::resource_zone::ResourceType;
    use crate::city::building::{BuildingType, BuildingState};
    use crate::core::types::Vec2;

    let mut world = World::new();

    // Setup: house, food, one human
    world.spawn_building(BuildingType::House, Vec2::new(0.0, 0.0));
    world.buildings.states[0] = BuildingState::Complete;
    world.stockpile.add(ResourceType::Food, 1000);
    world.spawn_human("Adam".into());

    // Human starts homeless
    assert!(world.humans.assigned_houses[0].is_none());

    // Advance to tick 100 (first daily tick)
    for _ in 0..100 {
        run_simulation_tick(&mut world);
    }

    // After daily tick:
    // - Should be housed (housing assignment ran)
    assert!(world.humans.assigned_houses[0].is_some(), "Should be housed after daily tick");

    // - Food should have been consumed
    assert!(world.stockpile.get(ResourceType::Food) < 1000, "Food should have been consumed");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_daily_systems_run_on_tick_100 --lib`
Expected: FAIL (daily systems not integrated yet)

**Step 3: Write minimal implementation**

Add imports at top of tick.rs:
```rust
use crate::simulation::housing::assign_housing;
use crate::simulation::consumption::consume_food;
use crate::simulation::population::try_population_growth;
```

Add constant:
```rust
const TICKS_PER_DAY: u64 = 100;
```

Add daily tick check in `run_simulation_tick`, after `world.tick()` but before memory decay:

```rust
    world.tick();

    // Daily systems (run once per day)
    if world.current_tick % TICKS_PER_DAY == 0 {
        assign_housing(&mut world);
        consume_food(&mut world);
        try_population_growth(&mut world);
    }

    decay_social_memories(world);
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_daily_systems_run_on_tick_100 --lib`
Expected: PASS

**Step 5: Commit**

```bash
git add src/simulation/tick.rs
git commit -m "feat(tick): integrate daily housing, food, and population systems"
```

---

## Task 9: Add iter_homeless helper to HumanArchetype

**Files:**
- Modify: `src/entity/species/human.rs`
- Test: `src/entity/species/human.rs`

**Step 1: Write the failing test**

Add to test module:

```rust
#[test]
fn test_iter_homeless() {
    use crate::city::building::BuildingId;

    let mut archetype = HumanArchetype::new();

    // Spawn 3 humans
    archetype.spawn(EntityId::new(), "Housed".into(), 0);
    archetype.spawn(EntityId::new(), "Homeless1".into(), 0);
    archetype.spawn(EntityId::new(), "Homeless2".into(), 0);

    // Assign first one to a house
    archetype.assigned_houses[0] = Some(BuildingId::new());

    let homeless: Vec<_> = archetype.iter_homeless().collect();

    assert_eq!(homeless.len(), 2);
    assert!(homeless.contains(&1));
    assert!(homeless.contains(&2));
    assert!(!homeless.contains(&0));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_iter_homeless --lib`
Expected: FAIL with "no method named `iter_homeless`"

**Step 3: Write minimal implementation**

Add to `impl HumanArchetype`:

```rust
pub fn iter_homeless(&self) -> impl Iterator<Item = usize> + '_ {
    self.alive.iter()
        .enumerate()
        .filter(|(idx, &alive)| alive && self.assigned_houses[*idx].is_none())
        .map(|(i, _)| i)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_iter_homeless --lib`
Expected: PASS

**Step 5: Commit**

```bash
git add src/entity/species/human.rs
git commit -m "feat(human): add iter_homeless helper method"
```

---

## Task 10: Integration test for full population cycle

**Files:**
- Create: `tests/population_integration.rs`

**Step 1: Write the integration test**

Create `tests/population_integration.rs`:

```rust
//! Integration tests for population and housing system

use arc_citadel::ecs::world::World;
use arc_citadel::city::building::{BuildingType, BuildingState};
use arc_citadel::simulation::tick::run_simulation_tick;
use arc_citadel::simulation::resource_zone::ResourceType;
use arc_citadel::core::types::Vec2;

#[test]
fn test_population_lifecycle() {
    let mut world = World::new();

    // Setup initial settlement
    // 2 houses = 8 capacity
    world.spawn_building(BuildingType::House, Vec2::new(0.0, 0.0));
    world.spawn_building(BuildingType::House, Vec2::new(10.0, 0.0));
    world.buildings.states[0] = BuildingState::Complete;
    world.buildings.states[1] = BuildingState::Complete;

    // Abundant food
    world.stockpile.add(ResourceType::Food, 10000);

    // Start with 2 humans
    world.spawn_human("Adam".into());
    world.spawn_human("Eve".into());

    let initial_pop = world.humans.count();
    assert_eq!(initial_pop, 2);

    // Run for several in-game days
    for _ in 0..1000 {
        run_simulation_tick(&mut world);
    }

    // Population should have grown
    let final_pop = world.humans.count();
    assert!(final_pop > initial_pop,
        "Population should grow from {} to more", initial_pop);

    // All should be housed (capacity 8, started with 2)
    let homeless: usize = world.humans.iter_homeless().count();
    if final_pop <= 8 {
        assert_eq!(homeless, 0, "All {} should be housed with capacity 8", final_pop);
    }

    // Food should have been consumed
    let food_remaining = world.stockpile.get(ResourceType::Food);
    assert!(food_remaining < 10000, "Food should have been consumed");
}

#[test]
fn test_homelessness_stress() {
    let mut world = World::new();

    // 1 house = 4 capacity
    world.spawn_building(BuildingType::House, Vec2::new(0.0, 0.0));
    world.buildings.states[0] = BuildingState::Complete;

    // Start with 6 humans (exceeds capacity)
    for i in 0..6 {
        world.spawn_human(format!("Human {}", i));
    }

    // Some food
    world.stockpile.add(ResourceType::Food, 100);

    // Run one day
    for _ in 0..100 {
        run_simulation_tick(&mut world);
    }

    // Should have 4 housed, 2 homeless
    let housed = world.humans.assigned_houses.iter()
        .filter(|h| h.is_some())
        .count();
    let homeless = world.humans.iter_homeless().count();

    assert_eq!(housed, 4, "Should have 4 housed");
    assert_eq!(homeless, 2, "Should have 2 homeless");

    // Homeless should have higher needs than housed (due to 1.5x decay)
    // We'd need to track individual decay for precise testing
}
```

**Step 2: Run test**

Run: `cargo test --test population_integration`
Expected: PASS

**Step 3: Commit**

```bash
git add tests/population_integration.rs
git commit -m "test: add population lifecycle integration tests"
```

---

## Summary

| Task | Component | Key Change |
|------|-----------|------------|
| 1 | BuildingType | `housing_capacity()` method |
| 2 | HumanArchetype | `assigned_houses` field |
| 3 | World | `stockpile` field |
| 4 | tick.rs | Homeless 1.5x decay multiplier |
| 5 | housing.rs | Housing assignment system |
| 6 | consumption.rs | Food consumption system |
| 7 | population.rs | Population growth system |
| 8 | tick.rs | Daily system integration |
| 9 | HumanArchetype | `iter_homeless()` helper |
| 10 | Integration | Full lifecycle test |

**Total: 10 tasks, ~30-45 minutes estimated**
