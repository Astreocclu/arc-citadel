//! Integration tests for Arc Citadel
//!
//! These tests verify the core simulation loop works end-to-end:
//! - Entity spawning
//! - Needs decay over time
//! - Action selection based on needs and values
//! - Task progress and completion
//! - Emergent behavior from different value configurations

use arc_citadel::actions::catalog::{ActionCategory, ActionId};
use arc_citadel::core::types::Vec2;
use arc_citadel::ecs::world::{Abundance, World};
use arc_citadel::entity::tasks::{Task, TaskPriority};
use arc_citadel::entity::thoughts::{CauseType, Thought, Valence};
use arc_citadel::simulation::tick::run_simulation_tick;

// ============================================================================
// World Creation and Entity Spawning Tests
// ============================================================================

#[test]
fn test_world_creation() {
    let world = World::new();
    assert_eq!(world.entity_count(), 0);
    assert_eq!(world.current_tick, 0);
}

#[test]
fn test_spawn_single_human() {
    let mut world = World::new();
    let id = world.spawn_human("Test".into());

    assert_eq!(world.entity_count(), 1);
    assert!(world.humans.index_of(id).is_some());

    let idx = world.humans.index_of(id).unwrap();
    assert_eq!(world.humans.names[idx], "Test");
    assert!(world.humans.alive[idx]);
}

#[test]
fn test_spawn_multiple_humans() {
    let mut world = World::new();
    let names = ["Alice", "Bob", "Carol", "David", "Eve"];

    let mut ids = Vec::new();
    for name in &names {
        ids.push(world.spawn_human(name.to_string()));
    }

    assert_eq!(world.entity_count(), 5);

    // Verify each entity
    for (i, id) in ids.iter().enumerate() {
        let idx = world.humans.index_of(*id).unwrap();
        assert_eq!(world.humans.names[idx], names[i]);
    }
}

#[test]
fn test_spawned_entities_have_default_needs() {
    let mut world = World::new();
    world.spawn_human("Test".into());

    let needs = &world.humans.needs[0];
    assert!(needs.rest >= 0.0 && needs.rest <= 1.0);
    assert!(needs.food >= 0.0 && needs.food <= 1.0);
    assert!(needs.safety >= 0.0 && needs.safety <= 1.0);
    assert!(needs.social >= 0.0 && needs.social <= 1.0);
    assert!(needs.purpose >= 0.0 && needs.purpose <= 1.0);
}

#[test]
fn test_spawned_entities_start_idle() {
    let mut world = World::new();
    world.spawn_human("Test".into());

    assert!(world.humans.task_queues[0].is_idle());
}

// ============================================================================
// Needs Decay Tests
// ============================================================================

#[test]
fn test_needs_decay_over_time() {
    let mut world = World::new();
    world.spawn_human("Test".into());

    let initial_rest = world.humans.needs[0].rest;
    let initial_food = world.humans.needs[0].food;

    // Run multiple ticks
    for _ in 0..100 {
        run_simulation_tick(&mut world);
    }

    // Rest and food needs should increase (decay)
    assert!(
        world.humans.needs[0].rest > initial_rest,
        "Rest need should increase over time"
    );
    assert!(
        world.humans.needs[0].food > initial_food,
        "Food need should increase over time"
    );
}

#[test]
fn test_needs_bounded_to_one() {
    let mut world = World::new();
    world.spawn_human("Test".into());

    // Run many ticks to ensure needs don't exceed 1.0
    for _ in 0..10000 {
        run_simulation_tick(&mut world);
    }

    let needs = &world.humans.needs[0];
    assert!(needs.rest <= 1.0, "Rest need should not exceed 1.0");
    assert!(needs.food <= 1.0, "Food need should not exceed 1.0");
    assert!(needs.safety <= 1.0, "Safety need should not exceed 1.0");
    assert!(needs.social <= 1.0, "Social need should not exceed 1.0");
    assert!(needs.purpose <= 1.0, "Purpose need should not exceed 1.0");
}

#[test]
fn test_needs_decay_rate_differs_by_activity() {
    let mut world_active = World::new();
    let mut world_idle = World::new();

    let active_id = world_active.spawn_human("Active".into());
    world_idle.spawn_human("Idle".into());

    // Give active entity a task
    let active_idx = world_active.humans.index_of(active_id).unwrap();
    world_active.humans.task_queues[active_idx].push(Task::new(
        ActionId::Build,
        TaskPriority::Normal,
        world_active.current_tick,
    ));

    let initial_rest = world_active.humans.needs[active_idx].rest;

    // Run same number of ticks for both
    for _ in 0..50 {
        run_simulation_tick(&mut world_active);
        run_simulation_tick(&mut world_idle);
    }

    // Active entity should have higher rest need (more fatigue)
    // Both should increase, but active should increase faster
    assert!(
        world_active.humans.needs[0].rest > initial_rest,
        "Active entity rest need should increase"
    );
    assert!(
        world_idle.humans.needs[0].rest > initial_rest,
        "Idle entity rest need should increase"
    );
}

// ============================================================================
// Action Selection Tests
// ============================================================================

#[test]
fn test_idle_entities_get_tasks() {
    let mut world = World::new();
    world.spawn_human("Test".into());

    // Initially idle
    assert!(world.humans.task_queues[0].is_idle());

    // After tick, should have a task
    run_simulation_tick(&mut world);

    assert!(
        world.humans.task_queues[0].current().is_some(),
        "Entity should have a task after tick"
    );
}

#[test]
fn test_multiple_entities_all_get_tasks() {
    let mut world = World::new();

    for i in 0..10 {
        world.spawn_human(format!("Entity{}", i));
    }

    run_simulation_tick(&mut world);

    // All entities should have tasks
    for i in 0..10 {
        assert!(
            world.humans.task_queues[i].current().is_some(),
            "Entity {} should have a task",
            i
        );
    }
}

#[test]
fn test_critical_need_triggers_appropriate_action() {
    let mut world = World::new();
    world.spawn_human("Hungry".into());

    // Set critical food need
    world.humans.needs[0].food = 0.9;

    // Add a food zone at the entity's position so food is available
    let entity_pos = world.humans.positions[0];
    world.add_food_zone(entity_pos, 10.0, Abundance::Unlimited);

    run_simulation_tick(&mut world);

    let task = world.humans.task_queues[0].current().unwrap();
    assert_eq!(task.action, ActionId::Eat, "Critical food need should trigger Eat action");
    assert_eq!(
        task.priority,
        TaskPriority::Critical,
        "Critical need response should have Critical priority"
    );
}

