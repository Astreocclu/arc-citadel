//! Tick system - orchestrates simulation updates
//!
//! This is the core loop that ties together:
//! perception -> thought generation -> need modification -> action selection -> action execution
//!
//! Each tick advances the simulation one step, processing all entities.
//!
//! Uses rayon for parallel processing where safe.

use crate::ecs::world::World;
use crate::spatial::sparse_hash::SparseHashGrid;
use crate::simulation::perception::{perception_system, find_nearest_food_zone, RelationshipType};
use crate::simulation::action_select::{select_action_human, SelectionContext};
use crate::entity::thoughts::{Thought, Valence, CauseType};
use crate::entity::needs::NeedType;
use crate::entity::tasks::Task;
use crate::actions::catalog::ActionId;
use rayon::prelude::*;

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
/// - Rest increases faster when active (but not during restful actions)
/// - Food increases steadily
/// - Social and purpose increase slowly
/// - Safety decreases naturally when no threats present
fn update_needs(world: &mut World) {
    let dt = 1.0;
    let living_indices: Vec<usize> = world.humans.iter_living().collect();

    // Sequential - this is already O(n) and very fast
    for i in living_indices {
        // Check if current action is restful (Rest, IdleObserve, Eat)
        let is_restful = world.humans.task_queues[i]
            .current()
            .map(|t| t.action.is_restful())
            .unwrap_or(true); // No task = resting

        let is_active = !is_restful;
        world.humans.needs[i].decay(dt, is_active);
    }
}

/// Run the perception system for all entities (PARALLEL when beneficial)
///
/// Creates a spatial hash grid for efficient neighbor queries,
/// then runs perception for each entity to determine what they can see.
/// Also populates nearest_food_zone for each entity.
fn run_perception(world: &World) -> Vec<crate::simulation::perception::Perception> {
    let mut grid = SparseHashGrid::new(10.0);

    // Collect positions and IDs for spatial queries
    let positions: Vec<_> = world.humans.positions.iter().cloned().collect();
    let ids: Vec<_> = world.humans.ids.iter().cloned().collect();

    // Build spatial grid
    grid.rebuild(ids.iter().cloned().zip(positions.iter().cloned()));

    // Use parallel for large entity counts, sequential for small
    let mut perceptions = if ids.len() >= PARALLEL_THRESHOLD {
        perception_system_parallel(&grid, &positions, &ids, 50.0)
    } else {
        perception_system(&grid, &positions, &ids, 50.0)
    };

    // Populate nearest_food_zone for each perception
    let perception_range = 50.0;
    for (i, perception) in perceptions.iter_mut().enumerate() {
        perception.nearest_food_zone = find_nearest_food_zone(
            positions[i],
            perception_range,
            &world.food_zones,
        );
    }

    perceptions
}

/// Parallel perception - each entity's perception is independent
fn perception_system_parallel(
    spatial_grid: &SparseHashGrid,
    positions: &[crate::core::types::Vec2],
    entity_ids: &[crate::core::types::EntityId],
    perception_range: f32,
) -> Vec<crate::simulation::perception::Perception> {
    use crate::simulation::perception::{Perception, PerceivedEntity};

    // Build O(1) lookup map
    let id_to_idx: ahash::AHashMap<_, _> = entity_ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    // PARALLEL: process each entity's perception independently
    entity_ids
        .par_iter()
        .enumerate()
        .map(|(i, &observer_id)| {
            let observer_pos = positions[i];

            let nearby: Vec<_> = spatial_grid
                .query_neighbors(observer_pos)
                .filter(|&e| e != observer_id)
                .collect();

            let perceived_entities: Vec<_> = nearby
                .iter()
                .filter_map(|&entity| {
                    let entity_idx = *id_to_idx.get(&entity)?;
                    let entity_pos = positions[entity_idx];
                    let distance = observer_pos.distance(&entity_pos);

                    if distance <= perception_range {
                        Some(PerceivedEntity {
                            entity,
                            distance,
                            relationship: crate::simulation::perception::RelationshipType::Unknown,
                            threat_level: 0.0,
                            notable_features: vec![],
                        })
                    } else {
                        None
                    }
                })
                .collect();

            Perception {
                observer: observer_id,
                perceived_entities,
                perceived_objects: vec![],
                perceived_events: vec![],
                nearest_food_zone: None, // Will be populated after with food zone data
            }
        })
        .collect()
}

