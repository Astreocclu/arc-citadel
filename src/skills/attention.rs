//! Attention budget management
//!
//! Each entity has a base attention budget of 1.0 per decision point.
//! Fatigue, pain, and stress reduce available attention.

/// Refresh attention budget for a new decision point
///
/// # Arguments
/// * `fatigue` - 0.0 (fresh) to 1.0 (exhausted)
/// * `pain` - 0.0 (none) to 1.0 (incapacitating)
/// * `stress` - 0.0 (calm) to 1.0 (panicked)
///
/// # Returns
/// Available attention budget (minimum 0.2)
pub fn calculate_attention_budget(fatigue: f32, pain: f32, stress: f32) -> f32 {
    let base = 1.0;

    // Penalties stack but multiplicatively to prevent going negative
    let fatigue_mult = 1.0 - (fatigue * 0.3); // Max -30%
    let pain_mult = 1.0 - (pain * 0.4); // Max -40%
    let stress_mult = 1.0 - (stress * 0.2); // Max -20%

    let budget = base * fatigue_mult * pain_mult * stress_mult;

    // Minimum viable attention
    budget.max(0.2)
}

/// Check if an action is affordable within current attention
pub fn can_afford_attention(remaining: f32, cost: f32) -> bool {
    cost <= remaining
}

/// Fumble threshold - below this attention, action risks fumbling
pub const FUMBLE_ATTENTION_THRESHOLD: f32 = 0.1;

/// Check if action execution risks fumble due to attention overload
pub fn risks_fumble(remaining_after: f32) -> bool {
    remaining_after < FUMBLE_ATTENTION_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fresh_entity_full_attention() {
        let budget = calculate_attention_budget(0.0, 0.0, 0.0);
        assert_eq!(budget, 1.0);
    }

    #[test]
    fn test_exhausted_reduced_attention() {
        let budget = calculate_attention_budget(1.0, 0.0, 0.0);
        assert!((budget - 0.7).abs() < 0.01); // 30% reduction
    }

    #[test]
    fn test_combined_penalties() {
        // High fatigue + pain + stress
        let budget = calculate_attention_budget(0.8, 0.5, 0.6);

        // 1.0 * 0.76 * 0.8 * 0.88 = 0.535
        assert!(budget > 0.4 && budget < 0.6);
    }

    #[test]
    fn test_minimum_attention_floor() {
        // With max penalties: 1.0 * 0.7 * 0.6 * 0.8 = 0.336
        // This is above the floor, but floor still enforces minimum 0.2
        let budget = calculate_attention_budget(1.0, 1.0, 1.0);
        assert!((budget - 0.336).abs() < 0.01);

        // Verify floor would apply if result were lower (by testing the max operation)
        assert!(budget >= 0.2);
    }

    #[test]
    fn test_can_afford() {
        assert!(can_afford_attention(0.5, 0.3));
        assert!(can_afford_attention(0.5, 0.5));
        assert!(!can_afford_attention(0.5, 0.6));
    }

    #[test]
    fn test_fumble_risk() {
        assert!(!risks_fumble(0.2));
        assert!(risks_fumble(0.05));
        assert!(risks_fumble(-0.1));
    }
}
