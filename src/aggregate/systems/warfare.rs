//! War resolution system

use rand::Rng;

use crate::aggregate::events::{EventType, HistoryLog};
use crate::aggregate::polity::{Polity, SpeciesState};
use crate::aggregate::world::{AggregateWorld, War, WarCause, WarState};
use crate::core::types::Species;

/// Update a polity's war exhaustion and morale after combat
fn update_war_state(polity: &mut Polity, won: bool, intensity: f32) {
    match &mut polity.species_state {
        SpeciesState::Human(s) => {
            s.war_exhaustion = (s.war_exhaustion + intensity * 0.1).min(1.0);
            s.morale = if won {
                (s.morale + 0.2).min(1.0)
            } else {
                (s.morale - 0.3).max(-1.0)
            };
        }
        SpeciesState::Dwarf(s) => {
            s.war_exhaustion = (s.war_exhaustion + intensity * 0.08).min(1.0); // Dwarves endure
            s.morale = if won {
                (s.morale + 0.15).min(1.0)
            } else {
                (s.morale - 0.2).max(-1.0)
            };
        }
        SpeciesState::Elf(s) => {
            s.war_exhaustion = (s.war_exhaustion + intensity * 0.15).min(1.0); // Elves tire of war
            s.morale = if won {
                (s.morale + 0.1).min(1.0)
            } else {
                (s.morale - 0.4).max(-1.0) // Losses hit elves hard
            };
        }
        _ => {}
    }
}

/// Resolve all active wars for this year
pub fn resolve_active_wars(world: &mut AggregateWorld, history: &mut HistoryLog, year: u32) {
    let mut war_outcomes = Vec::new();

    for war in &world.active_wars {
        let outcome = resolve_war_year(war, world, year);
        war_outcomes.push((war.id, outcome));
    }

    // Apply outcomes
    for (war_id, outcome) in war_outcomes {
        match outcome {
            WarYearOutcome::Continues => {}
            WarYearOutcome::RegionChanged { region, from, to } => {
                transfer_region(world, region, from, to);
                history.add_event(
                    EventType::RegionLost {
                        loser: from,
                        winner: to,
                        region,
                    },
                    year,
                    vec![from, to],
                    Some(region),
                );
            }
            WarYearOutcome::Ended { victor } => {
                if let Some(war) = world.active_wars.iter_mut().find(|w| w.id == war_id) {
                    war.state = WarState::Concluded { victor };
                }

                // End war relations
                let (aggressor, defender) = {
                    let war = world.active_wars.iter().find(|w| w.id == war_id).unwrap();
                    (war.aggressor, war.defender)
                };

                if let Some(p) = world.get_polity_mut(aggressor) {
                    if let Some(rel) = p.relations.get_mut(&defender) {
                        rel.at_war = false;
                    }
                }
                if let Some(p) = world.get_polity_mut(defender) {
                    if let Some(rel) = p.relations.get_mut(&aggressor) {
                        rel.at_war = false;
                    }
                }

                // Update war exhaustion and morale based on outcome
                if let Some(winner_id) = victor {
                    let loser_id = if winner_id == aggressor {
                        defender
                    } else {
                        aggressor
                    };
                    if let Some(winner) = world.get_polity_mut(winner_id) {
                        update_war_state(winner, true, 0.5);
                    }
                    if let Some(loser) = world.get_polity_mut(loser_id) {
                        update_war_state(loser, false, 0.5);
                    }
                }

                history.add_event(
                    EventType::WarEnded { war_id, victor },
                    year,
                    vec![aggressor, defender],
                    None,
                );
            }
        }
    }

    // Remove concluded wars
    world
        .active_wars
        .retain(|w| !matches!(w.state, WarState::Concluded { .. }));
}

enum WarYearOutcome {
    Continues,
    RegionChanged { region: u32, from: u32, to: u32 },
    Ended { victor: Option<u32> },
}

