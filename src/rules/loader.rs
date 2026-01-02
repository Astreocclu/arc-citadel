//! Load species rules from TOML files

use crate::actions::catalog::ActionId;
use crate::core::types::Species;
use crate::entity::tasks::TaskPriority;
use crate::rules::action_rules::{ActionRule, IdleBehavior, SpeciesRuleSet, SpeciesRules};
use crate::rules::value_dynamics::{TickDelta, ValueEvent, SpeciesDynamics, ValueDynamicsRules};
use std::fs;
use std::path::Path;

/// Load all species rules from the species/ directory
pub fn load_species_rules(species_dir: &Path) -> Result<SpeciesRules, String> {
    let mut rules = SpeciesRules::new();

    // Map of TOML file names to Species enum
    let species_files = [
        ("gnoll.toml", Species::Gnoll),
        ("vampire_llm.toml", Species::Vampire),
        ("kobold.toml", Species::Kobold),
        ("human.toml", Species::Human),
        ("lizardfolk.toml", Species::Lizardfolk),
        ("hobgoblin.toml", Species::Hobgoblin),
        ("ogre.toml", Species::Ogre),
        ("harpy.toml", Species::Harpy),
        ("centaur.toml", Species::Centaur),
        ("minotaur.toml", Species::Minotaur),
        ("satyr.toml", Species::Satyr),
        ("dryad.toml", Species::Dryad),
        ("demon_llm.toml", Species::AbyssalDemons),
        ("elemental_llm.toml", Species::Elemental),
        ("fey_llm.toml", Species::Fey),
        ("giant_llm.toml", Species::StoneGiants),
        ("golem_llm.toml", Species::Golem),
        ("merfolk_llm.toml", Species::Merfolk),
        ("naga_llm.toml", Species::Naga),
    ];

    for (filename, species) in species_files {
        let path = species_dir.join(filename);
        if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read {}: {}", filename, e))?;
            let rule_set = parse_species_toml(&content, species)?;
            rules.insert(species, rule_set);
        }
    }

    Ok(rules)
}

/// Load all species value dynamics from the species/ directory
pub fn load_species_dynamics(species_dir: &Path) -> Result<ValueDynamicsRules, String> {
    let mut dynamics = ValueDynamicsRules::new();

    // Map of TOML file names to Species enum (same as rules)
    let species_files = [
        ("gnoll.toml", Species::Gnoll),
        ("vampire_llm.toml", Species::Vampire),
        ("kobold.toml", Species::Kobold),
        ("human.toml", Species::Human),
        ("lizardfolk.toml", Species::Lizardfolk),
        ("hobgoblin.toml", Species::Hobgoblin),
        ("ogre.toml", Species::Ogre),
        ("harpy.toml", Species::Harpy),
        ("centaur.toml", Species::Centaur),
        ("minotaur.toml", Species::Minotaur),
        ("satyr.toml", Species::Satyr),
        ("dryad.toml", Species::Dryad),
        ("demon_llm.toml", Species::AbyssalDemons),
        ("elemental_llm.toml", Species::Elemental),
        ("fey_llm.toml", Species::Fey),
        ("giant_llm.toml", Species::StoneGiants),
        ("golem_llm.toml", Species::Golem),
        ("merfolk_llm.toml", Species::Merfolk),
        ("naga_llm.toml", Species::Naga),
    ];

    for (filename, species) in species_files {
        let path = species_dir.join(filename);
        if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read {}: {}", filename, e))?;
            let species_dynamics = parse_species_dynamics(&content, species)?;
            dynamics.insert(species, species_dynamics);
        }
    }

    Ok(dynamics)
}

fn parse_species_dynamics(content: &str, species: Species) -> Result<SpeciesDynamics, String> {
    let toml: toml::Value = content.parse()
        .map_err(|e| format!("{:?}: Invalid TOML: {}", species, e))?;

    let mut dynamics = SpeciesDynamics::default();

    // Parse value_dynamics section
    if let Some(dyn_table) = toml.get("value_dynamics").and_then(|v| v.as_table()) {
        for (name, config) in dyn_table {
            if let Some(table) = config.as_table() {
                let tick_delta = table.get("tick_delta")
                    .and_then(|v| v.as_float())
                    .unwrap_or(0.0) as f32;
                let min = table.get("min")
                    .and_then(|v| v.as_float())
                    .unwrap_or(0.0) as f32;
                let max = table.get("max")
                    .and_then(|v| v.as_float())
                    .unwrap_or(1.0) as f32;

                dynamics.tick_deltas.push(TickDelta {
                    value_name: name.clone(),
                    delta: tick_delta,
                    min,
                    max,
                });
            }
        }
    }

    // Parse value_events array
    if let Some(events) = toml.get("value_events").and_then(|v| v.as_array()) {
        for event in events {
            if let (Some(event_type), Some(value_name), Some(delta)) = (
                event.get("event").and_then(|v| v.as_str()),
                event.get("value").and_then(|v| v.as_str()),
                event.get("delta").and_then(|v| v.as_float()),
            ) {
                dynamics.events.push(ValueEvent {
                    event_type: event_type.to_string(),
                    value_name: value_name.to_string(),
                    delta: delta as f32,
                });
            }
        }
    }

    Ok(dynamics)
}

