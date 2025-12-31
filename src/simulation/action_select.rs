//! Action selection algorithm - the heart of autonomous behavior

use crate::actions::catalog::ActionId;
use crate::entity::needs::{Needs, NeedType};
use crate::entity::thoughts::ThoughtBuffer;
use crate::entity::body::BodyState;
use crate::entity::species::human::HumanValues;
use crate::entity::tasks::{Task, TaskPriority, TaskSource};
use crate::core::types::Tick;

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
}

/// Main action selection function for humans
///
/// This function implements autonomous behavior by:
/// 1. First checking for critical needs (safety, food, rest at > 0.8)
/// 2. If entity already has a task, returning None (don't interrupt)
/// 3. Checking for value-driven impulses from strong thoughts
/// 4. Addressing moderate needs (> 0.6)
/// 5. Falling back to idle behavior based on values
pub fn select_action_human(ctx: &SelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
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
    let action = match need {
        NeedType::Safety if ctx.threat_nearby => ActionId::Flee,
        NeedType::Safety => ActionId::SeekSafety,
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Food => ActionId::SeekSafety, // Forage/search when no food available
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Rest => ActionId::SeekSafety, // Find safe place to rest first
        _ => return None,
    };

    Some(Task {
        action,
        target_position: None,
        target_entity: None,
        priority: TaskPriority::Critical,
        created_tick: ctx.current_tick,
        progress: 0.0,
        source: TaskSource::Reaction,
    })
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

/// Address moderate needs (between 0.6 and 0.8)
fn address_moderate_need(ctx: &SelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    // Only address if need is moderately pressing
    if level < 0.6 {
        return None;
    }

    let action = match need_type {
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::IdleObserve, // Look for something to do
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Food if ctx.food_available => ActionId::Eat,
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
        };

        // Critical needs should still trigger even with existing task
        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::Flee);
        assert_eq!(task.priority, TaskPriority::Critical);
    }
}
