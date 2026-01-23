//! Tactical geometry calculations for combat engagement
//!
//! This module calculates engagement geometry between attackers and defenders,
//! including attack angles, line of sight quality, cover, and enfilade effects.

use serde::{Deserialize, Serialize};

use crate::battle::battle_map::BattleMap;
use crate::battle::hex::{BattleHexCoord, HexDirection};
use crate::battle::units::{BattleUnit, FormationShape};

/// Attack angle relative to defender's facing
///
/// Determines hit bonuses and whether flanking negates cover.
/// Angles are based on hex direction differences:
/// - Frontal: 0-1 steps (0-60 degrees from facing)
/// - Flank: 2 steps (120 degrees from facing)
/// - Rear: 3 steps (180 degrees, opposite to facing)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackAngle {
    /// Attack from within +/- 60 degrees of defender's facing
    Frontal,
    /// Attack from 120 degrees to the left of defender's facing
    FlankLeft,
    /// Attack from 120 degrees to the right of defender's facing
    FlankRight,
    /// Attack from behind (120-180 degrees from facing)
    Rear,
}

impl AttackAngle {
    /// Hit modifier bonus for this attack angle
    ///
    /// Returns an additive bonus to hit chance:
    /// - Frontal: 0.0 (no bonus)
    /// - Flank: 0.15 (15% bonus)
    /// - Rear: 0.30 (30% bonus)
    pub fn hit_modifier(&self) -> f32 {
        match self {
            AttackAngle::Frontal => 0.0,
            AttackAngle::FlankLeft | AttackAngle::FlankRight => 0.15,
            AttackAngle::Rear => 0.30,
        }
    }

    /// Whether this attack angle negates the defender's cover
    ///
    /// Flanking and rear attacks bypass cover from terrain features
    /// that only protect from frontal attacks.
    pub fn negates_cover(&self) -> bool {
        match self {
            AttackAngle::Frontal => false,
            AttackAngle::FlankLeft | AttackAngle::FlankRight | AttackAngle::Rear => true,
        }
    }

    /// Stress multiplier for receiving attacks from this angle
    ///
    /// Units suffer more morale stress when attacked from unexpected angles:
    /// - Frontal: 1.0 (normal stress)
    /// - Flank: 1.5 (50% more stress)
    /// - Rear: 2.0 (double stress)
    pub fn stress_multiplier(&self) -> f32 {
        match self {
            AttackAngle::Frontal => 1.0,
            AttackAngle::FlankLeft | AttackAngle::FlankRight => 1.5,
            AttackAngle::Rear => 2.0,
        }
    }
}

/// Line of sight quality between attacker and defender
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LosQuality {
    /// Unobstructed line of sight
    Clear,
    /// Partial concealment (e.g., treeline, smoke)
    Partial,
    /// Cannot see or shoot target
    Blocked,
}

impl LosQuality {
    /// Hit penalty for this line of sight quality
    ///
    /// Returns a multiplicative factor for hit chance:
    /// - Clear: 1.0 (no penalty)
    /// - Partial: 0.7 (30% penalty)
    /// - Blocked: 0.0 (cannot hit)
    pub fn hit_multiplier(&self) -> f32 {
        match self {
            LosQuality::Clear => 1.0,
            LosQuality::Partial => 0.7,
            LosQuality::Blocked => 0.0,
        }
    }
}

/// Complete engagement geometry between attacker and defender
///
/// Captures all tactical factors that affect combat resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementGeometry {
    /// Range in hex units
    pub range: u32,
    /// Angle of attack relative to defender's facing
    pub attack_angle: AttackAngle,
    /// Elevation difference (positive = attacker higher)
    pub elevation_diff: i8,
    /// Whether attack is along formation length (enfilade)
    pub is_enfilade: bool,
    /// Line of sight quality
    pub los_quality: LosQuality,
    /// Effective cover value after angle adjustments (0.0-1.0)
    pub cover_value: f32,
}

