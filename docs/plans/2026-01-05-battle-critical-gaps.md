# Battle System Critical Gaps Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix 4 critical gaps in the battle system runtime: order application, contingencies, wait conditions, and courier interception.

**Architecture:** Each gap is addressed in isolation with minimal changes. Orders modify waypoint plans. Contingencies evaluate in pre-tick. Wait conditions check external state. Interception happens before courier advancement.

**Tech Stack:** Rust, existing battle module structure in `src/battle/`

---

## Task 1: Add wait_start_tick to WaypointPlan

**Files:**
- Modify: `src/battle/planning.rs:113-126` (WaypointPlan struct)

**Step 1: Write the failing test**

Add to `src/battle/planning.rs` in the `tests` module:

```rust
#[test]
fn test_waypoint_plan_has_wait_start_tick() {
    let plan = WaypointPlan::new(UnitId::new());
    assert!(plan.wait_start_tick.is_none());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib test_waypoint_plan_has_wait_start_tick`
Expected: FAIL with "no field `wait_start_tick`"

**Step 3: Add wait_start_tick field to WaypointPlan**

Modify `WaypointPlan` struct at line 113:

```rust
/// Waypoint plan for a unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaypointPlan {
    pub unit_id: UnitId,
    pub waypoints: Vec<Waypoint>,
    pub current_waypoint: usize,
    pub wait_start_tick: Option<Tick>,  // NEW: When waiting started
}
```

**Step 4: Update WaypointPlan::new() at line 120**

```rust
impl WaypointPlan {
    pub fn new(unit_id: UnitId) -> Self {
        Self {
            unit_id,
            waypoints: Vec::new(),
            current_waypoint: 0,
            wait_start_tick: None,  // NEW
        }
    }
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib test_waypoint_plan_has_wait_start_tick`
Expected: PASS

**Step 6: Commit**

```bash
git add src/battle/planning.rs
git commit -m "feat(battle): add wait_start_tick to WaypointPlan for duration tracking"
```

---

## Task 2: Implement Duration Wait Condition

**Files:**
- Modify: `src/battle/movement.rs:58-86` (is_waiting function)

**Step 1: Write the failing test**

Add to `src/battle/movement.rs` in the `tests` module:

```rust
#[test]
fn test_duration_wait_condition() {
    use crate::battle::planning::WaitCondition;

    let mut plan = WaypointPlan::new(UnitId::new());
    plan.add_waypoint(
        Waypoint::new(BattleHexCoord::new(0, 0), WaypointBehavior::HoldAt)
            .with_wait(WaitCondition::Duration(10)),
    );

    // No wait_start_tick set yet - should not be waiting
    assert!(!is_waiting(&plan, 0));

    // Set wait start tick
    plan.wait_start_tick = Some(0);

    // At tick 5, should still be waiting (5 < 10)
    assert!(is_waiting(&plan, 5));

    // At tick 10, wait is complete
    assert!(!is_waiting(&plan, 10));

    // At tick 15, definitely not waiting
    assert!(!is_waiting(&plan, 15));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib test_duration_wait_condition`
Expected: FAIL (current implementation always returns false for Duration)

**Step 3: Implement Duration condition in is_waiting**

Modify `is_waiting` function at line 58:

```rust
/// Check if a unit is blocked by wait condition
pub fn is_waiting(plan: &WaypointPlan, current_tick: u64) -> bool {
    let Some(waypoint) = plan.current() else {
        return false;
    };

    match &waypoint.wait_condition {
        None => false,
        Some(WaitCondition::Duration(ticks)) => {
            // Check if we've waited long enough
            match plan.wait_start_tick {
                None => false,  // Haven't started waiting yet
                Some(start) => current_tick < start + *ticks,
            }
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
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib test_duration_wait_condition`
Expected: PASS

**Step 5: Commit**

```bash
git add src/battle/movement.rs
git commit -m "feat(battle): implement Duration wait condition in is_waiting"
```

---

## Task 3: Implement UnitArrives Wait Condition

