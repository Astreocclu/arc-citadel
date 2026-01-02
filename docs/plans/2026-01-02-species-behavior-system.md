# Species Behavior System Implementation Plan

**Date:** 2026-01-02
**Status:** Ready for implementation
**Scope:** Framework for all 24 species, full implementation for 3 demo species (Gnoll, Vampire, Kobold)

---

## Overview

This plan fixes 4 critical issues in the species system:

| Phase | Issue | Solution |
|-------|-------|----------|
| A | Action rules hardcoded in Rust | Runtime-loaded from TOML |
| B | Values never change | Hybrid dynamics (tick + events) |
| C | Defaults below thresholds | Rebalance TOML files |
| D | Polity behavior stubs | Implement faction logic |

**Sequence:** A → B → C → D (each phase builds on previous)

---

## Phase A: Runtime Action Thresholds

### Task A1: Create ValueAccessor trait

**File:** `src/entity/species/value_access.rs` (NEW)

```rust
//! Runtime value access for species-specific value structs

/// Trait for runtime access to species values by field name
pub trait ValueAccessor {
    /// Get a value by field name, returns None if field doesn't exist
    fn get_value(&self, field_name: &str) -> Option<f32>;

    /// Set a value by field name, returns false if field doesn't exist
    fn set_value(&mut self, field_name: &str, value: f32) -> bool;

    /// List all field names for validation
    fn field_names() -> &'static [&'static str];
}
```

**Verification:**
```bash
cargo build 2>&1 | grep -E "(error|warning.*value_access)"
# Expected: no errors
```

---

### Task A2: Implement ValueAccessor for demo species

**File:** `src/entity/species/vampire.rs` (MODIFY)

Add after `VampireValues` struct:

```rust
impl crate::entity::species::value_access::ValueAccessor for VampireValues {
    fn get_value(&self, field_name: &str) -> Option<f32> {
        match field_name {
            "bloodthirst" => Some(self.bloodthirst),
            "arrogance" => Some(self.arrogance),
            "secrecy" => Some(self.secrecy),
            "dominance" => Some(self.dominance),
            "ennui" => Some(self.ennui),
            _ => None,
        }
    }

    fn set_value(&mut self, field_name: &str, value: f32) -> bool {
        match field_name {
            "bloodthirst" => { self.bloodthirst = value; true }
            "arrogance" => { self.arrogance = value; true }
            "secrecy" => { self.secrecy = value; true }
            "dominance" => { self.dominance = value; true }
            "ennui" => { self.ennui = value; true }
            _ => false,
        }
    }

    fn field_names() -> &'static [&'static str] {
        &["bloodthirst", "arrogance", "secrecy", "dominance", "ennui"]
    }
}
```

**File:** `src/entity/species/gnoll.rs` (MODIFY)

```rust
impl crate::entity::species::value_access::ValueAccessor for GnollValues {
    fn get_value(&self, field_name: &str) -> Option<f32> {
        match field_name {
            "bloodlust" => Some(self.bloodlust),
            "pack_instinct" => Some(self.pack_instinct),
            "hunger" => Some(self.hunger),
            "cruelty" => Some(self.cruelty),
            "dominance" => Some(self.dominance),
            _ => None,
        }
    }

    fn set_value(&mut self, field_name: &str, value: f32) -> bool {
        match field_name {
            "bloodlust" => { self.bloodlust = value; true }
            "pack_instinct" => { self.pack_instinct = value; true }
            "hunger" => { self.hunger = value; true }
            "cruelty" => { self.cruelty = value; true }
            "dominance" => { self.dominance = value; true }
            _ => false,
        }
    }

    fn field_names() -> &'static [&'static str] {
        &["bloodlust", "pack_instinct", "hunger", "cruelty", "dominance"]
    }
}
```

**File:** `src/entity/species/kobold.rs` (MODIFY)

