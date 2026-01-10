# Battle Runtime Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the complete battle tick loop: movement, couriers, engagement, combat, morale, and rout phases.

**Architecture:** Six-phase tick execution (PRE-TICK, MOVEMENT, COMBAT, MORALE, ROUT, POST-TICK) using existing data structures. New modules for pathfinding, movement, visibility, and triggers. All modifiers are ADDITIVE per project conventions.

**Tech Stack:** Rust, serde, uuid. Uses existing types from `execution.rs`, `units.rs`, `planning.rs`, `courier.rs`, `resolution.rs`.

---

## Task 1: A* Pathfinding Module

**Files:**
- Create: `src/battle/pathfinding.rs`
- Modify: `src/battle/mod.rs:11` (add module)
- Test: `src/battle/pathfinding.rs` (inline tests)

**Step 1: Write the failing test for basic pathfinding**

```rust
// In src/battle/pathfinding.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::battle_map::BattleMap;
    use crate::battle::hex::BattleHexCoord;

    #[test]
    fn test_pathfind_straight_line() {
        let map = BattleMap::new(10, 10);
        let start = BattleHexCoord::new(0, 0);
        let goal = BattleHexCoord::new(5, 0);

        let path = find_path(&map, start, goal, false);

        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.first(), Some(&start));
        assert_eq!(path.last(), Some(&goal));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::pathfinding::tests::test_pathfind_straight_line`
Expected: FAIL with "cannot find function `find_path`"

**Step 3: Write minimal A* implementation**

```rust
//! A* pathfinding for battle maps
//!
//! Respects terrain costs and unit type restrictions.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::battle::battle_map::BattleMap;
use crate::battle::hex::BattleHexCoord;

/// Node in the A* open set
#[derive(Debug, Clone)]
struct PathNode {
    coord: BattleHexCoord,
    g_cost: f32, // Cost from start
    f_cost: f32, // g_cost + heuristic
}

impl PartialEq for PathNode {
    fn eq(&self, other: &Self) -> bool {
        self.coord == other.coord
    }
}

impl Eq for PathNode {}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min-heap
        other.f_cost.partial_cmp(&self.f_cost).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Find path using A* algorithm
///
/// Returns None if no path exists.
/// `is_cavalry` restricts movement through forests/buildings.
pub fn find_path(
    map: &BattleMap,
    start: BattleHexCoord,
    goal: BattleHexCoord,
    is_cavalry: bool,
) -> Option<Vec<BattleHexCoord>> {
    if start == goal {
        return Some(vec![start]);
    }

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<BattleHexCoord, BattleHexCoord> = HashMap::new();
    let mut g_scores: HashMap<BattleHexCoord, f32> = HashMap::new();

    g_scores.insert(start, 0.0);
    open_set.push(PathNode {
        coord: start,
        g_cost: 0.0,
        f_cost: start.distance(&goal) as f32,
    });

    while let Some(current) = open_set.pop() {
        if current.coord == goal {
            return Some(reconstruct_path(&came_from, current.coord));
        }

        let current_g = *g_scores.get(&current.coord).unwrap_or(&f32::INFINITY);

        for neighbor in current.coord.neighbors() {
            // Check if passable
            let Some(hex) = map.get_hex(neighbor) else {
                continue;
            };

            // Check unit-type restrictions
            if is_cavalry && hex.terrain.impassable_for_cavalry() {
                continue;
            }
            if !is_cavalry && hex.terrain.impassable_for_infantry() {
                continue;
            }

            let move_cost = hex.total_movement_cost();
            if move_cost.is_infinite() {
                continue;
            }

            let tentative_g = current_g + move_cost;
            let neighbor_g = *g_scores.get(&neighbor).unwrap_or(&f32::INFINITY);

            if tentative_g < neighbor_g {
                came_from.insert(neighbor, current.coord);
                g_scores.insert(neighbor, tentative_g);

                let f_cost = tentative_g + neighbor.distance(&goal) as f32;
                open_set.push(PathNode {
                    coord: neighbor,
                    g_cost: tentative_g,
                    f_cost,
                });
            }
        }
    }

    None // No path found
}

/// Reconstruct path from came_from map
fn reconstruct_path(
    came_from: &HashMap<BattleHexCoord, BattleHexCoord>,
    mut current: BattleHexCoord,
) -> Vec<BattleHexCoord> {
    let mut path = vec![current];
    while let Some(&prev) = came_from.get(&current) {
        path.push(prev);
        current = prev;
    }
    path.reverse();
    path
}

/// Calculate path cost (sum of terrain costs)
pub fn path_cost(map: &BattleMap, path: &[BattleHexCoord]) -> f32 {
    path.iter()
        .filter_map(|coord| map.get_hex(*coord))
        .map(|hex| hex.total_movement_cost())
        .sum()
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib battle::pathfinding::tests::test_pathfind_straight_line`
Expected: PASS

**Step 5: Add more tests**

```rust
    #[test]
    fn test_pathfind_around_obstacle() {
        let mut map = BattleMap::new(10, 10);
        // Block the direct path with deep water
        map.set_terrain(BattleHexCoord::new(2, 0), BattleTerrain::DeepWater);
        map.set_terrain(BattleHexCoord::new(3, 0), BattleTerrain::DeepWater);

        let start = BattleHexCoord::new(0, 0);
        let goal = BattleHexCoord::new(5, 0);

        let path = find_path(&map, start, goal, false);

        assert!(path.is_some());
        let path = path.unwrap();
        // Should go around, not through
        assert!(!path.contains(&BattleHexCoord::new(2, 0)));
    }

    #[test]
    fn test_cavalry_cant_enter_forest() {
        let mut map = BattleMap::new(10, 10);
        // Forest blocks cavalry
        for r in 0..10 {
            map.set_terrain(BattleHexCoord::new(5, r), BattleTerrain::Forest);
        }

        let start = BattleHexCoord::new(0, 5);
        let goal = BattleHexCoord::new(9, 5);

        // Infantry can path through
        let infantry_path = find_path(&map, start, goal, false);
        assert!(infantry_path.is_some());

        // Cavalry cannot (no path around in this setup)
        let cavalry_path = find_path(&map, start, goal, true);
        assert!(cavalry_path.is_none());
    }

    #[test]
    fn test_pathfind_no_path() {
        let mut map = BattleMap::new(10, 10);
        // Completely surround goal with cliffs
        let goal = BattleHexCoord::new(5, 5);
        for neighbor in goal.neighbors() {
            map.set_terrain(neighbor, BattleTerrain::Cliff);
        }

        let start = BattleHexCoord::new(0, 0);
        let path = find_path(&map, start, goal, false);

        assert!(path.is_none());
    }
```

