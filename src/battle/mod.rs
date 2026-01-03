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