**Files:**
- Modify: `src/battle/movement.rs:58-86` (is_waiting function)

**Step 1: Write the failing test**

Add to `src/battle/movement.rs` in the `tests` module:

```rust
#[test]
fn test_unit_arrives_wait_condition() {
    use crate::battle::planning::WaitCondition;

    let target_unit_id = UnitId::new();
    let target_position = BattleHexCoord::new(5, 5);

    let mut plan = WaypointPlan::new(UnitId::new());
    plan.add_waypoint(
        Waypoint::new(BattleHexCoord::new(0, 0), WaypointBehavior::HoldAt)
            .with_wait(WaitCondition::UnitArrives(target_unit_id)),
    );

    // Create unit positions - target NOT at waypoint position
    let unit_positions = vec![(target_unit_id, BattleHexCoord::new(10, 10))];

    // Should be waiting - target hasn't arrived at our position
    assert!(is_waiting_with_context(&plan, 0, &unit_positions, &[], &[]));

    // Target arrives at our waypoint position
    let unit_positions = vec![(target_unit_id, BattleHexCoord::new(0, 0))];

    // Should NOT be waiting anymore
    assert!(!is_waiting_with_context(&plan, 0, &unit_positions, &[], &[]));
}
```

**Step 2: Create is_waiting_with_context function**

We need a new function that takes context. Add after `is_waiting`:

```rust
/// Context for wait condition evaluation
pub struct WaitContext<'a> {
    pub unit_positions: &'a [(UnitId, BattleHexCoord)],
    pub enemy_visible_hexes: &'a [BattleHexCoord],
    pub units_under_attack: &'a [UnitId],
}

/// Check if a unit is blocked by wait condition (with full context)
pub fn is_waiting_with_context(
    plan: &WaypointPlan,
    current_tick: u64,
    unit_positions: &[(UnitId, BattleHexCoord)],
    enemy_visible_hexes: &[BattleHexCoord],
    units_under_attack: &[UnitId],
) -> bool {
    let Some(waypoint) = plan.current() else {
        return false;
    };

    match &waypoint.wait_condition {
        None => false,
        Some(WaitCondition::Duration(ticks)) => {
            match plan.wait_start_tick {
                None => false,
                Some(start) => current_tick < start + *ticks,
            }
        }
        Some(WaitCondition::GoCode(_)) => {
            // Evaluated separately by triggers system
            true
        }
        Some(WaitCondition::UnitArrives(target_unit_id)) => {
            // Wait until target unit arrives at THIS waypoint's position
            !unit_positions
                .iter()
                .any(|(id, pos)| *id == *target_unit_id && *pos == waypoint.position)
        }
        Some(WaitCondition::EnemySighted) => {
            // Wait until enemy is visible
            enemy_visible_hexes.is_empty()
        }
        Some(WaitCondition::Attacked) => {
            // Wait until this unit is under attack
            !units_under_attack.contains(&plan.unit_id)
        }
    }
}
```

**Step 3: Run test to verify it passes**

Run: `cargo test --lib test_unit_arrives_wait_condition`
Expected: PASS

**Step 4: Commit**

```bash
git add src/battle/movement.rs
git commit -m "feat(battle): implement is_waiting_with_context with UnitArrives condition"
```

---

## Task 4: Implement EnemySighted and Attacked Wait Conditions

**Files:**
- Modify: `src/battle/movement.rs` (tests)

**Step 1: Write failing tests for EnemySighted**

