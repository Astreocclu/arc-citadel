//! Enemy AI system for battle decision-making
//!
//! Architecture: Trait + Data hybrid
//! - BattleAI trait defines interface for swappable implementations
//! - AiPersonality struct holds TOML-loaded weights/preferences
//! - DecisionContext provides fog-of-war-filtered battle state

// Submodules will be added in later tasks:
// mod commander;
// mod decision_context;
// mod personality;
// mod phase_plans;

// Re-exports will be added as modules are implemented:
// pub use commander::AiCommander;
// pub use decision_context::DecisionContext;
// pub use personality::{AiPersonality, load_personality};
// pub use phase_plans::{PhasePlan, PhaseTransition};

use crate::battle::courier::Order;
use crate::battle::execution::BattleEventLog;
use crate::core::types::Tick;

/// Placeholder for AiPersonality until personality module is created
#[derive(Debug, Clone, Default)]
pub struct AiPersonality {
    pub name: String,
}

/// Placeholder for DecisionContext until decision_context module is created
pub struct DecisionContext<'a> {
    _marker: std::marker::PhantomData<&'a ()>,
}

/// Trait for battle AI implementations
pub trait BattleAI {
    /// Process a single tick - returns orders to dispatch via courier
    fn process_tick(
        &mut self,
        context: &DecisionContext,
        current_tick: Tick,
        events: &mut BattleEventLog,
    ) -> Vec<Order>;

    /// Get the personality configuration
    fn personality(&self) -> &AiPersonality;

    /// Check if AI cheats on fog of war
    fn ignores_fog_of_war(&self) -> bool;
}
