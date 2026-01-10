//! Stress and morale system
//!
//! Stress accumulates. When stress exceeds threshold, entity breaks.
//! What you do to enemies = what they can do to you (symmetric).

use crate::combat::constants::{BASE_STRESS_THRESHOLD, SHAKEN_THRESHOLD_RATIO};
use serde::{Deserialize, Serialize};

/// Sources of combat stress (symmetric - applies to both sides)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StressSource {
    // Combat stress
    TakingCasualties,
    TakingFire,
    MeleeViolence,
    WoundReceived,
    NearMiss,

    // Shock stress (spikes)
    OfficerKilled,
    FlankAttack,
    AmbushSprung,
    CavalryCharge,
    TerrifyingEnemy,

    // Pressure stress (sustained)
    Outnumbered,
    Surrounded,
    NoResponse,
    OverwatchFire,
    ProlongedCombat,

    // Social stress
    AlliesBreaking,
    AloneExposed,
}

impl StressSource {
    /// All stress sources
    pub fn all() -> &'static [StressSource] {
        &[
            StressSource::TakingCasualties,
            StressSource::TakingFire,
            StressSource::MeleeViolence,
            StressSource::WoundReceived,
            StressSource::NearMiss,
            StressSource::OfficerKilled,
            StressSource::FlankAttack,
            StressSource::AmbushSprung,
            StressSource::CavalryCharge,
            StressSource::TerrifyingEnemy,
            StressSource::Outnumbered,
            StressSource::Surrounded,
            StressSource::NoResponse,
            StressSource::OverwatchFire,
            StressSource::ProlongedCombat,
            StressSource::AlliesBreaking,
            StressSource::AloneExposed,
        ]
    }

    /// Base stress value (ADDITIVE, not percentage)
    pub fn base_stress(&self) -> f32 {
        match self {
            // Combat stress
            StressSource::TakingCasualties => 0.05,
            StressSource::TakingFire => 0.02,
            StressSource::MeleeViolence => 0.01,
            StressSource::WoundReceived => 0.15,
            StressSource::NearMiss => 0.03,

            // Shock stress (major spikes)
            StressSource::OfficerKilled => 0.30,
            StressSource::FlankAttack => 0.20,
            StressSource::AmbushSprung => 0.25,
            StressSource::CavalryCharge => 0.20,
            StressSource::TerrifyingEnemy => 0.15,

            // Pressure stress (per tick while true)
            StressSource::Outnumbered => 0.01,
            StressSource::Surrounded => 0.03,
            StressSource::NoResponse => 0.02,
            StressSource::OverwatchFire => 0.02,
            StressSource::ProlongedCombat => 0.005,

            // Social stress
            StressSource::AlliesBreaking => 0.10,
            StressSource::AloneExposed => 0.05,
        }
    }
}

/// Morale break result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakResult {
    /// Holding steady
    Holding,
    /// Shaken but not broken
    Shaken,
    /// Breaking - will flee
    Breaking,
}

/// Morale state for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoraleState {
    /// Accumulated stress (0.0 to unlimited)
    pub current_stress: f32,
    /// Personal breaking point
    pub base_threshold: f32,
}

impl Default for MoraleState {
    fn default() -> Self {
        Self {
            current_stress: 0.0,
            base_threshold: BASE_STRESS_THRESHOLD,
        }
    }
}

impl MoraleState {
    /// Apply stress from a source
    pub fn apply_stress(&mut self, source: StressSource) {
        self.current_stress += source.base_stress();
    }

    /// Check if entity is breaking
    pub fn check_break(&self) -> BreakResult {
        let effective_threshold = self.base_threshold;

        if self.current_stress > effective_threshold {
            BreakResult::Breaking
        } else if self.current_stress > effective_threshold * SHAKEN_THRESHOLD_RATIO {
            BreakResult::Shaken
        } else {
            BreakResult::Holding
        }
    }

    /// Decay stress over time (when safe)
    pub fn decay_stress(&mut self, rate: f32) {
        self.current_stress = (self.current_stress - rate).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stress_sources_have_positive_values() {
        for source in StressSource::all() {
            assert!(source.base_stress() > 0.0);
        }
    }

    #[test]
    fn test_shock_stress_higher_than_sustained() {
        assert!(
            StressSource::OfficerKilled.base_stress() > StressSource::ProlongedCombat.base_stress()
        );
        assert!(StressSource::AmbushSprung.base_stress() > StressSource::Outnumbered.base_stress());
    }

    #[test]
    fn test_stress_accumulates() {
        let mut state = MoraleState::default();
        let initial = state.current_stress;

        state.apply_stress(StressSource::TakingCasualties);

        assert!(state.current_stress > initial);
    }

    #[test]
    fn test_break_thresholds() {
        let mut state = MoraleState::default();

        // Initially holding
        assert_eq!(state.check_break(), BreakResult::Holding);

        // Add stress to reach shaken
        state.current_stress = state.base_threshold * 0.85;
        assert_eq!(state.check_break(), BreakResult::Shaken);

        // Add more to break
        state.current_stress = state.base_threshold * 1.1;
        assert_eq!(state.check_break(), BreakResult::Breaking);
    }
}
