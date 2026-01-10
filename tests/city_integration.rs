//! Integration tests for city layer
//!
//! These tests verify the complete city foundation and production pipeline:
//! - Building construction workflow (spawn -> assign task -> build -> complete)
//! - Production workflow (complete building -> start recipe -> produce outputs)
//! - End-to-end simulation of city building and production
//!
//! The city layer enables:
//! - Construction of buildings via worker-tick contributions
//! - Production recipes that transform inputs into outputs
//! - Stockpile management for settlement resources

use arc_citadel::actions::catalog::ActionId;
use arc_citadel::city::building::{BuildingState, BuildingType};
use arc_citadel::city::production::tick_production;
use arc_citadel::city::recipe::RecipeCatalog;
use arc_citadel::city::stockpile::Stockpile;
use arc_citadel::core::types::Vec2;
use arc_citadel::ecs::world::World;
use arc_citadel::entity::tasks::{Task, TaskPriority};
use arc_citadel::simulation::resource_zone::ResourceType;
use arc_citadel::simulation::tick::run_simulation_tick;

// ============================================================================
// Construction Workflow Integration Tests
// ============================================================================

/// Integration test: Complete construction workflow
///
/// This test verifies the full building construction pipeline:
/// 1. Spawn human with building skill
/// 2. Create building construction site
/// 3. Assign build task to human
/// 4. Run ticks until building completes
/// 5. Verify building state is Complete
/// 6. Verify worker skill improved
#[test]
fn test_complete_construction_workflow() {
    let mut world = World::new();

    // Spawn builder with high skill
    let builder = world.spawn_human("Mason".into());
    let idx = world.humans.index_of(builder).unwrap();
    world.humans.building_skills[idx] = 0.8;
    world.humans.positions[idx] = Vec2::new(50.0, 50.0);

    // Spawn a wall to build (80 work required)
    let building_id = world.spawn_building(BuildingType::Wall, Vec2::new(50.0, 50.0));

    // Assign build task
    let task = Task::new(ActionId::Build, TaskPriority::Normal, 0).with_building(building_id);
    world.humans.task_queues[idx].push(task);

    // Track initial skill
    let initial_skill = world.humans.building_skills[idx];

    // Run until building complete (max 200 ticks)
    let mut completed = false;
    for tick in 0..200 {
        run_simulation_tick(&mut world);

        let building_idx = world.buildings.index_of(building_id).unwrap();
        if world.buildings.states[building_idx] == BuildingState::Complete {
            completed = true;
            println!("Building completed at tick {}", tick);
            break;
        }
    }

    assert!(completed, "Building should complete within 200 ticks");

    // Skill should have improved
    let final_skill = world.humans.building_skills[idx];
    assert!(
        final_skill > initial_skill,
        "Skill should improve after completing building. Initial: {}, Final: {}",
        initial_skill,
        final_skill
    );

    // Building should be in Complete state
    let building_idx = world.buildings.index_of(building_id).unwrap();
    assert_eq!(
        world.buildings.states[building_idx],
        BuildingState::Complete,
        "Building state should be Complete"
    );
}