```rust
impl crate::entity::species::value_access::ValueAccessor for KoboldValues {
    fn get_value(&self, field_name: &str) -> Option<f32> {
        match field_name {
            "cunning" => Some(self.cunning),
            "cowardice" => Some(self.cowardice),
            "industriousness" => Some(self.industriousness),
            "pack_loyalty" => Some(self.pack_loyalty),
            "spite" => Some(self.spite),
            _ => None,
        }
    }

    fn set_value(&mut self, field_name: &str, value: f32) -> bool {
        match field_name {
            "cunning" => { self.cunning = value; true }
            "cowardice" => { self.cowardice = value; true }
            "industriousness" => { self.industriousness = value; true }
            "pack_loyalty" => { self.pack_loyalty = value; true }
            "spite" => { self.spite = value; true }
            _ => false,
        }
    }

    fn field_names() -> &'static [&'static str] {
        &["cunning", "cowardice", "industriousness", "pack_loyalty", "spite"]
    }
}
```

**Verification:**
```bash
cargo test value_access 2>&1
# Expected: tests pass
```

---

### Task A3: Create SpeciesRules struct

**File:** `src/rules/mod.rs` (NEW)

```rust
//! Runtime species rules loaded from TOML

mod action_rules;
mod loader;

pub use action_rules::{ActionRule, SpeciesRules};
pub use loader::load_species_rules;
```

**File:** `src/rules/action_rules.rs` (NEW)

```rust
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
```

**Verification:**
```bash
cargo build 2>&1 | grep -E "error"
# Expected: no errors
```

---

### Task A4: Create TOML loader

**File:** `src/rules/loader.rs` (NEW)

```rust
//! Load species rules from TOML files

use crate::actions::catalog::ActionId;
use crate::core::types::Species;
use crate::entity::tasks::TaskPriority;
use crate::rules::action_rules::{ActionRule, IdleBehavior, SpeciesRuleSet, SpeciesRules};
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
        // Add more as needed
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
}
```

**Verification:**
```bash
cargo test loader 2>&1
# Expected: test_parse_action_rule passes
```

---

### Task A5: Add SpeciesRules to World

**File:** `src/ecs/world.rs` (MODIFY)

Add field to World struct:

```rust
use crate::rules::SpeciesRules;

pub struct World {
    // ... existing fields ...

    /// Runtime-loaded species action rules
    pub species_rules: SpeciesRules,
}
```

Update World::new() to load rules:

```rust
impl World {
    pub fn new() -> Self {
        // Load species rules from TOML files
        let species_dir = std::path::Path::new("species");
        let species_rules = crate::rules::load_species_rules(species_dir)
            .unwrap_or_else(|e| {
                eprintln!("Warning: Failed to load species rules: {}", e);
                SpeciesRules::new()
            });

        Self {
            // ... existing initialization ...
            species_rules,
        }
    }
}
```

**Verification:**
```bash
cargo build 2>&1 | grep -E "error"
cargo test world 2>&1
# Expected: compiles and tests pass
```

---

### Task A6: Create generic rule evaluator

**File:** `src/simulation/rule_eval.rs` (NEW)

