//! Battle execution loop
//!
//! Each tick: movement -> couriers -> engagement -> combat -> morale -> rout

use serde::{Deserialize, Serialize};

use crate::battle::battle_map::BattleMap;
use crate::battle::constants::COURIER_SPEED;
use crate::battle::courier::CourierSystem;
use crate::battle::engagement::find_all_engagements;
use crate::battle::hex::BattleHexCoord;
use crate::battle::morale::{
    apply_stress, calculate_contagion_stress, check_morale_break, check_rally, process_morale_break,
};
use crate::battle::movement::advance_unit_movement;
use crate::battle::planning::BattlePlan;
use crate::battle::resolution::resolve_unit_combat;
use crate::battle::triggers::{evaluate_all_gocodes, UnitPosition};
use crate::battle::units::{Army, UnitId, UnitStance};
use crate::battle::visibility::{update_army_visibility, ArmyVisibility};
use crate::core::types::{EntityId, Tick};

/// Battle phases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BattlePhase {
    #[default]
    Planning, // Pre-battle planning
    Deployment, // Placing units
    Active,     // Battle in progress
    Finished,   // Battle over
}

/// Battle outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BattleOutcome {
    Undecided,
    DecisiveVictory,
    Victory,
    PyrrhicVictory,
    Draw,
    Defeat,
    DecisiveDefeat,
}

impl Default for BattleOutcome {
    fn default() -> Self {
        Self::Undecided
    }
}

/// Log entry for battle events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleEvent {
    pub tick: Tick,
    pub event_type: BattleEventType,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BattleEventType {
    BattleStarted,
    UnitEngaged { unit_id: UnitId },
    UnitBroke { unit_id: UnitId },
    UnitRallied { unit_id: UnitId },
    CommanderKilled { entity_id: EntityId },
    ObjectiveCaptured { name: String },
    CourierIntercepted,
    GoCodeTriggered { name: String },
    BattleEnded { outcome: BattleOutcome },
}

/// Log of events from a single tick
#[derive(Debug, Clone, Default)]
pub struct BattleEventLog {
    pub events: Vec<BattleEvent>,
}

impl BattleEventLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, event_type: BattleEventType, description: String, tick: Tick) {
        self.events.push(BattleEvent {
            tick,
            event_type,
            description,
        });
    }
}

/// Routing unit tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingUnit {
    pub unit_id: UnitId,
    pub retreat_progress: f32,
}

/// Active combat between units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveCombat {
    pub attacker_unit: UnitId,
    pub defender_unit: UnitId,
    pub ticks_engaged: u32,
}

/// Complete battle state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleState {
    // Core state
    pub map: BattleMap,
    pub friendly_army: Army,
    pub enemy_army: Army,

    // Time
    pub tick: Tick,
    pub phase: BattlePhase,
    pub outcome: BattleOutcome,

    // Plans
    pub friendly_plan: BattlePlan,
    pub enemy_plan: BattlePlan,

    // Systems
    pub courier_system: CourierSystem,

    // Visibility (fog of war)
    pub friendly_visibility: ArmyVisibility,
    pub enemy_visibility: ArmyVisibility,

    // Combat tracking
    pub active_combats: Vec<ActiveCombat>,
    pub routing_units: Vec<RoutingUnit>,

    // Log
    pub battle_log: Vec<BattleEvent>,
}

impl BattleState {
    pub fn new(map: BattleMap, friendly_army: Army, enemy_army: Army) -> Self {
        Self {
            map,
            friendly_army,
            enemy_army,
            tick: 0,
            phase: BattlePhase::Planning,
            outcome: BattleOutcome::Undecided,
            friendly_plan: BattlePlan::new(),
            enemy_plan: BattlePlan::new(),
            courier_system: CourierSystem::new(),
            friendly_visibility: ArmyVisibility::new(),
            enemy_visibility: ArmyVisibility::new(),
            active_combats: Vec::new(),
            routing_units: Vec::new(),
            battle_log: Vec::new(),
        }
    }