/// Integration test: Multiple builders work faster
///
/// This test verifies that having multiple workers on a building
/// results in faster construction time due to combined contributions.
#[test]
fn test_multiple_builders_faster() {
    let mut world1 = World::new();
    let mut world2 = World::new();

    // World 1: Single builder
    let builder1 = world1.spawn_human("Solo".into());
    let idx1 = world1.humans.index_of(builder1).unwrap();
    world1.humans.building_skills[idx1] = 0.5;
    world1.humans.positions[idx1] = Vec2::new(50.0, 50.0);
    let building1 = world1.spawn_building(BuildingType::House, Vec2::new(50.0, 50.0));
    world1.humans.task_queues[idx1]
        .push(Task::new(ActionId::Build, TaskPriority::Normal, 0).with_building(building1));

    // World 2: Two builders
    let builder2a = world2.spawn_human("Duo1".into());
    let builder2b = world2.spawn_human("Duo2".into());
    let idx2a = world2.humans.index_of(builder2a).unwrap();
    let idx2b = world2.humans.index_of(builder2b).unwrap();
    world2.humans.building_skills[idx2a] = 0.5;
    world2.humans.building_skills[idx2b] = 0.5;
    world2.humans.positions[idx2a] = Vec2::new(50.0, 50.0);
    world2.humans.positions[idx2b] = Vec2::new(50.0, 50.0);
    let building2 = world2.spawn_building(BuildingType::House, Vec2::new(50.0, 50.0));
    world2.humans.task_queues[idx2a]
        .push(Task::new(ActionId::Build, TaskPriority::Normal, 0).with_building(building2));
    world2.humans.task_queues[idx2b]
        .push(Task::new(ActionId::Build, TaskPriority::Normal, 0).with_building(building2));

    // Run both until complete
    let mut ticks1 = 0;
    let mut ticks2 = 0;

    for tick in 0..500 {
        if world1.buildings.states[0] != BuildingState::Complete {
            run_simulation_tick(&mut world1);
            ticks1 = tick + 1;
        }
        if world2.buildings.states[0] != BuildingState::Complete {
            run_simulation_tick(&mut world2);
            ticks2 = tick + 1;
        }
        if world1.buildings.states[0] == BuildingState::Complete
            && world2.buildings.states[0] == BuildingState::Complete
        {
            break;
        }
    }

    println!(
        "Single builder: {} ticks, Two builders: {} ticks",
        ticks1, ticks2
    );
    assert!(
        ticks2 < ticks1,
        "Two builders should complete faster than one. Single: {} ticks, Two: {} ticks",
        ticks1,
        ticks2
    );
}

/// Integration test: Building construction progress tracking
///
/// This test verifies that construction progress is correctly tracked
/// and visible on the building archetype.
#[test]
fn test_construction_progress_tracking() {
    let mut world = World::new();

    // Spawn builder with max skill
    let builder = world.spawn_human("Builder".into());
    let idx = world.humans.index_of(builder).unwrap();
    world.humans.building_skills[idx] = 1.0; // Max skill = 1.0 contribution per tick
    world.humans.positions[idx] = Vec2::new(50.0, 50.0);

    // Spawn a house (100 work required)
    let building_id = world.spawn_building(BuildingType::House, Vec2::new(50.0, 50.0));

    // Assign build task
    let task = Task::new(ActionId::Build, TaskPriority::Normal, 0).with_building(building_id);
    world.humans.task_queues[idx].push(task);

    // Run 50 ticks
    for _ in 0..50 {
        run_simulation_tick(&mut world);
    }

    // Check construction progress
    let building_idx = world.buildings.index_of(building_id).unwrap();
    let progress = world.buildings.construction_progress[building_idx];
    let work_required = BuildingType::House.work_required();

    // With skill 1.0 and no fatigue, contribution is 1.0 per tick
    // After 50 ticks, should have ~50 work done on 100 work building
    assert!(
        progress >= 40.0,
        "Should have significant progress. Expected ~50, got {}",
        progress
    );
    assert!(
        progress < work_required,
        "Should not be complete yet. Progress: {}, Required: {}",
        progress,
        work_required
    );

    // Building should still be under construction
    assert_eq!(
        world.buildings.states[building_idx],
        BuildingState::UnderConstruction,
        "Building should still be under construction"
    );
}

// ============================================================================
// Production Workflow Integration Tests
// ============================================================================

