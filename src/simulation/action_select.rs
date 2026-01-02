//! Action selection algorithm - the heart of autonomous behavior

use crate::actions::catalog::ActionId;
use crate::core::types::{EntityId, Tick, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::{Needs, NeedType};
use crate::entity::social::Disposition;
use crate::entity::species::human::HumanValues;
use crate::entity::species::orc::OrcValues;
// CODEGEN: species_values_imports
use crate::entity::tasks::{Task, TaskPriority, TaskSource};
use crate::entity::thoughts::ThoughtBuffer;

/// Context provided to the action selection algorithm
pub struct SelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a HumanValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    /// Nearest food zone: (zone_id, position, distance)
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    /// Nearby entities with their dispositions (from perception and social memory)
    /// Used for disposition-aware action selection (flee from hostile, talk to friendly)
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for humans
///
/// This function implements autonomous behavior by:
/// 1. First checking for critical needs (safety, food, rest at > 0.8)
/// 2. If entity already has a task, returning None (don't interrupt)
/// 3. Checking disposition-based responses (flee from hostile, approach friendly)
/// 4. Checking for value-driven impulses from strong thoughts
/// 5. Addressing moderate needs (> 0.6)
/// 6. Falling back to idle behavior based on values
pub fn select_action_human(ctx: &SelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Check for disposition-based responses
    // This uses social memory to react to nearby entities based on past experiences
    if let Some(task) = check_disposition_response(ctx) {
        return Some(task);
    }

    // Check for value-driven impulses from thoughts
    if let Some(task) = check_value_impulses(ctx) {
        return Some(task);
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action(ctx))
}

/// Handle critical needs (> 0.8) with immediate responses
fn select_critical_response(need: NeedType, ctx: &SelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            // Move to food zone
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander, // No food known, wander to find some
            target_position: None,
            target_entity: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety, // Find safe place to rest first
            target_position: None,
            target_entity: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Check for disposition-based responses to nearby entities
///
/// This function uses social memory to react to nearby entities based on
/// past experiences. It enables emergent social behavior where:
/// - Hostile entities trigger flight response when safety need is elevated
/// - Friendly entities encourage social interaction when social need is elevated
///
/// Returns a task if disposition-based action is warranted, None otherwise.
fn check_disposition_response(ctx: &SelectionContext) -> Option<Task> {
    // Check for hostile entities nearby when safety need is elevated
    // Threshold: safety need > 0.5 triggers concern about hostile entities
    let has_hostile = ctx.perceived_dispositions.iter()
        .any(|(_, d)| matches!(d, Disposition::Hostile | Disposition::Suspicious));

    if has_hostile && ctx.needs.safety > 0.5 {
        // Hostile entity nearby and we feel unsafe - flee!
        return Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            priority: TaskPriority::High,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        });
    }

    // Check for friendly entities when social need is elevated
    // Threshold: social need > 0.5 triggers desire to interact with friends
    let friendly_target = ctx.perceived_dispositions.iter()
        .find(|(_, d)| matches!(d, Disposition::Friendly | Disposition::Favorable))
        .map(|(id, _)| *id);

    if let Some(target_id) = friendly_target {
        if ctx.needs.social > 0.5 {
            // Friendly entity nearby and we want company - talk to them!
            return Some(Task {
                action: ActionId::TalkTo,
                target_position: None,
                target_entity: Some(target_id),
                priority: TaskPriority::Normal,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            });
        }
    }

    None
}

