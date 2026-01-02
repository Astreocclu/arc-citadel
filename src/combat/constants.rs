//! Combat system constants - all tunable values in one place
//!
//! These values are ADDITIVE, never multiplicative. No percentage modifiers.

// Time constants
pub const TICK_DURATION_MS: u32 = 100;
pub const RECOVERY_TICKS: u32 = 10;
pub const EXHAUSTION_THRESHOLD: f32 = 0.9;

// Fatigue constants (ADDITIVE, not multiplicative)
pub const FATIGUE_PER_ATTACK: f32 = 0.05;
pub const FATIGUE_PER_DEFEND: f32 = 0.03;
pub const FATIGUE_PER_MELEE_TICK: f32 = 0.01;
pub const FATIGUE_RECOVERY_RATE: f32 = 0.02;

// Stress constants (ADDITIVE thresholds)
pub const BASE_STRESS_THRESHOLD: f32 = 1.0;
pub const STRESS_DECAY_RATE: f32 = 0.001;
pub const SHAKEN_THRESHOLD_RATIO: f32 = 0.8;

// Formation constants
pub const FORMATION_BREAK_THRESHOLD: f32 = 0.4;
pub const COHESION_LOSS_PER_CASUALTY: f32 = 0.02;
pub const PRESSURE_DECAY_RATE: f32 = 0.01;

// Officer constants
pub const INTERVENTION_COST: f32 = 0.2;
pub const INTERVENTION_RANGE: f32 = 10.0;
pub const ATTENTION_RECOVERY_RATE: f32 = 0.1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fatigue_constants_reasonable() {
        assert!(FATIGUE_PER_ATTACK > 0.0 && FATIGUE_PER_ATTACK < 0.2);
        assert!(FATIGUE_PER_DEFEND > 0.0 && FATIGUE_PER_DEFEND < FATIGUE_PER_ATTACK);
        assert!(FATIGUE_RECOVERY_RATE > 0.0);
    }

    #[test]
    fn test_stress_constants_reasonable() {
        assert!(BASE_STRESS_THRESHOLD > 0.0);
        assert!(SHAKEN_THRESHOLD_RATIO > 0.0 && SHAKEN_THRESHOLD_RATIO < 1.0);
    }
}
