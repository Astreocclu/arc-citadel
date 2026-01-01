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
use crate::entity::social::EventType;
use crate::actions::catalog::ActionId;
use rayon::prelude::*;

/// Run a single simulation tick
///
/// This is the main entry point that orchestrates all simulation systems:
/// 1. Update needs (decay over time)
/// 2. Run perception (entities observe their surroundings)
/// 3. Generate thoughts (reactions to perceptions)
/// 4. Convert intense thoughts to memories (thoughts about entities become social memories)
/// 5. Decay thoughts (thoughts fade over time)
/// 6. Select actions (decide what to do based on needs, thoughts, values)
/// 7. Execute tasks (progress current tasks, satisfy needs)
/// 8. Regenerate food zones (scarce zones recover over time)
/// 9. Advance tick counter
/// 10. Decay social memories (once per day, after tick advances)
pub fn run_simulation_tick(world: &mut World) {
    update_needs(world);
    let perceptions = run_perception(world);
    generate_thoughts(world, &perceptions);
    convert_thoughts_to_memories(world);
    decay_thoughts(world);
    select_actions(world);
    execute_tasks(world);
    regenerate_food_zones(world);
    world.tick();
    decay_social_memories(world);
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

    // Collect social memories for perception lookup
    let social_memories: Vec<_> = world.humans.social_memories.iter().cloned().collect();

    // Use parallel for large entity counts, sequential for small
    let mut perceptions = if ids.len() >= PARALLEL_THRESHOLD {
        perception_system_parallel(&grid, &positions, &ids, &social_memories, 50.0)
    } else {
        perception_system(&grid, &positions, &ids, &social_memories, 50.0)
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
    social_memories: &[crate::entity::social::SocialMemory],
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
            let observer_memory = &social_memories[i];

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
                        // Look up disposition from social memory
                        let disposition = observer_memory.get_disposition(entity);

                        Some(PerceivedEntity {
                            entity,
                            distance,
                            relationship: crate::simulation::perception::RelationshipType::Unknown,
                            disposition,
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

/// Convert intense thoughts about entities to social memories
///
/// Scans each entity's ThoughtBuffer for thoughts that are:
/// - Intense (intensity >= THOUGHT_MEMORY_THRESHOLD)
/// - About another entity (cause_entity is Some)
///
/// These thoughts are converted to memories via record_encounter().
/// Uses EventType::Observation for now (can be refined later based on thought type).
fn convert_thoughts_to_memories(world: &mut World) {
    const THOUGHT_MEMORY_THRESHOLD: f32 = 0.7;

    let current_tick = world.current_tick;
    let living_indices: Vec<usize> = world.humans.iter_living().collect();

    for &idx in &living_indices {
        // Collect thoughts that should become memories
        let thoughts_to_convert: Vec<_> = world.humans.thoughts[idx]
            .iter()
            .filter(|t| t.intensity >= THOUGHT_MEMORY_THRESHOLD && t.cause_entity.is_some())
            .map(|t| (t.cause_entity.unwrap(), t.intensity))
            .collect();

        // Create memories from intense thoughts about entities
        for (target_id, intensity) in thoughts_to_convert {
            // Use EventType::Observation for now (can be refined later)
            world.humans.social_memories[idx].record_encounter(
                target_id,
                EventType::Observation,
                intensity,
                current_tick,
            );
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

/// Number of simulation ticks per day
///
/// Used for time-based systems like memory decay that should run once per day.
/// At 1000 ticks per day, daily events happen roughly every 1000 ticks.
pub const TICKS_PER_DAY: u64 = 1000;

/// Select actions for entities without current tasks (PARALLEL when beneficial)
///
/// Uses the action selection algorithm to choose appropriate actions
/// based on needs, thoughts, and values.
///
/// Builds a spatial grid to find nearby entities and looks up their dispositions
/// from social memory for disposition-aware action selection.
fn select_actions(world: &mut World) {
    let living_indices: Vec<usize> = world.humans.iter_living().collect();
    let current_tick = world.current_tick;

    // Build spatial grid for nearby entity queries
    let mut grid = SparseHashGrid::new(10.0);
    let positions: Vec<_> = world.humans.positions.iter().cloned().collect();
    let ids: Vec<_> = world.humans.ids.iter().cloned().collect();
    grid.rebuild(ids.iter().cloned().zip(positions.iter().cloned()));

    // Build O(1) lookup map for entity indices
    let id_to_idx: ahash::AHashMap<crate::core::types::EntityId, usize> = ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    let perception_range = 50.0;

    if living_indices.len() >= PARALLEL_THRESHOLD {
        // PARALLEL path for large entity counts
        let selected_actions: Vec<(usize, Option<Task>)> = living_indices
            .par_iter()
            .filter_map(|&i| {
                if world.humans.task_queues[i].current().is_some() {
                    return None;
                }
                let pos = world.humans.positions[i];
                let observer_id = world.humans.ids[i];

                // Check if entity is AT a food zone
                let food_available = world.food_zones.iter()
                    .any(|zone| zone.contains(pos));
                // Find nearest food zone within perception range
                let nearest_food_zone = find_nearest_food_zone(pos, perception_range, &world.food_zones);

                // Query nearby entities and build dispositions list
                let perceived_dispositions: Vec<_> = grid
                    .query_neighbors(pos)
                    .filter(|&e| e != observer_id)
                    .filter_map(|entity| {
                        let entity_idx = *id_to_idx.get(&entity)?;
                        let entity_pos = positions[entity_idx];
                        let distance = pos.distance(&entity_pos);
                        if distance <= perception_range {
                            let disposition = world.humans.social_memories[i].get_disposition(entity);
                            Some((entity, disposition))
                        } else {
                            None
                        }
                    })
                    .collect();

                let ctx = SelectionContext {
                    body: &world.humans.body_states[i],
                    needs: &world.humans.needs[i],
                    thoughts: &world.humans.thoughts[i],
                    values: &world.humans.values[i],
                    has_current_task: false,
                    threat_nearby: world.humans.needs[i].safety > 0.5,
                    food_available,
                    safe_location: world.humans.needs[i].safety < 0.3,
                    entity_nearby: !perceived_dispositions.is_empty(),
                    current_tick,
                    nearest_food_zone,
                    perceived_dispositions,
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
            let observer_id = world.humans.ids[i];

            // Check if entity is AT a food zone
            let food_available = world.food_zones.iter()
                .any(|zone| zone.contains(pos));
            // Find nearest food zone within perception range
            let nearest_food_zone = find_nearest_food_zone(pos, perception_range, &world.food_zones);

            // Query nearby entities and build dispositions list
            let perceived_dispositions: Vec<_> = grid
                .query_neighbors(pos)
                .filter(|&e| e != observer_id)
                .filter_map(|entity| {
                    let entity_idx = *id_to_idx.get(&entity)?;
                    let entity_pos = positions[entity_idx];
                    let distance = pos.distance(&entity_pos);
                    if distance <= perception_range {
                        let disposition = world.humans.social_memories[i].get_disposition(entity);
                        Some((entity, disposition))
                    } else {
                        None
                    }
                })
                .collect();

            let ctx = SelectionContext {
                body: &world.humans.body_states[i],
                needs: &world.humans.needs[i],
                thoughts: &world.humans.thoughts[i],
                values: &world.humans.values[i],
                has_current_task: false,
                threat_nearby: world.humans.needs[i].safety > 0.5,
                food_available,
                safe_location: world.humans.needs[i].safety < 0.3,
                entity_nearby: !perceived_dispositions.is_empty(),
                current_tick,
                nearest_food_zone,
                perceived_dispositions,
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
/// - Eat action: consumes from food zone, satisfies at `consumed * 0.5`
fn execute_tasks(world: &mut World) {
    // Collect indices first to avoid borrow conflicts
    let living_indices: Vec<usize> = world.humans.iter_living().collect();
    for i in living_indices {
        // For Follow action, we need to look up target entity position BEFORE borrowing task_queues
        // This avoids the borrow conflict between task_queues and index_of()
        let follow_target_pos: Option<(crate::core::types::Vec2, bool)> = {
            if let Some(task) = world.humans.task_queues[i].current() {
                if task.action == ActionId::Follow {
                    if let Some(target_id) = task.target_entity {
                        if let Some(target_idx) = world.humans.index_of(target_id) {
                            Some((world.humans.positions[target_idx], true))
                        } else {
                            Some((crate::core::types::Vec2::new(0.0, 0.0), false)) // Target doesn't exist
                        }
                    } else {
                        None // No target entity
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        // Get task info first (action, target_entity, target_position, and whether complete)
        let task_info = world.humans.task_queues[i].current_mut().map(|task| {
            let action = task.action;
            let target_pos = task.target_position;
            let target_entity = task.target_entity;

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
                ActionId::Follow => {
                    // Use pre-computed target position from before we borrowed task_queues
                    if let Some((target_pos, target_exists)) = follow_target_pos {
                        if target_exists {
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
                _ => false, // Not a movement action
            };

            // Progress for non-movement actions
            let is_movement = matches!(action, ActionId::MoveTo | ActionId::Flee | ActionId::Follow);
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
            (action, target_entity, is_complete)
        });

        if let Some((action, target_entity, is_complete)) = task_info {
            // Handle Eat action specially: consume from food zone
            if action == ActionId::Eat {
                let pos = world.humans.positions[i];
                // Find food zone entity is standing in and consume from it
                for zone in &mut world.food_zones {
                    if zone.contains(pos) {
                        let consumed = zone.consume(0.1); // Consume rate per tick
                        if consumed > 0.0 {
                            world.humans.needs[i].satisfy(NeedType::Food, consumed * 0.5);
                        }
                        break;
                    }
                }
            } else {
                // SATISFACTION MULTIPLIER: 0.05
                // Actions apply a fraction of their nominal satisfaction each tick.
                // This accumulates over the task duration to total satisfaction.
                // Without this, entities would fully satisfy needs in one tick.
                for (need, amount) in action.satisfies_needs() {
                    world.humans.needs[i].satisfy(need, amount * 0.05);
                }
            }

            // Complete task if done
            if is_complete {
                // Create social memories if this was a social action with a target
                if let Some(target_id) = target_entity {
                    create_social_memory_from_task(world, i, action, target_id, world.current_tick);
                }

                world.humans.task_queues[i].complete_current();
            }
        }
    }
}

/// Regenerate food for scarce zones
///
/// Scarce zones slowly replenish over time, up to their maximum capacity.
/// This creates dynamic resource availability and strategic considerations.
fn regenerate_food_zones(world: &mut World) {
    for zone in &mut world.food_zones {
        zone.regenerate();
    }
}

/// Decay social memories once per simulation day
///
/// Social memories fade over time, causing salience to decrease.
/// This runs once per day (every TICKS_PER_DAY ticks) to:
/// - Apply decay to all relationship slot memories
/// - Decay encounter buffer salience
/// - Remove near-zero encounters
///
/// Called at the END of run_simulation_tick, AFTER world.tick() has advanced
/// the counter, so we check (current_tick - 1) to see if we just completed a day.
fn decay_social_memories(world: &mut World) {
    // Only decay once per day
    // We check current_tick (which was just incremented) to see if we hit a day boundary
    if world.current_tick % TICKS_PER_DAY != 0 {
        return;
    }

    let current_tick = world.current_tick;
    let living_indices: Vec<usize> = world.humans.iter_living().collect();

    for &idx in &living_indices {
        world.humans.social_memories[idx].apply_decay(current_tick);
    }
}

/// Create social memories when a social action completes
///
/// Maps ActionId to EventType for both actor and target:
/// - Help: Actor remembers AidGiven, Target remembers AidReceived
/// - Attack: Actor remembers HarmGiven, Target remembers HarmReceived
/// - TalkTo/Trade: Both remember Transaction
///
/// Creates memories for BOTH parties - the actor remembers the target
/// and the target remembers the actor.
fn create_social_memory_from_task(
    world: &mut World,
    actor_idx: usize,
    action: ActionId,
    target_id: crate::core::types::EntityId,
    current_tick: u64,
) {
    let actor_id = world.humans.ids[actor_idx];

    // Find target index
    let target_idx = match world.humans.index_of(target_id) {
        Some(idx) => idx,
        None => return, // Target not found (may have died or been removed)
    };

    // Map action to event types for actor and target
    let (actor_event, target_event) = match action {
        ActionId::Help => (EventType::AidGiven, EventType::AidReceived),
        ActionId::TalkTo => (EventType::Transaction, EventType::Transaction),
        ActionId::Trade => (EventType::Transaction, EventType::Transaction),
        ActionId::Attack => (EventType::HarmGiven, EventType::HarmReceived),
        _ => return, // Non-social action with target, no memory created
    };

    // Record for actor (remembers helping/attacking/talking to target)
    world.humans.social_memories[actor_idx].record_encounter(
        target_id,
        actor_event,
        actor_event.base_intensity(),
        current_tick,
    );

    // Record for target (remembers being helped/attacked/talked to by actor)
    world.humans.social_memories[target_idx].record_encounter(
        actor_id,
        target_event,
        target_event.base_intensity(),
        current_tick,
    );
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
    fn test_eating_depletes_scarce_zone() {
        use crate::core::types::Vec2;
        use crate::ecs::world::Abundance;

        let mut world = World::new();
        world.spawn_human("Eater".into());

        // Add scarce food zone with no regeneration (to isolate depletion testing)
        let _zone_id = world.add_food_zone(
            Vec2::new(0.0, 0.0),
            10.0,
            Abundance::Scarce { current: 10.0, max: 100.0, regen: 0.0 },
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
            assert!(*current < 10.0, "Zone should be partially depleted, but current is {}", *current);
        } else {
            panic!("Zone should be Scarce");
        }

        // Entity should be less hungry
        assert!(world.humans.needs[0].food < 0.9, "Entity should be less hungry");
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

    #[test]
    fn test_scarce_zone_regenerates() {
        use crate::core::types::Vec2;
        use crate::ecs::world::Abundance;

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
            assert!(*current > 0.0, "Zone should have regenerated, but current is {}", *current);
        } else {
            panic!("Zone should be Scarce");
        }
    }

    #[test]
    fn test_help_action_creates_memories() {
        use crate::core::types::Vec2;
        use crate::entity::tasks::{TaskPriority, TaskSource};
        use crate::entity::social::Disposition;

        let mut world = World::new();
        let alice = world.spawn_human("Alice".into());
        let bob = world.spawn_human("Bob".into());

        let alice_idx = world.humans.index_of(alice).unwrap();
        let bob_idx = world.humans.index_of(bob).unwrap();

        // Position them together
        world.humans.positions[alice_idx] = Vec2::new(0.0, 0.0);
        world.humans.positions[bob_idx] = Vec2::new(1.0, 0.0);

        // Alice helps Bob
        let task = Task {
            action: ActionId::Help,
            target_entity: Some(bob),
            target_position: None,
            priority: TaskPriority::Normal,
            created_tick: 0,
            progress: 0.0,
            source: TaskSource::Autonomous,
        };
        world.humans.task_queues[alice_idx].push(task);

        // Run enough ticks for Help to complete (duration=30, progress_rate=0.05, so 20 ticks)
        for _ in 0..25 {
            run_simulation_tick(&mut world);
        }

        // Bob should remember being helped by Alice (AidReceived has intensity 0.7, above threshold 0.3)
        let bob_memory = &world.humans.social_memories[bob_idx];
        let disposition = bob_memory.get_disposition(alice);
        assert!(
            disposition == Disposition::Friendly || disposition == Disposition::Favorable,
            "Bob should have friendly/favorable disposition toward Alice, got {:?}",
            disposition
        );

        // Alice should remember helping Bob (AidGiven has intensity 0.5, above threshold 0.3)
        let alice_memory = &world.humans.social_memories[alice_idx];
        let alice_disposition = alice_memory.get_disposition(bob);
        assert!(
            alice_disposition == Disposition::Friendly || alice_disposition == Disposition::Favorable,
            "Alice should have friendly/favorable disposition toward Bob, got {:?}",
            alice_disposition
        );
    }

    #[test]
    fn test_intense_thoughts_create_memories() {
        use crate::core::types::Vec2;
        use crate::entity::thoughts::{Thought, CauseType};

        let mut world = World::new();
        let alice = world.spawn_human("Alice".into());
        let bob = world.spawn_human("Bob".into());

        let alice_idx = world.humans.index_of(alice).unwrap();

        // Position them somewhere (doesn't matter for this test)
        world.humans.positions[alice_idx] = Vec2::new(0.0, 0.0);

        // Manually add an intense thought about Bob
        let mut thought = Thought::new(
            Valence::Positive,
            0.9, // Very intense - above THOUGHT_MEMORY_THRESHOLD (0.7)
            "observation",
            "Bob did something amazing",
            CauseType::Entity,
            0,
        );
        thought.cause_entity = Some(bob);
        world.humans.thoughts[alice_idx].add(thought);

        // Run tick to process thoughts (convert_thoughts_to_memories happens after generate_thoughts)
        run_simulation_tick(&mut world);

        // Alice should remember Bob
        let alice_memory = &world.humans.social_memories[alice_idx];
        assert!(
            alice_memory.find_slot(bob).is_some(),
            "Alice should have a memory slot for Bob after intense thought about him"
        );
    }

    #[test]
    fn test_weak_thoughts_do_not_create_memories() {
        use crate::core::types::Vec2;
        use crate::entity::thoughts::{Thought, CauseType};

        let mut world = World::new();
        let alice = world.spawn_human("Alice".into());
        let bob = world.spawn_human("Bob".into());

        let alice_idx = world.humans.index_of(alice).unwrap();

        // Position them somewhere
        world.humans.positions[alice_idx] = Vec2::new(0.0, 0.0);

        // Add a weak thought about Bob (below THOUGHT_MEMORY_THRESHOLD of 0.7)
        let mut thought = Thought::new(
            Valence::Positive,
            0.5, // Below threshold
            "observation",
            "Bob walked by",
            CauseType::Entity,
            0,
        );
        thought.cause_entity = Some(bob);
        world.humans.thoughts[alice_idx].add(thought);

        // Run tick
        run_simulation_tick(&mut world);

        // Alice should NOT remember Bob (thought wasn't intense enough)
        let alice_memory = &world.humans.social_memories[alice_idx];
        assert!(
            alice_memory.find_slot(bob).is_none(),
            "Alice should NOT have a memory slot for Bob after weak thought"
        );
    }

    #[test]
    fn test_memory_decay_happens_daily() {
        let mut world = World::new();
        let alice = world.spawn_human("Alice".into());
        let bob = world.spawn_human("Bob".into());

        let alice_idx = world.humans.index_of(alice).unwrap();

        // Create a memory directly via record_encounter
        world.humans.social_memories[alice_idx].record_encounter(
            bob,
            EventType::AidReceived,
            0.8, // High intensity, above threshold
            0,   // Created at tick 0
        );

        // Get initial salience (should be 1.0)
        let initial_salience = world.humans.social_memories[alice_idx]
            .find_slot(bob)
            .expect("Alice should have a memory slot for Bob")
            .memories[0]
            .salience;
        assert!(
            (initial_salience - 1.0).abs() < 0.01,
            "Initial salience should be 1.0, got {}",
            initial_salience
        );

        // Run exactly TICKS_PER_DAY ticks (1000)
        // After tick 1000, decay_social_memories will run because current_tick % 1000 == 0
        for _ in 0..TICKS_PER_DAY {
            run_simulation_tick(&mut world);
        }

        // Verify we're at the right tick
        assert_eq!(world.current_tick, TICKS_PER_DAY, "Should be at tick 1000");

        // Salience should have decayed (decay runs at tick 1000)
        let final_salience = world.humans.social_memories[alice_idx]
            .find_slot(bob)
            .expect("Alice should still have a memory slot for Bob")
            .memories[0]
            .salience;

        assert!(
            final_salience < initial_salience,
            "Salience should have decayed after one day. Initial: {}, Final: {}",
            initial_salience,
            final_salience
        );

        // Verify decay is reasonable (should be around 98% of original with 2% decay)
        assert!(
            final_salience > 0.9,
            "Salience should still be > 0.9 after one day, got {}",
            final_salience
        );
    }

    #[test]
    fn test_follow_tracks_moving_target() {
        use crate::core::types::Vec2;
        use crate::entity::tasks::{Task, TaskPriority};
        use crate::actions::catalog::ActionId;

        let mut world = World::new();

        // Spawn follower at origin
        let follower_id = world.spawn_human("Follower".into());

        // Spawn target at (10, 0)
        let target_id = world.spawn_human("Target".into());

        let follower_idx = world.humans.index_of(follower_id).unwrap();
        let target_idx = world.humans.index_of(target_id).unwrap();

        // Position them
        world.humans.positions[follower_idx] = Vec2::new(0.0, 0.0);
        world.humans.positions[target_idx] = Vec2::new(10.0, 0.0);

        // Give follower a Follow task
        let task = Task::new(ActionId::Follow, TaskPriority::Normal, 0)
            .with_entity(target_id);
        world.humans.task_queues[follower_idx].push(task);

        // Run a tick
        run_simulation_tick(&mut world);

        // Follower should have moved toward target
        let follower_pos = world.humans.positions[follower_idx];
        assert!(follower_pos.x > 0.0, "Follower should move toward target");

        // Move target
        world.humans.positions[target_idx] = Vec2::new(10.0, 10.0);

        // Run another tick
        run_simulation_tick(&mut world);

        // Follower should now be moving toward new position
        let follower_pos = world.humans.positions[follower_idx];
        assert!(follower_pos.y > 0.0 || follower_pos.x > 2.0, "Follower should track moving target");
    }
}
