//! Combat skill system
//!
//! CRITICAL: Skill determines WHICH actions are available, NOT bonuses.
//! NO percentage modifiers. NO damage multipliers. NO hit chance bonuses.
//!
//! Skill unlocks capabilities and affects timing, not numbers.

use serde::{Deserialize, Serialize};

/// Skill level - unlocks capabilities, doesn't add bonuses
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub enum SkillLevel {
    /// High variance, slow transitions, poor reads
    #[default]
    Novice,
    /// Moderate variance, decent transitions
    Trained,
    /// Low variance, good transitions, finds gaps
    Veteran,
    /// Minimal variance, instant transitions, exploits openings
    Master,
}

impl SkillLevel {
    /// Can attempt a riposte (counterattack after successful defense)?
    pub fn can_attempt_riposte(&self) -> bool {
        matches!(self, SkillLevel::Veteran | SkillLevel::Master)
    }

    /// Can target a specific body zone instead of random?
    pub fn can_target_specific_zone(&self) -> bool {
        matches!(self, SkillLevel::Trained | SkillLevel::Veteran | SkillLevel::Master)
    }

    /// Can feint (fake attack to create opening)?
    pub fn can_feint(&self) -> bool {
        matches!(self, SkillLevel::Master)
    }

    /// Can disarm opponent?
    pub fn can_disarm(&self) -> bool {
        matches!(self, SkillLevel::Veteran | SkillLevel::Master)
    }
}

/// Complete combat skill profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CombatSkill {
    /// Overall combat skill level
    pub level: SkillLevel,
    // NOTE: No numeric fields! Skill affects capability, not numbers.
}

impl CombatSkill {
    pub fn novice() -> Self {
        Self { level: SkillLevel::Novice }
    }

    pub fn trained() -> Self {
        Self { level: SkillLevel::Trained }
    }

    pub fn veteran() -> Self {
        Self { level: SkillLevel::Veteran }
    }

    pub fn master() -> Self {
        Self { level: SkillLevel::Master }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_novice_cannot_riposte() {
        assert!(!SkillLevel::Novice.can_attempt_riposte());
    }

    #[test]
    fn test_veteran_can_riposte() {
        assert!(SkillLevel::Veteran.can_attempt_riposte());
        assert!(SkillLevel::Master.can_attempt_riposte());
    }

    #[test]
    fn test_only_master_can_feint() {
        assert!(!SkillLevel::Novice.can_feint());
        assert!(!SkillLevel::Trained.can_feint());
        assert!(!SkillLevel::Veteran.can_feint());
        assert!(SkillLevel::Master.can_feint());
    }

    #[test]
    fn test_no_numeric_bonuses() {
        let _skill = SkillLevel::Master;
        // This test documents that SkillLevel has no numeric bonus methods
    }

    #[test]
    fn test_skill_ordering() {
        assert!(SkillLevel::Master > SkillLevel::Veteran);
        assert!(SkillLevel::Veteran > SkillLevel::Trained);
        assert!(SkillLevel::Trained > SkillLevel::Novice);
    }
}
