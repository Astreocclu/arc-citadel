//! Event resolution and misc systems

use crate::aggregate::world::{AggregateWorld, War, WarState, WarCause};
use crate::aggregate::events::{HistoryLog, EventType};
use crate::aggregate::systems::warfare::find_contested_regions;

/// Get priority for event ordering (lower = higher priority)
pub fn event_priority(event: &EventType) -> u32 {
    match event {
        EventType::WarDeclared { .. } => 10,
        EventType::GrudgeDeclared { .. } => 15,
        EventType::Betrayal { .. } => 20,
        EventType::AllianceFormed { .. } => 30,
        EventType::AllianceBroken { .. } => 35,
        EventType::Expansion { .. } => 50,
        EventType::Settlement { .. } => 55,
        EventType::CivilWar { .. } => 60,
        EventType::PolityCollapsed { .. } => 70,
        EventType::PolityMerged { .. } => 75,
        EventType::DeliberationComplete { .. } => 80,
        _ => 100,
    }
}

/// Resolve an event
pub fn resolve_event(
    world: &mut AggregateWorld,
    history: &mut HistoryLog,
    event: EventType,
    year: u32,
) {
    match event {
        EventType::WarDeclared { aggressor, defender, ref cause } => {
            resolve_war_declaration(world, aggressor, defender, cause.clone(), year);
            history.add_event(EventType::WarDeclared { aggressor, defender, cause: cause.clone() }, year, vec![aggressor, defender], None);
        }

        EventType::Expansion { polity, region } => {
            resolve_expansion(world, polity, region);
            history.add_event(EventType::Expansion { polity, region }, year, vec![polity], Some(region));
        }

        EventType::Betrayal { betrayer, victim } => {
            resolve_betrayal(world, betrayer, victim, year);
            history.add_event(EventType::Betrayal { betrayer, victim }, year, vec![betrayer, victim], None);
        }

        EventType::GrudgeDeclared { polity, against, ref reason } => {
            add_grudge(world, polity, against, reason.clone(), year);
            history.add_event(EventType::GrudgeDeclared { polity, against, reason: reason.clone() }, year, vec![polity, against], None);
        }

        EventType::CivilWar { polity, ref faction_ids } => {
            resolve_civil_war(world, history, polity, faction_ids, year);
        }

        EventType::AllianceFormed { ref members } => {
            form_alliance(world, members);
            history.add_event(EventType::AllianceFormed { members: members.clone() }, year, members.clone(), None);
        }

        EventType::Isolation { polity } => {
            isolate_polity(world, polity);
            history.add_event(EventType::Isolation { polity }, year, vec![polity], None);
        }

        EventType::GriefEvent { polity, intensity } => {
            add_grief(world, polity, intensity);
            history.add_event(EventType::GriefEvent { polity, intensity }, year, vec![polity], None);
        }

        EventType::DeliberationComplete { polity, ref decision } => {
            execute_elf_decision(world, polity, decision, year);
            history.add_event(EventType::DeliberationComplete { polity, decision: decision.clone() }, year, vec![polity], None);
        }

        other => {
            // Log other events without special handling
            history.add_event(other, year, vec![], None);
        }
    }
}

fn resolve_war_declaration(
    world: &mut AggregateWorld,
    aggressor: u32,
    defender: u32,
    cause: WarCause,
    year: u32,
) {
    // Set at_war flags
    if let Some(p) = world.get_polity_mut(aggressor) {
        if let Some(rel) = p.relations.get_mut(&defender) {
            rel.at_war = true;
            rel.opinion = (rel.opinion - 30).max(-100);
        }
    }
    if let Some(p) = world.get_polity_mut(defender) {
        if let Some(rel) = p.relations.get_mut(&aggressor) {
            rel.at_war = true;
            rel.opinion = (rel.opinion - 30).max(-100);
        }
    }

    // Create war record
    let war = War {
        id: world.next_war_id(),
        aggressor,
        defender,
        cause,
        start_year: year,
        state: WarState::Active,
        contested_regions: find_contested_regions(world, aggressor, defender),
    };

    world.active_wars.push(war);
}

