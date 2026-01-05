//! Battle system - tactical combat with fog of war and courier delays
//!
//! NOT Total War: terrain is dense, vision is scarce, control is delegated.
//!
//! Key differences from typical RTS:
//! - Dense terrain constrains movement (navigation is a puzzle)
//! - Vision is scarce (information must be gathered)
//! - Orders go through couriers (not instant)
//! - Same simulation at all accessibility levels

pub mod battle_map;
pub mod constants;
pub mod courier;
pub mod engagement;
pub mod execution;
pub mod hex;
pub mod morale;
pub mod movement;
pub mod orders;
pub mod pathfinding;
pub mod planning;
pub mod resolution;
pub mod terrain;
pub mod triggers;
pub mod unit_type;
pub mod units;
pub mod visibility;

// Re-exports for convenient access
pub use battle_map::{BattleHex, BattleMap, Objective, VisibilityState};
pub use constants::*;
pub use courier::{
    CourierId, CourierInFlight, CourierStatus, CourierSystem, Order, OrderTarget, OrderType,
};
pub use engagement::{
    detect_engagement, find_all_engagements, is_flanked, is_surrounded, should_initiate_combat,
    PotentialEngagement,
};
pub use execution::{
    check_battle_end, ActiveCombat, BattleEvent, BattleEventLog, BattleEventType, BattleOutcome,
    BattlePhase, BattleState, RoutingUnit,
};
pub use hex::{BattleHexCoord, HexDirection};
pub use morale::{
    apply_stress, calculate_contagion_stress, calculate_officer_death_stress, check_morale_break,
    check_rally, process_morale_break, process_rally, MoraleCheckResult,
};
pub use movement::{advance_unit_movement, move_routing_unit, MovementResult};
pub use orders::{apply_order, ApplyOrderResult};
pub use pathfinding::{find_path, path_cost};
pub use planning::{
    BattlePlan, Contingency, ContingencyResponse, ContingencyTrigger, EngagementRule, GoCode,
    GoCodeId, GoCodeTrigger, MovementPace, UnitDeployment, WaitCondition, Waypoint,
    WaypointBehavior, WaypointPlan,
};
pub use resolution::{
    calculate_casualty_rate, calculate_stress_delta, determine_combat_lod, resolve_shock_attack,
    resolve_unit_combat, CombatLOD, ShockResult, UnitCombatResult,
};
pub use terrain::{BattleTerrain, TerrainFeature};
pub use triggers::{
    evaluate_all_contingencies, evaluate_all_gocodes, evaluate_contingency_trigger,
    evaluate_gocode_trigger, TriggerResults, UnitPosition,
};
pub use unit_type::{UnitProperties, UnitType};
pub use units::{
    Army, ArmyId, BattleFormation, BattleUnit, Element, FormationId, FormationShape, UnitId,
    UnitStance,
};
pub use visibility::{
    calculate_army_visibility, unit_vision_range, update_army_visibility, ArmyVisibility,
};
