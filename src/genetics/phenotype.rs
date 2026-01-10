//! Phenotype - physical traits affecting skill ceilings
//!
//! Phenotype provides the physical/cognitive baseline that chunks operate within.
//! A weak entity can become a skilled smith, but their output speed is capped by strength.

use serde::{Deserialize, Serialize};

/// Physical and cognitive traits that affect skill performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phenotype {
    /// Physical strength (0.5-1.5, default 1.0)
    /// Affects: Craft speed ceiling, melee damage, labor capacity
    pub strength: f32,

    /// Physical endurance (0.5-1.5, default 1.0)
    /// Affects: Duration before fatigue, sustained work capacity
    pub endurance: f32,

    /// Fine motor control and speed (0.5-1.5, default 1.0)
    /// Affects: Craft precision ceiling, surgery, ranged combat
    pub agility: f32,

    /// Sensory acuity (0.5-1.5, default 1.0)
    /// Affects: Diagnostic accuracy, social reading, stealth detection
    pub perception: f32,

    /// Voice quality and projection (0.5-1.5, default 1.0)
    /// Affects: All voice-based social chunks, command range
    pub voice_quality: f32,

    /// Cognitive learning speed (0.5-1.5, default 1.0)
    /// Affects: Chunk formation rate multiplier
    pub learning_rate: f32,
}

impl Default for Phenotype {
    fn default() -> Self {
        Self {
            strength: 1.0,
            endurance: 1.0,
            agility: 1.0,
            perception: 1.0,
            voice_quality: 1.0,
            learning_rate: 1.0,
        }
    }
}

impl Phenotype {
    /// Create a phenotype with random variance around defaults
    pub fn with_variance(variance: f32) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let mut vary = |base: f32| -> f32 {
            let delta = rng.gen_range(-variance..=variance);
            (base + delta).clamp(0.5, 1.5)
        };

        Self {
            strength: vary(1.0),
            endurance: vary(1.0),
            agility: vary(1.0),
            perception: vary(1.0),
            voice_quality: vary(1.0),
            learning_rate: vary(1.0),
        }
    }

    /// Get the ceiling for a specific domain based on relevant traits
    pub fn domain_ceiling(&self, domain: crate::skills::ChunkDomain) -> f32 {
        use crate::skills::ChunkDomain;

        match domain {
            ChunkDomain::Combat => (self.strength + self.agility) / 2.0,
            ChunkDomain::Craft => (self.strength.min(self.agility) + self.perception) / 2.0,
            ChunkDomain::Social => (self.voice_quality + self.perception) / 2.0,
            ChunkDomain::Medicine => (self.agility + self.perception) / 2.0,
            ChunkDomain::Leadership => (self.voice_quality + self.perception) / 2.0,
            ChunkDomain::Knowledge => self.perception,
            ChunkDomain::Physical => (self.strength + self.endurance + self.agility) / 3.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::ChunkDomain;

    #[test]
    fn test_default_phenotype() {
        let p = Phenotype::default();
        assert_eq!(p.strength, 1.0);
        assert_eq!(p.learning_rate, 1.0);
    }

    #[test]
    fn test_phenotype_variance() {
        let p = Phenotype::with_variance(0.2);
        assert!(p.strength >= 0.5 && p.strength <= 1.5);
        assert!(p.agility >= 0.5 && p.agility <= 1.5);
    }

    #[test]
    fn test_domain_ceiling() {
        let p = Phenotype {
            strength: 1.2,
            endurance: 1.0,
            agility: 0.8,
            perception: 1.1,
            voice_quality: 1.0,
            learning_rate: 1.0,
        };

        // Combat ceiling = (strength + agility) / 2 = (1.2 + 0.8) / 2 = 1.0
        assert!((p.domain_ceiling(ChunkDomain::Combat) - 1.0).abs() < 0.01);

        // Craft ceiling = (min(strength, agility) + perception) / 2 = (0.8 + 1.1) / 2 = 0.95
        assert!((p.domain_ceiling(ChunkDomain::Craft) - 0.95).abs() < 0.01);
    }
}
