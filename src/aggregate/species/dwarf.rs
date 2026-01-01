//! Dwarf species behavior - grudges, oaths, stone-bound

use crate::aggregate::polity::{Polity, GrudgeReason};
use crate::aggregate::world::{AggregateWorld, WarCause};
use crate::aggregate::events::EventType;
use crate::aggregate::systems::expansion::find_expansion_targets;
use crate::aggregate::region::Terrain;

const GRUDGE_WAR_THRESHOLD: f32 = 0.8;
const EXPANSION_POP_THRESHOLD: u32 = 5000;

pub fn tick(polity: &Polity, world: &AggregateWorld, year: u32) -> Vec<EventType> {
    let mut events = Vec::new();

    let state = match polity.dwarf_state() {
        Some(s) => s,
        None => return events,
    };

    // GRUDGE-BALANCE: Unresolved grudges MUST be addressed
    for (&target_id, grudges) in &state.grudge_ledger {
        let total_severity: f32 = grudges.iter().map(|g| g.severity).sum();

        if total_severity > GRUDGE_WAR_THRESHOLD {
            // Check if we can reach them and aren't already at war
            if let Some(rel) = polity.relations.get(&target_id) {
                if rel.at_war {
                    continue;
                }
            }

            if let Some(target) = world.get_polity(target_id) {
                if target.alive && can_reach(polity, target, world) {
                    // Dwarves will declare grudge war even if outmatched
                    // This is NOT strategic. This is compulsive.
                    events.push(EventType::WarDeclared {
                        aggressor: polity.id.0,
                        defender: target_id,
                        cause: WarCause::Grudge(grudges.iter().map(|g| g.id).collect()),
                    });
                }
            }
        }
    }

    // STONE-DEBT: Only expand into appropriate terrain
    let targets = find_expansion_targets(polity, world);
    let valid_expansion: Vec<u32> = targets.unclaimed.iter()
        .filter(|&&region_id| {
            world.get_region(region_id)
                .map(|r| matches!(r.terrain, Terrain::Mountain | Terrain::Hills))
                .unwrap_or(false)
        })
        .copied()
        .collect();

    if !valid_expansion.is_empty() && polity.population > EXPANSION_POP_THRESHOLD {
        events.push(EventType::Expansion {
            polity: polity.id.0,
            region: valid_expansion[0],
        });
    }

    // ANCESTOR-WEIGHT: Check if ancestral sites are held by others
    for &site in &state.ancestral_sites {
        if let Some(region) = world.get_region(site) {
            // Check if we control this region (region.controller is source of truth)
            let we_control = region.controller == Some(polity.id.0);
            if !we_control {
                if let Some(holder) = region.controller {
                    // Check if we already have a grudge for this
                    let has_grudge = state.grudge_ledger
                        .get(&holder)
                        .map(|gs| gs.iter().any(|g| matches!(g.reason, GrudgeReason::HoldsAncestralSite(s) if s == site)))
                        .unwrap_or(false);

                    if !has_grudge {
                        events.push(EventType::GrudgeDeclared {
                            polity: polity.id.0,
                            against: holder,
                            reason: GrudgeReason::HoldsAncestralSite(site),
                        });
                    }
                }
            }
        }
    }

    // HOLD-BOND: Dwarves NEVER betray their Hold
    // This is not a decision. It's hardcoded. They can't.

    events
}

fn can_reach(polity: &Polity, target: &Polity, world: &AggregateWorld) -> bool {
    // Simple: check if any of our controlled regions neighbors any of their controlled regions
    // Territory is now tracked via region.controller
    for region in &world.regions {
        if region.controller == Some(polity.id.0) {
            for &neighbor_id in &region.neighbors {
                if let Some(neighbor) = world.get_region(neighbor_id) {
                    if neighbor.controller == Some(target.id.0) {
                        return true;
                    }
                }
            }
        }
    }
    false
}
