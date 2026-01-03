//! World and polity generation

use std::collections::{HashMap, HashSet};
use rand::Rng;
use rand_chacha::ChaCha8Rng;

use crate::core::types::{Species, PolityId, PolityTier, GovernmentType, RulerId};
use crate::aggregate::region::{Region, Terrain, ResourceType};
use crate::aggregate::polity::{
    Polity, PolityType, CulturalDrift, Relation, SpeciesState,
    HumanState, DwarfState, ElfState, OrcState, CraftType,
    KoboldState,
    GnollState,
    LizardfolkState,
    HobgoblinState,
    OgreState,
    HarpyState,
    CentaurState,
    MinotaurState,
    SatyrState,
    DryadState,
    GoblinState,
    TrollState,
    AbyssalDemonsState,
    ElementalState,
    FeyState,
    StoneGiantsState,
    GolemState,
    MerfolkState,
    NagaState,
    RevenantState,
    VampireState,
    LupineState,
    // CODEGEN: species_state_imports
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

/// Territory assignment: (region_id, polity_id)
pub type TerritoryAssignment = (u32, u32);

/// Generate initial polities and their territory assignments
pub fn generate_polities(
    regions: &[Region],
    config: &PolityConfig,
    mut rng: ChaCha8Rng,
) -> (Vec<Polity>, Vec<TerritoryAssignment>) {
    let mut polities = Vec::new();
    let mut assignments = Vec::new();
    let mut claimed: HashSet<u32> = HashSet::new();
    let mut next_id: u32 = 0;

    // Generate polities for each species
    let (human_polities, human_assignments) = generate_species_polities(
        Species::Human, config.human_count, regions, &mut claimed, &mut rng, config, &mut next_id
    );
    polities.extend(human_polities);
    assignments.extend(human_assignments);

    let (dwarf_polities, dwarf_assignments) = generate_species_polities(
        Species::Dwarf, config.dwarf_count, regions, &mut claimed, &mut rng, config, &mut next_id
    );
    polities.extend(dwarf_polities);
    assignments.extend(dwarf_assignments);

    let (elf_polities, elf_assignments) = generate_species_polities(
        Species::Elf, config.elf_count, regions, &mut claimed, &mut rng, config, &mut next_id
    );
    polities.extend(elf_polities);
    assignments.extend(elf_assignments);

    (polities, assignments)
}

fn generate_species_polities(
    species: Species,
    count: u32,
    regions: &[Region],
    claimed: &mut HashSet<u32>,
    rng: &mut ChaCha8Rng,
    config: &PolityConfig,
    next_id: &mut u32,
) -> (Vec<Polity>, Vec<TerritoryAssignment>) {
    let mut polities = Vec::new();
    let mut assignments = Vec::new();

    // Find high-fitness regions for this species
    let mut candidates: Vec<(u32, f32)> = regions.iter()
        .filter(|r| !claimed.contains(&r.id))
        .map(|r| (r.id, *r.fitness.get(&species).unwrap_or(&0.0)))
        .collect();

    candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    for _ in 0..count {
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

        // Create polity with unique ID
        let polity_id = *next_id;
        *next_id += 1;
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
            Species::Orc => {
                if territory.len() > 10 { PolityType::Horde }
                else { PolityType::Warband }
            }
            Species::Kobold => {
                if territory.len() > 10 { PolityType::Horde }
                else if territory.len() > 5 { PolityType::Tribe }
                else { PolityType::Clan }
            }
            Species::Gnoll => {
                if territory.len() > 10 { PolityType::Horde }
                else if territory.len() > 5 { PolityType::Tribe }
                else { PolityType::Warband }
            }
            Species::Lizardfolk => {
                if territory.len() > 10 { PolityType::Kingdom }
                else if territory.len() > 5 { PolityType::Tribe }
                else { PolityType::Clan }
            }
            Species::Hobgoblin => {
                if territory.len() > 10 { PolityType::Kingdom }
                else if territory.len() > 5 { PolityType::Hold }
                else { PolityType::Warband }
            }
            Species::Ogre => {
                if territory.len() > 10 { PolityType::Horde }
                else if territory.len() > 5 { PolityType::Tribe }
                else { PolityType::Clan }
            }
            Species::Harpy => {
                if territory.len() > 10 { PolityType::Court }
                else if territory.len() > 5 { PolityType::Tribe }
                else { PolityType::Clan }
            }
            Species::Centaur => {
                if territory.len() > 10 { PolityType::Kingdom }
                else if territory.len() > 5 { PolityType::Tribe }
                else { PolityType::Clan }
            }
            Species::Minotaur => {
                if territory.len() > 10 { PolityType::Kingdom }
                else if territory.len() > 5 { PolityType::Hold }
                else { PolityType::Clan }
            }
            Species::Satyr => {
                if territory.len() > 10 { PolityType::Court }
                else if territory.len() > 5 { PolityType::Court }
                else { PolityType::Grove }
            }
            Species::Dryad => {
                if territory.len() > 10 { PolityType::Court }
                else if territory.len() > 5 { PolityType::Grove }
                else { PolityType::Grove }
            }
            Species::Goblin => {
                if territory.len() > 10 { PolityType::Horde }
                else if territory.len() > 5 { PolityType::Tribe }
                else { PolityType::Clan }
            }
            Species::Troll => {
                if territory.len() > 10 { PolityType::Horde }
                else if territory.len() > 5 { PolityType::Tribe }
                else { PolityType::Clan }
            }
            Species::AbyssalDemons => {
                if territory.len() > 10 { PolityType::Horde }
                else if territory.len() > 5 { PolityType::Court }
                else { PolityType::Clan }
            }
            Species::Elemental => {
                if territory.len() > 10 { PolityType::Horde }
                else if territory.len() > 5 { PolityType::Court }
                else { PolityType::Clan }
            }
            Species::Fey => {
                if territory.len() > 10 { PolityType::Kingdom }
                else if territory.len() > 5 { PolityType::Court }
                else { PolityType::Grove }
            }
            Species::StoneGiants => {
                if territory.len() > 10 { PolityType::Kingdom }
                else if territory.len() > 5 { PolityType::Hold }
                else { PolityType::Clan }
            }
            Species::Golem => {
                if territory.len() > 10 { PolityType::Kingdom }
                else if territory.len() > 5 { PolityType::Hold }
                else { PolityType::Clan }
            }
            Species::Merfolk => {
                if territory.len() > 10 { PolityType::Kingdom }
                else if territory.len() > 5 { PolityType::Tribe }
                else { PolityType::Clan }
            }
            Species::Naga => {
                if territory.len() > 10 { PolityType::Kingdom }
                else if territory.len() > 5 { PolityType::Court }
                else { PolityType::Clan }
            }
            Species::Revenant => {
                if territory.len() > 10 { PolityType::Horde }
                else if territory.len() > 5 { PolityType::Warband }
                else { PolityType::Clan }
            }
            Species::Vampire => {
                if territory.len() > 10 { PolityType::Kingdom }
                else if territory.len() > 5 { PolityType::Court }
                else { PolityType::Clan }
            }
            Species::Lupine => {
                if territory.len() > 10 { PolityType::Horde }
                else if territory.len() > 5 { PolityType::Clan }
                else { PolityType::Warband }
            }
            // CODEGEN: species_polity_type
        };

        // Species-specific starting population density
        // Long-lived species have more established, denser populations
        let pop_per_region = match species {
            Species::Human => 500,
            Species::Dwarf => 1500,   // Ancient holds, very high density
            Species::Elf => 1500,     // Ancient immortal populations
            Species::Orc => 400,      // Aggressive but sparse
            _ => 500,
        };
        let population = territory.len() as u32 * pop_per_region + rng.gen_range(100..1000);

        let species_state = match species {
            Species::Human => SpeciesState::Human(HumanState {
                expansion_pressure: rng.gen_range(0.3..0.7),
                internal_cohesion: rng.gen_range(0.5..0.9),
                reputation: rng.gen_range(0.4..0.8),
                piety: rng.gen_range(0.2..0.8),
                factions: Vec::new(),
                // Humans: balanced personality distribution
                boldness: rng.gen_range(0.3..0.7),
                caution: rng.gen_range(0.3..0.7),
                war_exhaustion: 0.0,
                morale: rng.gen_range(-0.1..0.1),
            }),
            Species::Dwarf => SpeciesState::Dwarf(DwarfState {
                grudge_ledger: HashMap::new(),
                oaths: Vec::new(),
                ancestral_sites: vec![capital_id],
                craft_focus: CraftType::Stone,
                // Dwarves: cautious but bold when honor demands
                boldness: rng.gen_range(0.4..0.8),
                caution: rng.gen_range(0.5..0.9),
                war_exhaustion: 0.0,
                morale: rng.gen_range(0.0..0.2),
            }),
            Species::Elf => SpeciesState::Elf(ElfState {
                memory: Vec::new(),
                grief_level: rng.gen_range(0.0..0.2),
                pending_decisions: Vec::new(),
                core_territory: territory.clone(),
                pattern_assessment: rng.gen_range(0.5..0.8),
                // Elves: very cautious, low boldness
                boldness: rng.gen_range(0.1..0.4),
                caution: rng.gen_range(0.6..0.9),
                war_exhaustion: 0.0,
                morale: rng.gen_range(-0.2..0.1),
            }),
            Species::Orc => SpeciesState::Orc(OrcState {
                waaagh_level: 0.0,
                raid_targets: Vec::new(),
                blood_feuds: Vec::new(),
                tribal_strength: 0.5,
            }),
            Species::Kobold => SpeciesState::Kobold(KoboldState {
                trap_density: 0.0,
                tunnel_network: 0,
                dragon_worship: 0.3,
                grudge_targets: Vec::new(),
            }),
            Species::Gnoll => SpeciesState::Gnoll(GnollState {
                pack_frenzy: 0.0,
                hunting_grounds: Vec::new(),
                demon_taint: 0.1,
                slave_count: 0,
            }),
            Species::Lizardfolk => SpeciesState::Lizardfolk(LizardfolkState {
                spawning_pools: 1,
                food_stores: 0.5,
                tribal_memory: Vec::new(),
                alliance_pragmatism: 0.5,
            }),
            Species::Hobgoblin => SpeciesState::Hobgoblin(HobgoblinState {
                military_doctrine: 0.5,
                legion_strength: 100,
                conquered_territories: Vec::new(),
                war_machine: 0.3,
            }),
            Species::Ogre => SpeciesState::Ogre(OgreState {
                meat_stores: 0.0,
                territory_size: 1,
                dominated_tribes: Vec::new(),
                giant_blood: 0.2,
            }),
            Species::Harpy => SpeciesState::Harpy(HarpyState {
                nesting_sites: Vec::new(),
                trinket_hoard: 0.0,
                cursed_ones: 0,
                flock_unity: 0.5,
            }),
            Species::Centaur => SpeciesState::Centaur(CentaurState {
                sacred_grounds: Vec::new(),
                herd_bonds: 0.5,
                star_wisdom: 0.3,
                oaths_sworn: Vec::new(),
            }),
            Species::Minotaur => SpeciesState::Minotaur(MinotaurState {
                labyrinth_depth: 1,
                sacrifices_claimed: 0,
                cursed_bloodline: 0.5,
                territorial_markers: Vec::new(),
            }),
            Species::Satyr => SpeciesState::Satyr(SatyrState {
                revelry_level: 0.3,
                wine_stores: 0.5,
                charmed_mortals: Vec::new(),
                fey_connection: 0.4,
            }),
            Species::Dryad => SpeciesState::Dryad(DryadState {
                sacred_trees: 1,
                forest_health: 1.0,
                corrupted_lands: Vec::new(),
                fey_pacts: Vec::new(),
            }),
            Species::Goblin => SpeciesState::Goblin(GoblinState {
                grudge_list: Vec::new(),
                hoard_value: 0.0,
                raid_targets: Vec::new(),
                war_exhaustion: 0.0,
            }),
            Species::Troll => SpeciesState::Troll(TrollState {
                grudge_list: Vec::new(),
                hoard_value: 0.0,
                war_exhaustion: 0.0,
            }),
            Species::AbyssalDemons => SpeciesState::AbyssalDemons(AbyssalDemonsState {
                grudge_list: Vec::new(),
                soul_hoard: 0,
                corruption_seeds_planted: Vec::new(),
            }),
            Species::Elemental => SpeciesState::Elemental(ElementalState {
                grudge_list: Vec::new(),
                claimed_terrain: Vec::new(),
                elemental_storm: 0.0,
            }),
            Species::Fey => SpeciesState::Fey(FeyState {
                grudge_list: Vec::new(),
                oath_ledger: Vec::new(),
                mischief_targets: Vec::new(),
            }),
            Species::StoneGiants => SpeciesState::StoneGiants(StoneGiantsState {
                grudge_list: Vec::new(),
                hoard_value: 0.0,
                tribute_demands: Vec::new(),
            }),
            Species::Golem => SpeciesState::Golem(GolemState {
                grudge_list: Vec::new(),
                core_hoard_value: 0.0,
            }),
            Species::Merfolk => SpeciesState::Merfolk(MerfolkState {
                grudge_list: Vec::new(),
                hoard_value: 0.0,
                trade_partners: Vec::new(),
            }),
            Species::Naga => SpeciesState::Naga(NagaState {
                grudge_list: Vec::new(),
                hoarded_secrets: Vec::new(),
                sacred_sites_claimed: 0,
            }),
            Species::Revenant => SpeciesState::Revenant(RevenantState {
                grudge_list: Vec::new(),
                hoard_of_souls: 0,
                war_exhaustion: 0.0,
            }),
            Species::Vampire => SpeciesState::Vampire(VampireState {
                thrall_network: Vec::new(),
                grudge_list: Vec::new(),
                hoard_value: 0.0,
                blood_debt_owed: 0,
            }),
            Species::Lupine => SpeciesState::Lupine(LupineState {
                grudge_list: Vec::new(),
                hoard_of_bones: 0,
                moon_phase_tracker: 0.0,
            }),
            // CODEGEN: species_state_generation
        };

        // Determine tier based on territory size
        let tier = match territory.len() {
            0..=5 => PolityTier::Barony,
            6..=10 => PolityTier::County,
            11..=20 => PolityTier::Duchy,
            21..=40 => PolityTier::Kingdom,
            _ => PolityTier::Empire,
        };

        // Species-specific strength multipliers (matching population.rs)
        let (mil_mult, econ_mult) = match species {
            Species::Human => (1.0, 1.0),
            Species::Dwarf => (2.5, 3.0),   // Master craftsmen, legendary equipment
            Species::Elf => (3.0, 2.5),     // Ancient magic, immortal warriors
            Species::Orc => (1.2, 0.6),
            _ => (1.0, 1.0),
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
            military_strength: population as f32 * 0.1 * mil_mult,
            economic_strength: population as f32 * 0.08 * econ_mult,
            cultural_drift: CulturalDrift::default(),
            relations: HashMap::new(),
            species_state,
            alive: true,
        };

        // Collect territory assignments for this polity
        for region_id in &territory {
            assignments.push((*region_id, polity_id));
        }

        polities.push(polity);
    }

    (polities, assignments)
}