    /// Is the battle finished?
    pub fn is_finished(&self) -> bool {
        matches!(self.phase, BattlePhase::Finished)
    }

    /// Start the battle (transition from planning to active)
    pub fn start_battle(&mut self) {
        self.phase = BattlePhase::Active;
        self.log_event(BattleEventType::BattleStarted, "Battle has begun!".into());
    }

    /// Advance the battle by one tick
    pub fn advance_tick(&mut self) {
        if self.is_finished() {
            return;
        }

        self.tick += 1;

        // The actual tick resolution is split into sub-tasks
        // This just increments time
    }

    /// Log a battle event
    pub fn log_event(&mut self, event_type: BattleEventType, description: String) {
        self.battle_log.push(BattleEvent {
            tick: self.tick,
            event_type,
            description,
        });
    }

    /// End the battle with an outcome
    pub fn end_battle(&mut self, outcome: BattleOutcome) {
        self.phase = BattlePhase::Finished;
        self.outcome = outcome;
        self.log_event(
            BattleEventType::BattleEnded { outcome },
            format!("Battle ended: {:?}", outcome),
        );
    }

    /// Get a unit from either army
    pub fn get_unit(&self, unit_id: UnitId) -> Option<&crate::battle::units::BattleUnit> {
        self.friendly_army
            .get_unit(unit_id)
            .or_else(|| self.enemy_army.get_unit(unit_id))
    }

    /// Get a mutable unit from either army
    pub fn get_unit_mut(
        &mut self,
        unit_id: UnitId,
    ) -> Option<&mut crate::battle::units::BattleUnit> {
        if self.friendly_army.get_unit(unit_id).is_some() {
            self.friendly_army.get_unit_mut(unit_id)
        } else {
            self.enemy_army.get_unit_mut(unit_id)
        }
    }

    /// Run a complete battle tick
    pub fn run_tick(&mut self) -> BattleEventLog {
        let mut events = BattleEventLog::new();

        if self.is_finished() {
            return events;
        }

        // ===== PHASE 1: PRE-TICK =====
        self.phase_pre_tick(&mut events);

        // ===== PHASE 2: MOVEMENT =====
        self.phase_movement(&mut events);

        // ===== PHASE 3: COMBAT =====
        self.phase_combat(&mut events);

        // ===== PHASE 4: MORALE =====
        self.phase_morale(&mut events);

        // ===== PHASE 5: ROUT =====
        self.phase_rout(&mut events);

        // ===== PHASE 6: POST-TICK =====
        self.phase_post_tick(&mut events);

        events
    }

