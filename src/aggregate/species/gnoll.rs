//! Gnoll-specific polity behavior - Raider archetype

use crate::aggregate::polity::Polity;
use crate::aggregate::world::AggregateWorld;
use crate::aggregate::events::EventType;
use crate::aggregate::behavior::PolityBehavior;
use crate::core::types::PolityId;

pub struct GnollBehavior;

impl PolityBehavior for GnollBehavior {
    fn tick(&self, polity: &Polity, world: &AggregateWorld, _year: u32) -> Vec<EventType> {
        let mut events = Vec::new();

        if let Some(state) = polity.gnoll_state() {
            // High pack_frenzy triggers raids
            if state.pack_frenzy > 0.7 {
                // Find weak neighbors to raid
                if let Some(target) = find_raid_target(polity, world) {
                    events.push(EventType::RaidLaunched {
                        attacker: polity.id,
                        target,
                    });
                }
            }

            // Demon taint grows over time
            if state.demon_taint > 0.5 {
                events.push(EventType::CorruptionSpreads {
                    polity: polity.id,
                    intensity: state.demon_taint,
                });
            }
        }

        events
    }

    fn on_event(&self, polity: &mut Polity, event: &EventType, _world: &AggregateWorld) {
        if let Some(state) = polity.gnoll_state_mut() {
            match event {
                EventType::BattleWon { .. } => {
                    state.pack_frenzy = (state.pack_frenzy + 0.2).min(1.0);
                }
                EventType::BattleLost { .. } => {
                    state.pack_frenzy = (state.pack_frenzy - 0.3).max(0.0);
                }
                _ => {}
            }
        }
    }
}

fn find_raid_target(polity: &Polity, world: &AggregateWorld) -> Option<PolityId> {
    // Find neighboring polities with lower military strength
    world.get_neighbors(polity.id)
        .into_iter()
        .find(|&neighbor_id| {
            if let Some(neighbor) = world.get_polity_by_polity_id(neighbor_id) {
                neighbor.military_strength < polity.military_strength * 0.8
            } else {
                false
            }
        })
}

/// Legacy tick function for backward compatibility with species/mod.rs dispatch
pub fn tick(polity: &Polity, world: &AggregateWorld, year: u32) -> Vec<EventType> {
    GnollBehavior.tick(polity, world, year)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::polity::*;
    use crate::core::types::{PolityId, Species, PolityTier, GovernmentType};
    use std::collections::HashMap;

    fn create_test_polity() -> Polity {
        Polity {
            id: PolityId(1),
            name: "Test Gnoll Polity".to_string(),
            species: Species::Gnoll,
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
            cultural_drift: CulturalDrift::default(),
            relations: HashMap::new(),
            species_state: SpeciesState::Gnoll(GnollState::default()),
            alive: true,
        }
    }

    fn create_frenzied_polity() -> Polity {
        let mut polity = create_test_polity();
        if let Some(state) = polity.gnoll_state_mut() {
            state.pack_frenzy = 0.8;
            state.demon_taint = 0.6;
        }
        polity
    }

    fn create_test_world() -> AggregateWorld {
        use rand_chacha::ChaCha8Rng;
        use rand_chacha::rand_core::SeedableRng;

        AggregateWorld::new(vec![], vec![], ChaCha8Rng::seed_from_u64(42))
    }

    #[test]
    fn test_gnoll_state_accessor() {
        let polity = create_test_polity();
        let state = polity.gnoll_state();
        assert!(state.is_some());
    }

    #[test]
    fn test_high_frenzy_no_neighbors_no_raid() {
        let polity = create_frenzied_polity();
        let world = create_test_world();

        let events = GnollBehavior.tick(&polity, &world, 1);

        // Should have CorruptionSpreads but no RaidLaunched (no neighbors)
        assert!(events.iter().any(|e| matches!(e, EventType::CorruptionSpreads { .. })));
        assert!(!events.iter().any(|e| matches!(e, EventType::RaidLaunched { .. })));
    }

    #[test]
    fn test_low_frenzy_no_events() {
        let polity = create_test_polity(); // Default frenzy is 0.0
        let world = create_test_world();

        let events = GnollBehavior.tick(&polity, &world, 1);
        assert!(events.is_empty());
    }

    #[test]
    fn test_battle_won_increases_frenzy() {
        let mut polity = create_test_polity();
        let world = create_test_world();
        let initial_frenzy = polity.gnoll_state().unwrap().pack_frenzy;

        GnollBehavior.on_event(&mut polity, &EventType::BattleWon { polity: PolityId(1) }, &world);

        let final_frenzy = polity.gnoll_state().unwrap().pack_frenzy;
        assert!(final_frenzy > initial_frenzy);
    }

    #[test]
    fn test_battle_lost_decreases_frenzy() {
        let mut polity = create_frenzied_polity();
        let world = create_test_world();
        let initial_frenzy = polity.gnoll_state().unwrap().pack_frenzy;

        GnollBehavior.on_event(&mut polity, &EventType::BattleLost { polity: PolityId(1) }, &world);

        let final_frenzy = polity.gnoll_state().unwrap().pack_frenzy;
        assert!(final_frenzy < initial_frenzy);
    }
}
