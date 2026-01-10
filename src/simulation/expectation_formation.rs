//! Expectation formation from observed actions
//!
//! When entities observe other entities performing actions, they form expectations
//! about future behavior. These expectations are stored in relationship slots and
//! used for violation detection.

use crate::actions::catalog::{ActionCategory, ActionId};
use crate::ecs::world::World;
use crate::entity::social::{
    BehaviorPattern, EventType, PatternType, RecentEvent, ServiceType, TraitIndicator,
};
use crate::simulation::perception::Perception;

/// Record an observation and form expectations
///
/// When an observer sees another entity performing an action:
/// 1. Infer behavioral patterns from the action
/// 2. Create or update the relationship slot via record_encounter
/// 3. Add expectations to the slot
/// 4. Record in observer's event buffer
pub fn record_observation(
    world: &mut World,
    observer_idx: usize,
    observed_idx: usize,
    action: ActionId,
    current_tick: u64,
) {
    let observed_id = world.humans.ids[observed_idx];

    // Infer patterns from action
    let patterns = infer_patterns_from_action(action, current_tick);

    if patterns.is_empty() {
        return;
    }

    // Get or create relationship slot via record_encounter
    world.humans.social_memories[observer_idx].record_encounter(
        observed_id,
        EventType::Observation,
        0.3, // Low intensity for just observing
        current_tick,
    );

    // Add expectations to the slot
    if let Some(slot) = world.humans.social_memories[observer_idx].find_slot_mut(observed_id) {
        for pattern in patterns {
            slot.add_expectation(pattern);
        }
    }

    // Also record in observer's event buffer
    world.humans.event_buffers[observer_idx].push(RecentEvent {
        event_type: EventType::Observation,
        actor: observed_id,
        tick: current_tick,
    });
}

/// Infer behavioral patterns from an observed action
///
/// Uses ServiceType::from_action() for ProvidesWhenAsked patterns,
/// TraitIndicator::from_action() for BehavesWithTrait patterns,
/// and creates RespondsToEvent patterns for Combat/Flee actions.
pub fn infer_patterns_from_action(action: ActionId, tick: u64) -> Vec<BehaviorPattern> {
    let mut patterns = Vec::new();

    // Service patterns
    if let Some(service) = ServiceType::from_action(action) {
        patterns.push(BehaviorPattern::new(
            PatternType::ProvidesWhenAsked {
                service_type: service,
            },
            tick,
        ));
    }

    // Trait patterns
    if let Some(trait_ind) = TraitIndicator::from_action(action) {
        patterns.push(BehaviorPattern::new(
            PatternType::BehavesWithTrait {
                trait_indicator: trait_ind,
            },
            tick,
        ));
    }

    // RespondsToEvent patterns based on action category
    match action.category() {
        ActionCategory::Combat => {
            patterns.push(BehaviorPattern::new(
                PatternType::RespondsToEvent {
                    event_type: EventType::HarmReceived,
                    typical_response: ActionCategory::Combat,
                },
                tick,
            ));
        }
        ActionCategory::Movement if action == ActionId::Flee => {
            patterns.push(BehaviorPattern::new(
                PatternType::RespondsToEvent {
                    event_type: EventType::HarmReceived,
                    typical_response: ActionCategory::Movement,
                },
                tick,
            ));
        }
        _ => {}
    }

    patterns
}

