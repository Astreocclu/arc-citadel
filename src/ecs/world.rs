//! ECS World - manages all entities and their components

use crate::blueprints::BlueprintRegistry;
use crate::city::building::{BuildingArchetype, BuildingId, BuildingType};
use crate::city::stockpile::Stockpile;
use crate::core::astronomy::AstronomicalState;
use crate::core::types::{EntityId, Species, Vec2};
use crate::entity::species::human::HumanArchetype;
use crate::entity::species::orc::OrcArchetype;
use crate::rules::SpeciesRules;
use crate::simulation::resource_zone::ResourceZone;
use crate::world::{BlockedCells, LoadError, PlacementLoader, WorldObjects};
use ahash::AHashMap;
use std::path::Path;

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
        if let Abundance::Scarce {
            current,
            max,
            regen,
        } = &mut self.abundance
        {
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
    /// Global stockpile for resources (MVP - later per-settlement)
    pub stockpile: Stockpile,
    /// World objects (walls, trees, etc.)
    pub world_objects: WorldObjects,
    /// Blocked cells for pathfinding
    pub blocked_cells: BlockedCells,
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
        let species_rules = crate::rules::load_species_rules(species_dir).unwrap_or_else(|e| {
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
            stockpile: Stockpile::new(),
            world_objects: WorldObjects::new(),
            blocked_cells: BlockedCells::new(),
        }
    }

    pub fn add_food_zone(&mut self, position: Vec2, radius: f32, abundance: Abundance) -> u32 {
        let id = self.next_food_zone_id;
        self.next_food_zone_id += 1;
        self.food_zones.push(FoodZone {
            id,
            position,
            radius,
            abundance,
        });
        id
    }

    pub fn spawn_human(&mut self, name: String) -> EntityId {
        let entity_id = EntityId::new();
        let index = *self.next_indices.get(&Species::Human).unwrap();

        self.humans.spawn(entity_id, name, self.current_tick);

        self.entity_registry
            .insert(entity_id, (Species::Human, index));
        *self.next_indices.get_mut(&Species::Human).unwrap() += 1;

        entity_id
    }

    pub fn spawn_orc(&mut self, name: String) -> EntityId {
        let entity_id = EntityId::new();
        let index = *self.next_indices.get(&Species::Orc).unwrap();

        self.orcs.spawn(entity_id, name, self.current_tick);

        self.entity_registry
            .insert(entity_id, (Species::Orc, index));
        *self.next_indices.get_mut(&Species::Orc).unwrap() += 1;

        entity_id
    }

    /// Spawn a new building at the given position
    pub fn spawn_building(&mut self, building_type: BuildingType, position: Vec2) -> BuildingId {
        let id = BuildingId::new();
        self.buildings
            .spawn(id, building_type, position, self.current_tick);
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

    /// Load world objects from JSON placement data
    pub fn load_world_objects_json(&mut self, json: &str) -> Result<usize, LoadError> {
        // Create a temporary registry for loading
        let mut registry = BlueprintRegistry::new();
        let data_path = Path::new("data/blueprints");
        if data_path.exists() {
            if let Err(e) = registry.load_directory(data_path) {
                eprintln!("Warning: Failed to load blueprints: {}", e);
            }
        }

        let loader = PlacementLoader::new(&registry);
        let objects = loader.load_from_json(json)?;

        let count = objects.len();

        // Update blocked cells for objects that block movement
        for obj in objects.iter() {
            if obj.military.blocks_movement && !obj.geometry.footprint.is_empty() {
                // Transform footprint to world space
                let world_footprint: Vec<glam::Vec2> = obj
                    .geometry
                    .footprint
                    .iter()
                    .map(|v| {
                        // Rotate and translate
                        let rotated = glam::Vec2::new(
                            v.x * obj.rotation.cos() - v.y * obj.rotation.sin(),
                            v.x * obj.rotation.sin() + v.y * obj.rotation.cos(),
                        );
                        rotated + obj.position
                    })
                    .collect();

                self.blocked_cells.block_footprint(&world_footprint, 1.0);
            }
        }

        self.world_objects = objects;
        Ok(count)
    }

    /// Load world objects from a file path
    pub fn load_world_objects_file(&mut self, path: &Path) -> Result<usize, LoadError> {
        let content = std::fs::read_to_string(path).map_err(LoadError::IoError)?;
        self.load_world_objects_json(&content)
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
    use crate::core::astronomy::Season;
    use crate::core::types::Vec2;

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

        world.add_food_zone(
            Vec2::new(500.0, 500.0),
            15.0,
            Abundance::Scarce {
                current: 100.0,
                max: 100.0,
                regen: 0.1,
            },
        );
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

    #[test]
    fn test_world_has_stockpile() {
        use crate::simulation::resource_zone::ResourceType;

        let mut world = World::new();

        // Stockpile should exist and be empty
        assert_eq!(world.stockpile.get(ResourceType::Food), 0);

        // Should be able to add resources
        world.stockpile.add(ResourceType::Food, 100);
        assert_eq!(world.stockpile.get(ResourceType::Food), 100);
    }

    #[test]
    fn test_world_has_objects() {
        let world = World::new();
        assert!(world.world_objects.is_empty());
    }

    #[test]
    fn test_world_has_blocked_cells() {
        let world = World::new();
        assert!(world.blocked_cells.is_empty());
    }

    #[test]
    fn test_load_world_objects() {
        let mut world = World::new();

        // Empty placements should work
        let json = r#"{
            "version": 1,
            "placements": []
        }"#;

        let result = world.load_world_objects_json(json);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
