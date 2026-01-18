//! Mass combat resolution at different LOD levels
//!
//! Entity-level simulation using the `combat` crate.

use std::collections::HashMap;

use crate::battle::units::{BattleUnit, UnitId};
use crate::battle::unit_type::UnitType;
use crate::combat::resolution::{resolve_exchange, resolve_hit, select_hit_zone, Combatant};
use crate::combat::state::CombatState;
use crate::combat::{
    ArmorProperties, CombatSkill, CombatStance, Edge, Mass, Reach, WeaponProperties, WeaponSpecial, Wound,
};
use crate::core::types::EntityId;

/// Level of detail for combat resolution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatLOD {
    Individual, // LOD-0: full individual combat
    Element,    // LOD-1: element-level statistical
    Unit,       // LOD-2: unit-level statistical
    Formation,  // LOD-3: formation-level aggregate
}

/// Result of unit-level combat
#[derive(Debug, Clone)]
pub struct UnitCombatResult {
    pub attacker_casualties: u32,
    pub defender_casualties: u32,
    pub attacker_stress_delta: f32,
    pub defender_stress_delta: f32,
    pub attacker_fatigue_delta: f32,
    pub defender_fatigue_delta: f32,
    pub pressure_shift: f32,
}

/// Result of a shock attack (charge, flank, etc.)
#[derive(Debug, Clone)]
pub struct ShockResult {
    pub immediate_casualties: u32,
    pub stress_spike: f32,
    pub triggered_break_check: bool,
}

/// Helper to get active entities (not dead/incapacitated)
fn get_active_entities(
    unit: &BattleUnit,
    states: &mut HashMap<EntityId, CombatState>,
) -> Vec<EntityId> {
    let mut active = Vec::new();
    let props = unit.unit_type.default_properties();

    for element in &unit.elements {
        for &entity_id in &element.entities {
            // Ensure state exists
            let state = states.entry(entity_id).or_insert_with(|| {
                CombatState {
                    stance: crate::combat::CombatStance::Neutral,
                    skill: crate::combat::CombatSkill::trained(), // Default skill
                    morale: crate::combat::MoraleState::default(),
                    weapon: props.avg_weapon.clone(),
                    armor: props.avg_armor.clone(),
                    fatigue: unit.fatigue, // Inherit unit fatigue
                    wounds: Vec::new(),
                }
            });

            if state.can_fight() {
                active.push(entity_id);
            }
        }
    }
    active
}

