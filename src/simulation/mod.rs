//! Simulation core - orchestrates the perception → thought → action → execution loop.
//!
//! This is the beating heart of Arc Citadel, containing 12,881 lines of core game logic.
//!
//! ## Key Components
//!
//! - `tick.rs` (4405 LOC) - Main simulation tick orchestration
//! - `action_select.rs` (6121 LOC) - Action selection for all species
//! - `perception.rs` - Spatial perception and entity awareness
//! - `violation_detection.rs` (601 LOC) - Behavioral pattern violation detection
//!
//! ## Tick Execution Order
//!
//! ```text
//! 1. update_needs()       - Needs decay over time
//! 2. perception_system()  - Build spatial index, query neighbors
//! 3. generate_thoughts()  - React to perceptions
//! 4. decay_thoughts()     - Fade old thoughts
//! 5. select_actions()     - Choose actions for idle entities
//! 6. execute_tasks()      - Progress current tasks
//! 7. resolve_combat()     - Combat resolution (line ~2280)
//! 8. world.tick()         - Advance time
//! ```
//!
//! ## Integration
//!
//! The simulation module integrates with:
//! - `entity/` - Reads/writes needs, thoughts, tasks
//! - `spatial/` - Uses SparseHashGrid for neighbor queries
//! - `combat/` - resolve_exchange() for combat resolution
//! - `ecs/` - World state management

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
pub use tick::{check_win_condition, GameOutcome, SimulationEvent};
pub use value_dynamics::{apply_event, apply_tick_dynamics};
pub use violation_detection::{
    check_pattern_violation, check_violations, process_violations, ViolationType,
};
