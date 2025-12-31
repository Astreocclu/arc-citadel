//! Species-specific behavior

mod human;
mod dwarf;
mod elf;

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
    }
}
