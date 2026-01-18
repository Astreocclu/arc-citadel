//! Battle resolution for campaign layer
//!
//! When armies engage, this system resolves the battle outcome
//! using a simplified model based on army strength, morale, terrain, and weather.

use serde::{Deserialize, Serialize};

use super::map::{CampaignMap, CampaignTerrain, HexCoord};
use super::route::{Army, ArmyId, ArmyStance};
use super::weather::Weather;

/// Base casualties per combat round as fraction of smaller army
pub const BASE_CASUALTY_RATE: f32 = 0.05;

/// Morale loss per combat round
pub const BASE_MORALE_LOSS: f32 = 0.1;

/// Morale threshold for routing
pub const ROUT_THRESHOLD: f32 = 0.2;

/// Combat strength modifiers by stance
pub const AGGRESSIVE_ATTACK_BONUS: f32 = 1.2;
pub const AGGRESSIVE_DEFENSE_PENALTY: f32 = 0.8;
pub const DEFENSIVE_DEFENSE_BONUS: f32 = 1.3;
pub const DEFENSIVE_ATTACK_PENALTY: f32 = 0.7;

/// Outcome of a battle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattleOutcome {
    /// Attacker won decisively
    AttackerVictory,
    /// Defender won decisively
    DefenderVictory,
    /// Both sides withdrew
    Draw,
    /// Battle continues (not resolved in one tick)
    Ongoing,
}

/// Results of a battle resolution
#[derive(Debug, Clone)]
pub struct BattleResult {
    pub outcome: BattleOutcome,
    pub attacker_id: ArmyId,
    pub defender_id: ArmyId,
    pub position: HexCoord,
    pub attacker_casualties: u32,
    pub defender_casualties: u32,
    pub attacker_routed: bool,
    pub defender_routed: bool,
    pub rounds_fought: u32,
}

/// Calculate combat strength of an army
pub fn calculate_combat_strength(
    army: &Army,
    terrain: CampaignTerrain,
    weather: Weather,
    is_attacker: bool,
) -> f32 {
    let base_strength = army.unit_count as f32;

    // Morale affects effectiveness
    let morale_mod = 0.5 + (army.morale * 0.5); // 50% to 100% effectiveness

    // Stance modifiers
    let stance_mod = match (army.stance, is_attacker) {
        (ArmyStance::Aggressive, true) => AGGRESSIVE_ATTACK_BONUS,
        (ArmyStance::Aggressive, false) => AGGRESSIVE_DEFENSE_PENALTY,
        (ArmyStance::Defensive, true) => DEFENSIVE_ATTACK_PENALTY,
        (ArmyStance::Defensive, false) => DEFENSIVE_DEFENSE_BONUS,
        (ArmyStance::Evasive, _) => 0.8, // Evasive is weak in combat
    };

    // Terrain defense bonus (only for defender)
    let terrain_mod = if is_attacker {
        1.0
    } else {
        1.0 + terrain.defense_bonus()
    };

    // Weather affects both sides
    let weather_mod = weather.movement_modifier(); // Bad weather = worse combat

    base_strength * morale_mod * stance_mod * terrain_mod * weather_mod
}

/// Calculate casualties for a combat round
fn calculate_casualties(attacker_strength: f32, defender_strength: f32, smaller_army: u32) -> (u32, u32) {
    let total_strength = attacker_strength + defender_strength;
    if total_strength <= 0.0 {
        return (0, 0);
    }

    let attacker_ratio = attacker_strength / total_strength;
    let defender_ratio = defender_strength / total_strength;

    // Casualties based on relative strength
    let base_casualties = (smaller_army as f32 * BASE_CASUALTY_RATE) as u32;

    // Stronger side inflicts more casualties
    let attacker_casualties = ((base_casualties as f32) * defender_ratio * 1.5) as u32;
    let defender_casualties = ((base_casualties as f32) * attacker_ratio * 1.5) as u32;

    (attacker_casualties.max(1), defender_casualties.max(1))
}

/// Calculate morale loss for a combat round
fn calculate_morale_loss(own_casualties: u32, own_units: u32, enemy_casualties: u32) -> f32 {
    if own_units == 0 {
        return 1.0; // Instant rout
    }

    let casualty_rate = own_casualties as f32 / own_units as f32;

    // Morale loss based on own casualties
    let casualty_morale = casualty_rate * 0.5;

    // Slight morale gain if inflicting heavy casualties
    let revenge_bonus = if enemy_casualties > own_casualties {
        0.02
    } else {
        0.0
    };

    (BASE_MORALE_LOSS + casualty_morale - revenge_bonus).max(0.0)
}

