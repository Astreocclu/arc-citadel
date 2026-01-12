//! Multi-phase battle planning
//!
//! AI can have different strategies for different battle phases:
//! - Opening: probing, positioning
//! - Main assault: committed attack
//! - Exploitation: pursuit of routed enemies
//! - Withdrawal: organized retreat

use serde::{Deserialize, Serialize};

use crate::battle::hex::BattleHexCoord;
use crate::core::types::Tick;

/// Condition that triggers phase transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhaseTransition {
    /// Transition after N ticks
    TimeElapsed(Tick),
    /// Transition when strength ratio falls below threshold
    StrengthRatioBelow(f32),
    /// Transition when casualties exceed percentage
    CasualtiesExceed(f32),
    /// Manual transition only (via contingency or go-code)
    Manual,
    /// Never transition (final phase)
    Never,
}

impl PhaseTransition {
    /// Check if transition condition is met
    pub fn is_triggered(&self, current_tick: Tick, strength_ratio: f32, casualties: f32) -> bool {
        match self {
            PhaseTransition::TimeElapsed(tick) => current_tick >= *tick,
            PhaseTransition::StrengthRatioBelow(threshold) => strength_ratio < *threshold,
            PhaseTransition::CasualtiesExceed(threshold) => casualties > *threshold,
            PhaseTransition::Manual => false,
            PhaseTransition::Never => false,
        }
    }
}

/// A phase of the battle plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhasePlan {
    /// Name of this phase
    pub name: String,
    /// Priority target positions
    pub priority_targets: Vec<BattleHexCoord>,
    /// How much of reserves to commit (0.0 to 1.0)
    pub reserve_commitment: f32,
    /// Modifier to base aggression (-1.0 to 1.0)
    pub aggression_modifier: f32,
    /// Condition to transition to next phase
    pub transition: PhaseTransition,
}

impl Default for PhasePlan {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            priority_targets: Vec::new(),
            reserve_commitment: 0.0,
            aggression_modifier: 0.0,
            transition: PhaseTransition::Never,
        }
    }
}

/// Manager for multi-phase battle plans
#[derive(Debug, Clone, Default)]
pub struct PhasePlanManager {
    phases: Vec<PhasePlan>,
    current_phase_index: usize,
    phase_start_tick: Tick,
}

impl PhasePlanManager {
    pub fn new() -> Self {
        Self {
            phases: vec![PhasePlan::default()],
            current_phase_index: 0,
            phase_start_tick: 0,
        }
    }

    /// Add a phase to the plan
    pub fn add_phase(&mut self, phase: PhasePlan) {
        // If only default phase exists, replace it
        if self.phases.len() == 1 && self.phases[0].name == "Default" {
            self.phases[0] = phase;
        } else {
            self.phases.push(phase);
        }
    }

    /// Get current phase
    pub fn current_phase(&self) -> &PhasePlan {
        &self.phases[self.current_phase_index]
    }

    /// Update and potentially transition phases
    pub fn update(&mut self, current_tick: Tick, strength_ratio: f32, casualties: f32) {
        let ticks_in_phase = current_tick.saturating_sub(self.phase_start_tick);

        if self
            .current_phase()
            .transition
            .is_triggered(ticks_in_phase, strength_ratio, casualties)
        {
            self.advance_phase(current_tick);
        }
    }

    /// Advance to next phase
    fn advance_phase(&mut self, current_tick: Tick) {
        if self.current_phase_index < self.phases.len() - 1 {
            self.current_phase_index += 1;
            self.phase_start_tick = current_tick;
        }
    }

    /// Force transition to next phase
    pub fn force_advance(&mut self, current_tick: Tick) {
        self.advance_phase(current_tick);
    }

    /// Check if in final phase
    pub fn is_final_phase(&self) -> bool {
        self.current_phase_index >= self.phases.len() - 1
    }

