//! Experience-based chunk generation
//!
//! Entities accumulate life experiences that generate skill chunks.
//! A 45-year-old farmer has deeper farming chunks than a 20-year-old.
//! A soldier who saw combat has different chunks than one fresh from training.

use serde::{Deserialize, Serialize};

/// A period of an entity's life that generated skill chunks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LifeExperience {
    /// What kind of activity
    pub activity: ActivityType,
    /// How long (in years)
    pub duration_years: f32,
    /// Intensity: full-time = 1.0, part-time = 0.5, casual = 0.2
    pub intensity: f32,
    /// Quality of training/environment (0.0 to 1.0)
    pub training_quality: f32,
}

/// Military unit types for training specialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitType {
    Infantry,
    Cavalry,
    Archer,
}

/// Activities that generate skill chunks over time
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActivityType {
    // === Physical Labor ===
    Farming,
    Mining,
    Construction,
    Hauling,

    // === Crafts ===
    Smithing,
    Carpentry,
    Masonry,
    Cooking,
    Tailoring,
    Leatherworking,
    Pottery,

    // === Combat ===
    MilitaryTraining { unit_type: UnitType },
    CombatExperience { battles_fought: u32 },
    GuardDuty,

    // === Social ===
    Trading,
    Diplomacy,
    CourtLife,
    PublicSpeaking,

    // === Leadership ===
    MilitaryCommand { soldiers_led: u32 },
    WorkforceManagement { workers_led: u32 },
    PoliticalOffice,

    // === Knowledge ===
    Literacy,
    FormalEducation,
    Apprenticeship { master_skill: f32 },
    Research,
    Teaching,

    // === Universal ===
    GeneralLife,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_life_experience_creation() {
        let exp = LifeExperience {
            activity: ActivityType::Farming,
            duration_years: 10.0,
            intensity: 0.9,
            training_quality: 0.5,
        };
        assert_eq!(exp.duration_years, 10.0);
        assert_eq!(exp.intensity, 0.9);
    }

    #[test]
    fn test_military_training_variants() {
        let infantry = ActivityType::MilitaryTraining { unit_type: UnitType::Infantry };
        let cavalry = ActivityType::MilitaryTraining { unit_type: UnitType::Cavalry };
        assert_ne!(infantry, cavalry);
    }
}