/// Resolve a single combat round between two armies
pub fn resolve_combat_round(
    attacker: &mut Army,
    defender: &mut Army,
    terrain: CampaignTerrain,
    weather: Weather,
) -> (u32, u32) {
    let attacker_strength = calculate_combat_strength(attacker, terrain, weather, true);
    let defender_strength = calculate_combat_strength(defender, terrain, weather, false);

    let smaller_army = attacker.unit_count.min(defender.unit_count);
    let (attacker_cas, defender_cas) = calculate_casualties(attacker_strength, defender_strength, smaller_army);

    // Apply casualties
    attacker.unit_count = attacker.unit_count.saturating_sub(attacker_cas);
    defender.unit_count = defender.unit_count.saturating_sub(defender_cas);

    // Apply morale loss
    let attacker_morale_loss = calculate_morale_loss(attacker_cas, attacker.unit_count + attacker_cas, defender_cas);
    let defender_morale_loss = calculate_morale_loss(defender_cas, defender.unit_count + defender_cas, attacker_cas);

    attacker.morale = (attacker.morale - attacker_morale_loss).max(0.0);
    defender.morale = (defender.morale - defender_morale_loss).max(0.0);

    (attacker_cas, defender_cas)
}

/// Check if an army should rout
pub fn check_rout(army: &Army) -> bool {
    army.morale < ROUT_THRESHOLD || army.unit_count == 0
}

/// Resolve a full battle between two armies
/// Returns battle result and modifies armies in place
pub fn resolve_battle(
    attacker: &mut Army,
    defender: &mut Army,
    map: &CampaignMap,
    weather: Weather,
    max_rounds: u32,
) -> BattleResult {
    let position = defender.position;
    let terrain = map
        .get(&position)
        .map(|t| t.terrain)
        .unwrap_or_default();

    let attacker_id = attacker.id;
    let defender_id = defender.id;

    let mut total_attacker_casualties = 0u32;
    let mut total_defender_casualties = 0u32;
    let mut rounds = 0u32;

    // Fight rounds until one side routs or max rounds reached
    while rounds < max_rounds {
        rounds += 1;

        let (att_cas, def_cas) = resolve_combat_round(attacker, defender, terrain, weather);
        total_attacker_casualties += att_cas;
        total_defender_casualties += def_cas;

        // Check for rout
        let attacker_routs = check_rout(attacker);
        let defender_routs = check_rout(defender);

        if attacker_routs && defender_routs {
            // Both sides break - draw
            return BattleResult {
                outcome: BattleOutcome::Draw,
                attacker_id,
                defender_id,
                position,
                attacker_casualties: total_attacker_casualties,
                defender_casualties: total_defender_casualties,
                attacker_routed: true,
                defender_routed: true,
                rounds_fought: rounds,
            };
        } else if attacker_routs {
            return BattleResult {
                outcome: BattleOutcome::DefenderVictory,
                attacker_id,
                defender_id,
                position,
                attacker_casualties: total_attacker_casualties,
                defender_casualties: total_defender_casualties,
                attacker_routed: true,
                defender_routed: false,
                rounds_fought: rounds,
            };
        } else if defender_routs {
            return BattleResult {
                outcome: BattleOutcome::AttackerVictory,
                attacker_id,
                defender_id,
                position,
                attacker_casualties: total_attacker_casualties,
                defender_casualties: total_defender_casualties,
                attacker_routed: false,
                defender_routed: true,
                rounds_fought: rounds,
            };
        }
    }

    // Max rounds reached - ongoing battle
    BattleResult {
        outcome: BattleOutcome::Ongoing,
        attacker_id,
        defender_id,
        position,
        attacker_casualties: total_attacker_casualties,
        defender_casualties: total_defender_casualties,
        attacker_routed: false,
        defender_routed: false,
        rounds_fought: rounds,
    }
}