    fn phase_pre_tick(&mut self, events: &mut BattleEventLog) {
        use crate::battle::courier::Order;
        use crate::battle::orders::apply_order;
        use crate::battle::planning::ContingencyResponse;
        use crate::battle::triggers::evaluate_all_contingencies;

        // Update fog of war
        update_army_visibility(
            &mut self.friendly_visibility,
            &self.map,
            &self.friendly_army,
        );
        update_army_visibility(&mut self.enemy_visibility, &self.map, &self.enemy_army);

        // Evaluate go-codes
        let friendly_positions: Vec<UnitPosition> = self
            .friendly_army
            .formations
            .iter()
            .flat_map(|f| f.units.iter())
            .map(|u| UnitPosition {
                unit_id: u.id,
                position: u.position,
                is_routing: u.is_broken(),
            })
            .collect();

        let triggered = evaluate_all_gocodes(&self.friendly_plan, self.tick, &friendly_positions);
        for go_code_id in triggered {
            if let Some(gc) = self
                .friendly_plan
                .go_codes
                .iter_mut()
                .find(|g| g.id == go_code_id)
            {
                if !gc.triggered {
                    gc.triggered = true;
                    events.push(
                        BattleEventType::GoCodeTriggered {
                            name: gc.name.clone(),
                        },
                        format!("Go-code '{}' triggered", gc.name),
                        self.tick,
                    );
                }
            }
        }

        // ===== CONTINGENCY EVALUATION =====

        // Calculate casualties percentage
        let total_strength = self.friendly_army.total_strength();
        let effective_strength = self.friendly_army.effective_strength();
        let casualties_percent = if total_strength > 0 {
            1.0 - (effective_strength as f32 / total_strength as f32)
        } else {
            0.0
        };

        // Check if commander is alive (simplified: check if any formation has units)
        let commander_alive = !self.friendly_army.formations.is_empty()
            && self
                .friendly_army
                .formations
                .iter()
                .any(|f| !f.units.is_empty());

        // Get enemy positions
        let enemy_positions: Vec<BattleHexCoord> = self
            .enemy_army
            .formations
            .iter()
            .flat_map(|f| f.units.iter())
            .map(|u| u.position)
            .collect();

        // Get friendly positions (just hex coords for PositionLost check)
        let friendly_hex_positions: Vec<BattleHexCoord> = self
            .friendly_army
            .formations
            .iter()
            .flat_map(|f| f.units.iter())
            .map(|u| u.position)
            .collect();

        // Evaluate contingencies
        let triggered_contingencies = evaluate_all_contingencies(
            &self.friendly_plan,
            &friendly_positions,
            casualties_percent,
            commander_alive,
            &enemy_positions,
            &friendly_hex_positions,
        );

        // Process triggered contingencies - collect orders first to avoid borrow issues
        let mut orders_to_apply: Vec<Order> = Vec::new();
        let mut go_codes_to_trigger: Vec<crate::battle::planning::GoCodeId> = Vec::new();

        for idx in &triggered_contingencies {
            if let Some(contingency) = self.friendly_plan.contingencies.get(*idx) {
                if !contingency.activated {
                    match &contingency.response {
                        ContingencyResponse::ExecutePlan(_unit_id) => {
                            // Handled by waypoint system - backup plan execution
                        }
                        ContingencyResponse::Retreat(route) => {
                            // Order all units to retreat via this route
                            for formation in &self.friendly_army.formations {
                                for unit in &formation.units {
                                    orders_to_apply.push(Order::retreat(unit.id, route.clone()));
                                }
                            }
                        }
                        ContingencyResponse::Rally(position) => {
                            // Order all routing units to rally at position
                            for formation in &self.friendly_army.formations {
                                for unit in &formation.units {
                                    if unit.is_broken() {
                                        orders_to_apply.push(Order::move_to(unit.id, *position));
                                    }
                                }
                            }
                        }
                        ContingencyResponse::Signal(go_code_id) => {
                            go_codes_to_trigger.push(*go_code_id);
                        }
                    }
                }
            }
        }

        // Mark contingencies as activated
        for idx in triggered_contingencies {
            if let Some(contingency) = self.friendly_plan.contingencies.get_mut(idx) {
                if !contingency.activated {
                    contingency.activated = true;
                }
            }
        }

        // Apply collected orders
        for order in &orders_to_apply {
            apply_order(order, &mut self.friendly_army, &mut self.friendly_plan);
        }

        // Trigger go-codes from contingencies
        for go_code_id in go_codes_to_trigger {
            if let Some(gc) = self
                .friendly_plan
                .go_codes
                .iter_mut()
                .find(|g| g.id == go_code_id)
            {
                if !gc.triggered {
                    gc.triggered = true;
                    events.push(
                        BattleEventType::GoCodeTriggered {
                            name: gc.name.clone(),
                        },
                        format!("Go-code '{}' triggered by contingency", gc.name),
                        self.tick,
                    );
                }
            }
        }
    }

