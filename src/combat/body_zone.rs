//! Body zones for wound tracking (11 zones)
//!
//! Replaces the old 6-zone BodyPart system with granular zones.

use serde::{Deserialize, Serialize};

/// Wound severity categories (not f32)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WoundSeverity {
    /// No wound
    None,
    /// Cosmetic only
    Scratch,
    /// Painful but functional
    Minor,
    /// Impaired function, bleeding
    Serious,
    /// Disabled, severe bleeding
    Critical,
    /// Limb gone / organ destroyed
    Destroyed,
}

/// Body zones for hit location (11 total)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BodyZone {
    /// Fatal threshold very low
    Head,
    /// Arterial - bleeds fast
    Neck,
    /// Center mass, most hits land here
    Torso,
    ArmLeft,
    ArmRight,
    HandLeft,
    HandRight,
    LegLeft,
    LegRight,
    FootLeft,
    FootRight,
}

impl BodyZone {
    /// Returns all body zones
    pub fn all() -> [BodyZone; 11] {
        [
            BodyZone::Head,
            BodyZone::Neck,
            BodyZone::Torso,
            BodyZone::ArmLeft,
            BodyZone::ArmRight,
            BodyZone::HandLeft,
            BodyZone::HandRight,
            BodyZone::LegLeft,
            BodyZone::LegRight,
            BodyZone::FootLeft,
            BodyZone::FootRight,
        ]
    }

    /// What severity of wound to this zone is fatal?
    pub fn fatality_threshold(&self) -> WoundSeverity {
        match self {
            BodyZone::Head | BodyZone::Neck => WoundSeverity::Serious,
            BodyZone::Torso => WoundSeverity::Critical,
            // Limbs don't kill directly
            _ => WoundSeverity::Destroyed,
        }
    }

    /// Relative probability of being hit when standing (sums to 1.0)
    pub fn hit_weight_standing(&self) -> f32 {
        match self {
            BodyZone::Torso => 0.35,
            BodyZone::ArmLeft | BodyZone::ArmRight => 0.10,
            BodyZone::LegLeft | BodyZone::LegRight => 0.12,
            BodyZone::Head => 0.08,
            BodyZone::Neck => 0.03,
            BodyZone::HandLeft | BodyZone::HandRight => 0.03,
            BodyZone::FootLeft | BodyZone::FootRight => 0.02,
        }
    }

    /// Is this a leg zone?
    pub fn is_leg(&self) -> bool {
        matches!(
            self,
            BodyZone::LegLeft | BodyZone::LegRight | BodyZone::FootLeft | BodyZone::FootRight
        )
    }

    /// Is this an arm zone?
    pub fn is_arm(&self) -> bool {
        matches!(self, BodyZone::ArmLeft | BodyZone::ArmRight)
    }

    /// Is this a hand zone?
    pub fn is_hand(&self) -> bool {
        matches!(self, BodyZone::HandLeft | BodyZone::HandRight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_zone_count() {
        assert_eq!(BodyZone::all().len(), 11);
    }

    #[test]
    fn test_hit_weights_sum_to_one() {
        let total: f32 = BodyZone::all()
            .iter()
            .map(|z| z.hit_weight_standing())
            .sum();
        assert!((total - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_head_is_fatal() {
        assert_eq!(BodyZone::Head.fatality_threshold(), WoundSeverity::Serious);
    }

    #[test]
    fn test_limbs_not_directly_fatal() {
        assert_eq!(
            BodyZone::ArmLeft.fatality_threshold(),
            WoundSeverity::Destroyed
        );
        assert_eq!(
            BodyZone::LegRight.fatality_threshold(),
            WoundSeverity::Destroyed
        );
    }

    #[test]
    fn test_zone_categories() {
        assert!(BodyZone::LegLeft.is_leg());
        assert!(BodyZone::FootRight.is_leg());
        assert!(!BodyZone::ArmLeft.is_leg());

        assert!(BodyZone::ArmRight.is_arm());
        assert!(!BodyZone::HandRight.is_arm());

        assert!(BodyZone::HandLeft.is_hand());
    }
}
