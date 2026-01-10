//! Vampire-specific polity behavior - Manipulator archetype

use crate::aggregate::behavior::PolityBehavior;
use crate::aggregate::events::EventType;
use crate::aggregate::polity::Polity;
use crate::aggregate::world::AggregateWorld;
use crate::core::types::PolityId;

pub struct VampireBehavior;

impl PolityBehavior for VampireBehavior {
    fn tick(&self, polity: &Polity, world: &AggregateWorld, _year: u32) -> Vec<EventType> {
        let mut events = Vec::new();

        if let Some(state) = polity.vampire_state() {
            // Expand thrall network through neighboring polities
            if state.thrall_network.len() < 3 {
                if let Some(target) = find_thrall_target(polity, world) {
                    events.push(EventType::InfiltrationAttempt {
                        infiltrator: polity.id,
                        target,
                    });
                }
            }

            // Blood debt collection
            if state.blood_debt_owed > 0 {
                events.push(EventType::TributeDemanded {
                    from: polity.id,
                    amount: state.blood_debt_owed,
                });
            }
        }

        events
    }

    fn on_event(&self, polity: &mut Polity, event: &EventType, _world: &AggregateWorld) {
        if let Some(state) = polity.vampire_state_mut() {
            match event {
                EventType::InfiltrationSuccess { target, .. } => {
                    state.thrall_network.push(target.0);
                }
                EventType::TributePaid { amount, .. } => {
                    state.blood_debt_owed = state.blood_debt_owed.saturating_sub(*amount);
                }
                _ => {}
            }
        }
    }
}

fn find_thrall_target(polity: &Polity, world: &AggregateWorld) -> Option<PolityId> {
    // Find wealthy neighbors not already in thrall network
    let state = polity.vampire_state()?;

    world
        .get_neighbors(polity.id)
        .into_iter()
        .filter(|&neighbor_id| !state.thrall_network.contains(&neighbor_id.0))
        .max_by(|&a, &b| {
            let wealth_a = world
                .get_polity_by_polity_id(a)
                .map(|p| p.economic_strength)
                .unwrap_or(0.0);
            let wealth_b = world
                .get_polity_by_polity_id(b)
                .map(|p| p.economic_strength)
                .unwrap_or(0.0);
            wealth_a
                .partial_cmp(&wealth_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

/// Legacy tick function for backward compatibility with species/mod.rs dispatch
pub fn tick(polity: &Polity, world: &AggregateWorld, year: u32) -> Vec<EventType> {
    VampireBehavior.tick(polity, world, year)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::polity::*;
    use crate::core::types::{GovernmentType, PolityId, PolityTier, Species};
    use std::collections::HashMap;

    fn create_test_polity() -> Polity {
        Polity {
            id: PolityId(1),
            name: "Test Vampire Polity".to_string(),
            species: Species::Vampire,
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
            species_state: SpeciesState::Vampire(VampireState::default()),
            alive: true,
        }
    }

    fn create_debt_polity() -> Polity {
        let mut polity = create_test_polity();
        if let Some(state) = polity.vampire_state_mut() {
            state.blood_debt_owed = 100;
        }
        polity
    }

    fn create_test_world() -> AggregateWorld {
        use rand_chacha::rand_core::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        AggregateWorld::new(vec![], vec![], ChaCha8Rng::seed_from_u64(42))
    }

    #[test]
    fn test_vampire_state_accessor() {
        let polity = create_test_polity();
        let state = polity.vampire_state();
        assert!(state.is_some());
    }

    #[test]
    fn test_blood_debt_triggers_demand() {
        let polity = create_debt_polity();
        let world = create_test_world();

        let events = VampireBehavior.tick(&polity, &world, 1);

        assert!(events
            .iter()
            .any(|e| matches!(e, EventType::TributeDemanded { .. })));
    }

    #[test]
    fn test_no_debt_no_demand() {
        let polity = create_test_polity(); // Default debt is 0
        let world = create_test_world();

        let events = VampireBehavior.tick(&polity, &world, 1);

        assert!(!events
            .iter()
            .any(|e| matches!(e, EventType::TributeDemanded { .. })));
    }

    #[test]
    fn test_infiltration_success_adds_thrall() {
        let mut polity = create_test_polity();
        let world = create_test_world();
        let initial_thralls = polity.vampire_state().unwrap().thrall_network.len();

        VampireBehavior.on_event(
            &mut polity,
            &EventType::InfiltrationSuccess {
                infiltrator: PolityId(1),
                target: PolityId(2),
            },
            &world,
        );

        let final_thralls = polity.vampire_state().unwrap().thrall_network.len();
        assert_eq!(final_thralls, initial_thralls + 1);
        assert!(polity.vampire_state().unwrap().thrall_network.contains(&2));
    }

    #[test]
    fn test_tribute_paid_reduces_debt() {
        let mut polity = create_debt_polity();
        let world = create_test_world();
        let initial_debt = polity.vampire_state().unwrap().blood_debt_owed;

        VampireBehavior.on_event(
            &mut polity,
            &EventType::TributePaid {
                to: PolityId(1),
                amount: 50,
            },
            &world,
        );

        let final_debt = polity.vampire_state().unwrap().blood_debt_owed;
        assert_eq!(final_debt, initial_debt - 50);
    }
}