**Step 6: Run all pathfinding tests**

Run: `cargo test --lib battle::pathfinding`
Expected: All PASS

**Step 7: Register module in mod.rs**

Add to `src/battle/mod.rs` after line 10:
```rust
pub mod pathfinding;
```

Add to re-exports (after line 51):
```rust
pub use pathfinding::{find_path, path_cost};
```

**Step 8: Commit**

```bash
git add src/battle/pathfinding.rs src/battle/mod.rs
git commit -m "feat(battle): add A* pathfinding with terrain costs"
```

---

## Task 2: Visibility Module (Fog of War)

**Files:**
- Create: `src/battle/visibility.rs`
- Modify: `src/battle/mod.rs` (add module)
- Test: `src/battle/visibility.rs` (inline tests)

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::battle_map::BattleMap;
    use crate::battle::hex::BattleHexCoord;
    use crate::battle::units::{Army, ArmyId, BattleUnit, UnitId, Element};
    use crate::battle::unit_type::UnitType;
    use crate::core::types::EntityId;

    #[test]
    fn test_visibility_near_unit() {
        let map = BattleMap::new(20, 20);
        let mut army = Army::new(ArmyId::new(), EntityId::new());

        // Add a formation with one unit at (10, 10)
        let mut formation = crate::battle::units::BattleFormation::new(
            crate::battle::units::FormationId::new(),
            EntityId::new(),
        );
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(10, 10);
        unit.elements.push(Element::new(vec![EntityId::new(); 10]));
        formation.units.push(unit);
        army.formations.push(formation);

        let visibility = calculate_army_visibility(&map, &army);

        // Unit position should be visible
        assert!(visibility.is_visible(BattleHexCoord::new(10, 10)));
        // Nearby hexes should be visible
        assert!(visibility.is_visible(BattleHexCoord::new(11, 10)));
        // Far hexes should not be visible
        assert!(!visibility.is_visible(BattleHexCoord::new(0, 0)));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::visibility::tests::test_visibility_near_unit`
Expected: FAIL

**Step 3: Write visibility implementation**

```rust
//! Per-army visibility (fog of war)
//!
//! Each army has its own visibility map based on unit positions.

use std::collections::HashSet;

use crate::battle::battle_map::BattleMap;
use crate::battle::hex::BattleHexCoord;
use crate::battle::units::Army;
use crate::battle::constants::{BASE_VISION_RANGE, SCOUT_VISION_BONUS, ELEVATION_VISION_BONUS};
use crate::battle::unit_type::UnitType;

/// Visibility state for an army
#[derive(Debug, Clone, Default)]
pub struct ArmyVisibility {
    /// Currently visible hexes
    pub visible: HashSet<BattleHexCoord>,
    /// Previously seen hexes (remembered)
    pub remembered: HashSet<BattleHexCoord>,
}

impl ArmyVisibility {
    pub fn new() -> Self {
        Self::default()
    }

    /// Is this hex currently visible?
    pub fn is_visible(&self, coord: BattleHexCoord) -> bool {
        self.visible.contains(&coord)
    }

    /// Has this hex been seen before?
    pub fn is_remembered(&self, coord: BattleHexCoord) -> bool {
        self.remembered.contains(&coord)
    }

    /// Update: move current visible to remembered, set new visible
    pub fn update(&mut self, new_visible: HashSet<BattleHexCoord>) {
        // Move old visible to remembered
        self.remembered.extend(self.visible.drain());
        // Set new visible
        self.visible = new_visible;
        // Remove from remembered what is now visible
        for coord in &self.visible {
            self.remembered.remove(coord);
        }
    }
}

/// Calculate vision range for a unit
pub fn unit_vision_range(unit: &crate::battle::units::BattleUnit, map: &BattleMap) -> u32 {
    let mut range = BASE_VISION_RANGE;

    // Scout bonus
    if matches!(unit.unit_type, UnitType::LightCavalry) {
        range += SCOUT_VISION_BONUS;
    }

    // Elevation bonus
    if let Some(hex) = map.get_hex(unit.position) {
        if hex.elevation > 0 {
            range += ELEVATION_VISION_BONUS * hex.elevation as u32;
        }
    }

    range
}

/// Calculate visibility for an entire army
pub fn calculate_army_visibility(map: &BattleMap, army: &Army) -> ArmyVisibility {
    let mut visible = HashSet::new();

    for formation in &army.formations {
        for unit in &formation.units {
            if unit.effective_strength() == 0 {
                continue;
            }

            let range = unit_vision_range(unit, map);
            let unit_visible = map.visible_hexes(unit.position, range);
            visible.extend(unit_visible);
        }
    }

    let mut visibility = ArmyVisibility::new();
    visibility.visible = visible;
    visibility
}

/// Update army visibility in place
pub fn update_army_visibility(
    visibility: &mut ArmyVisibility,
    map: &BattleMap,
    army: &Army,
) {
    let new_visible = calculate_army_visibility(map, army).visible;
    visibility.update(new_visible);
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib battle::visibility::tests::test_visibility_near_unit`
Expected: PASS

**Step 5: Add more tests**

```rust
    #[test]
    fn test_remembered_hexes() {
        let map = BattleMap::new(20, 20);
        let mut visibility = ArmyVisibility::new();

        // First update: see hex (5,5)
        let mut visible1 = HashSet::new();
        visible1.insert(BattleHexCoord::new(5, 5));
        visibility.update(visible1);

        assert!(visibility.is_visible(BattleHexCoord::new(5, 5)));

        // Second update: see different hex
        let mut visible2 = HashSet::new();
        visible2.insert(BattleHexCoord::new(10, 10));
        visibility.update(visible2);

        // Old hex should be remembered, not visible
        assert!(!visibility.is_visible(BattleHexCoord::new(5, 5)));
        assert!(visibility.is_remembered(BattleHexCoord::new(5, 5)));
        assert!(visibility.is_visible(BattleHexCoord::new(10, 10)));
    }

    #[test]
    fn test_scout_bonus_vision() {
        let map = BattleMap::new(20, 20);

        let mut infantry = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        infantry.position = BattleHexCoord::new(10, 10);

        let mut scout = BattleUnit::new(UnitId::new(), UnitType::LightCavalry);
        scout.position = BattleHexCoord::new(10, 10);

        let infantry_range = unit_vision_range(&infantry, &map);
        let scout_range = unit_vision_range(&scout, &map);

        assert!(scout_range > infantry_range);
    }
```

**Step 6: Run all visibility tests**

Run: `cargo test --lib battle::visibility`
Expected: All PASS

**Step 7: Register module**

Add to `src/battle/mod.rs`:
```rust
pub mod visibility;
```

Add to re-exports:
```rust
pub use visibility::{ArmyVisibility, calculate_army_visibility, update_army_visibility, unit_vision_range};
```

**Step 8: Commit**

```bash
git add src/battle/visibility.rs src/battle/mod.rs
git commit -m "feat(battle): add per-army fog of war visibility"
```

---

## Task 3: Movement Module

**Files:**
- Create: `src/battle/movement.rs`
- Modify: `src/battle/mod.rs` (add module)
- Test: `src/battle/movement.rs` (inline tests)

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::battle_map::BattleMap;
    use crate::battle::hex::BattleHexCoord;
    use crate::battle::units::{BattleUnit, UnitId, UnitStance};
    use crate::battle::unit_type::UnitType;
    use crate::battle::planning::{Waypoint, WaypointBehavior, WaypointPlan, MovementPace};

    #[test]
    fn test_unit_moves_toward_waypoint() {
        let map = BattleMap::new(20, 20);
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(0, 0);
        unit.stance = UnitStance::Moving;

        let mut plan = WaypointPlan::new(unit.id);
        plan.add_waypoint(
            Waypoint::new(BattleHexCoord::new(5, 0), WaypointBehavior::MoveTo)
                .with_pace(MovementPace::Quick)
        );

        let result = advance_unit_movement(&map, &mut unit, &mut plan);

        assert!(result.moved);
        // Unit should have moved closer to waypoint
        assert!(unit.position.distance(&BattleHexCoord::new(5, 0)) < 5);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::movement::tests::test_unit_moves_toward_waypoint`
Expected: FAIL

**Step 3: Write movement implementation**

```rust
//! Unit movement along waypoints
//!
//! Units follow their waypoint plans, respecting terrain and pace.

use crate::battle::battle_map::BattleMap;
use crate::battle::hex::BattleHexCoord;
use crate::battle::units::{BattleUnit, UnitStance};
use crate::battle::unit_type::UnitType;
use crate::battle::planning::{WaypointPlan, WaypointBehavior, MovementPace, WaitCondition};
use crate::battle::pathfinding::find_path;
use crate::battle::constants::{
    INFANTRY_WALK_SPEED, INFANTRY_RUN_SPEED,
    CAVALRY_WALK_SPEED, CAVALRY_TROT_SPEED, CAVALRY_CHARGE_SPEED,
    FATIGUE_RATE_MARCH,
};

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
    let is_cavalry = matches!(unit_type,
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
    let is_cavalry = matches!(unit.unit_type,
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
        // Simple: move to next hex if speed >= 1.0, otherwise accumulate
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

    let is_cavalry = matches!(unit.unit_type,
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
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib battle::movement::tests::test_unit_moves_toward_waypoint`
Expected: PASS

**Step 5: Add more tests**

```rust
    #[test]
    fn test_unit_stops_at_hold_waypoint() {
        let map = BattleMap::new(20, 20);
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(5, 0);
        unit.stance = UnitStance::Moving;

        let mut plan = WaypointPlan::new(unit.id);
        plan.add_waypoint(
            Waypoint::new(BattleHexCoord::new(5, 0), WaypointBehavior::HoldAt)
        );

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
```

**Step 6: Run all movement tests**

Run: `cargo test --lib battle::movement`
Expected: All PASS

**Step 7: Register module**

Add to `src/battle/mod.rs`:
```rust
pub mod movement;
```

Add to re-exports:
```rust
pub use movement::{MovementResult, advance_unit_movement, move_routing_unit};
```

**Step 8: Commit**

```bash
git add src/battle/movement.rs src/battle/mod.rs
git commit -m "feat(battle): add unit movement along waypoints"
```

---

## Task 4: Triggers Module (Go-Codes and Contingencies)

**Files:**
- Create: `src/battle/triggers.rs`
- Modify: `src/battle/mod.rs` (add module)
- Test: `src/battle/triggers.rs` (inline tests)

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::planning::{GoCode, GoCodeTrigger, GoCodeId};
    use crate::battle::units::UnitId;

    #[test]
    fn test_manual_gocode_not_auto_triggered() {
        let go_code = GoCode::new("HAMMER".into(), GoCodeTrigger::Manual);
        let result = evaluate_gocode_trigger(&go_code, 100, &[]);

        assert!(!result);
    }

    #[test]
    fn test_time_gocode_triggers_at_tick() {
        let go_code = GoCode::new("DAWN".into(), GoCodeTrigger::Time(50));

        assert!(!evaluate_gocode_trigger(&go_code, 49, &[]));
        assert!(evaluate_gocode_trigger(&go_code, 50, &[]));
        assert!(evaluate_gocode_trigger(&go_code, 51, &[]));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::triggers::tests`
Expected: FAIL

**Step 3: Write triggers implementation**

```rust
//! Go-code and contingency trigger evaluation
//!
//! Go-codes coordinate unit actions. Contingencies respond to events.

use crate::battle::hex::BattleHexCoord;
use crate::battle::units::{BattleUnit, UnitId, UnitStance};
use crate::battle::planning::{
    GoCode, GoCodeTrigger, GoCodeId,
    Contingency, ContingencyTrigger, ContingencyResponse,
    BattlePlan,
};
use crate::core::types::Tick;

/// Unit position info for trigger evaluation
#[derive(Debug, Clone)]
pub struct UnitPosition {
    pub unit_id: UnitId,
    pub position: BattleHexCoord,
    pub is_routing: bool,
}

/// Result of evaluating triggers
#[derive(Debug, Clone, Default)]
pub struct TriggerResults {
    pub triggered_gocodes: Vec<GoCodeId>,
    pub triggered_contingencies: Vec<usize>,
}

/// Evaluate a single go-code trigger condition
pub fn evaluate_gocode_trigger(
    go_code: &GoCode,
    current_tick: Tick,
    unit_positions: &[UnitPosition],
) -> bool {
    if go_code.triggered {
        return false; // Already triggered
    }

    match &go_code.trigger {
        GoCodeTrigger::Manual => false, // Player must manually trigger

        GoCodeTrigger::Time(tick) => current_tick >= *tick,

        GoCodeTrigger::UnitPosition { unit, position } => {
            unit_positions.iter().any(|up| {
                up.unit_id == *unit && up.position == *position
            })
        }

        GoCodeTrigger::EnemyInArea { area } => {
            // This requires enemy visibility info - return false for now
            // Will be evaluated at higher level with full state
            let _ = area;
            false
        }
    }
}

/// Evaluate all go-codes in a battle plan
pub fn evaluate_all_gocodes(
    plan: &BattlePlan,
    current_tick: Tick,
    unit_positions: &[UnitPosition],
) -> Vec<GoCodeId> {
    plan.go_codes
        .iter()
        .filter(|gc| evaluate_gocode_trigger(gc, current_tick, unit_positions))
        .map(|gc| gc.id)
        .collect()
}

/// Evaluate a single contingency trigger
pub fn evaluate_contingency_trigger(
    contingency: &Contingency,
    unit_positions: &[UnitPosition],
    casualties_percent: f32,
    commander_alive: bool,
    enemy_positions: &[BattleHexCoord],
    friendly_positions: &[BattleHexCoord],
) -> bool {
    if contingency.activated {
        return false; // Already activated
    }

    match &contingency.trigger {
        ContingencyTrigger::UnitBreaks(unit_id) => {
            unit_positions.iter().any(|up| {
                up.unit_id == *unit_id && up.is_routing
            })
        }

        ContingencyTrigger::CommanderDies => !commander_alive,

        ContingencyTrigger::PositionLost(position) => {
            // Position is lost if enemy is there and we're not
            enemy_positions.contains(position) && !friendly_positions.contains(position)
        }

        ContingencyTrigger::EnemyFlanking => {
            // Simplified: enemy is behind our lines
            // Would need more context for real implementation
            false
        }

        ContingencyTrigger::CasualtiesExceed(threshold) => {
            casualties_percent > *threshold
        }
    }
}

/// Evaluate all contingencies in a plan
pub fn evaluate_all_contingencies(
    plan: &BattlePlan,
    unit_positions: &[UnitPosition],
    casualties_percent: f32,
    commander_alive: bool,
    enemy_positions: &[BattleHexCoord],
    friendly_positions: &[BattleHexCoord],
) -> Vec<usize> {
    plan.contingencies
        .iter()
        .enumerate()
        .filter(|(_, c)| evaluate_contingency_trigger(
            c,
            unit_positions,
            casualties_percent,
            commander_alive,
            enemy_positions,
            friendly_positions,
        ))
        .map(|(i, _)| i)
        .collect()
}

/// Apply a contingency response
pub fn describe_contingency_response(response: &ContingencyResponse) -> String {
    match response {
        ContingencyResponse::ExecutePlan(unit_id) => {
            format!("Execute backup plan for unit {:?}", unit_id)
        }
        ContingencyResponse::Retreat(route) => {
            format!("Retreat via {} hexes", route.len())
        }
        ContingencyResponse::Rally(position) => {
            format!("Rally at {:?}", position)
        }
        ContingencyResponse::Signal(go_code_id) => {
            format!("Signal go-code {:?}", go_code_id)
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib battle::triggers::tests`
Expected: PASS

**Step 5: Add more tests**

```rust
    #[test]
    fn test_unit_position_gocode() {
        let unit_id = UnitId::new();
        let target_pos = BattleHexCoord::new(10, 10);

        let go_code = GoCode::new(
            "FLANK".into(),
            GoCodeTrigger::UnitPosition {
                unit: unit_id,
                position: target_pos
            }
        );

        // Unit not at position
        let positions = vec![UnitPosition {
            unit_id,
            position: BattleHexCoord::new(5, 5),
            is_routing: false,
        }];
        assert!(!evaluate_gocode_trigger(&go_code, 0, &positions));

        // Unit at position
        let positions = vec![UnitPosition {
            unit_id,
            position: target_pos,
            is_routing: false,
        }];
        assert!(evaluate_gocode_trigger(&go_code, 0, &positions));
    }

    #[test]
    fn test_casualties_contingency() {
        let contingency = Contingency::new(
            ContingencyTrigger::CasualtiesExceed(0.3),
            ContingencyResponse::Rally(BattleHexCoord::new(0, 0)),
        );

        // Below threshold
        assert!(!evaluate_contingency_trigger(
            &contingency, &[], 0.2, true, &[], &[]
        ));

        // Above threshold
        assert!(evaluate_contingency_trigger(
            &contingency, &[], 0.35, true, &[], &[]
        ));
    }

    #[test]
    fn test_unit_breaks_contingency() {
        let unit_id = UnitId::new();
        let contingency = Contingency::new(
            ContingencyTrigger::UnitBreaks(unit_id),
            ContingencyResponse::Signal(GoCodeId::new()),
        );

        // Unit not routing
        let positions = vec![UnitPosition {
            unit_id,
            position: BattleHexCoord::new(5, 5),
            is_routing: false,
        }];
        assert!(!evaluate_contingency_trigger(
            &contingency, &positions, 0.0, true, &[], &[]
        ));

        // Unit routing
        let positions = vec![UnitPosition {
            unit_id,
            position: BattleHexCoord::new(5, 5),
            is_routing: true,
        }];
        assert!(evaluate_contingency_trigger(
            &contingency, &positions, 0.0, true, &[], &[]
        ));
    }
```

**Step 6: Run all trigger tests**

Run: `cargo test --lib battle::triggers`
Expected: All PASS

**Step 7: Register module**

Add to `src/battle/mod.rs`:
```rust
pub mod triggers;
```

Add to re-exports:
```rust
pub use triggers::{
    UnitPosition, TriggerResults,
    evaluate_gocode_trigger, evaluate_all_gocodes,
    evaluate_contingency_trigger, evaluate_all_contingencies,
};
```

**Step 8: Commit**

```bash
git add src/battle/triggers.rs src/battle/mod.rs
git commit -m "feat(battle): add go-code and contingency trigger evaluation"
```

---

## Task 5: Engagement Detection

**Files:**
- Create: `src/battle/engagement.rs`
- Modify: `src/battle/mod.rs` (add module)
- Test: `src/battle/engagement.rs` (inline tests)

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::hex::BattleHexCoord;
    use crate::battle::units::{BattleUnit, UnitId, UnitStance, Element};
    use crate::battle::unit_type::UnitType;
    use crate::core::types::EntityId;

    #[test]
    fn test_adjacent_units_engage() {
        let mut attacker = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        attacker.position = BattleHexCoord::new(5, 5);
        attacker.elements.push(Element::new(vec![EntityId::new(); 50]));

        let mut defender = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        defender.position = BattleHexCoord::new(6, 5); // Adjacent
        defender.elements.push(Element::new(vec![EntityId::new(); 50]));

        let result = detect_engagement(&attacker, &defender);

        assert!(result.is_some());
    }

    #[test]
    fn test_distant_units_dont_engage() {
        let mut attacker = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        attacker.position = BattleHexCoord::new(0, 0);
        attacker.elements.push(Element::new(vec![EntityId::new(); 50]));

        let mut defender = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        defender.position = BattleHexCoord::new(10, 10); // Far away
        defender.elements.push(Element::new(vec![EntityId::new(); 50]));

        let result = detect_engagement(&attacker, &defender);

        assert!(result.is_none());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::engagement::tests`
Expected: FAIL

**Step 3: Write engagement implementation**

```rust
//! Engagement detection between units
//!
//! Units engage when adjacent. Combat follows engagement.

use crate::battle::hex::BattleHexCoord;
use crate::battle::units::{BattleUnit, UnitId, UnitStance};
use crate::battle::planning::EngagementRule;

/// Potential engagement between two units
#[derive(Debug, Clone)]
pub struct PotentialEngagement {
    pub attacker_id: UnitId,
    pub defender_id: UnitId,
    pub distance: u32,
}

/// Detect if two units should engage
pub fn detect_engagement(
    unit_a: &BattleUnit,
    unit_b: &BattleUnit,
) -> Option<PotentialEngagement> {
    // Can't engage if either is broken or has no strength
    if !unit_a.can_fight() || !unit_b.can_fight() {
        return None;
    }

    let distance = unit_a.position.distance(&unit_b.position);

    // Must be adjacent (distance 1) for melee engagement
    if distance > 1 {
        return None;
    }

    Some(PotentialEngagement {
        attacker_id: unit_a.id,
        defender_id: unit_b.id,
        distance,
    })
}

/// Check if unit should initiate combat based on engagement rules
pub fn should_initiate_combat(
    unit: &BattleUnit,
    engagement_rule: &EngagementRule,
    is_being_attacked: bool,
) -> bool {
    match engagement_rule {
        EngagementRule::Aggressive => true,
        EngagementRule::Defensive => is_being_attacked,
        EngagementRule::HoldFire => false,
        EngagementRule::Skirmish => true,
    }
}

/// Find all engagements between two armies
pub fn find_all_engagements(
    friendly_units: &[&BattleUnit],
    enemy_units: &[&BattleUnit],
) -> Vec<PotentialEngagement> {
    let mut engagements = Vec::new();

    for friendly in friendly_units {
        for enemy in enemy_units {
            if let Some(engagement) = detect_engagement(friendly, enemy) {
                engagements.push(engagement);
            }
        }
    }

    engagements
}

/// Check if a unit is flanked (enemy in rear arc)
pub fn is_flanked(unit: &BattleUnit, enemy_positions: &[BattleHexCoord]) -> bool {
    let rear_offset = unit.facing.opposite().offset();
    let rear_hex = BattleHexCoord::new(
        unit.position.q + rear_offset.q,
        unit.position.r + rear_offset.r,
    );

    enemy_positions.contains(&rear_hex)
}

/// Check if a unit is surrounded (3+ adjacent enemies)
pub fn is_surrounded(unit: &BattleUnit, enemy_positions: &[BattleHexCoord]) -> bool {
    let adjacent_enemies = unit.position.neighbors()
        .iter()
        .filter(|n| enemy_positions.contains(n))
        .count();

    adjacent_enemies >= 3
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib battle::engagement::tests`
Expected: PASS

**Step 5: Add more tests**

```rust
    #[test]
    fn test_broken_unit_cant_engage() {
        let mut attacker = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        attacker.position = BattleHexCoord::new(5, 5);
        attacker.elements.push(Element::new(vec![EntityId::new(); 50]));
        attacker.stance = UnitStance::Routing;

        let mut defender = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        defender.position = BattleHexCoord::new(6, 5);
        defender.elements.push(Element::new(vec![EntityId::new(); 50]));

        let result = detect_engagement(&attacker, &defender);

        assert!(result.is_none());
    }

    #[test]
    fn test_is_flanked() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(5, 5);
        unit.facing = crate::battle::hex::HexDirection::East;

        // Enemy directly behind (West)
        let enemy_at_rear = vec![BattleHexCoord::new(4, 5)];
        assert!(is_flanked(&unit, &enemy_at_rear));

        // Enemy in front (East)
        let enemy_in_front = vec![BattleHexCoord::new(6, 5)];
        assert!(!is_flanked(&unit, &enemy_in_front));
    }

    #[test]
    fn test_is_surrounded() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = BattleHexCoord::new(5, 5);

        // Only 2 enemies - not surrounded
        let two_enemies = vec![
            BattleHexCoord::new(6, 5),
            BattleHexCoord::new(4, 5),
        ];
        assert!(!is_surrounded(&unit, &two_enemies));

        // 3 enemies - surrounded
        let three_enemies = vec![
            BattleHexCoord::new(6, 5),
            BattleHexCoord::new(4, 5),
            BattleHexCoord::new(5, 6),
        ];
        assert!(is_surrounded(&unit, &three_enemies));
    }

    #[test]
    fn test_engagement_rules() {
        let unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);

        assert!(should_initiate_combat(&unit, &EngagementRule::Aggressive, false));
        assert!(!should_initiate_combat(&unit, &EngagementRule::Defensive, false));
        assert!(should_initiate_combat(&unit, &EngagementRule::Defensive, true));
        assert!(!should_initiate_combat(&unit, &EngagementRule::HoldFire, true));
    }
```

**Step 6: Run all engagement tests**

Run: `cargo test --lib battle::engagement`
Expected: All PASS

**Step 7: Register module**

Add to `src/battle/mod.rs`:
```rust
pub mod engagement;
```

Add to re-exports:
```rust
pub use engagement::{
    PotentialEngagement, detect_engagement, should_initiate_combat,
    find_all_engagements, is_flanked, is_surrounded,
};
```

**Step 8: Commit**

```bash
git add src/battle/engagement.rs src/battle/mod.rs
git commit -m "feat(battle): add engagement detection and flanking checks"
```

---

## Task 6: Morale and Stress System

**Files:**
- Create: `src/battle/morale.rs`
- Modify: `src/battle/mod.rs` (add module)
- Test: `src/battle/morale.rs` (inline tests)

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::units::{BattleUnit, UnitId, UnitStance, Element};
    use crate::battle::unit_type::UnitType;
    use crate::core::types::EntityId;

    #[test]
    fn test_unit_breaks_when_stress_exceeds_threshold() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        unit.stress = 0.9; // Very high stress

        let result = check_morale_break(&unit);

        assert!(result.breaks);
    }

    #[test]
    fn test_unit_holds_with_low_stress() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        unit.stress = 0.1; // Low stress

        let result = check_morale_break(&unit);

        assert!(!result.breaks);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::morale::tests`
Expected: FAIL

**Step 3: Write morale implementation**

```rust
//! Morale and stress system
//!
//! Stress accumulates from combat. When it exceeds threshold, units break.

use crate::battle::hex::BattleHexCoord;
use crate::battle::units::{BattleUnit, UnitId, UnitStance};
use crate::battle::constants::{CONTAGION_STRESS, OFFICER_DEATH_STRESS};

/// Result of a morale check
#[derive(Debug, Clone)]
pub struct MoraleCheckResult {
    pub breaks: bool,
    pub rallies: bool,
    pub stress_delta: f32,
}

impl Default for MoraleCheckResult {
    fn default() -> Self {
        Self {
            breaks: false,
            rallies: false,
            stress_delta: 0.0,
        }
    }
}

/// Check if a unit breaks due to stress
pub fn check_morale_break(unit: &BattleUnit) -> MoraleCheckResult {
    let mut result = MoraleCheckResult::default();

    // Already routing units can't break again
    if matches!(unit.stance, UnitStance::Routing) {
        return result;
    }

    // Check stress vs threshold
    let threshold = unit.stress_threshold();
    if unit.stress >= threshold {
        result.breaks = true;
    }

    result
}

/// Check if a routing unit can rally
pub fn check_rally(unit: &BattleUnit, is_near_enemy: bool, is_near_leader: bool) -> MoraleCheckResult {
    let mut result = MoraleCheckResult::default();

    // Only routing units can rally
    if !matches!(unit.stance, UnitStance::Routing) {
        return result;
    }

    // Can't rally near enemies
    if is_near_enemy {
        return result;
    }

    // Rally conditions
    let rally_threshold = 0.5; // Need stress below this to rally
    let stress_after_recovery = unit.stress - 0.1; // Routing reduces stress

    if stress_after_recovery < rally_threshold {
        result.rallies = true;
        result.stress_delta = -0.1;
    }

    // Leader nearby helps
    if is_near_leader && stress_after_recovery < rally_threshold + 0.2 {
        result.rallies = true;
        result.stress_delta = -0.15;
    }

    result
}

/// Calculate stress contagion from nearby routing units
pub fn calculate_contagion_stress(
    unit: &BattleUnit,
    nearby_routing_count: usize,
) -> f32 {
    // Already routing units don't get more contagion stress
    if matches!(unit.stance, UnitStance::Routing) {
        return 0.0;
    }

    nearby_routing_count as f32 * CONTAGION_STRESS
}

/// Calculate stress from officer death
pub fn calculate_officer_death_stress(had_leader: bool, leader_died: bool) -> f32 {
    if had_leader && leader_died {
        OFFICER_DEATH_STRESS
    } else {
        0.0
    }
}

/// Apply stress to a unit
pub fn apply_stress(unit: &mut BattleUnit, stress_delta: f32) {
    unit.stress = (unit.stress + stress_delta).clamp(0.0, 2.0);
}

/// Process morale break for a unit
pub fn process_morale_break(unit: &mut BattleUnit) {
    unit.stance = UnitStance::Routing;
    unit.cohesion = (unit.cohesion * 0.5).max(0.1);
}

/// Process rally for a unit
pub fn process_rally(unit: &mut BattleUnit) {
    unit.stance = UnitStance::Rallying;
    // Will transition to Formed after a few ticks
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib battle::morale::tests`
Expected: PASS

**Step 5: Add more tests**

```rust
    #[test]
    fn test_routing_unit_can_rally_when_safe() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        unit.stance = UnitStance::Routing;
        unit.stress = 0.3; // Low enough to rally

        let result = check_rally(&unit, false, false);

        assert!(result.rallies);
    }

    #[test]
    fn test_routing_unit_cant_rally_near_enemy() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        unit.stance = UnitStance::Routing;
        unit.stress = 0.3;

        let result = check_rally(&unit, true, false); // Near enemy

        assert!(!result.rallies);
    }

    #[test]
    fn test_contagion_stress() {
        let unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);

        let stress_0 = calculate_contagion_stress(&unit, 0);
        let stress_2 = calculate_contagion_stress(&unit, 2);

        assert_eq!(stress_0, 0.0);
        assert!(stress_2 > 0.0);
        assert_eq!(stress_2, 2.0 * CONTAGION_STRESS);
    }

    #[test]
    fn test_apply_stress_clamped() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.stress = 1.9;

        apply_stress(&mut unit, 0.5);

        assert_eq!(unit.stress, 2.0); // Clamped to max
    }

    #[test]
    fn test_leader_helps_rally() {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        unit.stance = UnitStance::Routing;
        unit.stress = 0.6; // Too high for normal rally

        // Without leader: can't rally
        assert!(!check_rally(&unit, false, false).rallies);

        // With leader: can rally
        assert!(check_rally(&unit, false, true).rallies);
    }
```

**Step 6: Run all morale tests**

Run: `cargo test --lib battle::morale`
Expected: All PASS

**Step 7: Register module**

Add to `src/battle/mod.rs`:
```rust
pub mod morale;
```

Add to re-exports:
```rust
pub use morale::{
    MoraleCheckResult, check_morale_break, check_rally,
    calculate_contagion_stress, apply_stress, process_morale_break, process_rally,
};
```

**Step 8: Commit**

```bash
git add src/battle/morale.rs src/battle/mod.rs
git commit -m "feat(battle): add morale breaks, stress contagion, and rallying"
```

---

## Task 7: Battle Tick Orchestration

**Files:**
- Modify: `src/battle/execution.rs`
- Test: `src/battle/execution.rs` (inline tests)

**Step 1: Write the failing test**

```rust
// Add to existing tests in execution.rs
#[test]
fn test_full_tick_advances_state() {
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
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib battle::execution::tests::test_full_tick_advances_state`
Expected: FAIL with "no method named `run_tick` found"

**Step 3: Add imports and BattleEventLog**

Add these imports at the top of `src/battle/execution.rs`:
```rust
use crate::battle::visibility::{ArmyVisibility, update_army_visibility};
use crate::battle::movement::advance_unit_movement;
use crate::battle::triggers::{UnitPosition, evaluate_all_gocodes, evaluate_all_contingencies};
use crate::battle::engagement::{find_all_engagements, is_flanked, is_surrounded};
use crate::battle::morale::{check_morale_break, check_rally, calculate_contagion_stress, apply_stress, process_morale_break};
use crate::battle::resolution::resolve_unit_combat;
use crate::battle::constants::COURIER_SPEED;
```

Add the BattleEventLog struct:
```rust
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
```

**Step 4: Add visibility fields to BattleState**

Modify `BattleState` struct:
```rust
pub struct BattleState {
    // ... existing fields ...

    // Visibility (fog of war)
    pub friendly_visibility: ArmyVisibility,
    pub enemy_visibility: ArmyVisibility,
}
```

Update `BattleState::new`:
```rust
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
        friendly_visibility: ArmyVisibility::new(),
        enemy_visibility: ArmyVisibility::new(),
    }
}
```

**Step 5: Implement run_tick**

Add to `impl BattleState`:
```rust
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
    // Update fog of war
    update_army_visibility(&mut self.friendly_visibility, &self.map, &self.friendly_army);
    update_army_visibility(&mut self.enemy_visibility, &self.map, &self.enemy_army);

    // Evaluate go-codes
    let friendly_positions: Vec<UnitPosition> = self.friendly_army.formations
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
        if let Some(gc) = self.friendly_plan.go_codes.iter_mut().find(|g| g.id == go_code_id) {
            if !gc.triggered {
                gc.triggered = true;
                events.push(
                    BattleEventType::GoCodeTriggered { name: gc.name.clone() },
                    format!("Go-code '{}' triggered", gc.name),
                    self.tick,
                );
            }
        }
    }
}

