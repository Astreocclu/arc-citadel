//! Apply value dynamics each tick

use crate::core::types::Species;
use crate::entity::species::value_access::ValueAccessor;
use crate::rules::value_dynamics::{TickDelta, ValueDynamicsRules};

/// Apply per-tick value changes to an entity
pub fn apply_tick_dynamics<V: ValueAccessor>(values: &mut V, deltas: &[TickDelta]) {
    for delta in deltas {
        if let Some(current) = values.get_value(&delta.value_name) {
            let new_value = (current + delta.delta).clamp(delta.min, delta.max);
            values.set_value(&delta.value_name, new_value);
        }
    }
}

/// Apply event-triggered value changes
pub fn apply_event<V: ValueAccessor>(
    values: &mut V,
    event_type: &str,
    dynamics: &ValueDynamicsRules,
    species: Species,
) {
    for event in dynamics.get_events_for_type(species, event_type) {
        if let Some(current) = values.get_value(&event.value_name) {
            let new_value = (current + event.delta).clamp(0.0, 1.0);
            values.set_value(&event.value_name, new_value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::species::gnoll::GnollValues;
    use crate::rules::value_dynamics::{SpeciesDynamics, ValueEvent};

    #[test]
    fn test_tick_dynamics_increases_value() {
        let mut values = GnollValues::default();
        values.bloodlust = 0.5;

        let deltas = vec![TickDelta {
            value_name: "bloodlust".to_string(),
            delta: 0.1,
            min: 0.0,
            max: 1.0,
        }];

        apply_tick_dynamics(&mut values, &deltas);

        assert!((values.bloodlust - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_tick_dynamics_clamps_to_max() {
        let mut values = GnollValues::default();
        values.bloodlust = 0.95;

        let deltas = vec![TickDelta {
            value_name: "bloodlust".to_string(),
            delta: 0.1,
            min: 0.0,
            max: 1.0,
        }];

        apply_tick_dynamics(&mut values, &deltas);

        assert!((values.bloodlust - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_tick_dynamics_clamps_to_min() {
        let mut values = GnollValues::default();
        values.bloodlust = 0.05;

        let deltas = vec![TickDelta {
            value_name: "bloodlust".to_string(),
            delta: -0.1,
            min: 0.0,
            max: 1.0,
        }];

        apply_tick_dynamics(&mut values, &deltas);

        assert!((values.bloodlust - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_event_applies_delta() {
        let mut values = GnollValues::default();
        values.bloodlust = 0.5;

        let mut dynamics = ValueDynamicsRules::new();
        dynamics.insert(
            Species::Gnoll,
            SpeciesDynamics {
                tick_deltas: vec![],
                events: vec![ValueEvent {
                    event_type: "combat_victory".to_string(),
                    value_name: "bloodlust".to_string(),
                    delta: 0.15,
                }],
            },
        );

        apply_event(&mut values, "combat_victory", &dynamics, Species::Gnoll);

        assert!((values.bloodlust - 0.65).abs() < 0.01);
    }

    #[test]
    fn test_event_negative_delta() {
        let mut values = GnollValues::default();
        values.hunger = 0.8;

        let mut dynamics = ValueDynamicsRules::new();
        dynamics.insert(
            Species::Gnoll,
            SpeciesDynamics {
                tick_deltas: vec![],
                events: vec![ValueEvent {
                    event_type: "feeding".to_string(),
                    value_name: "hunger".to_string(),
                    delta: -0.4,
                }],
            },
        );

        apply_event(&mut values, "feeding", &dynamics, Species::Gnoll);

        assert!((values.hunger - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_event_clamps_result() {
        let mut values = GnollValues::default();
        values.bloodlust = 0.95;

        let mut dynamics = ValueDynamicsRules::new();
        dynamics.insert(
            Species::Gnoll,
            SpeciesDynamics {
                tick_deltas: vec![],
                events: vec![ValueEvent {
                    event_type: "combat_victory".to_string(),
                    value_name: "bloodlust".to_string(),
                    delta: 0.15,
                }],
            },
        );

        apply_event(&mut values, "combat_victory", &dynamics, Species::Gnoll);

        // Should be clamped to 1.0
        assert!((values.bloodlust - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_unknown_event_does_nothing() {
        let mut values = GnollValues::default();
        let original_bloodlust = values.bloodlust;

        let mut dynamics = ValueDynamicsRules::new();
        dynamics.insert(
            Species::Gnoll,
            SpeciesDynamics {
                tick_deltas: vec![],
                events: vec![ValueEvent {
                    event_type: "combat_victory".to_string(),
                    value_name: "bloodlust".to_string(),
                    delta: 0.15,
                }],
            },
        );

        // Apply an event type that doesn't exist
        apply_event(&mut values, "unknown_event", &dynamics, Species::Gnoll);

        assert!((values.bloodlust - original_bloodlust).abs() < 0.01);
    }
}