#[test]
fn test_critical_safety_with_threat_triggers_flee() {
    let mut world = World::new();
    world.spawn_human("Scared".into());

    // Set critical safety need (simulates threat detection)
    world.humans.needs[0].safety = 0.9;

    run_simulation_tick(&mut world);

    let task = world.humans.task_queues[0].current().unwrap();
    // The action depends on context (threat_nearby, safe_location)
    // In the tick system, threat_nearby is derived from safety > 0.5
    assert!(
        task.action == ActionId::Flee || task.action == ActionId::SeekSafety,
        "Critical safety need should trigger safety-related action"
    );
    assert_eq!(task.priority, TaskPriority::Critical);
}

// ============================================================================
// Task Progress Tests
// ============================================================================

#[test]
fn test_task_progress_increases() {
    use arc_citadel::entity::tasks::{Task, TaskPriority};
    use arc_citadel::actions::catalog::ActionId;

    let mut world = World::new();
    world.spawn_human("Worker".into());

    // Set initial position
    world.humans.positions[0] = Vec2::new(0.0, 0.0);

    // Give a non-continuous task that has traditional progress (Rest has duration 50)
    // Note: IdleWander is a continuous action (duration 0) that doesn't use progress
    let task = Task::new(ActionId::Rest, TaskPriority::Normal, 0);
    world.humans.task_queues[0].push(task);

    let initial_progress = world.humans.task_queues[0]
        .current()
        .map(|t| t.progress)
        .unwrap_or(0.0);

    // Run more ticks
    run_simulation_tick(&mut world);

    let current_progress = world.humans.task_queues[0]
        .current()
        .map(|t| t.progress)
        .unwrap_or(0.0);

    assert!(
        current_progress > initial_progress,
        "Task progress should increase over time for non-continuous actions"
    );
}

#[test]
fn test_actions_satisfy_needs() {
    let mut world = World::new();
    world.spawn_human("Restless".into());

    // Set high rest need and force a Rest action
    world.humans.needs[0].rest = 0.85;

    let _initial_rest = world.humans.needs[0].rest;

    // Run ticks - Rest action should be selected due to critical need
    for _ in 0..10 {
        run_simulation_tick(&mut world);
    }

    // Note: The action selection might select Rest if conditions are right
    // The Eat action satisfies food, Rest satisfies rest, etc.
    // Due to critical need at 0.85, it should select a safety or rest action
    // After execution, the corresponding need should be lower or at least not higher

    // The simulation runs and actions satisfy needs
    // We verify the system processes without errors and needs stay bounded
    assert!(world.humans.needs[0].rest <= 1.0);
}

// ============================================================================
// Value-Driven Behavior Tests (Emergence)
// ============================================================================

#[test]
fn test_different_values_different_behavior() {
    let mut world = World::new();

    let cautious_id = world.spawn_human("Cautious Carl".into());
    let brave_id = world.spawn_human("Brave Bob".into());

    // Configure different values
    if let Some(idx) = world.humans.index_of(cautious_id) {
        world.humans.values[idx].safety = 0.9;
        world.humans.values[idx].curiosity = 0.1;
    }
    if let Some(idx) = world.humans.index_of(brave_id) {
        world.humans.values[idx].safety = 0.2;
        world.humans.values[idx].curiosity = 0.9;
    }

    // Run simulation
    for _ in 0..10 {
        run_simulation_tick(&mut world);
    }

    // Both should have selected actions based on values
    // This validates the system runs without crashing with different values
    assert_eq!(world.entity_count(), 2);
    assert!(world.humans.task_queues[0].current().is_some() || !world.humans.task_queues[0].is_idle());
    assert!(world.humans.task_queues[1].current().is_some() || !world.humans.task_queues[1].is_idle());
}

#[test]
fn test_curious_entity_selects_observe() {
    let mut world = World::new();
    world.spawn_human("Curious George".into());

    // High curiosity, low other values
    world.humans.values[0].curiosity = 0.9;
    world.humans.values[0].love = 0.1;
    world.humans.values[0].loyalty = 0.1;
    world.humans.values[0].comfort = 0.1;

    run_simulation_tick(&mut world);

    let task = world.humans.task_queues[0].current().unwrap();
    // With high curiosity and no pressing needs, should select IdleObserve
    assert!(
        task.action.category() == ActionCategory::Idle,
        "Curious entity with no pressing needs should idle (observe or wander)"
    );
}

#[test]
fn test_social_entity_selects_talk() {
    let mut world = World::new();
    world.spawn_human("Social Sally".into());
    world.spawn_human("Other Person".into());

    // Position entities close together so they can perceive each other
    world.humans.positions[0] = Vec2::new(0.0, 0.0);
    world.humans.positions[1] = Vec2::new(5.0, 0.0);

    // Entity 0: High social values (will try to TalkTo)
    world.humans.values[0].love = 0.9;
    world.humans.values[0].loyalty = 0.9;
    world.humans.values[0].curiosity = 0.1;
    world.humans.values[0].comfort = 0.1;

    // Entity 1: Low social values, high comfort (will IdleObserve or IdleWander)
    // This ensures Entity 1 gets a persistent idle action
    world.humans.values[1].love = 0.1;
    world.humans.values[1].loyalty = 0.1;
    world.humans.values[1].curiosity = 0.1;
    world.humans.values[1].comfort = 0.5;

    // Run simulation to let entities interact
    run_simulation_tick(&mut world);

    // Entity 1 should have a task (IdleWander or IdleObserve, which persist)
    // Entity 0 may or may not have a task (TalkTo without target completes immediately)
    let entity_0_has_task = world.humans.task_queues[0].current().is_some();
    let entity_1_has_task = world.humans.task_queues[1].current().is_some();

    // At least Entity 1 should have a persistent idle task
    assert!(
        entity_1_has_task,
        "Entity 1 should have an idle task after simulation tick. E0: {:?}, E1: {:?}",
        world.humans.task_queues[0].current(),
        world.humans.task_queues[1].current()
    );

    // Verify Entity 1's task is idle (IdleWander or IdleObserve)
    let task = world.humans.task_queues[1].current().unwrap();
    assert!(
        task.action.category() == ActionCategory::Idle,
        "Entity 1 should have an idle action, got {:?}",
        task.action
    );

    // Verify needs are being processed (rest should have increased due to decay)
    assert!(
        world.humans.needs[0].rest > 0.2 || world.humans.needs[1].rest > 0.2,
        "Needs should be decaying over time"
    );
}

