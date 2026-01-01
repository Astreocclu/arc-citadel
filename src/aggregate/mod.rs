//! Aggregate History Simulation
//!
//! World-generation module that simulates polity-level history.
//! Operates on ~1200 pseudo-node regions, not individual hexes.
//! Produces emergent history with species-authentic behavior.

pub mod world;
pub mod region;
pub mod polity;
pub mod ruler;
pub mod simulation;
pub mod events;
pub mod output;
pub mod systems;
pub mod species;

pub use world::AggregateWorld;
pub use region::{Region, Terrain, ResourceType};
pub use polity::{Polity, PolityType, SpeciesState};
// pub use ruler::Ruler;  // TODO: Uncomment when Ruler struct is implemented
pub use simulation::{simulate, SimulationConfig};
pub use events::{Event, EventType, HistoryLog};
pub use output::SimulationOutput;