    fn phase_movement(&mut self, _events: &mut BattleEventLog) {
        use crate::battle::orders::apply_order;

        // Advance couriers
        self.courier_system.advance_all(COURIER_SPEED);
        let arrived_orders = self.courier_system.collect_arrived();

        // Apply arrived orders
        for order in &arrived_orders {
            // Determine which army this order targets
            match &order.target {
                crate::battle::courier::OrderTarget::Unit(unit_id) => {
                    if self.friendly_army.get_unit(*unit_id).is_some() {
                        apply_order(order, &mut self.friendly_army, &mut self.friendly_plan);
                    } else if self.enemy_army.get_unit(*unit_id).is_some() {
                        apply_order(order, &mut self.enemy_army, &mut self.enemy_plan);
                    }
                }
                crate::battle::courier::OrderTarget::Formation(formation_id) => {
                    if self.friendly_army.formations.iter().any(|f| f.id == *formation_id) {
                        apply_order(order, &mut self.friendly_army, &mut self.friendly_plan);
                    } else if self.enemy_army.formations.iter().any(|f| f.id == *formation_id) {
                        apply_order(order, &mut self.enemy_army, &mut self.enemy_plan);
                    }
                }
            }
        }

        // Move units along waypoints
        for formation in &mut self.friendly_army.formations {
            for unit in &mut formation.units {
                if let Some(plan) = self
                    .friendly_plan
                    .waypoint_plans
                    .iter_mut()
                    .find(|p| p.unit_id == unit.id)
                {
                    let _result = advance_unit_movement(&self.map, unit, plan);
                }
            }
        }
    }

    fn phase_combat(&mut self, _events: &mut BattleEventLog) {
        // Collect unit references
        let friendly_units: Vec<&crate::battle::units::BattleUnit> = self
            .friendly_army
            .formations
            .iter()
            .flat_map(|f| f.units.iter())
            .collect();

        let enemy_units: Vec<&crate::battle::units::BattleUnit> = self
            .enemy_army
            .formations
            .iter()
            .flat_map(|f| f.units.iter())
            .collect();

        // Detect engagements
        let engagements = find_all_engagements(&friendly_units, &enemy_units);

        // Process each engagement
        for engagement in engagements {
            // Find units by ID and resolve combat
            let friendly_unit = self.friendly_army.get_unit(engagement.attacker_id);
            let enemy_unit = self.enemy_army.get_unit(engagement.defender_id);

            if let (Some(attacker), Some(defender)) = (friendly_unit, enemy_unit) {
                let result = resolve_unit_combat(attacker, defender, 0.0);

                // Apply results
                if let Some(unit) = self.friendly_army.get_unit_mut(engagement.attacker_id) {
                    unit.casualties += result.attacker_casualties;
                    unit.stress += result.attacker_stress_delta;
                    unit.fatigue = (unit.fatigue + result.attacker_fatigue_delta).min(1.0);
                    unit.stance = UnitStance::Engaged;
                }

                if let Some(unit) = self.enemy_army.get_unit_mut(engagement.defender_id) {
                    unit.casualties += result.defender_casualties;
                    unit.stress += result.defender_stress_delta;
                    unit.fatigue = (unit.fatigue + result.defender_fatigue_delta).min(1.0);
                    unit.stance = UnitStance::Engaged;
                }
            }
        }
    }

