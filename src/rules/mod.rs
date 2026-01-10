//! Runtime species rules loaded from TOML

pub mod action_rules;
mod loader;
pub mod value_dynamics;

pub use action_rules::{ActionRule, IdleBehavior, SpeciesRuleSet, SpeciesRules};
pub use loader::{load_species_dynamics, load_species_rules};
pub use value_dynamics::{SpeciesDynamics, TickDelta, ValueDynamicsRules, ValueEvent};
