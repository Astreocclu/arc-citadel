//! Task queue and execution

use crate::core::types::{EntityId, Vec2, Tick};
use crate::actions::catalog::ActionId;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// A task is an action with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub action: ActionId,
    pub target_position: Option<Vec2>,
    pub target_entity: Option<EntityId>,
    pub priority: TaskPriority,
    pub created_tick: Tick,
    pub progress: f32,
    pub source: TaskSource,
}

/// Task priority levels with explicit ordering values
///
/// Higher numeric value = higher priority.
/// This ordering is relied upon by TaskQueue::push for insertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum TaskPriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskSource {
    PlayerCommand,
    Autonomous,
    Reaction,
}

impl Task {
    pub fn new(action: ActionId, priority: TaskPriority, tick: Tick) -> Self {
        Self {
            action,
            target_position: None,
            target_entity: None,
            priority,
            created_tick: tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }
    }

    pub fn with_position(mut self, pos: Vec2) -> Self {
        self.target_position = Some(pos);
        self
    }

    pub fn with_entity(mut self, entity: EntityId) -> Self {
        self.target_entity = Some(entity);
        self
    }

    pub fn from_player(mut self) -> Self {
        self.source = TaskSource::PlayerCommand;
        self
    }
}

/// Queue of tasks for an entity
#[derive(Debug, Clone, Default)]
pub struct TaskQueue {
    current: Option<Task>,
    queued: VecDeque<Task>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            current: None,
            queued: VecDeque::new(),
        }
    }

    pub fn current(&self) -> Option<&Task> {
        self.current.as_ref()
    }

    pub fn current_mut(&mut self) -> Option<&mut Task> {
        self.current.as_mut()
    }

    pub fn push(&mut self, task: Task) {
        let pos = self.queued.iter()
            .position(|t| task.priority as u8 > t.priority as u8)
            .unwrap_or(self.queued.len());
        self.queued.insert(pos, task);

        if self.current.is_none() {
            self.current = self.queued.pop_front();
        }
    }

    pub fn complete_current(&mut self) {
        self.current = self.queued.pop_front();
    }

    pub fn cancel_current(&mut self) {
        self.current = self.queued.pop_front();
    }

    pub fn clear(&mut self) {
        self.current = None;
        self.queued.clear();
    }

    pub fn is_idle(&self) -> bool {
        self.current.is_none() && self.queued.is_empty()
    }
}