/// Integration test: Complete production workflow
///
/// This test verifies the full production pipeline:
/// 1. Create a completed farm building
/// 2. Start production with a recipe (farm_food)
/// 3. Assign workers to production
/// 4. Run ticks until production completes
/// 5. Verify output resources appear in stockpile
#[test]
fn test_complete_production_workflow() {
    // Setup: Create a completed farm
    let mut world = World::new();
    let farm_id = world.spawn_building(BuildingType::Farm, Vec2::new(0.0, 0.0));
    let farm_idx = world.buildings.index_of(farm_id).unwrap();

    // Mark farm as complete
    world.buildings.states[farm_idx] = BuildingState::Complete;

    // Start production
    assert!(
        world
            .buildings
            .start_production(farm_idx, "farm_food".into()),
        "Should be able to start production on completed building"
    );

    // Assign workers
    world.buildings.production_workers[farm_idx] = 2; // Full worker count for farm_food recipe

    // Setup stockpile and recipe catalog
    let recipes = RecipeCatalog::with_defaults();
    let mut stockpile = Stockpile::new();
    stockpile.set_capacity(ResourceType::Food, 1000);

    // Verify initial state
    assert_eq!(
        stockpile.get(ResourceType::Food),
        0,
        "Should start with 0 food"
    );

    // Run production ticks
    // farm_food: work_required=100, workers_needed=2
    // With 2 workers: rate = 1.0, progress per tick = 1.0/100 = 0.01
    let mut total_cycles = 0;
    for _ in 0..150 {
        let results = tick_production(&mut world.buildings, &recipes, &mut stockpile);
        total_cycles += results.len();
    }

    // Should have completed at least one production cycle
    assert!(
        total_cycles >= 1,
        "Should complete at least 1 production cycle in 150 ticks, got {}",
        total_cycles
    );

    // Should have produced food
    let food_produced = stockpile.get(ResourceType::Food);
    assert!(
        food_produced >= 5,
        "Should have produced at least 5 food (one cycle output), got {}",
        food_produced
    );
}

/// Integration test: Production consumes inputs
///
/// This test verifies that production recipes correctly consume
/// input resources from the stockpile.
#[test]
fn test_production_consumes_inputs() {
    // Setup: Create a completed workshop
    let mut world = World::new();
    let workshop_id = world.spawn_building(BuildingType::Workshop, Vec2::new(0.0, 0.0));
    let workshop_idx = world.buildings.index_of(workshop_id).unwrap();

    // Mark workshop as complete
    world.buildings.states[workshop_idx] = BuildingState::Complete;

    // Start iron smelting production (needs 3 ore per cycle)
    assert!(world
        .buildings
        .start_production(workshop_idx, "smelt_iron".into()));
    world.buildings.production_workers[workshop_idx] = 1;

    // Setup stockpile with ore
    let recipes = RecipeCatalog::with_defaults();
    let mut stockpile = Stockpile::new();
    stockpile.set_capacity(ResourceType::Ore, 100);
    stockpile.set_capacity(ResourceType::Iron, 100);
    stockpile.add(ResourceType::Ore, 6); // Enough for exactly 2 cycles (3 ore each)

    // Run production
    // smelt_iron: work_required=50, workers_needed=1
    // With 1 worker: rate = 1.0, progress = 1.0/50 = 0.02 per tick
    // 50 ticks per cycle, 102 ticks for 2 cycles
    let mut total_cycles = 0;
    for _ in 0..102 {
        let results = tick_production(&mut world.buildings, &recipes, &mut stockpile);
        total_cycles += results.len();
    }

    // Should have completed exactly 2 cycles
    assert_eq!(
        total_cycles, 2,
        "Should complete exactly 2 production cycles"
    );

    // Should have consumed all ore
    assert_eq!(
        stockpile.get(ResourceType::Ore),
        0,
        "Should have consumed all ore"
    );

    // Should have produced 2 iron
    assert_eq!(
        stockpile.get(ResourceType::Iron),
        2,
        "Should have produced 2 iron"
    );
}

