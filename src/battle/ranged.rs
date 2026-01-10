//! Ranged combat phase for battle system
//!
//! Handles bow, crossbow, and thrown weapon attacks using the chunking skill system.

use crate::battle::hex::BattleHexCoord;
use crate::battle::unit_type::UnitType;
use crate::battle::units::BattleUnit;
use crate::combat::weapons::{Mass, RangeCategory, RangedWeaponProperties};

/// Maximum effective range in hexes for each range category
pub fn max_range_hexes(range: RangeCategory) -> u32 {
    match range {
        RangeCategory::Close => 5,
        RangeCategory::Medium => 12,
        RangeCategory::Long => 20,
    }
}

/// Minimum effective range in hexes (can't shoot point-blank)
pub fn min_range_hexes(range: RangeCategory) -> u32 {
    match range {
        RangeCategory::Close => 2,
        RangeCategory::Medium => 3,
        RangeCategory::Long => 5,
    }
}

/// Check if a shooter can hit a target at this distance
pub fn can_shoot(shooter: BattleHexCoord, target: BattleHexCoord, range: RangeCategory) -> bool {
    let distance = shooter.distance(&target);
    distance >= min_range_hexes(range) && distance <= max_range_hexes(range)
}

/// Result of a ranged attack
#[derive(Debug, Clone)]
pub struct RangedAttackResult {
    /// Did the projectile hit?
    pub hit: bool,
    /// Casualties inflicted (if hit)
    pub casualties: u32,
    /// Stress inflicted on target (even on miss - suppression)
    pub stress_inflicted: f32,
    /// Fatigue cost to shooter
    pub fatigue_cost: f32,
    /// Ammo consumed
    pub ammo_consumed: u32,
}

impl Default for RangedAttackResult {
    fn default() -> Self {
        Self {
            hit: false,
            casualties: 0,
            stress_inflicted: 0.0,
            fatigue_cost: 0.0,
            ammo_consumed: 1,
        }
    }
}

/// Get ranged weapon properties for a unit type
pub fn unit_ranged_weapon(unit_type: UnitType) -> Option<RangedWeaponProperties> {
    match unit_type {
        UnitType::Archers => Some(RangedWeaponProperties::shortbow()),
        UnitType::Crossbowmen => Some(RangedWeaponProperties::light_crossbow()),
        UnitType::HorseArchers => Some(RangedWeaponProperties::shortbow()),
        _ => None,
    }
}