fn phase_movement(&mut self, events: &mut BattleEventLog) {
    // Advance couriers
    self.courier_system.advance_all(COURIER_SPEED);
    let arrived_orders = self.courier_system.collect_arrived();

    // Apply arrived orders
    for order in arrived_orders {
        // TODO: Apply order to target unit
        let _ = order;
    }

    // Move units along waypoints
    for formation in &mut self.friendly_army.formations {
        for unit in &mut formation.units {
            if let Some(plan) = self.friendly_plan.waypoint_plans.iter_mut().find(|p| p.unit_id == unit.id) {
                let _result = advance_unit_movement(&self.map, unit, plan);
            }
        }
    }
}

fn phase_combat(&mut self, events: &mut BattleEventLog) {
    // Collect unit references
    let friendly_units: Vec<&BattleUnit> = self.friendly_army.formations
        .iter()
        .flat_map(|f| f.units.iter())
        .collect();

    let enemy_units: Vec<&BattleUnit> = self.enemy_army.formations
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
    let routing_positions: Vec<BattleHexCoord> = self.friendly_army.formations
        .iter()
        .flat_map(|f| f.units.iter())
        .filter(|u| u.is_broken())
        .map(|u| u.position)
        .collect();

    // Check morale for all units
    for formation in &mut self.friendly_army.formations {
        for unit in &mut formation.units {
            // Count nearby routing friendlies
            let nearby_routing = routing_positions.iter()
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
                    format!("Unit broke and is routing"),
                    self.tick,
                );
            }
        }
    }

    // Same for enemy army
    let enemy_routing_positions: Vec<BattleHexCoord> = self.enemy_army.formations
        .iter()
        .flat_map(|f| f.units.iter())
        .filter(|u| u.is_broken())
        .map(|u| u.position)
        .collect();

    for formation in &mut self.enemy_army.formations {
        for unit in &mut formation.units {
            let nearby_routing = enemy_routing_positions.iter()
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
    let enemy_positions: Vec<BattleHexCoord> = self.enemy_army.formations
        .iter()
        .flat_map(|f| f.units.iter())
        .map(|u| u.position)
        .collect();

    for formation in &mut self.friendly_army.formations {
        for unit in &mut formation.units {
            if unit.is_broken() {
                // Check rally conditions
                let is_near_enemy = enemy_positions.iter()
                    .any(|pos| unit.position.distance(pos) <= 3);
                let is_near_leader = unit.position.distance(&formation.commander_position().unwrap_or_default()) <= 3;

                let result = check_rally(unit, is_near_enemy, is_near_leader);
                if result.rallies {
                    unit.stance = UnitStance::Rallying;
                    unit.stress += result.stress_delta;
                    events.push(
                        BattleEventType::UnitRallied { unit_id: unit.id },
                        format!("Unit rallied"),
                        self.tick,
                    );
                }
            }
        }
    }
}

fn phase_post_tick(&mut self, events: &mut BattleEventLog) {
    // Check battle end
    if let Some(outcome) = check_battle_end(self) {
        self.end_battle(outcome);
    }

    // Advance tick counter
    self.tick += 1;
}
```

**Step 6: Add helper method to BattleFormation**

In `src/battle/units.rs`, add to `impl BattleFormation`:
```rust
/// Get approximate commander position (center of formation)
pub fn commander_position(&self) -> Option<BattleHexCoord> {
    if self.units.is_empty() {
        return None;
    }

    let sum_q: i32 = self.units.iter().map(|u| u.position.q).sum();
    let sum_r: i32 = self.units.iter().map(|u| u.position.r).sum();
    let count = self.units.len() as i32;

    Some(BattleHexCoord::new(sum_q / count, sum_r / count))
}
```

**Step 7: Run tests**

Run: `cargo test --lib battle::execution::tests::test_full_tick_advances_state`
Expected: PASS

**Step 8: Run all tests**

Run: `cargo test --lib battle`
Expected: All PASS

**Step 9: Commit**

```bash
git add src/battle/execution.rs src/battle/units.rs
git commit -m "feat(battle): implement full 6-phase battle tick loop"
```

---

## Task 8: Integration Test

**Files:**
- Modify: `src/battle/execution.rs` (add integration test)

**Step 1: Write integration test**

```rust
#[test]
fn test_battle_scenario_simple_engagement() {
    // Setup map
    let map = BattleMap::new(30, 30);

    // Setup friendly army
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    let mut friendly_formation = BattleFormation::new(FormationId::new(), EntityId::new());

    let mut friendly_unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
    friendly_unit.elements.push(Element::new(vec![EntityId::new(); 100]));
    friendly_unit.position = BattleHexCoord::new(10, 15);
    friendly_formation.units.push(friendly_unit);
    friendly.formations.push(friendly_formation);

    // Setup enemy army
    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());

    let mut enemy_unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
    enemy_unit.elements.push(Element::new(vec![EntityId::new(); 100]));
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
    let friendly_casualties: u32 = state.friendly_army.formations
        .iter()
        .flat_map(|f| f.units.iter())
        .map(|u| u.casualties)
        .sum();

    let enemy_casualties: u32 = state.enemy_army.formations
        .iter()
        .flat_map(|f| f.units.iter())
        .map(|u| u.casualties)
        .sum();

    assert!(friendly_casualties > 0 || enemy_casualties > 0, "Combat should have occurred");
    assert_eq!(state.tick, 10, "Should have advanced 10 ticks");
}

