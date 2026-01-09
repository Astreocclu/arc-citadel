//! Combat skill system
//!
//! CRITICAL: Skill determines WHICH actions are available, NOT bonuses.
//! NO percentage modifiers. NO damage multipliers. NO hit chance bonuses.
//!
//! Skill unlocks capabilities and affects timing, not numbers.

use serde::{Deserialize, Serialize};

/// Skill level - unlocks capabilities, doesn't add bonuses
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
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
        matches!(
            self,
            SkillLevel::Trained | SkillLevel::Veteran | SkillLevel::Master
        )
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
        Self {
            level: SkillLevel::Novice,
        }
    }

    pub fn trained() -> Self {
        Self {
            level: SkillLevel::Trained,
        }
    }

    pub fn veteran() -> Self {
        Self {
            level: SkillLevel::Veteran,
        }
    }

    pub fn master() -> Self {
        Self {
            level: SkillLevel::Master,
        }
    }

    /// Derive combat skill from chunk library
    ///
    /// Uses highest encoding depth across all chunks to determine skill level:
    /// - < 0.3 → Novice
    /// - < 0.6 → Trained
    /// - < 0.85 → Veteran
    /// - >= 0.85 → Master
    pub fn from_chunk_library(library: &crate::skills::ChunkLibrary) -> Self {
        // Use the highest encoding depth to represent overall mastery
        let max_depth = library.chunks()
            .values()
            .map(|s| s.encoding_depth)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);

        let level = if max_depth >= 0.85 {
            SkillLevel::Master
        } else if max_depth >= 0.6 {
            SkillLevel::Veteran
        } else if max_depth >= 0.3 {
            SkillLevel::Trained
        } else {
            SkillLevel::Novice
        };

        Self { level }
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

    #[test]
    fn test_skill_from_chunk_library() {
        use crate::skills::ChunkLibrary;

        // Empty library = Novice
        let lib = ChunkLibrary::new();
        let skill = CombatSkill::from_chunk_library(&lib);
        assert_eq!(skill.level, SkillLevel::Novice);

        // Trained soldier - has BasicStance at 0.7 depth
        let lib = ChunkLibrary::trained_soldier(0);
        let skill = CombatSkill::from_chunk_library(&lib);
        assert!(skill.level >= SkillLevel::Trained);

        // Veteran - has BasicStance at 0.9 depth (Master threshold is 0.85)
        let lib = ChunkLibrary::veteran(0);
        let skill = CombatSkill::from_chunk_library(&lib);
        assert!(skill.level >= SkillLevel::Veteran);
    }
}
