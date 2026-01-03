//! Population growth system
//!
//! Entities reproduce when housing and food are available.

use crate::ecs::world::World;
use crate::simulation::resource_zone::ResourceType;
use crate::city::building::BuildingState;
use rand::Rng;

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
        .filter(|h: &&Option<crate::city::BuildingId>| h.is_some())
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
    let mut rng = rand::thread_rng();
    let roll: f32 = rng.gen();
    if roll >= 0.05 {
        return false;
    }

    // Spawn new human
    let name = format!("Newborn {}", world.current_tick);
    world.spawn_human(name);

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Vec2;
    use crate::city::building::BuildingType;

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
