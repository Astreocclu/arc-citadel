//! Order application logic
//!
//! Translates courier-delivered orders into waypoint plan modifications.

use crate::battle::courier::{Order, OrderTarget, OrderType};
use crate::battle::hex::BattleHexCoord;
use crate::battle::planning::{
    BattlePlan, EngagementRule, MovementPace, Waypoint, WaypointBehavior, WaypointPlan,
};
use crate::battle::units::{Army, FormationShape, UnitId, UnitStance};

/// Result of applying an order
#[derive(Debug, Clone)]
pub struct ApplyOrderResult {
    pub success: bool,
    pub affected_units: Vec<UnitId>,
    pub message: String,
}

/// Apply an order to the target unit(s)
pub fn apply_order(order: &Order, army: &mut Army, plan: &mut BattlePlan) -> ApplyOrderResult {
    match &order.target {
        OrderTarget::Unit(unit_id) => apply_order_to_unit(order, *unit_id, army, plan),
        OrderTarget::Formation(formation_id) => {
            // Apply to all units in formation
            let unit_ids: Vec<UnitId> = army
                .formations
                .iter()
                .find(|f| f.id == *formation_id)
                .map(|f| f.units.iter().map(|u| u.id).collect())
                .unwrap_or_default();

            let mut affected = Vec::new();
            for unit_id in unit_ids {
                let result = apply_order_to_unit(order, unit_id, army, plan);
                if result.success {
                    affected.extend(result.affected_units);
                }
            }

            ApplyOrderResult {
                success: !affected.is_empty(),
                affected_units: affected,
                message: format!("Order applied to formation {:?}", formation_id),
            }
        }
    }
}

