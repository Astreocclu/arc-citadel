//! Aggregate History Simulation
//!
//! World-generation module that simulates polity-level history.
//! Operates on ~1200 pseudo-node regions, not individual hexes.
//! Produces emergent history with species-authentic behavior.

pub mod behavior;
pub mod events;
pub mod hierarchy;
pub mod output;
pub mod polity;
pub mod region;
pub mod ruler;
pub mod simulation;
pub mod species;
pub mod systems;
pub mod world;

pub use events::{Event, EventType, HistoryLog};
pub use output::SimulationOutput;
pub use polity::{Polity, PolityType, SpeciesState};
pub use region::{Region, ResourceType, Terrain};
pub use ruler::Ruler;
pub use simulation::{simulate, SimulationConfig};
pub use world::AggregateWorld;
