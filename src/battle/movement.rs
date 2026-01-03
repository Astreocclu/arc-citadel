//! Unit movement along waypoints
//!
//! Units follow their waypoint plans, respecting terrain and pace.

use crate::battle::battle_map::BattleMap;
use crate::battle::constants::{
    CAVALRY_CHARGE_SPEED, CAVALRY_TROT_SPEED, CAVALRY_WALK_SPEED, FATIGUE_RATE_MARCH,
    INFANTRY_RUN_SPEED, INFANTRY_WALK_SPEED,
};
use crate::battle::hex::BattleHexCoord;
use crate::battle::pathfinding::find_path;
use crate::battle::planning::{MovementPace, WaitCondition, WaypointBehavior, WaypointPlan};
use crate::battle::unit_type::UnitType;
use crate::battle::units::{BattleUnit, UnitStance};

/// Result of a movement tick
#[derive(Debug, Clone)]
pub struct MovementResult {
    pub moved: bool,
    pub reached_waypoint: bool,
    pub fatigue_delta: f32,
    pub path_blocked: bool,
}

impl Default for MovementResult {
    fn default() -> Self {
        Self {
            moved: false,
            reached_waypoint: false,
            fatigue_delta: 0.0,
            path_blocked: false,
        }
    }
}

/// Get base speed for a unit type and pace
fn base_speed(unit_type: UnitType, pace: MovementPace) -> f32 {
    let is_cavalry = matches!(
        unit_type,
        UnitType::LightCavalry | UnitType::HeavyCavalry
    );

    if is_cavalry {
        match pace {
            MovementPace::Walk => CAVALRY_WALK_SPEED,
            MovementPace::Quick => CAVALRY_TROT_SPEED,
            MovementPace::Run => CAVALRY_TROT_SPEED * 1.5,
            MovementPace::Charge => CAVALRY_CHARGE_SPEED,
        }
    } else {
        match pace {
            MovementPace::Walk => INFANTRY_WALK_SPEED,
            MovementPace::Quick => INFANTRY_WALK_SPEED * 1.5,
            MovementPace::Run => INFANTRY_RUN_SPEED,
            MovementPace::Charge => INFANTRY_RUN_SPEED * 1.5,
        }
    }
}

/// Check if a unit is blocked by wait condition
pub fn is_waiting(plan: &WaypointPlan, _current_tick: u64) -> bool {
    let Some(waypoint) = plan.current() else {
        return false;
    };

    match &waypoint.wait_condition {
        None => false,
        Some(WaitCondition::Duration(_)) => {
            // TODO: Track wait start time
            false
        }
        Some(WaitCondition::GoCode(_)) => {
            // Evaluated separately by triggers system
            true
        }
        Some(WaitCondition::UnitArrives(_)) => {
            // TODO: Check unit position
            true
        }
        Some(WaitCondition::EnemySighted) => {
            // TODO: Check visibility
            true
        }
        Some(WaitCondition::Attacked) => {
            // TODO: Check if under attack
            true
        }
    }
}

/// Advance a unit's movement by one tick
pub fn advance_unit_movement(
    map: &BattleMap,
    unit: &mut BattleUnit,
    plan: &mut WaypointPlan,
) -> MovementResult {
    let mut result = MovementResult::default();

    // Can't move if not in moving stance
    if !matches!(unit.stance, UnitStance::Moving | UnitStance::Formed) {
        return result;
    }

    // Get current waypoint
    let Some(waypoint) = plan.current() else {
        return result;
    };

    // Check if already at waypoint
    if unit.position == waypoint.position {
        result.reached_waypoint = true;

        // Apply waypoint behavior
        match waypoint.behavior {
            WaypointBehavior::MoveTo => {
                plan.advance();
            }
            WaypointBehavior::HoldAt => {
                unit.stance = UnitStance::Formed;
            }
            WaypointBehavior::AttackFrom => {
                unit.stance = UnitStance::Alert;
            }
            WaypointBehavior::ScanFrom => {
                unit.stance = UnitStance::Patrol;
            }
            WaypointBehavior::RallyAt => {
                unit.stance = UnitStance::Formed;
            }
        }

        return result;
    }

    // Calculate movement
    let is_cavalry = matches!(
        unit.unit_type,
        UnitType::LightCavalry | UnitType::HeavyCavalry
    );

    // Find path to waypoint
    let Some(path) = find_path(map, unit.position, waypoint.position, is_cavalry) else {
        result.path_blocked = true;
        return result;
    };

    // Get speed
    let speed = base_speed(unit.unit_type, waypoint.pace);
    let fatigue_modifier = 1.0 - (unit.fatigue * 0.3); // Fatigue slows movement
    let effective_speed = speed * fatigue_modifier * waypoint.pace.speed_multiplier();

    // Move along path (speed is fraction of hex per tick)
    if path.len() > 1 {
        // Simple: move to next hex if speed >= threshold, otherwise accumulate
        // For now, just move one hex if speed allows
        if effective_speed >= 0.05 {
            unit.position = path[1];
            unit.stance = UnitStance::Moving;
            result.moved = true;
            result.fatigue_delta = FATIGUE_RATE_MARCH * waypoint.pace.fatigue_multiplier();
        }
    }

    result
}