/// Integration test: Production requires materials
///
/// This test verifies that production halts when required input
/// materials are not available in the stockpile.
#[test]
fn test_production_requires_materials() {
    // Setup: Create a completed workshop
    let mut world = World::new();
    let workshop_id = world.spawn_building(BuildingType::Workshop, Vec2::new(0.0, 0.0));
    let workshop_idx = world.buildings.index_of(workshop_id).unwrap();
    world.buildings.states[workshop_idx] = BuildingState::Complete;

    // Start iron smelting (needs ore)
    world
        .buildings
        .start_production(workshop_idx, "smelt_iron".into());
    world.buildings.production_workers[workshop_idx] = 1;

    // Setup stockpile WITHOUT ore
    let recipes = RecipeCatalog::with_defaults();
    let mut stockpile = Stockpile::new();
    stockpile.set_capacity(ResourceType::Iron, 100);
    // Note: No ore added

    // Run production
    let mut total_cycles = 0;
    for _ in 0..100 {
        let results = tick_production(&mut world.buildings, &recipes, &mut stockpile);
        total_cycles += results.len();
    }

    // Should not complete any cycles (no ore available)
    assert_eq!(
        total_cycles, 0,
        "Should not complete any cycles without required inputs"
    );

    // Should not have produced any iron
    assert_eq!(
        stockpile.get(ResourceType::Iron),
        0,
        "Should not produce iron without ore"
    );
}

// ============================================================================
// End-to-End Integration Tests
// ============================================================================

/// Integration test: Full city workflow - build then produce
///
/// This is the complete end-to-end test that verifies:
/// 1. Spawn human with high purpose need (drives work-seeking)
/// 2. Create building construction site
/// 3. Run ticks until building completes
/// 4. Verify building state is Complete
/// 5. Start production on the completed building
/// 6. Add resources to stockpile
/// 7. Run ticks to produce outputs
/// 8. Verify production output in stockpile
#[test]
fn test_full_city_workflow_build_then_produce() {
    let mut world = World::new();

    // ===== PHASE 1: Construction =====

    // Spawn builder with high purpose need and good skill
    let builder = world.spawn_human("CityBuilder".into());
    let idx = world.humans.index_of(builder).unwrap();
    world.humans.building_skills[idx] = 0.9; // High skill
    world.humans.needs[idx].purpose = 0.8; // High purpose need
    world.humans.positions[idx] = Vec2::new(50.0, 50.0);

    // Create farm construction site
    let farm_id = world.spawn_building(BuildingType::Farm, Vec2::new(50.0, 50.0));
    let farm_idx = world.buildings.index_of(farm_id).unwrap();

    // Verify initial state
    assert_eq!(
        world.buildings.states[farm_idx],
        BuildingState::UnderConstruction
    );

    // Assign build task
    let task = Task::new(ActionId::Build, TaskPriority::Normal, 0).with_building(farm_id);
    world.humans.task_queues[idx].push(task);

    // Run until farm is complete
    let mut construction_complete = false;
    for tick in 0..300 {
        run_simulation_tick(&mut world);

        if world.buildings.states[farm_idx] == BuildingState::Complete {
            construction_complete = true;
            println!("Farm construction completed at tick {}", tick);
            break;
        }
    }

    assert!(
        construction_complete,
        "Farm should complete construction within 300 ticks"
    );
    assert_eq!(
        world.buildings.states[farm_idx],
        BuildingState::Complete,
        "Farm should be in Complete state"
    );

    // ===== PHASE 2: Production =====

    // Start production on the completed farm
    assert!(
        world
            .buildings
            .start_production(farm_idx, "farm_food".into()),
        "Should be able to start production on completed farm"
    );
    world.buildings.production_workers[farm_idx] = 2;

    // Setup stockpile and recipe catalog
    let recipes = RecipeCatalog::with_defaults();
    let mut stockpile = Stockpile::new();
    stockpile.set_capacity(ResourceType::Food, 1000);

    // Verify no food initially
    assert_eq!(stockpile.get(ResourceType::Food), 0);

    // Run production ticks
    let mut production_cycles = 0;
    for _ in 0..150 {
        let results = tick_production(&mut world.buildings, &recipes, &mut stockpile);
        production_cycles += results.len();
    }

    // Verify production completed
    assert!(
        production_cycles >= 1,
        "Should complete at least 1 production cycle"
    );

    // Verify output in stockpile
    let food_produced = stockpile.get(ResourceType::Food);
    assert!(
        food_produced >= 5,
        "Should have produced at least 5 food, got {}",
        food_produced
    );

    println!(
        "Full workflow complete: Built farm, produced {} food in {} cycles",
        food_produced, production_cycles
    );
}