```rust
//! Generic rule evaluation using ValueAccessor

use crate::actions::catalog::ActionId;
use crate::core::types::Tick;
use crate::entity::species::value_access::ValueAccessor;
use crate::entity::tasks::{Task, TaskPriority, TaskSource};
use crate::rules::action_rules::{ActionRule, IdleBehavior};

/// Evaluate action rules against current values
/// Returns the first matching action (rules are priority-ordered)
pub fn evaluate_action_rules<V: ValueAccessor>(
    values: &V,
    rules: &[ActionRule],
    current_tick: Tick,
    entity_nearby: bool,
) -> Option<Task> {
    for rule in rules {
        // Skip rules requiring target if no entity nearby
        if rule.requires_target && !entity_nearby {
            continue;
        }

        if let Some(val) = values.get_value(&rule.trigger_value) {
            if val > rule.threshold {
                return Some(Task {
                    action: rule.action,
                    target_position: None,
                    target_entity: None,
                    priority: rule.priority,
                    created_tick: current_tick,
                    progress: 0.0,
                    source: TaskSource::Autonomous,
                });
            }
        }
    }
    None
}

/// Select idle behavior based on values
pub fn select_idle_behavior<V: ValueAccessor>(
    values: &V,
    behaviors: &[IdleBehavior],
    current_tick: Tick,
) -> Task {
    for behavior in behaviors {
        if let Some(val) = values.get_value(&behavior.value) {
            if val > behavior.threshold {
                return Task {
                    action: behavior.action,
                    target_position: None,
                    target_entity: None,
                    priority: TaskPriority::Low,
                    created_tick: current_tick,
                    progress: 0.0,
                    source: TaskSource::Autonomous,
                };
            }
        }
    }

    // Default idle behavior
    Task {
        action: ActionId::IdleWander,
        target_position: None,
        target_entity: None,
        priority: TaskPriority::Low,
        created_tick: current_tick,
        progress: 0.0,
        source: TaskSource::Autonomous,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::species::gnoll::GnollValues;
    use crate::entity::species::value_access::ValueAccessor;

    #[test]
    fn test_rule_triggers_when_above_threshold() {
        let mut values = GnollValues::default();
        values.bloodlust = 0.8; // Above threshold

        let rules = vec![ActionRule {
            trigger_value: "bloodlust".to_string(),
            threshold: 0.7,
            action: ActionId::Attack,
            priority: TaskPriority::High,
            requires_target: true,
            description: "Attack".to_string(),
        }];

        let task = evaluate_action_rules(&values, &rules, Tick(0), true);
        assert!(task.is_some());
        assert!(matches!(task.unwrap().action, ActionId::Attack));
    }

    #[test]
    fn test_rule_skipped_when_below_threshold() {
        let mut values = GnollValues::default();
        values.bloodlust = 0.5; // Below threshold

        let rules = vec![ActionRule {
            trigger_value: "bloodlust".to_string(),
            threshold: 0.7,
            action: ActionId::Attack,
            priority: TaskPriority::High,
            requires_target: true,
            description: "Attack".to_string(),
        }];

        let task = evaluate_action_rules(&values, &rules, Tick(0), true);
        assert!(task.is_none());
    }

    #[test]
    fn test_requires_target_skipped_when_no_entity() {
        let mut values = GnollValues::default();
        values.bloodlust = 0.8;

        let rules = vec![ActionRule {
            trigger_value: "bloodlust".to_string(),
            threshold: 0.7,
            action: ActionId::Attack,
            priority: TaskPriority::High,
            requires_target: true,
            description: "Attack".to_string(),
        }];

        let task = evaluate_action_rules(&values, &rules, Tick(0), false); // No entity nearby
        assert!(task.is_none());
    }
}
```

**Verification:**
```bash
cargo test rule_eval 2>&1
# Expected: all 3 tests pass
```

---

### Task A7: Wire up rule evaluation in action_select.rs

**File:** `src/simulation/action_select.rs` (MODIFY)

Add import at top:

```rust
use crate::rules::SpeciesRules;
use crate::simulation::rule_eval::{evaluate_action_rules, select_idle_behavior};
```

Create new species-generic selection function:

```rust
/// Generic action selection using runtime rules
pub fn select_action_with_rules<V: crate::entity::species::value_access::ValueAccessor>(
    values: &V,
    needs: &Needs,
    species_rules: &SpeciesRules,
    species: Species,
    has_current_task: bool,
    threat_nearby: bool,
    food_available: bool,
    entity_nearby: bool,
    current_tick: Tick,
) -> Option<Task> {
    // Critical needs always take priority (existing logic)
    if let Some(critical) = needs.has_critical() {
        return handle_critical_need(critical, threat_nearby, food_available, current_tick);
    }

    // Don't interrupt existing tasks
    if has_current_task {
        return None;
    }

    // Evaluate species-specific action rules from TOML
    let action_rules = species_rules.get_action_rules(species);
    if let Some(task) = evaluate_action_rules(values, action_rules, current_tick, entity_nearby) {
        return Some(task);
    }

    // Fall back to idle behavior from TOML
    let idle_behaviors = species_rules.get_idle_behaviors(species);
    Some(select_idle_behavior(values, idle_behaviors, current_tick))
}

fn handle_critical_need(
    need: NeedType,
    threat_nearby: bool,
    food_available: bool,
    current_tick: Tick,
) -> Option<Task> {
    match need {
        NeedType::Safety if threat_nearby => Some(Task::new(ActionId::Flee, TaskPriority::Critical, current_tick)),
        NeedType::Safety => Some(Task::new(ActionId::SeekSafety, TaskPriority::Critical, current_tick)),
        NeedType::Food if food_available => Some(Task::new(ActionId::Eat, TaskPriority::Critical, current_tick)),
        NeedType::Food => Some(Task::new(ActionId::IdleWander, TaskPriority::Critical, current_tick)),
        NeedType::Rest => Some(Task::new(ActionId::Rest, TaskPriority::Critical, current_tick)),
        _ => None,
    }
}
```

**Verification:**
```bash
cargo build 2>&1 | grep -E "error"
cargo test action_select 2>&1
# Expected: compiles, existing tests still pass
```

---

### Task A8: Update mod.rs files

**File:** `src/entity/species/mod.rs` (MODIFY)

Add:
```rust
pub mod value_access;
```

**File:** `src/simulation/mod.rs` (MODIFY)

Add:
```rust
pub mod rule_eval;
```

**File:** `src/lib.rs` (MODIFY)

Add:
```rust
pub mod rules;
```

**Verification:**
```bash
cargo build 2>&1
cargo test 2>&1
# Expected: all tests pass
```

---

## Phase B: Value Dynamics System

### Task B1: Extend TOML schema for dynamics

**File:** `species/gnoll.toml` (MODIFY)

Add new section:

```toml
[value_dynamics]
bloodlust = { tick_delta = 0.002, min = 0.0, max = 1.0 }
hunger = { tick_delta = 0.003, min = 0.0, max = 1.0 }
pack_instinct = { tick_delta = 0.0, min = 0.0, max = 1.0 }
cruelty = { tick_delta = 0.001, min = 0.0, max = 1.0 }
dominance = { tick_delta = 0.0, min = 0.0, max = 1.0 }

[[value_events]]
event = "combat_victory"
value = "bloodlust"
delta = 0.15

[[value_events]]
event = "combat_victory"
value = "dominance"
delta = 0.1

[[value_events]]
event = "feeding"
value = "hunger"
delta = -0.4
```

**File:** `species/vampire_llm.toml` (MODIFY)

Add:

```toml
[value_dynamics]
bloodthirst = { tick_delta = 0.003, min = 0.0, max = 1.0 }
arrogance = { tick_delta = 0.0, min = 0.0, max = 1.0 }
secrecy = { tick_delta = -0.001, min = 0.2, max = 1.0 }
dominance = { tick_delta = 0.001, min = 0.0, max = 1.0 }
ennui = { tick_delta = 0.002, min = 0.0, max = 1.0 }

[[value_events]]
event = "feeding"
value = "bloodthirst"
delta = -0.5

[[value_events]]
event = "thrall_created"
value = "dominance"
delta = 0.2

[[value_events]]
event = "exposed_to_sunlight"
value = "secrecy"
delta = -0.3
```

**File:** `species/kobold.toml` (MODIFY)

Add:

```toml
[value_dynamics]
cunning = { tick_delta = 0.0, min = 0.0, max = 1.0 }
cowardice = { tick_delta = 0.001, min = 0.0, max = 1.0 }
industriousness = { tick_delta = 0.0, min = 0.0, max = 1.0 }
pack_loyalty = { tick_delta = 0.0, min = 0.0, max = 1.0 }
spite = { tick_delta = 0.002, min = 0.0, max = 1.0 }

[[value_events]]
event = "trap_triggered"
value = "cunning"
delta = 0.1

[[value_events]]
event = "ally_killed"
value = "spite"
delta = 0.3

[[value_events]]
event = "fled_combat"
value = "cowardice"
delta = 0.05
```