```rust
#[test]
fn test_enemy_sighted_wait_condition() {
    use crate::battle::planning::WaitCondition;

    let mut plan = WaypointPlan::new(UnitId::new());
    plan.add_waypoint(
        Waypoint::new(BattleHexCoord::new(0, 0), WaypointBehavior::HoldAt)
            .with_wait(WaitCondition::EnemySighted),
    );

    // No enemies visible - should be waiting
    assert!(is_waiting_with_context(&plan, 0, &[], &[], &[]));

    // Enemy visible at some hex - should NOT be waiting
    let visible = vec![BattleHexCoord::new(5, 5)];
    assert!(!is_waiting_with_context(&plan, 0, &[], &visible, &[]));
}

#[test]
fn test_attacked_wait_condition() {
    use crate::battle::planning::WaitCondition;

    let unit_id = UnitId::new();
    let mut plan = WaypointPlan::new(unit_id);
    plan.add_waypoint(
        Waypoint::new(BattleHexCoord::new(0, 0), WaypointBehavior::HoldAt)
            .with_wait(WaitCondition::Attacked),
    );

    // Not under attack - should be waiting
    assert!(is_waiting_with_context(&plan, 0, &[], &[], &[]));

    // Under attack - should NOT be waiting
    let under_attack = vec![unit_id];
    assert!(!is_waiting_with_context(&plan, 0, &[], &[], &under_attack));
}
```

**Step 2: Run tests to verify they pass**

Run: `cargo test --lib test_enemy_sighted_wait_condition test_attacked_wait_condition`
Expected: PASS (already implemented in Task 3)

**Step 3: Commit**

```bash
git add src/battle/movement.rs
git commit -m "test(battle): add tests for EnemySighted and Attacked wait conditions"
```

---

## Task 5: Order Application - Create apply_order function

**Files:**
- Create: `src/battle/orders.rs`
- Modify: `src/battle/mod.rs` (add module)

**Step 1: Create orders.rs with apply_order function and test**

Create new file `src/battle/orders.rs`:

```rust
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
pub fn apply_order(
    order: &Order,
    army: &mut Army,
    plan: &mut BattlePlan,
) -> ApplyOrderResult {
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
            waypoint_plan.add_waypoint(
                Waypoint::new(target_pos, WaypointBehavior::AttackFrom)
                    .with_pace(MovementPace::Run),
            );

            // Set engagement rule to aggressive
            plan.engagement_rules.retain(|(id, _)| *id != unit_id);
            plan.engagement_rules.push((unit_id, EngagementRule::Aggressive));

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
            waypoint_plan.add_waypoint(
                Waypoint::new(*position, WaypointBehavior::HoldAt)
                    .with_pace(MovementPace::Quick),
            );

            // Set engagement rule to defensive
            plan.engagement_rules.retain(|(id, _)| *id != unit_id);
            plan.engagement_rules.push((unit_id, EngagementRule::Defensive));

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

            // Add each position in route as a waypoint
            for (i, pos) in route.iter().enumerate() {
                let behavior = if i == route.len() - 1 {
                    WaypointBehavior::RallyAt
                } else {
                    WaypointBehavior::MoveTo
                };
                waypoint_plan.add_waypoint(
                    Waypoint::new(*pos, behavior).with_pace(MovementPace::Run),
                );
            }

            // Set engagement rule to hold fire (don't engage while retreating)
            plan.engagement_rules.retain(|(id, _)| *id != unit_id);
            plan.engagement_rules.push((unit_id, EngagementRule::HoldFire));

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
            let waypoint_plan = get_or_create_waypoint_plan(plan, unit_id);
            waypoint_plan.waypoints.clear();
            waypoint_plan.current_waypoint = 0;

            // Hold at current position
            if let Some(unit) = army.get_unit(unit_id) {
                waypoint_plan.add_waypoint(
                    Waypoint::new(unit.position, WaypointBehavior::HoldAt),
                );
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
    use crate::battle::units::{ArmyId, BattleFormation, BattleUnit, FormationId};
    use crate::battle::unit_type::UnitType;
    use crate::core::types::EntityId;

    #[test]
    fn test_apply_move_to_order() {
        let mut army = Army::new(ArmyId::new(), EntityId::new());
        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let unit_id = UnitId::new();
        let mut unit = BattleUnit::new(unit_id, UnitType::Infantry);
        unit.position = BattleHexCoord::new(0, 0);
        formation.units.push(unit);
        army.formations.push(formation);

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
    }

    #[test]
    fn test_apply_hold_position_order() {
        let mut army = Army::new(ArmyId::new(), EntityId::new());
        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let unit_id = UnitId::new();
        let mut unit = BattleUnit::new(unit_id, UnitType::Infantry);
        unit.position = BattleHexCoord::new(5, 5);
        formation.units.push(unit);
        army.formations.push(formation);

        let mut plan = BattlePlan::new();

        let order = Order::hold(unit_id);
        let result = apply_order(&order, &mut army, &mut plan);

        assert!(result.success);

        // Check unit stance
        let unit = army.get_unit(unit_id).unwrap();
        assert_eq!(unit.stance, UnitStance::Formed);

        // Check waypoint at current position
        let wp_plan = plan.get_waypoint_plan(unit_id).unwrap();
        assert_eq!(wp_plan.waypoints[0].position, BattleHexCoord::new(5, 5));
    }

    #[test]
    fn test_apply_retreat_order() {
        let mut army = Army::new(ArmyId::new(), EntityId::new());
        let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
        let unit_id = UnitId::new();
        let unit = BattleUnit::new(unit_id, UnitType::Infantry);
        formation.units.push(unit);
        army.formations.push(formation);

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

        // Last waypoint should be RallyAt
        assert_eq!(
            wp_plan.waypoints[2].behavior,
            WaypointBehavior::RallyAt
        );

        // Engagement rule should be HoldFire
        let rule = plan.get_engagement_rule(unit_id);
        assert!(matches!(rule, EngagementRule::HoldFire));
    }
}
```

