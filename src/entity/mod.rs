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
