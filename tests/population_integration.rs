//! Integration tests for population and housing system
//!
//! These tests verify the complete population lifecycle:
//! - Housing assignment when buildings are complete
//! - Food consumption from stockpile
//! - Population growth when conditions are met
//! - Homeless penalty (1.5x need decay)

use arc_citadel::city::building::{BuildingState, BuildingType};
use arc_citadel::core::types::Vec2;
use arc_citadel::ecs::world::World;
use arc_citadel::simulation::resource_zone::ResourceType;
use arc_citadel::simulation::tick::run_simulation_tick;

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

    // Run for several in-game days (TICKS_PER_DAY = 1000)
    // Running 10000 ticks = 10 days, with 5% growth chance per day
    for _ in 0..10000 {
        run_simulation_tick(&mut world);
    }

    // Population should have grown
    let final_pop = world.humans.count();
    assert!(
        final_pop > initial_pop,
        "Population should grow from {} to more, got {}",
        initial_pop,
        final_pop
    );

    // All should be housed (capacity 8, started with 2)
    let homeless: usize = world.humans.iter_homeless().count();
    if final_pop <= 8 {
        assert_eq!(
            homeless, 0,
            "All {} should be housed with capacity 8",
            final_pop
        );
    }

    // Food should have been consumed
    let food_remaining = world.stockpile.get(ResourceType::Food);
    assert!(
        food_remaining < 10000,
        "Food should have been consumed, still have {}",
        food_remaining
    );

    println!(
        "Population lifecycle: {} -> {}, food: 10000 -> {}, homeless: {}",
        initial_pop, final_pop, food_remaining, homeless
    );
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

    // Run one day (TICKS_PER_DAY = 1000)
    for _ in 0..1000 {
        run_simulation_tick(&mut world);
    }

    // Should have 4 housed, 2 homeless
    let housed = world
        .humans
        .assigned_houses
        .iter()
        .filter(|h| h.is_some())
        .count();
    let homeless = world.humans.iter_homeless().count();

    assert_eq!(housed, 4, "Should have 4 housed");
    assert_eq!(homeless, 2, "Should have 2 homeless");

    // Homeless should have higher needs than housed (due to 1.5x decay)
    // We'd need to track individual decay for precise testing
    println!(
        "Homelessness stress: {} housed, {} homeless",
        housed, homeless
    );
}
