//! UI state management for live simulation

use crate::core::types::EntityId;
use std::collections::VecDeque;

/// Maximum action log entries to keep
const MAX_LOG_ENTRIES: usize = 50;

/// Game UI state
#[derive(Debug, Default)]
pub struct GameUI {
    /// Currently selected entity (if any)
    pub selected_entity: Option<EntityId>,
    /// Action log entries
    pub action_log: VecDeque<LogEntry>,
    /// Whether to show entity panel
    pub show_entity_panel: bool,
    /// Whether to show action log
    pub show_action_log: bool,
    /// Command input buffer
    pub command_input: String,
    /// Whether command input is focused
    pub command_focused: bool,
}

/// An entry in the action log
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub tick: u64,
    pub message: String,
    pub category: LogCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogCategory {
    Action,
    Combat,
    Production,
    System,
}

impl GameUI {
    pub fn new() -> Self {
        Self {
            selected_entity: None,
            action_log: VecDeque::with_capacity(MAX_LOG_ENTRIES),
            show_entity_panel: true,
            show_action_log: true,
            command_input: String::new(),
            command_focused: false,
        }
    }

    /// Add an entry to the action log
    pub fn log(&mut self, tick: u64, message: String, category: LogCategory) {
        if self.action_log.len() >= MAX_LOG_ENTRIES {
            self.action_log.pop_front();
        }
        self.action_log.push_back(LogEntry {
            tick,
            message,
            category,
        });
    }

    /// Select an entity by ID
    pub fn select(&mut self, entity_id: EntityId) {
        self.selected_entity = Some(entity_id);
    }

    /// Clear selection
    pub fn deselect(&mut self) {
        self.selected_entity = None;
    }

    /// Toggle selection
    pub fn toggle_select(&mut self, entity_id: EntityId) {
        if self.selected_entity == Some(entity_id) {
            self.deselect();
        } else {
            self.select(entity_id);
        }
    }
}
