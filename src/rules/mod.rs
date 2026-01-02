//! Runtime species rules loaded from TOML

pub mod action_rules;
pub mod value_dynamics;
mod loader;

pub use action_rules::{ActionRule, IdleBehavior, SpeciesRuleSet, SpeciesRules};
pub use value_dynamics::{TickDelta, ValueEvent, SpeciesDynamics, ValueDynamicsRules};
pub use loader::{load_species_rules, load_species_dynamics};