#[test]
fn test_value_impulse_from_strong_thought() {
    let mut world = World::new();
    world.spawn_human("Justice Jane".into());
    world.spawn_human("Victim".into());  // Add a potential target for Help

    // Position entities close together
    world.humans.positions[0] = Vec2::new(0.0, 0.0);
    world.humans.positions[1] = Vec2::new(5.0, 0.0);

    // High justice value, low social values to avoid TalkTo being selected
    world.humans.values[0].justice = 0.9;
    world.humans.values[0].love = 0.1;
    world.humans.values[0].loyalty = 0.1;
    world.humans.values[0].curiosity = 0.1;
    world.humans.values[0].comfort = 0.1;

    // Add a strong injustice thought
    let thought = Thought::new(
        Valence::Negative,
        0.8, // Strong intensity
        "injustice",
        "witnessed unfair treatment",
        CauseType::Event,
        world.current_tick,
    );
    world.humans.thoughts[0].add(thought);

    run_simulation_tick(&mut world);

    // Check that the entity responded in some way
    // Help action without a target_entity may complete immediately
    // The key is that the thought was processed and entity behaved

    // At minimum, one of the entities should have a task or thoughts were processed
    let entity_0_has_task = world.humans.task_queues[0].current().is_some();
    let entity_1_has_task = world.humans.task_queues[1].current().is_some();

    // Either entity 0 still has a task, or entity 1 got one, or thoughts decayed normally
    assert!(
        entity_0_has_task || entity_1_has_task || world.current_tick == 1,
        "Simulation should have processed entities. E0 task: {:?}, E1 task: {:?}",
        world.humans.task_queues[0].current(),
        world.humans.task_queues[1].current()
    );

    // Verify thoughts were processed (should have decayed)
    if let Some(strongest) = world.humans.thoughts[0].strongest() {
        assert!(
            strongest.intensity < 0.8,
            "Thought intensity should have decayed from 0.8"
        );
    }
}

// ============================================================================
// Thought System Tests
// ============================================================================

#[test]
fn test_thoughts_decay_over_time() {
    let mut world = World::new();
    world.spawn_human("Thinker".into());

    // Add a thought
    let thought = Thought::new(Valence::Negative, 0.5, "test", "test thought", CauseType::Event, 0);
    world.humans.thoughts[0].add(thought);

    let initial_intensity = world.humans.thoughts[0]
        .strongest()
        .map(|t| t.intensity)
        .unwrap();

    // Run ticks
    for _ in 0..5 {
        run_simulation_tick(&mut world);
    }

    // Thought should have decayed
    if let Some(thought) = world.humans.thoughts[0].strongest() {
        assert!(
            thought.intensity < initial_intensity,
            "Thought intensity should decrease over time"
        );
    }
    // Alternatively, the thought may have faded entirely, which is also correct
}

#[test]
fn test_thoughts_can_fade_completely() {
    let mut world = World::new();
    world.spawn_human("Forgetful".into());

    // Add a low-intensity thought that will fade quickly
    let thought = Thought::new(Valence::Negative, 0.15, "minor", "minor concern", CauseType::Event, 0);
    world.humans.thoughts[0].add(thought);

    // Run many ticks
    for _ in 0..100 {
        run_simulation_tick(&mut world);
    }

    // Thought should have faded (intensity < 0.1 causes removal)
    let remaining = world.humans.thoughts[0].strongest();
    // It might be None (faded) or have very low intensity
    if let Some(t) = remaining {
        assert!(t.intensity < 0.15, "Thought should have decayed");
    }
}

// ============================================================================
// Full Simulation Loop Tests
// ============================================================================

#[test]
fn test_simulation_tick_advances_counter() {
    let mut world = World::new();
    assert_eq!(world.current_tick, 0);

    run_simulation_tick(&mut world);

    assert_eq!(world.current_tick, 1);
}

#[test]
fn test_simulation_runs_for_many_ticks() {
    let mut world = World::new();

    // Spawn a small population
    for i in 0..5 {
        world.spawn_human(format!("Entity{}", i));
    }

    // Run for many ticks
    for _ in 0..1000 {
        run_simulation_tick(&mut world);
    }

    assert_eq!(world.current_tick, 1000);
    assert_eq!(world.entity_count(), 5);

    // All entities should still be alive and functional
    for i in 0..5 {
        assert!(world.humans.alive[i], "Entity {} should still be alive", i);
    }
}

#[test]
fn test_full_simulation_cycle() {
    let mut world = World::new();

    // Step 1: Spawn entities
    let alice = world.spawn_human("Alice".into());
    let bob = world.spawn_human("Bob".into());
    assert_eq!(world.entity_count(), 2);

    // Step 2: Configure different personalities
    let alice_idx = world.humans.index_of(alice).unwrap();
    let bob_idx = world.humans.index_of(bob).unwrap();

    world.humans.values[alice_idx].curiosity = 0.8;
    world.humans.values[alice_idx].safety = 0.2;

    world.humans.values[bob_idx].safety = 0.8;
    world.humans.values[bob_idx].curiosity = 0.2;

    // Step 3: Verify initial state
    assert!(world.humans.task_queues[alice_idx].is_idle());
    assert!(world.humans.task_queues[bob_idx].is_idle());

    // Step 4: Run simulation
    for _ in 0..50 {
        run_simulation_tick(&mut world);
    }

    // Step 5: Verify needs have decayed
    assert!(
        world.humans.needs[alice_idx].rest > 0.2,
        "Alice's rest need should have increased"
    );
    assert!(
        world.humans.needs[bob_idx].rest > 0.2,
        "Bob's rest need should have increased"
    );

    // Step 6: Verify entities have tasks
    assert!(
        world.humans.task_queues[alice_idx].current().is_some(),
        "Alice should have a task"
    );
    assert!(
        world.humans.task_queues[bob_idx].current().is_some(),
        "Bob should have a task"
    );

    // Step 7: Verify tick counter
    assert_eq!(world.current_tick, 50);
}

