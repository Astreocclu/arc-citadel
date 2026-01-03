//! Go-code and contingency trigger evaluation
//!
//! Go-codes coordinate unit actions. Contingencies respond to events.

use crate::battle::hex::BattleHexCoord;
use crate::battle::planning::{
    BattlePlan, Contingency, ContingencyResponse, ContingencyTrigger, GoCode, GoCodeId,
    GoCodeTrigger,
};
use crate::battle::units::UnitId;
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

        GoCodeTrigger::UnitPosition { unit, position } => unit_positions
            .iter()
            .any(|up| up.unit_id == *unit && up.position == *position),

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
        ContingencyTrigger::UnitBreaks(unit_id) => unit_positions
            .iter()
            .any(|up| up.unit_id == *unit_id && up.is_routing),

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

        ContingencyTrigger::CasualtiesExceed(threshold) => casualties_percent > *threshold,
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
        .filter(|(_, c)| {
            evaluate_contingency_trigger(
                c,
                unit_positions,
                casualties_percent,
                commander_alive,
                enemy_positions,
                friendly_positions,
            )
        })
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

    #[test]
    fn test_unit_position_gocode() {
        let unit_id = UnitId::new();
        let target_pos = BattleHexCoord::new(10, 10);

        let go_code = GoCode::new(
            "FLANK".into(),
            GoCodeTrigger::UnitPosition {
                unit: unit_id,
                position: target_pos,
            },
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
    fn test_already_triggered_gocode_returns_false() {
        let mut go_code = GoCode::new("TEST".into(), GoCodeTrigger::Time(10));
        go_code.triggered = true;

        // Even though time condition is met, it's already triggered
        assert!(!evaluate_gocode_trigger(&go_code, 100, &[]));
    }

    #[test]
    fn test_enemy_in_area_gocode_placeholder() {
        let go_code = GoCode::new(
            "AMBUSH".into(),
            GoCodeTrigger::EnemyInArea {
                area: vec![BattleHexCoord::new(5, 5)],
            },
        );

        // Currently returns false as it needs higher-level evaluation
        assert!(!evaluate_gocode_trigger(&go_code, 0, &[]));
    }

    #[test]
    fn test_evaluate_all_gocodes() {
        let mut plan = BattlePlan::new();
        plan.go_codes.push(GoCode::new("EARLY".into(), GoCodeTrigger::Time(10)));
        plan.go_codes.push(GoCode::new("LATE".into(), GoCodeTrigger::Time(100)));
        plan.go_codes.push(GoCode::new("MANUAL".into(), GoCodeTrigger::Manual));

        let triggered = evaluate_all_gocodes(&plan, 50, &[]);

        // Only EARLY should trigger (tick 50 >= 10, but tick 50 < 100)
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0], plan.go_codes[0].id);
    }

    #[test]
    fn test_casualties_contingency() {
        let contingency = Contingency::new(
            ContingencyTrigger::CasualtiesExceed(0.3),
            ContingencyResponse::Rally(BattleHexCoord::new(0, 0)),
        );

        // Below threshold
        assert!(!evaluate_contingency_trigger(
            &contingency,
            &[],
            0.2,
            true,
            &[],
            &[]
        ));

        // Above threshold
        assert!(evaluate_contingency_trigger(
            &contingency,
            &[],
            0.35,
            true,
            &[],
            &[]
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
            &contingency,
            &positions,
            0.0,
            true,
            &[],
            &[]
        ));

        // Unit routing
        let positions = vec![UnitPosition {
            unit_id,
            position: BattleHexCoord::new(5, 5),
            is_routing: true,
        }];
        assert!(evaluate_contingency_trigger(
            &contingency,
            &positions,
            0.0,
            true,
            &[],
            &[]
        ));
    }

    #[test]
    fn test_commander_dies_contingency() {
        let contingency = Contingency::new(
            ContingencyTrigger::CommanderDies,
            ContingencyResponse::Rally(BattleHexCoord::new(0, 0)),
        );

        // Commander alive
        assert!(!evaluate_contingency_trigger(
            &contingency,
            &[],
            0.0,
            true,
            &[],
            &[]
        ));

        // Commander dead
        assert!(evaluate_contingency_trigger(
            &contingency,
            &[],
            0.0,
            false,
            &[],
            &[]
        ));
    }

    #[test]
    fn test_position_lost_contingency() {
        let key_position = BattleHexCoord::new(10, 10);
        let contingency = Contingency::new(
            ContingencyTrigger::PositionLost(key_position),
            ContingencyResponse::Rally(BattleHexCoord::new(0, 0)),
        );

        // Position held by us
        let friendly = vec![key_position];
        assert!(!evaluate_contingency_trigger(
            &contingency,
            &[],
            0.0,
            true,
            &[],
            &friendly
        ));

        // Position contested (both there)
        let enemy = vec![key_position];
        assert!(!evaluate_contingency_trigger(
            &contingency,
            &[],
            0.0,
            true,
            &enemy,
            &friendly
        ));

        // Position lost (enemy there, we're not)
        assert!(evaluate_contingency_trigger(
            &contingency,
            &[],
            0.0,
            true,
            &enemy,
            &[]
        ));
    }

    #[test]
    fn test_already_activated_contingency_returns_false() {
        let mut contingency = Contingency::new(
            ContingencyTrigger::CasualtiesExceed(0.1),
            ContingencyResponse::Rally(BattleHexCoord::new(0, 0)),
        );
        contingency.activated = true;

        // Even though condition is met, it's already activated
        assert!(!evaluate_contingency_trigger(
            &contingency,
            &[],
            0.5,
            true,
            &[],
            &[]
        ));
    }

    #[test]
    fn test_evaluate_all_contingencies() {
        let mut plan = BattlePlan::new();
        plan.contingencies.push(Contingency::new(
            ContingencyTrigger::CasualtiesExceed(0.2),
            ContingencyResponse::Rally(BattleHexCoord::new(0, 0)),
        ));
        plan.contingencies.push(Contingency::new(
            ContingencyTrigger::CasualtiesExceed(0.5),
            ContingencyResponse::Rally(BattleHexCoord::new(5, 5)),
        ));
        plan.contingencies.push(Contingency::new(
            ContingencyTrigger::CommanderDies,
            ContingencyResponse::Rally(BattleHexCoord::new(10, 10)),
        ));

        // At 30% casualties, only first contingency triggers
        let triggered = evaluate_all_contingencies(&plan, &[], 0.3, true, &[], &[]);
        assert_eq!(triggered.len(), 1);
        assert_eq!(triggered[0], 0);

        // At 60% casualties, first two trigger
        let triggered = evaluate_all_contingencies(&plan, &[], 0.6, true, &[], &[]);
        assert_eq!(triggered.len(), 2);
        assert!(triggered.contains(&0));
        assert!(triggered.contains(&1));
    }

    #[test]
    fn test_describe_contingency_response() {
        let response1 = ContingencyResponse::ExecutePlan(UnitId::new());
        assert!(describe_contingency_response(&response1).contains("Execute backup plan"));

        let response2 = ContingencyResponse::Retreat(vec![
            BattleHexCoord::new(1, 1),
            BattleHexCoord::new(2, 2),
        ]);
        assert!(describe_contingency_response(&response2).contains("2 hexes"));

        let response3 = ContingencyResponse::Rally(BattleHexCoord::new(5, 5));
        assert!(describe_contingency_response(&response3).contains("Rally"));

        let response4 = ContingencyResponse::Signal(GoCodeId::new());
        assert!(describe_contingency_response(&response4).contains("Signal go-code"));
    }

    #[test]
    fn test_enemy_flanking_contingency_placeholder() {
        let contingency = Contingency::new(
            ContingencyTrigger::EnemyFlanking,
            ContingencyResponse::Rally(BattleHexCoord::new(0, 0)),
        );

        // Currently returns false as it needs more context
        assert!(!evaluate_contingency_trigger(
            &contingency,
            &[],
            0.0,
            true,
            &[],
            &[]
        ));
    }
}
