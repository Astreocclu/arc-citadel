//! Command execution - converts resolved intents to tasks

use crate::actions::catalog::ActionId;
use crate::command::resolver::{IntentResolution, IntentResolver};
use crate::core::types::{EntityId, Tick};
use crate::ecs::world::World;
use crate::entity::tasks::{Task, TaskPriority, TaskSource};
use crate::llm::parser::{IntentAction, IntentPriority, ParsedIntent};

/// Executes commands by creating tasks for entities
pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute a parsed intent, returning created tasks
    pub fn execute(world: &mut World, intent: &ParsedIntent, tick: Tick) -> ExecutionResult {
        let resolver = IntentResolver::new(world);
        let resolution = resolver.resolve(intent);

        if resolution.subjects.is_empty() && needs_subjects(&intent.action) {
            return ExecutionResult {
                tasks_created: 0,
                assigned_to: Vec::new(),
                error: Some("No matching entities found for command".to_string()),
            };
        }

        let priority = convert_priority(intent.priority);
        let mut tasks_created = 0;
        let mut assigned_to = Vec::new();

        for subject in &resolution.subjects {
            if let Some(task) = create_task(intent, &resolution, subject.entity_id, priority, tick)
            {
                if let Some(idx) = world.humans.index_of(subject.entity_id) {
                    world.humans.task_queues[idx].push(task);
                    tasks_created += 1;
                    assigned_to.push((subject.entity_id, subject.name.clone()));
                }
            }
        }

        ExecutionResult {
            tasks_created,
            assigned_to,
            error: None,
        }
    }
}

/// Result of executing a command
#[derive(Debug)]
pub struct ExecutionResult {
    pub tasks_created: usize,
    pub assigned_to: Vec<(EntityId, String)>,
    pub error: Option<String>,
}

fn needs_subjects(action: &IntentAction) -> bool {
    !matches!(action, IntentAction::Query)
}

fn convert_priority(priority: IntentPriority) -> TaskPriority {
    match priority {
        IntentPriority::Critical => TaskPriority::Critical,
        IntentPriority::High => TaskPriority::High,
        IntentPriority::Normal => TaskPriority::Normal,
        IntentPriority::Low => TaskPriority::Low,
    }
}

fn create_task(
    intent: &ParsedIntent,
    resolution: &IntentResolution,
    _entity_id: EntityId,
    priority: TaskPriority,
    tick: Tick,
) -> Option<Task> {
    let action_id = match intent.action {
        IntentAction::Build => ActionId::Build,
        IntentAction::Craft => ActionId::Craft,
        IntentAction::Combat => ActionId::Attack,
        IntentAction::Gather => ActionId::Gather,
        IntentAction::Move => ActionId::MoveTo,
        IntentAction::Rest => ActionId::Rest,
        IntentAction::Social => ActionId::TalkTo,
        IntentAction::Assign | IntentAction::Query | IntentAction::Unknown => return None,
    };

    let mut task = Task::new(action_id, priority, tick);
    task.source = TaskSource::PlayerCommand;

    // Apply location if resolved
    if let Some(pos) = resolution.location {
        task.target_position = Some(pos);
    }

    // Apply target entity if resolved
    if let Some(target) = resolution.target_entity {
        task.target_entity = Some(target);
    }

    Some(task)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_move_command() {
        let mut world = World::new();
        let marcus_id = world.spawn_human("Marcus".into());

        let intent = ParsedIntent {
            action: IntentAction::Move,
            target: None,
            location: Some("east".to_string()),
            subjects: Some(vec!["Marcus".to_string()]),
            priority: IntentPriority::Normal,
            ambiguous_concepts: Vec::new(),
            confidence: 0.9,
        };

        let result = CommandExecutor::execute(&mut world, &intent, 0);

        assert_eq!(result.tasks_created, 1);
        assert!(result.error.is_none());

        let idx = world.humans.index_of(marcus_id).unwrap();
        let task = world.humans.task_queues[idx].current().unwrap();
        assert_eq!(task.action, ActionId::MoveTo);
        assert!(task.target_position.is_some());
    }
}
