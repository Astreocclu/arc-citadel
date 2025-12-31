//! ECS World - manages all entities and their components

use ahash::AHashMap;
use crate::core::types::{EntityId, Species};
use crate::entity::species::human::HumanArchetype;

/// The game world containing all entities
pub struct World {
    pub current_tick: u64,
    entity_registry: AHashMap<EntityId, (Species, usize)>,
    pub humans: HumanArchetype,
    next_indices: AHashMap<Species, usize>,
}

impl World {
    pub fn new() -> Self {
        let mut next_indices = AHashMap::new();
        next_indices.insert(Species::Human, 0);
        next_indices.insert(Species::Dwarf, 0);
        next_indices.insert(Species::Elf, 0);

        Self {
            current_tick: 0,
            entity_registry: AHashMap::new(),
            humans: HumanArchetype::new(),
            next_indices,
        }
    }

    pub fn spawn_human(&mut self, name: String) -> EntityId {
        let entity_id = EntityId::new();
        let index = *self.next_indices.get(&Species::Human).unwrap();

        self.humans.spawn(entity_id, name, self.current_tick);

        self.entity_registry.insert(entity_id, (Species::Human, index));
        *self.next_indices.get_mut(&Species::Human).unwrap() += 1;

        entity_id
    }

    pub fn get_entity_info(&self, entity_id: EntityId) -> Option<(Species, usize)> {
        self.entity_registry.get(&entity_id).copied()
    }

    pub fn entity_count(&self) -> usize {
        self.humans.count()
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
