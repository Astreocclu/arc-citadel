//! Battle planning structures (waypoints, go-codes, contingencies)
//!
//! Plan like Rainbow Six - waypoints, triggers, and contingencies.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::battle::hex::{BattleHexCoord, HexDirection};
use crate::battle::units::{UnitId, UnitStance};
use crate::core::types::Tick;

/// Unique identifier for go-codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GoCodeId(pub Uuid);

impl GoCodeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for GoCodeId {
    fn default() -> Self {
        Self::new()
    }
}

/// Movement pace
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MovementPace {
    Walk, // Preserve stamina
    #[default]
    Quick, // Faster, some fatigue
    Run,  // Fast, tiring
    Charge, // Maximum speed, exhausting, triggers shock
}

impl MovementPace {
    /// Speed multiplier
    pub fn speed_multiplier(&self) -> f32 {
        match self {
            MovementPace::Walk => 0.5,
            MovementPace::Quick => 1.0,
            MovementPace::Run => 1.5,
            MovementPace::Charge => 2.0,
        }
    }

    /// Fatigue rate multiplier
    pub fn fatigue_multiplier(&self) -> f32 {
        match self {
            MovementPace::Walk => 0.5,
            MovementPace::Quick => 1.0,
            MovementPace::Run => 2.0,
            MovementPace::Charge => 4.0,
        }
    }
}

/// Waypoint behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum WaypointBehavior {
    #[default]
    MoveTo, // Just get there
    HoldAt,     // Stop and defend
    AttackFrom, // Assault from this position
    ScanFrom,   // Observe, report
    RallyAt,    // Reform here if broken
}

/// Condition to wait for at waypoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WaitCondition {
    Duration(u64),       // Wait for N ticks
    GoCode(GoCodeId),    // Wait for go-code
    UnitArrives(UnitId), // Wait for another unit
    EnemySighted,        // Wait until enemy seen
    Attacked,            // Wait until attacked
}

/// A waypoint in a movement plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    pub position: BattleHexCoord,
    pub behavior: WaypointBehavior,
    pub pace: MovementPace,
    pub wait_condition: Option<WaitCondition>,
}

impl Waypoint {
    pub fn new(position: BattleHexCoord, behavior: WaypointBehavior) -> Self {
        Self {
            position,
            behavior,
            pace: MovementPace::default(),
            wait_condition: None,
        }
    }

    pub fn with_pace(mut self, pace: MovementPace) -> Self {
        self.pace = pace;
        self
    }

    pub fn with_wait(mut self, condition: WaitCondition) -> Self {
        self.wait_condition = Some(condition);
        self
    }
}

/// Waypoint plan for a unit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaypointPlan {
    pub unit_id: UnitId,
    pub waypoints: Vec<Waypoint>,
    pub current_waypoint: usize,
    pub wait_start_tick: Option<Tick>,  // When waiting started
}

impl WaypointPlan {
    pub fn new(unit_id: UnitId) -> Self {
        Self {
            unit_id,
            waypoints: Vec::new(),
            current_waypoint: 0,
            wait_start_tick: None,
        }
    }

    pub fn add_waypoint(&mut self, waypoint: Waypoint) {
        self.waypoints.push(waypoint);
    }

    pub fn current(&self) -> Option<&Waypoint> {
        self.waypoints.get(self.current_waypoint)
    }

    pub fn advance(&mut self) -> bool {
        if self.current_waypoint < self.waypoints.len().saturating_sub(1) {
            self.current_waypoint += 1;
            true
        } else {
            false
        }
    }
}

/// Engagement rules for a unit
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum EngagementRule {
    #[default]
    Aggressive, // Attack enemies on sight
    Defensive, // Only attack if attacked first
    HoldFire,  // No attacking unless directly ordered
    Skirmish,  // Engage then withdraw
}

impl EngagementRule {
    pub fn should_attack_on_sight(&self) -> bool {
        matches!(self, EngagementRule::Aggressive)
    }

    pub fn should_withdraw_after_engagement(&self) -> bool {
        matches!(self, EngagementRule::Skirmish)
    }
}

/// Go-code trigger condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoCodeTrigger {
    Manual,     // Player activates
    Time(Tick), // At specific tick
    UnitPosition {
        unit: UnitId,
        position: BattleHexCoord,
    },
    EnemyInArea {
        area: Vec<BattleHexCoord>,
    },
}

/// A go-code (coordinated trigger)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoCode {
    pub id: GoCodeId,
    pub name: String,
    pub trigger: GoCodeTrigger,
    pub subscribers: Vec<UnitId>,
    pub triggered: bool,
}

impl GoCode {
    pub fn new(name: String, trigger: GoCodeTrigger) -> Self {
        Self {
            id: GoCodeId::new(),
            name,
            trigger,
            subscribers: Vec::new(),
            triggered: false,
        }
    }

    pub fn subscribe(&mut self, unit_id: UnitId) {
        if !self.subscribers.contains(&unit_id) {
            self.subscribers.push(unit_id);
        }
    }
}