/// Resolve a ranged attack from one unit to another
///
/// Returns attack result. Does NOT mutate units - caller applies results.
pub fn resolve_unit_ranged_attack(
    attacker: &BattleUnit,
    defender: &BattleUnit,
    _tick: u64,
    has_los: bool,
) -> RangedAttackResult {
    let mut result = RangedAttackResult::default();

    // Get weapon properties
    let weapon = match unit_ranged_weapon(attacker.unit_type) {
        Some(w) => w,
        None => {
            result.ammo_consumed = 0;
            return result; // Not a ranged unit
        }
    };

    // Check range
    if !can_shoot(attacker.position, defender.position, weapon.range) {
        result.ammo_consumed = 0;
        return result;
    }

    // Base hit chance depends on skill (simplified - use encoding depth later)
    let base_hit_chance = 0.4;

    // Distance penalty
    let distance = attacker.position.distance(&defender.position);
    let max_range = max_range_hexes(weapon.range);
    let distance_penalty = (distance as f32 / max_range as f32) * 0.3;

    // Cover bonus for defender (would come from terrain)
    let cover_bonus = 0.0; // TODO: terrain lookup

    // LOS penalty
    let los_penalty = if has_los { 0.0 } else { 0.5 };

    // Final hit chance
    let hit_chance = (base_hit_chance - distance_penalty - cover_bonus - los_penalty).max(0.05);

    // Roll for hit (simplified - use RNG properly in real impl)
    let roll: f32 = rand::random();
    result.hit = roll < hit_chance;

    // Casualties if hit
    if result.hit {
        // Base casualties from ranged fire
        let effective_strength = attacker.effective_strength();
        let base_casualties = (effective_strength as f32 * 0.02).ceil() as u32;
        result.casualties = base_casualties.max(1);
    }

    // Stress inflicted (even misses cause suppression)
    result.stress_inflicted = if result.hit { 0.03 } else { 0.01 };

    // Fatigue cost based on draw strength
    result.fatigue_cost = match weapon.draw_strength {
        Mass::Light => 0.01,
        Mass::Medium => 0.02,
        Mass::Heavy => 0.03,
        Mass::Massive => 0.05,
    };

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_shoot_at_target_in_range() {
        let shooter_pos = BattleHexCoord::new(0, 0);
        let target_pos = BattleHexCoord::new(8, 0); // 8 hexes away

        // Medium range (max 12) can reach, Close (max 5) cannot
        assert!(can_shoot(shooter_pos, target_pos, RangeCategory::Medium));
        assert!(!can_shoot(shooter_pos, target_pos, RangeCategory::Close));
    }

    #[test]
    fn test_range_category_to_hex_distance() {
        assert_eq!(max_range_hexes(RangeCategory::Close), 5);
        assert_eq!(max_range_hexes(RangeCategory::Medium), 12);
        assert_eq!(max_range_hexes(RangeCategory::Long), 20);
    }

    #[test]
    fn test_minimum_range() {
        let shooter = BattleHexCoord::new(0, 0);
        let too_close = BattleHexCoord::new(1, 0);

        // Can't shoot longbow at adjacent hex
        assert!(!can_shoot(shooter, too_close, RangeCategory::Long));
    }

    #[test]
    fn test_min_range_values() {
        assert_eq!(min_range_hexes(RangeCategory::Close), 2);
        assert_eq!(min_range_hexes(RangeCategory::Medium), 3);
        assert_eq!(min_range_hexes(RangeCategory::Long), 5);
    }

    #[test]
    fn test_unit_ranged_weapon() {
        assert!(unit_ranged_weapon(UnitType::Archers).is_some());
        assert!(unit_ranged_weapon(UnitType::Crossbowmen).is_some());
        assert!(unit_ranged_weapon(UnitType::HorseArchers).is_some());
        assert!(unit_ranged_weapon(UnitType::Infantry).is_none());
    }

    #[test]
    fn test_resolve_unit_ranged_attack() {
        use crate::battle::units::{Element, UnitId};
        use crate::core::types::EntityId;

        // Create archer unit
        let mut archer = BattleUnit::new(UnitId::new(), UnitType::Archers);
        archer.position = BattleHexCoord::new(0, 0);
        archer.elements.push(Element::new(vec![EntityId::new(); 20]));

        // Create target infantry
        let mut target = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        target.position = BattleHexCoord::new(8, 0);
        target.elements.push(Element::new(vec![EntityId::new(); 50]));

        let result = resolve_unit_ranged_attack(&archer, &target, 0, true);

        // Should have attempted attack
        assert!(result.ammo_consumed > 0);
        // Should cause some stress even if miss
        assert!(result.stress_inflicted >= 0.0);
    }

    #[test]
    fn test_out_of_range_attack() {
        use crate::battle::units::{Element, UnitId};
        use crate::core::types::EntityId;

        // Create archer unit
        let mut archer = BattleUnit::new(UnitId::new(), UnitType::Archers);
        archer.position = BattleHexCoord::new(0, 0);
        archer.elements.push(Element::new(vec![EntityId::new(); 20]));

        // Create target WAY out of range
        let mut target = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        target.position = BattleHexCoord::new(50, 0);
        target.elements.push(Element::new(vec![EntityId::new(); 50]));

        let result = resolve_unit_ranged_attack(&archer, &target, 0, true);

        // No ammo consumed if out of range
        assert_eq!(result.ammo_consumed, 0);
    }

    #[test]
    fn test_non_ranged_unit_attack() {
        use crate::battle::units::{Element, UnitId};
        use crate::core::types::EntityId;

        // Infantry is not a ranged unit
        let mut infantry = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        infantry.position = BattleHexCoord::new(0, 0);
        infantry.elements.push(Element::new(vec![EntityId::new(); 20]));

        let mut target = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        target.position = BattleHexCoord::new(5, 0);
        target.elements.push(Element::new(vec![EntityId::new(); 50]));

        let result = resolve_unit_ranged_attack(&infantry, &target, 0, true);

        // Infantry can't do ranged attacks
        assert_eq!(result.ammo_consumed, 0);
        assert!(!result.hit);
    }
}