fn apply_order_to_unit(
    order: &Order,
    unit_id: UnitId,
    army: &mut Army,
    plan: &mut BattlePlan,
) -> ApplyOrderResult {
    match &order.order_type {
        OrderType::MoveTo(destination) => {
            // Clear existing waypoints, add new destination
            let waypoint_plan = get_or_create_waypoint_plan(plan, unit_id);
            waypoint_plan.waypoints.clear();
            waypoint_plan.current_waypoint = 0;
            waypoint_plan.wait_start_tick = None;
            waypoint_plan.add_waypoint(
                Waypoint::new(*destination, WaypointBehavior::MoveTo)
                    .with_pace(MovementPace::Quick),
            );

            // Set unit to moving stance
            if let Some(unit) = army.get_unit_mut(unit_id) {
                unit.stance = UnitStance::Moving;
            }

            ApplyOrderResult {
                success: true,
                affected_units: vec![unit_id],
                message: format!("MoveTo {:?}", destination),
            }
        }

        OrderType::Attack(target_id) => {
            // Get target position, create attack waypoint
            let target_pos = army
                .get_unit(*target_id)
                .map(|u| u.position)
                .unwrap_or_default();

            let waypoint_plan = get_or_create_waypoint_plan(plan, unit_id);
            waypoint_plan.waypoints.clear();
            waypoint_plan.current_waypoint = 0;
            waypoint_plan.wait_start_tick = None;
            waypoint_plan.add_waypoint(
                Waypoint::new(target_pos, WaypointBehavior::AttackFrom)
                    .with_pace(MovementPace::Run),
            );

            // Set engagement rule to aggressive
            plan.engagement_rules.retain(|(id, _)| *id != unit_id);
            plan.engagement_rules
                .push((unit_id, EngagementRule::Aggressive));

            ApplyOrderResult {
                success: true,
                affected_units: vec![unit_id],
                message: format!("Attack {:?}", target_id),
            }
        }

        OrderType::Defend(position) => {
            let waypoint_plan = get_or_create_waypoint_plan(plan, unit_id);
            waypoint_plan.waypoints.clear();
            waypoint_plan.current_waypoint = 0;
            waypoint_plan.wait_start_tick = None;
            waypoint_plan.add_waypoint(
                Waypoint::new(*position, WaypointBehavior::HoldAt)
                    .with_pace(MovementPace::Quick),
            );

            // Set engagement rule to defensive
            plan.engagement_rules.retain(|(id, _)| *id != unit_id);
            plan.engagement_rules
                .push((unit_id, EngagementRule::Defensive));

            ApplyOrderResult {
                success: true,
                affected_units: vec![unit_id],
                message: format!("Defend {:?}", position),
            }
        }

        OrderType::Retreat(route) => {
            let waypoint_plan = get_or_create_waypoint_plan(plan, unit_id);
            waypoint_plan.waypoints.clear();
            waypoint_plan.current_waypoint = 0;
            waypoint_plan.wait_start_tick = None;

            // Add each position in route as a waypoint
            for (i, pos) in route.iter().enumerate() {
                let behavior = if i == route.len() - 1 {
                    WaypointBehavior::RallyAt
                } else {
                    WaypointBehavior::MoveTo
                };
                waypoint_plan
                    .add_waypoint(Waypoint::new(*pos, behavior).with_pace(MovementPace::Run));
            }

            // Set engagement rule to hold fire (don't engage while retreating)
            plan.engagement_rules.retain(|(id, _)| *id != unit_id);
            plan.engagement_rules
                .push((unit_id, EngagementRule::HoldFire));

            ApplyOrderResult {
                success: true,
                affected_units: vec![unit_id],
                message: "Retreat ordered".to_string(),
            }
        }

        OrderType::ChangeFormation(shape) => {
            if let Some(unit) = army.get_unit_mut(unit_id) {
                unit.formation_shape = shape.clone();
            }

            ApplyOrderResult {
                success: true,
                affected_units: vec![unit_id],
                message: format!("Formation changed to {:?}", shape),
            }
        }

        OrderType::ChangeEngagement(rule) => {
            plan.engagement_rules.retain(|(id, _)| *id != unit_id);
            plan.engagement_rules.push((unit_id, rule.clone()));

            ApplyOrderResult {
                success: true,
                affected_units: vec![unit_id],
                message: format!("Engagement rule changed to {:?}", rule),
            }
        }

        OrderType::ExecuteGoCode(go_code_id) => {
            // Mark go-code as triggered
            if let Some(gc) = plan.go_codes.iter_mut().find(|g| g.id == *go_code_id) {
                gc.triggered = true;
            }

            ApplyOrderResult {
                success: true,
                affected_units: vec![unit_id],
                message: format!("Go-code {:?} executed", go_code_id),
            }
        }

        OrderType::Rally => {
            if let Some(unit) = army.get_unit_mut(unit_id) {
                if unit.is_broken() {
                    unit.stance = UnitStance::Rallying;
                    unit.stress = (unit.stress - 0.2).max(0.0);
                }
            }

            ApplyOrderResult {
                success: true,
                affected_units: vec![unit_id],
                message: "Rally ordered".to_string(),
            }
        }

        OrderType::HoldPosition => {
            // Get current position before mutable borrow
            let current_pos = army.get_unit(unit_id).map(|u| u.position);

            let waypoint_plan = get_or_create_waypoint_plan(plan, unit_id);
            waypoint_plan.waypoints.clear();
            waypoint_plan.current_waypoint = 0;
            waypoint_plan.wait_start_tick = None;

            // Hold at current position
            if let Some(pos) = current_pos {
                waypoint_plan.add_waypoint(Waypoint::new(pos, WaypointBehavior::HoldAt));
            }

            if let Some(unit) = army.get_unit_mut(unit_id) {
                unit.stance = UnitStance::Formed;
            }

            ApplyOrderResult {
                success: true,
                affected_units: vec![unit_id],
                message: "Holding position".to_string(),
            }
        }
    }
}

