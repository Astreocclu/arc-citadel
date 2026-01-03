//! Battle system - tactical combat with fog of war and courier delays
//!
//! NOT Total War: terrain is dense, vision is scarce, control is delegated.
//!
//! Key differences from typical RTS:
//! - Dense terrain constrains movement (navigation is a puzzle)
//! - Vision is scarce (information must be gathered)
//! - Orders go through couriers (not instant)
//! - Same simulation at all accessibility levels

pub mod constants;
pub mod hex;
pub mod terrain;
pub mod battle_map;
pub mod unit_type;
pub mod units;
pub mod planning;
pub mod courier;
pub mod execution;
pub mod resolution;
pub mod pathfinding;
pub mod triggers;
pub mod visibility;
pub mod movement;
pub mod morale;
pub mod engagement;

// Re-exports for convenient access
pub use constants::*;
pub use hex::{BattleHexCoord, HexDirection};
pub use terrain::{BattleTerrain, TerrainFeature};
pub use battle_map::{BattleHex, BattleMap, VisibilityState, Objective};
pub use unit_type::{UnitType, UnitProperties};
pub use units::{
    ArmyId, FormationId, UnitId,
    Element, BattleUnit, BattleFormation, Army,
    UnitStance, FormationShape,
};
pub use planning::{
    GoCodeId, MovementPace, WaypointBehavior, WaitCondition,
    Waypoint, WaypointPlan, EngagementRule,
    GoCodeTrigger, GoCode, ContingencyTrigger, ContingencyResponse,
    Contingency, UnitDeployment, BattlePlan,
};
pub use courier::{
    CourierId, OrderType, OrderTarget, Order,
    CourierStatus, CourierInFlight, CourierSystem,
};
pub use execution::{
    BattlePhase, BattleOutcome, BattleEvent, BattleEventType,
    RoutingUnit, ActiveCombat, BattleState, check_battle_end,
};
pub use resolution::{
    CombatLOD, UnitCombatResult, ShockResult,
    calculate_casualty_rate, calculate_stress_delta,
    resolve_unit_combat, resolve_shock_attack, determine_combat_lod,
};
pub use pathfinding::{find_path, path_cost};
pub use triggers::{
    evaluate_all_contingencies, evaluate_all_gocodes, evaluate_contingency_trigger,
    evaluate_gocode_trigger, TriggerResults, UnitPosition,
};
pub use visibility::{
    ArmyVisibility, calculate_army_visibility, unit_vision_range, update_army_visibility,
};
pub use movement::{MovementResult, advance_unit_movement, move_routing_unit};
pub use morale::{
    MoraleCheckResult, check_morale_break, check_rally,
    calculate_contagion_stress, calculate_officer_death_stress,
    apply_stress, process_morale_break, process_rally,
};
pub use engagement::{
    PotentialEngagement, detect_engagement, should_initiate_combat,
    find_all_engagements, is_flanked, is_surrounded,
};