/// Check if strong thoughts combined with values should trigger action
///
/// This is where emergence happens - values filter which thoughts
/// lead to action. A high-justice entity reacts differently to
/// witnessing injustice than a high-comfort entity.
fn check_value_impulses(ctx: &SelectionContext) -> Option<Task> {
    if let Some(thought) = ctx.thoughts.strongest() {
        // Only react to strong thoughts
        if thought.intensity > 0.7 {
            // Justice-oriented response to witnessing injustice
            if ctx.values.justice > 0.7 && thought.concept_category == "injustice" {
                return Some(Task::new(ActionId::Help, TaskPriority::High, ctx.current_tick));
            }

            // Loyalty-driven response to ally in need
            if ctx.values.loyalty > 0.7 && thought.concept_category == "ally_distress" {
                return Some(Task::new(ActionId::Help, TaskPriority::High, ctx.current_tick));
            }

            // Honor-driven response to challenge
            if ctx.values.honor > 0.7 && thought.concept_category == "challenge" {
                return Some(Task::new(ActionId::Defend, TaskPriority::High, ctx.current_tick));
            }

            // Safety-oriented response to perceived danger
            if ctx.values.safety > 0.7 && thought.concept_category == "danger" {
                return Some(Task::new(ActionId::Flee, TaskPriority::High, ctx.current_tick));
            }

            // Piety-driven response to sacred events
            if ctx.values.piety > 0.7 && thought.concept_category == "sacred" {
                return Some(Task::new(ActionId::IdleObserve, TaskPriority::High, ctx.current_tick));
            }
        }
    }

    None
}