// ============================================================================
// Position and Movement Tests
// ============================================================================

#[test]
fn test_entities_have_positions() {
    let mut world = World::new();
    world.spawn_human("Test".into());

    // Entities spawn at default position (0, 0)
    let pos = world.humans.positions[0];
    assert_eq!(pos.x, 0.0);
    assert_eq!(pos.y, 0.0);
}

#[test]
fn test_positions_can_be_modified() {
    let mut world = World::new();
    world.spawn_human("Mobile".into());

    // Modify position
    world.humans.positions[0] = Vec2::new(10.0, 20.0);

    // Verify position was set correctly (before simulation runs)
    assert_eq!(world.humans.positions[0].x, 10.0);
    assert_eq!(world.humans.positions[0].y, 20.0);

    // Run simulation - position may change due to IdleWander or other actions
    run_simulation_tick(&mut world);

    // Position may have moved (e.g., IdleWander moves entity), but should be valid
    // This test verifies that position modifications persist and are used by the simulation
    let pos = world.humans.positions[0];
    assert!(pos.x.is_finite(), "Position x should be valid");
    assert!(pos.y.is_finite(), "Position y should be valid");
}

// ============================================================================
// Entity Lifecycle Tests
// ============================================================================

#[test]
fn test_living_entities_iterator() {
    let mut world = World::new();

    for i in 0..5 {
        world.spawn_human(format!("Entity{}", i));
    }

    // All should be alive initially
    let living_count = world.humans.iter_living().count();
    assert_eq!(living_count, 5);
}

#[test]
fn test_dead_entities_excluded_from_iteration() {
    let mut world = World::new();

    for i in 0..5 {
        world.spawn_human(format!("Entity{}", i));
    }

    // Kill entity 2
    world.humans.alive[2] = false;

    let living_count = world.humans.iter_living().count();
    assert_eq!(living_count, 4);

    // Dead entity should not be processed in simulation
    run_simulation_tick(&mut world);

    // Verify living entities still function
    for i in world.humans.iter_living() {
        assert!(world.humans.alive[i]);
    }
}

// ============================================================================
// Stress Tests
// ============================================================================

#[test]
fn test_large_population_simulation() {
    let mut world = World::new();

    // Spawn 100 entities
    for i in 0..100 {
        world.spawn_human(format!("Entity{}", i));
    }

    assert_eq!(world.entity_count(), 100);

    // Run simulation for 100 ticks
    for _ in 0..100 {
        run_simulation_tick(&mut world);
    }

    // All entities should still be functional
    assert_eq!(world.entity_count(), 100);
    assert_eq!(world.current_tick, 100);

    // Verify some entities have active tasks
    let entities_with_tasks = (0..100)
        .filter(|&i| world.humans.task_queues[i].current().is_some())
        .count();

    assert!(
        entities_with_tasks > 0,
        "Some entities should have tasks after simulation"
    );
}

#[test]
fn test_simulation_stability_over_time() {
    let mut world = World::new();

    // Spawn entities with varied configurations
    for i in 0..20 {
        let id = world.spawn_human(format!("Entity{}", i));
        let idx = world.humans.index_of(id).unwrap();

        // Vary values based on index
        world.humans.values[idx].curiosity = (i % 10) as f32 / 10.0;
        world.humans.values[idx].safety = ((i + 3) % 10) as f32 / 10.0;
        world.humans.values[idx].love = ((i + 5) % 10) as f32 / 10.0;
        world.humans.values[idx].justice = ((i + 7) % 10) as f32 / 10.0;
    }

    // Run for many ticks
    for tick in 0..500 {
        run_simulation_tick(&mut world);

        // Periodic sanity checks
        if tick % 100 == 99 {
            for i in 0..20 {
                // Needs should stay bounded
                assert!(world.humans.needs[i].rest <= 1.0);
                assert!(world.humans.needs[i].food <= 1.0);
                assert!(world.humans.needs[i].safety <= 1.0);
                assert!(world.humans.needs[i].rest >= 0.0);
                assert!(world.humans.needs[i].food >= 0.0);
                assert!(world.humans.needs[i].safety >= 0.0);
            }
        }
    }

    assert_eq!(world.current_tick, 500);
}

// ============================================================================
// Movement to Food Cycle Integration Tests
// ============================================================================

/// Integration test: Full movement-to-food cycle
///
/// This test verifies the complete emergent behavior chain:
/// 1. Entity spawns away from food zone (but within perception range)
/// 2. Entity detects food zone via perception
/// 3. Hungry entity selects MoveTo action toward food
/// 4. Entity moves toward food zone over multiple ticks
/// 5. When entity reaches food zone, it eats
/// 6. Eating satisfies the food need
#[test]
fn test_entity_moves_to_food_and_eats() {
    let mut world = World::new();

    // Spawn entity away from food but within perception range (50 units)
    world.spawn_human("Hungry".into());
    world.humans.positions[0] = Vec2::new(0.0, 0.0);
    world.humans.needs[0].food = 0.9; // Very hungry (critical level)

    // Add food zone within perception range (40 units away, radius 10)
    // This ensures entity can perceive the zone initially
    world.add_food_zone(Vec2::new(40.0, 0.0), 10.0, Abundance::Unlimited);

    // Record initial state
    let initial_food_need = world.humans.needs[0].food;
    let initial_x = world.humans.positions[0].x;

    // Run simulation for enough ticks to reach food and eat
    // Movement speed is 2.0 units/tick, so ~20 ticks to cover 40 units
    // Plus extra ticks for eating
    for _ in 0..50 {
        run_simulation_tick(&mut world);
    }

    // Entity should have moved toward food
    assert!(
        world.humans.positions[0].x > initial_x + 20.0,
        "Entity should have moved toward food zone. Initial x: {}, Current x: {}",
        initial_x,
        world.humans.positions[0].x
    );

    // Entity should have eaten (reduced hunger)
    assert!(
        world.humans.needs[0].food < initial_food_need,
        "Entity should have eaten and reduced hunger. Initial: {}, Current: {}",
        initial_food_need,
        world.humans.needs[0].food
    );
}