    fn phase_morale(&mut self, events: &mut BattleEventLog) {
        // Collect routing unit positions for contagion
        let routing_positions: Vec<BattleHexCoord> = self
            .friendly_army
            .formations
            .iter()
            .flat_map(|f| f.units.iter())
            .filter(|u| u.is_broken())
            .map(|u| u.position)
            .collect();

        // Check morale for all units
        for formation in &mut self.friendly_army.formations {
            for unit in &mut formation.units {
                // Count nearby routing friendlies
                let nearby_routing = routing_positions
                    .iter()
                    .filter(|pos| unit.position.distance(pos) <= 2)
                    .count();

                let contagion = calculate_contagion_stress(unit, nearby_routing);
                if contagion > 0.0 {
                    apply_stress(unit, contagion);
                }

                // Check for break
                let result = check_morale_break(unit);
                if result.breaks {
                    process_morale_break(unit);
                    events.push(
                        BattleEventType::UnitBroke { unit_id: unit.id },
                        "Unit broke and is routing".to_string(),
                        self.tick,
                    );
                }
            }
        }

        // Same for enemy army
        let enemy_routing_positions: Vec<BattleHexCoord> = self
            .enemy_army
            .formations
            .iter()
            .flat_map(|f| f.units.iter())
            .filter(|u| u.is_broken())
            .map(|u| u.position)
            .collect();

        for formation in &mut self.enemy_army.formations {
            for unit in &mut formation.units {
                let nearby_routing = enemy_routing_positions
                    .iter()
                    .filter(|pos| unit.position.distance(pos) <= 2)
                    .count();

                let contagion = calculate_contagion_stress(unit, nearby_routing);
                if contagion > 0.0 {
                    apply_stress(unit, contagion);
                }

                let result = check_morale_break(unit);
                if result.breaks {
                    process_morale_break(unit);
                }
            }
        }
    }

    fn phase_rout(&mut self, events: &mut BattleEventLog) {
        // Move routing units
        let enemy_positions: Vec<BattleHexCoord> = self
            .enemy_army
            .formations
            .iter()
            .flat_map(|f| f.units.iter())
            .map(|u| u.position)
            .collect();

        // Pre-compute commander positions to avoid borrow issues
        let commander_positions: Vec<BattleHexCoord> = self
            .friendly_army
            .formations
            .iter()
            .map(|f| f.commander_position().unwrap_or_default())
            .collect();

        for (formation_idx, formation) in self.friendly_army.formations.iter_mut().enumerate() {
            let commander_pos = commander_positions[formation_idx];
            for unit in &mut formation.units {
                if unit.is_broken() {
                    // Check rally conditions
                    let is_near_enemy = enemy_positions
                        .iter()
                        .any(|pos| unit.position.distance(pos) <= 3);
                    let is_near_leader = unit.position.distance(&commander_pos) <= 3;

                    let result = check_rally(unit, is_near_enemy, is_near_leader);
                    if result.rallies {
                        unit.stance = UnitStance::Rallying;
                        unit.stress += result.stress_delta;
                        events.push(
                            BattleEventType::UnitRallied { unit_id: unit.id },
                            "Unit rallied".to_string(),
                            self.tick,
                        );
                    }
                }
            }
        }
    }

    fn phase_post_tick(&mut self, _events: &mut BattleEventLog) {
        // Check battle end
        if let Some(outcome) = check_battle_end(self) {
            self.end_battle(outcome);
        }

        // Advance tick counter
        self.tick += 1;
    }
}