/// Apply retreat to a routed army (move away from battle)
pub fn apply_retreat(army: &mut Army, battle_position: HexCoord, map: &CampaignMap) {
    // Find a valid hex to retreat to (away from battle)
    let neighbors = map.passable_neighbors(&army.position);

    // Prefer hexes further from battle position
    let retreat_hex = neighbors
        .iter()
        .max_by_key(|h| h.distance(&battle_position))
        .copied();

    if let Some(hex) = retreat_hex {
        army.position = hex;
    }

    // Clear orders - routed army needs to regroup
    army.orders = None;
    army.path_cache = None;

    // Routed armies suffer additional morale penalty
    army.morale = (army.morale - 0.1).max(0.0);
}

/// Battle events for campaign log
#[derive(Debug, Clone)]
pub enum BattleEvent {
    BattleStarted {
        attacker: ArmyId,
        defender: ArmyId,
        position: HexCoord,
    },
    BattleRound {
        attacker: ArmyId,
        defender: ArmyId,
        attacker_casualties: u32,
        defender_casualties: u32,
        round: u32,
    },
    BattleEnded {
        result: BattleResult,
    },
    ArmyRouted {
        army: ArmyId,
        retreated_to: HexCoord,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::PolityId;

    fn test_army(id: u32, units: u32, stance: ArmyStance) -> Army {
        let mut army = Army::new(
            ArmyId(id),
            format!("Army {}", id),
            PolityId(id),
            HexCoord::new(5, 5),
        );
        army.unit_count = units;
        army.stance = stance;
        army
    }

    #[test]
    fn test_combat_strength() {
        let army = test_army(1, 100, ArmyStance::Defensive);

        let attack_str = calculate_combat_strength(&army, CampaignTerrain::Plains, Weather::Clear, true);
        let defend_str = calculate_combat_strength(&army, CampaignTerrain::Plains, Weather::Clear, false);

        // Defensive stance should be stronger when defending
        assert!(defend_str > attack_str);
    }

    #[test]
    fn test_terrain_bonus() {
        let army = test_army(1, 100, ArmyStance::Defensive);

        let plains_def = calculate_combat_strength(&army, CampaignTerrain::Plains, Weather::Clear, false);
        let hills_def = calculate_combat_strength(&army, CampaignTerrain::Hills, Weather::Clear, false);
        let mountains_def = calculate_combat_strength(&army, CampaignTerrain::Mountains, Weather::Clear, false);

        // Higher terrain should give better defense
        assert!(hills_def > plains_def);
        assert!(mountains_def > hills_def);
    }

    #[test]
    fn test_combat_round() {
        let mut attacker = test_army(1, 100, ArmyStance::Aggressive);
        let mut defender = test_army(2, 100, ArmyStance::Defensive);

        let (att_cas, def_cas) = resolve_combat_round(
            &mut attacker,
            &mut defender,
            CampaignTerrain::Plains,
            Weather::Clear,
        );

        assert!(att_cas > 0);
        assert!(def_cas > 0);
        assert!(attacker.unit_count < 100);
        assert!(defender.unit_count < 100);
    }

    #[test]
    fn test_full_battle() {
        let mut attacker = test_army(1, 200, ArmyStance::Aggressive);
        let mut defender = test_army(2, 100, ArmyStance::Defensive);

        let map = CampaignMap::generate_simple(10, 10, 42);

        let result = resolve_battle(&mut attacker, &mut defender, &map, Weather::Clear, 20);

        // Larger army should generally win
        assert!(matches!(result.outcome, BattleOutcome::AttackerVictory | BattleOutcome::Ongoing));
        assert!(result.rounds_fought > 0);
    }

    #[test]
    fn test_rout_check() {
        let mut army = test_army(1, 100, ArmyStance::Defensive);

        army.morale = 0.5;
        assert!(!check_rout(&army));

        army.morale = 0.1;
        assert!(check_rout(&army));

        army.morale = 0.5;
        army.unit_count = 0;
        assert!(check_rout(&army));
    }

    #[test]
    fn test_retreat() {
        let map = CampaignMap::generate_simple(10, 10, 42);
        let mut army = test_army(1, 50, ArmyStance::Defensive);
        army.position = HexCoord::new(5, 5);

        let battle_position = HexCoord::new(5, 5);
        apply_retreat(&mut army, battle_position, &map);

        // Should have moved
        assert_ne!(army.position, battle_position);
        // Orders should be cleared
        assert!(army.orders.is_none());
    }
}
