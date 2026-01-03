//! Human species behavior - expansion, fragmentation, ambition

use crate::aggregate::polity::Polity;
use crate::aggregate::world::{AggregateWorld, WarCause};
use crate::aggregate::events::EventType;
use crate::aggregate::systems::expansion::{find_expansion_targets, calculate_human_expansion_pressure};

const EXPANSION_THRESHOLD: f32 = 0.3;  // More aggressive expansion
const CIVIL_WAR_THRESHOLD: f32 = 0.4;  // More internal strife
const REPUTATION_CRISIS: f32 = 0.3;    // More honor wars
const BETRAYAL_THRESHOLD: f32 = 0.5;   // More backstabbing

pub fn tick(polity: &Polity, world: &AggregateWorld, _year: u32) -> Vec<EventType> {
    let mut events = Vec::new();

    let state = match polity.human_state() {
        Some(s) => s,
        None => return events,
    };

    // AMBITION: Expansion pressure
    let expansion_pressure = calculate_human_expansion_pressure(polity, world);

    let dynamic_expansion_threshold = EXPANSION_THRESHOLD + polity.decision_modifier() * 0.5;
    if expansion_pressure > dynamic_expansion_threshold {
        let targets = find_expansion_targets(polity, world);

        if let Some(&easy_target) = targets.unclaimed.first() {
            events.push(EventType::Expansion { polity: polity.id.0, region: easy_target });
        } else if let Some(&(_region, controller)) = targets.weak_neighbors.first() {
            // Consider war
            if should_declare_war(polity, controller, world) {
                events.push(EventType::WarDeclared {
                    aggressor: polity.id.0,
                    defender: controller,
                    cause: WarCause::Expansion,
                });
            }
        }
    }

    // INTERNAL COHESION: Risk of civil war (based on population, since territory tracking moved to regions)
    if state.internal_cohesion < CIVIL_WAR_THRESHOLD && polity.population > 2500 {
        events.push(EventType::CivilWar {
            polity: polity.id.0,
            faction_ids: vec![],
        });
    }

    // HONOR: Reputation crisis
    if state.reputation < REPUTATION_CRISIS {
        if let Some(target) = find_honor_target(polity, world) {
            events.push(EventType::WarDeclared {
                aggressor: polity.id.0,
                defender: target,
                cause: WarCause::Honor,
            });
        }
    }

    // BETRAYAL: Humans can break alliances
    if should_betray_ally(polity, world) {
        if let Some(victim) = pick_betrayal_victim(polity, world) {
            events.push(EventType::Betrayal { betrayer: polity.id.0, victim });
        }
    }

    events
}

fn should_declare_war(polity: &Polity, target: u32, world: &AggregateWorld) -> bool {
    // Already at war with them?
    if let Some(rel) = polity.relations.get(&target) {
        if rel.at_war || rel.alliance {
            return false;
        }
    }

    // Dynamic threshold based on personality and state
    let base_threshold = 0.8;
    let modifier = polity.decision_modifier();
    let threshold = (base_threshold + modifier).clamp(0.5, 1.5);

    // Compare strength with dynamic threshold
    if let Some(target_polity) = world.get_polity(target) {
        polity.military_strength > target_polity.military_strength * threshold
    } else {
        false
    }
}

fn find_honor_target(polity: &Polity, world: &AggregateWorld) -> Option<u32> {
    // Find a weak neighbor to pick a fight with
    polity.relations.iter()
        .filter(|(_, rel)| !rel.at_war && !rel.alliance && rel.opinion < 0)
        .filter_map(|(&id, _)| world.get_polity(id).map(|p| (id, p)))
        .filter(|(_, other)| other.alive && other.military_strength < polity.military_strength)
        .map(|(id, _)| id)
        .next()
}

fn should_betray_ally(polity: &Polity, _world: &AggregateWorld) -> bool {
    let state = match polity.human_state() {
        Some(s) => s,
        None => return false,
    };

    // Bold polities with low cohesion betray more easily
    let boldness_factor = state.boldness * 0.2;
    let adjusted_threshold = BETRAYAL_THRESHOLD - boldness_factor;

    state.internal_cohesion < 0.5 && state.expansion_pressure > adjusted_threshold
}

fn pick_betrayal_victim(polity: &Polity, world: &AggregateWorld) -> Option<u32> {
    polity.relations.iter()
        .filter(|(_, rel)| rel.alliance)
        .filter_map(|(&id, _)| world.get_polity(id).map(|p| (id, p)))
        .filter(|(_, other)| other.alive && other.military_strength < polity.military_strength)
        .map(|(id, _)| id)
        .next()
}