impl EngagementGeometry {
    /// Combined hit modifier from all geometry factors
    ///
    /// Returns an additive modifier to base hit chance.
    /// Factors include:
    /// - Attack angle bonus
    /// - Elevation advantage/disadvantage
    /// - Cover penalty (if not negated by flanking)
    pub fn hit_modifier(&self) -> f32 {
        let mut modifier = self.attack_angle.hit_modifier();

        // Elevation bonus: +0.1 per level advantage, -0.05 per level disadvantage
        if self.elevation_diff > 0 {
            modifier += 0.1 * self.elevation_diff as f32;
        } else if self.elevation_diff < 0 {
            modifier -= 0.05 * (-self.elevation_diff) as f32;
        }

        // Cover penalty (negative modifier)
        modifier -= self.cover_value;

        // Apply LOS quality as a multiplier to the final result
        // Note: This affects the modifier, not base hit chance
        modifier * self.los_quality.hit_multiplier()
    }

    /// Damage multiplier from enfilade and elevation
    ///
    /// Returns a multiplicative factor for damage:
    /// - Enfilade: +25% damage (shots affect more of formation)
    /// - High ground: +10% per elevation level
    pub fn damage_multiplier(&self) -> f32 {
        let mut multiplier = 1.0;

        // Enfilade bonus
        if self.is_enfilade {
            multiplier += 0.25;
        }

        // Elevation bonus to damage (plunging fire)
        if self.elevation_diff > 0 {
            multiplier += 0.1 * self.elevation_diff as f32;
        }

        multiplier
    }
}

/// Calculate attack angle from positions and defender facing
///
/// Determines whether an attack comes from the front, flank, or rear
/// based on the direction from attacker to defender compared to
/// the defender's facing direction.
pub fn calculate_attack_angle(
    attacker_pos: BattleHexCoord,
    defender_pos: BattleHexCoord,
    defender_facing: HexDirection,
) -> AttackAngle {
    // Get direction FROM defender TO attacker (where the attack is coming from)
    let attack_direction = defender_pos.direction_to(attacker_pos);

    // Compare attack direction to defender's facing
    // If defender faces the same direction as the attack comes from, it's frontal
    let angle_diff = defender_facing.angle_difference(attack_direction);

    match angle_diff {
        0 | 1 => AttackAngle::Frontal, // 0-60 degrees from facing
        2 => {
            // 120 degrees - determine left or right flank
            // Use hex index to determine side
            let facing_idx = defender_facing.to_index();
            let attack_idx = attack_direction.to_index();

            // Calculate clockwise distance from facing to attack direction
            let clockwise_dist = (attack_idx as i8 - facing_idx as i8 + 6) % 6;

            if clockwise_dist == 2 {
                // Attack is 2 steps clockwise from facing = right flank
                AttackAngle::FlankRight
            } else {
                // Attack is 2 steps counter-clockwise from facing = left flank
                AttackAngle::FlankLeft
            }
        }
        3 => AttackAngle::Rear, // 180 degrees (opposite)
        _ => AttackAngle::Frontal, // Shouldn't happen, but safe default
    }
}

/// Calculate full engagement geometry between two units
///
/// Analyzes all tactical factors:
/// - Range between units
/// - Attack angle (frontal/flank/rear)
/// - Elevation difference
/// - Line of sight quality
/// - Cover from terrain (adjusted for attack angle)
/// - Enfilade potential based on formation shape
pub fn calculate_engagement_geometry(
    attacker: &BattleUnit,
    defender: &BattleUnit,
    map: &BattleMap,
) -> EngagementGeometry {
    let range = attacker.position.distance(&defender.position);
    let attack_angle = calculate_attack_angle(attacker.position, defender.position, defender.facing);
    let elevation_diff = map.elevation_difference(attacker.position, defender.position);

    // Check line of sight
    let has_los = map.has_line_of_sight(attacker.position, defender.position);
    let los_quality = if has_los {
        LosQuality::Clear
    } else {
        LosQuality::Blocked
    };

    // Calculate enfilade
    let is_enfilade = check_enfilade(attacker, defender);

    // Calculate cover
    let cover_value = calculate_cover(attacker, defender, &attack_angle, map);

    EngagementGeometry {
        range,
        attack_angle,
        elevation_diff,
        is_enfilade,
        los_quality,
        cover_value,
    }
}