/// Process all observations from perception data
///
/// Iterates through perceptions and forms expectations based on
/// the current actions of observed entities.
pub fn process_observations(world: &mut World, perceptions: &[Perception]) {
    let current_tick = world.current_tick;

    // Build O(1) lookup map once instead of O(N) index_of per perceived entity
    let id_to_idx: ahash::AHashMap<crate::core::types::EntityId, usize> = world
        .humans
        .ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i))
        .collect();

    for (observer_idx, perception) in perceptions.iter().enumerate() {
        for perceived in &perception.perceived_entities {
            // Get observed entity's current action (O(1) lookup)
            if let Some(&observed_idx) = id_to_idx.get(&perceived.entity) {
                if let Some(task) = world.humans.task_queues[observed_idx].current() {
                    record_observation(
                        world,
                        observer_idx,
                        observed_idx,
                        task.action,
                        current_tick,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Vec2;
    use crate::entity::tasks::{Task, TaskPriority};

    #[test]
    fn test_observation_forms_expectation() {
        let mut world = World::new();

        // Spawn observer and actor
        let observer_id = world.spawn_human("Observer".into());
        let actor_id = world.spawn_human("Actor".into());

        let observer_idx = world.humans.index_of(observer_id).unwrap();
        let actor_idx = world.humans.index_of(actor_id).unwrap();

        // Position them near each other
        world.humans.positions[observer_idx] = Vec2::new(0.0, 0.0);
        world.humans.positions[actor_idx] = Vec2::new(5.0, 0.0);

        // Actor is doing Help action
        let task = Task::new(ActionId::Help, TaskPriority::Normal, 0).with_entity(observer_id);
        world.humans.task_queues[actor_idx].push(task);

        // Observer sees actor
        record_observation(&mut world, observer_idx, actor_idx, ActionId::Help, 100);

        // Observer should form expectation about actor
        let slot = world.humans.social_memories[observer_idx].find_slot(actor_id);
        assert!(slot.is_some(), "Observer should have slot for actor");

        let slot = slot.unwrap();
        let expectation = slot.find_expectation(&PatternType::BehavesWithTrait {
            trait_indicator: TraitIndicator::Generous,
        });
        assert!(
            expectation.is_some(),
            "Observer should expect actor to be generous"
        );
    }

    #[test]
    fn test_infer_patterns_help_action() {
        let patterns = infer_patterns_from_action(ActionId::Help, 100);

        // Help should produce both Helping service and Generous trait
        assert!(patterns.iter().any(|p| matches!(
            &p.pattern_type,
            PatternType::ProvidesWhenAsked {
                service_type: ServiceType::Helping
            }
        )));
        assert!(patterns.iter().any(|p| matches!(
            &p.pattern_type,
            PatternType::BehavesWithTrait {
                trait_indicator: TraitIndicator::Generous
            }
        )));
    }

    #[test]
    fn test_infer_patterns_attack_action() {
        let patterns = infer_patterns_from_action(ActionId::Attack, 100);

        // Attack should produce Aggressive trait and RespondsToEvent
        assert!(patterns.iter().any(|p| matches!(
            &p.pattern_type,
            PatternType::BehavesWithTrait {
                trait_indicator: TraitIndicator::Aggressive
            }
        )));
        assert!(patterns.iter().any(|p| matches!(
            &p.pattern_type,
            PatternType::RespondsToEvent {
                event_type: EventType::HarmReceived,
                typical_response: ActionCategory::Combat,
            }
        )));
    }

    #[test]
    fn test_infer_patterns_flee_action() {
        let patterns = infer_patterns_from_action(ActionId::Flee, 100);

        // Flee should produce Peaceful trait and RespondsToEvent with Movement
        assert!(patterns.iter().any(|p| matches!(
            &p.pattern_type,
            PatternType::BehavesWithTrait {
                trait_indicator: TraitIndicator::Peaceful
            }
        )));
        assert!(patterns.iter().any(|p| matches!(
            &p.pattern_type,
            PatternType::RespondsToEvent {
                event_type: EventType::HarmReceived,
                typical_response: ActionCategory::Movement,
            }
        )));
    }

    #[test]
    fn test_infer_patterns_craft_action() {
        let patterns = infer_patterns_from_action(ActionId::Craft, 100);

        // Craft should produce Crafting service
        assert!(patterns.iter().any(|p| matches!(
            &p.pattern_type,
            PatternType::ProvidesWhenAsked {
                service_type: ServiceType::Crafting
            }
        )));
    }

    #[test]
    fn test_infer_patterns_moveto_action() {
        let patterns = infer_patterns_from_action(ActionId::MoveTo, 100);

        // MoveTo shouldn't produce any patterns
        assert!(patterns.is_empty(), "MoveTo should not produce patterns");
    }

    #[test]
    fn test_observation_adds_to_event_buffer() {
        let mut world = World::new();

        let observer_id = world.spawn_human("Observer".into());
        let actor_id = world.spawn_human("Actor".into());

        let observer_idx = world.humans.index_of(observer_id).unwrap();
        let actor_idx = world.humans.index_of(actor_id).unwrap();

        let initial_buffer_len = world.humans.event_buffers[observer_idx].len();

        record_observation(&mut world, observer_idx, actor_idx, ActionId::Help, 100);

        assert!(
            world.humans.event_buffers[observer_idx].len() > initial_buffer_len,
            "Event buffer should grow after observation"
        );
    }

    #[test]
    fn test_repeated_observation_strengthens_expectation() {
        let mut world = World::new();

        let observer_id = world.spawn_human("Observer".into());
        let actor_id = world.spawn_human("Actor".into());

        let observer_idx = world.humans.index_of(observer_id).unwrap();
        let actor_idx = world.humans.index_of(actor_id).unwrap();

        // First observation
        record_observation(&mut world, observer_idx, actor_idx, ActionId::Craft, 100);

        let initial_obs_count = {
            let slot = world.humans.social_memories[observer_idx]
                .find_slot(actor_id)
                .unwrap();
            let exp = slot
                .find_expectation(&PatternType::ProvidesWhenAsked {
                    service_type: ServiceType::Crafting,
                })
                .unwrap();
            exp.observation_count
        };

        // Second observation
        record_observation(&mut world, observer_idx, actor_idx, ActionId::Craft, 200);

        let final_obs_count = {
            let slot = world.humans.social_memories[observer_idx]
                .find_slot(actor_id)
                .unwrap();
            let exp = slot
                .find_expectation(&PatternType::ProvidesWhenAsked {
                    service_type: ServiceType::Crafting,
                })
                .unwrap();
            exp.observation_count
        };

        assert!(
            final_obs_count > initial_obs_count,
            "Repeated observation should increase observation count"
        );
    }

    #[test]
    fn test_process_observations() {
        use crate::entity::social::Disposition;
        use crate::simulation::perception::{PerceivedEntity, RelationshipType};

        let mut world = World::new();

        let observer_id = world.spawn_human("Observer".into());
        let actor_id = world.spawn_human("Actor".into());

        let observer_idx = world.humans.index_of(observer_id).unwrap();
        let actor_idx = world.humans.index_of(actor_id).unwrap();

        // Position them
        world.humans.positions[observer_idx] = Vec2::new(0.0, 0.0);
        world.humans.positions[actor_idx] = Vec2::new(5.0, 0.0);

        // Give actor a task
        let task = Task::new(ActionId::Trade, TaskPriority::Normal, 0);
        world.humans.task_queues[actor_idx].push(task);

        // Create perception data
        let perceptions = vec![
            Perception {
                observer: observer_id,
                perceived_entities: vec![PerceivedEntity {
                    entity: actor_id,
                    distance: 5.0,
                    relationship: RelationshipType::Unknown,
                    disposition: Disposition::Unknown,
                    threat_level: 0.0,
                    notable_features: vec![],
                }],
                perceived_objects: vec![],
                perceived_events: vec![],
                nearest_food_zone: None,
                nearest_building_site: None,
            },
            Perception {
                observer: actor_id,
                perceived_entities: vec![PerceivedEntity {
                    entity: observer_id,
                    distance: 5.0,
                    relationship: RelationshipType::Unknown,
                    disposition: Disposition::Unknown,
                    threat_level: 0.0,
                    notable_features: vec![],
                }],
                perceived_objects: vec![],
                perceived_events: vec![],
                nearest_food_zone: None,
                nearest_building_site: None,
            },
        ];

        process_observations(&mut world, &perceptions);

        // Observer should now have expectations about actor (Trading service)
        let slot = world.humans.social_memories[observer_idx].find_slot(actor_id);
        assert!(
            slot.is_some(),
            "Observer should have slot for actor after process_observations"
        );

        let slot = slot.unwrap();
        let has_trading = slot.find_expectation(&PatternType::ProvidesWhenAsked {
            service_type: ServiceType::Trading,
        });
        assert!(
            has_trading.is_some(),
            "Observer should expect actor to trade"
        );
    }
}
