//! Combat stance system
//!
//! Combat is pressure and timing, not turns. Stances determine
//! what actions are available and who strikes first.

use serde::{Deserialize, Serialize};

/// Combat stance - every combatant is always in exactly one stance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CombatStance {
    /// Attacking, has initiative
    Pressing,
    /// Balanced, can attack or defend
    #[default]
    Neutral,
    /// Focused on blocking/parrying
    Defensive,
    /// Catching breath, vulnerable
    Recovering,
    /// Out of fight (wounded/fled)
    Broken,
}

impl CombatStance {
    /// Can this stance initiate an attack?
    pub fn can_attack(&self) -> bool {
        matches!(self, CombatStance::Pressing | CombatStance::Neutral)
    }

    /// Can this stance perform active defense?
    pub fn can_defend(&self) -> bool {
        matches!(self, CombatStance::Neutral | CombatStance::Defensive)
    }

    /// Is this stance vulnerable to free hits?
    pub fn vulnerable(&self) -> bool {
        matches!(self, CombatStance::Recovering | CombatStance::Broken)
    }
}

/// Events that trigger stance transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransitionTrigger {
    // Self-initiated
    InitiateAttack,
    RaiseGuard,
    DropGuard,
    CatchBreath,

    // Combat outcomes
    AttackCompleted,
    AttackBlocked,
    AttackMissed,
    DefenseSucceeded,
    DefenseFailed,
    TookHit,
    Staggered,
    Knockdown,

    // Fatigue
    Exhausted,
    Recovered,

    // Incapacitation (leads to Broken)
    CriticalWoundHead,
    CriticalWoundTorso,
    MoraleBreak,
    WoundThresholdExceeded,
}

/// Stance transition rules (state machine)
pub struct StanceTransitions;

impl StanceTransitions {
    pub fn new() -> Self {
        Self
    }

    /// Apply a transition trigger to get the next stance
    pub fn apply(&self, current: CombatStance, trigger: TransitionTrigger) -> CombatStance {
        use CombatStance::*;
        use TransitionTrigger::*;

        match (current, trigger) {
            // Self-initiated transitions
            (Neutral, InitiateAttack) => Pressing,
            (Neutral, RaiseGuard) => Defensive,
            (Defensive, DropGuard) => Neutral,
            (_, CatchBreath) => Recovering,

            // Attack outcomes
            (Pressing, AttackCompleted) => Neutral,
            (Pressing, AttackBlocked) => Neutral,
            (Pressing, AttackMissed) => Recovering, // Overextended

            // Defense outcomes
            (Defensive, DefenseSucceeded) => Neutral,
            (Defensive, DefenseFailed) => Recovering,

            // Taking damage
            (_, TookHit) => Recovering,
            (_, Staggered) => Recovering,
            (_, Knockdown) => Recovering,

            // Fatigue
            (_, Exhausted) => Recovering,
            (Recovering, Recovered) => Neutral,

            // Incapacitation - leads to Broken (out of fight)
            (_, CriticalWoundHead) => Broken,
            (_, CriticalWoundTorso) => Broken,
            (_, MoraleBreak) => Broken,
            (_, WoundThresholdExceeded) => Broken,

            // No change for invalid transitions
            _ => current,
        }
    }
}

impl Default for StanceTransitions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pressing_can_attack() {
        assert!(CombatStance::Pressing.can_attack());
        assert!(CombatStance::Neutral.can_attack());
        assert!(!CombatStance::Defensive.can_attack());
        assert!(!CombatStance::Recovering.can_attack());
    }

    #[test]
    fn test_recovering_is_vulnerable() {
        assert!(CombatStance::Recovering.vulnerable());
        assert!(CombatStance::Broken.vulnerable());
        assert!(!CombatStance::Pressing.vulnerable());
    }

    #[test]
    fn test_stance_transitions() {
        let transitions = StanceTransitions::new();

        let next = transitions.apply(CombatStance::Neutral, TransitionTrigger::InitiateAttack);
        assert_eq!(next, CombatStance::Pressing);

        let next = transitions.apply(CombatStance::Pressing, TransitionTrigger::AttackMissed);
        assert_eq!(next, CombatStance::Recovering);
    }

    #[test]
    fn test_recovery_cycle() {
        let transitions = StanceTransitions::new();

        // Get hit -> Recovering
        let stance = transitions.apply(CombatStance::Neutral, TransitionTrigger::TookHit);
        assert_eq!(stance, CombatStance::Recovering);

        // Recover -> Neutral
        let stance = transitions.apply(CombatStance::Recovering, TransitionTrigger::Recovered);
        assert_eq!(stance, CombatStance::Neutral);
    }

    #[test]
    fn test_incapacitation_leads_to_broken() {
        let transitions = StanceTransitions::new();

        // Critical head wound -> Broken
        let stance = transitions.apply(CombatStance::Pressing, TransitionTrigger::CriticalWoundHead);
        assert_eq!(stance, CombatStance::Broken);

        // Critical torso wound -> Broken
        let stance = transitions.apply(CombatStance::Neutral, TransitionTrigger::CriticalWoundTorso);
        assert_eq!(stance, CombatStance::Broken);

        // Morale break -> Broken
        let stance = transitions.apply(CombatStance::Defensive, TransitionTrigger::MoraleBreak);
        assert_eq!(stance, CombatStance::Broken);

        // Wound threshold exceeded -> Broken
        let stance =
            transitions.apply(CombatStance::Recovering, TransitionTrigger::WoundThresholdExceeded);
        assert_eq!(stance, CombatStance::Broken);
    }
}
