//! Territory expansion system

use crate::aggregate::world::AggregateWorld;
use crate::aggregate::polity::Polity;
use crate::aggregate::region::Terrain;
use crate::core::types::Species;

/// Targets for expansion
pub struct ExpansionTargets {
    pub unclaimed: Vec<u32>,
    pub weak_neighbors: Vec<(u32, u32)>, // (region_id, controller_id)
}

/// Find expansion targets for a polity
/// Territory is now tracked via region.controller, not polity.territory
pub fn find_expansion_targets(polity: &Polity, world: &AggregateWorld) -> ExpansionTargets {
    let mut unclaimed = Vec::new();
    let mut weak_neighbors = Vec::new();

    // Find all regions controlled by this polity
    let our_regions: Vec<u32> = world.regions.iter()
        .filter(|r| r.controller == Some(polity.id.0))
        .map(|r| r.id)
        .collect();

    // Check all adjacent regions to our territory
    for &region_id in &our_regions {
        if let Some(region) = world.get_region(region_id) {
            for &neighbor_id in &region.neighbors {
                // Skip if we already control this region
                if our_regions.contains(&neighbor_id) {
                    continue;
                }

                if let Some(neighbor) = world.get_region(neighbor_id) {
                    // Check if terrain is suitable for this species
                    let fitness = neighbor.fitness.get(&polity.species).unwrap_or(&0.0);

                    let is_valid_terrain = match polity.species {
                        Species::Dwarf => matches!(neighbor.terrain, Terrain::Mountain | Terrain::Hills),
                        Species::Elf => matches!(neighbor.terrain, Terrain::Forest) && *fitness > 0.8,
                        Species::Human => *fitness > 0.3,
                    };

                    if neighbor.controller.is_none() && is_valid_terrain {
                        if !unclaimed.contains(&neighbor_id) {
                            unclaimed.push(neighbor_id);
                        }
                    } else if let Some(controller) = neighbor.controller {
                        if controller != polity.id.0 {
                            // Check if controller is weak
                            if let Some(other) = world.get_polity(controller) {
                                if other.military_strength < polity.military_strength * 0.7 {
                                    weak_neighbors.push((neighbor_id, controller));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort unclaimed by fitness
    unclaimed.sort_by(|a, b| {
        let fa = world.get_region(*a)
            .and_then(|r| r.fitness.get(&polity.species))
            .unwrap_or(&0.0);
        let fb = world.get_region(*b)
            .and_then(|r| r.fitness.get(&polity.species))
            .unwrap_or(&0.0);
        fb.partial_cmp(fa).unwrap()
    });

    ExpansionTargets { unclaimed, weak_neighbors }
}

/// Calculate expansion pressure for humans
/// Territory is now tracked via region.controller, not polity.territory
pub fn calculate_human_expansion_pressure(polity: &Polity, world: &AggregateWorld) -> f32 {
    let base = polity.human_state()
        .map(|s| s.expansion_pressure)
        .unwrap_or(0.0);

    // Pressure increases with population density
    // Find total capacity of all regions we control
    let total_capacity: u32 = world.regions.iter()
        .filter(|r| r.controller == Some(polity.id.0))
        .map(|r| r.max_population)
        .sum();

    let density = if total_capacity > 0 {
        polity.population as f32 / total_capacity as f32
    } else {
        0.0
    };

    base + (density * 0.5)
}
