//! ECS World - manages all entities and their components

use ahash::AHashMap;
use crate::city::building::{BuildingArchetype, BuildingId, BuildingType};
use crate::core::types::{EntityId, Species, Vec2};
use crate::core::astronomy::AstronomicalState;
use crate::entity::species::human::HumanArchetype;
use crate::entity::species::orc::OrcArchetype;
use crate::simulation::resource_zone::ResourceZone;
use crate::rules::SpeciesRules;

/// Abundance level of a food zone
#[derive(Debug, Clone)]
pub enum Abundance {
    /// Infinite food - never depletes
    Unlimited,
    /// Limited food - depletes when eaten, regenerates over time
    Scarce { current: f32, max: f32, regen: f32 },
}

/// A static zone where entities can find food
#[derive(Debug, Clone)]
pub struct FoodZone {
    pub id: u32,
    pub position: Vec2,
    pub radius: f32,
    pub abundance: Abundance,
}

impl FoodZone {
    /// Check if entity at given position can eat from this zone
    pub fn contains(&self, pos: Vec2) -> bool {
        self.position.distance(&pos) <= self.radius
    }

    /// Try to consume food from this zone. Returns amount actually consumed.
    pub fn consume(&mut self, amount: f32) -> f32 {
        match &mut self.abundance {
            Abundance::Unlimited => amount,
            Abundance::Scarce { current, .. } => {
                let consumed = amount.min(*current);
                *current -= consumed;
                consumed
            }
        }
    }

    /// Regenerate food for scarce zones
    pub fn regenerate(&mut self) {
        if let Abundance::Scarce { current, max, regen } = &mut self.abundance {
            *current = (*current + *regen).min(*max);
        }
    }
}

/// The game world containing all entities
pub struct World {
    pub current_tick: u64,
    entity_registry: AHashMap<EntityId, (Species, usize)>,
    pub humans: HumanArchetype,
    pub orcs: OrcArchetype,
    next_indices: AHashMap<Species, usize>,
    pub food_zones: Vec<FoodZone>,
    next_food_zone_id: u32,
    pub resource_zones: Vec<ResourceZone>,
    pub astronomy: AstronomicalState,
    /// Runtime-loaded species action rules
    pub species_rules: SpeciesRules,
    /// All buildings in the world
    pub buildings: BuildingArchetype,
}

impl World {
    pub fn new() -> Self {
        let mut next_indices = AHashMap::new();
        next_indices.insert(Species::Human, 0);
        next_indices.insert(Species::Dwarf, 0);
        next_indices.insert(Species::Elf, 0);
        next_indices.insert(Species::Orc, 0);

        // Load species rules from TOML files
        let species_dir = std::path::Path::new("species");
        let species_rules = crate::rules::load_species_rules(species_dir)
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to load species rules: {}", e);
                SpeciesRules::new()
            });

        Self {
            current_tick: 0,
            entity_registry: AHashMap::new(),
            humans: HumanArchetype::new(),
            orcs: OrcArchetype::new(),
            next_indices,
            food_zones: Vec::new(),
            next_food_zone_id: 0,
            resource_zones: Vec::new(),
            astronomy: AstronomicalState::default(),
            species_rules,
            buildings: BuildingArchetype::new(),
        }
    }

    pub fn add_food_zone(&mut self, position: Vec2, radius: f32, abundance: Abundance) -> u32 {
        let id = self.next_food_zone_id;
        self.next_food_zone_id += 1;
        self.food_zones.push(FoodZone { id, position, radius, abundance });
        id
    }

    pub fn spawn_human(&mut self, name: String) -> EntityId {
        let entity_id = EntityId::new();
        let index = *self.next_indices.get(&Species::Human).unwrap();

        self.humans.spawn(entity_id, name, self.current_tick);

        self.entity_registry.insert(entity_id, (Species::Human, index));
        *self.next_indices.get_mut(&Species::Human).unwrap() += 1;

        entity_id
    }

    pub fn spawn_orc(&mut self, name: String) -> EntityId {
        let entity_id = EntityId::new();
        let index = *self.next_indices.get(&Species::Orc).unwrap();

        self.orcs.spawn(entity_id, name, self.current_tick);

        self.entity_registry.insert(entity_id, (Species::Orc, index));
        *self.next_indices.get_mut(&Species::Orc).unwrap() += 1;

        entity_id
    }

    /// Spawn a new building at the given position
    pub fn spawn_building(&mut self, building_type: BuildingType, position: Vec2) -> BuildingId {
        let id = BuildingId::new();
        self.buildings.spawn(id, building_type, position, self.current_tick);
        id
    }

    pub fn get_entity_info(&self, entity_id: EntityId) -> Option<(Species, usize)> {
        self.entity_registry.get(&entity_id).copied()
    }

    pub fn entity_count(&self) -> usize {
        self.humans.count() + self.orcs.count()
    }

    pub fn tick(&mut self) {
        self.current_tick += 1;
    }

    pub fn human_entities(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.entity_registry
            .iter()
            .filter(|(_, (species, _))| *species == Species::Human)
            .map(|(id, _)| *id)
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::Vec2;
    use crate::core::astronomy::Season;

    #[test]
    fn test_world_has_astronomy() {
        let world = World::new();

        assert_eq!(world.astronomy.year, 1);
        assert_eq!(world.astronomy.day_of_year, 1);
        assert_eq!(world.astronomy.season, Season::Spring);
    }

    #[test]
    fn test_food_zones_exist() {
        let mut world = World::new();
        assert!(world.food_zones.is_empty());

        world.add_food_zone(Vec2::new(100.0, 100.0), 20.0, Abundance::Unlimited);
        assert_eq!(world.food_zones.len(), 1);

        world.add_food_zone(Vec2::new(500.0, 500.0), 15.0, Abundance::Scarce { current: 100.0, max: 100.0, regen: 0.1 });
        assert_eq!(world.food_zones.len(), 2);
    }

    #[test]
    fn test_world_has_buildings() {
        let mut world = World::new();
        assert_eq!(world.buildings.count(), 0);

        let id = world.spawn_building(BuildingType::House, Vec2::new(50.0, 50.0));

        assert_eq!(world.buildings.count(), 1);
        assert_eq!(world.buildings.index_of(id), Some(0));
    }

    #[test]
    fn test_spawn_building_returns_unique_ids() {
        let mut world = World::new();

        let id1 = world.spawn_building(BuildingType::House, Vec2::new(0.0, 0.0));
        let id2 = world.spawn_building(BuildingType::Farm, Vec2::new(10.0, 10.0));
        let id3 = world.spawn_building(BuildingType::Wall, Vec2::new(20.0, 20.0));

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);
        assert_eq!(world.buildings.count(), 3);
    }
}