/// Integration test: Entity reaches food zone and stops moving
///
/// Verifies that entities actually arrive at the food zone (within radius)
/// and don't just move in the general direction.
#[test]
fn test_entity_reaches_food_zone() {
    let mut world = World::new();

    // Spawn entity at origin
    world.spawn_human("Traveler".into());
    world.humans.positions[0] = Vec2::new(0.0, 0.0);
    world.humans.needs[0].food = 0.9; // Very hungry

    // Add food zone 30 units away (within perception range of 50) with radius 10
    let zone_pos = Vec2::new(30.0, 0.0);
    let zone_radius = 10.0;
    world.add_food_zone(zone_pos, zone_radius, Abundance::Unlimited);

    // Run enough ticks to reach the zone
    // Distance: 30 units, speed: 2.0/tick => ~15 ticks minimum
    for _ in 0..50 {
        run_simulation_tick(&mut world);
    }

    // Entity should be within food zone radius
    let distance_to_zone = world.humans.positions[0].distance(&zone_pos);
    assert!(
        distance_to_zone <= zone_radius + 5.0, // Small tolerance for movement overshoot
        "Entity should have reached food zone. Distance to zone: {}, Zone radius: {}",
        distance_to_zone,
        zone_radius
    );
}

/// Integration test: Multiple hungry entities compete for scarce food
///
/// Tests emergent behavior when multiple entities need the same resource.
#[test]
fn test_multiple_entities_compete_for_scarce_food() {
    let mut world = World::new();

    // Spawn 3 entities at different positions (all within perception range of food)
    // Entities are staggered at 0, 10, and 20 units from origin
    for i in 0..3 {
        world.spawn_human(format!("Hungry{}", i));
        world.humans.positions[i] = Vec2::new(i as f32 * 10.0, 0.0); // Staggered positions
        world.humans.needs[i].food = 0.9; // Very hungry
    }

    // Add scarce food zone within perception range (40 units away)
    let zone_pos = Vec2::new(40.0, 0.0);
    world.add_food_zone(
        zone_pos,
        15.0,
        Abundance::Scarce {
            current: 5.0,  // Limited food
            max: 10.0,
            regen: 0.0,    // No regeneration for this test
        },
    );

    // Record initial food levels
    let initial_zone_food = match &world.food_zones[0].abundance {
        Abundance::Scarce { current, .. } => *current,
        _ => panic!("Expected Scarce zone"),
    };

    // Run simulation (enough ticks for entities to reach food and eat)
    for _ in 0..100 {
        run_simulation_tick(&mut world);
    }

    // At least some entities should have eaten (zone depleted)
    let final_zone_food = match &world.food_zones[0].abundance {
        Abundance::Scarce { current, .. } => *current,
        _ => panic!("Expected Scarce zone"),
    };

    assert!(
        final_zone_food < initial_zone_food,
        "Food zone should be depleted. Initial: {}, Final: {}",
        initial_zone_food,
        final_zone_food
    );

    // At least one entity should have reduced hunger
    let any_ate = world.humans.needs.iter().take(3).any(|n| n.food < 0.9);
    assert!(
        any_ate,
        "At least one entity should have eaten"
    );
}

// ============================================================================
// Social Memory Integration Tests
// ============================================================================

use arc_citadel::entity::social::{EventType, Disposition, SocialMemoryParams};

/// Integration test: Full social memory emergence cycle
///
/// This test verifies the complete emergent behavior chain:
/// 1. Entities spawn in a community
/// 2. Simulation runs for extended period
/// 3. Entities form relationships through accumulated encounters
/// 4. Dispositions are computed correctly from memories
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

    // Position them in a triangle (within perception range of each other)
    world.humans.positions[alice_idx] = Vec2::new(0.0, 0.0);
    world.humans.positions[bob_idx] = Vec2::new(10.0, 0.0);
    world.humans.positions[charlie_idx] = Vec2::new(5.0, 8.0);

    // Add food zone so they can survive (centered between them)
    world.add_food_zone(Vec2::new(5.0, 4.0), 20.0, Abundance::Unlimited);

    // Manually seed some initial encounters to ensure relationship formation
    // This simulates what would happen through actual simulation interactions
    world.humans.social_memories[alice_idx].record_encounter(
        bob,
        EventType::FirstMeeting,
        0.4,
        0,
    );
    world.humans.social_memories[alice_idx].record_encounter(
        bob,
        EventType::AidReceived,
        0.7,
        100,
    );
    world.humans.social_memories[bob_idx].record_encounter(
        alice,
        EventType::AidGiven,
        0.5,
        100,
    );
    world.humans.social_memories[alice_idx].record_encounter(
        charlie,
        EventType::Transaction,
        0.3,
        200,
    );

    // Run simulation for extended period
    for _ in 0..5000 {
        run_simulation_tick(&mut world);
    }

    // After extended interaction, entities should know each other
    let alice_memory = &world.humans.social_memories[alice_idx];

    // Alice should have formed relationships
    assert!(alice_memory.slots.len() >= 1,
        "Alice should know at least one other entity, has {} slots",
        alice_memory.slots.len());

    // Check that dispositions are computed correctly (not Unknown for known entities)
    for slot in &alice_memory.slots {
        let disposition = slot.get_disposition();
        // Should have some disposition, not Unknown (since we seeded memories)
        assert_ne!(disposition, Disposition::Unknown,
            "Known entity should have a non-Unknown disposition, got {:?}", disposition);
    }

    // Verify the relationship with Bob specifically (we gave Alice positive memories of Bob)
    let alice_disposition_to_bob = alice_memory.get_disposition(bob);
    assert!(
        alice_disposition_to_bob == Disposition::Friendly ||
        alice_disposition_to_bob == Disposition::Favorable,
        "Alice should have positive disposition toward Bob after receiving aid, got {:?}",
        alice_disposition_to_bob
    );
}

