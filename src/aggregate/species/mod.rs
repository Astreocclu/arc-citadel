//! Species-specific behavior

mod abyssal_demons;
mod centaur;
mod dryad;
mod dwarf;
mod elemental;
mod elf;
mod fey;
pub mod gnoll;
mod goblin;
mod golem;
mod harpy;
mod hobgoblin;
mod human;
pub mod kobold;
mod lizardfolk;
mod lupine;
mod merfolk;
mod minotaur;
mod naga;
mod ogre;
mod orc;
mod revenant;
mod satyr;
mod stone_giants;
mod troll;
pub mod vampire;
// CODEGEN: species_behavior_mods

use crate::aggregate::events::EventType;
use crate::aggregate::polity::Polity;
use crate::aggregate::world::AggregateWorld;
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
        Species::AbyssalDemons => abyssal_demons::tick(polity, world, year),
        Species::Elemental => elemental::tick(polity, world, year),
        Species::Fey => fey::tick(polity, world, year),
        Species::StoneGiants => stone_giants::tick(polity, world, year),
        Species::Golem => golem::tick(polity, world, year),
        Species::Merfolk => merfolk::tick(polity, world, year),
        Species::Naga => naga::tick(polity, world, year),
        Species::Revenant => revenant::tick(polity, world, year),
        Species::Vampire => vampire::tick(polity, world, year),
        Species::Lupine => lupine::tick(polity, world, year),
        // CODEGEN: species_tick_arms
    }
}
