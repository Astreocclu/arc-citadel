//! Behavioral expectations for expectation-based social dynamics
//!
//! BehaviorPattern represents what an entity expects from another entity's behavior.
//! PatternType categorizes the different kinds of behavioral expectations.

use serde::{Deserialize, Serialize};
use crate::core::types::EntityId;
use crate::core::calendar::TimePeriod;
use crate::actions::catalog::ActionCategory;
use super::event_types::EventType;
use super::service_types::{ServiceType, TraitIndicator};

// Constants
pub const PRIOR_WEIGHT: f32 = 2.0;
pub const INITIAL_SALIENCE: f32 = 0.5;
pub const OBSERVATION_BOOST: f32 = 0.15;
pub const SALIENCE_BOOST: f32 = 0.1;
pub const MAX_PATTERNS_PER_SLOT: usize = 8;
pub const SALIENCE_THRESHOLD: f32 = 0.1;
pub const SALIENCE_FLOOR: f32 = 0.05;

/// What kind of behavior we expect
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternType {
    /// "They provide this service when asked"
    ProvidesWhenAsked { service_type: ServiceType },

    /// "They behave with this trait"
    BehavesWithTrait { trait_indicator: TraitIndicator },

    /// "They're at this location during this time"
    LocationDuring { location_id: EntityId, time_period: TimePeriod },

    /// "They respond to this event with this action type"
    RespondsToEvent { event_type: EventType, typical_response: ActionCategory },
}

/// A behavioral expectation about an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorPattern {
    pub pattern_type: PatternType,

    // Confidence tracking
    pub observation_count: u32,
    pub violation_count: u32,
    pub confidence: f32,

    // Recency
    pub last_confirmed: u64,
    pub last_violated: u64,

    // Decay (like memories)
    pub salience: f32,
}

impl BehaviorPattern {
    pub fn new(pattern_type: PatternType, tick: u64) -> Self {
        Self {
            pattern_type,
            observation_count: 1,
            violation_count: 0,
            confidence: Self::calculate_confidence(1, 0),
            last_confirmed: tick,
            last_violated: 0,
            salience: INITIAL_SALIENCE,
        }
    }

    fn calculate_confidence(observations: u32, violations: u32) -> f32 {
        let total = observations as f32 + violations as f32 + PRIOR_WEIGHT;
        observations as f32 / total
    }

    pub fn record_observation(&mut self, tick: u64) {
        self.observation_count += 1;
        self.last_confirmed = tick;
        self.salience = (self.salience + SALIENCE_BOOST).min(1.0);
        self.confidence = Self::calculate_confidence(self.observation_count, self.violation_count);
    }

    pub fn record_violation(&mut self, tick: u64) {
        self.violation_count += 1;
        self.last_violated = tick;
        self.confidence = Self::calculate_confidence(self.observation_count, self.violation_count);
    }

    pub fn apply_decay(&mut self, decay_rate: f32) {
        self.salience *= 1.0 - decay_rate;
    }