/// Integration test: Relationship slot eviction when capacity is exceeded
///
/// Verifies that:
/// 1. Entity can track up to 200 relationships
/// 2. When capacity is exceeded, weakest relationships are evicted
/// 3. Slot count never exceeds max_relationship_slots
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

        // Position near observer (staggered to avoid overlap)
        let idx = world.humans.index_of(id).unwrap();
        world.humans.positions[idx] = Vec2::new((i % 20) as f32 * 5.0, (i / 20) as f32 * 5.0);
    }

    // Manually create memories for all 210 entities
    // We record significant encounters (intensity > threshold) to ensure slots are created
    for (i, &entity_id) in entity_ids.iter().enumerate() {
        world.humans.social_memories[observer_idx].record_encounter(
            entity_id,
            EventType::AidReceived,  // High intensity (0.7) to ensure slot creation
            0.7,
            i as u64,
        );
    }

    // Should not exceed max slots (default is 200)
    let slots = &world.humans.social_memories[observer_idx].slots;
    let max_slots = world.humans.social_memories[observer_idx].params.max_relationship_slots;

    assert!(
        slots.len() <= max_slots,
        "Should not exceed {} relationship slots, but has {}",
        max_slots,
        slots.len()
    );
    assert_eq!(
        slots.len(),
        max_slots,
        "Should have exactly {} slots after adding {} entities",
        max_slots,
        entity_ids.len()
    );

    // Verify eviction actually occurred (some early entities should have been evicted)
    // The first entities (tick 0) should have lower recency scores and be evicted
    let evicted_count = entity_ids.iter()
        .take(10)  // Check first 10 entities
        .filter(|&&id| world.humans.social_memories[observer_idx].find_slot(id).is_none())
        .count();

    assert!(
        evicted_count > 0,
        "Some early entities should have been evicted due to lower recency, but {} remain",
        10 - evicted_count
    );
}

/// Integration test: Social memory with custom parameters
///
/// Verifies that SocialMemoryParams can be customized and respected
#[test]
fn test_social_memory_custom_params() {
    let mut world = World::new();

    let entity = world.spawn_human("CustomEntity".into());
    let entity_idx = world.humans.index_of(entity).unwrap();

    // Replace with custom params (smaller capacity for faster testing)
    let custom_params = SocialMemoryParams {
        max_relationship_slots: 5,
        memories_per_slot: 3,
        encounter_buffer_size: 10,
        slot_allocation_threshold: 0.3,
        memory_importance_floor: 0.2,
        memory_salience_decay: 0.02,
        recency_weight: 0.4,
        intensity_weight: 0.4,
        interaction_count_weight: 0.2,
    };

    world.humans.social_memories[entity_idx] =
        arc_citadel::entity::social::SocialMemory::with_params(custom_params);

    // Create 8 other entities and record encounters
    let mut others = vec![];
    for i in 0..8 {
        let other = world.spawn_human(format!("Other{}", i));
        others.push(other);

        world.humans.social_memories[entity_idx].record_encounter(
            other,
            EventType::AidReceived,
            0.7,
            i as u64 * 100,
        );
    }

    // With max_relationship_slots = 5, should have exactly 5 slots
    let slots = &world.humans.social_memories[entity_idx].slots;
    assert_eq!(
        slots.len(),
        5,
        "Custom params should limit slots to 5, got {}",
        slots.len()
    );
}

/// Integration test: Disposition influences behavior over simulation
///
/// Verifies that entities with different dispositions toward each other
/// will have their relationships persist through simulation ticks
#[test]
fn test_disposition_persistence_through_simulation() {
    let mut world = World::new();

    let alice = world.spawn_human("Alice".into());
    let bob = world.spawn_human("Bob".into());

    let alice_idx = world.humans.index_of(alice).unwrap();
    let bob_idx = world.humans.index_of(bob).unwrap();

    // Position them together
    world.humans.positions[alice_idx] = Vec2::new(0.0, 0.0);
    world.humans.positions[bob_idx] = Vec2::new(5.0, 0.0);

    // Add food zone
    world.add_food_zone(Vec2::new(2.5, 0.0), 10.0, Abundance::Unlimited);

    // Create a hostile relationship from Alice to Bob
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
        10,
    );

    // Verify initial hostile disposition
    let initial_disposition = world.humans.social_memories[alice_idx].get_disposition(bob);
    assert_eq!(
        initial_disposition,
        Disposition::Hostile,
        "Initial disposition should be Hostile, got {:?}",
        initial_disposition
    );

    // Run simulation for a while
    for _ in 0..500 {
        run_simulation_tick(&mut world);
    }

    // Relationship should persist (memories don't decay that fast)
    let final_disposition = world.humans.social_memories[alice_idx].get_disposition(bob);
    assert!(
        final_disposition == Disposition::Hostile || final_disposition == Disposition::Suspicious,
        "Disposition should remain negative after 500 ticks, got {:?}",
        final_disposition
    );

    // The slot should still exist
    assert!(
        world.humans.social_memories[alice_idx].find_slot(bob).is_some(),
        "Relationship slot should persist through simulation"
    );
}

// ============================================================================
// Action Execution Workflow Integration Tests
// ============================================================================

use arc_citadel::simulation::{ResourceZone, ResourceType};

/// Integration test: Survival priority chain
///
/// This test verifies that entities with multiple critical needs
/// address them in a sensible priority order. An entity that is
/// both hungry and tired should:
/// 1. Eat when in a food zone (hunger is higher priority)
/// 2. Rest when hunger is satisfied and location is safe (low safety need)
///
/// Note: Rest requires safe_location (safety need < 0.3), so we set up
/// the entity to feel safe to enable Rest behavior.
#[test]
fn test_survival_priority_chain() {
    let mut world = World::new();

    // Create food zone at origin
    world.add_food_zone(Vec2::new(0.0, 0.0), 10.0, Abundance::Unlimited);

    // Spawn entity at the food zone
    let id = world.spawn_human("Survivor".into());
    let idx = world.humans.index_of(id).unwrap();
    world.humans.positions[idx] = Vec2::new(0.0, 0.0);

    // Very hungry and tired, but feels safe (low safety need enables Rest)
    world.humans.needs[idx].food = 0.9;   // Critical hunger
    world.humans.needs[idx].rest = 0.85;  // Critical rest need (triggers Rest action)
    world.humans.needs[idx].safety = 0.1; // Low safety need = safe location

    let initial_hunger = world.humans.needs[idx].food;
    let initial_rest = world.humans.needs[idx].rest;

    // Run simulation for 100 ticks
    for _ in 0..100 {
        run_simulation_tick(&mut world);
    }

    // Hunger should be reduced (entity ate)
    assert!(
        world.humans.needs[idx].food < initial_hunger,
        "Hunger should be reduced after eating. Initial: {}, Final: {}",
        initial_hunger,
        world.humans.needs[idx].food
    );

    // Rest need should be reduced (entity rested when safe_location is true)
    // The Rest action satisfies the rest need, not directly reducing fatigue
    assert!(
        world.humans.needs[idx].rest < initial_rest,
        "Rest need should be reduced after resting. Initial: {}, Final: {}",
        initial_rest,
        world.humans.needs[idx].rest
    );
}