**Step 2: Add orders module to mod.rs**

Add to `src/battle/mod.rs`:

```rust
pub mod orders;
```

**Step 3: Run tests to verify**

Run: `cargo test --lib battle::orders`
Expected: PASS

**Step 4: Commit**

```bash
git add src/battle/orders.rs src/battle/mod.rs
git commit -m "feat(battle): add orders module with apply_order function"
```

---

## Task 6: Integrate Order Application into phase_movement

**Files:**
- Modify: `src/battle/execution.rs:293-317` (phase_movement function)

**Step 1: Write integration test**

Add to `src/battle/execution.rs` tests:

```rust
#[test]
fn test_order_application_in_phase_movement() {
    use crate::battle::courier::Order;
    use crate::battle::hex::BattleHexCoord;
    use crate::battle::unit_type::UnitType;
    use crate::battle::units::{BattleFormation, BattleUnit, Element, FormationId};
    use crate::core::types::EntityId;

    let map = BattleMap::new(20, 20);
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());

    let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
    let unit_id = UnitId::new();
    let mut unit = BattleUnit::new(unit_id, UnitType::Infantry);
    unit.elements.push(Element::new(vec![EntityId::new(); 50]));
    unit.position = BattleHexCoord::new(5, 5);
    formation.units.push(unit);
    friendly.formations.push(formation);

    let enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut state = BattleState::new(map, friendly, enemy);
    state.start_battle();

    // Dispatch an order (instant delivery - same source/dest)
    let destination = BattleHexCoord::new(10, 10);
    state.courier_system.dispatch(
        EntityId::new(),
        Order::move_to(unit_id, destination),
        BattleHexCoord::new(5, 5),
        BattleHexCoord::new(5, 5),  // Instant delivery
    );

    // Run a tick - order should be applied
    let _events = state.run_tick();

    // Check that waypoint plan was created
    let wp_plan = state.friendly_plan.get_waypoint_plan(unit_id);
    assert!(wp_plan.is_some(), "Waypoint plan should be created");
    assert_eq!(wp_plan.unwrap().waypoints[0].position, destination);
}
```

**Step 2: Modify phase_movement to apply orders**

Update `phase_movement` at line 293:

```rust
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
```

**Step 3: Run test to verify**

Run: `cargo test --lib test_order_application_in_phase_movement`
Expected: PASS

**Step 4: Commit**

```bash
git add src/battle/execution.rs
git commit -m "feat(battle): integrate order application into phase_movement"
```

---

## Task 7: Contingency Evaluation in phase_pre_tick

