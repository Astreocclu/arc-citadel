//! Generic rule evaluation using ValueAccessor

use crate::actions::catalog::ActionId;
use crate::core::types::Tick;
use crate::entity::species::value_access::ValueAccessor;
use crate::entity::tasks::{Task, TaskPriority, TaskSource};
use crate::rules::action_rules::{ActionRule, IdleBehavior};

/// Evaluate action rules against current values
/// Returns the first matching action (rules are priority-ordered)
pub fn evaluate_action_rules<V: ValueAccessor>(
    values: &V,
    rules: &[ActionRule],
    current_tick: Tick,
    entity_nearby: bool,
) -> Option<Task> {
    for rule in rules {
        // Skip rules requiring target if no entity nearby
        if rule.requires_target && !entity_nearby {
            continue;
        }

        if let Some(val) = values.get_value(&rule.trigger_value) {
            if val > rule.threshold {
                return Some(Task {
                    action: rule.action,
                    target_position: None,
                    target_entity: None,
                    target_building: None,
                    priority: rule.priority,
                    created_tick: current_tick,
                    progress: 0.0,
                    source: TaskSource::Autonomous,
                });
            }
        }
    }
    None
}

/// Select idle behavior based on values
pub fn select_idle_behavior<V: ValueAccessor>(
    values: &V,
    behaviors: &[IdleBehavior],
    current_tick: Tick,
) -> Task {
    for behavior in behaviors {
        if let Some(val) = values.get_value(&behavior.value) {
            if val > behavior.threshold {
                return Task {
                    action: behavior.action,
                    target_position: None,
                    target_entity: None,
                    target_building: None,
                    priority: TaskPriority::Low,
                    created_tick: current_tick,
                    progress: 0.0,
                    source: TaskSource::Autonomous,
                };
            }
        }
    }

    // Default idle behavior
    Task {
        action: ActionId::IdleWander,
        target_position: None,
        target_entity: None,
        target_building: None,
        priority: TaskPriority::Low,
        created_tick: current_tick,
        progress: 0.0,
        source: TaskSource::Autonomous,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::species::gnoll::GnollValues;

    #[test]
    fn test_rule_triggers_when_above_threshold() {
        let mut values = GnollValues::default();
        values.bloodlust = 0.8; // Above threshold

        let rules = vec![ActionRule {
            trigger_value: "bloodlust".to_string(),
            threshold: 0.7,
            action: ActionId::Attack,
            priority: TaskPriority::High,
            requires_target: true,
            description: "Attack".to_string(),
        }];

        let task = evaluate_action_rules(&values, &rules, 0, true);
        assert!(task.is_some());
        assert!(matches!(task.unwrap().action, ActionId::Attack));
    }

    #[test]
    fn test_rule_skipped_when_below_threshold() {
        let mut values = GnollValues::default();
        values.bloodlust = 0.5; // Below threshold

        let rules = vec![ActionRule {
            trigger_value: "bloodlust".to_string(),
            threshold: 0.7,
            action: ActionId::Attack,
            priority: TaskPriority::High,
            requires_target: true,
            description: "Attack".to_string(),
        }];

        let task = evaluate_action_rules(&values, &rules, 0, true);
        assert!(task.is_none());
    }

    #[test]
    fn test_requires_target_skipped_when_no_entity() {
        let mut values = GnollValues::default();
        values.bloodlust = 0.8;

        let rules = vec![ActionRule {
            trigger_value: "bloodlust".to_string(),
            threshold: 0.7,
            action: ActionId::Attack,
            priority: TaskPriority::High,
            requires_target: true,
            description: "Attack".to_string(),
        }];

        let task = evaluate_action_rules(&values, &rules, 0, false); // No entity nearby
        assert!(task.is_none());
    }

    #[test]
    fn test_first_matching_rule_wins() {
        let mut values = GnollValues::default();
        values.bloodlust = 0.8;
        values.hunger = 0.7;

        let rules = vec![
            ActionRule {
                trigger_value: "bloodlust".to_string(),
                threshold: 0.7,
                action: ActionId::Attack,
                priority: TaskPriority::High,
                requires_target: true,
                description: "Attack".to_string(),
            },
            ActionRule {
                trigger_value: "hunger".to_string(),
                threshold: 0.6,
                action: ActionId::Gather,
                priority: TaskPriority::Normal,
                requires_target: true,
                description: "Hunt".to_string(),
            },
        ];

        let task = evaluate_action_rules(&values, &rules, 0, true);
        assert!(task.is_some());
        // First rule (Attack) should win
        assert!(matches!(task.unwrap().action, ActionId::Attack));
    }

    #[test]
    fn test_idle_behavior_selection() {
        let mut values = GnollValues::default();
        values.hunger = 0.5;

        let behaviors = vec![
            IdleBehavior {
                value: "hunger".to_string(),
                threshold: 0.4,
                action: ActionId::IdleWander,
                requires_target: false,
                description: "Prowl".to_string(),
            },
        ];

        let task = select_idle_behavior(&values, &behaviors, 0);
        assert!(matches!(task.action, ActionId::IdleWander));
    }

    #[test]
    fn test_idle_defaults_to_wander() {
        let values = GnollValues::default();
        let behaviors: Vec<IdleBehavior> = vec![];

        let task = select_idle_behavior(&values, &behaviors, 0);
        assert!(matches!(task.action, ActionId::IdleWander));
    }
}