fn resolve_expansion(world: &mut AggregateWorld, polity_id: u32, region_id: u32) {
    // Territory is now tracked via region.controller, not polity.territory
    if let Some(region) = world.regions.get_mut(region_id as usize) {
        if region.controller.is_none() {
            region.controller = Some(polity_id);
            // Note: polity.territory was removed - region.controller is source of truth
        }
    }
}

fn resolve_betrayal(world: &mut AggregateWorld, betrayer: u32, victim: u32, year: u32) {
    // Break alliance
    if let Some(p) = world.get_polity_mut(betrayer) {
        if let Some(rel) = p.relations.get_mut(&victim) {
            rel.alliance = false;
        }
    }
    if let Some(p) = world.get_polity_mut(victim) {
        if let Some(rel) = p.relations.get_mut(&betrayer) {
            rel.alliance = false;
            rel.opinion = (rel.opinion - 50).max(-100);
            rel.trust = (rel.trust - 50).max(-100);
        }
    }

    // Dwarves get a grudge
    if let Some(victim_polity) = world.get_polity_mut(victim) {
        if victim_polity.species == crate::core::types::Species::Dwarf {
            if let Some(state) = victim_polity.dwarf_state_mut() {
                let grudge = crate::aggregate::polity::Grudge {
                    id: state.grudge_ledger.values().map(|v| v.len()).sum::<usize>() as u32,
                    against: betrayer,
                    reason: crate::aggregate::polity::GrudgeReason::Betrayal,
                    severity: 1.0,
                    year_incurred: year,
                };
                state.grudge_ledger.entry(betrayer).or_default().push(grudge);
            }
        }
    }
}

fn add_grudge(
    world: &mut AggregateWorld,
    polity_id: u32,
    against: u32,
    reason: crate::aggregate::polity::GrudgeReason,
    year: u32,
) {
    if let Some(polity) = world.get_polity_mut(polity_id) {
        if let Some(state) = polity.dwarf_state_mut() {
            let grudge = crate::aggregate::polity::Grudge {
                id: state.grudge_ledger.values().map(|v| v.len()).sum::<usize>() as u32,
                against,
                reason,
                severity: 0.5,
                year_incurred: year,
            };
            state.grudge_ledger.entry(against).or_default().push(grudge);
        }
    }
}

fn resolve_civil_war(
    world: &mut AggregateWorld,
    history: &mut HistoryLog,
    polity_id: u32,
    _faction_ids: &[u32],
    year: u32,
) {
    use crate::core::types::{PolityId, PolityTier, RulerId};

    // Get regions controlled by this polity (territory now tracked via region.controller)
    let territory_vec: Vec<u32> = world.regions.iter()
        .filter(|r| r.controller == Some(polity_id))
        .map(|r| r.id)
        .collect();

    let split_point = territory_vec.len() / 2;

    if split_point == 0 {
        return;
    }

    // Collect data before modifying world
    let (rebel_id, polity_name, polity_species, polity_type, polity_species_state, government) = {
        let polity = match world.get_polity(polity_id) {
            Some(p) => p,
            None => return,
        };
        let rebel_id = world.polities.len() as u32;
        (rebel_id, polity.name.clone(), polity.species, polity.polity_type, polity.species_state.clone(), polity.government)
    };

    // Assign rebel regions to new controller
    let rebel_regions = &territory_vec[split_point..];
    for &region_id in rebel_regions {
        if let Some(region) = world.regions.get_mut(region_id as usize) {
            region.controller = Some(rebel_id);
        }
    }

    // Create rebel polity
    let rebel_capital = rebel_regions.first().copied().unwrap_or(0);
    let rebel = crate::aggregate::polity::Polity {
        id: PolityId(rebel_id),
        name: format!("{}_Rebels", polity_name),
        species: polity_species,
        polity_type,
        tier: PolityTier::Barony,
        government,
        parent: None,
        rulers: vec![RulerId(rebel_id)],
        council_roles: std::collections::HashMap::new(),
        population: 0,  // Will be set below
        capital: rebel_capital,
        military_strength: 0.0,  // Will be set below
        economic_strength: 0.0,  // Will be set below
        cultural_drift: crate::aggregate::polity::CulturalDrift::default(),
        relations: std::collections::HashMap::new(),
        species_state: polity_species_state,
        alive: true,
    };

    // Now modify the original polity
    if let Some(polity) = world.get_polity_mut(polity_id) {
        polity.population /= 2;
    }

    // Copy population to rebel
    let mut rebel = rebel;
    if let Some(polity) = world.get_polity(polity_id) {
        rebel.population = polity.population;
        rebel.military_strength = polity.military_strength;
        rebel.economic_strength = polity.economic_strength;
    }

    world.polities.push(rebel);

    history.add_event(
        EventType::PolityCollapsed {
            polity: polity_id,
            successor_states: vec![polity_id, rebel_id]
        },
        year,
        vec![polity_id, rebel_id],
        None,
    );
}

