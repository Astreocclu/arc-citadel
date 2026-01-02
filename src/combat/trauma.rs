//! Trauma resolution: Mass vs Padding lookup table
//!
//! NO PERCENTAGE MODIFIERS. Categorical comparison only.

use serde::{Deserialize, Serialize};
use crate::combat::weapons::Mass;
use crate::combat::armor::Padding;

/// Result of mass vs padding comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TraumaResult {
    /// No mechanical effect
    Negligible,
    /// Stamina cost, accumulates
    Fatigue,
    /// Brief vulnerability window
    Stagger,
    /// On ground + internal bruising
    KnockdownBruise,
    /// On ground + broken bones
    KnockdownCrush,
}

/// Resolve trauma using categorical lookup table
///
/// # Arguments
/// * `mass` - Weapon's mass category
/// * `padding` - Armor's padding category
///
/// # Returns
/// Categorical trauma result (no percentages)
pub fn resolve_trauma(mass: Mass, padding: Padding) -> TraumaResult {
    use TraumaResult::*;

    // Lookup table (NO MULTIPLICATION)
    match (mass, padding) {
        // Light weapons - minimal trauma regardless of padding
        (Mass::Light, Padding::None) => Negligible,
        (Mass::Light, Padding::Light) => Negligible,
        (Mass::Light, Padding::Heavy) => Negligible,

        // Medium weapons - depends on padding
        (Mass::Medium, Padding::None) => Stagger,
        (Mass::Medium, Padding::Light) => Fatigue,
        (Mass::Medium, Padding::Heavy) => Negligible,

        // Heavy weapons - padding helps but doesn't eliminate
        (Mass::Heavy, Padding::None) => KnockdownBruise,
        (Mass::Heavy, Padding::Light) => Stagger,
        (Mass::Heavy, Padding::Heavy) => Fatigue,

        // Massive (cavalry charge, siege) - padding barely matters
        (Mass::Massive, Padding::None) => KnockdownCrush,
        (Mass::Massive, Padding::Light) => KnockdownBruise,
        (Mass::Massive, Padding::Heavy) => Stagger,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heavy_vs_no_padding_knocks_down() {
        let result = resolve_trauma(Mass::Heavy, Padding::None);
        assert_eq!(result, TraumaResult::KnockdownBruise);
    }

    #[test]
    fn test_light_always_negligible() {
        assert_eq!(resolve_trauma(Mass::Light, Padding::None), TraumaResult::Negligible);
        assert_eq!(resolve_trauma(Mass::Light, Padding::Light), TraumaResult::Negligible);
        assert_eq!(resolve_trauma(Mass::Light, Padding::Heavy), TraumaResult::Negligible);
    }

    #[test]
    fn test_massive_overwhelms() {
        assert_eq!(resolve_trauma(Mass::Massive, Padding::None), TraumaResult::KnockdownCrush);
        assert_eq!(resolve_trauma(Mass::Massive, Padding::Heavy), TraumaResult::Stagger);
    }

    #[test]
    fn test_mace_vs_plate_knight() {
        // Spec example: Heavy mace vs Heavy padding = Fatigue
        // Knight is fatigued but not wounded - historically accurate
        let result = resolve_trauma(Mass::Heavy, Padding::Heavy);
        assert_eq!(result, TraumaResult::Fatigue);
    }

    #[test]
    fn test_all_mass_padding_combinations() {
        for mass in [Mass::Light, Mass::Medium, Mass::Heavy, Mass::Massive] {
            for padding in [Padding::None, Padding::Light, Padding::Heavy] {
                let _ = resolve_trauma(mass, padding);
            }
        }
    }
}
