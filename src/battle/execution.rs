//! Battle execution loop
//!
//! Each tick: movement -> couriers -> engagement -> combat -> morale -> rout

use serde::{Deserialize, Serialize};

use crate::battle::battle_map::BattleMap;
use crate::battle::units::{Army, UnitId};
use crate::battle::planning::BattlePlan;
use crate::battle::courier::CourierSystem;
use crate::core::types::{EntityId, Tick};

/// Battle phases
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BattlePhase {
    #[default]
    Planning,   // Pre-battle planning
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
    pub fn get_unit_mut(&mut self, unit_id: UnitId) -> Option<&mut crate::battle::units::BattleUnit> {
        if self.friendly_army.get_unit(unit_id).is_some() {
            self.friendly_army.get_unit_mut(unit_id)
        } else {
            self.enemy_army.get_unit_mut(unit_id)
        }
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
        unit.elements.push(crate::battle::units::Element::new(vec![EntityId::new(); 50]));
        formation.units.push(unit);
        friendly.formations.push(formation);

        let state = BattleState::new(map, friendly, enemy);

        // Enemy has no units = decisive victory
        let outcome = check_battle_end(&state);
        assert_eq!(outcome, Some(BattleOutcome::DecisiveVictory));
    }
}
