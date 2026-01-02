//! Combat state component for entities
//!
//! Every entity has combat state (mandatory but minimal).

use serde::{Deserialize, Serialize};
use crate::combat::{
    CombatStance, CombatSkill,
    MoraleState,
    WeaponProperties, ArmorProperties,
};

/// Combat state component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    /// Current combat stance
    pub stance: CombatStance,
    /// Combat skill profile
    pub skill: CombatSkill,
    /// Morale/stress state
    pub morale: MoraleState,
    /// Currently equipped weapon
    pub weapon: WeaponProperties,
    /// Currently worn armor
    pub armor: ArmorProperties,
    /// Combat fatigue (0.0 to 1.0)
    pub fatigue: f32,
}

impl Default for CombatState {
    fn default() -> Self {
        Self {
            stance: CombatStance::default(),
            skill: CombatSkill::default(),
            morale: MoraleState::default(),
            weapon: WeaponProperties::default(),
            armor: ArmorProperties::default(),
            fatigue: 0.0,
        }
    }
}

impl CombatState {
    /// Can this entity participate in combat?
    pub fn can_fight(&self) -> bool {
        !matches!(self.stance, CombatStance::Broken)
    }

    /// Is this entity actively in combat?
    pub fn in_combat(&self) -> bool {
        matches!(self.stance, CombatStance::Pressing | CombatStance::Defensive)
    }

    /// Apply fatigue (additive, clamped)
    pub fn add_fatigue(&mut self, amount: f32) {
        self.fatigue = (self.fatigue + amount).min(1.0);
    }

    /// Recover fatigue
    pub fn recover_fatigue(&mut self, amount: f32) {
        self.fatigue = (self.fatigue - amount).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_combat_state() {
        let state = CombatState::default();
        assert_eq!(state.stance, CombatStance::Neutral);
        assert_eq!(state.morale.current_stress, 0.0);
    }

    #[test]
    fn test_can_fight_when_healthy() {
        let state = CombatState::default();
        assert!(state.can_fight());
    }

    #[test]
    fn test_cannot_fight_when_broken() {
        let mut state = CombatState::default();
        state.stance = CombatStance::Broken;
        assert!(!state.can_fight());
    }

    #[test]
    fn test_fatigue_clamped() {
        let mut state = CombatState::default();
        state.add_fatigue(0.5);
        assert_eq!(state.fatigue, 0.5);

        state.add_fatigue(0.7);
        assert_eq!(state.fatigue, 1.0); // Clamped

        state.recover_fatigue(0.3);
        assert!((state.fatigue - 0.7).abs() < 0.001);
    }
}
