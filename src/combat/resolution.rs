//! Combat exchange resolution
//!
//! An exchange occurs when PRESSING meets any other stance.
//! NO PERCENTAGE MODIFIERS. Property comparisons only.
//!
//! ## Closing Mechanic
//! When a shorter-reach attacker engages a longer-reach defender, the attacker
//! must "close" the distance. This costs them one free hit from the defender
//! (the "closing casualty"), but then both fight at equal reach.
//!
//! This models charging through a pike hedge - you take losses on the way in,
//! but once you're inside their guard, pikes become unwieldy.

use crate::combat::{
    combine_results, resolve_penetration, resolve_trauma, ArmorProperties, BodyZone, CombatSkill,
    CombatStance, Reach, SkillLevel, WeaponProperties, WeaponSpecial, Wound,
};

/// A combatant in an exchange
#[derive(Debug, Clone)]
pub struct Combatant {
    pub weapon: WeaponProperties,
    pub armor: ArmorProperties,
    pub stance: CombatStance,
    pub skill: CombatSkill,
    /// Whether this combatant is mounted (affects anti-cavalry weapons)
    pub is_mounted: bool,
}

impl Combatant {
    /// Test combatant: swordsman with no armor
    pub fn test_swordsman() -> Self {
        Self {
            weapon: WeaponProperties::sword(),
            armor: ArmorProperties::none(),
            stance: CombatStance::Pressing,
            skill: CombatSkill::trained(),
            is_mounted: false,
        }
    }

    /// Test combatant: spearman with no armor
    pub fn test_spearman() -> Self {
        Self {
            weapon: WeaponProperties::spear(),
            armor: ArmorProperties::none(),
            stance: CombatStance::Pressing,
            skill: CombatSkill::trained(),
            is_mounted: false,
        }
    }

    /// Test combatant: plate knight with sword
    pub fn test_plate_knight() -> Self {
        Self {
            weapon: WeaponProperties::sword(),
            armor: ArmorProperties::plate(),
            stance: CombatStance::Neutral,
            skill: CombatSkill::veteran(),
            is_mounted: false,
        }
    }

    /// Test combatant: unarmored civilian
    pub fn test_unarmored() -> Self {
        Self {
            weapon: WeaponProperties::fists(),
            armor: ArmorProperties::none(),
            stance: CombatStance::Neutral,
            skill: CombatSkill::novice(),
            is_mounted: false,
        }
    }

    /// Test combatant: mounted heavy cavalry
    pub fn test_heavy_cavalry() -> Self {
        Self {
            weapon: WeaponProperties {
                edge: crate::combat::Edge::Sharp,
                mass: crate::combat::Mass::Heavy,
                reach: Reach::Medium,
                special: vec![],
            },
            armor: ArmorProperties::plate(),
            stance: CombatStance::Pressing,
            skill: CombatSkill::veteran(),
            is_mounted: true,
        }
    }
}

/// Result of an exchange
#[derive(Debug, Clone)]
pub struct ExchangeResult {
    /// Did attacker hit defender?
    pub defender_hit: bool,
    /// Did defender hit attacker?
    pub attacker_hit: bool,
    /// Who struck first (if both attacked)?
    pub attacker_struck_first: bool,
    /// Wound to defender (if any)
    pub defender_wound: Option<Wound>,
    /// Wound to attacker (if any)
    pub attacker_wound: Option<Wound>,
    /// Wound from closing (attacker charging through reach disadvantage)
    pub closing_wound: Option<Wound>,
}

/// Select a hit zone (deterministic based on skill)
pub fn select_hit_zone(skill: SkillLevel) -> BodyZone {
    // Higher skill = more likely to hit vital areas
    // This is deterministic, not random
    match skill {
        SkillLevel::Master => BodyZone::Head,
        SkillLevel::Veteran => BodyZone::Torso,
        SkillLevel::Trained => BodyZone::Torso,
        SkillLevel::Novice => BodyZone::Torso,
    }
}

