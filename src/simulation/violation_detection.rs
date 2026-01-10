//! Violation detection for expectation-based social dynamics
//!
//! Detects when observed behavior violates expectations and generates thoughts.
//! When an entity observes another performing an action that contradicts their
//! expectations, they experience negative thoughts (betrayal, disappointment, surprise).

use crate::actions::catalog::ActionId;
use crate::ecs::world::World;
use crate::entity::social::{BehaviorPattern, PatternType, TraitIndicator, SALIENCE_THRESHOLD};
use crate::entity::thoughts::{CauseType, Thought, Valence};
use crate::simulation::perception::Perception;

/// Types of violations that can occur
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ViolationType {
    /// Action trait contradicts expected trait (e.g., expected Generous, observed Aggressive)
    TraitContradiction,
    /// Service was expected but refused (e.g., expected crafter doesn't craft)
    ServiceRefused,
    /// Response to event differs from expected (e.g., expected Combat response, got Movement)
    UnexpectedResponse,
}

/// Check if an observed action violates any expectations
///
/// Gets expectations about the observed entity, checks each pattern for violations,
/// updates pattern stats, and generates negative thoughts for violations.
///
/// # Arguments
/// * `world` - Mutable world state
/// * `observer_idx` - Index of the observing entity
/// * `observed_idx` - Index of the entity being observed
/// * `observed_action` - The action the observed entity is performing
/// * `current_tick` - Current simulation tick
pub fn check_violations(
    world: &mut World,
    observer_idx: usize,
    observed_idx: usize,
    observed_action: ActionId,
    current_tick: u64,
) {
    let observed_id = world.humans.ids[observed_idx];

    // Get expectations about this entity
    let violations = {
        let slot = match world.humans.social_memories[observer_idx].find_slot(observed_id) {
            Some(s) => s,
            None => return,
        };

        let mut violations = Vec::new();

        for pattern in &slot.expectations {
            if pattern.salience < SALIENCE_THRESHOLD {
                continue;
            }

            if let Some(violation) = check_pattern_violation(pattern, observed_action) {
                violations.push((pattern.pattern_type.clone(), pattern.confidence, violation));
            }
        }

        violations
    };

    // Process violations
    for (pattern_type, confidence, violation_type) in violations {
        // Update pattern stats
        if let Some(slot) = world.humans.social_memories[observer_idx].find_slot_mut(observed_id) {
            if let Some(pattern) = slot.find_expectation_mut(&pattern_type) {
                pattern.record_violation(current_tick);
            }
        }

        // Generate violation thought
        // Intensity based on pattern confidence * 0.8 (high confidence = high disappointment)
        let intensity = (confidence * 0.8).min(1.0);
        let concept = match violation_type {
            ViolationType::TraitContradiction => "BETRAYAL",
            ViolationType::ServiceRefused => "DISAPPOINTMENT",
            ViolationType::UnexpectedResponse => "SURPRISE",
        };

        let mut thought = Thought::new(
            Valence::Negative,
            intensity,
            concept,
            format!("{} violated expectation", concept.to_lowercase()),
            CauseType::Entity,
            current_tick,
        );
        thought.cause_entity = Some(observed_id);

        world.humans.thoughts[observer_idx].add(thought);
    }
}