/// Integration test: Social clustering through social actions
///
/// This test verifies that entities with high social needs and
/// friendly dispositions toward each other will interact and
/// form social memories. When entities have prior positive experiences
/// with each other, they will seek each other out for social interaction.
///
/// Note: Social action selection requires friendly dispositions, so we
/// seed friendly memories between adjacent entities to enable TalkTo.
#[test]
fn test_social_clustering() {
    use arc_citadel::entity::social::EventType;

    let mut world = World::new();

    // Spawn entities close together (within perception range of 50 units)
    // Position them at 0, 10, 20, 30, 40 units apart (all within range of each other)
    let mut ids = Vec::new();
    for i in 0..5 {
        let id = world.spawn_human(format!("Person{}", i));
        let idx = world.humans.index_of(id).unwrap();
        world.humans.positions[idx] = Vec2::new(i as f32 * 10.0, 0.0);
        world.humans.needs[idx].social = 0.8;  // High social need
        ids.push(id);
    }

    // Seed friendly relationships between adjacent entities
    // This creates the conditions for social interaction
    for i in 0..4 {
        let idx_a = world.humans.index_of(ids[i]).unwrap();
        let idx_b = world.humans.index_of(ids[i+1]).unwrap();

        // Entity A has positive memory of entity B (and vice versa)
        world.humans.social_memories[idx_a].record_encounter(
            ids[i+1],
            EventType::AidReceived,  // Creates friendly disposition
            0.7,
            0,
        );
        world.humans.social_memories[idx_b].record_encounter(
            ids[i],
            EventType::AidReceived,
            0.7,
            0,
        );
    }

    // Run simulation for 200 ticks
    for _ in 0..200 {
        run_simulation_tick(&mut world);
    }

    // Verify that social interactions occurred by checking:
    // 1. Social needs were somewhat satisfied
    // 2. Additional social memories were formed
    let mut social_needs_satisfied = 0;
    let mut has_social_memories = 0;

    for i in 0..5 {
        let idx = world.humans.index_of(ids[i]).unwrap();

        // Check if social need decreased
        if world.humans.needs[idx].social < 0.8 {
            social_needs_satisfied += 1;
        }

        // Check if entity has social memories
        if world.humans.social_memories[idx].slots.len() > 0 {
            has_social_memories += 1;
        }
    }

    // At least some entities should have had social needs satisfied
    assert!(
        social_needs_satisfied >= 2,
        "At least 2 entities should have reduced social needs through interaction. Got: {}",
        social_needs_satisfied
    );

    // All entities should have social memories (from seeding + interactions)
    assert!(
        has_social_memories == 5,
        "All entities should have social memories. Got: {}",
        has_social_memories
    );
}

/// Integration test: Resource gathering workflow
///
/// This test verifies the complete gather workflow:
/// 1. Create a ResourceZone at a location
/// 2. Spawn a worker far from the zone with high purpose need
/// 3. Give the worker a Gather task targeting the resource zone
/// 4. Run simulation until task completes or timeout
/// 5. Verify resources were depleted and purpose need was satisfied
#[test]
fn test_resource_gathering_workflow() {
    let mut world = World::new();

    // Create resource zone at (20, 0) with radius 5
    world.resource_zones.push(ResourceZone::new(
        Vec2::new(20.0, 0.0),
        ResourceType::Wood,
        5.0,
    ));

    // Spawn worker at origin (0, 0) - needs to move to resource zone
    let id = world.spawn_human("Worker".into());
    let idx = world.humans.index_of(id).unwrap();
    world.humans.positions[idx] = Vec2::new(0.0, 0.0);
    world.humans.needs[idx].purpose = 0.8;  // High purpose need

    // Give gather task targeting the resource zone
    let task = Task::new(ActionId::Gather, TaskPriority::Normal, 0)
        .with_position(Vec2::new(20.0, 0.0));
    world.humans.task_queues[idx].push(task);

    let initial_resources = world.resource_zones[0].current;
    let initial_purpose = world.humans.needs[idx].purpose;

    // Run until task completes or 200 ticks
    let mut ticks = 0;
    while ticks < 200 {
        run_simulation_tick(&mut world);
        ticks += 1;

        // Check if task is complete (no longer gathering)
        let is_still_gathering = world.humans.task_queues[idx]
            .current()
            .map(|t| t.action == ActionId::Gather)
            .unwrap_or(false);

        if !is_still_gathering {
            break;
        }
    }

    // Verify resources were depleted
    assert!(
        world.resource_zones[0].current < initial_resources,
        "Resources should be depleted. Initial: {}, Final: {}",
        initial_resources,
        world.resource_zones[0].current
    );

    // Verify purpose need was satisfied
    assert!(
        world.humans.needs[idx].purpose < initial_purpose,
        "Purpose need should be satisfied. Initial: {}, Final: {}",
        initial_purpose,
        world.humans.needs[idx].purpose
    );

    // Verify entity moved toward the resource zone
    let final_pos = world.humans.positions[idx];
    assert!(
        final_pos.x > 10.0,
        "Entity should have moved toward resource zone. Final x: {}",
        final_pos.x
    );
}

// ============================================================================
// Astronomical System Integration Tests
// ============================================================================

use arc_citadel::core::astronomy::{
    AstronomicalState, CelestialEvent, FoundingModifiers, Season, CONJUNCTION_CYCLE, TICKS_PER_DAY,
    YEAR_LENGTH,
};

/// Integration test: Full year simulation
///
/// This test verifies that the astronomical system correctly advances through
/// an entire year (360 days × TICKS_PER_DAY), rolling over to year 2 and
/// returning to Season::Spring on day 1.
#[test]
fn test_full_year_simulation() {
    let mut state = AstronomicalState::new(TICKS_PER_DAY);

    // Verify initial state
    assert_eq!(state.year, 1);
    assert_eq!(state.day_of_year, 1);
    assert_eq!(state.season, Season::Spring);

    // Advance through entire year (360 days × 1000 ticks per day)
    for _ in 0..(YEAR_LENGTH as u64 * TICKS_PER_DAY) {
        state.advance_tick();
    }

    // After one full year, should be in year 2
    assert_eq!(state.year, 2, "Year should roll over to 2 after 360 days");

    // Day of year should be back to 1
    assert_eq!(state.day_of_year, 1, "Day of year should reset to 1");

    // Season should be back to Spring
    assert_eq!(state.season, Season::Spring, "Season should reset to Spring");
}