/// Check if attack is along formation length (enfilade)
///
/// Enfilade fire occurs when attacking along the length of an enemy formation,
/// allowing shots to hit multiple ranks or files.
fn check_enfilade(attacker: &BattleUnit, defender: &BattleUnit) -> bool {
    // Get attack direction
    let attack_direction = defender.position.direction_to(attacker.position);

    // Determine formation orientation based on shape and facing
    // For Line formations: formation extends perpendicular to facing
    // For Column formations: formation extends parallel to facing
    match &defender.formation_shape {
        FormationShape::Line { .. } => {
            // Line extends perpendicular to facing
            // Enfilade if attack comes from the side (perpendicular to facing)
            // That means attack direction is ~90 degrees (2 steps) from facing
            let angle_diff = defender.facing.angle_difference(attack_direction);
            angle_diff == 2 // Attack from flank = enfilade on a line
        }
        FormationShape::Column { .. } => {
            // Column extends parallel to facing
            // Enfilade if attack comes from front or rear (along facing axis)
            let angle_diff = defender.facing.angle_difference(attack_direction);
            angle_diff == 0 || angle_diff == 3 // Attack from front/rear = enfilade on a column
        }
        FormationShape::Wedge { .. } => {
            // Wedge is angled, less vulnerable to enfilade
            false
        }
        FormationShape::Square => {
            // Square formation is resistant to enfilade from any direction
            false
        }
        FormationShape::Skirmish { .. } => {
            // Dispersed formation cannot be enfiladed
            false
        }
    }
}