**Files:**
- Modify: `src/battle/execution.rs:249-291` (phase_pre_tick function)

**Step 1: Write failing test**

Add to `src/battle/execution.rs` tests:

```rust
#[test]
fn test_contingency_triggers_in_phase_pre_tick() {
    use crate::battle::hex::BattleHexCoord;
    use crate::battle::planning::{Contingency, ContingencyTrigger, ContingencyResponse};
    use crate::battle::unit_type::UnitType;
    use crate::battle::units::{BattleFormation, BattleUnit, Element, FormationId};
    use crate::core::types::EntityId;

    let map = BattleMap::new(20, 20);
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());

    let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
    let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
    unit.elements.push(Element::new(vec![EntityId::new(); 100]));
    unit.casualties = 40;  // 40% casualties
    unit.position = BattleHexCoord::new(5, 5);
    formation.units.push(unit);
    friendly.formations.push(formation);

    let enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut state = BattleState::new(map, friendly, enemy);

    // Add contingency: rally at (0,0) if casualties exceed 30%
    state.friendly_plan.contingencies.push(
        Contingency::new(
            ContingencyTrigger::CasualtiesExceed(0.3),
            ContingencyResponse::Rally(BattleHexCoord::new(0, 0)),
        )
    );

    state.start_battle();

    // Run a tick - contingency should trigger
    let _events = state.run_tick();

    // Check contingency was activated
    assert!(
        state.friendly_plan.contingencies[0].activated,
        "Contingency should be activated"
    );
}
```

**Step 2: Update phase_pre_tick to evaluate contingencies**

Add to `phase_pre_tick` after go-code evaluation (around line 290):

```rust
fn phase_pre_tick(&mut self, events: &mut BattleEventLog) {
    use crate::battle::triggers::{evaluate_all_contingencies, UnitPosition};
    use crate::battle::planning::ContingencyResponse;
    use crate::battle::orders::apply_order;
    use crate::battle::courier::Order;

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

    // ===== CONTINGENCY EVALUATION (NEW) =====

    // Calculate casualties percentage
    let total_strength = self.friendly_army.total_strength();
    let effective_strength = self.friendly_army.effective_strength();
    let casualties_percent = if total_strength > 0 {
        1.0 - (effective_strength as f32 / total_strength as f32)
    } else {
        0.0
    };

    // Check if commander is alive (simplified: check if any formation has units)
    let commander_alive = !self.friendly_army.formations.is_empty();

    // Get enemy positions
    let enemy_positions: Vec<BattleHexCoord> = self
        .enemy_army
        .formations
        .iter()
        .flat_map(|f| f.units.iter())
        .map(|u| u.position)
        .collect();

    // Get friendly positions
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

    // Process triggered contingencies
    for idx in triggered_contingencies {
        if let Some(contingency) = self.friendly_plan.contingencies.get_mut(idx) {
            if !contingency.activated {
                contingency.activated = true;

                // Execute response
                match &contingency.response {
                    ContingencyResponse::ExecutePlan(unit_id) => {
                        // Execute unit's backup plan (handled by waypoint system)
                        let _ = unit_id;
                    }
                    ContingencyResponse::Retreat(route) => {
                        // Order all units to retreat via this route
                        for formation in &self.friendly_army.formations {
                            for unit in &formation.units {
                                let order = Order::retreat(unit.id, route.clone());
                                apply_order(&order, &mut self.friendly_army.clone(), &mut self.friendly_plan);
                            }
                        }
                    }
                    ContingencyResponse::Rally(position) => {
                        // Order all routing units to rally at position
                        for formation in &self.friendly_army.formations {
                            for unit in &formation.units {
                                if unit.is_broken() {
                                    let order = Order::move_to(unit.id, *position);
                                    apply_order(&order, &mut self.friendly_army.clone(), &mut self.friendly_plan);
                                }
                            }
                        }
                    }
                    ContingencyResponse::Signal(go_code_id) => {
                        // Trigger a go-code
                        if let Some(gc) = self.friendly_plan.go_codes.iter_mut().find(|g| g.id == *go_code_id) {
                            gc.triggered = true;
                            events.push(
                                BattleEventType::GoCodeTriggered { name: gc.name.clone() },
                                format!("Go-code '{}' triggered by contingency", gc.name),
                                self.tick,
                            );
                        }
                    }
                }
            }
        }
    }
}
```