fn generate_polity_name(species: Species, rng: &mut ChaCha8Rng) -> String {
    let prefixes = match species {
        Species::Human => ["Alden", "Bran", "Cael", "Dorn", "Eld", "Frey", "Grim", "Hal", "Isen", "Kael"],
        Species::Dwarf => ["Kaz", "Dun", "Bel", "Thor", "Grun", "Mor", "Dur", "Bal", "Khor", "Zar"],
        Species::Elf => ["Aen", "Cel", "Ith", "Lor", "Mel", "Sil", "Thal", "Val", "Yen", "Zeph"],
        Species::Orc => ["Grak", "Thok", "Zug", "Mog", "Gor", "Skul", "Nar", "Krag", "Urg", "Drak"],
        Species::Kobold => ["Mik", "Pok", "Sniv", "Krik", "Yik", "Drak", "Snik", "Tik", "Rik", "Zik"],
        Species::Gnoll => ["Gnar", "Yeen", "Rip", "Shak", "Kro", "Fang", "Howl", "Pak", "Gor", "Snarl"],
        Species::Lizardfolk => ["Ssi", "Keth", "Ras", "Vex", "Zal", "Thresh", "Sek", "Kar", "Nax", "Ish"],
        Species::Hobgoblin => ["Karg", "Dur", "Maz", "Gol", "Tar", "Brak", "Vor", "Zog", "Nar", "Kul"],
        Species::Ogre => ["Grug", "Mog", "Thud", "Krag", "Bonk", "Smash", "Grub", "Lunk", "Durg", "Glob"],
        Species::Harpy => ["Shri", "Kee", "Scr", "Wail", "Sky", "Tal", "Fea", "Wing", "Caw", "Rav"],
        Species::Centaur => ["Chir", "Nes", "Phol", "Sag", "Ther", "Rix", "Gal", "Oran", "Stell", "Vor"],
        Species::Minotaur => ["Ast", "Bov", "Kron", "Maz", "Thor", "Gor", "Bel", "Tar", "Vor", "Krag"],
        Species::Satyr => ["Pan", "Sil", "Bac", "Fen", "Riv", "Glen", "Pip", "Mer", "Fawn", "Tyl"],
        Species::Dryad => ["Oak", "Wil", "Ash", "Elm", "Bir", "Ivy", "Fern", "Moss", "Haze", "Mist"],
        Species::Goblin => ["Snik", "Griz", "Mog", "Krag", "Zog", "Snarl", "Drik", "Fizz", "Glob", "Rikk"],
        Species::Troll => ["Stone", "Mire", "River", "Bog", "Moss", "Crag", "Deep", "Old", "Grim", "Silent"],
        Species::AbyssalDemons => ["Ash", "Blight", "Cinder", "Dread", "Gloom", "Harrow", "Mire", "Scorch", "Vile", "Wrath"],
        Species::Elemental => ["Ember", "Stone", "Tide", "Gale", "Cinder", "Slate", "Torrent", "Zephyr", "Magma", "Frost"],
        Species::Fey => ["Glimmer", "Whisper", "Mist", "Thorn", "Puck", "Trick", "Willow", "Briar", "Gossamer", "Riddle"],
        Species::StoneGiants => ["Stone", "Iron", "Granite", "Thunder", "Crag", "Boulder", "Mountain", "Deep", "High", "Grim"],
        Species::Golem => ["Stone", "Iron", "Clay", "Granite", "Obsidian", "Ancient", "Weathered", "Runic", "Silent", "Guardian"],
        Species::Merfolk => ["Coral", "Pearl", "Abyssal", "Tidal", "Azure", "Siren", "Kelp", "Nautilus", "Trident", "Deep"],
        Species::Naga => ["Sesha", "Vasuki", "Nagini", "Apep", "Jormun", "Tiamat", "Quetzal", "Mucalinda", "Ananta", "Shesha"],
        Species::Revenant => ["Bone", "Rot", "Grim", "Ash", "Dust", "Shade", "Wight", "Crypt", "Grave", "Carrion"],
        Species::Vampire => ["Blood", "Shadow", "Night", "Crimson", "Obsidian", "Ancient", "Eternal", "Pale", "Silent", "Veiled"],
        Species::Lupine => ["Grey", "Blood", "Moon", "Night", "Iron", "Stone", "Fang", "Shadow", "Winter", "Howling"],
        // CODEGEN: species_name_prefixes
    };

    let suffixes = match species {
        Species::Human => ["mark", "ford", "heim", "dale", "wick", "ton", "bury", "wood", "vale", "gate"],
        Species::Dwarf => ["heim", "hold", "delve", "deep", "forge", "gard", "mount", "hall", "peak", "stone"],
        Species::Elf => ["wen", "dor", "las", "iel", "ion", "eth", "ath", "oth", "ril", "dal"],
        Species::Orc => ["gash", "gore", "skull", "bone", "rot", "maw", "fang", "claw", "blood", "war"],
        Species::Kobold => ["-nak", "-pik", "-snit", "-krak", "-yap", "-dig", "-trap", "-claw", "-scale", "-tail"],
        Species::Gnoll => ["-ak", "-ul", "-ek", "-maw", "-fang", "-claw", "-blood", "-bone", "-pack", "-hunt"],
        Species::Lizardfolk => ["-ith", "-ax", "-ul", "-ek", "-os", "-ar", "-ix", "-eth", "-ak", "-is"],
        Species::Hobgoblin => ["-oth", "-uk", "-ar", "-ash", "-or", "-az", "-ul", "-ek", "-os", "-ag"],
        Species::Ogre => ["-ug", "-ash", "-uk", "-og", "-unk", "-ag", "-ulk", "-ub", "-ok", "-um"],
        Species::Harpy => ["-iek", "-eek", "-aal", "-iss", "-ara", "-ina", "-ona", "-yx", "-ia", "-ela"],
        Species::Centaur => ["-on", "-us", "-ax", "-ion", "-or", "-an", "-is", "-eus", "-os", "-ar"],
        Species::Minotaur => ["-ion", "-os", "-ax", "-ur", "-oth", "-an", "-is", "-uk", "-ar", "-ul"],
        Species::Satyr => ["-us", "-an", "-os", "-ix", "-el", "-ion", "-yr", "-is", "-eon", "-as"],
        Species::Dryad => ["-ara", "-iel", "-yn", "-wen", "-eth", "-ia", "-ora", "-ine", "-ana", "-ina"],
        Species::Goblin => ["-git", "-snatch", "-grab", "-ear", "-fang", "-claw", "-sneak", "-grin", "-tooth", "-eye"],
        Species::Troll => ["Gut", "Hide", "Fang", "Claw", "Bone", "Tusk", "Maw", "Scale", "Wart", "Spine"],
        Species::AbyssalDemons => ["bane", "claw", "fang", "fiend", "maw", "rend", "shade", "spawn", "thorn", "wraith"],
        Species::Elemental => ["Heart", "Fury", "Spire", "Crash", "Brand", "Crag", "Surge", "Howl", "Flow", "Core"],
        Species::Fey => ["dancer", "tongue", "shadow", "leaf", "bind", "weaver", "spark", "thorn", "song", "trick"],
        Species::StoneGiants => ["breaker", "hewer", "fist", "back", "heart", "roar", "hold", "fall", "crusher", "lord"],
        Species::Golem => ["Sentinel", "Warden", "Construct", "Monolith", "Colossus", "Watcher", "Remnant", "Golem", "Statue", "Automaton"],
        Species::Merfolk => ["Reef", "Tide", "Current", "Depths", "Shoal", "Breaker", "Crest", "Grotto", "Spire", "Shell"],
        Species::Naga => ["-Guardian", "-Watcher", "-Coiled", "-the-Keeper", "-of-Secrets", "-Venom-Tongue", "-Ancient-Scale", "-Temple-Born", "-Oracle", "-the-Cursed"],
        Species::Revenant => ["Bane", "Claw", "Walker", "Shambler", "Wraith", "Reaper", "Husk", "Blight", "Howler", "Keeper"],
        Species::Vampire => ["Fang", "Claw", "Court", "Keep", "Sanctum", "Masque", "Vein", "Shroud", "Crypt", "Thirst"],
        Species::Lupine => ["maw", "hide", "claw", "runner", "stalker", "bane", "heart", "caller", "pelt", "hunter"],
        // CODEGEN: species_name_suffixes
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