/// Generate thoughts from perceptions
///
/// This is where entities react to what they perceive based on their values.
/// Perceptions are filtered through values to generate appropriate thoughts.
fn generate_thoughts(world: &mut World, perceptions: &[crate::simulation::perception::Perception]) {
    // Build O(1) lookup map once, not O(n) search per perception
    let id_to_idx: ahash::AHashMap<crate::core::types::EntityId, usize> = world.humans.ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    for perception in perceptions {
        let Some(&idx) = id_to_idx.get(&perception.observer) else { continue };
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

/// Threshold for parallelization (below this, sequential is faster due to thread overhead)
const PARALLEL_THRESHOLD: usize = 1000;

/// Select actions for entities without current tasks (PARALLEL when beneficial)
///
/// Uses the action selection algorithm to choose appropriate actions
/// based on needs, thoughts, and values.
fn select_actions(world: &mut World) {
    let living_indices: Vec<usize> = world.humans.iter_living().collect();
    let current_tick = world.current_tick;

    if living_indices.len() >= PARALLEL_THRESHOLD {
        // PARALLEL path for large entity counts
        let selected_actions: Vec<(usize, Option<Task>)> = living_indices
            .par_iter()
            .filter_map(|&i| {
                if world.humans.task_queues[i].current().is_some() {
                    return None;
                }
                let pos = world.humans.positions[i];
                // Check if entity is AT a food zone
                let food_available = world.food_zones.iter()
                    .any(|zone| zone.contains(pos));
                // Find nearest food zone within perception range
                let nearest_food_zone = find_nearest_food_zone(pos, 50.0, &world.food_zones);

                let ctx = SelectionContext {
                    body: &world.humans.body_states[i],
                    needs: &world.humans.needs[i],
                    thoughts: &world.humans.thoughts[i],
                    values: &world.humans.values[i],
                    has_current_task: false,
                    threat_nearby: world.humans.needs[i].safety > 0.5,
                    food_available,
                    safe_location: world.humans.needs[i].safety < 0.3,
                    entity_nearby: true,
                    current_tick,
                    nearest_food_zone,
                };
                Some((i, select_action_human(&ctx)))
            })
            .collect();

        for (i, task_opt) in selected_actions {
            if let Some(task) = task_opt {
                world.humans.task_queues[i].push(task);
            }
        }
    } else {
        // Sequential path for small entity counts (avoids thread overhead)
        for i in living_indices {
            if world.humans.task_queues[i].current().is_some() {
                continue;
            }
            let pos = world.humans.positions[i];
            // Check if entity is AT a food zone
            let food_available = world.food_zones.iter()
                .any(|zone| zone.contains(pos));
            // Find nearest food zone within perception range
            let nearest_food_zone = find_nearest_food_zone(pos, 50.0, &world.food_zones);

            let ctx = SelectionContext {
                body: &world.humans.body_states[i],
                needs: &world.humans.needs[i],
                thoughts: &world.humans.thoughts[i],
                values: &world.humans.values[i],
                has_current_task: false,
                threat_nearby: world.humans.needs[i].safety > 0.5,
                food_available,
                safe_location: world.humans.needs[i].safety < 0.3,
                entity_nearby: true,
                current_tick,
                nearest_food_zone,
            };
            if let Some(task) = select_action_human(&ctx) {
                world.humans.task_queues[i].push(task);
            }
        }
    }
}

/// Execute current tasks for all entities
///
/// Progresses tasks and applies their effects (need satisfaction).
/// Completes tasks when they reach full progress.
///
/// # Movement Actions
/// - MoveTo: Moves toward target_position at 2.0 units/tick, completes within 2.0 units
/// - Flee: Moves away from target_position at 3.0 units/tick, never auto-completes
///
/// # Progress Rates (non-movement)
/// - Continuous (duration 0): 0.1/tick → NEVER completes (cancelled/replaced)
/// - Quick (duration 1-60): 0.05/tick → completes in 20 ticks
/// - Long (duration > 60): 0.02/tick → completes in 50 ticks
///
/// # Need Satisfaction
/// Actions satisfy needs at `amount * 0.05` per tick.
/// This creates meaningful time investment:
/// - Rest action: 0.3 × 0.05 × 50 ticks = 0.75 total satisfaction
/// - Eat action: 0.5 × 0.05 × 20 ticks = 0.5 total satisfaction
fn execute_tasks(world: &mut World) {
    // Collect indices first to avoid borrow conflicts
    let living_indices: Vec<usize> = world.humans.iter_living().collect();
    for i in living_indices {
        // Get task info first (action, target_position, and whether complete)
        let task_info = world.humans.task_queues[i].current_mut().map(|task| {
            let action = task.action;
            let target_pos = task.target_position;

            // Handle movement actions separately
            let movement_complete = match action {
                ActionId::MoveTo => {
                    if let Some(target) = target_pos {
                        let current = world.humans.positions[i];
                        let direction = (target - current).normalize();
                        let speed = 2.0; // units per tick

                        // Move toward target (normalize handles zero-length vectors)
                        if direction.length() > 0.0 {
                            world.humans.positions[i] = current + direction * speed;
                        }

                        // Check if arrived (within 2.0 units)
                        world.humans.positions[i].distance(&target) < 2.0
                    } else {
                        true // No target, complete immediately
                    }
                }
                ActionId::Flee => {
                    if let Some(threat_pos) = target_pos {
                        let current = world.humans.positions[i];
                        let away = (current - threat_pos).normalize();
                        let speed = 3.0; // Flee faster

                        // Move away from threat (normalize handles zero-length vectors)
                        if away.length() > 0.0 {
                            world.humans.positions[i] = current + away * speed;
                        }
                    }
                    false // Flee continues until interrupted
                }
                _ => false, // Not a movement action
            };

            // Progress for non-movement actions
            let is_movement = matches!(action, ActionId::MoveTo | ActionId::Flee);
            if !is_movement {
                let duration = task.action.base_duration();
                let progress_rate = match duration {
                    0 => 0.1,       // Continuous actions progress but never complete
                    1..=60 => 0.05, // Quick: Eat, TalkTo, Attack (20 ticks)
                    _ => 0.02,      // Long: Build, Craft, Rest (50 ticks)
                };
                task.progress += progress_rate;
            }

            let duration = task.action.base_duration();
            // Duration 0 = continuous actions (IdleWander, IdleObserve)
            // These NEVER complete automatically - they get cancelled/replaced
            let progress_complete = duration > 0 && task.progress >= 1.0;
            let is_complete = movement_complete || progress_complete;
            (action, is_complete)
        });

        if let Some((action, is_complete)) = task_info {
            // SATISFACTION MULTIPLIER: 0.05
            // Actions apply a fraction of their nominal satisfaction each tick.
            // This accumulates over the task duration to total satisfaction.
            // Without this, entities would fully satisfy needs in one tick.
            for (need, amount) in action.satisfies_needs() {
                world.humans.needs[i].satisfy(need, amount * 0.05);
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

    #[test]
    fn test_moveto_changes_position() {
        use crate::core::types::Vec2;
        use crate::entity::tasks::{Task, TaskPriority, TaskSource};
        use crate::actions::catalog::ActionId;

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
}
