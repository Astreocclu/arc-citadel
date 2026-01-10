//! Kobold-specific polity behavior - Trapper archetype

use crate::aggregate::behavior::PolityBehavior;
use crate::aggregate::events::EventType;
use crate::aggregate::polity::Polity;
use crate::aggregate::world::AggregateWorld;
use crate::core::types::PolityId;

pub struct KoboldBehavior;

impl PolityBehavior for KoboldBehavior {
    fn tick(&self, polity: &Polity, _world: &AggregateWorld, _year: u32) -> Vec<EventType> {
        let mut events = Vec::new();

        if let Some(state) = polity.kobold_state() {
            // Build traps when tunnel network is large enough
            if state.tunnel_network > 5 && state.trap_density < 0.8 {
                events.push(EventType::TrapConstruction {
                    polity: polity.id,
                    trap_count: (state.tunnel_network / 2) as u32,
                });
            }

            // Grudge-based spite attacks
            if !state.grudge_targets.is_empty() && polity.military_strength > 50.0 {
                let target_id = state.grudge_targets[0];
                events.push(EventType::SpiteRaid {
                    attacker: polity.id,
                    target: PolityId(target_id),
                });
            }

            // Dragon worship increases when dragon is nearby (placeholder)
            if state.dragon_worship > 0.8 {
                events.push(EventType::DragonTributeOffered { polity: polity.id });
            }
        }

        events
    }

    fn on_event(&self, polity: &mut Polity, event: &EventType, _world: &AggregateWorld) {
        if let Some(state) = polity.kobold_state_mut() {
            match event {
                EventType::TrapTriggered { casualties, .. } => {
                    state.trap_density = (state.trap_density - 0.1).max(0.0);
                    // Successful trap increases cunning confidence
                    if *casualties > 0 {
                        state.tunnel_network += 1; // Expand network after success
                    }
                }
                EventType::TrapConstruction { trap_count, .. } => {
                    state.trap_density =
                        (state.trap_density + (*trap_count as f32 * 0.05)).min(1.0);
                }
                EventType::SpiteRaid { target, .. } => {
                    // Remove target from grudge list after raiding
                    state.grudge_targets.retain(|&id| id != target.0);
                }
                _ => {}
            }
        }
    }
}

/// Legacy tick function for backward compatibility with species/mod.rs dispatch
pub fn tick(polity: &Polity, world: &AggregateWorld, year: u32) -> Vec<EventType> {
    KoboldBehavior.tick(polity, world, year)
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
            name: "Test Kobold Polity".to_string(),
            species: Species::Kobold,
            polity_type: PolityType::Horde,
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
            species_state: SpeciesState::Kobold(KoboldState::default()),
            alive: true,
        }
    }

    fn create_trap_network_polity() -> Polity {
        let mut polity = create_test_polity();
        if let Some(state) = polity.kobold_state_mut() {
            state.tunnel_network = 10;
            state.trap_density = 0.3;
        }
        polity
    }

    fn create_grudge_polity() -> Polity {
        let mut polity = create_test_polity();
        if let Some(state) = polity.kobold_state_mut() {
            state.grudge_targets = vec![2, 3];
        }
        polity
    }

    fn create_test_world() -> AggregateWorld {
        use rand_chacha::rand_core::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        AggregateWorld::new(vec![], vec![], ChaCha8Rng::seed_from_u64(42))
    }

    #[test]
    fn test_kobold_state_accessor() {
        let polity = create_test_polity();
        let state = polity.kobold_state();
        assert!(state.is_some());
    }

    #[test]
    fn test_large_tunnel_network_builds_traps() {
        let polity = create_trap_network_polity();
        let world = create_test_world();

        let events = KoboldBehavior.tick(&polity, &world, 1);

        assert!(events
            .iter()
            .any(|e| matches!(e, EventType::TrapConstruction { .. })));
    }

    #[test]
    fn test_no_tunnel_no_traps() {
        let polity = create_test_polity(); // Default tunnel_network is 0
        let world = create_test_world();

        let events = KoboldBehavior.tick(&polity, &world, 1);

        assert!(!events
            .iter()
            .any(|e| matches!(e, EventType::TrapConstruction { .. })));
    }

    #[test]
    fn test_grudge_triggers_spite_raid() {
        let polity = create_grudge_polity();
        let world = create_test_world();

        let events = KoboldBehavior.tick(&polity, &world, 1);

        assert!(events
            .iter()
            .any(|e| matches!(e, EventType::SpiteRaid { .. })));
    }

    #[test]
    fn test_trap_triggered_reduces_density() {
        let mut polity = create_trap_network_polity();
        let world = create_test_world();
        let initial_density = polity.kobold_state().unwrap().trap_density;

        KoboldBehavior.on_event(
            &mut polity,
            &EventType::TrapTriggered {
                polity: PolityId(1),
                casualties: 5,
            },
            &world,
        );

        let final_density = polity.kobold_state().unwrap().trap_density;
        assert!(final_density < initial_density);
    }

    #[test]
    fn test_trap_construction_increases_density() {
        let mut polity = create_test_polity();
        let world = create_test_world();
        let initial_density = polity.kobold_state().unwrap().trap_density;

        KoboldBehavior.on_event(
            &mut polity,
            &EventType::TrapConstruction {
                polity: PolityId(1),
                trap_count: 10,
            },
            &world,
        );

        let final_density = polity.kobold_state().unwrap().trap_density;
        assert!(final_density > initial_density);
    }

    #[test]
    fn test_spite_raid_removes_grudge() {
        let mut polity = create_grudge_polity();
        let world = create_test_world();
        assert!(polity.kobold_state().unwrap().grudge_targets.contains(&2));

        KoboldBehavior.on_event(
            &mut polity,
            &EventType::SpiteRaid {
                attacker: PolityId(1),
                target: PolityId(2),
            },
            &world,
        );

        assert!(!polity.kobold_state().unwrap().grudge_targets.contains(&2));
    }
}
