//! Tick system - orchestrates simulation updates
//!
//! This is the core loop that ties together:
//! perception -> thought generation -> need modification -> action selection -> action execution
//!
//! Each tick advances the simulation one step, processing all entities.

use crate::ecs::world::World;
use crate::spatial::sparse_hash::SparseHashGrid;
use crate::simulation::perception::{perception_system, RelationshipType};
use crate::simulation::action_select::{select_action_human, SelectionContext};
use crate::entity::thoughts::{Thought, Valence, CauseType};
use crate::entity::needs::NeedType;

/// Run a single simulation tick
///
/// This is the main entry point that orchestrates all simulation systems:
/// 1. Update needs (decay over time)
/// 2. Run perception (entities observe their surroundings)
/// 3. Generate thoughts (reactions to perceptions)
/// 4. Decay thoughts (thoughts fade over time)
/// 5. Select actions (decide what to do based on needs, thoughts, values)
/// 6. Execute tasks (progress current tasks, satisfy needs)
/// 7. Advance tick counter
pub fn run_simulation_tick(world: &mut World) {
    update_needs(world);
    let perceptions = run_perception(world);
    generate_thoughts(world, &perceptions);
    decay_thoughts(world);
    select_actions(world);
    execute_tasks(world);
    world.tick();
}

/// Update all entity needs based on time passage
///
/// Needs decay (increase) over time:
/// - Rest increases faster when active
/// - Food increases steadily
/// - Social and purpose increase slowly
/// - Safety decreases naturally when no threats present
fn update_needs(world: &mut World) {
    let dt = 1.0; // One tick = one time unit
    // Collect indices first to avoid borrow conflicts
    let living_indices: Vec<usize> = world.humans.iter_living().collect();
    for i in living_indices {
        let is_active = world.humans.task_queues[i].current().is_some();
        world.humans.needs[i].decay(dt, is_active);
    }
}

/// Run the perception system for all entities
///
/// Creates a spatial hash grid for efficient neighbor queries,
/// then runs perception for each entity to determine what they can see.
fn run_perception(world: &World) -> Vec<crate::simulation::perception::Perception> {
    let mut grid = SparseHashGrid::new(10.0);

    // Collect positions and IDs for spatial queries
    let positions: Vec<_> = world.humans.positions.iter().cloned().collect();
    let ids: Vec<_> = world.humans.ids.iter().cloned().collect();

    // Build spatial grid
    grid.rebuild(ids.iter().cloned().zip(positions.iter().cloned()));

    // Run perception for all entities
    perception_system(&grid, &positions, &ids, 50.0)
}

/// Generate thoughts from perceptions
///
/// This is where entities react to what they perceive based on their values.
/// Perceptions are filtered through values to generate appropriate thoughts.
fn generate_thoughts(world: &mut World, perceptions: &[crate::simulation::perception::Perception]) {
    for perception in perceptions {
        let Some(idx) = world.humans.index_of(perception.observer) else { continue };
        let values = &world.humans.values[idx];

        // Process perceived entities
        for perceived in &perception.perceived_entities {
            // Generate threat-based thoughts
            if perceived.threat_level > 0.5 {
                let thought = Thought::new(
                    Valence::Negative,
                    perceived.threat_level,
                    if values.safety > 0.5 { "fear" } else { "concern" },
                    "threatening entity nearby",
                    CauseType::Entity,
                    world.current_tick,
                );
                world.humans.thoughts[idx].add(thought);

                // Increase safety need based on threat
                world.humans.needs[idx].safety =
                    (world.humans.needs[idx].safety + perceived.threat_level * 0.3).min(1.0);
            }

            // Allies reduce social need
            if perceived.relationship == RelationshipType::Ally {
                world.humans.needs[idx].satisfy(NeedType::Social, 0.1);
            }
        }

        // Process perceived events
        for event in &perception.perceived_events {
            // Generate thoughts based on event significance
            if event.significance > 0.5 {
                let valence = if event.event_type.contains("positive") ||
                                 event.event_type.contains("celebration") {
                    Valence::Positive
                } else {
                    Valence::Negative
                };

                let thought = Thought::new(
                    valence,
                    event.significance,
                    &event.event_type,
                    format!("witnessed {}", event.event_type),
                    CauseType::Event,
                    world.current_tick,
                );
                world.humans.thoughts[idx].add(thought);
            }
        }
    }
}

/// Decay all thoughts over time
///
/// Thoughts naturally fade, and faded thoughts are removed from the buffer.
fn decay_thoughts(world: &mut World) {
    // Collect indices first to avoid borrow conflicts
    let living_indices: Vec<usize> = world.humans.iter_living().collect();
    for i in living_indices {
        world.humans.thoughts[i].decay_all();
    }
}