/// Move all routing units away from enemies
pub fn move_routing_unit(
    map: &BattleMap,
    unit: &mut BattleUnit,
    retreat_direction: BattleHexCoord,
) -> bool {
    if !matches!(unit.stance, UnitStance::Routing) {
        return false;
    }

    let is_cavalry = matches!(
        unit.unit_type,
        UnitType::LightCavalry | UnitType::HeavyCavalry
    );

    // Find path toward retreat direction
    if let Some(path) = find_path(map, unit.position, retreat_direction, is_cavalry) {
        if path.len() > 1 {
            unit.position = path[1];
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::battle_map::BattleMap;
    use crate::battle::hex::BattleHexCoord;
    use crate::battle::planning::{MovementPace, Waypoint, WaypointBehavior, WaypointPlan};
    use crate::battle::unit_type::UnitType;
    use crate::battle::units::{BattleUnit, UnitId, UnitStance};

    #[test]
    fn test_unit_moves_toward_waypoint() {
        let map = BattleMap::new(20, 20);
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(0, 0);
        unit.stance = UnitStance::Moving;

        let mut plan = WaypointPlan::new(unit.id);
        plan.add_waypoint(
            Waypoint::new(BattleHexCoord::new(5, 0), WaypointBehavior::MoveTo)
                .with_pace(MovementPace::Quick),
        );

        let result = advance_unit_movement(&map, &mut unit, &mut plan);

        assert!(result.moved);
        // Unit should have moved closer to waypoint
        assert!(unit.position.distance(&BattleHexCoord::new(5, 0)) < 5);
    }

    #[test]
    fn test_unit_stops_at_hold_waypoint() {
        let map = BattleMap::new(20, 20);
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(5, 0);
        unit.stance = UnitStance::Moving;

        let mut plan = WaypointPlan::new(unit.id);
        plan.add_waypoint(Waypoint::new(
            BattleHexCoord::new(5, 0),
            WaypointBehavior::HoldAt,
        ));

        let result = advance_unit_movement(&map, &mut unit, &mut plan);

        assert!(result.reached_waypoint);
        assert_eq!(unit.stance, UnitStance::Formed);
    }

    #[test]
    fn test_cavalry_faster_than_infantry() {
        let infantry_speed = base_speed(UnitType::Infantry, MovementPace::Quick);
        let cavalry_speed = base_speed(UnitType::HeavyCavalry, MovementPace::Quick);

        assert!(cavalry_speed > infantry_speed);
    }

    #[test]
    fn test_charge_faster_than_walk() {
        let walk = base_speed(UnitType::Infantry, MovementPace::Walk);
        let charge = base_speed(UnitType::Infantry, MovementPace::Charge);

        assert!(charge > walk);
    }

    #[test]
    fn test_unit_cannot_move_when_routing() {
        let map = BattleMap::new(20, 20);
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(0, 0);
        unit.stance = UnitStance::Routing;

        let mut plan = WaypointPlan::new(unit.id);
        plan.add_waypoint(
            Waypoint::new(BattleHexCoord::new(5, 0), WaypointBehavior::MoveTo)
                .with_pace(MovementPace::Quick),
        );

        let result = advance_unit_movement(&map, &mut unit, &mut plan);

        assert!(!result.moved);
    }

    #[test]
    fn test_unit_cannot_move_when_engaged() {
        let map = BattleMap::new(20, 20);
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(0, 0);
        unit.stance = UnitStance::Engaged;

        let mut plan = WaypointPlan::new(unit.id);
        plan.add_waypoint(
            Waypoint::new(BattleHexCoord::new(5, 0), WaypointBehavior::MoveTo)
                .with_pace(MovementPace::Quick),
        );

        let result = advance_unit_movement(&map, &mut unit, &mut plan);

        assert!(!result.moved);
    }

    #[test]
    fn test_routing_unit_moves_toward_retreat() {
        let map = BattleMap::new(20, 20);
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(10, 10);
        unit.stance = UnitStance::Routing;

        let initial_position = unit.position;
        let retreat_point = BattleHexCoord::new(0, 0);

        let moved = move_routing_unit(&map, &mut unit, retreat_point);

        assert!(moved);
        assert_ne!(unit.position, initial_position);
        // Should be closer to retreat point
        assert!(unit.position.distance(&retreat_point) < initial_position.distance(&retreat_point));
    }

    #[test]
    fn test_non_routing_unit_doesnt_retreat() {
        let map = BattleMap::new(20, 20);
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(10, 10);
        unit.stance = UnitStance::Formed;

        let retreat_point = BattleHexCoord::new(0, 0);

        let moved = move_routing_unit(&map, &mut unit, retreat_point);

        assert!(!moved);
    }

    #[test]
    fn test_attack_from_waypoint_sets_alert() {
        let map = BattleMap::new(20, 20);
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(5, 5);
        unit.stance = UnitStance::Moving;

        let mut plan = WaypointPlan::new(unit.id);
        plan.add_waypoint(Waypoint::new(
            BattleHexCoord::new(5, 5),
            WaypointBehavior::AttackFrom,
        ));

        let result = advance_unit_movement(&map, &mut unit, &mut plan);

        assert!(result.reached_waypoint);
        assert_eq!(unit.stance, UnitStance::Alert);
    }

    #[test]
    fn test_scan_from_waypoint_sets_patrol() {
        let map = BattleMap::new(20, 20);
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(5, 5);
        unit.stance = UnitStance::Moving;

        let mut plan = WaypointPlan::new(unit.id);
        plan.add_waypoint(Waypoint::new(
            BattleHexCoord::new(5, 5),
            WaypointBehavior::ScanFrom,
        ));

        let result = advance_unit_movement(&map, &mut unit, &mut plan);

        assert!(result.reached_waypoint);
        assert_eq!(unit.stance, UnitStance::Patrol);
    }

    #[test]
    fn test_move_to_advances_to_next_waypoint() {
        let map = BattleMap::new(20, 20);
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(0, 0);
        unit.stance = UnitStance::Moving;

        let mut plan = WaypointPlan::new(unit.id);
        plan.add_waypoint(Waypoint::new(
            BattleHexCoord::new(0, 0),
            WaypointBehavior::MoveTo,
        ));
        plan.add_waypoint(Waypoint::new(
            BattleHexCoord::new(5, 5),
            WaypointBehavior::HoldAt,
        ));

        assert_eq!(plan.current_waypoint, 0);

        let result = advance_unit_movement(&map, &mut unit, &mut plan);

        assert!(result.reached_waypoint);
        assert_eq!(plan.current_waypoint, 1);
    }

    #[test]
    fn test_fatigue_increases_with_movement() {
        let map = BattleMap::new(20, 20);
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(0, 0);
        unit.stance = UnitStance::Moving;

        let mut plan = WaypointPlan::new(unit.id);
        plan.add_waypoint(
            Waypoint::new(BattleHexCoord::new(5, 0), WaypointBehavior::MoveTo)
                .with_pace(MovementPace::Run),
        );

        let result = advance_unit_movement(&map, &mut unit, &mut plan);

        assert!(result.moved);
        assert!(result.fatigue_delta > 0.0);
    }

    #[test]
    fn test_charge_causes_more_fatigue_than_walk() {
        let map = BattleMap::new(20, 20);

        let mut unit_walk = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit_walk.position = BattleHexCoord::new(0, 0);
        unit_walk.stance = UnitStance::Moving;

        let mut plan_walk = WaypointPlan::new(unit_walk.id);
        plan_walk.add_waypoint(
            Waypoint::new(BattleHexCoord::new(5, 0), WaypointBehavior::MoveTo)
                .with_pace(MovementPace::Walk),
        );

        let mut unit_charge = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit_charge.position = BattleHexCoord::new(0, 0);
        unit_charge.stance = UnitStance::Moving;

        let mut plan_charge = WaypointPlan::new(unit_charge.id);
        plan_charge.add_waypoint(
            Waypoint::new(BattleHexCoord::new(5, 0), WaypointBehavior::MoveTo)
                .with_pace(MovementPace::Charge),
        );

        let result_walk = advance_unit_movement(&map, &mut unit_walk, &mut plan_walk);
        let result_charge = advance_unit_movement(&map, &mut unit_charge, &mut plan_charge);

        assert!(result_charge.fatigue_delta > result_walk.fatigue_delta);
    }

    #[test]
    fn test_all_movement_paces_for_infantry() {
        let walk = base_speed(UnitType::Infantry, MovementPace::Walk);
        let quick = base_speed(UnitType::Infantry, MovementPace::Quick);
        let run = base_speed(UnitType::Infantry, MovementPace::Run);
        let charge = base_speed(UnitType::Infantry, MovementPace::Charge);

        // Each pace should be faster than the previous
        assert!(quick > walk);
        assert!(run > quick);
        assert!(charge > run);
    }

    #[test]
    fn test_all_movement_paces_for_cavalry() {
        let walk = base_speed(UnitType::HeavyCavalry, MovementPace::Walk);
        let quick = base_speed(UnitType::HeavyCavalry, MovementPace::Quick);
        let run = base_speed(UnitType::HeavyCavalry, MovementPace::Run);
        let charge = base_speed(UnitType::HeavyCavalry, MovementPace::Charge);

        // Each pace should be faster than the previous
        assert!(quick > walk);
        assert!(run > quick);
        assert!(charge > run);
    }

    #[test]
    fn test_light_cavalry_same_speed_as_heavy() {
        let light = base_speed(UnitType::LightCavalry, MovementPace::Charge);
        let heavy = base_speed(UnitType::HeavyCavalry, MovementPace::Charge);

        // In this implementation they're the same, but test documents the behavior
        assert_eq!(light, heavy);
    }
}