/// Address moderate needs (between 0.5 and 0.8)
fn address_moderate_need(ctx: &SelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    // Address needs proactively at 0.5 (not waiting until 0.6)
    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather, // Do productive work
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on the entity's values
///
/// Different values lead to different idle behaviors:
/// - High curiosity: observe surroundings
/// - High love/loyalty (social values): talk to nearby entities
/// - High comfort: stay put
/// - Default: wander
fn select_idle_action(ctx: &SelectionContext) -> Task {
    // Use love + loyalty as social tendency proxy
    let social_tendency = (ctx.values.love + ctx.values.loyalty) / 2.0;

    let action = if ctx.values.curiosity > social_tendency && ctx.values.curiosity > ctx.values.comfort {
        ActionId::IdleObserve
    } else if social_tendency > ctx.values.comfort && ctx.entity_nearby {
        ActionId::TalkTo
    } else if ctx.values.comfort > 0.7 {
        ActionId::IdleObserve // Stay put, observe
    } else {
        ActionId::IdleWander
    };

    Task::new(action, TaskPriority::Low, ctx.current_tick)
}

// ============================================================================
// ORC ACTION SELECTION
// ============================================================================

/// Context provided to orc action selection
pub struct OrcSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a OrcValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for orcs
///
/// Orcs have different behavioral priorities than humans:
/// 1. Critical needs still take priority (survival)
/// 2. High rage triggers aggressive action
/// 3. Blood debt creates revenge-seeking behavior
/// 4. Dominance drives territorial/assertive actions
/// 5. Clan loyalty triggers defense of nearby clan members
/// 6. Idle behavior is aggressive (patrol, challenge)
pub fn select_action_orc(ctx: &OrcSelectionContext) -> Option<Task> {
    // Critical needs always take priority (even orcs need to survive)
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_orc(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // High rage triggers aggressive action
    if let Some(task) = check_rage_response(ctx) {
        return Some(task);
    }

    // Blood debt creates revenge-seeking behavior
    if let Some(task) = check_blood_debt_response(ctx) {
        return Some(task);
    }

    // Dominance drives territorial behavior
    if let Some(task) = check_dominance_response(ctx) {
        return Some(task);
    }

    // Clan loyalty triggers defense
    if let Some(task) = check_clan_loyalty_response(ctx) {
        return Some(task);
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_orc(ctx) {
        return Some(task);
    }

    // Fall back to orc idle behavior
    Some(select_idle_action_orc(ctx))
}

/// Handle critical needs for orcs
fn select_critical_response_orc(need: NeedType, ctx: &OrcSelectionContext) -> Option<Task> {
    match need {
        // Orcs with high strength may fight instead of flee
        NeedType::Safety if ctx.threat_nearby && ctx.values.strength > 0.7 => Some(Task {
            action: ActionId::Defend,
            target_position: None,
            target_entity: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// High rage triggers aggressive action toward nearby entities
fn check_rage_response(ctx: &OrcSelectionContext) -> Option<Task> {
    if ctx.values.rage > 0.7 && ctx.entity_nearby {
        // Find any target (orcs in rage attack anyone nearby)
        let target = ctx.perceived_dispositions.first().map(|(id, _)| *id);

        return Some(Task {
            action: ActionId::Defend, // Attack/fight action
            target_position: None,
            target_entity: target,
            priority: TaskPriority::High,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        });
    }
    None
}

/// Blood debt creates revenge-seeking behavior against hostile entities
fn check_blood_debt_response(ctx: &OrcSelectionContext) -> Option<Task> {
    if ctx.values.blood_debt > 0.5 {
        // Look for hostile entities to attack (settling blood debts)
        let hostile_target = ctx.perceived_dispositions.iter()
            .find(|(_, d)| matches!(d, Disposition::Hostile | Disposition::Suspicious))
            .map(|(id, _)| *id);

        if let Some(target_id) = hostile_target {
            return Some(Task {
                action: ActionId::Defend, // Attack to settle blood debt
                target_position: None,
                target_entity: Some(target_id),
                priority: TaskPriority::High,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            });
        }
    }
    None
}

/// Dominance drives territorial/assertive actions
fn check_dominance_response(ctx: &OrcSelectionContext) -> Option<Task> {
    if ctx.values.dominance > 0.7 && ctx.values.territory > 0.5 {
        // High dominance + territory = patrol and defend territory
        return Some(Task {
            action: ActionId::IdleWander, // Patrol behavior
            target_position: None,
            target_entity: None,
            priority: TaskPriority::Normal,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        });
    }
    None
}

/// Clan loyalty triggers defense of nearby friendly entities
fn check_clan_loyalty_response(ctx: &OrcSelectionContext) -> Option<Task> {
    if ctx.values.clan_loyalty > 0.7 {
        // Check if friendly entity is nearby with hostile also present
        let has_friendly = ctx.perceived_dispositions.iter()
            .any(|(_, d)| matches!(d, Disposition::Friendly | Disposition::Favorable));
        let hostile_target = ctx.perceived_dispositions.iter()
            .find(|(_, d)| matches!(d, Disposition::Hostile))
            .map(|(id, _)| *id);

        if has_friendly && hostile_target.is_some() {
            // Clan member nearby with hostile threat - defend!
            return Some(Task {
                action: ActionId::Defend,
                target_position: None,
                target_entity: hostile_target,
                priority: TaskPriority::High,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Reaction,
            });
        }
    }
    None
}

/// Address moderate needs for orcs
fn address_moderate_need_orc(ctx: &OrcSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo, // Even orcs socialize
        NeedType::Purpose => ActionId::Gather, // Productive work
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on orc values
///
/// Orc idle behaviors reflect their aggressive nature:
/// - High combat prowess: patrol/wander aggressively
/// - High dominance: observe (watch for challenges)
/// - High strength: gather (work on strength-building tasks)
/// - Default: wander looking for trouble
fn select_idle_action_orc(ctx: &OrcSelectionContext) -> Task {
    let action = if ctx.values.combat_prowess > 0.7 {
        ActionId::IdleWander // Patrol for combat opportunities
    } else if ctx.values.dominance > 0.7 {
        ActionId::IdleObserve // Watch for challenges to dominance
    } else if ctx.values.strength > 0.7 {
        ActionId::Gather // Work on strength-building tasks
    } else {
        ActionId::IdleWander // Default: wander
    };

    Task::new(action, TaskPriority::Low, ctx.current_tick)
}

// CODEGEN: species_selection_context

// CODEGEN: species_select_action

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::thoughts::{Thought, Valence, CauseType};

    #[test]
    fn test_critical_safety_need_with_threat() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.safety = 0.9; // Critical level
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: true,
            food_available: true,
            safe_location: false,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::Flee);
        assert_eq!(task.priority, TaskPriority::Critical);
    }

    #[test]
    fn test_critical_food_need() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.food = 0.9; // Critical level
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::Eat);
        assert_eq!(task.priority, TaskPriority::Critical);
    }

    #[test]
    fn test_dont_interrupt_existing_task() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: true, // Already has a task
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_none()); // Should not interrupt
    }

    #[test]
    fn test_high_justice_responds_to_injustice() {
        let body = BodyState::new();
        let needs = Needs::default();
        let mut thoughts = ThoughtBuffer::new();
        thoughts.add(Thought::new(
            Valence::Negative,
            0.8,
            "injustice",
            "witnessed unfair treatment",
            CauseType::Event,
            0,
        ));
        let mut values = HumanValues::default();
        values.justice = 0.9;

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::Help);
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn test_moderate_social_need() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.social = 0.7; // Moderate level
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true, // Someone to talk to
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::TalkTo);
        assert_eq!(task.priority, TaskPriority::Normal);
    }

    #[test]
    fn test_curious_entity_observes() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let mut values = HumanValues::default();
        values.curiosity = 0.9;
        values.love = 0.1;
        values.loyalty = 0.1;
        values.comfort = 0.1;

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::IdleObserve);
        assert_eq!(task.priority, TaskPriority::Low);
    }

    #[test]
    fn test_social_entity_talks_when_others_nearby() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let mut values = HumanValues::default();
        values.love = 0.9;
        values.loyalty = 0.9;
        values.curiosity = 0.1;
        values.comfort = 0.1;

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::TalkTo);
        assert_eq!(task.priority, TaskPriority::Low);
    }

    #[test]
    fn test_default_idle_wander() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default(); // All values at default (0.0)

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false, // No one to talk to
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::IdleWander);
        assert_eq!(task.priority, TaskPriority::Low);
    }

    #[test]
    fn test_high_safety_value_flees_from_danger_thought() {
        let body = BodyState::new();
        let needs = Needs::default();
        let mut thoughts = ThoughtBuffer::new();
        thoughts.add(Thought::new(
            Valence::Negative,
            0.8,
            "danger",
            "sensed imminent threat",
            CauseType::Event,
            0,
        ));
        let mut values = HumanValues::default();
        values.safety = 0.9;

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::Flee);
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn test_critical_need_overrides_existing_task() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.safety = 0.95; // Critical safety need
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: true, // Has existing task
            threat_nearby: true,
            food_available: true,
            safe_location: false,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        // Critical needs should still trigger even with existing task
        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::Flee);
        assert_eq!(task.priority, TaskPriority::Critical);
    }

    #[test]
    fn test_hungry_entity_moves_to_food() {
        use crate::core::types::Vec2;

        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.food = 0.85; // Critical hunger
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: false, // No food at current position
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: Some((0, Vec2::new(100.0, 100.0), 50.0)), // Food zone nearby
            perceived_dispositions: vec![],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::MoveTo);
        // Check target_position coordinates since Vec2 doesn't implement PartialEq
        let target = task.target_position.expect("Expected target_position to be set");
        assert!((target.x - 100.0).abs() < 0.001);
        assert!((target.y - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_hostile_disposition_influences_action() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.safety = 0.6; // Elevated safety need (not critical, but concerned)
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a hostile entity
        let hostile_entity = EntityId::new();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![(hostile_entity, Disposition::Hostile)],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // With hostile entity nearby and elevated safety need, should flee
        assert_eq!(task.action, ActionId::Flee);
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn test_suspicious_disposition_triggers_flee() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.safety = 0.6; // Elevated safety need
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a suspicious entity
        let suspicious_entity = EntityId::new();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![(suspicious_entity, Disposition::Suspicious)],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // Suspicious entities also trigger flee response
        assert_eq!(task.action, ActionId::Flee);
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn test_friendly_disposition_influences_action() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.social = 0.6; // Elevated social need
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a friendly entity
        let friendly_entity = EntityId::new();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![(friendly_entity, Disposition::Friendly)],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // With friendly entity nearby and elevated social need, should talk
        assert_eq!(task.action, ActionId::TalkTo);
        assert_eq!(task.target_entity, Some(friendly_entity));
        assert_eq!(task.priority, TaskPriority::Normal);
    }

    #[test]
    fn test_favorable_disposition_triggers_talk() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.social = 0.6; // Elevated social need
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a favorable entity
        let favorable_entity = EntityId::new();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![(favorable_entity, Disposition::Favorable)],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // Favorable entities also trigger talk response
        assert_eq!(task.action, ActionId::TalkTo);
        assert_eq!(task.target_entity, Some(favorable_entity));
    }

    #[test]
    fn test_hostile_takes_priority_over_friendly() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.safety = 0.6; // Both safety and social needs elevated
        needs.social = 0.6;
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create both hostile and friendly entities
        let hostile_entity = EntityId::new();
        let friendly_entity = EntityId::new();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![
                (hostile_entity, Disposition::Hostile),
                (friendly_entity, Disposition::Friendly),
            ],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // Safety response (flee from hostile) takes priority over social (talk to friendly)
        assert_eq!(task.action, ActionId::Flee);
    }

    #[test]
    fn test_low_safety_need_ignores_hostile() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.safety = 0.3; // Low safety need (entity feels safe)
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a hostile entity
        let hostile_entity = EntityId::new();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![(hostile_entity, Disposition::Hostile)],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // With low safety need, hostile entity doesn't trigger flee
        assert_ne!(task.action, ActionId::Flee);
    }

    #[test]
    fn test_low_social_need_ignores_friendly() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.social = 0.3; // Low social need (entity doesn't want to talk)
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a friendly entity
        let friendly_entity = EntityId::new();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![(friendly_entity, Disposition::Friendly)],
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // With low social need, friendly entity doesn't trigger talk
        // (falls through to idle behavior)
        assert_ne!(task.action, ActionId::TalkTo);
    }

    // ========================================================================
    // ORC ACTION SELECTION TESTS
    // ========================================================================

    #[test]
    fn test_orc_high_rage_attacks() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let mut values = OrcValues::default();
        values.rage = 0.9; // High rage

        let target_entity = EntityId::new();

        let ctx = OrcSelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![(target_entity, Disposition::Neutral)],
        };

        let task = select_action_orc(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::Defend);
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn test_orc_blood_debt_seeks_revenge() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let mut values = OrcValues::default();
        values.blood_debt = 0.8; // High blood debt

        let hostile_entity = EntityId::new();

        let ctx = OrcSelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![(hostile_entity, Disposition::Hostile)],
        };

        let task = select_action_orc(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::Defend);
        assert_eq!(task.target_entity, Some(hostile_entity));
    }

    #[test]
    fn test_orc_strong_fights_instead_of_flees() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.safety = 0.9; // Critical safety need
        let thoughts = ThoughtBuffer::new();
        let mut values = OrcValues::default();
        values.strength = 0.9; // High strength

        let ctx = OrcSelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: true,
            food_available: true,
            safe_location: false,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_orc(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // Strong orcs fight instead of flee
        assert_eq!(task.action, ActionId::Defend);
        assert_eq!(task.priority, TaskPriority::Critical);
    }

    #[test]
    fn test_orc_weak_flees_from_threat() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.safety = 0.9; // Critical safety need
        let thoughts = ThoughtBuffer::new();
        let mut values = OrcValues::default();
        values.strength = 0.3; // Low strength

        let ctx = OrcSelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: true,
            food_available: true,
            safe_location: false,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_orc(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // Weak orcs flee
        assert_eq!(task.action, ActionId::Flee);
    }

    #[test]
    fn test_orc_clan_loyalty_defends_ally() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let mut values = OrcValues::default();
        values.clan_loyalty = 0.9; // High clan loyalty

        let friendly_entity = EntityId::new();
        let hostile_entity = EntityId::new();

        let ctx = OrcSelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![
                (friendly_entity, Disposition::Friendly),
                (hostile_entity, Disposition::Hostile),
            ],
        };

        let task = select_action_orc(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // Loyal orc defends clan member
        assert_eq!(task.action, ActionId::Defend);
        assert_eq!(task.target_entity, Some(hostile_entity));
    }

    #[test]
    fn test_orc_idle_high_combat_prowess_wanders() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let mut values = OrcValues::default();
        values.combat_prowess = 0.9;

        let ctx = OrcSelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_orc(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::IdleWander);
        assert_eq!(task.priority, TaskPriority::Low);
    }

    #[test]
    fn test_orc_idle_high_dominance_observes() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let mut values = OrcValues::default();
        values.dominance = 0.9;
        values.combat_prowess = 0.3; // Low combat prowess so dominance takes precedence

        let ctx = OrcSelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_orc(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::IdleObserve);
    }
}