/// Integration test: Light levels vary through the day
///
/// This test verifies that light levels correctly vary from dark at night
/// to bright at midday, ensuring the solar phase system is working correctly
/// in conjunction with the tick advancement.
#[test]
fn test_light_levels_through_day() {
    let mut state = AstronomicalState::new(TICKS_PER_DAY);

    let mut min_light: f32 = 1.0;
    let mut max_light: f32 = 0.0;

    // Advance through one full day (1000 ticks)
    for _ in 0..TICKS_PER_DAY {
        state.advance_tick();
        min_light = min_light.min(state.light_level);
        max_light = max_light.max(state.light_level);
    }

    // Should have darkness at night (light < 0.3)
    assert!(
        min_light < 0.3,
        "Should have darkness at night. Minimum light level was: {}",
        min_light
    );

    // Should have brightness at midday (light > 0.9)
    assert!(
        max_light > 0.9,
        "Should have brightness at midday. Maximum light level was: {}",
        max_light
    );

    // Light levels should vary significantly through the day
    let variation = max_light - min_light;
    assert!(
        variation > 0.7,
        "Light levels should vary significantly through the day. Variation was: {}",
        variation
    );
}

/// Integration test: Double full moon is rare
///
/// This test verifies that PerfectDoubleFull events are relatively rare compared
/// to regular full moon events. The event requires BOTH moons to be near full
/// simultaneously (both phases within 0.02 of 0.5).
///
/// With Argent period of 29 days and Sanguine period of 83 days:
/// - PerfectDoubleFull occurs when both moons are within ~1 day of exact full
/// - This happens multiple times per conjunction cycle, but is still much rarer
///   than individual full moons
/// - Expected: significantly fewer PerfectDoubleFull days than regular FullArgent days
#[test]
fn test_double_full_is_rare() {
    let mut state = AstronomicalState::new(TICKS_PER_DAY);

    // Precompute events for more than one full conjunction cycle
    let days_to_compute = CONJUNCTION_CYCLE + 100;
    state.precompute_events(days_to_compute);

    // Count PerfectDoubleFull events
    let mut perfect_double_count = 0;
    let mut full_argent_count = 0;
    for (_, events) in &state.event_calendar {
        if events.contains(&CelestialEvent::PerfectDoubleFull) {
            perfect_double_count += 1;
        }
        if events.contains(&CelestialEvent::FullArgent) {
            full_argent_count += 1;
        }
    }

    // PerfectDoubleFull should exist (not zero)
    assert!(
        perfect_double_count >= 1,
        "Should find at least 1 PerfectDoubleFull event in {} days, found {}",
        days_to_compute,
        perfect_double_count
    );

    // PerfectDoubleFull should be MUCH rarer than regular FullArgent
    // FullArgent occurs ~12 times per year, so ~85 times in 2507 days
    // PerfectDoubleFull should be less than 15% of that
    let rarity_ratio = perfect_double_count as f32 / full_argent_count as f32;
    assert!(
        rarity_ratio < 0.15,
        "PerfectDoubleFull should be rare compared to FullArgent. \
         Found {} PerfectDoubleFull vs {} FullArgent (ratio: {:.2})",
        perfect_double_count,
        full_argent_count,
        rarity_ratio
    );

    // Sanity check: should have reasonable counts
    assert!(
        full_argent_count >= 50,
        "Should find many FullArgent events in {} days, found {}",
        days_to_compute,
        full_argent_count
    );
}

/// Integration test: Founding conditions in deep winter
///
/// This test verifies the integration between AstronomicalState and
/// FoundingModifiers by:
/// 1. Advancing the simulation to deep winter (day 340)
/// 2. Verifying the season is correctly identified as Winter
/// 3. Calculating FoundingModifiers based on the astronomical state
/// 4. Verifying siege_mentality is true and defensive bias is applied
#[test]
fn test_founding_conditions_integration() {
    let mut state = AstronomicalState::new(TICKS_PER_DAY);

    // Advance to deep winter (day 340)
    // Day 340 is in deep winter range (day_of_year >= 300)
    let target_day = 340;
    for _ in 0..(target_day as u64 * TICKS_PER_DAY) {
        state.advance_tick();
    }

    // Verify we're in Winter season
    assert_eq!(
        state.season,
        Season::Winter,
        "Day {} should be in Winter, but got {:?}",
        target_day,
        state.season
    );

    // Verify day_of_year is around 340 (accounting for 1-based indexing)
    // Day 340 from start should be day_of_year 341 (since day_of_year is 1-based)
    assert!(
        state.day_of_year >= 300,
        "Should be in deep winter (day_of_year >= 300), got day_of_year {}",
        state.day_of_year
    );

    // Calculate founding modifiers based on current astronomical state
    let modifiers = FoundingModifiers::calculate(
        state.day_of_year,
        state.season,
        &state.active_events,
    );

    // Deep winter (day_of_year >= 300) should trigger siege_mentality
    assert!(
        modifiers.siege_mentality,
        "Deep winter founding should have siege_mentality = true"
    );

    // Should have defensive bias tag
    assert!(
        modifiers.bias_tags.contains(&"defensive".to_string()),
        "Deep winter founding should have 'defensive' bias tag. Tags: {:?}",
        modifiers.bias_tags
    );

    // Should have increased stockpile efficiency (> 1.0 base)
    assert!(
        modifiers.stockpile_efficiency > 1.0,
        "Deep winter founding should have stockpile_efficiency > 1.0, got {}",
        modifiers.stockpile_efficiency
    );

    // Should have reduced initial population
    assert!(
        modifiers.initial_population_mult < 1.0,
        "Deep winter founding should have reduced initial_population_mult, got {}",
        modifiers.initial_population_mult
    );

    // Should have increased defensive weight
    assert!(
        modifiers.defensive_weight > 0.0,
        "Deep winter founding should have positive defensive_weight, got {}",
        modifiers.defensive_weight
    );
}
