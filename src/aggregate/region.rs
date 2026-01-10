//! Region - pseudo-node representing strategic territory

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::types::Species;

/// A pseudo-node representing a strategic region (~100+ hexes when expanded)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Region {
    pub id: u32,
    pub name: String,

    // Geography
    pub terrain: Terrain,
    pub resources: ResourceType,
    pub neighbors: Vec<u32>,

    // Species fitness (0.0 to 1.0) - how suitable for each species
    pub fitness: HashMap<Species, f32>,

    // Ownership
    pub controller: Option<u32>,
    pub contested_by: Vec<u32>,

    // Population capacity
    pub max_population: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Terrain {
    Mountain,
    Forest,
    Plains,
    Marsh,
    Coast,
    Desert,
    Hills,
    River,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    None,
    Iron,
    Gold,
    Timber,
    Grain,
    Stone,
    Fish,
    Gems,
}

impl Region {
    /// Calculate species fitness based on terrain
    pub fn calculate_fitness(terrain: Terrain) -> HashMap<Species, f32> {
        let mut fitness = HashMap::new();

        match terrain {
            Terrain::Mountain => {
                fitness.insert(Species::Dwarf, 1.0);
                fitness.insert(Species::Elf, 0.1);
                fitness.insert(Species::Human, 0.2);
                fitness.insert(Species::Orc, 0.6);
                fitness.insert(Species::Kobold, 0.9);
                fitness.insert(Species::Gnoll, 0.3);
                fitness.insert(Species::Lizardfolk, 0.1);
                fitness.insert(Species::Hobgoblin, 0.5);
                fitness.insert(Species::Ogre, 0.7);
                fitness.insert(Species::Harpy, 0.95);
                fitness.insert(Species::Centaur, 0.3);
                fitness.insert(Species::Minotaur, 0.8);
                fitness.insert(Species::Satyr, 0.4);
                fitness.insert(Species::Dryad, 0.1);
                fitness.insert(Species::Goblin, 0.8);
                fitness.insert(Species::Troll, 0.4);
                fitness.insert(Species::AbyssalDemons, 0.8);
                fitness.insert(Species::Elemental, 0.8);
                fitness.insert(Species::Fey, 0.2);
                fitness.insert(Species::StoneGiants, 0.9);
                fitness.insert(Species::Golem, 0.9);
                fitness.insert(Species::Merfolk, 0.1);
                fitness.insert(Species::Naga, 0.8);
                fitness.insert(Species::Revenant, 0.3);
                fitness.insert(Species::Vampire, 0.8);
                fitness.insert(Species::Lupine, 0.8);
                // CODEGEN: species_fitness_mountain
            }
            Terrain::Hills => {
                fitness.insert(Species::Dwarf, 0.8);
                fitness.insert(Species::Elf, 0.5);
                fitness.insert(Species::Human, 0.7);
                fitness.insert(Species::Orc, 0.8);
                fitness.insert(Species::Kobold, 0.8);
                fitness.insert(Species::Gnoll, 0.6);
                fitness.insert(Species::Lizardfolk, 0.2);
                fitness.insert(Species::Hobgoblin, 0.7);
                fitness.insert(Species::Ogre, 0.8);
                fitness.insert(Species::Harpy, 0.8);
                fitness.insert(Species::Centaur, 0.6);
                fitness.insert(Species::Minotaur, 0.7);
                fitness.insert(Species::Satyr, 0.6);
                fitness.insert(Species::Dryad, 0.3);
                fitness.insert(Species::Goblin, 0.9);
                fitness.insert(Species::Troll, 0.7);
                fitness.insert(Species::AbyssalDemons, 0.7);
                fitness.insert(Species::Elemental, 0.6);
                fitness.insert(Species::Fey, 0.4);
                fitness.insert(Species::StoneGiants, 0.8);
                fitness.insert(Species::Golem, 0.8);
                fitness.insert(Species::Merfolk, 0.2);
                fitness.insert(Species::Naga, 0.7);
                fitness.insert(Species::Revenant, 0.5);
                fitness.insert(Species::Vampire, 0.7);
                fitness.insert(Species::Lupine, 0.9);
                // CODEGEN: species_fitness_hills
            }
            Terrain::Forest => {
                fitness.insert(Species::Dwarf, 0.2);
                fitness.insert(Species::Elf, 1.0);
                fitness.insert(Species::Human, 0.5);
                fitness.insert(Species::Orc, 0.5);
                fitness.insert(Species::Kobold, 0.5);
                fitness.insert(Species::Gnoll, 0.5);
                fitness.insert(Species::Lizardfolk, 0.5);
                fitness.insert(Species::Hobgoblin, 0.4);
                fitness.insert(Species::Ogre, 0.6);
                fitness.insert(Species::Harpy, 0.3);
                fitness.insert(Species::Centaur, 0.7);
                fitness.insert(Species::Minotaur, 0.4);
                fitness.insert(Species::Satyr, 0.9);
                fitness.insert(Species::Dryad, 0.99);
                fitness.insert(Species::Goblin, 0.7);
                fitness.insert(Species::Troll, 0.8);
                fitness.insert(Species::AbyssalDemons, 0.3);
                fitness.insert(Species::Elemental, 0.3);
                fitness.insert(Species::Fey, 0.9);
                fitness.insert(Species::StoneGiants, 0.3);
                fitness.insert(Species::Golem, 0.4);
                fitness.insert(Species::Merfolk, 0.1);
                fitness.insert(Species::Naga, 0.6);
                fitness.insert(Species::Revenant, 0.7);
                fitness.insert(Species::Vampire, 0.9);
                fitness.insert(Species::Lupine, 0.9);
                // CODEGEN: species_fitness_forest
            }
            Terrain::Plains => {
                fitness.insert(Species::Dwarf, 0.1);
                fitness.insert(Species::Elf, 0.3);
                fitness.insert(Species::Human, 0.9);
                fitness.insert(Species::Orc, 0.7);
                fitness.insert(Species::Kobold, 0.2);
                fitness.insert(Species::Gnoll, 0.9);
                fitness.insert(Species::Lizardfolk, 0.3);
                fitness.insert(Species::Hobgoblin, 0.8);
                fitness.insert(Species::Ogre, 0.5);
                fitness.insert(Species::Harpy, 0.4);
                fitness.insert(Species::Centaur, 0.95);
                fitness.insert(Species::Minotaur, 0.3);
                fitness.insert(Species::Satyr, 0.5);
                fitness.insert(Species::Dryad, 0.2);
                fitness.insert(Species::Goblin, 0.4);
                fitness.insert(Species::Troll, 0.3);
                fitness.insert(Species::AbyssalDemons, 0.4);
                fitness.insert(Species::Elemental, 0.5);
                fitness.insert(Species::Fey, 0.5);
                fitness.insert(Species::StoneGiants, 0.2);
                fitness.insert(Species::Golem, 0.5);
                fitness.insert(Species::Merfolk, 0.1);
                fitness.insert(Species::Naga, 0.3);
                fitness.insert(Species::Revenant, 0.6);
                fitness.insert(Species::Vampire, 0.3);
                fitness.insert(Species::Lupine, 0.6);
                // CODEGEN: species_fitness_plains
            }
            Terrain::Marsh => {
                fitness.insert(Species::Dwarf, 0.0);
                fitness.insert(Species::Elf, 0.2);
                fitness.insert(Species::Human, 0.3);
                fitness.insert(Species::Orc, 0.6);
                fitness.insert(Species::Kobold, 0.4);
                fitness.insert(Species::Gnoll, 0.4);
                fitness.insert(Species::Lizardfolk, 0.95);
                fitness.insert(Species::Hobgoblin, 0.3);
                fitness.insert(Species::Ogre, 0.7);
                fitness.insert(Species::Harpy, 0.2);
                fitness.insert(Species::Centaur, 0.2);
                fitness.insert(Species::Minotaur, 0.3);
                fitness.insert(Species::Satyr, 0.3);
                fitness.insert(Species::Dryad, 0.4);
                fitness.insert(Species::Goblin, 0.6);
                fitness.insert(Species::Troll, 0.9);
                fitness.insert(Species::AbyssalDemons, 0.9);
                fitness.insert(Species::Elemental, 0.2);
                fitness.insert(Species::Fey, 0.8);
                fitness.insert(Species::StoneGiants, 0.1);
                fitness.insert(Species::Golem, 0.2);
                fitness.insert(Species::Merfolk, 0.8);
                fitness.insert(Species::Naga, 0.9);
                fitness.insert(Species::Revenant, 0.9);
                fitness.insert(Species::Vampire, 0.6);
                fitness.insert(Species::Lupine, 0.2);
                // CODEGEN: species_fitness_marsh
            }
            Terrain::Coast => {
                fitness.insert(Species::Dwarf, 0.3);
                fitness.insert(Species::Elf, 0.4);
                fitness.insert(Species::Human, 0.9);
                fitness.insert(Species::Orc, 0.3);
                fitness.insert(Species::Kobold, 0.1);
                fitness.insert(Species::Gnoll, 0.3);
                fitness.insert(Species::Lizardfolk, 0.8);
                fitness.insert(Species::Hobgoblin, 0.5);
                fitness.insert(Species::Ogre, 0.3);
                fitness.insert(Species::Harpy, 0.7);
                fitness.insert(Species::Centaur, 0.4);
                fitness.insert(Species::Minotaur, 0.2);
                fitness.insert(Species::Satyr, 0.5);
                fitness.insert(Species::Dryad, 0.2);
                fitness.insert(Species::Goblin, 0.3);
                fitness.insert(Species::Troll, 0.8);
                fitness.insert(Species::AbyssalDemons, 0.6);
                fitness.insert(Species::Elemental, 0.7);
                fitness.insert(Species::Fey, 0.3);
                fitness.insert(Species::StoneGiants, 0.4);
                fitness.insert(Species::Golem, 0.3);
                fitness.insert(Species::Merfolk, 0.9);
                fitness.insert(Species::Naga, 0.5);
                fitness.insert(Species::Revenant, 0.4);
                fitness.insert(Species::Vampire, 0.4);
                fitness.insert(Species::Lupine, 0.3);
                // CODEGEN: species_fitness_coast
            }
            Terrain::Desert => {
                fitness.insert(Species::Dwarf, 0.2);
                fitness.insert(Species::Elf, 0.1);
                fitness.insert(Species::Human, 0.4);
                fitness.insert(Species::Orc, 0.5);
                fitness.insert(Species::Kobold, 0.3);
                fitness.insert(Species::Gnoll, 0.7);
                fitness.insert(Species::Lizardfolk, 0.2);
                fitness.insert(Species::Hobgoblin, 0.6);
                fitness.insert(Species::Ogre, 0.4);
                fitness.insert(Species::Harpy, 0.5);
                fitness.insert(Species::Centaur, 0.5);
                fitness.insert(Species::Minotaur, 0.5);
                fitness.insert(Species::Satyr, 0.1);
                fitness.insert(Species::Dryad, 0.0);
                fitness.insert(Species::Goblin, 0.2);
                fitness.insert(Species::Troll, 0.1);
                fitness.insert(Species::AbyssalDemons, 0.8);
                fitness.insert(Species::Elemental, 0.9);
                fitness.insert(Species::Fey, 0.1);
                fitness.insert(Species::StoneGiants, 0.1);
                fitness.insert(Species::Golem, 0.7);
                fitness.insert(Species::Merfolk, 0.1);
                fitness.insert(Species::Naga, 0.2);
                fitness.insert(Species::Revenant, 0.2);
                fitness.insert(Species::Vampire, 0.1);
                fitness.insert(Species::Lupine, 0.1);
                // CODEGEN: species_fitness_desert
            }
            Terrain::River => {
                fitness.insert(Species::Dwarf, 0.4);
                fitness.insert(Species::Elf, 0.6);
                fitness.insert(Species::Human, 1.0);
                fitness.insert(Species::Orc, 0.4);
                fitness.insert(Species::Kobold, 0.3);
                fitness.insert(Species::Gnoll, 0.5);
                fitness.insert(Species::Lizardfolk, 0.9);
                fitness.insert(Species::Hobgoblin, 0.6);
                fitness.insert(Species::Ogre, 0.4);
                fitness.insert(Species::Harpy, 0.3);
                fitness.insert(Species::Centaur, 0.5);
                fitness.insert(Species::Minotaur, 0.3);
                fitness.insert(Species::Satyr, 0.7);
                fitness.insert(Species::Dryad, 0.5);
                fitness.insert(Species::Goblin, 0.5);
                fitness.insert(Species::Troll, 0.9);
                fitness.insert(Species::AbyssalDemons, 0.2);
                fitness.insert(Species::Elemental, 0.4);
                fitness.insert(Species::Fey, 0.7);
                fitness.insert(Species::StoneGiants, 0.5);
                fitness.insert(Species::Golem, 0.3);
                fitness.insert(Species::Merfolk, 0.7);
                fitness.insert(Species::Naga, 0.7);
                fitness.insert(Species::Revenant, 0.1);
                fitness.insert(Species::Vampire, 0.5);
                fitness.insert(Species::Lupine, 0.7);
                // CODEGEN: species_fitness_river
            }
        }

        fitness
    }
}
