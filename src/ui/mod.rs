//! UI module - egui-based overlay for live simulation

pub mod display;
pub mod input;
pub mod state;
pub mod terminal;

pub use state::{GameUI, LogCategory, LogEntry};