    pub fn is_stale(&self, salience_floor: f32) -> bool {
        self.salience < salience_floor
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_creation() {
        let pattern = BehaviorPattern::new(
            PatternType::ProvidesWhenAsked { service_type: ServiceType::Crafting },
            100,
        );
        assert_eq!(pattern.observation_count, 1);
        assert_eq!(pattern.violation_count, 0);
        assert!(pattern.confidence > 0.0);
        assert!((pattern.salience - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_confidence_calculation() {
        let mut pattern = BehaviorPattern::new(
            PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Reliable },
            0,
        );

        // Initial confidence with 1 observation
        let initial = pattern.confidence;

        // Add more observations
        pattern.record_observation(100);
        pattern.record_observation(200);

        // Confidence should increase
        assert!(pattern.confidence > initial);
        assert_eq!(pattern.observation_count, 3);
    }

    #[test]
    fn test_violation_reduces_confidence() {
        let mut pattern = BehaviorPattern::new(
            PatternType::ProvidesWhenAsked { service_type: ServiceType::Trading },
            0,
        );

        // Build up confidence
        for i in 1..=5 {
            pattern.record_observation(i * 100);
        }
        let high_confidence = pattern.confidence;

        // Record violation
        pattern.record_violation(600);

        // Confidence should decrease
        assert!(pattern.confidence < high_confidence);
        assert_eq!(pattern.violation_count, 1);
    }

    #[test]
    fn test_salience_boost_on_observation() {
        let mut pattern = BehaviorPattern::new(
            PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Generous },
            0,
        );

        let initial_salience = pattern.salience;
        pattern.record_observation(100);

        // Salience should increase by SALIENCE_BOOST
        assert!((pattern.salience - (initial_salience + SALIENCE_BOOST)).abs() < 0.001);
    }

    #[test]
    fn test_salience_capped_at_one() {
        let mut pattern = BehaviorPattern::new(
            PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Peaceful },
            0,
        );

        // Record many observations to try to exceed 1.0
        for i in 1..=20 {
            pattern.record_observation(i * 100);
        }

        assert!(pattern.salience <= 1.0);
    }

    #[test]
    fn test_apply_decay() {
        let mut pattern = BehaviorPattern::new(
            PatternType::ProvidesWhenAsked { service_type: ServiceType::Helping },
            0,
        );

        let initial_salience = pattern.salience;
        pattern.apply_decay(0.1); // 10% decay

        assert!((pattern.salience - initial_salience * 0.9).abs() < 0.001);
    }

    #[test]
    fn test_is_stale() {
        let mut pattern = BehaviorPattern::new(
            PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Reliable },
            0,
        );

        // Initially not stale
        assert!(!pattern.is_stale(SALIENCE_FLOOR));

        // Decay until stale
        for _ in 0..100 {
            pattern.apply_decay(0.1);
        }

        assert!(pattern.is_stale(SALIENCE_FLOOR));
    }

    #[test]
    fn test_location_during_pattern() {
        let location_id = EntityId::new();
        let pattern = BehaviorPattern::new(
            PatternType::LocationDuring {
                location_id,
                time_period: TimePeriod::Morning,
            },
            0,
        );

        match &pattern.pattern_type {
            PatternType::LocationDuring { location_id: loc, time_period } => {
                assert_eq!(*loc, location_id);
                assert_eq!(*time_period, TimePeriod::Morning);
            }
            _ => panic!("Expected LocationDuring pattern"),
        }
    }

    #[test]
    fn test_responds_to_event_pattern() {
        let pattern = BehaviorPattern::new(
            PatternType::RespondsToEvent {
                event_type: EventType::HarmReceived,
                typical_response: ActionCategory::Combat,
            },
            0,
        );

        match &pattern.pattern_type {
            PatternType::RespondsToEvent { event_type, typical_response } => {
                assert_eq!(*event_type, EventType::HarmReceived);
                assert_eq!(*typical_response, ActionCategory::Combat);
            }
            _ => panic!("Expected RespondsToEvent pattern"),
        }
    }

    #[test]
    fn test_confidence_formula() {
        // With PRIOR_WEIGHT = 2.0:
        // confidence = observations / (observations + violations + 2.0)

        // 1 observation, 0 violations: 1 / (1 + 0 + 2) = 1/3 = 0.333...
        let pattern = BehaviorPattern::new(
            PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Reliable },
            0,
        );
        assert!((pattern.confidence - (1.0 / 3.0)).abs() < 0.001);

        // After 2 more observations: 3 / (3 + 0 + 2) = 3/5 = 0.6
        let mut pattern = pattern;
        pattern.record_observation(100);
        pattern.record_observation(200);
        assert!((pattern.confidence - 0.6).abs() < 0.001);
    }

    #[test]
    fn test_last_violated_updates() {
        let mut pattern = BehaviorPattern::new(
            PatternType::BehavesWithTrait { trait_indicator: TraitIndicator::Aggressive },
            0,
        );

        assert_eq!(pattern.last_violated, 0);

        pattern.record_violation(500);
        assert_eq!(pattern.last_violated, 500);

        pattern.record_violation(1000);
        assert_eq!(pattern.last_violated, 1000);
    }

    #[test]
    fn test_last_confirmed_updates() {
        let mut pattern = BehaviorPattern::new(
            PatternType::ProvidesWhenAsked { service_type: ServiceType::Labor },
            100,
        );

        assert_eq!(pattern.last_confirmed, 100);

        pattern.record_observation(500);
        assert_eq!(pattern.last_confirmed, 500);
    }
}
