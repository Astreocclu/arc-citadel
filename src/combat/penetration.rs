//! Penetration resolution: Edge vs Rigidity lookup table
//!
//! NO PERCENTAGE MODIFIERS. Categorical comparison only.

use crate::combat::armor::Rigidity;
use crate::combat::weapons::Edge;
use serde::{Deserialize, Serialize};

/// Result of edge vs rigidity comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PenetrationResult {
    /// Severe wound, arterial risk
    DeepCut,
    /// Standard wound
    Cut,
    /// Minor wound
    ShallowCut,
    /// Weapon stuck, no wound, attacker vulnerable
    Snag,
    /// No effect from edge
    Deflect,
    /// Blunt weapon, skip to trauma resolution
    NoPenetrationAttempt,
}

/// Resolve penetration using categorical lookup table
///
/// # Arguments
/// * `edge` - Weapon's edge type
/// * `rigidity` - Armor's rigidity type
/// * `has_piercing` - Whether weapon has Piercing special property
/// * `has_anti_cavalry` - Whether weapon has AntiCavalry special property
/// * `target_is_mounted` - Whether target is mounted (cavalry)
///
/// # Returns
/// Categorical penetration result (no percentages)
pub fn resolve_penetration(
    edge: Edge,
    rigidity: Rigidity,
    has_piercing: bool,
    has_anti_cavalry: bool,
    target_is_mounted: bool,
) -> PenetrationResult {
    use Edge::*;
    use PenetrationResult::*;
    use Rigidity::*;

    // Base lookup table (NO MULTIPLICATION)
    let base_result = match (edge, rigidity) {
        // Razor edge - surgical sharpness
        (Razor, Cloth) => DeepCut,
        (Razor, Leather) => Cut,
        (Razor, Mail) => Snag,
        (Razor, Plate) => Deflect,

        // Sharp edge - combat weapons
        (Sharp, Cloth) => Cut,
        (Sharp, Leather) => ShallowCut,
        (Sharp, Mail) => Deflect,
        (Sharp, Plate) => Deflect,

        // Blunt - no penetration attempt, skip to trauma
        (Blunt, _) => NoPenetrationAttempt,
    };

    // Piercing special: one category better vs Mail/Plate
    // This is a CATEGORY SHIFT, not a percentage modifier
    let after_piercing = if has_piercing {
        match (edge, rigidity, base_result) {
            // Piercing vs Mail: one step better
            (Razor, Mail, Snag) => ShallowCut,
            (Sharp, Mail, Deflect) => ShallowCut,
            // Piercing vs Plate: one step better
            (Razor, Plate, Deflect) => Snag,
            (Sharp, Plate, Deflect) => Snag,
            _ => base_result,
        }
    } else {
        base_result
    };

    // AntiCavalry special: TWO categories better vs mounted targets
    // This represents striking the horse or finding gaps in barding
    // Spears are devastating to cavalry - can one-shot a horse
    if has_anti_cavalry && target_is_mounted {
        match after_piercing {
            Deflect => Cut,           // Deflect → Cut (skip 2 levels)
            Snag => DeepCut,          // Snag → DeepCut (skip 2 levels)
            ShallowCut => DeepCut,    // ShallowCut → DeepCut (skip 1 level, capped)
            other => other,           // Already good enough
        }
    } else {
        after_piercing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sharp_vs_plate_deflects() {
        let result = resolve_penetration(Edge::Sharp, Rigidity::Plate, false, false, false);
        assert_eq!(result, PenetrationResult::Deflect);
    }

    #[test]
    fn test_razor_vs_cloth_deep_cut() {
        let result = resolve_penetration(Edge::Razor, Rigidity::Cloth, false, false, false);
        assert_eq!(result, PenetrationResult::DeepCut);
    }

    #[test]
    fn test_blunt_no_penetration() {
        let result = resolve_penetration(Edge::Blunt, Rigidity::Plate, false, false, false);
        assert_eq!(result, PenetrationResult::NoPenetrationAttempt);
    }

    #[test]
    fn test_piercing_improves_vs_mail() {
        let without = resolve_penetration(Edge::Sharp, Rigidity::Mail, false, false, false);
        assert_eq!(without, PenetrationResult::Deflect);

        let with = resolve_penetration(Edge::Sharp, Rigidity::Mail, true, false, false);
        assert_eq!(with, PenetrationResult::ShallowCut);
    }

    #[test]
    fn test_anti_cavalry_vs_mounted() {
        // Sharp+Piercing vs Plate normally = Snag
        let without_anti_cav = resolve_penetration(Edge::Sharp, Rigidity::Plate, true, false, false);
        assert_eq!(without_anti_cav, PenetrationResult::Snag);

        // With AntiCavalry vs mounted = DeepCut (Snag + 2 categories)
        let with_anti_cav = resolve_penetration(Edge::Sharp, Rigidity::Plate, true, true, true);
        assert_eq!(with_anti_cav, PenetrationResult::DeepCut);
    }

    #[test]
    fn test_anti_cavalry_only_affects_mounted() {
        // AntiCavalry has no effect vs non-mounted
        let result = resolve_penetration(Edge::Sharp, Rigidity::Plate, true, true, false);
        assert_eq!(result, PenetrationResult::Snag);
    }

    #[test]
    fn test_all_edge_rigidity_combinations() {
        // Verify all combinations produce valid results (no panics)
        for edge in [Edge::Razor, Edge::Sharp, Edge::Blunt] {
            for rigidity in [
                Rigidity::Cloth,
                Rigidity::Leather,
                Rigidity::Mail,
                Rigidity::Plate,
            ] {
                for piercing in [false, true] {
                    for anti_cav in [false, true] {
                        for mounted in [false, true] {
                            let _ = resolve_penetration(edge, rigidity, piercing, anti_cav, mounted);
                        }
                    }
                }
            }
        }
    }
}
