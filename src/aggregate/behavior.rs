//! Polity behavior framework

use crate::aggregate::events::EventType;
use crate::aggregate::polity::Polity;
use crate::aggregate::world::AggregateWorld;
use crate::core::types::Species;

/// Trait for species-specific polity behavior
pub trait PolityBehavior {
    /// Generate events for this tick
    fn tick(&self, polity: &Polity, world: &AggregateWorld, year: u32) -> Vec<EventType>;

    /// React to an event
    fn on_event(&self, polity: &mut Polity, event: &EventType, world: &AggregateWorld);
}

/// Get the behavior handler for a species
pub fn get_behavior(species: Species) -> Box<dyn PolityBehavior> {
    match species {
        Species::Gnoll => Box::new(super::species::gnoll::GnollBehavior),
        Species::Vampire => Box::new(super::species::vampire::VampireBehavior),
        Species::Kobold => Box::new(super::species::kobold::KoboldBehavior),
        // Default no-op behavior for other species
        _ => Box::new(DefaultBehavior),
    }
}

struct DefaultBehavior;

impl PolityBehavior for DefaultBehavior {
    fn tick(&self, _polity: &Polity, _world: &AggregateWorld, _year: u32) -> Vec<EventType> {
        Vec::new()
    }

    fn on_event(&self, _polity: &mut Polity, _event: &EventType, _world: &AggregateWorld) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_behavior_returns_empty() {
        let behavior = DefaultBehavior;
        let events = behavior.tick(&create_test_polity(), &create_test_world(), 1);
        assert!(events.is_empty());
    }

    fn create_test_polity() -> Polity {
        use crate::aggregate::polity::*;
        use crate::core::types::{GovernmentType, PolityId, PolityTier};
        use std::collections::HashMap;

        Polity {
            id: PolityId(1),
            name: "Test Polity".to_string(),
            species: Species::Human,
            polity_type: PolityType::Kingdom,
            tier: PolityTier::Kingdom,
            government: GovernmentType::Autocracy,
            parent: None,
            rulers: vec![],
            council_roles: HashMap::new(),
            population: 1000,
            capital: 0,
            military_strength: 100.0,
            economic_strength: 100.0,
            founding_conditions: FoundingConditions::default(),
            cultural_drift: CulturalDrift::default(),
            relations: HashMap::new(),
            species_state: SpeciesState::Human(HumanState::default()),
            alive: true,
        }
    }

    fn create_test_world() -> AggregateWorld {
        use rand_chacha::rand_core::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        AggregateWorld::new(vec![], vec![], ChaCha8Rng::seed_from_u64(42))
    }
}