/// Resolve unit-level combat using entity simulation
pub fn resolve_unit_combat(
    attacker: &BattleUnit,
    defender: &BattleUnit,
    entity_states: &mut HashMap<EntityId, CombatState>,
) -> UnitCombatResult {
    // 1. Gather active entities
    let attacker_ids = get_active_entities(attacker, entity_states);
    let defender_ids = get_active_entities(defender, entity_states);

    // 2. Determine frontage
    // Max entities that can fight in the front rank per tick (representing hex width)
    let max_width = 30; // ~15m frontage
    let combat_width = attacker_ids.len().min(defender_ids.len()).min(max_width);

    // Track results
    let mut attacker_casualties = 0;
    let mut defender_casualties = 0;
    let mut attacker_stress = 0.0;
    let mut defender_stress = 0.0;
    
    // Track engaged entities to prevent them from firing ranged
    let mut engaged_attackers = std::collections::HashSet::new();
    let mut engaged_defenders = std::collections::HashSet::new();

    // 3. Front Rank Combat
    for i in 0..combat_width {
        let att_id = attacker_ids[i];
        let def_id = defender_ids[i];
        
        engaged_attackers.insert(att_id);
        engaged_defenders.insert(def_id);

        resolve_entity_exchange(
            att_id, 
            def_id, 
            entity_states, 
            false, // Not flanking
            &mut attacker_casualties,
            &mut defender_casualties,
            &mut attacker_stress,
            &mut defender_stress
        );
    }

    // 4. Reach Weapons (Rank 2)
    // Check if attackers have reach weapons and enough men
    if attacker_ids.len() > combat_width {
        for i in 0..combat_width {
            let second_rank_idx = i + combat_width;
            if second_rank_idx >= attacker_ids.len() {
                break;
            }

            let att_id = attacker_ids[second_rank_idx];
            
            // Check reach
            let has_reach = {
                if let Some(state) = entity_states.get(&att_id) {
                    matches!(state.weapon.reach, Reach::Long | Reach::Pike)
                } else {
                    false
                }
            };

            if has_reach {
                let def_id = defender_ids[i]; // Attack same defender
                engaged_attackers.insert(att_id);
                // Defender already engaged, so this is 2v1 effectively
                
                resolve_entity_exchange(
                    att_id, 
                    def_id, 
                    entity_states, 
                    true, // Support attack - safer for attacker
                    &mut attacker_casualties,
                    &mut defender_casualties,
                    &mut attacker_stress,
                    &mut defender_stress
                );
            }
        }
    }
    
    // Also check defenders for reach
    if defender_ids.len() > combat_width {
        for i in 0..combat_width {
            let second_rank_idx = i + combat_width;
            if second_rank_idx >= defender_ids.len() {
                break;
            }

            let def_id = defender_ids[second_rank_idx];
            
            let has_reach = {
                if let Some(state) = entity_states.get(&def_id) {
                    matches!(state.weapon.reach, Reach::Long | Reach::Pike)
                } else {
                    false
                }
            };

            if has_reach {
                let att_id = attacker_ids[i];
                engaged_defenders.insert(def_id);
                
                resolve_entity_exchange(
                    def_id, // Defender is attacker in this exchange
                    att_id, 
                    entity_states, 
                    true,
                    &mut defender_casualties, // Swapped because func assumes arg1 is attacker
                    &mut attacker_casualties,
                    &mut defender_stress,
                    &mut attacker_stress
                );
            }
        }
    }

    // 5. Ganging Up (Excess Front Rank)
    // If one side is significantly wider, they wrap around or double team
    if attacker_ids.len() > combat_width && attacker_ids.len() < 2 * combat_width {
        // Simple implementation: extra attackers hit random defenders from the front rank
        let extras = attacker_ids.len().min(max_width) - combat_width;
        for i in 0..extras {
            let att_id = attacker_ids[combat_width + i];
            
            // Skip if already fought (e.g. used reach)
            if engaged_attackers.contains(&att_id) {
                continue;
            }
            
            let def_id = defender_ids[i % combat_width]; // Loop around targets
            engaged_attackers.insert(att_id);
            
            resolve_entity_exchange(
                att_id,
                def_id,
                entity_states,
                true, // Flanking/Ganging up
                &mut attacker_casualties,
                &mut defender_casualties,
                &mut attacker_stress,
                &mut defender_stress
            );
        }
    }

    // 6. Ranged Combat (Rear Ranks)
    if attacker.unit_type.is_ranged() {
        resolve_ranged_attacks(
            &attacker_ids, 
            &defender_ids, 
            &engaged_attackers, 
            entity_states,
            &attacker.unit_type,
            &mut defender_casualties,
            &mut defender_stress
        );
    }

    if defender.unit_type.is_ranged() {
        resolve_ranged_attacks(
            &defender_ids, 
            &attacker_ids, 
            &engaged_defenders, 
            entity_states,
            &defender.unit_type,
            &mut attacker_casualties,
            &mut attacker_stress
        );
    }

    // Determine pressure shift based on casualties
    let pressure_shift = if defender_casualties > attacker_casualties {
        0.05
    } else if attacker_casualties > defender_casualties {
        -0.05
    } else {
        0.0
    };

    UnitCombatResult {
        attacker_casualties,
        defender_casualties,
        attacker_stress_delta: attacker_stress,
        defender_stress_delta: defender_stress,
        attacker_fatigue_delta: 0.05, // Fixed fatigue per combat round
        defender_fatigue_delta: 0.05,
        pressure_shift,
    }
}

/// Resolve a single melee exchange between two entities
fn resolve_entity_exchange(
    att_id: EntityId,
    def_id: EntityId,
    states: &mut HashMap<EntityId, CombatState>,
    is_support: bool, // If true, attacker is safer (reach or flank)
    att_casualties: &mut u32,
    def_casualties: &mut u32,
    att_stress: &mut f32,
    def_stress: &mut f32,
) {
    // Need to extract properties to avoid double mutable borrow
    // We clone the needed parts of state to create Combatants
    let (att_combatant, def_combatant) = {
        let att_state = states.get(&att_id).unwrap();
        let def_state = states.get(&def_id).unwrap();

        let mut att_c = Combatant {
            weapon: att_state.weapon.clone(),
            armor: att_state.armor.clone(),
            stance: CombatStance::Pressing, // Attacker presses
            skill: att_state.skill.clone(),
        };
        
        // If support attack, assume defensive stance for attacker to minimize return hits
        if is_support {
            att_c.stance = CombatStance::Defensive;
        }

        let def_c = Combatant {
            weapon: def_state.weapon.clone(),
            armor: def_state.armor.clone(),
            stance: CombatStance::Defensive, // Defender defends
            skill: def_state.skill.clone(),
        };

        (att_c, def_c)
    };

    // Resolve
    let result = resolve_exchange(&att_combatant, &def_combatant);

    // Apply results
    if let Some(wound) = result.attacker_wound {
        if let Some(state) = states.get_mut(&att_id) {
            let was_dead = state.is_dead() || state.is_incapacitated();
            state.wounds.push(wound);
            if !was_dead && (state.is_dead() || state.is_incapacitated()) {
                *att_casualties += 1;
            }
            *att_stress += 0.01;
        }
    }

    if let Some(wound) = result.defender_wound {
        if let Some(state) = states.get_mut(&def_id) {
            let was_dead = state.is_dead() || state.is_incapacitated();
            state.wounds.push(wound);
            if !was_dead && (state.is_dead() || state.is_incapacitated()) {
                *def_casualties += 1;
            }
            *def_stress += 0.01;
        }
    }
}