fn resolve_war_year(war: &War, world: &AggregateWorld, _year: u32) -> WarYearOutcome {
    let aggressor = match world.get_polity(war.aggressor) {
        Some(p) if p.alive => p,
        _ => {
            return WarYearOutcome::Ended {
                victor: Some(war.defender),
            }
        }
    };

    let defender = match world.get_polity(war.defender) {
        Some(p) if p.alive => p,
        _ => {
            return WarYearOutcome::Ended {
                victor: Some(war.aggressor),
            }
        }
    };

    // Calculate relative strength
    let aggressor_strength = calculate_war_strength(aggressor, war, world);
    let defender_strength = calculate_war_strength(defender, war, world);

    let total = aggressor_strength + defender_strength;
    if total <= 0.0 {
        return WarYearOutcome::Continues;
    }

    let aggressor_chance = aggressor_strength / total;

    // Roll for contested regions
    let mut rng = world.rng.clone();

    for &region_id in &war.contested_regions {
        if let Some(region) = world.get_region(region_id) {
            if region.controller == Some(war.defender) {
                let roll: f32 = rng.gen();
                if roll < aggressor_chance * 0.3 {
                    // Aggressor takes region
                    return WarYearOutcome::RegionChanged {
                        region: region_id,
                        from: war.defender,
                        to: war.aggressor,
                    };
                }
            }
        }
    }

    // Check war exhaustion
    let aggressor_exhausted = is_war_exhausted(aggressor, world);
    let defender_exhausted = is_war_exhausted(defender, world);

    // Dwarves never accept exhaustion for grudge wars
    let aggressor_gives_up = aggressor_exhausted
        && !(aggressor.species == Species::Dwarf && matches!(war.cause, WarCause::Grudge(_)));

    if aggressor_gives_up {
        return WarYearOutcome::Ended {
            victor: Some(war.defender),
        };
    }

    if defender_exhausted {
        return WarYearOutcome::Ended {
            victor: Some(war.aggressor),
        };
    }

    WarYearOutcome::Continues
}

fn calculate_war_strength(polity: &Polity, _war: &War, world: &AggregateWorld) -> f32 {
    let base = polity.military_strength;

    // Get regions controlled by this polity (territory now tracked via region.controller)
    let our_regions: Vec<&crate::aggregate::region::Region> = world
        .regions
        .iter()
        .filter(|r| r.controller == Some(polity.id.0))
        .collect();

    let region_count = our_regions.len().max(1);

    // Terrain bonus if defending
    let terrain_bonus: f32 = our_regions
        .iter()
        .map(|r| match r.terrain {
            crate::aggregate::region::Terrain::Mountain => 0.3,
            crate::aggregate::region::Terrain::Hills => 0.15,
            crate::aggregate::region::Terrain::Forest => 0.1,
            _ => 0.0,
        })
        .sum::<f32>()
        / region_count as f32;

    base * (1.0 + terrain_bonus)
}

fn is_war_exhausted(polity: &Polity, world: &AggregateWorld) -> bool {
    // Count regions controlled by this polity (territory now tracked via region.controller)
    let region_count = world
        .regions
        .iter()
        .filter(|r| r.controller == Some(polity.id.0))
        .count();

    // Exhausted if population dropped significantly or territory shrunk
    polity.population < 500 || region_count < 2
}

fn transfer_region(world: &mut AggregateWorld, region_id: u32, _from: u32, to: u32) {
    // Update region controller (this is now the source of truth for territory)
    if let Some(region) = world.regions.get_mut(region_id as usize) {
        region.controller = Some(to);
    }
    // Note: polity.territory was removed - region.controller is source of truth
}

/// Find contested regions between two polities
/// Territory is now tracked via region.controller, not polity.territory
pub fn find_contested_regions(world: &AggregateWorld, p1: u32, p2: u32) -> Vec<u32> {
    let mut contested = Vec::new();

    // Find all regions controlled by polity p1
    for region in &world.regions {
        if region.controller == Some(p1) {
            for &neighbor_id in &region.neighbors {
                if let Some(neighbor) = world.get_region(neighbor_id) {
                    if neighbor.controller == Some(p2) && !contested.contains(&neighbor_id) {
                        contested.push(neighbor_id);
                    }
                }
            }
        }
    }

    contested
}
