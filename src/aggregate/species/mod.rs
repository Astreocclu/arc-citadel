//! Species-specific behavior

mod human;
mod dwarf;
mod elf;
mod orc;
mod kobold;
mod gnoll;
mod lizardfolk;
mod hobgoblin;
mod ogre;
mod harpy;
mod centaur;
mod minotaur;
mod satyr;
mod dryad;
mod goblin;
mod troll;
// CODEGEN: species_behavior_mods

use crate::aggregate::polity::Polity;
use crate::aggregate::world::AggregateWorld;
use crate::aggregate::events::EventType;
use crate::core::types::Species;

/// Generate events for a polity based on its species
pub fn tick(polity: &Polity, world: &AggregateWorld, year: u32) -> Vec<EventType> {
    match polity.species {
        Species::Human => human::tick(polity, world, year),
        Species::Dwarf => dwarf::tick(polity, world, year),
        Species::Elf => elf::tick(polity, world, year),
        Species::Orc => orc::tick(polity, world, year),
        Species::Kobold => kobold::tick(polity, world, year),
        Species::Gnoll => gnoll::tick(polity, world, year),
        Species::Lizardfolk => lizardfolk::tick(polity, world, year),
        Species::Hobgoblin => hobgoblin::tick(polity, world, year),
        Species::Ogre => ogre::tick(polity, world, year),
        Species::Harpy => harpy::tick(polity, world, year),
        Species::Centaur => centaur::tick(polity, world, year),
        Species::Minotaur => minotaur::tick(polity, world, year),
        Species::Satyr => satyr::tick(polity, world, year),
        Species::Dryad => dryad::tick(polity, world, year),
        Species::Goblin => goblin::tick(polity, world, year),
        Species::Troll => troll::tick(polity, world, year),
        // CODEGEN: species_tick_arms
    }
}