/// Integration test: Workshop iron smelting chain
///
/// This test verifies a more complex production chain:
/// 1. Build a workshop
/// 2. Start iron smelting production
/// 3. Provide ore inputs
/// 4. Verify iron outputs
#[test]
fn test_workshop_iron_smelting_chain() {
    let mut world = World::new();

    // Spawn builder
    let builder = world.spawn_human("Smith".into());
    let idx = world.humans.index_of(builder).unwrap();
    world.humans.building_skills[idx] = 1.0;
    world.humans.positions[idx] = Vec2::new(0.0, 0.0);

    // Create workshop (200 work required - more than wall or house)
    let workshop_id = world.spawn_building(BuildingType::Workshop, Vec2::new(0.0, 0.0));

    // Build the workshop
    let task = Task::new(ActionId::Build, TaskPriority::Normal, 0).with_building(workshop_id);
    world.humans.task_queues[idx].push(task);

    // Run construction
    for _ in 0..300 {
        run_simulation_tick(&mut world);
        let workshop_idx = world.buildings.index_of(workshop_id).unwrap();
        if world.buildings.states[workshop_idx] == BuildingState::Complete {
            break;
        }
    }

    let workshop_idx = world.buildings.index_of(workshop_id).unwrap();
    assert_eq!(
        world.buildings.states[workshop_idx],
        BuildingState::Complete,
        "Workshop should be complete"
    );

    // Start iron smelting
    world
        .buildings
        .start_production(workshop_idx, "smelt_iron".into());
    world.buildings.production_workers[workshop_idx] = 2; // Extra workers for bonus rate

    // Setup stockpile with ore
    let recipes = RecipeCatalog::with_defaults();
    let mut stockpile = Stockpile::new();
    stockpile.set_capacity(ResourceType::Ore, 100);
    stockpile.set_capacity(ResourceType::Iron, 100);
    stockpile.add(ResourceType::Ore, 15); // Enough for 5 cycles

    // Run production
    let mut cycles = 0;
    for _ in 0..300 {
        let results = tick_production(&mut world.buildings, &recipes, &mut stockpile);
        cycles += results.len();
    }

    // Verify iron production
    let iron_produced = stockpile.get(ResourceType::Iron);
    assert!(
        iron_produced >= 3,
        "Should have produced at least 3 iron, got {}",
        iron_produced
    );

    // Verify ore consumed
    let ore_remaining = stockpile.get(ResourceType::Ore);
    assert!(
        ore_remaining < 15,
        "Should have consumed some ore. Started with 15, now have {}",
        ore_remaining
    );

    println!(
        "Workshop chain complete: {} iron produced, {} ore remaining, {} cycles",
        iron_produced, ore_remaining, cycles
    );
}

/// Integration test: Building cannot produce while under construction
///
/// This test verifies that production cannot start on buildings
/// that are still under construction.
#[test]
fn test_cannot_produce_while_under_construction() {
    let mut world = World::new();

    // Create farm but don't complete it
    let farm_id = world.spawn_building(BuildingType::Farm, Vec2::new(0.0, 0.0));
    let farm_idx = world.buildings.index_of(farm_id).unwrap();

    // Verify it's under construction
    assert_eq!(
        world.buildings.states[farm_idx],
        BuildingState::UnderConstruction
    );

    // Try to start production - should fail
    let result = world
        .buildings
        .start_production(farm_idx, "farm_food".into());
    assert!(
        !result,
        "Should not be able to start production on under-construction building"
    );

    // Recipe should not be set
    assert_eq!(
        world.buildings.active_recipes[farm_idx], None,
        "Recipe should not be set on under-construction building"
    );
}

