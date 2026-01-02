//! Formation combat for LOD 1+
//!
//! At formation level, individual exchanges become statistical.
//! Property matchups determine casualty rates, not percentages.

use serde::{Deserialize, Serialize};
use crate::core::types::EntityId;
use crate::combat::constants::FORMATION_BREAK_THRESHOLD;

/// Formation state for LOD 1 combat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationState {
    /// Entities in this formation
    pub entities: Vec<EntityId>,
    /// Front line entities (engage in combat)
    pub front_line: Vec<EntityId>,
    /// Pressure: -1.0 (losing) to +1.0 (winning)
    pub pressure: f32,
    /// Cohesion: 0.0 (scattered) to 1.0 (tight formation)
    pub cohesion: f32,
    /// Fatigue: 0.0 (fresh) to 1.0 (exhausted)
    pub fatigue: f32,
    /// Formation-level stress
    pub stress: f32,
    /// Count of broken/routed entities
    pub broken_count: u32,
}

impl FormationState {
    /// Create a new formation
    pub fn new(entities: Vec<EntityId>) -> Self {
        let front_line = entities.iter().take(entities.len() / 3).copied().collect();
        Self {
            entities,
            front_line,
            pressure: 0.0,
            cohesion: 1.0,
            fatigue: 0.0,
            stress: 0.0,
            broken_count: 0,
        }
    }

    /// Apply pressure delta (clamped to -1.0 to 1.0)
    pub fn apply_pressure_delta(&mut self, delta: f32) {
        self.pressure = (self.pressure + delta).clamp(-1.0, 1.0);
    }

    /// Effective fighting strength
    pub fn effective_strength(&self) -> usize {
        self.entities.len().saturating_sub(self.broken_count as usize)
    }

    /// Check if formation is broken
    pub fn is_broken(&self) -> bool {
        if self.entities.is_empty() {
            return true;
        }
        let broken_ratio = self.broken_count as f32 / self.entities.len() as f32;
        broken_ratio >= FORMATION_BREAK_THRESHOLD
    }

    /// Get pressure category (for display, not calculations)
    pub fn pressure_category(&self) -> PressureCategory {
        match self.pressure {
            p if p <= -0.7 => PressureCategory::Collapsing,
            p if p <= -0.3 => PressureCategory::Losing,
            p if p <= 0.3 => PressureCategory::Neutral,
            p if p <= 0.7 => PressureCategory::Pushing,
            _ => PressureCategory::Overwhelming,
        }
    }
}

/// Pressure categories (for display only)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureCategory {
    Collapsing,
    Losing,
    Neutral,
    Pushing,
    Overwhelming,
}

/// Shock attack types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShockType {
    CavalryCharge,
    FlankAttack,
    RearCharge,
    Ambush,
}

impl ShockType {
    /// Stress spike from shock attack
    pub fn stress_spike(&self) -> f32 {
        match self {
            ShockType::CavalryCharge => 0.30,
            ShockType::FlankAttack => 0.20,
            ShockType::RearCharge => 0.40,
            ShockType::Ambush => 0.35,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formation_creation() {
        let formation = FormationState::new(vec![EntityId::new(), EntityId::new()]);
        assert_eq!(formation.entities.len(), 2);
        assert_eq!(formation.pressure, 0.0);
    }

    #[test]
    fn test_pressure_clamped() {
        let mut formation = FormationState::new(vec![]);
        formation.apply_pressure_delta(2.0);
        assert_eq!(formation.pressure, 1.0);

        formation.apply_pressure_delta(-3.0);
        assert_eq!(formation.pressure, -1.0);
    }

    #[test]
    fn test_formation_break_threshold() {
        let mut formation = FormationState::new(vec![EntityId::new(); 10]);
        formation.broken_count = 4;

        assert!(formation.is_broken());
    }

    #[test]
    fn test_pressure_categories() {
        let mut formation = FormationState::new(vec![]);

        formation.pressure = -0.8;
        assert_eq!(formation.pressure_category(), PressureCategory::Collapsing);

        formation.pressure = 0.0;
        assert_eq!(formation.pressure_category(), PressureCategory::Neutral);

        formation.pressure = 0.9;
        assert_eq!(formation.pressure_category(), PressureCategory::Overwhelming);
    }
}
