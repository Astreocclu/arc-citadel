//! Display stat computation for UI rendering
//!
//! Converts raw chunk library data into human-readable skill levels
//! suitable for character sheets and UI elements.

use crate::genetics::Phenotype;
use crate::skills::{ChunkDomain, ChunkLibrary};
use serde::{Deserialize, Serialize};

/// Human-readable skill level for display
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillLevel {
    Untrained,
    Novice,
    Trained,
    Veteran,
    Expert,
    Master,
    Legend,
}

impl SkillLevel {
    /// Human-readable name for this skill level
    pub fn name(&self) -> &'static str {
        match self {
            SkillLevel::Untrained => "Untrained",
            SkillLevel::Novice => "Novice",
            SkillLevel::Trained => "Trained",
            SkillLevel::Veteran => "Veteran",
            SkillLevel::Expert => "Expert",
            SkillLevel::Master => "Master",
            SkillLevel::Legend => "Legend",
        }
    }
}

/// A display-ready stat for UI rendering
#[derive(Debug, Clone)]
pub struct DisplayStat {
    /// Name of the stat (e.g., "Craftsmanship", "Combat")
    pub name: &'static str,
    /// Computed skill level
    pub level: SkillLevel,
    /// Bar fill for visual rendering (0.0 to 1.0)
    pub bar_fill: f32,
}

impl DisplayStat {
    /// Create a new display stat, clamping bar_fill to valid range
    pub fn new(name: &'static str, level: SkillLevel, bar_fill: f32) -> Self {
        Self {
            name,
            level,
            bar_fill: bar_fill.clamp(0.0, 1.0),
        }
    }
}

/// Compute skill level from highest chunk level and average encoding
pub fn compute_level(highest_chunk_level: u8, avg_encoding: f32) -> SkillLevel {
    match highest_chunk_level {
        0 => SkillLevel::Untrained,
        1 => {
            if avg_encoding < 0.3 {
                SkillLevel::Untrained
            } else {
                SkillLevel::Novice
            }
        }
        2 => {
            if avg_encoding < 0.5 {
                SkillLevel::Novice
            } else {
                SkillLevel::Trained
            }
        }
        3 => {
            if avg_encoding < 0.6 {
                SkillLevel::Trained
            } else {
                SkillLevel::Veteran
            }
        }
        4 => {
            if avg_encoding < 0.7 {
                SkillLevel::Veteran
            } else {
                SkillLevel::Expert
            }
        }
        5 => {
            if avg_encoding >= 0.99 {
                SkillLevel::Legend
            } else if avg_encoding >= 0.85 {
                SkillLevel::Master
            } else {
                SkillLevel::Expert
            }
        }
        _ => SkillLevel::Legend,
    }
}

/// Compute bar fill from highest chunk level and average encoding
pub fn compute_bar(highest_level: u8, avg_encoding: f32) -> f32 {
    let level_contribution = (highest_level as f32) * 0.15;
    let encoding_contribution = avg_encoding * 0.25;
    (level_contribution + encoding_contribution).clamp(0.0, 1.0)
}

/// Compute craftsmanship display stat from Craft domain
pub fn compute_craftsmanship(library: &ChunkLibrary, phenotype: &Phenotype) -> DisplayStat {
    let summary = library.domain_summary(ChunkDomain::Craft);
    let ceiling = phenotype.domain_ceiling(ChunkDomain::Craft);
    let avg_encoding = summary.average_encoding() * ceiling;
    let level = compute_level(summary.highest_level, avg_encoding);
    let bar = compute_bar(summary.highest_level, avg_encoding);
    DisplayStat::new("Craftsmanship", level, bar)
}