/// Select actions for entities without current tasks
///
/// Uses the action selection algorithm to choose appropriate actions
/// based on needs, thoughts, and values.
fn select_actions(world: &mut World) {
    // Collect indices first to avoid borrow conflicts
    let living_indices: Vec<usize> = world.humans.iter_living().collect();
    for i in living_indices {
        // Skip entities that already have a task
        if world.humans.task_queues[i].current().is_some() {
            continue;
        }

        // Build selection context
        let ctx = SelectionContext {
            body: &world.humans.body_states[i],
            needs: &world.humans.needs[i],
            thoughts: &world.humans.thoughts[i],
            values: &world.humans.values[i],
            has_current_task: false,
            threat_nearby: world.humans.needs[i].safety > 0.5,
            food_available: true, // TODO: check actual food availability
            safe_location: world.humans.needs[i].safety < 0.3,
            entity_nearby: true, // TODO: check actual entity proximity
            current_tick: world.current_tick,
        };

        // Select and assign action
        if let Some(task) = select_action_human(&ctx) {
            world.humans.task_queues[i].push(task);
        }
    }
}

/// Execute current tasks for all entities
///
/// Progresses tasks and applies their effects (need satisfaction).
/// Completes tasks when they reach full progress.
fn execute_tasks(world: &mut World) {
    // Collect indices first to avoid borrow conflicts
    let living_indices: Vec<usize> = world.humans.iter_living().collect();
    for i in living_indices {
        // Get task info first (action and whether complete)
        let task_info = world.humans.task_queues[i].current_mut().map(|task| {
            // Progress the task
            task.progress += 0.01;
            let action = task.action;
            let duration = task.action.base_duration();
            let is_complete = duration > 0 && task.progress >= 1.0;
            (action, is_complete)
        });

        if let Some((action, is_complete)) = task_info {
            // Satisfy needs based on action type
            for (need, amount) in action.satisfies_needs() {
                world.humans.needs[i].satisfy(need, amount * 0.01);
            }

            // Complete task if done
            if is_complete {
                world.humans.task_queues[i].complete_current();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_advances_counter() {
        let mut world = World::new();
        assert_eq!(world.current_tick, 0);

        run_simulation_tick(&mut world);

        assert_eq!(world.current_tick, 1);
    }

    #[test]
    fn test_tick_with_entities() {
        let mut world = World::new();
        world.spawn_human("Alice".into());
        world.spawn_human("Bob".into());

        // Run several ticks
        for _ in 0..10 {
            run_simulation_tick(&mut world);
        }

        assert_eq!(world.current_tick, 10);
        assert_eq!(world.entity_count(), 2);
    }

    #[test]
    fn test_needs_decay_over_ticks() {
        let mut world = World::new();
        world.spawn_human("Test".into());

        let initial_rest = world.humans.needs[0].rest;

        // Run some ticks
        for _ in 0..10 {
            run_simulation_tick(&mut world);
        }

        // Rest need should have increased (decayed)
        assert!(world.humans.needs[0].rest > initial_rest);
    }

    #[test]
    fn test_idle_entities_get_tasks() {
        let mut world = World::new();
        world.spawn_human("Idle Entity".into());

        // Initially no task
        assert!(world.humans.task_queues[0].is_idle());

        // After a tick, should have been assigned an action
        run_simulation_tick(&mut world);

        // Should now have a task (idle action)
        assert!(world.humans.task_queues[0].current().is_some());
    }

    #[test]
    fn test_thoughts_decay() {
        let mut world = World::new();
        world.spawn_human("Thinker".into());

        // Add a thought manually
        let thought = Thought::new(
            Valence::Negative,
            0.5,
            "test",
            "test thought",
            CauseType::Event,
            0,
        );
        world.humans.thoughts[0].add(thought);

        // Get initial intensity
        let initial_intensity = world.humans.thoughts[0].strongest().map(|t| t.intensity);
        assert!(initial_intensity.is_some());

        // Run ticks
        for _ in 0..5 {
            run_simulation_tick(&mut world);
        }

        // Thought should have decayed
        if let Some(thought) = world.humans.thoughts[0].strongest() {
            assert!(thought.intensity < initial_intensity.unwrap());
        }
    }

    #[test]
    fn test_multiple_entities_processed() {
        let mut world = World::new();

        // Spawn multiple entities
        for i in 0..5 {
            world.spawn_human(format!("Entity{}", i));
        }

        // Run tick
        run_simulation_tick(&mut world);

        // All entities should have tasks
        for i in 0..5 {
            assert!(world.humans.task_queues[i].current().is_some());
        }
    }

    #[test]
    fn test_task_progress() {
        let mut world = World::new();
        world.spawn_human("Worker".into());

        // Run first tick to get a task
        run_simulation_tick(&mut world);

        // Get initial progress
        let initial_progress = world.humans.task_queues[0]
            .current()
            .map(|t| t.progress)
            .unwrap_or(0.0);

        // Run more ticks
        run_simulation_tick(&mut world);

        // Progress should have increased
        let current_progress = world.humans.task_queues[0]
            .current()
            .map(|t| t.progress)
            .unwrap_or(0.0);

        assert!(current_progress > initial_progress);
    }
}