fn get_or_create_waypoint_plan(plan: &mut BattlePlan, unit_id: UnitId) -> &mut WaypointPlan {
    // Find existing plan index
    let existing_idx = plan
        .waypoint_plans
        .iter()
        .position(|p| p.unit_id == unit_id);

    match existing_idx {
        Some(idx) => &mut plan.waypoint_plans[idx],
        None => {
            plan.waypoint_plans.push(WaypointPlan::new(unit_id));
            plan.waypoint_plans.last_mut().unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::planning::{GoCode, GoCodeTrigger};
    use crate::battle::unit_type::UnitType;
    use crate::battle::units::{ArmyId, BattleFormation, BattleUnit, Element, FormationId};
    use crate::core::types::EntityId;

    fn create_test_army_with_unit() -> (Army, UnitId) {
        let mut army = Army::new(ArmyId::new(), EntityId::new());
        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let unit_id = UnitId::new();
        let mut unit = BattleUnit::new(unit_id, UnitType::Infantry);
        unit.position = BattleHexCoord::new(0, 0);
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        formation.units.push(unit);
        army.formations.push(formation);
        (army, unit_id)
    }

    #[test]
    fn test_apply_move_to_order() {
        let (mut army, unit_id) = create_test_army_with_unit();
        let mut plan = BattlePlan::new();
        let destination = BattleHexCoord::new(10, 10);

        let order = Order::move_to(unit_id, destination);
        let result = apply_order(&order, &mut army, &mut plan);

        assert!(result.success);
        assert_eq!(result.affected_units, vec![unit_id]);

        // Check waypoint was created
        let wp_plan = plan.get_waypoint_plan(unit_id).unwrap();
        assert_eq!(wp_plan.waypoints.len(), 1);
        assert_eq!(wp_plan.waypoints[0].position, destination);
        assert_eq!(wp_plan.waypoints[0].behavior, WaypointBehavior::MoveTo);

        // Check unit stance changed
        let unit = army.get_unit(unit_id).unwrap();
        assert_eq!(unit.stance, UnitStance::Moving);
    }

    #[test]
    fn test_apply_attack_order() {
        let (mut army, unit_id) = create_test_army_with_unit();

        // Add a target unit
        let target_id = UnitId::new();
        let mut target_unit = BattleUnit::new(target_id, UnitType::Infantry);
        target_unit.position = BattleHexCoord::new(5, 5);
        army.formations[0].units.push(target_unit);

        let mut plan = BattlePlan::new();

        let order = Order::attack(unit_id, target_id);
        let result = apply_order(&order, &mut army, &mut plan);

        assert!(result.success);

        // Check waypoint was created at target position
        let wp_plan = plan.get_waypoint_plan(unit_id).unwrap();
        assert_eq!(wp_plan.waypoints[0].position, BattleHexCoord::new(5, 5));
        assert_eq!(wp_plan.waypoints[0].behavior, WaypointBehavior::AttackFrom);
        assert_eq!(wp_plan.waypoints[0].pace, MovementPace::Run);

        // Check engagement rule is aggressive
        let rule = plan.get_engagement_rule(unit_id);
        assert!(matches!(rule, EngagementRule::Aggressive));
    }

    #[test]
    fn test_apply_defend_order() {
        let (mut army, unit_id) = create_test_army_with_unit();
        let mut plan = BattlePlan::new();
        let position = BattleHexCoord::new(3, 3);

        let order = Order {
            order_type: OrderType::Defend(position),
            target: OrderTarget::Unit(unit_id),
            issued_at: 0,
        };
        let result = apply_order(&order, &mut army, &mut plan);

        assert!(result.success);

        // Check waypoint was created
        let wp_plan = plan.get_waypoint_plan(unit_id).unwrap();
        assert_eq!(wp_plan.waypoints[0].position, position);
        assert_eq!(wp_plan.waypoints[0].behavior, WaypointBehavior::HoldAt);

        // Check engagement rule is defensive
        let rule = plan.get_engagement_rule(unit_id);
        assert!(matches!(rule, EngagementRule::Defensive));
    }

    #[test]
    fn test_apply_retreat_order() {
        let (mut army, unit_id) = create_test_army_with_unit();
        let mut plan = BattlePlan::new();
        let route = vec![
            BattleHexCoord::new(5, 5),
            BattleHexCoord::new(3, 3),
            BattleHexCoord::new(0, 0),
        ];

        let order = Order::retreat(unit_id, route.clone());
        let result = apply_order(&order, &mut army, &mut plan);

        assert!(result.success);

        // Check waypoints match route
        let wp_plan = plan.get_waypoint_plan(unit_id).unwrap();
        assert_eq!(wp_plan.waypoints.len(), 3);
        assert_eq!(wp_plan.waypoints[0].position, route[0]);
        assert_eq!(wp_plan.waypoints[1].position, route[1]);
        assert_eq!(wp_plan.waypoints[2].position, route[2]);

        // Last waypoint should be RallyAt
        assert_eq!(wp_plan.waypoints[2].behavior, WaypointBehavior::RallyAt);

        // All waypoints should use Run pace
        for wp in &wp_plan.waypoints {
            assert_eq!(wp.pace, MovementPace::Run);
        }

        // Engagement rule should be HoldFire
        let rule = plan.get_engagement_rule(unit_id);
        assert!(matches!(rule, EngagementRule::HoldFire));
    }

    #[test]
    fn test_apply_change_formation_order() {
        let (mut army, unit_id) = create_test_army_with_unit();
        let mut plan = BattlePlan::new();
        let new_shape = FormationShape::Square;

        let order = Order {
            order_type: OrderType::ChangeFormation(new_shape),
            target: OrderTarget::Unit(unit_id),
            issued_at: 0,
        };
        let result = apply_order(&order, &mut army, &mut plan);

        assert!(result.success);

        // Check formation shape changed
        let unit = army.get_unit(unit_id).unwrap();
        assert!(matches!(unit.formation_shape, FormationShape::Square));
    }

    #[test]
    fn test_apply_change_engagement_order() {
        let (mut army, unit_id) = create_test_army_with_unit();
        let mut plan = BattlePlan::new();

        let order = Order {
            order_type: OrderType::ChangeEngagement(EngagementRule::Skirmish),
            target: OrderTarget::Unit(unit_id),
            issued_at: 0,
        };
        let result = apply_order(&order, &mut army, &mut plan);

        assert!(result.success);

        // Check engagement rule changed
        let rule = plan.get_engagement_rule(unit_id);
        assert!(matches!(rule, EngagementRule::Skirmish));
    }

    #[test]
    fn test_apply_execute_go_code_order() {
        let (mut army, unit_id) = create_test_army_with_unit();
        let mut plan = BattlePlan::new();

        // Add a go-code
        let go_code = GoCode::new("ATTACK".into(), GoCodeTrigger::Manual);
        let go_code_id = go_code.id;
        plan.go_codes.push(go_code);

        let order = Order {
            order_type: OrderType::ExecuteGoCode(go_code_id),
            target: OrderTarget::Unit(unit_id),
            issued_at: 0,
        };
        let result = apply_order(&order, &mut army, &mut plan);

        assert!(result.success);

        // Check go-code was triggered
        let gc = plan.go_codes.iter().find(|g| g.id == go_code_id).unwrap();
        assert!(gc.triggered);
    }

    #[test]
    fn test_apply_rally_order_to_broken_unit() {
        let (mut army, unit_id) = create_test_army_with_unit();
        let mut plan = BattlePlan::new();

        // Set unit to routing with high stress
        if let Some(unit) = army.get_unit_mut(unit_id) {
            unit.stance = UnitStance::Routing;
            unit.stress = 0.8;
        }

        let order = Order {
            order_type: OrderType::Rally,
            target: OrderTarget::Unit(unit_id),
            issued_at: 0,
        };
        let result = apply_order(&order, &mut army, &mut plan);

        assert!(result.success);

        // Check unit is rallying with reduced stress
        let unit = army.get_unit(unit_id).unwrap();
        assert_eq!(unit.stance, UnitStance::Rallying);
        assert!((unit.stress - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_apply_rally_order_to_formed_unit() {
        let (mut army, unit_id) = create_test_army_with_unit();
        let mut plan = BattlePlan::new();

        // Unit is formed, not routing
        let order = Order {
            order_type: OrderType::Rally,
            target: OrderTarget::Unit(unit_id),
            issued_at: 0,
        };
        let result = apply_order(&order, &mut army, &mut plan);

        assert!(result.success);

        // Unit should still be formed (not broken, so rally doesn't change stance)
        let unit = army.get_unit(unit_id).unwrap();
        assert_eq!(unit.stance, UnitStance::Formed);
    }

    #[test]
    fn test_apply_hold_position_order() {
        let (mut army, unit_id) = create_test_army_with_unit();
        let mut plan = BattlePlan::new();

        // Set unit position
        if let Some(unit) = army.get_unit_mut(unit_id) {
            unit.position = BattleHexCoord::new(5, 5);
            unit.stance = UnitStance::Moving;
        }

        let order = Order::hold(unit_id);
        let result = apply_order(&order, &mut army, &mut plan);

        assert!(result.success);

        // Check unit stance
        let unit = army.get_unit(unit_id).unwrap();
        assert_eq!(unit.stance, UnitStance::Formed);

        // Check waypoint at current position
        let wp_plan = plan.get_waypoint_plan(unit_id).unwrap();
        assert_eq!(wp_plan.waypoints[0].position, BattleHexCoord::new(5, 5));
        assert_eq!(wp_plan.waypoints[0].behavior, WaypointBehavior::HoldAt);
    }

    #[test]
    fn test_apply_order_to_formation() {
        let mut army = Army::new(ArmyId::new(), EntityId::new());
        let formation_id = FormationId::new();
        let mut formation = BattleFormation::new(formation_id, EntityId::new());

        // Add multiple units to formation
        let unit1_id = UnitId::new();
        let mut unit1 = BattleUnit::new(unit1_id, UnitType::Infantry);
        unit1.position = BattleHexCoord::new(0, 0);
        formation.units.push(unit1);

        let unit2_id = UnitId::new();
        let mut unit2 = BattleUnit::new(unit2_id, UnitType::Infantry);
        unit2.position = BattleHexCoord::new(1, 0);
        formation.units.push(unit2);

        army.formations.push(formation);

        let mut plan = BattlePlan::new();
        let destination = BattleHexCoord::new(10, 10);

        let order = Order {
            order_type: OrderType::MoveTo(destination),
            target: OrderTarget::Formation(formation_id),
            issued_at: 0,
        };
        let result = apply_order(&order, &mut army, &mut plan);

        assert!(result.success);
        assert_eq!(result.affected_units.len(), 2);
        assert!(result.affected_units.contains(&unit1_id));
        assert!(result.affected_units.contains(&unit2_id));

        // Both units should have waypoint plans
        assert!(plan.get_waypoint_plan(unit1_id).is_some());
        assert!(plan.get_waypoint_plan(unit2_id).is_some());
    }

    #[test]
    fn test_move_to_clears_previous_waypoints() {
        let (mut army, unit_id) = create_test_army_with_unit();
        let mut plan = BattlePlan::new();

        // Add initial waypoints
        let mut wp_plan = WaypointPlan::new(unit_id);
        wp_plan.add_waypoint(Waypoint::new(
            BattleHexCoord::new(1, 1),
            WaypointBehavior::MoveTo,
        ));
        wp_plan.add_waypoint(Waypoint::new(
            BattleHexCoord::new(2, 2),
            WaypointBehavior::HoldAt,
        ));
        wp_plan.wait_start_tick = Some(5);
        plan.waypoint_plans.push(wp_plan);

        // Apply new move order
        let destination = BattleHexCoord::new(10, 10);
        let order = Order::move_to(unit_id, destination);
        apply_order(&order, &mut army, &mut plan);

        // Should only have 1 waypoint now
        let wp_plan = plan.get_waypoint_plan(unit_id).unwrap();
        assert_eq!(wp_plan.waypoints.len(), 1);
        assert_eq!(wp_plan.current_waypoint, 0);
        assert!(wp_plan.wait_start_tick.is_none());
    }
}
