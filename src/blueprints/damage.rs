//! Damage resolution and state transitions for blueprint instances.
//!
//! This module handles damage application and automatic state transitions
//! based on HP thresholds defined in blueprints. When an instance takes damage,
//! this system determines if the damage triggers a state change and applies
//! any associated property overrides, breaches, or rubble production.

use glam::Vec2;

use super::instance::{BlueprintInstance, Breach};
use super::schema::{Blueprint, DamageStateDef};

/// Result of applying damage to an instance
#[derive(Debug, Clone)]
pub struct DamageResult {
    /// New damage state name if state changed, None if unchanged
    pub new_state: Option<String>,
    /// Whether the instance was destroyed (HP reached 0)
    pub destroyed: bool,
    /// New breach created if state transition created one
    pub new_breach: Option<Breach>,
    /// Whether rubble was produced by this state transition
    pub rubble_produced: bool,
}

/// Apply damage to an instance and update its state based on HP thresholds.
///
/// This function:
/// 1. Applies the damage amount to the instance's current HP
/// 2. Determines the appropriate damage state based on the new HP ratio
/// 3. If the state changed, applies property overrides and creates breaches/rubble
///
/// # Arguments
/// * `instance` - The blueprint instance to damage
/// * `amount` - Amount of damage to apply (positive value)
/// * `impact_point` - World position where damage was applied (used for breach location)
/// * `blueprint` - The blueprint definition containing damage state thresholds
///
/// # Returns
/// A `DamageResult` describing what changed
pub fn apply_damage(
    instance: &mut BlueprintInstance,
    amount: f32,
    impact_point: Vec2,
    blueprint: &Blueprint,
) -> DamageResult {
    let old_state = instance.damage_state.clone();
    let was_destroyed = instance.apply_damage(amount);

    // Find new damage state based on HP ratio
    let hp_ratio = instance.hp_ratio();
    let new_state = find_damage_state(&blueprint.damage_states, hp_ratio);

    let mut result = DamageResult {
        new_state: None,
        destroyed: was_destroyed,
        new_breach: None,
        rubble_produced: false,
    };

    if let Some(state) = new_state {
        if state.name != old_state {
            // State changed
            instance.damage_state = state.name.clone();
            result.new_state = Some(state.name.clone());

            // Apply property overrides
            instance.apply_overrides(&state.overrides);

            // Handle breach creation
            if state.creates_breach {
                let breach = Breach {
                    position: impact_point,
                    width: 2.0, // Default breach width
                };
                instance.breaches.push(breach.clone());
                result.new_breach = Some(breach);
            }

            // Handle rubble production
            if state.produces_rubble {
                result.rubble_produced = true;
            }
        }
    }

    result
}