    /// Get phase index
    pub fn phase_index(&self) -> usize {
        self.current_phase_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_transition() {
        let transition = PhaseTransition::TimeElapsed(100);
        assert!(transition.is_triggered(100, 0.5, 0.1));
        assert!(!transition.is_triggered(50, 0.5, 0.1));
    }

    #[test]
    fn test_strength_ratio_transition() {
        let transition = PhaseTransition::StrengthRatioBelow(0.5);
        assert!(transition.is_triggered(0, 0.3, 0.1));
        assert!(!transition.is_triggered(0, 0.7, 0.1));
    }

    #[test]
    fn test_casualties_transition() {
        let transition = PhaseTransition::CasualtiesExceed(0.5);
        assert!(transition.is_triggered(0, 1.0, 0.6));
        assert!(!transition.is_triggered(0, 1.0, 0.3));
    }

    #[test]
    fn test_never_transition() {
        let transition = PhaseTransition::Never;
        assert!(!transition.is_triggered(1000, 0.0, 1.0));
    }

    #[test]
    fn test_manual_transition() {
        let transition = PhaseTransition::Manual;
        assert!(!transition.is_triggered(1000, 0.0, 1.0));
    }

    #[test]
    fn test_phase_plan_manager_advances() {
        let mut manager = PhasePlanManager::new();
        manager.add_phase(PhasePlan {
            name: "Opening".to_string(),
            priority_targets: vec![],
            reserve_commitment: 0.0,
            aggression_modifier: 0.0,
            transition: PhaseTransition::TimeElapsed(10),
        });
        manager.add_phase(PhasePlan {
            name: "Main".to_string(),
            priority_targets: vec![],
            reserve_commitment: 0.5,
            aggression_modifier: 0.2,
            transition: PhaseTransition::Never,
        });

        assert_eq!(manager.current_phase().name, "Opening");

        manager.update(10, 1.0, 0.1);
        assert_eq!(manager.current_phase().name, "Main");
    }

    #[test]
    fn test_phase_stays_on_final() {
        let mut manager = PhasePlanManager::new();
        manager.add_phase(PhasePlan {
            name: "Only".to_string(),
            priority_targets: vec![],
            reserve_commitment: 0.0,
            aggression_modifier: 0.0,
            transition: PhaseTransition::TimeElapsed(1),
        });

        manager.update(100, 1.0, 0.0);
        assert!(manager.is_final_phase());
        assert_eq!(manager.current_phase().name, "Only");
    }

    #[test]
    fn test_force_advance() {
        let mut manager = PhasePlanManager::new();
        manager.add_phase(PhasePlan {
            name: "First".to_string(),
            priority_targets: vec![],
            reserve_commitment: 0.0,
            aggression_modifier: 0.0,
            transition: PhaseTransition::Never,
        });
        manager.add_phase(PhasePlan {
            name: "Second".to_string(),
            priority_targets: vec![],
            reserve_commitment: 0.0,
            aggression_modifier: 0.0,
            transition: PhaseTransition::Never,
        });

        assert_eq!(manager.current_phase().name, "First");
        manager.force_advance(50);
        assert_eq!(manager.current_phase().name, "Second");
    }

    #[test]
    fn test_phase_index() {
        let mut manager = PhasePlanManager::new();
        manager.add_phase(PhasePlan {
            name: "First".to_string(),
            priority_targets: vec![],
            reserve_commitment: 0.0,
            aggression_modifier: 0.0,
            transition: PhaseTransition::TimeElapsed(5),
        });
        manager.add_phase(PhasePlan {
            name: "Second".to_string(),
            priority_targets: vec![],
            reserve_commitment: 0.0,
            aggression_modifier: 0.0,
            transition: PhaseTransition::Never,
        });

        assert_eq!(manager.phase_index(), 0);
        manager.update(10, 1.0, 0.0);
        assert_eq!(manager.phase_index(), 1);
    }
}
