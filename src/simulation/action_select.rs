//! Action selection algorithm - the heart of autonomous behavior

use crate::actions::catalog::ActionId;
use crate::city::building::BuildingId;
use crate::core::types::{EntityId, Tick, Vec2};
use crate::entity::body::BodyState;
use crate::entity::needs::{NeedType, Needs};
use crate::entity::social::Disposition;
use crate::entity::species::abyssal_demons::AbyssalDemonsValues;
use crate::entity::species::centaur::CentaurValues;
use crate::entity::species::dryad::DryadValues;
use crate::entity::species::elemental::ElementalValues;
use crate::entity::species::fey::FeyValues;
use crate::entity::species::gnoll::GnollValues;
use crate::entity::species::goblin::GoblinValues;
use crate::entity::species::golem::GolemValues;
use crate::entity::species::harpy::HarpyValues;
use crate::entity::species::hobgoblin::HobgoblinValues;
use crate::entity::species::human::HumanValues;
use crate::entity::species::kobold::KoboldValues;
use crate::entity::species::lizardfolk::LizardfolkValues;
use crate::entity::species::lupine::LupineValues;
use crate::entity::species::merfolk::MerfolkValues;
use crate::entity::species::minotaur::MinotaurValues;
use crate::entity::species::naga::NagaValues;
use crate::entity::species::ogre::OgreValues;
use crate::entity::species::orc::OrcValues;
use crate::entity::species::revenant::RevenantValues;
use crate::entity::species::satyr::SatyrValues;
use crate::entity::species::stone_giants::StoneGiantsValues;
use crate::entity::species::troll::TrollValues;
use crate::entity::species::vampire::VampireValues;
// CODEGEN: species_values_imports
use crate::core::types::Species;
use crate::entity::species::value_access::ValueAccessor;
use crate::entity::tasks::{Task, TaskPriority, TaskSource};
use crate::entity::thoughts::ThoughtBuffer;
use crate::rules::SpeciesRules;
use crate::simulation::rule_eval::{evaluate_action_rules, select_idle_behavior};

/// Pick a talk target from perceived entities with disposition preference (E5)
///
/// Preference order: Friendly/Favorable > Neutral > Unknown > (avoid Hostile/Suspicious)
/// This enables E1 (positive relationship formation) by allowing first contact with Unknown,
/// and E5 (disposition-aware selection) by preferring friendly entities.
fn pick_talk_target(perceived_dispositions: &[(EntityId, Disposition)]) -> Option<EntityId> {
    // First try to find a Friendly/Favorable target
    for (id, disposition) in perceived_dispositions {
        if matches!(disposition, Disposition::Friendly | Disposition::Favorable) {
            return Some(*id);
        }
    }
    // Then try Neutral
    for (id, disposition) in perceived_dispositions {
        if matches!(disposition, Disposition::Neutral) {
            return Some(*id);
        }
    }
    // Then try Unknown (first contact - this enables relationship formation)
    for (id, disposition) in perceived_dispositions {
        if matches!(disposition, Disposition::Unknown) {
            return Some(*id);
        }
    }
    // Avoid Hostile/Suspicious - don't talk to enemies
    None
}

/// Generic action selection using runtime-loaded rules
///
/// This function evaluates species-specific rules loaded from TOML files,
/// allowing behavior customization without recompilation.
///
/// # Arguments
/// * `values` - Species-specific values implementing ValueAccessor
/// * `needs` - Universal needs (safety, hunger, rest, social, purpose)
/// * `species_rules` - Runtime-loaded rules from World
/// * `species` - The species type for rule lookup
/// * `has_current_task` - Whether entity already has an active task
/// * `threat_nearby` - Whether a threat is perceived nearby
/// * `food_available` - Whether food is accessible
/// * `entity_nearby` - Whether another entity is nearby (for target-requiring actions)
/// * `current_tick` - Current simulation tick
///
/// # Returns
/// `Some(Task)` if rules match, `None` if no applicable rule found
pub fn select_action_with_rules<V: ValueAccessor>(
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
    // Critical needs always take priority (uses universal needs system)
    if let Some(critical) = needs.has_critical() {
        return select_critical_response_generic(
            critical,
            threat_nearby,
            food_available,
            current_tick,
        );
    }

    // Don't interrupt existing tasks
    if has_current_task {
        return None;
    }

    // Get action rules for this species
    let action_rules = species_rules.get_action_rules(species);

    // Evaluate rules against current values
    if let Some(task) = evaluate_action_rules(values, action_rules, current_tick, entity_nearby) {
        return Some(task);
    }

    // Fall back to idle behavior from rules
    let idle_behaviors = species_rules.get_idle_behaviors(species);
    Some(select_idle_behavior(values, idle_behaviors, current_tick))
}