/// Resolve a single hit
pub fn resolve_hit(weapon: &WeaponProperties, armor: &ArmorProperties, zone: BodyZone) -> Wound {
    resolve_hit_with_mounted(weapon, armor, zone, false)
}

/// Resolve a single hit with mounted flag
pub fn resolve_hit_with_mounted(
    weapon: &WeaponProperties,
    armor: &ArmorProperties,
    zone: BodyZone,
    target_is_mounted: bool,
) -> Wound {
    let has_piercing = weapon.has_special(WeaponSpecial::Piercing);
    let has_anti_cavalry = weapon.has_special(WeaponSpecial::AntiCavalry);
    let pen = resolve_penetration(weapon.edge, armor.rigidity, has_piercing, has_anti_cavalry, target_is_mounted);
    let trauma = resolve_trauma(weapon.mass, armor.padding);
    combine_results(pen, trauma, zone)
}

/// Calculate the reach difference for closing
/// Returns how many "reach levels" the attacker must close
fn reach_difference(attacker_reach: Reach, defender_reach: Reach) -> u8 {
    let att_level = match attacker_reach {
        Reach::Grapple => 0,
        Reach::Short => 1,
        Reach::Medium => 2,
        Reach::Long => 3,
        Reach::Pike => 4,
    };
    let def_level = match defender_reach {
        Reach::Grapple => 0,
        Reach::Short => 1,
        Reach::Medium => 2,
        Reach::Long => 3,
        Reach::Pike => 4,
    };

    if def_level > att_level {
        def_level - att_level
    } else {
        0
    }
}