---

### Task B2: Create ValueDynamics struct

**File:** `src/rules/value_dynamics.rs` (NEW)

```rust
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
```

---

### Task B3: Extend loader for dynamics

**File:** `src/rules/loader.rs` (MODIFY)

Add to parse_species_toml:

```rust
use crate::rules::value_dynamics::{TickDelta, ValueEvent, SpeciesDynamics};

// In parse_species_toml, add:
fn parse_species_dynamics(toml: &toml::Value, species: Species) -> SpeciesDynamics {
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

    dynamics
}
```

---

### Task B4: Create dynamics application system

**File:** `src/simulation/value_dynamics.rs` (NEW)

```rust
//! Apply value dynamics each tick

use crate::entity::species::value_access::ValueAccessor;
use crate::rules::value_dynamics::{TickDelta, ValueEvent, ValueDynamicsRules};
use crate::core::types::Species;

/// Apply per-tick value changes to an entity
pub fn apply_tick_dynamics<V: ValueAccessor>(
    values: &mut V,
    deltas: &[TickDelta],
) {
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
}
```

**Verification:**
```bash
cargo test value_dynamics 2>&1
# Expected: tests pass
```

---

## Phase C: Rebalance Defaults/Thresholds

### Task C1: Rebalance gnoll.toml

**File:** `species/gnoll.toml` (MODIFY)

Change entity_values defaults to be closer to thresholds:

```toml
[entity_values]
bloodlust = { type = "f32", default = 0.55, description = "Frenzy triggered by scent of blood." }
pack_instinct = { type = "f32", default = 0.65, description = "Coordination with packmates in hunts." }
hunger = { type = "f32", default = 0.5, description = "Constant need to feed." }
cruelty = { type = "f32", default = 0.4, description = "Enjoyment of suffering." }
dominance = { type = "f32", default = 0.35, description = "Drive to establish pack hierarchy." }
```

Adjust thresholds in action_rules:

```toml
[[action_rules]]
trigger_value = "bloodlust"
threshold = 0.6
action = "Attack"
priority = "High"
requires_target = true
description = "Enter killing frenzy when blood is scented."

[[action_rules]]
trigger_value = "hunger"
threshold = 0.55
action = "Gather"
priority = "Normal"
requires_target = true
description = "Hunt for food."

[[action_rules]]
trigger_value = "pack_instinct"
threshold = 0.6
action = "Follow"
priority = "Normal"
requires_target = true
description = "Coordinate with packmates."
```

---

### Task C2: Rebalance vampire_llm.toml

**File:** `species/vampire_llm.toml` (MODIFY)

```toml
[entity_values]
bloodthirst = { type = "f32", default = 0.5, description = "The gnawing hunger for blood." }
arrogance = { type = "f32", default = 0.6, description = "Pride and sense of superiority." }
secrecy = { type = "f32", default = 0.7, description = "The drive to remain hidden." }
dominance = { type = "f32", default = 0.5, description = "The desire to control others." }
ennui = { type = "f32", default = 0.3, description = "The boredom of eternal existence." }

[[action_rules]]
trigger_value = "bloodthirst"
threshold = 0.65
action = "Attack"
priority = "High"
requires_target = true
description = "Overwhelming hunger drives the vampire to feed."

[[action_rules]]
trigger_value = "dominance"
threshold = 0.6
action = "TalkTo"
priority = "Normal"
requires_target = true
description = "Attempt to charm and enthrall a target."

[[action_rules]]
trigger_value = "secrecy"
threshold = 0.75
action = "Flee"
priority = "High"
requires_target = false
description = "Retreat when exposed or threatened."
```

---

### Task C3: Rebalance kobold.toml