/// Calculate effective cover value for defender from attacker's direction
///
/// Takes into account:
/// - Base terrain cover at defender's position
/// - Terrain features with directional protection
/// - Attack angle (flanking may negate cover)
fn calculate_cover(
    attacker: &BattleUnit,
    defender: &BattleUnit,
    attack_angle: &AttackAngle,
    map: &BattleMap,
) -> f32 {
    // If flanking negates cover, return 0
    if attack_angle.negates_cover() {
        return 0.0;
    }

    // Get defender's hex
    let defender_hex = match map.get_hex(defender.position) {
        Some(hex) => hex,
        None => return 0.0,
    };

    // Start with base terrain cover
    let mut cover = defender_hex.terrain.cover_value();

    // Get attack direction
    let attack_direction = defender.position.direction_to(attacker.position);

    // Add cover from directional features that block this attack direction
    for feature in &defender_hex.features {
        if feature.provides_cover_from(attack_direction) {
            cover += feature.defense_bonus();
        }
    }

    // Cap at 1.0
    cover.min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::unit_type::UnitType;
    use crate::battle::units::UnitId;

    #[test]
    fn test_attack_angle_from_front() {
        let attacker_pos = BattleHexCoord::new(5, 5);
        // Defender is directly in front (attacker at higher r, defender facing SouthEast toward attacker)
        let defender_pos = BattleHexCoord::new(5, 4);
        let defender_facing = HexDirection::SouthEast; // Facing toward attacker

        let angle = calculate_attack_angle(attacker_pos, defender_pos, defender_facing);
        assert!(matches!(angle, AttackAngle::Frontal));
    }

    #[test]
    fn test_attack_angle_from_rear() {
        let attacker_pos = BattleHexCoord::new(5, 5);
        let defender_pos = BattleHexCoord::new(5, 4);
        let defender_facing = HexDirection::NorthWest; // Facing away from attacker

        let angle = calculate_attack_angle(attacker_pos, defender_pos, defender_facing);
        assert!(matches!(angle, AttackAngle::Rear));
    }

    #[test]
    fn test_attack_angle_from_flank() {
        let attacker_pos = BattleHexCoord::new(6, 5);
        let defender_pos = BattleHexCoord::new(5, 5);
        let defender_facing = HexDirection::NorthWest; // Facing perpendicular

        let angle = calculate_attack_angle(attacker_pos, defender_pos, defender_facing);
        assert!(matches!(
            angle,
            AttackAngle::FlankLeft | AttackAngle::FlankRight
        ));
    }

    #[test]
    fn test_flanking_negates_cover() {
        assert!(!AttackAngle::Frontal.negates_cover());
        assert!(AttackAngle::FlankLeft.negates_cover());
        assert!(AttackAngle::FlankRight.negates_cover());
        assert!(AttackAngle::Rear.negates_cover());
    }

    #[test]
    fn test_attack_angle_hit_modifiers() {
        assert_eq!(AttackAngle::Frontal.hit_modifier(), 0.0);
        assert_eq!(AttackAngle::FlankLeft.hit_modifier(), 0.15);
        assert_eq!(AttackAngle::FlankRight.hit_modifier(), 0.15);
        assert_eq!(AttackAngle::Rear.hit_modifier(), 0.30);
    }

    #[test]
    fn test_stress_multipliers() {
        assert_eq!(AttackAngle::Frontal.stress_multiplier(), 1.0);
        assert_eq!(AttackAngle::FlankLeft.stress_multiplier(), 1.5);
        assert_eq!(AttackAngle::FlankRight.stress_multiplier(), 1.5);
        assert_eq!(AttackAngle::Rear.stress_multiplier(), 2.0);
    }

    #[test]
    fn test_los_quality_multipliers() {
        assert_eq!(LosQuality::Clear.hit_multiplier(), 1.0);
        assert_eq!(LosQuality::Partial.hit_multiplier(), 0.7);
        assert_eq!(LosQuality::Blocked.hit_multiplier(), 0.0);
    }

    #[test]
    fn test_enfilade_line_formation() {
        let mut attacker = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        attacker.position = BattleHexCoord::new(6, 5); // East of defender

        let mut defender = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        defender.position = BattleHexCoord::new(5, 5);
        defender.facing = HexDirection::NorthWest; // Line extends NE-SW
        defender.formation_shape = FormationShape::Line { depth: 2 };

        // Attack from East on a line facing NorthWest = enfilade (along the line)
        assert!(check_enfilade(&attacker, &defender));
    }

    #[test]
    fn test_no_enfilade_square_formation() {
        let mut attacker = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        attacker.position = BattleHexCoord::new(6, 5);

        let mut defender = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        defender.position = BattleHexCoord::new(5, 5);
        defender.facing = HexDirection::East;
        defender.formation_shape = FormationShape::Square;

        // Square is resistant to enfilade
        assert!(!check_enfilade(&attacker, &defender));
    }

    #[test]
    fn test_enfilade_column_formation() {
        let mut attacker = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        attacker.position = BattleHexCoord::new(6, 5); // East of defender

        let mut defender = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        defender.position = BattleHexCoord::new(5, 5);
        defender.facing = HexDirection::East; // Column extends East-West
        defender.formation_shape = FormationShape::Column { width: 2 };

        // Attack from East on a column facing East = enfilade (along the column)
        assert!(check_enfilade(&attacker, &defender));
    }

    #[test]
    fn test_engagement_geometry_basic() {
        let map = BattleMap::new(10, 10);

        let mut attacker = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        attacker.position = BattleHexCoord::new(5, 5);

        let mut defender = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        defender.position = BattleHexCoord::new(5, 3);
        defender.facing = HexDirection::SouthEast;

        let geometry = calculate_engagement_geometry(&attacker, &defender, &map);

        assert_eq!(geometry.range, 2);
        assert!(matches!(geometry.los_quality, LosQuality::Clear));
    }

    #[test]
    fn test_enfilade_damage_bonus() {
        let geometry = EngagementGeometry {
            range: 3,
            attack_angle: AttackAngle::Frontal,
            elevation_diff: 0,
            is_enfilade: true,
            los_quality: LosQuality::Clear,
            cover_value: 0.0,
        };

        assert!(geometry.damage_multiplier() > 1.0);
        assert_eq!(geometry.damage_multiplier(), 1.25);
    }

    #[test]
    fn test_elevation_damage_bonus() {
        let geometry = EngagementGeometry {
            range: 3,
            attack_angle: AttackAngle::Frontal,
            elevation_diff: 2, // Attacker is 2 levels higher
            is_enfilade: false,
            los_quality: LosQuality::Clear,
            cover_value: 0.0,
        };

        // +10% per level = +20%
        assert_eq!(geometry.damage_multiplier(), 1.2);
    }

    #[test]
    fn test_combined_enfilade_and_elevation() {
        let geometry = EngagementGeometry {
            range: 3,
            attack_angle: AttackAngle::Frontal,
            elevation_diff: 2,
            is_enfilade: true,
            los_quality: LosQuality::Clear,
            cover_value: 0.0,
        };

        // +25% enfilade + 20% elevation = 1.45
        assert!((geometry.damage_multiplier() - 1.45).abs() < 0.001);
    }

    #[test]
    fn test_hit_modifier_with_cover() {
        let geometry = EngagementGeometry {
            range: 3,
            attack_angle: AttackAngle::Frontal,
            elevation_diff: 0,
            is_enfilade: false,
            los_quality: LosQuality::Clear,
            cover_value: 0.3,
        };

        // Frontal: 0.0, no elevation, -0.3 cover = -0.3
        assert!((geometry.hit_modifier() - (-0.3)).abs() < 0.001);
    }

    #[test]
    fn test_hit_modifier_with_elevation_advantage() {
        let geometry = EngagementGeometry {
            range: 3,
            attack_angle: AttackAngle::Frontal,
            elevation_diff: 2,
            is_enfilade: false,
            los_quality: LosQuality::Clear,
            cover_value: 0.0,
        };

        // Frontal: 0.0, +0.1 * 2 elevation = 0.2
        assert!((geometry.hit_modifier() - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_hit_modifier_flank_attack() {
        let geometry = EngagementGeometry {
            range: 3,
            attack_angle: AttackAngle::FlankLeft,
            elevation_diff: 0,
            is_enfilade: false,
            los_quality: LosQuality::Clear,
            cover_value: 0.0, // Cover would be 0 anyway due to flanking
        };

        // Flank: +0.15
        assert!((geometry.hit_modifier() - 0.15).abs() < 0.001);
    }

    #[test]
    fn test_blocked_los_zeroes_hit_modifier() {
        let geometry = EngagementGeometry {
            range: 3,
            attack_angle: AttackAngle::Rear, // +0.30 bonus
            elevation_diff: 3,               // +0.30 bonus
            is_enfilade: true,
            los_quality: LosQuality::Blocked,
            cover_value: 0.0,
        };

        // Even with great bonuses, blocked LOS = 0
        assert_eq!(geometry.hit_modifier(), 0.0);
    }

    #[test]
    fn test_skirmish_no_enfilade() {
        let mut attacker = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        attacker.position = BattleHexCoord::new(6, 5);

        let mut defender = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        defender.position = BattleHexCoord::new(5, 5);
        defender.facing = HexDirection::East;
        defender.formation_shape = FormationShape::Skirmish { dispersion: 2.0 };

        // Skirmish cannot be enfiladed
        assert!(!check_enfilade(&attacker, &defender));
    }

    #[test]
    fn test_wedge_no_enfilade() {
        let mut attacker = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        attacker.position = BattleHexCoord::new(6, 5);

        let mut defender = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        defender.position = BattleHexCoord::new(5, 5);
        defender.facing = HexDirection::East;
        defender.formation_shape = FormationShape::Wedge { angle: 60.0 };

        // Wedge is less vulnerable to enfilade
        assert!(!check_enfilade(&attacker, &defender));
    }
}
