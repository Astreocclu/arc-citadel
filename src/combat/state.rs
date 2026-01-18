//! Combat state component for entities
//!
//! Every entity has combat state (mandatory but minimal).

use crate::combat::{ArmorProperties, CombatSkill, CombatStance, MoraleState, WeaponProperties};
use crate::combat::wounds::Wound;
use crate::combat::body_zone::WoundSeverity;
use serde::{Deserialize, Serialize};

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
    /// Active wounds
    pub wounds: Vec<Wound>,
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
            wounds: Vec::new(),
        }
    }
}

impl CombatState {
    /// Can this entity participate in combat?
    pub fn can_fight(&self) -> bool {
        !matches!(self.stance, CombatStance::Broken) && !self.is_incapacitated()
    }

    /// Is this entity actively in combat?
    pub fn in_combat(&self) -> bool {
        matches!(
            self.stance,
            CombatStance::Pressing | CombatStance::Defensive
        )
    }

    /// Apply fatigue (additive, clamped)
    pub fn add_fatigue(&mut self, amount: f32) {
        self.fatigue = (self.fatigue + amount).min(1.0);
    }

    /// Recover fatigue
    pub fn recover_fatigue(&mut self, amount: f32) {
        self.fatigue = (self.fatigue - amount).max(0.0);
    }

    /// Check if entity is incapacitated (unable to fight)
    ///
    /// Incapacitation occurs from:
    /// - Any Critical wound
    /// - Accumulated wound severity (3+ Serious, 5+ Minor wounds, etc.)
    pub fn is_incapacitated(&self) -> bool {
        // Any single critical wound
        if self.wounds.iter().any(|w| w.severity >= WoundSeverity::Critical) {
            return true;
        }

        // Accumulating wounds - convert to severity points
        // Scratch=1, Minor=2, Serious=4
        // Incapacitated at 10+ points (e.g., 5 Minor, or 2 Serious + 1 Minor, etc.)
        let total_severity: u32 = self.wounds.iter().map(|w| {
            match w.severity {
                WoundSeverity::None => 0,
                WoundSeverity::Scratch => 1,
                WoundSeverity::Minor => 2,
                WoundSeverity::Serious => 4,
                WoundSeverity::Critical | WoundSeverity::Destroyed => 10, // Already handled above
            }
        }).sum();

        total_severity >= 10
    }

    /// Check if entity is dead
    pub fn is_dead(&self) -> bool {
        // Dead if destroyed or if head/neck/torso critical
        self.wounds.iter().any(|w| {
            w.severity == WoundSeverity::Destroyed ||
            (w.severity >= WoundSeverity::Critical && (w.zone == crate::combat::body_zone::BodyZone::Head || w.zone == crate::combat::body_zone::BodyZone::Neck || w.zone == crate::combat::body_zone::BodyZone::Torso))
        })
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