/// Resolve an exchange between attacker and defender
///
/// # Arguments
/// * `attacker` - The combatant initiating (must be PRESSING)
/// * `defender` - The combatant receiving
///
/// # Returns
/// Exchange result with hits and wounds
///
/// # Closing Mechanic
/// If attacker has shorter reach than defender, attacker takes a "closing wound"
/// representing the cost of charging through the reach advantage. After closing,
/// both fight at equal footing.
pub fn resolve_exchange(attacker: &Combatant, defender: &Combatant) -> ExchangeResult {
    // Step 1: Check if defender can respond
    let defender_can_respond = !defender.stance.vulnerable();

    if !defender_can_respond {
        // Free hit - defender is recovering or broken
        let zone = select_hit_zone(attacker.skill.level);
        let wound = resolve_hit_with_mounted(&attacker.weapon, &defender.armor, zone, defender.is_mounted);

        return ExchangeResult {
            defender_hit: true,
            attacker_hit: false,
            attacker_struck_first: true,
            defender_wound: Some(wound),
            attacker_wound: None,
            closing_wound: None,
        };
    }

    // Step 2: Check for reach disadvantage and apply closing mechanic
    let attacker_reach = attacker.weapon.reach;
    let defender_reach = defender.weapon.reach;
    let reach_gap = reach_difference(attacker_reach, defender_reach);

    let closing_wound = if reach_gap > 0 && defender.stance.can_riposte() {
        // Attacker must close - they take a free hit from the defender
        // The hit quality is based on reach difference (bigger gap = worse hit)
        let zone = match reach_gap {
            1 => BodyZone::ArmLeft,  // Small gap - glancing hit to arm
            2 => BodyZone::Torso,    // Medium gap - solid hit
            _ => BodyZone::Torso,    // Large gap - solid hit (pike vs grapple)
        };
        Some(resolve_hit_with_mounted(&defender.weapon, &attacker.armor, zone, attacker.is_mounted))
    } else {
        None
    };

    // Step 3: After closing, both fight at equal reach
    // Attacker struck first because they initiated (pressing stance)
    // This is the "inside the guard" phase where reach no longer matters

    // Step 4: Resolve attacker's hit on defender (check if defender is mounted)
    let attacker_zone = select_hit_zone(attacker.skill.level);
    let defender_wound = resolve_hit_with_mounted(&attacker.weapon, &defender.armor, attacker_zone, defender.is_mounted);

    // Step 5: Resolve defender's counter (if they can riposte)
    // After closing, defender can counter-attack normally
    let (attacker_hit, attacker_wound) = if defender.stance.can_riposte() {
        let defender_zone = select_hit_zone(defender.skill.level);
        let wound = resolve_hit_with_mounted(&defender.weapon, &attacker.armor, defender_zone, attacker.is_mounted);
        (true, Some(wound))
    } else {
        (false, None)
    };

    ExchangeResult {
        defender_hit: true,
        attacker_hit,
        attacker_struck_first: true, // Attacker always strikes first after closing
        defender_wound: Some(defender_wound),
        attacker_wound,
        closing_wound,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::body_zone::WoundSeverity;

    #[test]
    fn test_pressing_vs_recovering_is_free_hit() {
        let mut attacker = Combatant::test_swordsman();
        attacker.stance = CombatStance::Pressing;

        let mut defender = Combatant::test_unarmored();
        defender.stance = CombatStance::Recovering;

        let result = resolve_exchange(&attacker, &defender);

        assert!(result.defender_hit);
        assert!(!result.attacker_hit);
        assert!(result.closing_wound.is_none()); // No closing vs vulnerable
    }

    #[test]
    fn test_closing_mechanic_sword_vs_spear() {
        // Swordsman (Short reach) attacks Spearman (Long reach)
        let mut attacker = Combatant::test_swordsman();
        attacker.stance = CombatStance::Pressing;

        let mut defender = Combatant::test_spearman();
        defender.stance = CombatStance::Defensive; // Can riposte

        let result = resolve_exchange(&attacker, &defender);

        // Attacker should take a closing wound
        assert!(
            result.closing_wound.is_some(),
            "Shorter reach attacker should take closing wound"
        );

        // Both should still exchange hits after closing
        assert!(result.defender_hit, "Attacker should hit defender after closing");
        assert!(result.attacker_hit, "Defender should riposte after closing");
    }

    #[test]
    fn test_no_closing_wound_when_attacker_has_reach() {
        // Spearman (Long reach) attacks Swordsman (Short reach)
        let mut attacker = Combatant::test_spearman();
        attacker.stance = CombatStance::Pressing;

        let mut defender = Combatant::test_swordsman();
        defender.stance = CombatStance::Defensive;

        let result = resolve_exchange(&attacker, &defender);

        // No closing wound - attacker has reach advantage
        assert!(
            result.closing_wound.is_none(),
            "Longer reach attacker should NOT take closing wound"
        );
    }

    #[test]
    fn test_equal_reach_no_closing() {
        // Two swordsmen - equal reach
        let mut attacker = Combatant::test_swordsman();
        attacker.stance = CombatStance::Pressing;

        let mut defender = Combatant::test_swordsman();
        defender.stance = CombatStance::Defensive;

        let result = resolve_exchange(&attacker, &defender);

        assert!(
            result.closing_wound.is_none(),
            "Equal reach should have no closing wound"
        );
    }

    #[test]
    fn test_sword_vs_plate_no_wound() {
        let attacker = Combatant::test_swordsman();
        let defender = Combatant::test_plate_knight();

        let result = resolve_exchange(&attacker, &defender);

        if let Some(wound) = &result.defender_wound {
            // Sharp vs Plate = Deflect, Medium vs Heavy = Fatigue
            // Worse of (None, None) = None
            assert_eq!(wound.severity, WoundSeverity::None);
        }
    }

    #[test]
    fn test_both_combatants_can_be_hit() {
        // Two swordsmen attacking each other
        let mut attacker = Combatant::test_swordsman();
        attacker.stance = CombatStance::Pressing;

        let mut defender = Combatant::test_swordsman();
        defender.stance = CombatStance::Pressing; // Both attacking

        let result = resolve_exchange(&attacker, &defender);

        // Equal reach = simultaneous, both should be hit
        assert!(result.defender_hit);
        assert!(result.attacker_hit);
    }
}