// ============================================================================
// Stockpile Integration Tests
// ============================================================================

/// Integration test: Stockpile resource management
///
/// This test verifies stockpile add/remove operations work correctly
/// in an integrated context.
#[test]
fn test_stockpile_integration() {
    let mut stockpile = Stockpile::new();

    // Set capacities
    stockpile.set_capacity(ResourceType::Wood, 100);
    stockpile.set_capacity(ResourceType::Stone, 50);
    stockpile.set_capacity(ResourceType::Iron, 25);

    // Add resources
    assert_eq!(stockpile.add(ResourceType::Wood, 80), 80);
    assert_eq!(stockpile.add(ResourceType::Stone, 30), 30);
    assert_eq!(stockpile.add(ResourceType::Iron, 10), 10);

    // Check amounts
    assert_eq!(stockpile.get(ResourceType::Wood), 80);
    assert_eq!(stockpile.get(ResourceType::Stone), 30);
    assert_eq!(stockpile.get(ResourceType::Iron), 10);

    // Try to add beyond capacity
    assert_eq!(stockpile.add(ResourceType::Wood, 50), 20); // Only 20 space left
    assert_eq!(stockpile.get(ResourceType::Wood), 100); // At capacity

    // Check material requirements
    let house_materials = BuildingType::House.required_materials();
    // House needs 20 Wood + 10 Stone
    assert!(
        stockpile.has_materials(&house_materials),
        "Should have materials for house"
    );

    // Consume materials
    assert!(stockpile.consume_materials(&house_materials));
    assert_eq!(stockpile.get(ResourceType::Wood), 80); // 100 - 20
    assert_eq!(stockpile.get(ResourceType::Stone), 20); // 30 - 10
}

/// Integration test: Multiple production buildings
///
/// This test verifies that multiple buildings can produce simultaneously.
#[test]
fn test_multiple_production_buildings() {
    let mut world = World::new();

    // Create and complete two farms
    let farm1_id = world.spawn_building(BuildingType::Farm, Vec2::new(0.0, 0.0));
    let farm2_id = world.spawn_building(BuildingType::Farm, Vec2::new(50.0, 0.0));
    let farm1_idx = world.buildings.index_of(farm1_id).unwrap();
    let farm2_idx = world.buildings.index_of(farm2_id).unwrap();

    // Mark both as complete
    world.buildings.states[farm1_idx] = BuildingState::Complete;
    world.buildings.states[farm2_idx] = BuildingState::Complete;

    // Start production on both
    world
        .buildings
        .start_production(farm1_idx, "farm_food".into());
    world
        .buildings
        .start_production(farm2_idx, "farm_food".into());
    world.buildings.production_workers[farm1_idx] = 2;
    world.buildings.production_workers[farm2_idx] = 2;

    // Setup
    let recipes = RecipeCatalog::with_defaults();
    let mut stockpile = Stockpile::new();
    stockpile.set_capacity(ResourceType::Food, 1000);

    // Run production
    let mut total_cycles = 0;
    for _ in 0..150 {
        let results = tick_production(&mut world.buildings, &recipes, &mut stockpile);
        total_cycles += results.len();
    }

    // Should have more cycles than single building
    assert!(
        total_cycles >= 2,
        "Should complete at least 2 cycles with 2 farms, got {}",
        total_cycles
    );

    // Should have more food
    let food = stockpile.get(ResourceType::Food);
    assert!(
        food >= 10,
        "Should produce at least 10 food from 2 farms, got {}",
        food
    );
}