/// Contingency trigger condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContingencyTrigger {
    UnitBreaks(UnitId),
    CommanderDies,
    PositionLost(BattleHexCoord),
    EnemyFlanking,
    CasualtiesExceed(f32), // Percentage
}

/// Contingency response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContingencyResponse {
    ExecutePlan(UnitId),          // Execute unit's backup plan
    Retreat(Vec<BattleHexCoord>), // Retreat route
    Rally(BattleHexCoord),        // Rally point
    Signal(GoCodeId),             // Trigger a go-code
}

/// A contingency (pre-planned response)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contingency {
    pub trigger: ContingencyTrigger,
    pub response: ContingencyResponse,
    pub priority: u8,
    pub activated: bool,
}

impl Contingency {
    pub fn new(trigger: ContingencyTrigger, response: ContingencyResponse) -> Self {
        Self {
            trigger,
            response,
            priority: 0,
            activated: false,
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

/// Unit deployment in the plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitDeployment {
    pub unit_id: UnitId,
    pub position: BattleHexCoord,
    pub facing: HexDirection,
    pub initial_stance: UnitStance,
}

/// Complete battle plan
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BattlePlan {
    pub deployments: Vec<UnitDeployment>,
    pub waypoint_plans: Vec<WaypointPlan>,
    pub engagement_rules: Vec<(UnitId, EngagementRule)>,
    pub go_codes: Vec<GoCode>,
    pub contingencies: Vec<Contingency>,
}

impl BattlePlan {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get waypoint plan for a unit
    pub fn get_waypoint_plan(&self, unit_id: UnitId) -> Option<&WaypointPlan> {
        self.waypoint_plans.iter().find(|p| p.unit_id == unit_id)
    }

    /// Get engagement rule for a unit
    pub fn get_engagement_rule(&self, unit_id: UnitId) -> EngagementRule {
        self.engagement_rules
            .iter()
            .find(|(id, _)| *id == unit_id)
            .map(|(_, rule)| rule.clone())
            .unwrap_or_default()
    }

    /// Get go-code by name
    pub fn get_go_code(&self, name: &str) -> Option<&GoCode> {
        self.go_codes.iter().find(|g| g.name == name)
    }

    /// Get mutable go-code by name
    pub fn get_go_code_mut(&mut self, name: &str) -> Option<&mut GoCode> {
        self.go_codes.iter_mut().find(|g| g.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waypoint_creation() {
        let wp = Waypoint::new(BattleHexCoord::new(5, 5), WaypointBehavior::MoveTo);
        assert_eq!(wp.position, BattleHexCoord::new(5, 5));
    }

    #[test]
    fn test_go_code_creation() {
        let go_code = GoCode::new("HAMMER".into(), GoCodeTrigger::Manual);
        assert_eq!(go_code.name, "HAMMER");
    }

    #[test]
    fn test_battle_plan_add_deployment() {
        let mut plan = BattlePlan::new();
        let deployment = UnitDeployment {
            unit_id: UnitId::new(),
            position: BattleHexCoord::new(0, 0),
            facing: HexDirection::East,
            initial_stance: UnitStance::Formed,
        };
        plan.deployments.push(deployment);
        assert_eq!(plan.deployments.len(), 1);
    }

    #[test]
    fn test_engagement_rule_aggressive() {
        let rule = EngagementRule::Aggressive;
        assert!(rule.should_attack_on_sight());
    }

    #[test]
    fn test_waypoint_plan_advance() {
        let mut plan = WaypointPlan::new(UnitId::new());
        plan.add_waypoint(Waypoint::new(
            BattleHexCoord::new(0, 0),
            WaypointBehavior::MoveTo,
        ));
        plan.add_waypoint(Waypoint::new(
            BattleHexCoord::new(5, 5),
            WaypointBehavior::HoldAt,
        ));

        assert_eq!(plan.current_waypoint, 0);
        assert!(plan.advance());
        assert_eq!(plan.current_waypoint, 1);
        assert!(!plan.advance()); // Can't advance past last
    }

    #[test]
    fn test_go_code_subscribe() {
        let mut go_code = GoCode::new("TEST".into(), GoCodeTrigger::Manual);
        let unit_id = UnitId::new();

        go_code.subscribe(unit_id);
        assert_eq!(go_code.subscribers.len(), 1);

        // Subscribing again shouldn't duplicate
        go_code.subscribe(unit_id);
        assert_eq!(go_code.subscribers.len(), 1);
    }

    #[test]
    fn test_movement_pace_speed() {
        assert!(MovementPace::Charge.speed_multiplier() > MovementPace::Run.speed_multiplier());
        assert!(MovementPace::Run.speed_multiplier() > MovementPace::Quick.speed_multiplier());
    }

    #[test]
    fn test_waypoint_plan_has_wait_start_tick() {
        let plan = WaypointPlan::new(UnitId::new());
        assert!(plan.wait_start_tick.is_none());
    }
}