/// Find the appropriate damage state for a given HP ratio.
///
/// The threshold represents the HP ratio boundary where you enter each state.
/// States are checked from lowest threshold to highest. You're in a state when
/// your HP ratio is greater than its threshold (or equal for the lowest state).
///
/// For example with states [destroyed=0.0, breached=0.25, damaged=0.5, intact=1.0]:
/// - HP ratio 1.0 -> intact (at threshold 1.0)
/// - HP ratio 0.6 -> damaged (above 0.5, below 1.0)
/// - HP ratio 0.5 -> damaged (at threshold 0.5)
/// - HP ratio 0.3 -> damaged (above 0.25, below 0.5)
/// - HP ratio 0.25 -> breached (at threshold 0.25)
/// - HP ratio 0.2 -> breached (above 0.0, at or below 0.25)
/// - HP ratio 0.0 -> destroyed (at threshold 0.0)
///
/// # Arguments
/// * `states` - List of damage state definitions from the blueprint
/// * `hp_ratio` - Current HP as a ratio of max HP (0.0 to 1.0)
///
/// # Returns
/// The matching damage state definition, or None if no states defined
pub fn find_damage_state<'a>(
    states: &'a [DamageStateDef],
    hp_ratio: f32,
) -> Option<&'a DamageStateDef> {
    if states.is_empty() {
        return None;
    }

    // Sort states by threshold ascending (lowest first: 0.0, 0.25, 0.5, 1.0)
    let mut sorted: Vec<&DamageStateDef> = states.iter().collect();
    sorted.sort_by(|a, b| {
        a.threshold
            .partial_cmp(&b.threshold)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Find the state with the highest threshold that hp_ratio is greater than
    // This is the state we've "fallen past" (or are at)
    let mut result = sorted.first().copied();
    for state in &sorted {
        if hp_ratio >= state.threshold {
            result = Some(*state);
        } else {
            break;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprints::*;
    use std::collections::HashMap;

    fn create_test_blueprint() -> Blueprint {
        let toml_str = r#"
[meta]
id = "test_wall"
name = "Test Wall"
category = "wall"

[parameters]
length = { type = "float", min = 1.0, max = 20.0, default = 5.0 }
height = { type = "float", min = 1.0, max = 5.0, default = 2.5 }

[geometry]
width = "length"
depth = "0.5"
height = "height"

[stats.military]
max_hp = "100"
cover_value = "1.0"
blocks_movement = "1"

[[damage_states]]
name = "intact"
threshold = 1.0

[[damage_states]]
name = "damaged"
threshold = 0.5

[[damage_states]]
name = "breached"
threshold = 0.25
creates_breach = true
produces_rubble = true
overrides = { cover_value = 0.5, blocks_movement = false }

[[damage_states]]
name = "destroyed"
threshold = 0.0
"#;
        toml::from_str(toml_str).unwrap()
    }

    #[test]
    fn test_find_damage_state() {
        let blueprint = create_test_blueprint();

        // Threshold defines lower bound of HP range for each state:
        // - intact (1.0): hp >= 1.0
        // - damaged (0.5): 0.5 <= hp < 1.0
        // - breached (0.25): 0.25 <= hp < 0.5
        // - destroyed (0.0): hp < 0.25

        // At full HP (1.0), should be intact
        let state = find_damage_state(&blueprint.damage_states, 1.0);
        assert_eq!(state.unwrap().name, "intact");

        // At 60% HP (0.6 >= 0.5), should be damaged
        let state = find_damage_state(&blueprint.damage_states, 0.6);
        assert_eq!(state.unwrap().name, "damaged");

        // At 50% HP (exactly at damaged threshold), should be damaged
        let state = find_damage_state(&blueprint.damage_states, 0.5);
        assert_eq!(state.unwrap().name, "damaged");

        // At 30% HP (0.3 >= 0.25), should be breached
        let state = find_damage_state(&blueprint.damage_states, 0.3);
        assert_eq!(state.unwrap().name, "breached");

        // At 25% HP (exactly at breached threshold), should be breached
        let state = find_damage_state(&blueprint.damage_states, 0.25);
        assert_eq!(state.unwrap().name, "breached");

        // At 20% HP (0.2 >= 0.0 but < 0.25), should be destroyed
        let state = find_damage_state(&blueprint.damage_states, 0.2);
        assert_eq!(state.unwrap().name, "destroyed");

        // At 0% HP, should be destroyed
        let state = find_damage_state(&blueprint.damage_states, 0.0);
        assert_eq!(state.unwrap().name, "destroyed");
    }

    #[test]
    fn test_find_damage_state_empty() {
        let state = find_damage_state(&[], 0.5);
        assert!(state.is_none());
    }

    #[test]
    fn test_apply_damage_state_transition() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();
        let id = registry.register(blueprint.clone());

        let mut instance = registry
            .instantiate(
                id,
                HashMap::new(),
                Vec2::ZERO,
                0.0,
                PlacedBy::TerrainGen,
                None,
            )
            .unwrap();

        // Force construction complete so HP is at max
        instance.construction_progress = 1.0;
        instance.current_hp = instance.max_hp;

        // Start at full HP
        assert_eq!(instance.damage_state, "intact");
        assert_eq!(instance.current_hp, 100.0);

        // Damage to 60% HP - should transition to "damaged" (60% >= 0.5 threshold)
        let result = apply_damage(&mut instance, 40.0, Vec2::ZERO, &blueprint);
        assert_eq!(instance.damage_state, "damaged");
        assert_eq!(result.new_state, Some("damaged".to_string()));
        assert!(!result.destroyed);
        assert_eq!(instance.current_hp, 60.0);

        // More damage to 55% HP - should stay damaged (55% >= 0.5)
        let result = apply_damage(&mut instance, 5.0, Vec2::ZERO, &blueprint);
        assert_eq!(instance.damage_state, "damaged");
        assert!(result.new_state.is_none()); // No state change
        assert!(!result.destroyed);
        assert_eq!(instance.current_hp, 55.0);

        // Damage to 30% HP - should transition to breached (30% >= 0.25, < 0.5)
        let result = apply_damage(&mut instance, 25.0, Vec2::new(1.0, 0.0), &blueprint);
        assert_eq!(instance.damage_state, "breached");
        assert_eq!(result.new_state, Some("breached".to_string()));
        assert!(result.new_breach.is_some());
        assert!(result.rubble_produced);
        assert!(!instance.military.blocks_movement); // Override applied
        assert_eq!(instance.military.cover_value, 0.5); // Override applied
        assert!(!result.destroyed);
        assert_eq!(instance.current_hp, 30.0);

        // Verify breach was added
        assert_eq!(instance.breaches.len(), 1);
        assert_eq!(instance.breaches[0].position, Vec2::new(1.0, 0.0));

        // Damage to 20% HP - should transition to destroyed (20% < 0.25)
        let result = apply_damage(&mut instance, 10.0, Vec2::ZERO, &blueprint);
        assert_eq!(instance.damage_state, "destroyed");
        assert_eq!(result.new_state, Some("destroyed".to_string()));
        assert!(!result.destroyed); // Not yet at 0 HP
        assert_eq!(instance.current_hp, 20.0);

        // Finish destroying
        let result = apply_damage(&mut instance, 100.0, Vec2::ZERO, &blueprint);
        assert!(result.destroyed);
        assert_eq!(instance.current_hp, 0.0);
    }

    #[test]
    fn test_apply_damage_no_state_change() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();
        let id = registry.register(blueprint.clone());

        let mut instance = registry
            .instantiate(
                id,
                HashMap::new(),
                Vec2::ZERO,
                0.0,
                PlacedBy::TerrainGen,
                None,
            )
            .unwrap();

        // Force construction complete and set to damaged state (60% HP)
        instance.construction_progress = 1.0;
        instance.current_hp = 60.0;
        instance.damage_state = "damaged".to_string();

        // Small damage that keeps us in damaged range (60% -> 55%, both >= 0.5)
        let result = apply_damage(&mut instance, 5.0, Vec2::ZERO, &blueprint);
        assert!(result.new_state.is_none()); // No state change
        assert!(!result.destroyed);
        assert!(result.new_breach.is_none());
        assert!(!result.rubble_produced);
        assert_eq!(instance.damage_state, "damaged");
        assert_eq!(instance.current_hp, 55.0);
    }

    #[test]
    fn test_apply_damage_multiple_breaches() {
        let mut registry = BlueprintRegistry::new();
        let blueprint = create_test_blueprint();
        let id = registry.register(blueprint.clone());

        let mut instance = registry
            .instantiate(
                id,
                HashMap::new(),
                Vec2::ZERO,
                0.0,
                PlacedBy::TerrainGen,
                None,
            )
            .unwrap();

        // Force to breached state
        instance.construction_progress = 1.0;
        instance.current_hp = 24.0;
        instance.damage_state = "breached".to_string();
        instance.breaches.push(Breach {
            position: Vec2::new(0.0, 0.0),
            width: 2.0,
        });

        // More damage at different location - won't create new breach since
        // we're already in breached state
        let result = apply_damage(&mut instance, 10.0, Vec2::new(5.0, 0.0), &blueprint);

        // State didn't change (still breached), so no new breach
        assert!(result.new_breach.is_none());
        assert_eq!(instance.breaches.len(), 1);
    }

    #[test]
    fn test_damage_result_fields() {
        let result = DamageResult {
            new_state: Some("damaged".to_string()),
            destroyed: false,
            new_breach: Some(Breach {
                position: Vec2::new(1.0, 2.0),
                width: 2.0,
            }),
            rubble_produced: true,
        };

        assert_eq!(result.new_state.as_deref(), Some("damaged"));
        assert!(!result.destroyed);
        assert!(result.new_breach.is_some());
        assert!(result.rubble_produced);
    }
}