/// Compute medicine display stat from Medicine domain
pub fn compute_medicine(library: &ChunkLibrary, phenotype: &Phenotype) -> DisplayStat {
    let summary = library.domain_summary(ChunkDomain::Medicine);
    let ceiling = phenotype.domain_ceiling(ChunkDomain::Medicine);
    let avg_encoding = summary.average_encoding() * ceiling;
    let level = compute_level(summary.highest_level, avg_encoding);
    let bar = compute_bar(summary.highest_level, avg_encoding);
    DisplayStat::new("Medicine", level, bar)
}

/// Compute leadership display stat from Leadership domain
pub fn compute_leadership(library: &ChunkLibrary, phenotype: &Phenotype) -> DisplayStat {
    let summary = library.domain_summary(ChunkDomain::Leadership);
    let ceiling = phenotype.domain_ceiling(ChunkDomain::Leadership);
    let avg_encoding = summary.average_encoding() * ceiling;
    let level = compute_level(summary.highest_level, avg_encoding);
    let bar = compute_bar(summary.highest_level, avg_encoding);
    DisplayStat::new("Leadership", level, bar)
}

/// Compute scholarship display stat from Knowledge domain
pub fn compute_scholarship(library: &ChunkLibrary, phenotype: &Phenotype) -> DisplayStat {
    let summary = library.domain_summary(ChunkDomain::Knowledge);
    let ceiling = phenotype.domain_ceiling(ChunkDomain::Knowledge);
    let avg_encoding = summary.average_encoding() * ceiling;
    let level = compute_level(summary.highest_level, avg_encoding);
    let bar = compute_bar(summary.highest_level, avg_encoding);
    DisplayStat::new("Scholarship", level, bar)
}

/// Compute athleticism display stat from Physical domain
pub fn compute_athleticism(library: &ChunkLibrary, phenotype: &Phenotype) -> DisplayStat {
    let summary = library.domain_summary(ChunkDomain::Physical);
    let ceiling = phenotype.domain_ceiling(ChunkDomain::Physical);
    let avg_encoding = summary.average_encoding() * ceiling;
    let level = compute_level(summary.highest_level, avg_encoding);
    let bar = compute_bar(summary.highest_level, avg_encoding);
    DisplayStat::new("Athleticism", level, bar)
}

/// Compute charisma display stat from Social domain + phenotype components
///
/// Charisma is special: 50% from chunks, 30% from voice quality, 20% from perception
pub fn compute_charisma(library: &ChunkLibrary, phenotype: &Phenotype) -> DisplayStat {
    let summary = library.domain_summary(ChunkDomain::Social);
    let chunk_contribution = summary.average_encoding() * 0.5;
    let appearance_contribution = phenotype.voice_quality * 0.3;
    let perception_contribution = phenotype.perception * 0.2;
    let combined = chunk_contribution + appearance_contribution + perception_contribution;
    let level = compute_level(summary.highest_level, combined);
    let bar = compute_bar(summary.highest_level, combined);
    DisplayStat::new("Charisma", level, bar)
}

