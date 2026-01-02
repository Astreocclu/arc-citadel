//! Action rule definitions and storage

use crate::actions::catalog::ActionId;
use crate::core::types::Species;
use crate::entity::tasks::TaskPriority;
use std::collections::HashMap;

/// A single action rule loaded from TOML
#[derive(Debug, Clone)]
pub struct ActionRule {
    pub trigger_value: String,
    pub threshold: f32,
    pub action: ActionId,
    pub priority: TaskPriority,
    pub requires_target: bool,
    pub description: String,
}

/// An idle behavior rule
#[derive(Debug, Clone)]
pub struct IdleBehavior {
    pub value: String,
    pub threshold: f32,
    pub action: ActionId,
    pub requires_target: bool,
    pub description: String,
}

/// All rules for a single species
#[derive(Debug, Clone, Default)]
pub struct SpeciesRuleSet {
    pub action_rules: Vec<ActionRule>,
    pub idle_behaviors: Vec<IdleBehavior>,
}

/// Central storage for all species rules
#[derive(Debug, Default)]
pub struct SpeciesRules {
    rules: HashMap<Species, SpeciesRuleSet>,
}

impl SpeciesRules {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get action rules for a species
    pub fn get_action_rules(&self, species: Species) -> &[ActionRule] {
        self.rules
            .get(&species)
            .map(|r| r.action_rules.as_slice())
            .unwrap_or(&[])
    }

    /// Get idle behaviors for a species
    pub fn get_idle_behaviors(&self, species: Species) -> &[IdleBehavior] {
        self.rules
            .get(&species)
            .map(|r| r.idle_behaviors.as_slice())
            .unwrap_or(&[])
    }

    /// Insert rules for a species
    pub fn insert(&mut self, species: Species, rules: SpeciesRuleSet) {
        self.rules.insert(species, rules);
    }

    /// Validate that all trigger_value fields exist in the species' ValueAccessor
    pub fn validate<V: crate::entity::species::value_access::ValueAccessor>(
        &self,
        species: Species,
    ) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let valid_fields = V::field_names();

        if let Some(rule_set) = self.rules.get(&species) {
            for rule in &rule_set.action_rules {
                if !valid_fields.contains(&rule.trigger_value.as_str()) {
                    errors.push(format!(
                        "{:?}: Unknown trigger_value '{}' in action rule",
                        species, rule.trigger_value
                    ));
                }
            }
            for behavior in &rule_set.idle_behaviors {
                if !valid_fields.contains(&behavior.value.as_str()) {
                    errors.push(format!(
                        "{:?}: Unknown value '{}' in idle behavior",
                        species, behavior.value
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::species::gnoll::GnollValues;
    use crate::entity::species::value_access::ValueAccessor;

    #[test]
    fn test_species_rules_empty_by_default() {
        let rules = SpeciesRules::new();
        assert!(rules.get_action_rules(Species::Gnoll).is_empty());
        assert!(rules.get_idle_behaviors(Species::Gnoll).is_empty());
    }

    #[test]
    fn test_species_rules_insert_and_retrieve() {
        let mut rules = SpeciesRules::new();

        let rule_set = SpeciesRuleSet {
            action_rules: vec![ActionRule {
                trigger_value: "bloodlust".to_string(),
                threshold: 0.7,
                action: ActionId::Attack,
                priority: TaskPriority::High,
                requires_target: true,
                description: "Attack when bloodlust is high".to_string(),
            }],
            idle_behaviors: vec![],
        };

        rules.insert(Species::Gnoll, rule_set);

        let action_rules = rules.get_action_rules(Species::Gnoll);
        assert_eq!(action_rules.len(), 1);
        assert_eq!(action_rules[0].trigger_value, "bloodlust");
    }

    #[test]
    fn test_validate_valid_rules() {
        let mut rules = SpeciesRules::new();

        let rule_set = SpeciesRuleSet {
            action_rules: vec![ActionRule {
                trigger_value: "bloodlust".to_string(),
                threshold: 0.7,
                action: ActionId::Attack,
                priority: TaskPriority::High,
                requires_target: true,
                description: "Attack".to_string(),
            }],
            idle_behaviors: vec![],
        };

        rules.insert(Species::Gnoll, rule_set);

        assert!(rules.validate::<GnollValues>(Species::Gnoll).is_ok());
    }

    #[test]
    fn test_validate_invalid_rules() {
        let mut rules = SpeciesRules::new();

        let rule_set = SpeciesRuleSet {
            action_rules: vec![ActionRule {
                trigger_value: "nonexistent_value".to_string(),
                threshold: 0.7,
                action: ActionId::Attack,
                priority: TaskPriority::High,
                requires_target: true,
                description: "Attack".to_string(),
            }],
            idle_behaviors: vec![],
        };

        rules.insert(Species::Gnoll, rule_set);

        let result = rules.validate::<GnollValues>(Species::Gnoll);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("nonexistent_value"));
    }
}
