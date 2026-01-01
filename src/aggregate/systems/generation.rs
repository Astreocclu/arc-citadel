//! World and polity generation

use std::collections::{HashMap, HashSet};
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use crate::core::types::{Species, PolityId, PolityTier, GovernmentType, RulerId};
use crate::aggregate::region::{Region, Terrain, ResourceType};
use crate::aggregate::polity::{
    Polity, PolityType, CulturalDrift, Relation, SpeciesState,
    HumanState, DwarfState, ElfState, CraftType,
};
use crate::aggregate::simulation::{MapConfig, PolityConfig};
use crate::aggregate::world::AggregateWorld;

/// Generate the pseudo-node map
pub fn generate_map(config: &MapConfig, mut rng: ChaCha8Rng) -> Vec<Region> {
    let mut regions = Vec::new();
    let mut id = 0;

    for y in 0..config.height {
        for x in 0..config.width {
            let terrain = generate_terrain(x, y, config, &mut rng);
            let resources = generate_resources(terrain, &mut rng);
            let fitness = Region::calculate_fitness(terrain);

            let region = Region {
                id,
                name: format!("Region_{}", id),
                terrain,
                resources,
                neighbors: Vec::new(), // Will be filled after all regions created
                fitness,
                controller: None,
                contested_by: Vec::new(),
                max_population: calculate_max_pop(terrain, resources),
            };

            regions.push(region);
            id += 1;
        }
    }

    // Build neighbor graph
    for id in 0..(regions.len() as u32) {
        let neighbors = get_neighbors(id, config.width, config.height);
        regions[id as usize].neighbors = neighbors;
    }

    regions
}

fn generate_terrain(x: u32, y: u32, config: &MapConfig, rng: &mut ChaCha8Rng) -> Terrain {
    // Simple noise-based terrain generation
    let noise = simple_noise(x, y, config.seed);

    // Edge of map tends to be coast/water
    let edge_dist = (x.min(config.width - 1 - x).min(y).min(config.height - 1 - y)) as f32;
    let edge_factor = (edge_dist / 5.0).min(1.0);

    if edge_factor < 0.3 && rng.gen::<f32>() < config.water_frequency {
        return Terrain::Coast;
    }

    if noise > 0.7 && rng.gen::<f32>() < config.mountain_frequency {
        return Terrain::Mountain;
    }

    if noise > 0.5 && rng.gen::<f32>() < config.mountain_frequency * 0.7 {
        return Terrain::Hills;
    }

    if noise < 0.4 && rng.gen::<f32>() < config.forest_frequency {
        return Terrain::Forest;
    }

    if noise < 0.2 && rng.gen::<f32>() < 0.3 {
        return Terrain::Marsh;
    }

    if rng.gen::<f32>() < 0.1 {
        return Terrain::River;
    }

    Terrain::Plains
}

fn simple_noise(x: u32, y: u32, seed: u64) -> f32 {
    // Very simple pseudo-noise
    let n = (x as u64).wrapping_mul(374761393)
        .wrapping_add((y as u64).wrapping_mul(668265263))
        .wrapping_add(seed);
    let n = n.wrapping_mul(n).wrapping_mul(n);
    (n as f32) / (u64::MAX as f32)
}

fn generate_resources(terrain: Terrain, rng: &mut ChaCha8Rng) -> ResourceType {
    let roll: f32 = rng.gen();

    if roll > 0.7 {
        return ResourceType::None;
    }

    match terrain {
        Terrain::Mountain => {
            if roll < 0.2 { ResourceType::Gold }
            else if roll < 0.35 { ResourceType::Gems }
            else if roll < 0.5 { ResourceType::Iron }
            else { ResourceType::Stone }
        }
        Terrain::Hills => {
            if roll < 0.3 { ResourceType::Iron }
            else if roll < 0.5 { ResourceType::Stone }
            else { ResourceType::None }
        }
        Terrain::Forest => {
            if roll < 0.5 { ResourceType::Timber }
            else { ResourceType::None }
        }
        Terrain::Plains => {
            if roll < 0.4 { ResourceType::Grain }
            else { ResourceType::None }
        }
        Terrain::Coast | Terrain::River => {
            if roll < 0.4 { ResourceType::Fish }
            else { ResourceType::None }
        }
        _ => ResourceType::None,
    }
}