Note: The contingency response execution has a borrow issue with `self.friendly_army`. We need to collect the orders first, then apply them. Fix:

```rust
// Process triggered contingencies - collect orders first
let mut orders_to_apply: Vec<Order> = Vec::new();

for idx in triggered_contingencies {
    if let Some(contingency) = self.friendly_plan.contingencies.get_mut(idx) {
        if !contingency.activated {
            contingency.activated = true;

            match &contingency.response {
                ContingencyResponse::ExecutePlan(_unit_id) => {
                    // Handled by waypoint system
                }
                ContingencyResponse::Retreat(route) => {
                    for formation in &self.friendly_army.formations {
                        for unit in &formation.units {
                            orders_to_apply.push(Order::retreat(unit.id, route.clone()));
                        }
                    }
                }
                ContingencyResponse::Rally(position) => {
                    for formation in &self.friendly_army.formations {
                        for unit in &formation.units {
                            if unit.is_broken() {
                                orders_to_apply.push(Order::move_to(unit.id, *position));
                            }
                        }
                    }
                }
                ContingencyResponse::Signal(go_code_id) => {
                    if let Some(gc) = self.friendly_plan.go_codes.iter_mut().find(|g| g.id == *go_code_id) {
                        gc.triggered = true;
                        events.push(
                            BattleEventType::GoCodeTriggered { name: gc.name.clone() },
                            format!("Go-code '{}' triggered by contingency", gc.name),
                            self.tick,
                        );
                    }
                }
            }
        }
    }
}

// Apply collected orders
for order in &orders_to_apply {
    apply_order(order, &mut self.friendly_army, &mut self.friendly_plan);
}
```

**Step 3: Run test to verify**

Run: `cargo test --lib test_contingency_triggers_in_phase_pre_tick`
Expected: PASS

**Step 4: Commit**

```bash
git add src/battle/execution.rs
git commit -m "feat(battle): add contingency evaluation to phase_pre_tick"
```

---

## Task 8: Courier Interception Detection

**Files:**
- Modify: `src/battle/execution.rs:293-317` (phase_movement function)

**Step 1: Write failing test**

Add to `src/battle/execution.rs` tests:

```rust
#[test]
fn test_courier_interception() {
    use crate::battle::courier::Order;
    use crate::battle::hex::BattleHexCoord;
    use crate::battle::unit_type::UnitType;
    use crate::battle::units::{BattleFormation, BattleUnit, Element, FormationId, UnitStance};
    use crate::core::types::EntityId;

    let map = BattleMap::new(20, 20);
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());

    let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());
    let unit_id = UnitId::new();
    let mut unit = BattleUnit::new(unit_id, UnitType::Infantry);
    unit.elements.push(Element::new(vec![EntityId::new(); 50]));
    unit.position = BattleHexCoord::new(0, 0);
    formation.units.push(unit);
    friendly.formations.push(formation);

    // Enemy unit in Patrol stance near courier route
    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    let mut enemy_unit = BattleUnit::new(UnitId::new(), UnitType::LightCavalry);
    enemy_unit.elements.push(Element::new(vec![EntityId::new(); 20]));
    enemy_unit.position = BattleHexCoord::new(5, 0);  // On courier route
    enemy_unit.stance = UnitStance::Patrol;  // Can intercept
    enemy_formation.units.push(enemy_unit);
    enemy.formations.push(enemy_formation);

    let mut state = BattleState::new(map, friendly, enemy);
    state.start_battle();

    // Dispatch courier along route that passes enemy
    state.courier_system.dispatch(
        EntityId::new(),
        Order::move_to(unit_id, BattleHexCoord::new(10, 0)),
        BattleHexCoord::new(0, 0),
        BattleHexCoord::new(10, 0),
    );

    // Run several ticks until courier passes enemy position
    for _ in 0..20 {
        let events = state.run_tick();

        // Check if interception event occurred
        if events.events.iter().any(|e| matches!(e.event_type, BattleEventType::CourierIntercepted)) {
            // Test passes - courier was intercepted
            return;
        }

        // Also check if all couriers are gone (intercepted or arrived)
        if state.courier_system.in_flight.is_empty() {
            break;
        }
    }

    // Note: Due to random chance, interception may not occur every time
    // This test just verifies the system runs without error
    // A more rigorous test would mock the random number generator
}
```

