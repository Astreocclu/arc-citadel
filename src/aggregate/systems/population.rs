//! Population and economy system

use crate::aggregate::world::AggregateWorld;
use crate::aggregate::polity::SpeciesState;
use crate::core::types::Species;

/// Update populations for all polities
pub fn update_populations(world: &mut AggregateWorld) {
    // Collect population updates (to avoid borrow issues)
    let updates: Vec<(u32, u32, f32, f32)> = world.polities.iter()
        .filter(|p| p.alive)
        .map(|polity| {
            let base_growth = match polity.species {
                Species::Human => 1.01,  // 1% per year
                Species::Dwarf => 1.005, // 0.5% per year
                Species::Elf => 1.002,   // 0.2% per year
                Species::Orc => 1.02,    // 2% per year (fast breeding)
                Species::Kobold => 1.08,
                Species::Gnoll => 1.06,
                Species::Lizardfolk => 1.04,
                Species::Hobgoblin => 1.05,
                Species::Ogre => 1.02,
                Species::Harpy => 1.03,
                Species::Centaur => 1.02,
                Species::Minotaur => 1.01,
                Species::Satyr => 1.03,
                Species::Dryad => 1.01,
                Species::Goblin => 1.07,
                Species::Troll => 1.03,
                Species::AbyssalDemons => 1.04,
                Species::Elemental => 1.04,
                Species::Fey => 1.03,
                Species::StoneGiants => 1.03,
                Species::Golem => 1.02,
                Species::Merfolk => 1.03,
                Species::Naga => 1.03,
                Species::Revenant => 1.04,
                Species::Vampire => 1.02,
                Species::Lupine => 1.04,
                // CODEGEN: species_growth_rates
            };

            // Territory quality affects growth
            let territory_quality = calculate_territory_quality(polity, world);

            // War penalty
            let at_war = polity.relations.values().any(|r| r.at_war);
            let war_modifier = if at_war { 0.95 } else { 1.0 };

            // Calculate carrying capacity (territory now tracked via region.controller)
            let capacity: u32 = world.regions.iter()
                .filter(|r| r.controller == Some(polity.id.0))
                .map(|r| r.max_population)
                .sum();

            // Logistic growth - slow down as approaching capacity
            // base_growth is the MAX growth rate when pop << capacity
            // Growth approaches 1.0 (no growth) as pop -> capacity
            let density = if capacity > 0 {
                (polity.population as f32 / capacity as f32).min(1.0)
            } else {
                1.0 // No territory = no growth
            };
            let growth_room = (1.0 - density).max(0.0);

            // Actual growth = 1.0 + (base_growth - 1.0) * growth_room * modifiers
            let effective_growth = 1.0 + (base_growth - 1.0) * growth_room * territory_quality * war_modifier;

            let new_pop = (polity.population as f32 * effective_growth) as u32;

            // Species-specific strength multipliers
            // Slower-growing species compensate with quality over quantity
            let (mil_mult, econ_mult) = match polity.species {
                Species::Human => (1.0, 1.0),      // Baseline
                Species::Dwarf => (2.5, 3.0),     // Master craftsmen, legendary equipment
                Species::Elf => (3.0, 2.5),       // Ancient magic, immortal warriors
                Species::Orc => (1.2, 0.6),       // Strong but poor economy
                _ => (1.0, 1.0),
            };

            // Update strengths
            let econ = new_pop as f32 * 0.08 * territory_quality * econ_mult;
            let mil = new_pop as f32 * 0.1 * mil_mult;

            (polity.id.0, new_pop, econ, mil)
        })
        .collect();

    // Apply updates
    for (id, pop, econ, mil) in updates {
        if let Some(polity) = world.get_polity_mut(id) {
            polity.population = pop;
            polity.economic_strength = econ;
            polity.military_strength = mil;
        }
    }

    // War exhaustion decays slowly over peacetime
    decay_war_exhaustion(world);
}

/// Decay war exhaustion for polities not at war
fn decay_war_exhaustion(world: &mut AggregateWorld) {
    for polity in &mut world.polities {
        if !polity.alive {
            continue;
        }

        let at_war = polity.relations.values().any(|r| r.at_war);
        if at_war {
            continue;
        }

        match &mut polity.species_state {
            SpeciesState::Human(s) => {
                s.war_exhaustion = (s.war_exhaustion - 0.05).max(0.0);
            }
            SpeciesState::Dwarf(s) => {
                s.war_exhaustion = (s.war_exhaustion - 0.03).max(0.0);
            }
            SpeciesState::Elf(s) => {
                s.war_exhaustion = (s.war_exhaustion - 0.02).max(0.0);
            }
            _ => {}
        }
    }
}

fn calculate_territory_quality(polity: &crate::aggregate::polity::Polity, world: &AggregateWorld) -> f32 {
    // Get regions controlled by this polity (territory now tracked via region.controller)
    let our_regions: Vec<&crate::aggregate::region::Region> = world.regions.iter()
        .filter(|r| r.controller == Some(polity.id.0))
        .collect();

    if our_regions.is_empty() {
        return 0.5;
    }

    let mut quality_sum = 0.0;

    for region in &our_regions {
        let fitness = region.fitness.get(&polity.species).unwrap_or(&0.5);
        let resource_bonus = if region.resources != crate::aggregate::region::ResourceType::None {
            0.1
        } else {
            0.0
        };
        quality_sum += fitness + resource_bonus;
    }

    quality_sum / our_regions.len() as f32
}