fn calculate_max_pop(terrain: Terrain, resources: ResourceType) -> u32 {
    let base = match terrain {
        Terrain::Plains => 10000,
        Terrain::River => 12000,
        Terrain::Coast => 8000,
        Terrain::Hills => 5000,
        Terrain::Forest => 4000,
        Terrain::Mountain => 2000,
        Terrain::Marsh => 1500,
        Terrain::Desert => 1000,
    };

    let resource_bonus = match resources {
        ResourceType::Grain => 3000,
        ResourceType::Fish => 2000,
        ResourceType::None => 0,
        _ => 1000,
    };

    base + resource_bonus
}

fn get_neighbors(id: u32, width: u32, height: u32) -> Vec<u32> {
    let x = id % width;
    let y = id / width;
    let mut neighbors = Vec::new();

    // 6-directional (hex-like) neighbors
    let offsets: [(i32, i32); 6] = [
        (-1, 0), (1, 0),  // Left, right
        (0, -1), (0, 1),  // Up, down
        (-1, if y % 2 == 0 { -1 } else { 1 }),
        (1, if y % 2 == 0 { -1 } else { 1 }),
    ];

    for (dx, dy) in offsets {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;

        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
            neighbors.push((ny as u32 * width) + nx as u32);
        }
    }

    neighbors
}

/// Generate initial polities
pub fn generate_polities(
    regions: &[Region],
    config: &PolityConfig,
    mut rng: ChaCha8Rng,
) -> Vec<Polity> {
    let mut polities = Vec::new();
    let mut claimed: HashSet<u32> = HashSet::new();

    // Generate polities for each species
    polities.extend(generate_species_polities(
        Species::Human, config.human_count, regions, &mut claimed, &mut rng, config
    ));
    polities.extend(generate_species_polities(
        Species::Dwarf, config.dwarf_count, regions, &mut claimed, &mut rng, config
    ));
    polities.extend(generate_species_polities(
        Species::Elf, config.elf_count, regions, &mut claimed, &mut rng, config
    ));

    polities
}

fn generate_species_polities(
    species: Species,
    count: u32,
    regions: &[Region],
    claimed: &mut HashSet<u32>,
    rng: &mut ChaCha8Rng,
    config: &PolityConfig,
) -> Vec<Polity> {
    let mut polities = Vec::new();

    // Find high-fitness regions for this species
    let mut candidates: Vec<(u32, f32)> = regions.iter()
        .filter(|r| !claimed.contains(&r.id))
        .map(|r| (r.id, *r.fitness.get(&species).unwrap_or(&0.0)))
        .collect();

    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for i in 0..count {
        // Pick capital - mostly high fitness, sometimes low (misplaced)
        let is_misplaced = rng.gen::<f32>() < config.misplaced_fraction;

        let capital_id = if is_misplaced && candidates.len() > 10 {
            // Pick from bottom half
            let idx = rng.gen_range(candidates.len() / 2..candidates.len());
            candidates.remove(idx).0
        } else if !candidates.is_empty() {
            candidates.remove(0).0
        } else {
            continue; // No more space
        };

        claimed.insert(capital_id);

        // Claim starting territory around capital
        let mut territory = HashSet::new();
        territory.insert(capital_id);

        let capital_region = &regions[capital_id as usize];
        let mut frontier: Vec<u32> = capital_region.neighbors.clone();

        let target_size = config.avg_starting_territory.saturating_sub(1);

        while territory.len() < target_size as usize && !frontier.is_empty() {
            let idx = rng.gen_range(0..frontier.len());
            let next = frontier.remove(idx);

            if !claimed.contains(&next) {
                territory.insert(next);
                claimed.insert(next);

                // Add new neighbors to frontier
                for neighbor in &regions[next as usize].neighbors {
                    if !claimed.contains(neighbor) && !frontier.contains(neighbor) {
                        frontier.push(*neighbor);
                    }
                }
            }
        }

        // Create polity
        let polity_id = polities.len() as u32 + i;
        let polity_type = match species {
            Species::Human => {
                if territory.len() > 20 { PolityType::Kingdom }
                else if territory.len() > 5 { PolityType::CityState }
                else { PolityType::Tribe }
            }
            Species::Dwarf => {
                if territory.len() > 10 { PolityType::Hold }
                else { PolityType::Clan }
            }
            Species::Elf => {
                if territory.len() > 15 { PolityType::Court }
                else { PolityType::Grove }
            }
        };

        let population = territory.len() as u32 * 500 + rng.gen_range(100..1000);

        let species_state = match species {
            Species::Human => SpeciesState::Human(HumanState {
                expansion_pressure: rng.gen_range(0.3..0.7),
                internal_cohesion: rng.gen_range(0.5..0.9),
                reputation: rng.gen_range(0.4..0.8),
                piety: rng.gen_range(0.2..0.8),
                factions: Vec::new(),
            }),
            Species::Dwarf => SpeciesState::Dwarf(DwarfState {
                grudge_ledger: HashMap::new(),
                oaths: Vec::new(),
                ancestral_sites: vec![capital_id],
                craft_focus: CraftType::Stone,
            }),
            Species::Elf => SpeciesState::Elf(ElfState {
                memory: Vec::new(),
                grief_level: rng.gen_range(0.0..0.2),
                pending_decisions: Vec::new(),
                core_territory: territory.clone(),
                pattern_assessment: rng.gen_range(0.5..0.8),
            }),
        };

        // Determine tier based on territory size
        let tier = match territory.len() {
            0..=5 => PolityTier::Barony,
            6..=10 => PolityTier::County,
            11..=20 => PolityTier::Duchy,
            21..=40 => PolityTier::Kingdom,
            _ => PolityTier::Empire,
        };

        let polity = Polity {
            id: PolityId(polity_id),
            name: generate_polity_name(species, rng),
            species,
            polity_type,
            tier,
            government: GovernmentType::Autocracy,
            parent: None, // All polities start as sovereign
            rulers: vec![RulerId(polity_id)], // Create a ruler with same ID as polity
            council_roles: HashMap::new(),
            population,
            capital: capital_id,
            military_strength: population as f32 * 0.1,
            economic_strength: population as f32 * 0.08,
            cultural_drift: CulturalDrift::default(),
            relations: HashMap::new(),
            species_state,
            alive: true,
        };

        // Update regions to point to this polity as controller
        // Note: territory is now tracked via region.controller, not polity.territory

        polities.push(polity);
    }

    polities
}

