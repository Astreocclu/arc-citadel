pub mod action_execute;
pub mod action_select;
pub mod consumption;
pub mod expectation_formation;
pub mod housing;
pub mod perception;
pub mod population;
pub mod resource_zone;
pub mod rule_eval;
pub mod thought_gen;
pub mod tick;
pub mod value_dynamics;
pub mod violation_detection;

pub use action_select::select_action_with_rules;
pub use expectation_formation::{
    infer_patterns_from_action, process_observations, record_observation,
};
pub use resource_zone::{ResourceType, ResourceZone};
pub use rule_eval::{evaluate_action_rules, select_idle_behavior};
pub use value_dynamics::{apply_event, apply_tick_dynamics};
pub use violation_detection::{
    check_pattern_violation, check_violations, process_violations, ViolationType,
};