/// Check if an action violates a specific pattern
///
/// # Arguments
/// * `pattern` - The behavioral pattern to check against
/// * `action` - The observed action
///
/// # Returns
/// * `Some(ViolationType)` if the action violates the pattern
/// * `None` if no violation
pub fn check_pattern_violation(
    pattern: &BehaviorPattern,
    action: ActionId,
) -> Option<ViolationType> {
    match &pattern.pattern_type {
        PatternType::BehavesWithTrait { trait_indicator } => {
            // Check if action contradicts expected trait
            if let Some(action_trait) = TraitIndicator::from_action(action) {
                if action_trait == trait_indicator.opposite() {
                    return Some(ViolationType::TraitContradiction);
                }
            }
            None
        }

        PatternType::ProvidesWhenAsked { service_type: _ } => {
            // This would be checked when a service request is refused
            // For now, just return None (service refusal detection requires
            // tracking requests, not just observations)
            None
        }

        PatternType::RespondsToEvent {
            typical_response, ..
        } => {
            // Check if action category matches expected response
            if action.category() != *typical_response {
                return Some(ViolationType::UnexpectedResponse);
            }
            None
        }

        PatternType::LocationDuring { .. } => {
            // Location checking requires more context (position, time)
            // Not checked via action observation
            None
        }
    }
}