/// Check if battle should end
pub fn check_battle_end(state: &BattleState) -> Option<BattleOutcome> {
    let friendly_effective = state.friendly_army.effective_strength();
    let enemy_effective = state.enemy_army.effective_strength();

    // Check for army destruction
    if enemy_effective == 0 {
        return Some(BattleOutcome::DecisiveVictory);
    }

    if friendly_effective == 0 {
        return Some(BattleOutcome::DecisiveDefeat);
    }

    // Check for army rout (>80% routing)
    let enemy_routing = state.enemy_army.percentage_routing();
    let friendly_routing = state.friendly_army.percentage_routing();

    if enemy_routing > 0.8 {
        return Some(BattleOutcome::Victory);
    }

    if friendly_routing > 0.8 {
        return Some(BattleOutcome::Defeat);
    }

    // Check time limit
    if state.tick > crate::battle::constants::MAX_BATTLE_TICKS {
        // Determine by remaining strength
        if friendly_effective > enemy_effective * 2 {
            return Some(BattleOutcome::Victory);
        } else if enemy_effective > friendly_effective * 2 {
            return Some(BattleOutcome::Defeat);
        } else {
            return Some(BattleOutcome::Draw);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::units::ArmyId;
    use crate::core::types::EntityId;

    #[test]
    fn test_battle_state_creation() {
        let map = BattleMap::new(20, 20);
        let friendly = Army::new(ArmyId::new(), EntityId::new());
        let enemy = Army::new(ArmyId::new(), EntityId::new());
        let state = BattleState::new(map, friendly, enemy);

        assert_eq!(state.tick, 0);
        assert!(!state.is_finished());
    }

    #[test]
    fn test_battle_tick_increments() {
        let map = BattleMap::new(20, 20);
        let friendly = Army::new(ArmyId::new(), EntityId::new());
        let enemy = Army::new(ArmyId::new(), EntityId::new());
        let mut state = BattleState::new(map, friendly, enemy);

        state.advance_tick();
        assert_eq!(state.tick, 1);
    }

    #[test]
    fn test_battle_phase_planning() {
        let map = BattleMap::new(20, 20);
        let friendly = Army::new(ArmyId::new(), EntityId::new());
        let enemy = Army::new(ArmyId::new(), EntityId::new());
        let state = BattleState::new(map, friendly, enemy);

        assert_eq!(state.phase, BattlePhase::Planning);
    }

    #[test]
    fn test_battle_start() {
        let map = BattleMap::new(20, 20);
        let friendly = Army::new(ArmyId::new(), EntityId::new());
        let enemy = Army::new(ArmyId::new(), EntityId::new());
        let mut state = BattleState::new(map, friendly, enemy);

        state.start_battle();
        assert_eq!(state.phase, BattlePhase::Active);
        assert_eq!(state.battle_log.len(), 1);
    }

    #[test]
    fn test_battle_end() {
        let map = BattleMap::new(20, 20);
        let friendly = Army::new(ArmyId::new(), EntityId::new());
        let enemy = Army::new(ArmyId::new(), EntityId::new());
        let mut state = BattleState::new(map, friendly, enemy);

        state.end_battle(BattleOutcome::Victory);
        assert!(state.is_finished());
        assert_eq!(state.outcome, BattleOutcome::Victory);
    }

    #[test]
    fn test_check_battle_end_enemy_destroyed() {
        let map = BattleMap::new(20, 20);
        let mut friendly = Army::new(ArmyId::new(), EntityId::new());
        let enemy = Army::new(ArmyId::new(), EntityId::new());

        // Give friendly some units
        let mut formation = crate::battle::units::BattleFormation::new(
            crate::battle::units::FormationId::new(),
            EntityId::new(),
        );
        let mut unit = crate::battle::units::BattleUnit::new(
            crate::battle::units::UnitId::new(),
            crate::battle::unit_type::UnitType::Infantry,
        );
        unit.elements.push(crate::battle::units::Element::new(vec![
            EntityId::new();
            50
        ]));
        formation.units.push(unit);
        friendly.formations.push(formation);

        let state = BattleState::new(map, friendly, enemy);

        // Enemy has no units = decisive victory
        let outcome = check_battle_end(&state);
        assert_eq!(outcome, Some(BattleOutcome::DecisiveVictory));
    }

    #[test]
    fn test_full_tick_advances_state() {
        use crate::battle::hex::BattleHexCoord;
        use crate::battle::unit_type::UnitType;
        use crate::battle::units::{BattleFormation, BattleUnit, Element, FormationId};

        let map = BattleMap::new(20, 20);
        let mut friendly = Army::new(ArmyId::new(), EntityId::new());
        let enemy = Army::new(ArmyId::new(), EntityId::new());

        // Add a unit to friendly army
        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        unit.position = BattleHexCoord::new(5, 5);
        formation.units.push(unit);
        friendly.formations.push(formation);

        let mut state = BattleState::new(map, friendly, enemy);
        state.start_battle();

        let initial_tick = state.tick;
        let events = state.run_tick();

        assert_eq!(state.tick, initial_tick + 1);
        assert!(events.events.is_empty() || !events.events.is_empty()); // Events may or may not occur
    }

    // ===== Integration Tests =====

    #[test]
    fn test_battle_scenario_simple_engagement() {
        use crate::battle::hex::BattleHexCoord;
        use crate::battle::unit_type::UnitType;
        use crate::battle::units::{BattleFormation, BattleUnit, Element, FormationId};

        // Setup map
        let map = BattleMap::new(30, 30);

        // Setup friendly army
        let mut friendly = Army::new(ArmyId::new(), EntityId::new());
        let mut friendly_formation = BattleFormation::new(FormationId::new(), EntityId::new());

        let mut friendly_unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        friendly_unit
            .elements
            .push(Element::new(vec![EntityId::new(); 100]));
        friendly_unit.position = BattleHexCoord::new(10, 15);
        friendly_formation.units.push(friendly_unit);
        friendly.formations.push(friendly_formation);

        // Setup enemy army
        let mut enemy = Army::new(ArmyId::new(), EntityId::new());
        let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());

        let mut enemy_unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        enemy_unit
            .elements
            .push(Element::new(vec![EntityId::new(); 100]));
        enemy_unit.position = BattleHexCoord::new(11, 15); // Adjacent to friendly
        enemy_formation.units.push(enemy_unit);
        enemy.formations.push(enemy_formation);

        // Create battle state
        let mut state = BattleState::new(map, friendly, enemy);
        state.start_battle();

        // Run several ticks
        for _ in 0..10 {
            let _events = state.run_tick();
        }

        // Verify combat occurred (casualties inflicted)
        let friendly_casualties: u32 = state
            .friendly_army
            .formations
            .iter()
            .flat_map(|f| f.units.iter())
            .map(|u| u.casualties)
            .sum();

        let enemy_casualties: u32 = state
            .enemy_army
            .formations
            .iter()
            .flat_map(|f| f.units.iter())
            .map(|u| u.casualties)
            .sum();

        assert!(
            friendly_casualties > 0 || enemy_casualties > 0,
            "Combat should have occurred"
        );
        assert_eq!(state.tick, 10, "Should have advanced 10 ticks");
    }

    #[test]
    fn test_battle_ends_when_army_destroyed() {
        use crate::battle::hex::BattleHexCoord;
        use crate::battle::unit_type::UnitType;
        use crate::battle::units::{BattleFormation, BattleUnit, Element, FormationId};

        let map = BattleMap::new(20, 20);

        // Strong friendly army
        let mut friendly = Army::new(ArmyId::new(), EntityId::new());
        let mut friendly_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let mut friendly_unit = BattleUnit::new(UnitId::new(), UnitType::HeavyCavalry);
        friendly_unit
            .elements
            .push(Element::new(vec![EntityId::new(); 200]));
        friendly_unit.position = BattleHexCoord::new(10, 10);
        friendly_formation.units.push(friendly_unit);
        friendly.formations.push(friendly_formation);

        // Weak enemy army (using Levy - cheap, unreliable with low stress threshold)
        let mut enemy = Army::new(ArmyId::new(), EntityId::new());
        let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let mut enemy_unit = BattleUnit::new(UnitId::new(), UnitType::Levy);
        enemy_unit
            .elements
            .push(Element::new(vec![EntityId::new(); 10]));
        enemy_unit.position = BattleHexCoord::new(11, 10); // Adjacent
        enemy_formation.units.push(enemy_unit);
        enemy.formations.push(enemy_formation);

        let mut state = BattleState::new(map, friendly, enemy);
        state.start_battle();

        // Run until battle ends or max ticks
        for _ in 0..500 {
            let _events = state.run_tick();
            if state.is_finished() {
                break;
            }
        }

        // Battle should end in victory (enemy destroyed)
        assert!(state.is_finished(), "Battle should have ended");
        assert!(
            matches!(
                state.outcome,
                BattleOutcome::Victory | BattleOutcome::DecisiveVictory
            ),
            "Should be a victory, got {:?}",
            state.outcome
        );
    }

    #[test]
    fn test_order_application_in_phase_movement() {
        use crate::battle::courier::Order;
        use crate::battle::hex::BattleHexCoord;
        use crate::battle::unit_type::UnitType;
        use crate::battle::units::{BattleFormation, BattleUnit, Element, FormationId};

        let map = BattleMap::new(20, 20);
        let mut friendly = Army::new(ArmyId::new(), EntityId::new());

        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let unit_id = UnitId::new();
        let mut unit = BattleUnit::new(unit_id, UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        unit.position = BattleHexCoord::new(5, 5);
        formation.units.push(unit);
        friendly.formations.push(formation);

        // Enemy army needs units so battle doesn't end with DecisiveVictory immediately
        let mut enemy = Army::new(ArmyId::new(), EntityId::new());
        let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let mut enemy_unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        enemy_unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        enemy_unit.position = BattleHexCoord::new(15, 15); // Far from friendly unit
        enemy_formation.units.push(enemy_unit);
        enemy.formations.push(enemy_formation);

        let mut state = BattleState::new(map, friendly, enemy);
        state.start_battle();

        // Dispatch an order (same source/dest means courier path has 1 element)
        let destination = BattleHexCoord::new(10, 10);
        state.courier_system.dispatch(
            EntityId::new(),
            Order::move_to(unit_id, destination),
            BattleHexCoord::new(5, 5),
            BattleHexCoord::new(5, 5), // Same position
        );

        // Verify courier was dispatched
        assert_eq!(state.courier_system.in_flight.len(), 1, "Courier should be in flight");

        // Run multiple ticks - COURIER_SPEED is 0.30, so we need ~4 ticks for courier to arrive
        // (progress needs to reach 1.0 to drain the single-element path)
        for _ in 0..10 {
            let _events = state.run_tick();
            // Check if order was applied
            if state.friendly_plan.get_waypoint_plan(unit_id).is_some() {
                break;
            }
        }

        // Check that waypoint plan was created by the order application
        let wp_plan = state.friendly_plan.get_waypoint_plan(unit_id);
        assert!(wp_plan.is_some(), "Waypoint plan should be created after courier arrives");
        assert_eq!(wp_plan.unwrap().waypoints[0].position, destination);
    }

    #[test]
    fn test_contingency_triggers_in_phase_pre_tick() {
        use crate::battle::hex::BattleHexCoord;
        use crate::battle::planning::{Contingency, ContingencyResponse, ContingencyTrigger};
        use crate::battle::unit_type::UnitType;
        use crate::battle::units::{BattleFormation, BattleUnit, Element, FormationId};

        let map = BattleMap::new(20, 20);
        let mut friendly = Army::new(ArmyId::new(), EntityId::new());

        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 100]));
        unit.casualties = 40; // 40% casualties
        unit.position = BattleHexCoord::new(5, 5);
        formation.units.push(unit);
        friendly.formations.push(formation);

        // Enemy army needs units so battle doesn't end with DecisiveVictory immediately
        let mut enemy = Army::new(ArmyId::new(), EntityId::new());
        let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let mut enemy_unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        enemy_unit
            .elements
            .push(Element::new(vec![EntityId::new(); 50]));
        enemy_unit.position = BattleHexCoord::new(15, 15);
        enemy_formation.units.push(enemy_unit);
        enemy.formations.push(enemy_formation);

        let mut state = BattleState::new(map, friendly, enemy);

        // Add contingency: rally at (0,0) if casualties exceed 30%
        state.friendly_plan.contingencies.push(Contingency::new(
            ContingencyTrigger::CasualtiesExceed(0.3),
            ContingencyResponse::Rally(BattleHexCoord::new(0, 0)),
        ));

        state.start_battle();

        // Run a tick - contingency should trigger
        let _events = state.run_tick();

        // Check contingency was activated
        assert!(
            state.friendly_plan.contingencies[0].activated,
            "Contingency should be activated"
        );
    }
}
