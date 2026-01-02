//! Wound system: combines penetration and trauma results
//!
//! Wounds track zone, severity, and effects (bleeding, mobility, grip).

use serde::{Deserialize, Serialize};
use crate::combat::body_zone::{BodyZone, WoundSeverity};
use crate::combat::penetration::PenetrationResult;
use crate::combat::trauma::TraumaResult;

/// A wound on a specific body zone
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wound {
    pub zone: BodyZone,
    pub severity: WoundSeverity,
    pub bleeding: bool,
    pub mobility_impact: bool,
    pub grip_impact: bool,
}

impl Wound {
    /// Create a wound with no effects
    pub fn none(zone: BodyZone) -> Self {
        Self {
            zone,
            severity: WoundSeverity::None,
            bleeding: false,
            mobility_impact: false,
            grip_impact: false,
        }
    }
}

/// Combine penetration and trauma results into a wound
///
/// Takes the WORSE of the two results (no multiplication).
pub fn combine_results(
    pen: PenetrationResult,
    trauma: TraumaResult,
    zone: BodyZone,
) -> Wound {
    use PenetrationResult::*;
    use TraumaResult::*;
    use WoundSeverity::*;

    // Penetration severity
    let pen_severity = match pen {
        DeepCut => Critical,
        Cut => Serious,
        ShallowCut => Minor,
        Snag | Deflect | NoPenetrationAttempt => None,
    };

    // Trauma severity
    let trauma_severity = match trauma {
        KnockdownCrush => Critical,
        KnockdownBruise => Serious,
        Stagger => Scratch,
        Fatigue | Negligible => None,
    };

    // Take worse of the two (max by Ord)
    let severity = pen_severity.max(trauma_severity);

    // Bleeding only from cuts
    let bleeding = matches!(pen, DeepCut | Cut);

    // Mobility impact from legs OR knockdown trauma
    let mobility_impact = zone.is_leg()
        || matches!(trauma, KnockdownCrush | KnockdownBruise);

    // Grip impact from arms or hands
    let grip_impact = zone.is_arm() || zone.is_hand();

    Wound {
        zone,
        severity,
        bleeding,
        mobility_impact,
        grip_impact,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deep_cut_is_critical() {
        let wound = combine_results(
            PenetrationResult::DeepCut,
            TraumaResult::Negligible,
            BodyZone::Torso,
        );
        assert_eq!(wound.severity, WoundSeverity::Critical);
        assert!(wound.bleeding);
    }

    #[test]
    fn test_deflect_with_knockdown_still_wounds() {
        let wound = combine_results(
            PenetrationResult::Deflect,
            TraumaResult::KnockdownBruise,
            BodyZone::Torso,
        );
        assert_eq!(wound.severity, WoundSeverity::Serious);
        assert!(!wound.bleeding);
    }

    #[test]
    fn test_leg_wound_affects_mobility() {
        let wound = combine_results(
            PenetrationResult::Cut,
            TraumaResult::Stagger,
            BodyZone::LegLeft,
        );
        assert!(wound.mobility_impact);
    }

    #[test]
    fn test_arm_wound_affects_grip() {
        let wound = combine_results(
            PenetrationResult::ShallowCut,
            TraumaResult::Negligible,
            BodyZone::ArmRight,
        );
        assert!(wound.grip_impact);
    }

    #[test]
    fn test_no_wound_from_deflect_and_negligible() {
        let wound = combine_results(
            PenetrationResult::Deflect,
            TraumaResult::Negligible,
            BodyZone::Torso,
        );
        assert_eq!(wound.severity, WoundSeverity::None);
    }
}