fn form_alliance(world: &mut AggregateWorld, members: &[u32]) {
    for &member1 in members {
        for &member2 in members {
            if member1 != member2 {
                if let Some(p) = world.get_polity_mut(member1) {
                    if let Some(rel) = p.relations.get_mut(&member2) {
                        rel.alliance = true;
                        rel.opinion = (rel.opinion + 20).min(100);
                        rel.trust = (rel.trust + 10).min(100);
                    }
                }
            }
        }
    }
}

fn isolate_polity(world: &mut AggregateWorld, polity_id: u32) {
    if let Some(polity) = world.get_polity_mut(polity_id) {
        // Break all alliances, reduce relations
        for rel in polity.relations.values_mut() {
            rel.alliance = false;
        }

        // Set elf state to isolation
        if let Some(state) = polity.elf_state_mut() {
            state.grief_level *= 0.5; // Isolation helps heal grief
        }
    }
}

fn add_grief(world: &mut AggregateWorld, polity_id: u32, intensity: f32) {
    if let Some(polity) = world.get_polity_mut(polity_id) {
        if let Some(state) = polity.elf_state_mut() {
            state.grief_level = (state.grief_level + intensity).min(1.0);
        }
    }
}

fn execute_elf_decision(
    world: &mut AggregateWorld,
    polity_id: u32,
    decision: &crate::aggregate::polity::DecisionType,
    year: u32,
) {
    match decision {
        crate::aggregate::polity::DecisionType::War { target } => {
            resolve_war_declaration(
                world,
                polity_id,
                *target,
                WarCause::Grief,
                year,
            );
        }
        crate::aggregate::polity::DecisionType::Alliance { with } => {
            form_alliance(world, &[polity_id, *with]);
        }
        crate::aggregate::polity::DecisionType::Isolation => {
            isolate_polity(world, polity_id);
        }
        _ => {}
    }
}

/// Check if polities should die
pub fn check_polity_viability(world: &mut AggregateWorld, history: &mut HistoryLog, year: u32) {
    // Territory is now tracked via region.controller, not polity.territory
    // First, count regions per polity
    let mut region_counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
    for region in &world.regions {
        if let Some(controller) = region.controller {
            *region_counts.entry(controller).or_insert(0) += 1;
        }
    }

    let dead_polities: Vec<u32> = world.polities.iter()
        .filter(|p| {
            p.alive && (region_counts.get(&p.id.0).copied().unwrap_or(0) == 0 || p.population < 100)
        })
        .map(|p| p.id.0)
        .collect();

    for polity_id in dead_polities {
        if let Some(polity) = world.get_polity_mut(polity_id) {
            polity.alive = false;
        }

        history.add_event(
            EventType::PolityCollapsed { polity: polity_id, successor_states: vec![] },
            year,
            vec![polity_id],
            None,
        );
    }
}

/// Apply cultural drift over time
pub fn apply_cultural_drift(world: &mut AggregateWorld, _year: u32) {
    // Simplified: just decay drift values slightly
    for polity in &mut world.polities {
        if !polity.alive { continue; }

        if let Some((_, ref mut val)) = polity.cultural_drift.primary_drift {
            *val *= 0.99;
        }
    }
}
