//! Elf species behavior - memory, deliberation, grief, patterns

use crate::aggregate::polity::{Polity, DecisionType};
use crate::aggregate::world::AggregateWorld;
use crate::aggregate::events::EventType;
use crate::aggregate::systems::expansion::find_expansion_targets;
use crate::aggregate::region::Terrain;
use crate::core::types::Species;

const GRIEF_PARALYSIS_THRESHOLD: f32 = 0.9;
const GRIEF_ERUPTION_THRESHOLD: f32 = 0.7;
const EXPANSION_POP_THRESHOLD: u32 = 8000;

pub fn tick(polity: &Polity, world: &AggregateWorld, year: u32) -> Vec<EventType> {
    let mut events = Vec::new();

    let state = match polity.elf_state() {
        Some(s) => s,
        None => return events,
    };

    // LONG-CONSEQUENCE: Check if any deliberations are complete
    for decision in &state.pending_decisions {
        let elapsed = year.saturating_sub(decision.deliberation_started);

        if elapsed >= decision.deliberation_required {
            // NOW they act - and it's decisive
            events.push(EventType::DeliberationComplete {
                polity: polity.id.0,
                decision: decision.decision_type.clone(),
            });
        }
    }

    // CHANGE-GRIEF: Calculate grief from recent changes
    let grief_this_year = calculate_grief_this_year(polity, world, year);

    if grief_this_year > 0.01 {
        events.push(EventType::GriefEvent {
            polity: polity.id.0,
            intensity: grief_this_year,
        });
    }

    let total_grief = state.grief_level + grief_this_year;

    if total_grief > GRIEF_PARALYSIS_THRESHOLD {
        // Withdraw from world
        events.push(EventType::Isolation { polity: polity.id.0 });
    } else if total_grief > GRIEF_ERUPTION_THRESHOLD {
        // Lash out at primary grief source
        if let Some(target) = find_primary_grief_source(polity, world) {
            // Start deliberation about war (they don't attack immediately)
            if !state.pending_decisions.iter().any(|d|
                matches!(&d.decision_type, DecisionType::War { target: t } if *t == target)
            ) {
                // Will be picked up next year as a pending decision
                events.push(EventType::DeliberationComplete {
                    polity: polity.id.0,
                    decision: DecisionType::War { target },
                });
            }
        }
    }

    // FOREST-SELF: Absolute defense of core territory
    for &region_id in &state.core_territory {
        if let Some(region) = world.get_region(region_id) {
            if !region.contested_by.is_empty() {
                // Would trigger immediate military response
                // (handled in warfare system based on core_territory)
            }
        }
    }

    // SLOW-GROWTH: Elves rarely expand
    if polity.population > EXPANSION_POP_THRESHOLD {
        let targets = find_expansion_targets(polity, world);
        let perfect_forest: Option<u32> = targets.unclaimed.iter()
            .filter(|&&region_id| {
                world.get_region(region_id)
                    .map(|r| {
                        r.terrain == Terrain::Forest &&
                        r.fitness.get(&Species::Elf).unwrap_or(&0.0) > &0.9
                    })
                    .unwrap_or(false)
            })
            .copied()
            .next();

        if let Some(region) = perfect_forest {
            events.push(EventType::Expansion { polity: polity.id.0, region });
        }
    }

    events
}

fn calculate_grief_this_year(polity: &Polity, world: &AggregateWorld, _year: u32) -> f32 {
    let mut grief: f32 = 0.0;

    // Territory changes cause grief
    let state = match polity.elf_state() {
        Some(s) => s,
        None => return 0.0,
    };

    // Core territory violations are devastating
    // Check if we still control each core region (region.controller is source of truth)
    for &core_region in &state.core_territory {
        if let Some(region) = world.get_region(core_region) {
            if region.controller != Some(polity.id.0) {
                grief += 0.3; // Major grief for lost core territory
            }
        }
    }

    // Population decline causes grief
    // (Would need to track previous population - simplified here)

    // Nearby destruction causes grief
    // Check all regions we control for non-elf neighbors
    for region in &world.regions {
        if region.controller == Some(polity.id.0) {
            for &neighbor_id in &region.neighbors {
                if let Some(neighbor) = world.get_region(neighbor_id) {
                    // If formerly-elf territory is now non-elf
                    if let Some(controller) = neighbor.controller {
                        if let Some(other) = world.get_polity(controller) {
                            if other.species != Species::Elf {
                                grief += 0.02; // Minor grief for nearby non-elf presence
                            }
                        }
                    }
                }
            }
        }
    }

    grief.min(0.3) // Cap grief per year
}

fn find_primary_grief_source(polity: &Polity, world: &AggregateWorld) -> Option<u32> {
    // Find who's most responsible for grief - usually whoever took core territory
    let state = polity.elf_state()?;

    for &core_region in &state.core_territory {
        if let Some(region) = world.get_region(core_region) {
            // Check if we still control this core region (region.controller is source of truth)
            if region.controller != Some(polity.id.0) {
                if let Some(controller) = region.controller {
                    return Some(controller);
                }
            }
        }
    }

    // Otherwise, find the most hated neighbor
    polity.relations.iter()
        .filter(|(_, rel)| rel.opinion < -30 && !rel.at_war)
        .min_by_key(|(_, rel)| rel.opinion)
        .map(|(&id, _)| id)
}