/// Process violation checks for all perceptions
///
/// Iterates through perceptions and checks for violations based on
/// the current actions of observed entities.
pub fn process_violations(world: &mut World, perceptions: &[Perception]) {
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
            // O(1) lookup instead of O(N) index_of
            if let Some(&observed_idx) = id_to_idx.get(&perceived.entity) {
                if let Some(task) = world.humans.task_queues[observed_idx].current() {
                    check_violations(world, observer_idx, observed_idx, task.action, current_tick);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Vec2;
    use crate::entity::social::{BehaviorPattern, PatternType};
    use crate::entity::tasks::{Task, TaskPriority};
    use crate::simulation::expectation_formation::record_observation;

    #[test]
    fn test_violation_generates_thought() {
        let mut world = World::new();

        // Setup: Observer expects Actor to be Peaceful (via Flee action)
        let observer_id = world.spawn_human("Observer".into());
        let actor_id = world.spawn_human("Actor".into());

        let observer_idx = world.humans.index_of(observer_id).unwrap();
        let actor_idx = world.humans.index_of(actor_id).unwrap();

        // Position them near each other
        world.humans.positions[observer_idx] = Vec2::new(0.0, 0.0);
        world.humans.positions[actor_idx] = Vec2::new(5.0, 0.0);

        // Build expectation (observe Flee action 5 times - this creates Peaceful trait expectation)
        // Peaceful.opposite() = Aggressive, and Attack produces Aggressive
        for i in 0..5 {
            record_observation(&mut world, observer_idx, actor_idx, ActionId::Flee, i * 100);
        }

        // Verify expectation exists with decent confidence
        let slot = world.humans.social_memories[observer_idx]
            .find_slot(actor_id)
            .unwrap();
        let exp = slot
            .find_expectation(&PatternType::BehavesWithTrait {
                trait_indicator: TraitIndicator::Peaceful,
            })
            .unwrap();
        assert!(
            exp.confidence > 0.5,
            "Confidence should be > 0.5, got {}",
            exp.confidence
        );

        let initial_thoughts = world.humans.thoughts[observer_idx].iter().count();

        // Now actor does something Aggressive (Attack - violates Peaceful expectation)
        // Attack -> Aggressive, Peaceful.opposite() = Aggressive, so this is a violation
        check_violations(&mut world, observer_idx, actor_idx, ActionId::Attack, 500);

        // Should have generated a negative thought
        let final_thoughts = world.humans.thoughts[observer_idx].iter().count();
        assert!(
            final_thoughts > initial_thoughts,
            "Should have generated a thought"
        );

        let thought = world.humans.thoughts[observer_idx].strongest();
        assert!(thought.is_some(), "Should have a strongest thought");
        assert_eq!(
            thought.unwrap().valence,
            Valence::Negative,
            "Thought should be negative"
        );
    }

    #[test]
    fn test_check_pattern_violation_trait_contradiction() {
        // Test that Aggressive action violates Peaceful expectation
        // Note: Generous.opposite() = Stingy, not Aggressive, so we use Peaceful/Aggressive
        let peaceful_pattern = BehaviorPattern::new(
            PatternType::BehavesWithTrait {
                trait_indicator: TraitIndicator::Peaceful,
            },
            0,
        );

        // Attack -> Aggressive, Peaceful.opposite() = Aggressive
        let violation = check_pattern_violation(&peaceful_pattern, ActionId::Attack);
        assert_eq!(violation, Some(ViolationType::TraitContradiction));

        // Flee -> Peaceful, Peaceful.opposite() = Aggressive
        // Peaceful is not opposite of Peaceful, so no violation
        let no_violation = check_pattern_violation(&peaceful_pattern, ActionId::Flee);
        assert_eq!(no_violation, None);
    }

    #[test]
    fn test_check_pattern_violation_aggressive_vs_peaceful() {
        // Expect Aggressive, observe Peaceful (Flee)
        let pattern = BehaviorPattern::new(
            PatternType::BehavesWithTrait {
                trait_indicator: TraitIndicator::Aggressive,
            },
            0,
        );

        // Flee -> Peaceful, Aggressive.opposite() = Peaceful
        let violation = check_pattern_violation(&pattern, ActionId::Flee);
        assert_eq!(violation, Some(ViolationType::TraitContradiction));
    }

    #[test]
    fn test_check_pattern_violation_responds_to_event() {
        use crate::actions::catalog::ActionCategory;
        use crate::entity::social::EventType;

        // Expect Combat response
        let pattern = BehaviorPattern::new(
            PatternType::RespondsToEvent {
                event_type: EventType::HarmReceived,
                typical_response: ActionCategory::Combat,
            },
            0,
        );

        // Flee is Movement category, not Combat
        let violation = check_pattern_violation(&pattern, ActionId::Flee);
        assert_eq!(violation, Some(ViolationType::UnexpectedResponse));

        // Attack is Combat category - no violation
        let no_violation = check_pattern_violation(&pattern, ActionId::Attack);
        assert_eq!(no_violation, None);
    }

    #[test]
    fn test_violation_updates_pattern_stats() {
        let mut world = World::new();

        let observer_id = world.spawn_human("Observer".into());
        let actor_id = world.spawn_human("Actor".into());

        let observer_idx = world.humans.index_of(observer_id).unwrap();
        let actor_idx = world.humans.index_of(actor_id).unwrap();

        world.humans.positions[observer_idx] = Vec2::new(0.0, 0.0);
        world.humans.positions[actor_idx] = Vec2::new(5.0, 0.0);

        // Build expectation: Peaceful
        for i in 0..5 {
            record_observation(&mut world, observer_idx, actor_idx, ActionId::Flee, i * 100);
        }

        // Get initial violation count
        let initial_violations = {
            let slot = world.humans.social_memories[observer_idx]
                .find_slot(actor_id)
                .unwrap();
            let exp = slot
                .find_expectation(&PatternType::BehavesWithTrait {
                    trait_indicator: TraitIndicator::Peaceful,
                })
                .unwrap();
            exp.violation_count
        };

        // Actor does Attack (Aggressive - violates Peaceful)
        check_violations(&mut world, observer_idx, actor_idx, ActionId::Attack, 600);

        // Violation count should increase
        let final_violations = {
            let slot = world.humans.social_memories[observer_idx]
                .find_slot(actor_id)
                .unwrap();
            let exp = slot
                .find_expectation(&PatternType::BehavesWithTrait {
                    trait_indicator: TraitIndicator::Peaceful,
                })
                .unwrap();
            exp.violation_count
        };

        assert!(
            final_violations > initial_violations,
            "Violation count should increase from {} to > {}",
            initial_violations,
            initial_violations
        );
    }

    #[test]
    fn test_no_violation_for_unknown_entity() {
        let mut world = World::new();

        let observer_id = world.spawn_human("Observer".into());
        let stranger_id = world.spawn_human("Stranger".into());

        let observer_idx = world.humans.index_of(observer_id).unwrap();
        let stranger_idx = world.humans.index_of(stranger_id).unwrap();

        world.humans.positions[observer_idx] = Vec2::new(0.0, 0.0);
        world.humans.positions[stranger_idx] = Vec2::new(5.0, 0.0);

        // No expectations about stranger (never observed)
        let initial_thoughts = world.humans.thoughts[observer_idx].iter().count();

        // Check for violations (should find none since no relationship)
        check_violations(
            &mut world,
            observer_idx,
            stranger_idx,
            ActionId::Attack,
            100,
        );

        let final_thoughts = world.humans.thoughts[observer_idx].iter().count();
        assert_eq!(
            final_thoughts, initial_thoughts,
            "Should not generate thought for unknown entity"
        );
    }

    #[test]
    fn test_low_salience_pattern_not_checked() {
        let mut world = World::new();

        let observer_id = world.spawn_human("Observer".into());
        let actor_id = world.spawn_human("Actor".into());

        let observer_idx = world.humans.index_of(observer_id).unwrap();
        let actor_idx = world.humans.index_of(actor_id).unwrap();

        world.humans.positions[observer_idx] = Vec2::new(0.0, 0.0);
        world.humans.positions[actor_idx] = Vec2::new(5.0, 0.0);

        // Build expectation (Flee creates Peaceful AND RespondsToEvent patterns)
        record_observation(&mut world, observer_idx, actor_idx, ActionId::Flee, 100);

        // Manually decay salience below threshold for ALL patterns
        {
            let slot = world.humans.social_memories[observer_idx]
                .find_slot_mut(actor_id)
                .unwrap();
            for exp in &mut slot.expectations {
                exp.salience = 0.05; // Below SALIENCE_THRESHOLD (0.1)
            }
        }

        let initial_thoughts = world.humans.thoughts[observer_idx].iter().count();

        // This should violate both patterns, but all are below salience threshold
        check_violations(&mut world, observer_idx, actor_idx, ActionId::Attack, 200);

        let final_thoughts = world.humans.thoughts[observer_idx].iter().count();
        assert_eq!(
            final_thoughts, initial_thoughts,
            "Should not check low salience patterns"
        );
    }

    #[test]
    fn test_thought_intensity_based_on_confidence() {
        let mut world = World::new();

        let observer_id = world.spawn_human("Observer".into());
        let actor_id = world.spawn_human("Actor".into());

        let observer_idx = world.humans.index_of(observer_id).unwrap();
        let actor_idx = world.humans.index_of(actor_id).unwrap();

        world.humans.positions[observer_idx] = Vec2::new(0.0, 0.0);
        world.humans.positions[actor_idx] = Vec2::new(5.0, 0.0);

        // Build high-confidence expectation (many observations)
        for i in 0..20 {
            record_observation(&mut world, observer_idx, actor_idx, ActionId::Flee, i * 100);
        }

        let confidence = {
            let slot = world.humans.social_memories[observer_idx]
                .find_slot(actor_id)
                .unwrap();
            slot.find_expectation(&PatternType::BehavesWithTrait {
                trait_indicator: TraitIndicator::Peaceful,
            })
            .unwrap()
            .confidence
        };

        // Violation
        check_violations(&mut world, observer_idx, actor_idx, ActionId::Attack, 2100);

        // Check thought intensity
        let thought = world.humans.thoughts[observer_idx].strongest().unwrap();
        let expected_intensity = (confidence * 0.8).min(1.0);

        assert!(
            (thought.intensity - expected_intensity).abs() < 0.01,
            "Intensity should be {} (confidence {} * 0.8), got {}",
            expected_intensity,
            confidence,
            thought.intensity
        );
    }

    #[test]
    fn test_thought_concept_based_on_violation_type() {
        let mut world = World::new();

        let observer_id = world.spawn_human("Observer".into());
        let actor_id = world.spawn_human("Actor".into());

        let observer_idx = world.humans.index_of(observer_id).unwrap();
        let actor_idx = world.humans.index_of(actor_id).unwrap();

        world.humans.positions[observer_idx] = Vec2::new(0.0, 0.0);
        world.humans.positions[actor_idx] = Vec2::new(5.0, 0.0);

        // Build expectation for Peaceful trait (NOT RespondsToEvent which also gets created)
        // Use only Flee which creates both Peaceful and RespondsToEvent with Movement
        // Attack will trigger both TraitContradiction (BETRAYAL) and UnexpectedResponse (SURPRISE)
        // The first violation found will be TraitContradiction since BehavesWithTrait is checked first
        for i in 0..5 {
            record_observation(&mut world, observer_idx, actor_idx, ActionId::Flee, i * 100);
        }

        // Clear any thoughts from observations
        world.humans.thoughts[observer_idx] = crate::entity::thoughts::ThoughtBuffer::new();

        // Violate with Attack (TraitContradiction -> BETRAYAL)
        // Attack triggers Aggressive trait, Peaceful.opposite() = Aggressive -> BETRAYAL
        check_violations(&mut world, observer_idx, actor_idx, ActionId::Attack, 600);

        // Check that BETRAYAL thought exists (there may also be SURPRISE from RespondsToEvent)
        let has_betrayal = world.humans.thoughts[observer_idx]
            .iter()
            .any(|t| t.concept_category == "BETRAYAL");
        assert!(
            has_betrayal,
            "TraitContradiction should produce BETRAYAL thought"
        );
    }

    #[test]
    fn test_thought_has_cause_entity() {
        let mut world = World::new();

        let observer_id = world.spawn_human("Observer".into());
        let actor_id = world.spawn_human("Actor".into());

        let observer_idx = world.humans.index_of(observer_id).unwrap();
        let actor_idx = world.humans.index_of(actor_id).unwrap();

        world.humans.positions[observer_idx] = Vec2::new(0.0, 0.0);
        world.humans.positions[actor_idx] = Vec2::new(5.0, 0.0);

        // Build expectation
        for i in 0..5 {
            record_observation(&mut world, observer_idx, actor_idx, ActionId::Flee, i * 100);
        }

        // Violate
        check_violations(&mut world, observer_idx, actor_idx, ActionId::Attack, 600);

        let thought = world.humans.thoughts[observer_idx].strongest().unwrap();
        assert_eq!(
            thought.cause_entity,
            Some(actor_id),
            "Thought should reference the violating entity"
        );
    }

    #[test]
    fn test_process_violations_iterates_perceptions() {
        use crate::entity::social::Disposition;
        use crate::simulation::perception::{PerceivedEntity, Perception, RelationshipType};

        let mut world = World::new();

        let observer_id = world.spawn_human("Observer".into());
        let actor_id = world.spawn_human("Actor".into());

        let observer_idx = world.humans.index_of(observer_id).unwrap();
        let actor_idx = world.humans.index_of(actor_id).unwrap();

        world.humans.positions[observer_idx] = Vec2::new(0.0, 0.0);
        world.humans.positions[actor_idx] = Vec2::new(5.0, 0.0);

        // Build expectation
        for i in 0..5 {
            record_observation(&mut world, observer_idx, actor_idx, ActionId::Flee, i * 100);
        }

        // Give actor a violating task
        let task = Task::new(ActionId::Attack, TaskPriority::Normal, 600);
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
                perceived_entities: vec![],
                perceived_objects: vec![],
                perceived_events: vec![],
                nearest_food_zone: None,
                nearest_building_site: None,
            },
        ];

        let initial_thoughts = world.humans.thoughts[observer_idx].iter().count();

        // Process violations
        world.current_tick = 600;
        process_violations(&mut world, &perceptions);

        let final_thoughts = world.humans.thoughts[observer_idx].iter().count();
        assert!(
            final_thoughts > initial_thoughts,
            "process_violations should detect and generate thoughts"
        );
    }
}
