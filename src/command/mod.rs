//! Command execution pipeline
//!
//! Converts LLM ParsedIntent into executable Tasks:
//! ParsedIntent -> IntentResolver -> IntentResolution -> TaskCreator -> Vec<Task>

pub mod executor;
pub mod resolver;

pub use executor::CommandExecutor;
pub use resolver::{IntentResolution, IntentResolver, SubjectMatch};
