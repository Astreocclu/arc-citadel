//! Simulation systems

mod generation;
mod expansion;
mod warfare;
mod diplomacy;
mod population;
mod resolution;

pub use generation::{generate_map, generate_polities, initialize_relations};
pub use expansion::find_expansion_targets;
pub use warfare::resolve_active_wars;
pub use diplomacy::decay_relations;
pub use population::update_populations;
pub use resolution::{resolve_event, event_priority, check_polity_viability, apply_cultural_drift};