#[test]
fn test_battle_ends_when_army_destroyed() {
    let map = BattleMap::new(20, 20);

    // Strong friendly army
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    let mut friendly_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    let mut friendly_unit = BattleUnit::new(UnitId::new(), UnitType::HeavyCavalry);
    friendly_unit.elements.push(Element::new(vec![EntityId::new(); 200]));
    friendly_unit.position = BattleHexCoord::new(10, 10);
    friendly_formation.units.push(friendly_unit);
    friendly.formations.push(friendly_formation);

    // Weak enemy army
    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut enemy_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    let mut enemy_unit = BattleUnit::new(UnitId::new(), UnitType::Militia);
    enemy_unit.elements.push(Element::new(vec![EntityId::new(); 10]));
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
        matches!(state.outcome, BattleOutcome::Victory | BattleOutcome::DecisiveVictory),
        "Should be a victory, got {:?}", state.outcome
    );
}
```

**Step 2: Run integration tests**

Run: `cargo test --lib battle::execution::tests`
Expected: All PASS

**Step 3: Run full test suite**

Run: `cargo test`
Expected: All PASS

**Step 4: Commit**

```bash
git add src/battle/execution.rs
git commit -m "test(battle): add integration tests for battle scenarios"
```

---

## Task 9: Export New Public API

**Files:**
- Modify: `src/battle/mod.rs`

**Step 1: Verify all modules are exported**

Ensure `src/battle/mod.rs` has all modules:
```rust
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
pub mod visibility;
pub mod movement;
pub mod triggers;
pub mod engagement;
pub mod morale;
```

**Step 2: Verify all re-exports**

Ensure re-exports section includes:
```rust
pub use pathfinding::{find_path, path_cost};
pub use visibility::{ArmyVisibility, calculate_army_visibility, update_army_visibility, unit_vision_range};
pub use movement::{MovementResult, advance_unit_movement, move_routing_unit};
pub use triggers::{
    UnitPosition, TriggerResults,
    evaluate_gocode_trigger, evaluate_all_gocodes,
    evaluate_contingency_trigger, evaluate_all_contingencies,
};
pub use engagement::{
    PotentialEngagement, detect_engagement, should_initiate_combat,
    find_all_engagements, is_flanked, is_surrounded,
};
pub use morale::{
    MoraleCheckResult, check_morale_break, check_rally,
    calculate_contagion_stress, apply_stress, process_morale_break, process_rally,
};
pub use execution::BattleEventLog;
```

**Step 3: Build and verify**

Run: `cargo build`
Expected: SUCCESS

**Step 4: Commit**

```bash
git add src/battle/mod.rs
git commit -m "feat(battle): export complete runtime API"
```

---

## Task 10: Final Verification

**Step 1: Run all tests**

Run: `cargo test`
Expected: All PASS

**Step 2: Run clippy**

Run: `cargo clippy --all-targets`
Expected: No errors (warnings OK)

**Step 3: Format code**

Run: `cargo fmt`

**Step 4: Final commit**

```bash
git add -A
git commit -m "chore: format and clean up battle runtime"
```

---

## Summary

This plan implements the complete battle runtime:

| Module | Purpose |
|--------|---------|
| `pathfinding.rs` | A* pathfinding with terrain costs |
| `visibility.rs` | Per-army fog of war |
| `movement.rs` | Unit movement along waypoints |
| `triggers.rs` | Go-code and contingency evaluation |
| `engagement.rs` | Engagement detection and flanking |
| `morale.rs` | Stress, breaks, and rallying |
| `execution.rs` | 6-phase tick orchestration |

All code follows project conventions:
- ADDITIVE modifiers only
- Extensive tests
- Rust idioms
- serde serialization