/// Handle critical needs for generic rule-based selection
fn select_critical_response_generic(
    need: NeedType,
    threat_nearby: bool,
    food_available: bool,
    current_tick: Tick,
) -> Option<Task> {
    match need {
        NeedType::Safety if threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Food if food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Food => Some(Task {
            action: ActionId::Gather,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::High,
            created_tick: current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Social => Some(Task {
            action: ActionId::TalkTo,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::High,
            created_tick: current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Purpose => Some(Task {
            action: ActionId::Gather,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Normal,
            created_tick: current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
    }
}

/// Context provided to the action selection algorithm
pub struct SelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a HumanValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    /// Nearest food zone: (zone_id, position, distance)
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    /// Nearby entities with their dispositions (from perception and social memory)
    /// Used for disposition-aware action selection (flee from hostile, talk to friendly)
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
    /// Building skill level (0.0 to 1.0) for construction work
    pub building_skill: f32,
    /// Nearest building under construction: (building_id, position, distance)
    pub nearest_building_site: Option<(BuildingId, Vec2, f32)>,
}

/// Main action selection function for humans
///
/// This function implements autonomous behavior by:
/// 1. First checking for critical needs (safety, food, rest at > 0.8)
/// 2. If entity already has a task, returning None (don't interrupt)
/// 3. Checking disposition-based responses (flee from hostile, approach friendly)
/// 4. Checking for value-driven impulses from strong thoughts
/// 5. Checking for purpose-driven building work (when purpose need is high and building site nearby)
/// 6. Addressing moderate needs (> 0.6)
/// 7. Falling back to idle behavior based on values
pub fn select_action_human(ctx: &SelectionContext) -> Option<Task> {
    // Detected threats trigger safety response (E7: flee) OR honor response (E4: attack)
    // High-honor entities (> 0.5) stand and fight; others flee
    if ctx.threat_nearby {
        if ctx.values.honor > 0.5 {
            // E4: High-honor humans defend/attack instead of fleeing
            // Find the hostile entity to target
            let hostile_target = ctx
                .perceived_dispositions
                .iter()
                .find(|(_, d)| *d == Disposition::Hostile)
                .map(|(id, _)| *id);

            return Some(Task {
                action: ActionId::Attack,
                target_position: None,
                target_entity: hostile_target,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Reaction,
            });
        }
        return select_critical_response(NeedType::Safety, ctx);
    }

    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Check for disposition-based responses
    // This uses social memory to react to nearby entities based on past experiences
    if let Some(task) = check_disposition_response(ctx) {
        return Some(task);
    }

    // Check for value-driven impulses from thoughts
    if let Some(task) = check_value_impulses(ctx) {
        return Some(task);
    }

    // Check for purpose-driven building work
    // When purpose need is high and there's a nearby construction site, seek work
    if let Some(task) = should_seek_building_work(ctx) {
        return Some(task);
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action(ctx))
}

/// Handle critical needs (> 0.8) with immediate responses
fn select_critical_response(need: NeedType, ctx: &SelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            // Move to food zone
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::Gather, // No food known, actively forage/search for food
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety, // Find safe place to rest first
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Check for disposition-based responses to nearby entities
///
/// This function uses social memory to react to nearby entities based on
/// past experiences. It enables emergent social behavior where:
/// - Hostile entities trigger flight response when safety need is elevated
/// - Friendly entities encourage social interaction when social need is elevated
///
/// Returns a task if disposition-based action is warranted, None otherwise.
fn check_disposition_response(ctx: &SelectionContext) -> Option<Task> {
    // Check for hostile entities nearby when safety need is elevated
    // Threshold: safety need > 0.5 triggers concern about hostile entities
    let has_hostile = ctx
        .perceived_dispositions
        .iter()
        .any(|(_, d)| matches!(d, Disposition::Hostile | Disposition::Suspicious));

    if has_hostile && ctx.needs.safety > 0.5 {
        // Hostile entity nearby and we feel unsafe - flee!
        return Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::High,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        });
    }

    // Check for approachable entities when social need is elevated
    // Threshold is modulated by love/loyalty values - social entities seek interaction more readily
    // Base threshold: 0.55, but high love/loyalty lowers it significantly
    // Very social entities (avg love+loyalty > 0.7) will chat even at low social need
    let social_modifier = (ctx.values.love + ctx.values.loyalty) / 2.0;
    let social_threshold = 0.55 - (social_modifier * 0.5); // 0.55 down to 0.05 for max social values

    // Only Friendly/Favorable - strangers (Unknown) require moderate need handler
    let friendly_target = ctx
        .perceived_dispositions
        .iter()
        .find(|(_, d)| matches!(d, Disposition::Friendly | Disposition::Favorable))
        .map(|(id, _)| *id);

    if let Some(target_id) = friendly_target {
        if ctx.needs.social > social_threshold {
            // Friendly entity nearby and we want company - talk to them!
            // Priority scales with how much we actually need social interaction
            // Value-driven (low need, high values) = Low priority (casual)
            // Need-driven (high need) = Normal priority (seeking connection)
            let priority = if ctx.needs.social > 0.5 {
                TaskPriority::Normal
            } else {
                TaskPriority::Low
            };
            return Some(Task {
                action: ActionId::TalkTo,
                target_position: None,
                target_entity: Some(target_id),
                target_building: None,
                priority,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            });
        }
    }

    None
}

/// Check if strong thoughts combined with values should trigger action
///
/// This is where emergence happens - values filter which thoughts
/// lead to action. A high-justice entity reacts differently to
/// witnessing injustice than a high-comfort entity.
fn check_value_impulses(ctx: &SelectionContext) -> Option<Task> {
    if let Some(thought) = ctx.thoughts.strongest() {
        // Only react to strong thoughts
        if thought.intensity > 0.7 {
            // Justice-oriented response to witnessing injustice
            if ctx.values.justice > 0.7 && thought.concept_category == "injustice" {
                return Some(Task::new(
                    ActionId::Help,
                    TaskPriority::High,
                    ctx.current_tick,
                ));
            }

            // Loyalty-driven response to ally in need
            if ctx.values.loyalty > 0.7 && thought.concept_category == "ally_distress" {
                return Some(Task::new(
                    ActionId::Help,
                    TaskPriority::High,
                    ctx.current_tick,
                ));
            }

            // Honor-driven response to challenge
            if ctx.values.honor > 0.7 && thought.concept_category == "challenge" {
                return Some(Task::new(
                    ActionId::Defend,
                    TaskPriority::High,
                    ctx.current_tick,
                ));
            }

            // Safety-oriented response to perceived danger
            if ctx.values.safety > 0.7 && thought.concept_category == "danger" {
                return Some(Task::new(
                    ActionId::Flee,
                    TaskPriority::High,
                    ctx.current_tick,
                ));
            }

            // Piety-driven response to sacred events
            if ctx.values.piety > 0.7 && thought.concept_category == "sacred" {
                return Some(Task::new(
                    ActionId::IdleObserve,
                    TaskPriority::High,
                    ctx.current_tick,
                ));
            }
        }
    }

    None
}

/// Address moderate needs (between 0.5 and 0.8)
fn address_moderate_need(ctx: &SelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    // Address needs proactively at 0.5 (not waiting until 0.6)
    if level < 0.5 {
        return None;
    }

    match need_type {
        NeedType::Food if ctx.food_available => {
            Some(Task::new(ActionId::Eat, TaskPriority::Normal, ctx.current_tick))
        }
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            // Hungry but not at food - seek food zone
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Normal,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Rest if ctx.safe_location => {
            Some(Task::new(ActionId::Rest, TaskPriority::Normal, ctx.current_tick))
        }
        NeedType::Social if ctx.entity_nearby => {
            // E5: Pick target with disposition preference: Friendly > Unknown > avoid Hostile
            let talk_target = pick_talk_target(&ctx.perceived_dispositions);
            talk_target.map(|target_id| Task {
                action: ActionId::TalkTo,
                target_position: None,
                target_entity: Some(target_id),
                target_building: None,
                priority: TaskPriority::Normal,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Purpose => {
            Some(Task::new(ActionId::Gather, TaskPriority::Normal, ctx.current_tick))
        }
        NeedType::Safety if !ctx.safe_location => {
            Some(Task::new(ActionId::SeekSafety, TaskPriority::Normal, ctx.current_tick))
        }
        _ => None,
    }
}

/// Check if entity should seek building work based on purpose need
///
/// When an entity has high purpose need (> 0.6) and there's a nearby building site,
/// they may choose to work on construction. Higher building skill increases likelihood.
///
/// Returns Some((ActionId::Build, building_id, position)) if work should be sought.
fn should_seek_building_work(ctx: &SelectionContext) -> Option<Task> {
    // Only seek work if purpose need is high (> 0.6)
    if ctx.needs.purpose < 0.6 {
        return None;
    }

    // Need a building site within range
    let (building_id, pos, _distance) = ctx.nearest_building_site?;

    // Higher skill = more likely to seek building work
    // Skill weight: 0.5 base + 0.5 from skill (so 0.5 to 1.0 range)
    let skill_weight = 0.5 + ctx.building_skill * 0.5;
    // Purpose weight: maps 0.6-1.0 to 0.0-1.0 (so 0.0 to 1.0 range)
    let purpose_weight = (ctx.needs.purpose - 0.6) * 2.5;

    // Combined weight threshold: if skill_weight * purpose_weight > 0.3, seek work
    // Examples:
    //   skill=0.0, purpose=1.0 => 0.5 * 1.0 = 0.5 > 0.3 => seek work
    //   skill=0.8, purpose=0.7 => 0.9 * 0.25 = 0.225 < 0.3 => no work (purpose too low)
    //   skill=0.8, purpose=0.8 => 0.9 * 0.5 = 0.45 > 0.3 => seek work
    if skill_weight * purpose_weight > 0.3 {
        Some(
            Task::new(ActionId::Build, TaskPriority::Normal, ctx.current_tick)
                .with_building(building_id)
                .with_position(pos),
        )
    } else {
        None
    }
}

/// Select an idle action based on the entity's values
///
/// Different values lead to different idle behaviors:
/// - High curiosity (> 0.6): observe surroundings (E5 behavior)
/// - High ambition/purpose: gather resources or work
/// - High comfort: stay put and observe
/// - High social + entity nearby + social need > 0.4: talk
/// - Default: wander
fn select_idle_action(ctx: &SelectionContext) -> Task {
    // E5: High curiosity entities (> 0.6) should strongly prefer IdleObserve
    // This creates clear correlation between curiosity and exploration
    if ctx.values.curiosity > 0.6 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    }

    if ctx.values.ambition > 0.6 {
        // High ambition leads to productive idle behavior
        return Task::new(ActionId::Gather, TaskPriority::Low, ctx.current_tick);
    }

    if ctx.values.comfort > 0.7 {
        // High comfort leads to staying put
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    }

    // Use love + loyalty as social tendency proxy
    let social_tendency = (ctx.values.love + ctx.values.loyalty) / 2.0;

    // Only talk if social tendency is high, entity nearby, AND social need elevated
    if social_tendency > 0.6 && ctx.entity_nearby && ctx.needs.social > 0.4 {
        // E5: Pick target with disposition preference: Friendly > Unknown > avoid Hostile
        if let Some(target_id) = pick_talk_target(&ctx.perceived_dispositions) {
            return Task {
                action: ActionId::TalkTo,
                target_position: None,
                target_entity: Some(target_id),
                target_building: None,
                priority: TaskPriority::Low,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            };
        }
    }

    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}

// ============================================================================
// ORC ACTION SELECTION
// ============================================================================

/// Context provided to orc action selection
pub struct OrcSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a OrcValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for orcs
///
/// Orcs have different behavioral priorities than humans:
/// 1. Critical needs still take priority (survival)
/// 2. High rage triggers aggressive action
/// 3. Blood debt creates revenge-seeking behavior
/// 4. Dominance drives territorial/assertive actions
/// 5. Clan loyalty triggers defense of nearby clan members
/// 6. Idle behavior is aggressive (patrol, challenge)
pub fn select_action_orc(ctx: &OrcSelectionContext) -> Option<Task> {
    // Critical needs always take priority (even orcs need to survive)
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_orc(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // High rage triggers aggressive action
    if let Some(task) = check_rage_response(ctx) {
        return Some(task);
    }

    // Blood debt creates revenge-seeking behavior
    if let Some(task) = check_blood_debt_response(ctx) {
        return Some(task);
    }

    // Dominance drives territorial behavior
    if let Some(task) = check_dominance_response(ctx) {
        return Some(task);
    }

    // Clan loyalty triggers defense
    if let Some(task) = check_clan_loyalty_response(ctx) {
        return Some(task);
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_orc(ctx) {
        return Some(task);
    }

    // Fall back to orc idle behavior
    Some(select_idle_action_orc(ctx))
}

/// Handle critical needs for orcs
fn select_critical_response_orc(need: NeedType, ctx: &OrcSelectionContext) -> Option<Task> {
    match need {
        // Orcs with high strength may fight instead of flee
        NeedType::Safety if ctx.threat_nearby && ctx.values.strength > 0.7 => Some(Task {
            action: ActionId::Defend,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// High rage triggers aggressive action toward nearby entities
fn check_rage_response(ctx: &OrcSelectionContext) -> Option<Task> {
    if ctx.values.rage > 0.7 && ctx.entity_nearby {
        // Find any target (orcs in rage attack anyone nearby)
        let target = ctx.perceived_dispositions.first().map(|(id, _)| *id);

        return Some(Task {
            action: ActionId::Defend, // Attack/fight action
            target_position: None,
            target_entity: target,
            target_building: None,
            priority: TaskPriority::High,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        });
    }
    None
}

/// Blood debt creates revenge-seeking behavior against hostile entities
fn check_blood_debt_response(ctx: &OrcSelectionContext) -> Option<Task> {
    if ctx.values.blood_debt > 0.5 {
        // Look for hostile entities to attack (settling blood debts)
        let hostile_target = ctx
            .perceived_dispositions
            .iter()
            .find(|(_, d)| matches!(d, Disposition::Hostile | Disposition::Suspicious))
            .map(|(id, _)| *id);

        if let Some(target_id) = hostile_target {
            return Some(Task {
                action: ActionId::Defend, // Attack to settle blood debt
                target_position: None,
                target_entity: Some(target_id),
                target_building: None,
                priority: TaskPriority::High,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            });
        }
    }
    None
}

/// Dominance drives territorial/assertive actions
fn check_dominance_response(ctx: &OrcSelectionContext) -> Option<Task> {
    if ctx.values.dominance > 0.7 && ctx.values.territory > 0.5 {
        // High dominance + territory = patrol and defend territory
        return Some(Task {
            action: ActionId::IdleWander, // Patrol behavior
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Normal,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        });
    }
    None
}

/// Clan loyalty triggers defense of nearby friendly entities
fn check_clan_loyalty_response(ctx: &OrcSelectionContext) -> Option<Task> {
    if ctx.values.clan_loyalty > 0.7 {
        // Check if friendly entity is nearby with hostile also present
        let has_friendly = ctx
            .perceived_dispositions
            .iter()
            .any(|(_, d)| matches!(d, Disposition::Friendly | Disposition::Favorable));
        let hostile_target = ctx
            .perceived_dispositions
            .iter()
            .find(|(_, d)| matches!(d, Disposition::Hostile))
            .map(|(id, _)| *id);

        if has_friendly && hostile_target.is_some() {
            // Clan member nearby with hostile threat - defend!
            return Some(Task {
                action: ActionId::Defend,
                target_position: None,
                target_entity: hostile_target,
                target_building: None,
                priority: TaskPriority::High,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Reaction,
            });
        }
    }
    None
}

/// Address moderate needs for orcs
fn address_moderate_need_orc(ctx: &OrcSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo, // Even orcs socialize
        NeedType::Purpose => ActionId::Gather,                     // Productive work
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on orc values
///
/// Orc idle behaviors reflect their aggressive nature:
/// - High combat prowess: patrol/wander aggressively
/// - High dominance: observe (watch for challenges)
/// - High strength: gather (work on strength-building tasks)
/// - Default: wander looking for trouble
fn select_idle_action_orc(ctx: &OrcSelectionContext) -> Task {
    let action = if ctx.values.combat_prowess > 0.7 {
        ActionId::IdleWander // Patrol for combat opportunities
    } else if ctx.values.dominance > 0.7 {
        ActionId::IdleObserve // Watch for challenges to dominance
    } else if ctx.values.strength > 0.7 {
        ActionId::Gather // Work on strength-building tasks
    } else {
        ActionId::IdleWander // Default: wander
    };

    Task::new(action, TaskPriority::Low, ctx.current_tick)
}

// CODEGEN: species_selection_context

/// Context provided to kobold action selection
pub struct KoboldSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a KoboldValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for kobold
pub fn select_action_kobold(ctx: &KoboldSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_kobold(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Scout and plan ambush locations.
    if ctx.values.cunning > 0.7 {
        return Some(Task::new(
            ActionId::IdleObserve,
            TaskPriority::Normal,
            ctx.current_tick,
        ));
    }

    // Run when threatened by stronger foes.
    if ctx.values.cowardice > 0.75 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Flee,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Collect resources for the warren.
    if ctx.values.industriousness > 0.6 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Gather,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_kobold(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_kobold(ctx))
}

/// Handle critical needs for kobold
fn select_critical_response_kobold(need: NeedType, ctx: &KoboldSelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for kobold
fn address_moderate_need_kobold(ctx: &KoboldSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on kobold values
fn select_idle_action_kobold(ctx: &KoboldSelectionContext) -> Task {
    if ctx.values.cunning > 0.5 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.pack_loyalty > 0.4 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to gnoll action selection
pub struct GnollSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a GnollValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for gnoll
pub fn select_action_gnoll(ctx: &GnollSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_gnoll(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Enter killing frenzy when blood is scented.
    if ctx.values.bloodlust > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Hunt for food.
    if ctx.values.hunger > 0.65 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Gather,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Coordinate with packmates.
    if ctx.values.pack_instinct > 0.6 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Follow,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_gnoll(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_gnoll(ctx))
}

/// Handle critical needs for gnoll
fn select_critical_response_gnoll(need: NeedType, ctx: &GnollSelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for gnoll
fn address_moderate_need_gnoll(ctx: &GnollSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on gnoll values
fn select_idle_action_gnoll(ctx: &GnollSelectionContext) -> Task {
    if ctx.values.hunger > 0.4 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.dominance > 0.5 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to lizardfolk action selection
pub struct LizardfolkSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a LizardfolkValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for lizardfolk
pub fn select_action_lizardfolk(ctx: &LizardfolkSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_lizardfolk(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Retreat when survival is threatened.
    if ctx.values.survival > 0.8 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Flee,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Hunt efficiently for sustenance.
    if ctx.values.hunger > 0.6 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Gather,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Drive intruders from territory.
    if ctx.values.territoriality > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_lizardfolk(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_lizardfolk(ctx))
}

/// Handle critical needs for lizardfolk
fn select_critical_response_lizardfolk(
    need: NeedType,
    ctx: &LizardfolkSelectionContext,
) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for lizardfolk
fn address_moderate_need_lizardfolk(ctx: &LizardfolkSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on lizardfolk values
fn select_idle_action_lizardfolk(ctx: &LizardfolkSelectionContext) -> Task {
    if ctx.values.patience > 0.5 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.pragmatism > 0.4 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to hobgoblin action selection
pub struct HobgoblinSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a HobgoblinValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for hobgoblin
pub fn select_action_hobgoblin(ctx: &HobgoblinSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_hobgoblin(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Execute orders with precision.
    if ctx.values.discipline > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Follow,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Strike at enemies to gain glory.
    if ctx.values.ambition > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Secure resources for the legion.
    if ctx.values.pragmatism > 0.6 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Gather,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_hobgoblin(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_hobgoblin(ctx))
}

/// Handle critical needs for hobgoblin
fn select_critical_response_hobgoblin(
    need: NeedType,
    ctx: &HobgoblinSelectionContext,
) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for hobgoblin
fn address_moderate_need_hobgoblin(ctx: &HobgoblinSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on hobgoblin values
fn select_idle_action_hobgoblin(ctx: &HobgoblinSelectionContext) -> Task {
    if ctx.values.discipline > 0.5 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.honor > 0.4 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to ogre action selection
pub struct OgreSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a OgreValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for ogre
pub fn select_action_ogre(ctx: &OgreSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_ogre(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Hunt for food, anything edible.
    if ctx.values.hunger > 0.6 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Gather,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Smash things that annoy.
    if ctx.values.brutality > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Nap after eating.
    if ctx.values.laziness > 0.8 {
        return Some(Task::new(
            ActionId::Rest,
            TaskPriority::Normal,
            ctx.current_tick,
        ));
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_ogre(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_ogre(ctx))
}

/// Handle critical needs for ogre
fn select_critical_response_ogre(need: NeedType, ctx: &OgreSelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for ogre
fn address_moderate_need_ogre(ctx: &OgreSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on ogre values
fn select_idle_action_ogre(ctx: &OgreSelectionContext) -> Task {
    if ctx.values.laziness > 0.5 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.dullness > 0.6 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to harpy action selection
pub struct HarpySelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a HarpyValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for harpy
pub fn select_action_harpy(ctx: &HarpySelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_harpy(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Dive-bomb intruders near the nest.
    if ctx.values.territoriality > 0.8 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Swoop down to snatch prey.
    if ctx.values.hunger > 0.6 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Gather,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Collect shiny objects for the nest.
    if ctx.values.vanity > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Gather,
                TaskPriority::Low,
                ctx.current_tick,
            ));
        }
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_harpy(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_harpy(ctx))
}

/// Handle critical needs for harpy
fn select_critical_response_harpy(need: NeedType, ctx: &HarpySelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for harpy
fn address_moderate_need_harpy(ctx: &HarpySelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on harpy values
fn select_idle_action_harpy(ctx: &HarpySelectionContext) -> Task {
    if ctx.values.territoriality > 0.5 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.sisterhood > 0.4 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to centaur action selection
pub struct CentaurSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a CentaurValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for centaur
pub fn select_action_centaur(ctx: &CentaurSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_centaur(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Challenge those who insult the herd.
    if ctx.values.honor > 0.8 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Stand with allies in battle.
    if ctx.values.loyalty > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Follow,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Travel to new grazing lands.
    if ctx.values.wanderlust > 0.6 {
        return Some(Task::new(
            ActionId::IdleWander,
            TaskPriority::Normal,
            ctx.current_tick,
        ));
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_centaur(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_centaur(ctx))
}

/// Handle critical needs for centaur
fn select_critical_response_centaur(need: NeedType, ctx: &CentaurSelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for centaur
fn address_moderate_need_centaur(ctx: &CentaurSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on centaur values
fn select_idle_action_centaur(ctx: &CentaurSelectionContext) -> Task {
    if ctx.values.wanderlust > 0.5 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.pride > 0.4 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to minotaur action selection
pub struct MinotaurSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a MinotaurValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for minotaur
pub fn select_action_minotaur(ctx: &MinotaurSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_minotaur(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Charge and gore intruders.
    if ctx.values.rage > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Hunt down trespassers in the maze.
    if ctx.values.territoriality > 0.75 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Stalk prey through tunnels.
    if ctx.values.hunger > 0.6 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Gather,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_minotaur(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_minotaur(ctx))
}

/// Handle critical needs for minotaur
fn select_critical_response_minotaur(
    need: NeedType,
    ctx: &MinotaurSelectionContext,
) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for minotaur
fn address_moderate_need_minotaur(ctx: &MinotaurSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on minotaur values
fn select_idle_action_minotaur(ctx: &MinotaurSelectionContext) -> Task {
    if ctx.values.isolation > 0.5 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.cunning > 0.4 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to satyr action selection
pub struct SatyrSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a SatyrValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for satyr
pub fn select_action_satyr(ctx: &SatyrSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_satyr(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Run from danger while laughing.
    if ctx.values.cowardice > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Flee,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Indulge in wine and song.
    if ctx.values.hedonism > 0.6 {
        return Some(Task::new(
            ActionId::Rest,
            TaskPriority::Normal,
            ctx.current_tick,
        ));
    }

    // Seek targets for pranks.
    if ctx.values.mischief > 0.7 {
        return Some(Task::new(
            ActionId::IdleWander,
            TaskPriority::Normal,
            ctx.current_tick,
        ));
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_satyr(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_satyr(ctx))
}

/// Handle critical needs for satyr
fn select_critical_response_satyr(need: NeedType, ctx: &SatyrSelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for satyr
fn address_moderate_need_satyr(ctx: &SatyrSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on satyr values
fn select_idle_action_satyr(ctx: &SatyrSelectionContext) -> Task {
    if ctx.values.hedonism > 0.4 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.nature_bond > 0.5 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to dryad action selection
pub struct DryadSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a DryadValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for dryad
pub fn select_action_dryad(ctx: &DryadSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_dryad(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Strike down those who harm the forest.
    if ctx.values.protectiveness > 0.8 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Punish despoilers with nature's fury.
    if ctx.values.wrath > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Commune with the heart-tree.
    if ctx.values.nature_bond > 0.6 {
        return Some(Task::new(
            ActionId::Rest,
            TaskPriority::Normal,
            ctx.current_tick,
        ));
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_dryad(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_dryad(ctx))
}

/// Handle critical needs for dryad
fn select_critical_response_dryad(need: NeedType, ctx: &DryadSelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for dryad
fn address_moderate_need_dryad(ctx: &DryadSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on dryad values
fn select_idle_action_dryad(ctx: &DryadSelectionContext) -> Task {
    if ctx.values.patience > 0.4 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.nature_bond > 0.5 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to goblin action selection
pub struct GoblinSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a GoblinValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for goblin
pub fn select_action_goblin(ctx: &GoblinSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_goblin(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Raid a nearby settlement or entity for loot when greed is high.
    if ctx.values.greed > 0.75 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Gather,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Run from a powerful enemy when fear overcomes greed.
    if ctx.values.cowardice > 0.8 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Flee,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Join allies in a frenzied swarm attack when many are nearby.
    if ctx.values.pack_rage > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Scavenge for food when hungry.
    if ctx.values.hunger > 0.65 {
        return Some(Task::new(
            ActionId::Eat,
            TaskPriority::Normal,
            ctx.current_tick,
        ));
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_goblin(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_goblin(ctx))
}

/// Handle critical needs for goblin
fn select_critical_response_goblin(need: NeedType, ctx: &GoblinSelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for goblin
fn address_moderate_need_goblin(ctx: &GoblinSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on goblin values
fn select_idle_action_goblin(ctx: &GoblinSelectionContext) -> Task {
    if ctx.values.sneakiness > 0.6 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.cowardice > 0.4 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to troll action selection
pub struct TrollSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a TrollValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for troll
pub fn select_action_troll(ctx: &TrollSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_troll(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Attack any entity that intrudes deep into the troll's territory.
    if ctx.values.territoriality > 0.8 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Hunt prey to sate growing hunger.
    if ctx.values.hunger > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Enter a furious, reckless charge when severely wounded.
    if ctx.values.rage > 0.85 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Charge,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Stand and fight rather than flee, trusting in regeneration.
    if ctx.values.recklessness > 0.6 {
        return Some(Task::new(
            ActionId::HoldPosition,
            TaskPriority::Low,
            ctx.current_tick,
        ));
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_troll(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_troll(ctx))
}

/// Handle critical needs for troll
fn select_critical_response_troll(need: NeedType, ctx: &TrollSelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for troll
fn address_moderate_need_troll(ctx: &TrollSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on troll values
fn select_idle_action_troll(ctx: &TrollSelectionContext) -> Task {
    if ctx.values.patience > 0.5 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.hunger > 0.4 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to abyssal_demons action selection
pub struct AbyssalDemonsSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a AbyssalDemonsValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for abyssal_demons
pub fn select_action_abyssal_demons(ctx: &AbyssalDemonsSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_abyssal_demons(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Overwhelming hunger drives the demon to assault a target to claim its soul.
    if ctx.values.soul_hunger > 0.75 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // The demon manifests a structure or altar to spread its corrupting influence in the area.
    if ctx.values.corruptive_urge > 0.8 {
        return Some(Task::new(
            ActionId::Build,
            TaskPriority::Normal,
            ctx.current_tick,
        ));
    }

    // The demon attempts to parley, offering a deceptive bargain or contract.
    if ctx.values.malicious_cunning > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::TalkTo,
                TaskPriority::Low,
                ctx.current_tick,
            ));
        }
    }

    // A burst of chaotic fury causes the demon to recklessly assault the nearest foe.
    if ctx.values.abyssal_rage > 0.85 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Charge,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_abyssal_demons(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_abyssal_demons(ctx))
}

/// Handle critical needs for abyssal_demons
fn select_critical_response_abyssal_demons(
    need: NeedType,
    ctx: &AbyssalDemonsSelectionContext,
) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for abyssal_demons
fn address_moderate_need_abyssal_demons(ctx: &AbyssalDemonsSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on abyssal_demons values
fn select_idle_action_abyssal_demons(ctx: &AbyssalDemonsSelectionContext) -> Task {
    if ctx.values.corruptive_urge > 0.6 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.malicious_cunning > 0.5 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to elemental action selection
pub struct ElementalSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a ElementalValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for elemental
pub fn select_action_elemental(ctx: &ElementalSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_elemental(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Lash out with raw elemental force at those who have despoiled the land.
    if ctx.values.elemental_rage > 0.8 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Aggressively drive out intruders from claimed terrain.
    if ctx.values.territorial_instinct > 0.75 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Charge,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // In a moment of clarity, attempt to communicate with another entity.
    if ctx.values.flickering_will > 0.6 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::TalkTo,
                TaskPriority::Low,
                ctx.current_tick,
            ));
        }
    }

    // Seek out terrain of high elemental fitness to manifest more fully.
    if ctx.values.primal_urge > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::MoveTo,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_elemental(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_elemental(ctx))
}

/// Handle critical needs for elemental
fn select_critical_response_elemental(
    need: NeedType,
    ctx: &ElementalSelectionContext,
) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for elemental
fn address_moderate_need_elemental(ctx: &ElementalSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on elemental values
fn select_idle_action_elemental(ctx: &ElementalSelectionContext) -> Task {
    if ctx.values.primal_urge > 0.4 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.flickering_will > 0.5 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to fey action selection
pub struct FeySelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a FeyValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for fey
pub fn select_action_fey(ctx: &FeySelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_fey(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Seeks to engage another entity in conversation, aiming to propose a binding bargain or oath.
    if ctx.values.bargain_hunger > 0.75 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::TalkTo,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Lashes out with cruel magic when malice overcomes whimsy.
    if ctx.values.cruelty > 0.8 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Retreats from entities perceived to wield or be made of cold iron.
    if ctx.values.fear_of_iron > 0.6 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Flee,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Defends a claimed site, such as a mushroom ring or ancient tree, from intruders.
    if ctx.values.territoriality > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Defend,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_fey(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_fey(ctx))
}

/// Handle critical needs for fey
fn select_critical_response_fey(need: NeedType, ctx: &FeySelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for fey
fn address_moderate_need_fey(ctx: &FeySelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on fey values
fn select_idle_action_fey(ctx: &FeySelectionContext) -> Task {
    if ctx.values.whimsy > 0.5 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.whimsy > 0.4 {
        if ctx.entity_nearby {
            return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
        }
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to stone_giants action selection
pub struct StoneGiantsSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a StoneGiantsValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for stone_giants
pub fn select_action_stone_giants(ctx: &StoneGiantsSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_stone_giants(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Fly into a devastating rage and attack the source of provocation.
    if ctx.values.rage > 0.8 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Demand tribute or trade from a weaker entity, leveraging size for intimidation.
    if ctx.values.greed > 0.75 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Trade,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Charge at intruders in their territory to drive them off.
    if ctx.values.territoriality > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Charge,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Overcome pride to initiate cautious communication, often gruff and demanding.
    if ctx.values.loneliness > 0.65 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::TalkTo,
                TaskPriority::Low,
                ctx.current_tick,
            ));
        }
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_stone_giants(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_stone_giants(ctx))
}

/// Handle critical needs for stone_giants
fn select_critical_response_stone_giants(
    need: NeedType,
    ctx: &StoneGiantsSelectionContext,
) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for stone_giants
fn address_moderate_need_stone_giants(ctx: &StoneGiantsSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on stone_giants values
fn select_idle_action_stone_giants(ctx: &StoneGiantsSelectionContext) -> Task {
    if ctx.values.pride > 0.6 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.territoriality > 0.5 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to golem action selection
pub struct GolemSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a GolemValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for golem
pub fn select_action_golem(ctx: &GolemSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_golem(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Stands vigilant at a post, obeying old commands.
    if ctx.values.obedience > 0.75 {
        return Some(Task::new(
            ActionId::HoldPosition,
            TaskPriority::High,
            ctx.current_tick,
        ));
    }

    // Violently expels intruders from their claimed terrain.
    if ctx.values.territoriality > 0.8 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Withdraws to a secure location to rest and avoid further damage.
    if ctx.values.weariness > 0.7 {
        return Some(Task::new(
            ActionId::SeekSafety,
            TaskPriority::Normal,
            ctx.current_tick,
        ));
    }

    // Attempts cautious communication, a sign of emerging sentience.
    if ctx.values.curiosity > 0.6 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::TalkTo,
                TaskPriority::Low,
                ctx.current_tick,
            ));
        }
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_golem(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_golem(ctx))
}

/// Handle critical needs for golem
fn select_critical_response_golem(need: NeedType, ctx: &GolemSelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for golem
fn address_moderate_need_golem(ctx: &GolemSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on golem values
fn select_idle_action_golem(ctx: &GolemSelectionContext) -> Task {
    if ctx.values.curiosity > 0.4 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.weariness > 0.5 {
        return Task::new(ActionId::Repair, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to merfolk action selection
pub struct MerfolkSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a MerfolkValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for merfolk
pub fn select_action_merfolk(ctx: &MerfolkSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_merfolk(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Attack outsiders who encroach on sacred waters.
    if ctx.values.xenophobia > 0.8 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Initiate trade if the potential profit is high enough.
    if ctx.values.greed > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Trade,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Defend the polity's territory against any aggressor.
    if ctx.values.pride > 0.75 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Defend,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_merfolk(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_merfolk(ctx))
}

/// Handle critical needs for merfolk
fn select_critical_response_merfolk(need: NeedType, ctx: &MerfolkSelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for merfolk
fn address_moderate_need_merfolk(ctx: &MerfolkSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on merfolk values
fn select_idle_action_merfolk(ctx: &MerfolkSelectionContext) -> Task {
    if ctx.values.curiosity > 0.5 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.pride > 0.3 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to naga action selection
pub struct NagaSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a NagaValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for naga
pub fn select_action_naga(ctx: &NagaSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_naga(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Attack any non-Naga entity that enters a sacred site.
    if ctx.values.territoriality > 0.75 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Launch a venomous strike against a target that has provoked them.
    if ctx.values.venomous_rage > 0.85 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Charge,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Stand guard at a sacred site or temple entrance.
    if ctx.values.duty > 0.6 {
        return Some(Task::new(
            ActionId::HoldPosition,
            TaskPriority::Normal,
            ctx.current_tick,
        ));
    }

    // Attempt to parley with an intelligent trespasser, seeking knowledge or offering a warning.
    if ctx.values.curiosity > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::TalkTo,
                TaskPriority::Low,
                ctx.current_tick,
            ));
        }
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_naga(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_naga(ctx))
}

/// Handle critical needs for naga
fn select_critical_response_naga(need: NeedType, ctx: &NagaSelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for naga
fn address_moderate_need_naga(ctx: &NagaSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on naga values
fn select_idle_action_naga(ctx: &NagaSelectionContext) -> Task {
    if ctx.values.curiosity > 0.5 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.duty > 0.4 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to revenant action selection
pub struct RevenantSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a RevenantValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for revenant
pub fn select_action_revenant(ctx: &RevenantSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_revenant(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Overwhelming hunger drives the Revenant to assault the living.
    if ctx.values.hunger_for_life > 0.8 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Compelled to follow a master or a stronger undead entity.
    if ctx.values.obedience > 0.75 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Follow,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Seeks to expand the borders of its blighted domain.
    if ctx.values.territorial_rot > 0.7 {
        return Some(Task::new(
            ActionId::MoveTo,
            TaskPriority::Normal,
            ctx.current_tick,
        ));
    }

    // A burst of fury from a past life fuels a reckless assault.
    if ctx.values.lingering_rage > 0.6 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Charge,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_revenant(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_revenant(ctx))
}

/// Handle critical needs for revenant
fn select_critical_response_revenant(
    need: NeedType,
    ctx: &RevenantSelectionContext,
) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for revenant
fn address_moderate_need_revenant(ctx: &RevenantSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on revenant values
fn select_idle_action_revenant(ctx: &RevenantSelectionContext) -> Task {
    if ctx.values.lingering_rage > 0.3 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.obedience > 0.4 {
        return Task::new(ActionId::HoldPosition, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to vampire action selection
pub struct VampireSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a VampireValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for vampire
pub fn select_action_vampire(ctx: &VampireSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_vampire(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // Overwhelming hunger drives the vampire to feed on the nearest living creature.
    if ctx.values.bloodthirst > 0.8 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // The vampire attempts to charm and enthrall a target, adding them to the thrall network.
    if ctx.values.dominance > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::TalkTo,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // When exposed or threatened with discovery, the vampire retreats to a safe, dark location.
    if ctx.values.secrecy > 0.8 {
        return Some(Task::new(
            ActionId::Flee,
            TaskPriority::High,
            ctx.current_tick,
        ));
    }

    // The vampire deigns to negotiate, offering ancient knowledge or artifacts for blood or service.
    if ctx.values.arrogance > 0.6 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Trade,
                TaskPriority::Low,
                ctx.current_tick,
            ));
        }
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_vampire(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_vampire(ctx))
}

/// Handle critical needs for vampire
fn select_critical_response_vampire(need: NeedType, ctx: &VampireSelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for vampire
fn address_moderate_need_vampire(ctx: &VampireSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on vampire values
fn select_idle_action_vampire(ctx: &VampireSelectionContext) -> Task {
    if ctx.values.ennui > 0.5 {
        return Task::new(ActionId::IdleObserve, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.dominance > 0.4 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
/// Context provided to lupine action selection
pub struct LupineSelectionContext<'a> {
    pub body: &'a BodyState,
    pub needs: &'a Needs,
    pub thoughts: &'a ThoughtBuffer,
    pub values: &'a LupineValues,
    pub has_current_task: bool,
    pub threat_nearby: bool,
    pub food_available: bool,
    pub safe_location: bool,
    pub entity_nearby: bool,
    pub current_tick: Tick,
    pub nearest_food_zone: Option<(u32, Vec2, f32)>,
    pub perceived_dispositions: Vec<(EntityId, Disposition)>,
}

/// Main action selection function for lupine
pub fn select_action_lupine(ctx: &LupineSelectionContext) -> Option<Task> {
    // Critical needs always take priority
    if let Some(critical) = ctx.needs.has_critical() {
        return select_critical_response_lupine(critical, ctx);
    }

    // Don't interrupt existing tasks
    if ctx.has_current_task {
        return None;
    }

    // The beast takes over, launching into a furious, infectious attack.
    if ctx.values.bestial_rage > 0.85 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Charge,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // Leaps to the aid of a threatened packmate.
    if ctx.values.pack_loyalty > 0.75 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Defend,
                TaskPriority::High,
                ctx.current_tick,
            ));
        }
    }

    // A flicker of humanity allows for wary parley.
    if ctx.values.human_restraint > 0.6 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::TalkTo,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Defends claimed territory from intruders.
    if ctx.values.territorial_hunger > 0.7 {
        if ctx.entity_nearby {
            return Some(Task::new(
                ActionId::Attack,
                TaskPriority::Normal,
                ctx.current_tick,
            ));
        }
    }

    // Address moderate needs
    if let Some(task) = address_moderate_need_lupine(ctx) {
        return Some(task);
    }

    // Fall back to idle behavior
    Some(select_idle_action_lupine(ctx))
}

/// Handle critical needs for lupine
fn select_critical_response_lupine(need: NeedType, ctx: &LupineSelectionContext) -> Option<Task> {
    match need {
        NeedType::Safety if ctx.threat_nearby => Some(Task {
            action: ActionId::Flee,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Safety => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.food_available => Some(Task {
            action: ActionId::Eat,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Food if ctx.nearest_food_zone.is_some() => {
            let (_, food_pos, _) = ctx.nearest_food_zone.unwrap();
            Some(Task {
                action: ActionId::MoveTo,
                target_position: Some(food_pos),
                target_entity: None,
                target_building: None,
                priority: TaskPriority::Critical,
                created_tick: ctx.current_tick,
                progress: 0.0,
                source: TaskSource::Autonomous,
            })
        }
        NeedType::Food => Some(Task {
            action: ActionId::IdleWander,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Autonomous,
        }),
        NeedType::Rest if ctx.safe_location => Some(Task {
            action: ActionId::Rest,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        NeedType::Rest => Some(Task {
            action: ActionId::SeekSafety,
            target_position: None,
            target_entity: None,
            target_building: None,
            priority: TaskPriority::Critical,
            created_tick: ctx.current_tick,
            progress: 0.0,
            source: TaskSource::Reaction,
        }),
        _ => None,
    }
}

/// Address moderate needs for lupine
fn address_moderate_need_lupine(ctx: &LupineSelectionContext) -> Option<Task> {
    let (need_type, level) = ctx.needs.most_pressing();

    if level < 0.5 {
        return None;
    }

    let action = match need_type {
        NeedType::Food if ctx.food_available => ActionId::Eat,
        NeedType::Rest if ctx.safe_location => ActionId::Rest,
        NeedType::Social if ctx.entity_nearby => ActionId::TalkTo,
        NeedType::Purpose => ActionId::Gather,
        NeedType::Safety if !ctx.safe_location => ActionId::SeekSafety,
        _ => return None,
    };

    Some(Task::new(action, TaskPriority::Normal, ctx.current_tick))
}

/// Select an idle action based on lupine values
fn select_idle_action_lupine(ctx: &LupineSelectionContext) -> Task {
    if ctx.values.territorial_hunger > 0.4 {
        return Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick);
    } else if ctx.values.pack_loyalty > 0.5 {
        if ctx.entity_nearby {
            return Task::new(ActionId::Follow, TaskPriority::Low, ctx.current_tick);
        }
    }
    // Default idle behavior
    Task::new(ActionId::IdleWander, TaskPriority::Low, ctx.current_tick)
}
// CODEGEN: species_select_action

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::thoughts::{CauseType, Thought, Valence};

    #[test]
    fn test_critical_safety_need_with_threat() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.safety = 0.9; // Critical level
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: true,
            food_available: true,
            safe_location: false,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::Flee);
        assert_eq!(task.priority, TaskPriority::Critical);
    }

    #[test]
    fn test_critical_food_need() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.food = 0.9; // Critical level
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::Eat);
        assert_eq!(task.priority, TaskPriority::Critical);
    }

    #[test]
    fn test_dont_interrupt_existing_task() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: true, // Already has a task
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_none()); // Should not interrupt
    }

    #[test]
    fn test_high_justice_responds_to_injustice() {
        let body = BodyState::new();
        let needs = Needs::default();
        let mut thoughts = ThoughtBuffer::new();
        thoughts.add(Thought::new(
            Valence::Negative,
            0.8,
            "injustice",
            "witnessed unfair treatment",
            CauseType::Event,
            0,
        ));
        let mut values = HumanValues::default();
        values.justice = 0.9;

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::Help);
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn test_moderate_social_need() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.social = 0.7; // Moderate level
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true, // Someone to talk to
            current_tick: 0,
            nearest_food_zone: None,
            // TalkTo requires perceived targets; use a neutral disposition
            perceived_dispositions: vec![(EntityId::new(), Disposition::Neutral)],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::TalkTo);
        assert_eq!(task.priority, TaskPriority::Normal);
    }

    #[test]
    fn test_curious_entity_observes() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let mut values = HumanValues::default();
        values.curiosity = 0.9;
        values.love = 0.1;
        values.loyalty = 0.1;
        values.comfort = 0.1;

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::IdleObserve);
        assert_eq!(task.priority, TaskPriority::Low);
    }

    #[test]
    fn test_social_entity_talks_when_others_nearby() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let mut values = HumanValues::default();
        values.love = 0.9;
        values.loyalty = 0.9;
        values.curiosity = 0.1;
        values.comfort = 0.1;

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            // TalkTo requires perceived targets; use a friendly disposition for social entity
            perceived_dispositions: vec![(EntityId::new(), Disposition::Friendly)],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::TalkTo);
        assert_eq!(task.priority, TaskPriority::Low);
    }

    #[test]
    fn test_default_idle_wander() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default(); // All values at default (0.0)

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false, // No one to talk to
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::IdleWander);
        assert_eq!(task.priority, TaskPriority::Low);
    }

    #[test]
    fn test_high_safety_value_flees_from_danger_thought() {
        let body = BodyState::new();
        let needs = Needs::default();
        let mut thoughts = ThoughtBuffer::new();
        thoughts.add(Thought::new(
            Valence::Negative,
            0.8,
            "danger",
            "sensed imminent threat",
            CauseType::Event,
            0,
        ));
        let mut values = HumanValues::default();
        values.safety = 0.9;

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::Flee);
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn test_critical_need_overrides_existing_task() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.safety = 0.95; // Critical safety need
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: true, // Has existing task
            threat_nearby: true,
            food_available: true,
            safe_location: false,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        // Critical needs should still trigger even with existing task
        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::Flee);
        assert_eq!(task.priority, TaskPriority::Critical);
    }

    #[test]
    fn test_hungry_entity_moves_to_food() {
        use crate::core::types::Vec2;

        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.food = 0.85; // Critical hunger
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: false, // No food at current position
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: Some((0, Vec2::new(100.0, 100.0), 50.0)), // Food zone nearby
            perceived_dispositions: vec![],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::MoveTo);
        // Check target_position coordinates since Vec2 doesn't implement PartialEq
        let target = task
            .target_position
            .expect("Expected target_position to be set");
        assert!((target.x - 100.0).abs() < 0.001);
        assert!((target.y - 100.0).abs() < 0.001);
    }

    #[test]
    fn test_hostile_disposition_influences_action() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.safety = 0.6; // Elevated safety need (not critical, but concerned)
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a hostile entity
        let hostile_entity = EntityId::new();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![(hostile_entity, Disposition::Hostile)],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // With hostile entity nearby and elevated safety need, should flee
        assert_eq!(task.action, ActionId::Flee);
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn test_suspicious_disposition_triggers_flee() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.safety = 0.6; // Elevated safety need
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a suspicious entity
        let suspicious_entity = EntityId::new();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![(suspicious_entity, Disposition::Suspicious)],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // Suspicious entities also trigger flee response
        assert_eq!(task.action, ActionId::Flee);
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn test_friendly_disposition_influences_action() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.social = 0.6; // Elevated social need
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a friendly entity
        let friendly_entity = EntityId::new();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![(friendly_entity, Disposition::Friendly)],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // With friendly entity nearby and elevated social need, should talk
        assert_eq!(task.action, ActionId::TalkTo);
        assert_eq!(task.target_entity, Some(friendly_entity));
        assert_eq!(task.priority, TaskPriority::Normal);
    }

    #[test]
    fn test_favorable_disposition_triggers_talk() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.social = 0.6; // Elevated social need
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a favorable entity
        let favorable_entity = EntityId::new();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![(favorable_entity, Disposition::Favorable)],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // Favorable entities also trigger talk response
        assert_eq!(task.action, ActionId::TalkTo);
        assert_eq!(task.target_entity, Some(favorable_entity));
    }

    #[test]
    fn test_hostile_takes_priority_over_friendly() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.safety = 0.6; // Both safety and social needs elevated
        needs.social = 0.6;
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create both hostile and friendly entities
        let hostile_entity = EntityId::new();
        let friendly_entity = EntityId::new();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![
                (hostile_entity, Disposition::Hostile),
                (friendly_entity, Disposition::Friendly),
            ],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // Safety response (flee from hostile) takes priority over social (talk to friendly)
        assert_eq!(task.action, ActionId::Flee);
    }

    #[test]
    fn test_low_safety_need_ignores_hostile() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.safety = 0.3; // Low safety need (entity feels safe)
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a hostile entity
        let hostile_entity = EntityId::new();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![(hostile_entity, Disposition::Hostile)],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // With low safety need, hostile entity doesn't trigger flee
        assert_ne!(task.action, ActionId::Flee);
    }

    #[test]
    fn test_low_social_need_ignores_friendly() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.social = 0.3; // Low social need (entity doesn't want to talk)
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a friendly entity
        let friendly_entity = EntityId::new();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![(friendly_entity, Disposition::Friendly)],
            building_skill: 0.0,
            nearest_building_site: None,
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // With low social need, friendly entity doesn't trigger talk
        // (falls through to idle behavior)
        assert_ne!(task.action, ActionId::TalkTo);
    }

    // ========================================================================
    // PURPOSE-DRIVEN BUILDING WORK TESTS
    // ========================================================================

    #[test]
    fn test_high_purpose_seeks_building_work() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.purpose = 0.8; // High purpose need
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a building site
        let building_id = BuildingId::new();
        let building_pos = Vec2::new(10.0, 10.0);

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
            building_skill: 0.5,
            nearest_building_site: Some((building_id, building_pos, 20.0)),
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // With high purpose need and building site nearby, should seek building work
        assert_eq!(task.action, ActionId::Build);
        assert_eq!(task.target_building, Some(building_id));
        assert!(task.target_position.is_some());
        assert_eq!(task.priority, TaskPriority::Normal);
    }

    #[test]
    fn test_low_purpose_does_not_seek_building() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.purpose = 0.4; // Low purpose need (below 0.6 threshold)
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a building site
        let building_id = BuildingId::new();
        let building_pos = Vec2::new(10.0, 10.0);

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
            building_skill: 0.5,
            nearest_building_site: Some((building_id, building_pos, 20.0)),
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // With low purpose need, should NOT seek building work (falls to idle)
        assert_ne!(task.action, ActionId::Build);
    }

    #[test]
    fn test_high_purpose_no_building_site() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.purpose = 0.8; // High purpose need
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
            building_skill: 0.5,
            nearest_building_site: None, // No building site
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // With high purpose but no building site, should NOT build
        // Falls through to address_moderate_need which returns Gather for Purpose
        assert_ne!(task.action, ActionId::Build);
    }

    #[test]
    fn test_very_high_purpose_overrides_low_skill() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.purpose = 1.0; // Maximum purpose need
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a building site
        let building_id = BuildingId::new();
        let building_pos = Vec2::new(10.0, 10.0);

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
            building_skill: 0.0, // Zero skill
            nearest_building_site: Some((building_id, building_pos, 20.0)),
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // With very high purpose, even zero skill should seek building work
        // skill_weight = 0.5, purpose_weight = (1.0-0.6)*2.5 = 1.0, product = 0.5 > 0.3
        assert_eq!(task.action, ActionId::Build);
    }

    #[test]
    fn test_moderate_purpose_needs_skill() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.purpose = 0.7; // Moderate purpose need
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a building site
        let building_id = BuildingId::new();
        let building_pos = Vec2::new(10.0, 10.0);

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
            building_skill: 0.0, // Zero skill
            nearest_building_site: Some((building_id, building_pos, 20.0)),
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // With moderate purpose and zero skill, the threshold isn't met
        // skill_weight = 0.5, purpose_weight = (0.7-0.6)*2.5 = 0.25, product = 0.125 < 0.3
        assert_ne!(task.action, ActionId::Build);
    }

    #[test]
    fn test_skilled_builder_seeks_work_at_moderate_purpose() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.purpose = 0.73; // Just above threshold with skill
        let thoughts = ThoughtBuffer::new();
        let values = HumanValues::default();

        // Create a building site
        let building_id = BuildingId::new();
        let building_pos = Vec2::new(10.0, 10.0);

        let ctx = SelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
            building_skill: 1.0, // Max skill
            nearest_building_site: Some((building_id, building_pos, 20.0)),
        };

        let task = select_action_human(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // With max skill (1.0), skilled builder seeks work earlier
        // skill_weight = 1.0, purpose_weight = (0.73-0.6)*2.5 = 0.325, product = 0.325 > 0.3
        assert_eq!(task.action, ActionId::Build);
    }

    // ========================================================================
    // ORC ACTION SELECTION TESTS
    // ========================================================================

    #[test]
    fn test_orc_high_rage_attacks() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let mut values = OrcValues::default();
        values.rage = 0.9; // High rage

        let target_entity = EntityId::new();

        let ctx = OrcSelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![(target_entity, Disposition::Neutral)],
        };

        let task = select_action_orc(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::Defend);
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn test_orc_blood_debt_seeks_revenge() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let mut values = OrcValues::default();
        values.blood_debt = 0.8; // High blood debt

        let hostile_entity = EntityId::new();

        let ctx = OrcSelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![(hostile_entity, Disposition::Hostile)],
        };

        let task = select_action_orc(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::Defend);
        assert_eq!(task.target_entity, Some(hostile_entity));
    }

    #[test]
    fn test_orc_strong_fights_instead_of_flees() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.safety = 0.9; // Critical safety need
        let thoughts = ThoughtBuffer::new();
        let mut values = OrcValues::default();
        values.strength = 0.9; // High strength

        let ctx = OrcSelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: true,
            food_available: true,
            safe_location: false,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_orc(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // Strong orcs fight instead of flee
        assert_eq!(task.action, ActionId::Defend);
        assert_eq!(task.priority, TaskPriority::Critical);
    }

    #[test]
    fn test_orc_weak_flees_from_threat() {
        let body = BodyState::new();
        let mut needs = Needs::default();
        needs.safety = 0.9; // Critical safety need
        let thoughts = ThoughtBuffer::new();
        let mut values = OrcValues::default();
        values.strength = 0.3; // Low strength

        let ctx = OrcSelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: true,
            food_available: true,
            safe_location: false,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_orc(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // Weak orcs flee
        assert_eq!(task.action, ActionId::Flee);
    }

    #[test]
    fn test_orc_clan_loyalty_defends_ally() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let mut values = OrcValues::default();
        values.clan_loyalty = 0.9; // High clan loyalty

        let friendly_entity = EntityId::new();
        let hostile_entity = EntityId::new();

        let ctx = OrcSelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: true,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![
                (friendly_entity, Disposition::Friendly),
                (hostile_entity, Disposition::Hostile),
            ],
        };

        let task = select_action_orc(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        // Loyal orc defends clan member
        assert_eq!(task.action, ActionId::Defend);
        assert_eq!(task.target_entity, Some(hostile_entity));
    }

    #[test]
    fn test_orc_idle_high_combat_prowess_wanders() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let mut values = OrcValues::default();
        values.combat_prowess = 0.9;

        let ctx = OrcSelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_orc(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::IdleWander);
        assert_eq!(task.priority, TaskPriority::Low);
    }

    #[test]
    fn test_orc_idle_high_dominance_observes() {
        let body = BodyState::new();
        let needs = Needs::default();
        let thoughts = ThoughtBuffer::new();
        let mut values = OrcValues::default();
        values.dominance = 0.9;
        values.combat_prowess = 0.3; // Low combat prowess so dominance takes precedence

        let ctx = OrcSelectionContext {
            body: &body,
            needs: &needs,
            thoughts: &thoughts,
            values: &values,
            has_current_task: false,
            threat_nearby: false,
            food_available: true,
            safe_location: true,
            entity_nearby: false,
            current_tick: 0,
            nearest_food_zone: None,
            perceived_dispositions: vec![],
        };

        let task = select_action_orc(&ctx);
        assert!(task.is_some());
        let task = task.unwrap();
        assert_eq!(task.action, ActionId::IdleObserve);
    }
}
