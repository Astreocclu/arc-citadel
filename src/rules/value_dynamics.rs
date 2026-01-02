//! Value dynamics system - tick-based changes and event responses

use crate::core::types::Species;
use std::collections::HashMap;

/// Per-tick change configuration for a single value
#[derive(Debug, Clone)]
pub struct TickDelta {
    pub value_name: String,
    pub delta: f32,
    pub min: f32,
    pub max: f32,
}

/// Event-triggered value change
#[derive(Debug, Clone)]
pub struct ValueEvent {
    pub event_type: String,
    pub value_name: String,
    pub delta: f32,
}

/// All dynamics for a species
#[derive(Debug, Clone, Default)]
pub struct SpeciesDynamics {
    pub tick_deltas: Vec<TickDelta>,
    pub events: Vec<ValueEvent>,
}

/// Central storage for all species dynamics
#[derive(Debug, Default)]
pub struct ValueDynamicsRules {
    dynamics: HashMap<Species, SpeciesDynamics>,
}

impl ValueDynamicsRules {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_tick_deltas(&self, species: Species) -> &[TickDelta] {
        self.dynamics
            .get(&species)
            .map(|d| d.tick_deltas.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_events_for_type(&self, species: Species, event_type: &str) -> Vec<&ValueEvent> {
        self.dynamics
            .get(&species)
            .map(|d| {
                d.events
                    .iter()
                    .filter(|e| e.event_type == event_type)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn insert(&mut self, species: Species, dynamics: SpeciesDynamics) {
        self.dynamics.insert(species, dynamics);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_dynamics_storage() {
        let mut rules = ValueDynamicsRules::new();

        let dynamics = SpeciesDynamics {
            tick_deltas: vec![TickDelta {
                value_name: "bloodlust".to_string(),
                delta: 0.002,
                min: 0.0,
                max: 1.0,
            }],
            events: vec![ValueEvent {
                event_type: "combat_victory".to_string(),
                value_name: "bloodlust".to_string(),
                delta: 0.15,
            }],
        };

        rules.insert(Species::Gnoll, dynamics);

        let deltas = rules.get_tick_deltas(Species::Gnoll);
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].value_name, "bloodlust");

        let events = rules.get_events_for_type(Species::Gnoll, "combat_victory");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].delta, 0.15);
    }

    #[test]
    fn test_empty_species_returns_empty() {
        let rules = ValueDynamicsRules::new();

        let deltas = rules.get_tick_deltas(Species::Human);
        assert!(deltas.is_empty());

        let events = rules.get_events_for_type(Species::Human, "any_event");
        assert!(events.is_empty());
    }
}