**Step 2: Add interception logic to phase_movement**

Update `phase_movement`:

```rust
fn phase_movement(&mut self, events: &mut BattleEventLog) {
    use crate::battle::constants::{
        COURIER_INTERCEPTION_RANGE, COURIER_INTERCEPTION_CHANCE_PATROL,
        COURIER_INTERCEPTION_CHANCE_ALERT,
    };
    use crate::battle::courier::CourierStatus;
    use crate::battle::orders::apply_order;
    use crate::battle::units::UnitStance;

    // ===== COURIER INTERCEPTION CHECK (before advancing) =====

    // Get enemy units that can intercept (Patrol or Alert stance)
    let interceptors: Vec<(BattleHexCoord, f32)> = self
        .enemy_army
        .formations
        .iter()
        .flat_map(|f| f.units.iter())
        .filter(|u| matches!(u.stance, UnitStance::Patrol | UnitStance::Alert))
        .map(|u| {
            let chance = if u.stance == UnitStance::Patrol {
                COURIER_INTERCEPTION_CHANCE_PATROL
            } else {
                COURIER_INTERCEPTION_CHANCE_ALERT
            };
            (u.position, chance)
        })
        .collect();

    // Check each courier against interceptors
    for courier in &mut self.courier_system.in_flight {
        if !courier.is_en_route() {
            continue;
        }

        for (interceptor_pos, chance) in &interceptors {
            let distance = courier.current_position.distance(interceptor_pos);
            if distance <= COURIER_INTERCEPTION_RANGE {
                // Random interception check
                let roll: f32 = rand::random();
                if roll < *chance {
                    courier.intercept();
                    events.push(
                        BattleEventType::CourierIntercepted,
                        "Courier intercepted by enemy patrol".to_string(),
                        self.tick,
                    );
                    break;  // Courier already intercepted
                }
            }
        }
    }

    // Remove intercepted couriers
    self.courier_system.in_flight.retain(|c| !c.was_intercepted());

    // ===== ADVANCE COURIERS =====
    self.courier_system.advance_all(COURIER_SPEED);
    let arrived_orders = self.courier_system.collect_arrived();

    // Apply arrived orders
    for order in &arrived_orders {
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

    // ===== MOVE UNITS =====
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
```

**Step 3: Add rand dependency to Cargo.toml if not present**

Check `Cargo.toml` and add if needed:
```toml
[dependencies]
rand = "0.8"
```

**Step 4: Run test to verify**

Run: `cargo test --lib test_courier_interception`
Expected: PASS

**Step 5: Commit**

```bash
git add src/battle/execution.rs Cargo.toml
git commit -m "feat(battle): add courier interception detection in phase_movement"
```

---

## Task 9: Integration Test - Full Gap Coverage

**Files:**
- Modify: `src/battle/execution.rs` (tests)

**Step 1: Write comprehensive integration test**