**File:** `species/kobold.toml` (MODIFY)

```toml
[entity_values]
cunning = { type = "f32", default = 0.6, description = "Cleverness in setting traps." }
cowardice = { type = "f32", default = 0.7, description = "Fear of direct confrontation." }
industriousness = { type = "f32", default = 0.55, description = "Drive to dig and build." }
pack_loyalty = { type = "f32", default = 0.5, description = "Devotion to the warren." }
spite = { type = "f32", default = 0.35, description = "Desire to harm enemies." }

[[action_rules]]
trigger_value = "cowardice"
threshold = 0.65
action = "Flee"
priority = "High"
requires_target = true
description = "Run when threatened by stronger foes."

[[action_rules]]
trigger_value = "industriousness"
threshold = 0.5
action = "Gather"
priority = "Normal"
requires_target = true
description = "Collect resources for the warren."

[[action_rules]]
trigger_value = "cunning"
threshold = 0.55
action = "IdleObserve"
priority = "Normal"
requires_target = false
description = "Scout and plan ambush locations."
```

**Verification:**
```bash
cargo test 2>&1
# Expected: all tests pass
```

---

## Phase D: Polity Behaviors

### Task D1: Create polity behavior framework

**File:** `src/aggregate/behavior.rs` (NEW)

```rust
//! Polity behavior framework

use crate::aggregate::polity::Polity;
use crate::aggregate::world::AggregateWorld;
use crate::aggregate::events::EventType;
use crate::core::types::Species;

/// Trait for species-specific polity behavior
pub trait PolityBehavior {
    /// Generate events for this tick
    fn tick(&self, polity: &Polity, world: &AggregateWorld, year: u32) -> Vec<EventType>;

    /// React to an event
    fn on_event(&self, polity: &mut Polity, event: &EventType, world: &AggregateWorld);
}

/// Get the behavior handler for a species
pub fn get_behavior(species: Species) -> Box<dyn PolityBehavior> {
    match species {
        Species::Gnoll => Box::new(super::species::gnoll::GnollBehavior),
        Species::Vampire => Box::new(super::species::vampire::VampireBehavior),
        Species::Kobold => Box::new(super::species::kobold::KoboldBehavior),
        // Default no-op behavior for other species
        _ => Box::new(DefaultBehavior),
    }
}

struct DefaultBehavior;

impl PolityBehavior for DefaultBehavior {
    fn tick(&self, _polity: &Polity, _world: &AggregateWorld, _year: u32) -> Vec<EventType> {
        Vec::new()
    }

    fn on_event(&self, _polity: &mut Polity, _event: &EventType, _world: &AggregateWorld) {}
}
```

---

### Task D2: Implement Gnoll polity behavior

**File:** `src/aggregate/species/gnoll.rs` (REPLACE)

```rust
//! Gnoll-specific polity behavior - Raider archetype

use crate::aggregate::polity::Polity;
use crate::aggregate::world::AggregateWorld;
use crate::aggregate::events::EventType;
use crate::aggregate::behavior::PolityBehavior;

pub struct GnollBehavior;

impl PolityBehavior for GnollBehavior {
    fn tick(&self, polity: &Polity, world: &AggregateWorld, _year: u32) -> Vec<EventType> {
        let mut events = Vec::new();

        if let Some(state) = polity.gnoll_state() {
            // High pack_frenzy triggers raids
            if state.pack_frenzy > 0.7 {
                // Find weak neighbors to raid
                if let Some(target) = find_raid_target(polity, world) {
                    events.push(EventType::RaidLaunched {
                        attacker: polity.id,
                        target,
                    });
                }
            }

            // Demon taint grows over time
            if state.demon_taint > 0.5 {
                events.push(EventType::CorruptionSpreads {
                    polity: polity.id,
                    intensity: state.demon_taint,
                });
            }
        }

        events
    }

    fn on_event(&self, polity: &mut Polity, event: &EventType, _world: &AggregateWorld) {
        if let Some(state) = polity.gnoll_state_mut() {
            match event {
                EventType::BattleWon { .. } => {
                    state.pack_frenzy = (state.pack_frenzy + 0.2).min(1.0);
                }
                EventType::BattleLost { .. } => {
                    state.pack_frenzy = (state.pack_frenzy - 0.3).max(0.0);
                }
                _ => {}
            }
        }
    }
}

fn find_raid_target(polity: &Polity, world: &AggregateWorld) -> Option<crate::core::types::PolityId> {
    // Find neighboring polities with lower military strength
    world.get_neighbors(polity.id)
        .iter()
        .filter(|&neighbor_id| {
            if let Some(neighbor) = world.get_polity(*neighbor_id) {
                neighbor.military_strength < polity.military_strength * 0.8
            } else {
                false
            }
        })
        .next()
        .copied()
}
```