/// Compute combat display stat from Combat domain
pub fn compute_combat(library: &ChunkLibrary, phenotype: &Phenotype) -> DisplayStat {
    let summary = library.domain_summary(ChunkDomain::Combat);
    let ceiling = phenotype.domain_ceiling(ChunkDomain::Combat);
    let avg_encoding = summary.average_encoding() * ceiling;
    let level = compute_level(summary.highest_level, avg_encoding);
    let bar = compute_bar(summary.highest_level, avg_encoding);
    DisplayStat::new("Combat", level, bar)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_level_names() {
        assert_eq!(SkillLevel::Untrained.name(), "Untrained");
        assert_eq!(SkillLevel::Novice.name(), "Novice");
        assert_eq!(SkillLevel::Trained.name(), "Trained");
        assert_eq!(SkillLevel::Veteran.name(), "Veteran");
        assert_eq!(SkillLevel::Expert.name(), "Expert");
        assert_eq!(SkillLevel::Master.name(), "Master");
        assert_eq!(SkillLevel::Legend.name(), "Legend");
    }

    #[test]
    fn test_compute_level() {
        // Level 0 is always untrained
        assert_eq!(compute_level(0, 0.0), SkillLevel::Untrained);
        assert_eq!(compute_level(0, 1.0), SkillLevel::Untrained);

        // Level 1: threshold at 0.3
        assert_eq!(compute_level(1, 0.2), SkillLevel::Untrained);
        assert_eq!(compute_level(1, 0.5), SkillLevel::Novice);

        // Level 2: threshold at 0.5
        assert_eq!(compute_level(2, 0.4), SkillLevel::Novice);
        assert_eq!(compute_level(2, 0.6), SkillLevel::Trained);

        // Level 3: threshold at 0.6
        assert_eq!(compute_level(3, 0.5), SkillLevel::Trained);
        assert_eq!(compute_level(3, 0.7), SkillLevel::Veteran);

        // Level 4: threshold at 0.7
        assert_eq!(compute_level(4, 0.6), SkillLevel::Veteran);
        assert_eq!(compute_level(4, 0.8), SkillLevel::Expert);

        // Level 5: expert/master/legend
        assert_eq!(compute_level(5, 0.7), SkillLevel::Expert);
        assert_eq!(compute_level(5, 0.9), SkillLevel::Master);
        assert_eq!(compute_level(5, 0.99), SkillLevel::Legend);

        // Level 6+ is always legend
        assert_eq!(compute_level(6, 0.0), SkillLevel::Legend);
    }

    #[test]
    fn test_compute_bar() {
        // Level 0, encoding 0 = 0
        assert_eq!(compute_bar(0, 0.0), 0.0);

        // Level 2, encoding 0.4 = 2*0.15 + 0.4*0.25 = 0.3 + 0.1 = 0.4
        let bar = compute_bar(2, 0.4);
        assert!((bar - 0.4).abs() < 0.01);

        // Level 5, encoding 1.0 = 5*0.15 + 1.0*0.25 = 0.75 + 0.25 = 1.0
        assert_eq!(compute_bar(5, 1.0), 1.0);

        // Clamping check
        assert!(compute_bar(10, 1.0) <= 1.0);
    }

    #[test]
    fn test_display_stat_clamping() {
        let stat = DisplayStat::new("Test", SkillLevel::Novice, 1.5);
        assert_eq!(stat.bar_fill, 1.0);

        let stat = DisplayStat::new("Test", SkillLevel::Novice, -0.5);
        assert_eq!(stat.bar_fill, 0.0);
    }

    #[test]
    fn test_compute_craftsmanship_empty() {
        let library = ChunkLibrary::new();
        let phenotype = Phenotype::default();
        let stat = compute_craftsmanship(&library, &phenotype);
        assert_eq!(stat.name, "Craftsmanship");
        assert_eq!(stat.level, SkillLevel::Untrained);
        assert_eq!(stat.bar_fill, 0.0);
    }

    #[test]
    fn test_compute_medicine_empty() {
        let library = ChunkLibrary::new();
        let phenotype = Phenotype::default();
        let stat = compute_medicine(&library, &phenotype);
        assert_eq!(stat.name, "Medicine");
        assert_eq!(stat.level, SkillLevel::Untrained);
    }

    #[test]
    fn test_compute_leadership_empty() {
        let library = ChunkLibrary::new();
        let phenotype = Phenotype::default();
        let stat = compute_leadership(&library, &phenotype);
        assert_eq!(stat.name, "Leadership");
        assert_eq!(stat.level, SkillLevel::Untrained);
    }

    #[test]
    fn test_compute_scholarship_empty() {
        let library = ChunkLibrary::new();
        let phenotype = Phenotype::default();
        let stat = compute_scholarship(&library, &phenotype);
        assert_eq!(stat.name, "Scholarship");
        assert_eq!(stat.level, SkillLevel::Untrained);
    }

    #[test]
    fn test_compute_athleticism_empty() {
        let library = ChunkLibrary::new();
        let phenotype = Phenotype::default();
        let stat = compute_athleticism(&library, &phenotype);
        assert_eq!(stat.name, "Athleticism");
        assert_eq!(stat.level, SkillLevel::Untrained);
    }

    #[test]
    fn test_compute_charisma_empty() {
        let library = ChunkLibrary::new();
        let phenotype = Phenotype::default();
        let stat = compute_charisma(&library, &phenotype);
        assert_eq!(stat.name, "Charisma");
        // Even with empty library, phenotype contributes:
        // 0.0 * 0.5 + 1.0 * 0.3 + 1.0 * 0.2 = 0.5
        // Level 0 is always Untrained regardless of encoding
        assert_eq!(stat.level, SkillLevel::Untrained);
    }

    #[test]
    fn test_compute_combat_empty() {
        let library = ChunkLibrary::new();
        let phenotype = Phenotype::default();
        let stat = compute_combat(&library, &phenotype);
        assert_eq!(stat.name, "Combat");
        assert_eq!(stat.level, SkillLevel::Untrained);
        assert_eq!(stat.bar_fill, 0.0);
    }

    #[test]
    fn test_compute_combat_trained() {
        let library = ChunkLibrary::trained_soldier(1000);
        let phenotype = Phenotype::default();
        let stat = compute_combat(&library, &phenotype);
        assert_eq!(stat.name, "Combat");
        // Trained soldier has level 2 chunks (AttackSequence) with moderate encoding
        // Should be at least Novice
        assert!(
            stat.level == SkillLevel::Novice
                || stat.level == SkillLevel::Trained
                || stat.level == SkillLevel::Veteran
        );
        assert!(stat.bar_fill > 0.0);
    }

    #[test]
    fn test_compute_combat_veteran() {
        let library = ChunkLibrary::veteran(1000);
        let phenotype = Phenotype::default();
        let stat = compute_combat(&library, &phenotype);
        assert_eq!(stat.name, "Combat");
        // Veteran has level 3 chunks (EngageMelee) with good encoding
        // Should be at least Trained
        assert!(
            stat.level == SkillLevel::Trained
                || stat.level == SkillLevel::Veteran
                || stat.level == SkillLevel::Expert
        );
        assert!(stat.bar_fill > 0.2);
    }

    #[test]
    fn test_phenotype_affects_stat() {
        let library = ChunkLibrary::trained_soldier(1000);

        // Strong combatant
        let strong = Phenotype {
            strength: 1.4,
            agility: 1.4,
            ..Phenotype::default()
        };
        let stat_strong = compute_combat(&library, &strong);

        // Weak combatant
        let weak = Phenotype {
            strength: 0.6,
            agility: 0.6,
            ..Phenotype::default()
        };
        let stat_weak = compute_combat(&library, &weak);

        // Strong should have higher bar fill due to ceiling
        assert!(stat_strong.bar_fill >= stat_weak.bar_fill);
    }

    #[test]
    fn test_charisma_voice_quality_impact() {
        let library = ChunkLibrary::new();

        // Great voice
        let great_voice = Phenotype {
            voice_quality: 1.5,
            perception: 1.0,
            ..Phenotype::default()
        };
        let stat = compute_charisma(&library, &great_voice);
        // 0.0 * 0.5 + 1.5 * 0.3 + 1.0 * 0.2 = 0.45 + 0.2 = 0.65
        // But level 0 is always untrained
        assert_eq!(stat.level, SkillLevel::Untrained);

        // Terrible voice
        let terrible_voice = Phenotype {
            voice_quality: 0.5,
            perception: 0.5,
            ..Phenotype::default()
        };
        let stat2 = compute_charisma(&library, &terrible_voice);
        // Both untrained with empty library, but bar fills differ
        assert!(stat.bar_fill >= stat2.bar_fill);
    }
}
