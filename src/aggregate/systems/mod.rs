//! Simulation systems

mod diplomacy;
pub mod expansion;
mod generation;
mod population;
mod resolution;
mod warfare;

pub use diplomacy::decay_relations;
pub use expansion::{calculate_human_expansion_pressure, find_expansion_targets};
pub use generation::{generate_map, generate_polities, initialize_relations};
pub use population::update_populations;
pub use resolution::{apply_cultural_drift, check_polity_viability, event_priority, resolve_event};
pub use warfare::resolve_active_wars;