fn parse_species_toml(content: &str, species: Species) -> Result<SpeciesRuleSet, String> {
    let toml: toml::Value = content.parse()
        .map_err(|e| format!("{:?}: Invalid TOML: {}", species, e))?;

    let mut rule_set = SpeciesRuleSet::default();

    // Parse action_rules
    if let Some(rules) = toml.get("action_rules").and_then(|v| v.as_array()) {
        for rule in rules {
            rule_set.action_rules.push(parse_action_rule(rule, species)?);
        }
    }

    // Parse idle_behaviors
    if let Some(behaviors) = toml.get("idle_behaviors").and_then(|v| v.as_array()) {
        for behavior in behaviors {
            rule_set.idle_behaviors.push(parse_idle_behavior(behavior, species)?);
        }
    }

    Ok(rule_set)
}

fn parse_action_rule(value: &toml::Value, species: Species) -> Result<ActionRule, String> {
    let trigger_value = value.get("trigger_value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("{:?}: action_rule missing trigger_value", species))?
        .to_string();

    let threshold = value.get("threshold")
        .and_then(|v| v.as_float())
        .ok_or_else(|| format!("{:?}: action_rule missing threshold", species))?
        as f32;

    let action_str = value.get("action")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("{:?}: action_rule missing action", species))?;

    let action = parse_action_id(action_str)
        .ok_or_else(|| format!("{:?}: Unknown action '{}'", species, action_str))?;

    let priority_str = value.get("priority")
        .and_then(|v| v.as_str())
        .unwrap_or("Normal");

    let priority = match priority_str {
        "Critical" => TaskPriority::Critical,
        "High" => TaskPriority::High,
        "Normal" => TaskPriority::Normal,
        "Low" => TaskPriority::Low,
        _ => TaskPriority::Normal,
    };

    let requires_target = value.get("requires_target")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let description = value.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(ActionRule {
        trigger_value,
        threshold,
        action,
        priority,
        requires_target,
        description,
    })
}

fn parse_idle_behavior(value: &toml::Value, species: Species) -> Result<IdleBehavior, String> {
    let val = value.get("value")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("{:?}: idle_behavior missing value", species))?
        .to_string();

    let threshold = value.get("threshold")
        .and_then(|v| v.as_float())
        .ok_or_else(|| format!("{:?}: idle_behavior missing threshold", species))?
        as f32;

    let action_str = value.get("action")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("{:?}: idle_behavior missing action", species))?;

    let action = parse_action_id(action_str)
        .ok_or_else(|| format!("{:?}: Unknown action '{}'", species, action_str))?;

    let requires_target = value.get("requires_target")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let description = value.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(IdleBehavior {
        value: val,
        threshold,
        action,
        requires_target,
        description,
    })
}