---

### Task D3: Implement Vampire polity behavior

**File:** `src/aggregate/species/vampire.rs` (REPLACE)

```rust
//! Vampire-specific polity behavior - Manipulator archetype

use crate::aggregate::polity::Polity;
use crate::aggregate::world::AggregateWorld;
use crate::aggregate::events::EventType;
use crate::aggregate::behavior::PolityBehavior;

pub struct VampireBehavior;

impl PolityBehavior for VampireBehavior {
    fn tick(&self, polity: &Polity, world: &AggregateWorld, _year: u32) -> Vec<EventType> {
        let mut events = Vec::new();

        if let Some(state) = polity.vampire_state() {
            // Expand thrall network through neighboring polities
            if state.thrall_network.len() < 3 {
                if let Some(target) = find_thrall_target(polity, world) {
                    events.push(EventType::InfiltrationAttempt {
                        infiltrator: polity.id,
                        target,
                    });
                }
            }

            // Blood debt collection
            if state.blood_debt_owed > 0 {
                events.push(EventType::TributedemTributeDemanded {
                    from: polity.id,
                    amount: state.blood_debt_owed,
                });
            }
        }

        events
    }

    fn on_event(&self, polity: &mut Polity, event: &EventType, _world: &AggregateWorld) {
        if let Some(state) = polity.vampire_state_mut() {
            match event {
                EventType::InfiltrationSuccess { target, .. } => {
                    state.thrall_network.push(target.0);
                }
                EventType::TributePaid { amount, .. } => {
                    state.blood_debt_owed = state.blood_debt_owed.saturating_sub(*amount);
                }
                _ => {}
            }
        }
    }
}

fn find_thrall_target(polity: &Polity, world: &AggregateWorld) -> Option<crate::core::types::PolityId> {
    // Find wealthy neighbors not already in thrall network
    let state = polity.vampire_state()?;

    world.get_neighbors(polity.id)
        .iter()
        .filter(|&neighbor_id| {
            !state.thrall_network.contains(&neighbor_id.0)
        })
        .max_by(|&a, &b| {
            let wealth_a = world.get_polity(*a).map(|p| p.economic_strength).unwrap_or(0.0);
            let wealth_b = world.get_polity(*b).map(|p| p.economic_strength).unwrap_or(0.0);
            wealth_a.partial_cmp(&wealth_b).unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
}
```

---

### Task D4: Implement Kobold polity behavior

**File:** `src/aggregate/species/kobold.rs` (REPLACE)

