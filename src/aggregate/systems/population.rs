//! Population and economy system

use crate::aggregate::world::AggregateWorld;
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
            let density = if capacity > 0 {
                polity.population as f32 / capacity as f32
            } else {
                1.0
            };
            let logistic_modifier = (1.0 - density).max(0.0);

            let new_pop = (polity.population as f32
                * base_growth
                * territory_quality
                * war_modifier
                * (1.0 + logistic_modifier * 0.1)) as u32;

            // Update strengths
            let econ = new_pop as f32 * 0.08 * territory_quality;
            let mil = new_pop as f32 * 0.1;

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