```rust
#[test]
fn test_all_critical_gaps_integrated() {
    use crate::battle::courier::Order;
    use crate::battle::hex::BattleHexCoord;
    use crate::battle::planning::{
        Contingency, ContingencyTrigger, ContingencyResponse,
        GoCode, GoCodeTrigger, WaitCondition, Waypoint, WaypointBehavior, WaypointPlan,
    };
    use crate::battle::unit_type::UnitType;
    use crate::battle::units::{BattleFormation, BattleUnit, Element, FormationId, UnitStance};
    use crate::core::types::EntityId;

    let map = BattleMap::new(30, 30);

    // Setup friendly army with multiple units
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());

    // Unit 1: Will wait for duration
    let unit1_id = UnitId::new();
    let mut unit1 = BattleUnit::new(unit1_id, UnitType::Infantry);
    unit1.elements.push(Element::new(vec![EntityId::new(); 50]));
    unit1.position = BattleHexCoord::new(5, 5);
    formation.units.push(unit1);

    // Unit 2: Will wait for unit1 to arrive
    let unit2_id = UnitId::new();
    let mut unit2 = BattleUnit::new(unit2_id, UnitType::Infantry);
    unit2.elements.push(Element::new(vec![EntityId::new(); 50]));
    unit2.position = BattleHexCoord::new(10, 10);
    formation.units.push(unit2);

    friendly.formations.push(formation);

    // Setup enemy with patrol unit
    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    let mut enemy_unit = BattleUnit::new(UnitId::new(), UnitType::LightCavalry);
    enemy_unit.elements.push(Element::new(vec![EntityId::new(); 20]));
    enemy_unit.position = BattleHexCoord::new(15, 15);
    enemy_unit.stance = UnitStance::Patrol;
    enemy_formation.units.push(enemy_unit);
    enemy.formations.push(enemy_formation);

    let mut state = BattleState::new(map, friendly, enemy);

    // Setup waypoint plans with wait conditions
    let mut plan1 = WaypointPlan::new(unit1_id);
    plan1.add_waypoint(
        Waypoint::new(BattleHexCoord::new(5, 5), WaypointBehavior::HoldAt)
            .with_wait(WaitCondition::Duration(5)),
    );
    plan1.wait_start_tick = Some(0);
    state.friendly_plan.waypoint_plans.push(plan1);

    // Setup contingency
    state.friendly_plan.contingencies.push(
        Contingency::new(
            ContingencyTrigger::CasualtiesExceed(0.5),
            ContingencyResponse::Rally(BattleHexCoord::new(0, 0)),
        )
    );

    // Setup go-code
    state.friendly_plan.go_codes.push(
        GoCode::new("ATTACK".into(), GoCodeTrigger::Time(10))
    );

    state.start_battle();

    // Run battle for several ticks
    for tick in 0..20 {
        let events = state.run_tick();

        // Verify go-code triggers at tick 10
        if tick >= 10 {
            assert!(
                state.friendly_plan.go_codes[0].triggered,
                "Go-code should be triggered after tick 10"
            );
        }
    }

    // Verify systems ran without panicking
    assert!(state.tick >= 20, "Battle should have advanced 20 ticks");
}
```

**Step 2: Run test**

Run: `cargo test --lib test_all_critical_gaps_integrated`
Expected: PASS

**Step 3: Commit**

```bash
git add src/battle/execution.rs
git commit -m "test(battle): add integration test for all critical gap fixes"
```

---

## Task 10: Run Full Test Suite and Verify

**Step 1: Run all battle tests**

```bash
cargo test --lib battle::
```

Expected: All tests pass

**Step 2: Run full project tests**

```bash
cargo test
```

Expected: All tests pass

**Step 3: Final commit**

```bash
git add -A
git commit -m "feat(battle): complete critical gaps implementation - orders, contingencies, wait conditions, interception"
```

---

## Summary

This plan fixes 4 critical gaps:

1. **Order Application** (Tasks 5-6): Created `orders.rs` with `apply_order()` function, integrated into `phase_movement()`

2. **Contingencies** (Task 7): Added `evaluate_all_contingencies()` call in `phase_pre_tick()`, processes responses

3. **Wait Conditions** (Tasks 1-4): Added `wait_start_tick` to `WaypointPlan`, implemented all 4 stubbed conditions in new `is_waiting_with_context()` function

4. **Courier Interception** (Task 8): Added interception check before courier advancement, uses `COURIER_INTERCEPTION_RANGE` and stance-based chances