fn generate_polity_name(species: Species, rng: &mut ChaCha8Rng) -> String {
    let prefixes = match species {
        Species::Human => ["Alden", "Bran", "Cael", "Dorn", "Eld", "Frey", "Grim", "Hal", "Isen", "Kael"],
        Species::Dwarf => ["Kaz", "Dun", "Bel", "Thor", "Grun", "Mor", "Dur", "Bal", "Khor", "Zar"],
        Species::Elf => ["Aen", "Cel", "Ith", "Lor", "Mel", "Sil", "Thal", "Val", "Yen", "Zeph"],
    };

    let suffixes = match species {
        Species::Human => ["mark", "ford", "heim", "dale", "wick", "ton", "bury", "wood", "vale", "gate"],
        Species::Dwarf => ["heim", "hold", "delve", "deep", "forge", "gard", "mount", "hall", "peak", "stone"],
        Species::Elf => ["wen", "dor", "las", "iel", "ion", "eth", "ath", "oth", "ril", "dal"],
    };

    let prefix = prefixes[rng.gen_range(0..prefixes.len())];
    let suffix = suffixes[rng.gen_range(0..suffixes.len())];

    format!("{}{}", prefix, suffix)
}

/// Initialize relations between all polities
pub fn initialize_relations(world: &mut AggregateWorld) {
    let polity_ids: Vec<PolityId> = world.polities.iter().map(|p| p.id).collect();
    let species_map: HashMap<PolityId, Species> = world.polities.iter()
        .map(|p| (p.id, p.species))
        .collect();

    for polity in &mut world.polities {
        for &other_id in &polity_ids {
            if other_id != polity.id {
                let other_species = species_map.get(&other_id).unwrap();

                // Same species = slightly positive, different = slightly negative
                let base_opinion = if polity.species == *other_species { 10 } else { -10 };

                // Relations are keyed by the inner u32 for backwards compatibility
                polity.relations.insert(other_id.0, Relation {
                    opinion: base_opinion,
                    trust: 0,
                    at_war: false,
                    alliance: false,
                    grudges: Vec::new(),
                    treaties: Vec::new(),
                });
            }
        }
    }
}