fn parse_action_id(s: &str) -> Option<ActionId> {
    match s {
        "MoveTo" => Some(ActionId::MoveTo),
        "Follow" => Some(ActionId::Follow),
        "Flee" => Some(ActionId::Flee),
        "Rest" => Some(ActionId::Rest),
        "Eat" => Some(ActionId::Eat),
        "SeekSafety" => Some(ActionId::SeekSafety),
        "Build" => Some(ActionId::Build),
        "Craft" => Some(ActionId::Craft),
        "Gather" => Some(ActionId::Gather),
        "Repair" => Some(ActionId::Repair),
        "TalkTo" => Some(ActionId::TalkTo),
        "Help" => Some(ActionId::Help),
        "Trade" => Some(ActionId::Trade),
        "Attack" => Some(ActionId::Attack),
        "Defend" => Some(ActionId::Defend),
        "Charge" => Some(ActionId::Charge),
        "HoldPosition" => Some(ActionId::HoldPosition),
        "IdleWander" => Some(ActionId::IdleWander),
        "IdleObserve" => Some(ActionId::IdleObserve),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_action_rule() {
        let toml_str = r#"
[[action_rules]]
trigger_value = "bloodlust"
threshold = 0.7
action = "Attack"
priority = "High"
requires_target = true
description = "Enter killing frenzy"
"#;
        let toml: toml::Value = toml_str.parse().unwrap();
        let rules = toml.get("action_rules").unwrap().as_array().unwrap();
        let rule = parse_action_rule(&rules[0], Species::Gnoll).unwrap();

        assert_eq!(rule.trigger_value, "bloodlust");
        assert!((rule.threshold - 0.7).abs() < 0.01);
        assert!(matches!(rule.action, ActionId::Attack));
        assert!(matches!(rule.priority, TaskPriority::High));
        assert!(rule.requires_target);
    }

    #[test]
    fn test_parse_idle_behavior() {
        let toml_str = r#"
[[idle_behaviors]]
value = "hunger"
threshold = 0.4
action = "IdleWander"
requires_target = false
description = "Prowl territory seeking prey."
"#;
        let toml: toml::Value = toml_str.parse().unwrap();
        let behaviors = toml.get("idle_behaviors").unwrap().as_array().unwrap();
        let behavior = parse_idle_behavior(&behaviors[0], Species::Gnoll).unwrap();

        assert_eq!(behavior.value, "hunger");
        assert!((behavior.threshold - 0.4).abs() < 0.01);
        assert!(matches!(behavior.action, ActionId::IdleWander));
        assert!(!behavior.requires_target);
    }

    #[test]
    fn test_load_species_rules_from_directory() {
        let species_dir = Path::new("species");
        if species_dir.exists() {
            let rules = load_species_rules(species_dir).unwrap();

            // Check gnoll rules loaded
            let gnoll_rules = rules.get_action_rules(Species::Gnoll);
            assert!(!gnoll_rules.is_empty(), "Gnoll should have action rules");

            // Check vampire rules loaded
            let vampire_rules = rules.get_action_rules(Species::Vampire);
            assert!(!vampire_rules.is_empty(), "Vampire should have action rules");

            // Check kobold rules loaded
            let kobold_rules = rules.get_action_rules(Species::Kobold);
            assert!(!kobold_rules.is_empty(), "Kobold should have action rules");
        }
    }

    #[test]
    fn test_parse_action_id() {
        assert!(matches!(parse_action_id("Attack"), Some(ActionId::Attack)));
        assert!(matches!(parse_action_id("Flee"), Some(ActionId::Flee)));
        assert!(matches!(parse_action_id("IdleWander"), Some(ActionId::IdleWander)));
        assert!(parse_action_id("NonexistentAction").is_none());
    }

    #[test]
    fn test_parse_value_dynamics() {
        let toml_str = r#"
[value_dynamics]
bloodlust = { tick_delta = 0.002, min = 0.0, max = 1.0 }
hunger = { tick_delta = 0.003, min = 0.0, max = 1.0 }

[[value_events]]
event = "combat_victory"
value = "bloodlust"
delta = 0.15

[[value_events]]
event = "feeding"
value = "hunger"
delta = -0.4
"#;
        let dynamics = parse_species_dynamics(toml_str, Species::Gnoll).unwrap();

        assert_eq!(dynamics.tick_deltas.len(), 2);
        assert_eq!(dynamics.events.len(), 2);

        // Check tick deltas
        let bloodlust_delta = dynamics.tick_deltas.iter().find(|d| d.value_name == "bloodlust").unwrap();
        assert!((bloodlust_delta.delta - 0.002).abs() < 0.0001);
        assert!((bloodlust_delta.min - 0.0).abs() < 0.0001);
        assert!((bloodlust_delta.max - 1.0).abs() < 0.0001);

        // Check events
        let combat_event = dynamics.events.iter().find(|e| e.event_type == "combat_victory").unwrap();
        assert_eq!(combat_event.value_name, "bloodlust");
        assert!((combat_event.delta - 0.15).abs() < 0.0001);
    }

    #[test]
    fn test_load_species_dynamics_from_directory() {
        let species_dir = Path::new("species");
        if species_dir.exists() {
            let dynamics = load_species_dynamics(species_dir).unwrap();

            // Check gnoll dynamics loaded
            let gnoll_deltas = dynamics.get_tick_deltas(Species::Gnoll);
            assert!(!gnoll_deltas.is_empty(), "Gnoll should have tick deltas");

            // Check vampire dynamics loaded
            let vampire_deltas = dynamics.get_tick_deltas(Species::Vampire);
            assert!(!vampire_deltas.is_empty(), "Vampire should have tick deltas");

            // Check event lookup
            let gnoll_combat_events = dynamics.get_events_for_type(Species::Gnoll, "combat_victory");
            assert!(!gnoll_combat_events.is_empty(), "Gnoll should have combat_victory events");
        }
    }
}
