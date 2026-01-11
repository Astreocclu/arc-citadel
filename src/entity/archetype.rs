//! Entity archetype definitions for spawn loadout generation

use serde::{Deserialize, Serialize};

/// High-level entity role determining spawn chunk loadout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityArchetype {
    /// Rural farmer/villager - basic physical labor, no specialization
    Peasant,
    /// Construction/hauling worker - strong physical skills
    Laborer,
    /// Skilled tradesperson with specialty
    Craftsman { specialty: CraftSpecialty },
    /// Military personnel with training level
    Soldier { training: TrainingLevel },
    /// Aristocrat - social, leadership, some combat
    Noble,
    /// Trader - social, assessment skills
    Merchant,
    /// Educated person - knowledge, teaching
    Scholar,
    /// Young person (age < 16) - only universal chunks
    Child,
}

/// Craft specialization for craftsmen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CraftSpecialty {
    Smithing,
    Carpentry,
    Masonry,
    Cooking,
    Tailoring,
    Leatherwork,
}

/// Military training level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrainingLevel {
    /// Farmers with spears
    Levy,
    /// Part-time trained
    Militia,
    /// Professional soldiers
    Regular,
    /// Battle-hardened
    Veteran,
    /// Best of the best
    Elite,
}

impl TrainingLevel {
    /// Base skill level for this training (0.0 to 1.0)
    pub fn base_skill(&self) -> f32 {
        match self {
            Self::Levy => 0.2,
            Self::Militia => 0.35,
            Self::Regular => 0.5,
            Self::Veteran => 0.7,
            Self::Elite => 0.85,
        }
    }
}

impl Default for EntityArchetype {
    fn default() -> Self {
        Self::Peasant
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archetype_variants_exist() {
        let _peasant = EntityArchetype::Peasant;
        let _laborer = EntityArchetype::Laborer;
        let _craftsman = EntityArchetype::Craftsman { specialty: CraftSpecialty::Smithing };
        let _soldier = EntityArchetype::Soldier { training: TrainingLevel::Levy };
        let _noble = EntityArchetype::Noble;
        let _merchant = EntityArchetype::Merchant;
        let _scholar = EntityArchetype::Scholar;
        let _child = EntityArchetype::Child;
    }

    #[test]
    fn test_training_levels() {
        assert!(TrainingLevel::Levy.base_skill() < TrainingLevel::Elite.base_skill());
    }
}
