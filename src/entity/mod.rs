//! Entity components - the building blocks of simulated beings.
//!
//! This module contains all the components that make up an entity:
//! - `needs` - Universal needs (food, rest, safety, social, purpose)
//! - `thoughts` - Thought generation and memory
//! - `tasks` - Task queue and execution
//! - `body` - Physical state (fatigue, wounds)
//! - `species/` - Species-specific values and archetypes
//! - `relationships` - Inter-entity relationships
//! - `social/` - Social memory and group dynamics
//!
//! ## Architecture
//!
//! Entities use a Structure of Arrays (SoA) pattern for cache efficiency:
//!
//! ```text
//! pub struct HumanArchetype {
//!     pub ids: Vec<EntityId>,
//!     pub positions: Vec<Vec2>,
//!     pub needs: Vec<Needs>,
//!     pub thoughts: Vec<ThoughtBuffer>,
//!     pub values: Vec<HumanValues>,
//!     pub task_queues: Vec<TaskQueue>,
//!     // ... parallel arrays for all components
//! }
//! ```
//!
//! Access by index: `archetype.needs[idx]`, `archetype.values[idx]`

pub mod archetype;
pub mod body;

// Re-export deprecated types for backwards compatibility
// These will be removed once migration to skills::Role is complete
#[allow(deprecated)]
pub use archetype::{CraftSpecialty, EntityArchetype, TrainingLevel};
pub mod identity;
pub mod needs;
pub mod relationships;
pub mod social;
pub mod species;
pub mod tasks;
pub mod thoughts;