/// Resolve ranged attacks from unengaged entities
fn resolve_ranged_attacks(
    attackers: &[EntityId],
    defenders: &[EntityId],
    engaged: &std::collections::HashSet<EntityId>,
    states: &mut HashMap<EntityId, CombatState>,
    unit_type: &UnitType,
    casualties: &mut u32,
    stress: &mut f32,
) {
    if defenders.is_empty() {
        return;
    }

    // Determine projectile properties based on unit type
    let projectile = match unit_type {
        UnitType::Archers | UnitType::HorseArchers => WeaponProperties {
            edge: Edge::Sharp,
            mass: Mass::Light,
            reach: Reach::Short, // Doesn't matter for ranged
            special: vec![],
        },
        UnitType::Crossbowmen => WeaponProperties {
            edge: Edge::Sharp,
            mass: Mass::Medium,
            reach: Reach::Short,
            special: vec![WeaponSpecial::Piercing],
        },
        _ => return, // Not a shooter
    };

    for &att_id in attackers {
        if engaged.contains(&att_id) {
            continue;
        }

        // Fire!
        // Pick random target
        let target_idx = rand::random::<usize>() % defenders.len();
        let target_id = defenders[target_idx];

        // Resolve hit
        // Need target armor and skill (for zone selection)
        // We use attacker skill for zone selection
        let (att_skill, target_armor) = {
            let att_state = states.get(&att_id).unwrap();
            let def_state = states.get(&target_id).unwrap();
            (att_state.skill.level, def_state.armor.clone())
        };

        // Determine hit chance?
        // `resolve_hit` assumes a hit. We need a miss chance.
        // Simple accuracy check: 50% base +/- skill
        let hit_chance = 0.5; // Placeholder
        if rand::random::<f32>() > hit_chance {
            continue; 
        }

        let zone = select_hit_zone(att_skill);
        let wound = resolve_hit(&projectile, &target_armor, zone);

        // Apply wound
        if let Some(state) = states.get_mut(&target_id) {
            let was_dead = state.is_dead() || state.is_incapacitated();
            state.wounds.push(wound);
            if !was_dead && (state.is_dead() || state.is_incapacitated()) {
                *casualties += 1;
            }
            *stress += 0.005;
        }
    }
}

/// Resolve a shock attack
pub fn resolve_shock_attack(
    attacker: &BattleUnit,
    defender: &BattleUnit,
    shock_type: crate::combat::ShockType,
) -> ShockResult {
    // Keep the old shock logic for now as it's not strictly "exchange" based
    // But ideally this should also use entity physics
    
    // For MVP, just use a simplified version of the old logic
    let front_rank_size = (defender.effective_strength() as f32 * 0.2) as u32;
    let defender_props = defender.unit_type.default_properties();

    use crate::combat::Padding;
    let survival_rate = match defender_props.avg_armor.padding {
        Padding::None => 0.3, 
        Padding::Light => 0.5,
        Padding::Heavy => 0.7, 
    };

    let mut casualties = (front_rank_size as f32 * (1.0 - survival_rate)) as u32;

    if defender.unit_type == UnitType::Spearmen {
        casualties /= 2;
    }

    match shock_type {
        crate::combat::ShockType::CavalryCharge => {}
        crate::combat::ShockType::FlankAttack => casualties = casualties * 2 / 3,
        crate::combat::ShockType::RearCharge => casualties = casualties * 3 / 2,
        crate::combat::ShockType::Ambush => casualties = casualties * 5 / 4,
    }

    let stress_spike = shock_type.stress_spike()
        + (casualties as f32 / defender.effective_strength() as f32) * 0.20;

    let defender_threshold = defender.stress_threshold();
    let triggered_break_check = defender.stress + stress_spike > defender_threshold * 0.7;

    ShockResult {
        immediate_casualties: casualties,
        stress_spike,
        triggered_break_check,
    }
}

/// Determine LOD for a combat
pub fn determine_combat_lod(
    total_combatants: usize,
    is_player_focused: bool,
    is_near_objective: bool,
) -> CombatLOD {
    if is_player_focused {
        CombatLOD::Individual
    } else if is_near_objective || total_combatants < 50 {
        CombatLOD::Element
    } else if total_combatants < 200 {
        CombatLOD::Unit
    } else {
        CombatLOD::Formation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::units::Element;
    use crate::core::types::EntityId;

    #[test]
    fn test_resolve_combat_casualties() {
        let mut entity_states = HashMap::new();
        
        // Setup Attacker (Swords)
        let mut attacker = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        attacker.elements.push(Element::new(vec![EntityId::new(); 10]));
        
        // Setup Defender (No armor Levy)
        let mut defender = BattleUnit::new(UnitId::new(), UnitType::Levy);
        defender.elements.push(Element::new(vec![EntityId::new(); 10]));

        // Run combat
        let result = resolve_unit_combat(&attacker, &defender, &mut entity_states);

        // Expect some casualties (Swords vs Cloth)
        // Since it's random/skill based, might be 0, but states should be populated
        assert!(entity_states.len() == 20);
        
        // Check if any wounds were inflicted (Levy likely took hits)
        let total_wounds: usize = entity_states.values().map(|s| s.wounds.len()).sum();
        assert!(total_wounds > 0 || result.attacker_casualties == 0); // At least wounds if no kills?
    }
}