```rust
//! Kobold-specific polity behavior - Trapper archetype

use crate::aggregate::polity::Polity;
use crate::aggregate::world::AggregateWorld;
use crate::aggregate::events::EventType;
use crate::aggregate::behavior::PolityBehavior;

pub struct KoboldBehavior;

impl PolityBehavior for KoboldBehavior {
    fn tick(&self, polity: &Polity, _world: &AggregateWorld, _year: u32) -> Vec<EventType> {
        let mut events = Vec::new();

        if let Some(state) = polity.kobold_state() {
            // Build traps when tunnel network is large enough
            if state.tunnel_network > 5 && state.trap_density < 0.8 {
                events.push(EventType::TrapConstruction {
                    polity: polity.id,
                    trap_count: (state.tunnel_network / 2) as u32,
                });
            }

            // Grudge-based spite attacks
            if !state.grudge_targets.is_empty() && polity.military_strength > 50.0 {
                let target_id = state.grudge_targets[0];
                events.push(EventType::SpiteRaid {
                    attacker: polity.id,
                    target: crate::core::types::PolityId(target_id),
                });
            }

            // Dragon worship increases when dragon is nearby (placeholder)
            if state.dragon_worship > 0.8 {
                events.push(EventType::DragonTributeOffered {
                    polity: polity.id,
                });
            }
        }

        events
    }

    fn on_event(&self, polity: &mut Polity, event: &EventType, _world: &AggregateWorld) {
        if let Some(state) = polity.kobold_state_mut() {
            match event {
                EventType::TrapTriggered { casualties, .. } => {
                    state.trap_density = (state.trap_density - 0.1).max(0.0);
                    // Successful trap increases cunning reputation
                    if *casualties > 0 {
                        state.tunnel_network += 1;
                    }
                }
                EventType::WarrenAttacked { attacker, .. } => {
                    // Add to grudge list
                    if !state.grudge_targets.contains(&attacker.0) {
                        state.grudge_targets.push(attacker.0);
                    }
                }
                _ => {}
            }
        }
    }
}
```

---

## Verification & Testing

### Final Integration Test

**File:** `tests/species_behavior_integration.rs` (NEW)

```rust
//! Integration tests for the species behavior system

use arc_citadel::core::types::Species;
use arc_citadel::entity::species::gnoll::GnollValues;
use arc_citadel::entity::species::value_access::ValueAccessor;
use arc_citadel::rules::load_species_rules;
use arc_citadel::simulation::rule_eval::evaluate_action_rules;
use std::path::Path;

#[test]
fn test_gnoll_attacks_when_bloodlust_high() {
    let species_rules = load_species_rules(Path::new("species")).unwrap();

    let mut values = GnollValues::default();
    values.bloodlust = 0.8; // Above threshold

    let rules = species_rules.get_action_rules(Species::Gnoll);
    let task = evaluate_action_rules(&values, rules, arc_citadel::core::types::Tick(0), true);

    assert!(task.is_some());
    let task = task.unwrap();
    assert!(matches!(task.action, arc_citadel::actions::catalog::ActionId::Attack));
}

#[test]
fn test_gnoll_does_not_attack_when_bloodlust_low() {
    let species_rules = load_species_rules(Path::new("species")).unwrap();

    let mut values = GnollValues::default();
    values.bloodlust = 0.3; // Below threshold

    let rules = species_rules.get_action_rules(Species::Gnoll);
    let task = evaluate_action_rules(&values, rules, arc_citadel::core::types::Tick(0), true);

    // Should not trigger attack rule
    if let Some(task) = task {
        assert!(!matches!(task.action, arc_citadel::actions::catalog::ActionId::Attack));
    }
}

#[test]
fn test_value_dynamics_increases_bloodlust() {
    use arc_citadel::rules::value_dynamics::TickDelta;
    use arc_citadel::simulation::value_dynamics::apply_tick_dynamics;

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
```

**Verification:**
```bash
cargo test species_behavior_integration 2>&1
# Expected: all 3 tests pass
```

---

## Summary

| Phase | Tasks | New Files | Modified Files |
|-------|-------|-----------|----------------|
| A | A1-A8 | 4 | 6 |
| B | B1-B4 | 2 | 4 |
| C | C1-C3 | 0 | 3 |
| D | D1-D4 | 1 | 3 |
| **Total** | **19** | **7** | **16** |

**Estimated effort:** 4-5 hours for full implementation

**Success criteria:**
1. All tests pass: `cargo test`
2. Demo species (Gnoll, Vampire, Kobold) use runtime rules
3. Values change over time via dynamics
4. Polity behaviors generate events
