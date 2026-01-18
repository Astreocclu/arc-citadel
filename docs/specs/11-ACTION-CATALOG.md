# 11-ACTION-CATALOG
> Complete reference of all actions entities can perform

## Overview

Actions are the atomic behaviors entities can execute. Each action has properties, requirements, outcomes, and need satisfactions. Actions are selected through the cognitive pipeline (values + needs + context) rather than scripted behavior.

---

## Action Structure

```rust
pub enum ActionId {
    // Movement
    Walk,
    Run,
    Flee,
    Approach,

    // Social
    Talk,
    Greet,
    Argue,
    Comfort,
    Intimidate,
    Flirt,
    Trade,

    // Work
    Gather,
    Craft,
    Build,
    Farm,
    Mine,
    Hunt,
    Fish,
    Cook,
    Heal,

    // Combat
    Attack,
    Defend,
    Parry,
    Dodge,
    Charge,
    Retreat,
    Rally,

    // Self-Care
    Eat,
    Drink,
    Rest,
    Sleep,

    // Misc
    Wait,
    Wander,
    Observe,
    Pray,
}

pub enum ActionCategory {
    Movement,
    Social,
    Work,
    Combat,
    SelfCare,
    Misc,
}

impl ActionId {
    pub fn category(&self) -> ActionCategory {
        match self {
            Self::Walk | Self::Run | Self::Flee | Self::Approach => ActionCategory::Movement,
            Self::Talk | Self::Greet | Self::Argue | Self::Comfort |
            Self::Intimidate | Self::Flirt | Self::Trade => ActionCategory::Social,
            Self::Gather | Self::Craft | Self::Build | Self::Farm |
            Self::Mine | Self::Hunt | Self::Fish | Self::Cook | Self::Heal => ActionCategory::Work,
            Self::Attack | Self::Defend | Self::Parry | Self::Dodge |
            Self::Charge | Self::Retreat | Self::Rally => ActionCategory::Combat,
            Self::Eat | Self::Drink | Self::Rest | Self::Sleep => ActionCategory::SelfCare,
            _ => ActionCategory::Misc,
        }
    }
}
```

---

## Action Properties

```rust
pub struct ActionProperties {
    pub duration_ticks: u32,       // Base time to complete
    pub interruptible: bool,       // Can be stopped mid-action
    pub requires_target: bool,     // Needs a target entity/location
    pub stamina_cost: f32,         // Physical exertion
    pub cognitive_load: f32,       // Mental effort (0.0-1.0)
    pub noise_level: f32,          // How detectable (0.0-1.0)
}

impl ActionId {
    pub fn properties(&self) -> ActionProperties {
        match self {
            // Movement
            Self::Walk => ActionProperties {
                duration_ticks: 1,
                interruptible: true,
                requires_target: true,
                stamina_cost: 0.01,
                cognitive_load: 0.0,
                noise_level: 0.2,
            },
            Self::Run => ActionProperties {
                duration_ticks: 1,
                interruptible: true,
                requires_target: true,
                stamina_cost: 0.05,
                cognitive_load: 0.0,
                noise_level: 0.5,
            },
            Self::Flee => ActionProperties {
                duration_ticks: 1,
                interruptible: false,  // Panic state
                requires_target: false, // Away from threat
                stamina_cost: 0.08,
                cognitive_load: 0.0,
                noise_level: 0.7,
            },

            // Social
            Self::Talk => ActionProperties {
                duration_ticks: 10,
                interruptible: true,
                requires_target: true,
                stamina_cost: 0.0,
                cognitive_load: 0.3,
                noise_level: 0.4,
            },
            Self::Trade => ActionProperties {
                duration_ticks: 30,
                interruptible: true,
                requires_target: true,
                stamina_cost: 0.0,
                cognitive_load: 0.5,
                noise_level: 0.3,
            },

            // Work
            Self::Mine => ActionProperties {
                duration_ticks: 60,
                interruptible: true,
                requires_target: true,
                stamina_cost: 0.15,
                cognitive_load: 0.1,
                noise_level: 0.8,
            },
            Self::Craft => ActionProperties {
                duration_ticks: 120,
                interruptible: false, // Would ruin work
                requires_target: true,
                stamina_cost: 0.05,
                cognitive_load: 0.6,
                noise_level: 0.3,
            },

            // Combat
            Self::Attack => ActionProperties {
                duration_ticks: 3,
                interruptible: false,
                requires_target: true,
                stamina_cost: 0.1,
                cognitive_load: 0.4,
                noise_level: 0.9,
            },
            Self::Defend => ActionProperties {
                duration_ticks: 1,
                interruptible: false,
                requires_target: false,
                stamina_cost: 0.03,
                cognitive_load: 0.5,
                noise_level: 0.3,
            },

            // Self-Care
            Self::Eat => ActionProperties {
                duration_ticks: 20,
                interruptible: true,
                requires_target: true, // Food item
                stamina_cost: 0.0,
                cognitive_load: 0.0,
                noise_level: 0.1,
            },
            Self::Sleep => ActionProperties {
                duration_ticks: 480, // 8 in-game hours
                interruptible: true,
                requires_target: false,
                stamina_cost: -0.5, // Recovers stamina
                cognitive_load: 0.0,
                noise_level: 0.1, // Snoring
            },

            _ => ActionProperties::default(),
        }
    }
}
```

---

## Need Satisfaction

Each action satisfies needs at different rates:

```rust
impl ActionId {
    /// Returns (NeedType, satisfaction_per_tick) pairs
    pub fn satisfies_needs(&self) -> Vec<(NeedType, f32)> {
        match self {
            // Self-Care
            Self::Eat => vec![
                (NeedType::Hunger, -0.05),  // Negative = reduces need level
            ],
            Self::Drink => vec![
                (NeedType::Thirst, -0.1),
            ],
            Self::Rest => vec![
                (NeedType::Rest, -0.02),
            ],
            Self::Sleep => vec![
                (NeedType::Rest, -0.005),
                (NeedType::Hunger, 0.001),  // Get hungry while sleeping
            ],

            // Social
            Self::Talk => vec![
                (NeedType::Social, -0.03),
            ],
            Self::Greet => vec![
                (NeedType::Social, -0.01),
            ],
            Self::Comfort => vec![
                (NeedType::Social, -0.02),
                (NeedType::Purpose, -0.01), // Helping feels purposeful
            ],

            // Work
            Self::Craft => vec![
                (NeedType::Purpose, -0.02),
            ],
            Self::Build => vec![
                (NeedType::Purpose, -0.03),
            ],
            Self::Gather => vec![
                (NeedType::Purpose, -0.01),
            ],

            // Combat (increases safety need when successful)
            Self::Attack => vec![
                (NeedType::Safety, 0.02),    // Danger increases safety need
                (NeedType::Purpose, -0.01),  // But feels purposeful
            ],
            Self::Flee => vec![
                (NeedType::Safety, -0.03),   // Distance reduces safety need
            ],
            Self::Rally => vec![
                (NeedType::Safety, -0.02),
                (NeedType::Social, -0.02),
            ],

            // Misc
            Self::Pray => vec![
                (NeedType::Purpose, -0.02),
                (NeedType::Social, -0.01),   // Community feeling
            ],
            Self::Wander => vec![
                (NeedType::Rest, -0.005),    // Light activity is restful
            ],

            _ => vec![],
        }
    }
}
```

---

## Action Requirements

```rust
impl ActionId {
    /// Check if action can be performed
    pub fn can_perform(&self, entity: &EntityState, world: &World) -> ActionResult {
        match self {
            Self::Eat => {
                if !entity.has_food() {
                    ActionResult::Blocked(BlockReason::NoResource("food"))
                } else {
                    ActionResult::Allowed
                }
            }
            Self::Drink => {
                if !entity.has_water_access(world) {
                    ActionResult::Blocked(BlockReason::NoResource("water"))
                } else {
                    ActionResult::Allowed
                }
            }
            Self::Talk | Self::Trade | Self::Greet => {
                if !entity.has_nearby_person(world) {
                    ActionResult::Blocked(BlockReason::NoTarget)
                } else {
                    ActionResult::Allowed
                }
            }
            Self::Mine => {
                if !entity.has_mining_tool() {
                    ActionResult::Blocked(BlockReason::NoTool("pickaxe"))
                } else if !entity.at_mineable_location(world) {
                    ActionResult::Blocked(BlockReason::WrongLocation)
                } else {
                    ActionResult::Allowed
                }
            }
            Self::Attack => {
                if !entity.has_weapon() && !entity.can_unarmed_attack() {
                    ActionResult::Blocked(BlockReason::NoWeapon)
                } else if !entity.has_valid_target(world) {
                    ActionResult::Blocked(BlockReason::NoTarget)
                } else if entity.fatigue > 0.95 {
                    ActionResult::Blocked(BlockReason::TooExhausted)
                } else {
                    ActionResult::Allowed
                }
            }
            Self::Sleep => {
                if entity.in_combat() {
                    ActionResult::Blocked(BlockReason::InCombat)
                } else if entity.needs.rest < 0.3 {
                    ActionResult::Blocked(BlockReason::NotTired)
                } else {
                    ActionResult::Allowed
                }
            }
            Self::Craft => {
                if !entity.has_materials_for(entity.current_recipe) {
                    ActionResult::Blocked(BlockReason::NoMaterials)
                } else if !entity.at_workshop(world) {
                    ActionResult::Blocked(BlockReason::NoWorkshop)
                } else {
                    ActionResult::Allowed
                }
            }
            _ => ActionResult::Allowed,
        }
    }
}

pub enum ActionResult {
    Allowed,
    Blocked(BlockReason),
}

pub enum BlockReason {
    NoResource(&'static str),
    NoTarget,
    NoTool(&'static str),
    NoWeapon,
    NoMaterials,
    NoWorkshop,
    WrongLocation,
    TooExhausted,
    InCombat,
    NotTired,
}
```

---

## Action Execution

### Movement Actions

```rust
pub mod movement {
    pub fn execute_walk(
        entity_idx: usize,
        target: Vec2,
        positions: &mut [Vec2],
        velocities: &mut [Vec2],
        dt: f32,
    ) -> ExecutionResult {
        let current = positions[entity_idx];
        let direction = (target - current).normalize_or_zero();
        let speed = 2.0;  // units per second

        velocities[entity_idx] = direction * speed;
        positions[entity_idx] += velocities[entity_idx] * dt;

        if positions[entity_idx].distance(target) < 0.5 {
            ExecutionResult::Completed
        } else {
            ExecutionResult::InProgress
        }
    }

    pub fn execute_flee(
        entity_idx: usize,
        threat_pos: Vec2,
        positions: &mut [Vec2],
        velocities: &mut [Vec2],
        dt: f32,
    ) -> ExecutionResult {
        let current = positions[entity_idx];
        let away = (current - threat_pos).normalize_or_zero();
        let speed = 4.0;  // Faster than walking

        velocities[entity_idx] = away * speed;
        positions[entity_idx] += velocities[entity_idx] * dt;

        // Flee continues until safe
        ExecutionResult::InProgress
    }
}
```

### Social Actions

```rust
pub mod social {
    pub fn execute_talk(
        speaker: EntityId,
        listener: EntityId,
        world: &mut World,
        tick_progress: u32,
    ) -> ExecutionResult {
        // Generate conversation topic from values intersection
        if tick_progress == 0 {
            let topic = select_conversation_topic(speaker, listener, world);
            world.active_conversations.insert(speaker, Conversation {
                participants: vec![speaker, listener],
                topic,
                sentiment: 0.0,
            });
        }

        // Update social memory
        if tick_progress >= 10 {
            update_social_memory(speaker, listener, world);
            update_social_memory(listener, speaker, world);
            return ExecutionResult::Completed;
        }

        ExecutionResult::InProgress
    }

    pub fn execute_trade(
        buyer: EntityId,
        seller: EntityId,
        item: ItemId,
        world: &mut World,
    ) -> ExecutionResult {
        let price = calculate_price(item, buyer, seller, world);

        if world.get_money(buyer) >= price {
            world.transfer_item(seller, buyer, item);
            world.transfer_money(buyer, seller, price);
            update_trade_reputation(buyer, seller, world);
            ExecutionResult::Completed
        } else {
            ExecutionResult::Failed(FailReason::InsufficientFunds)
        }
    }
}
```

### Work Actions

```rust
pub mod work {
    pub fn execute_mine(
        miner: EntityId,
        deposit: DepositId,
        world: &mut World,
        tick_progress: u32,
    ) -> ExecutionResult {
        let skill = world.get_skill(miner, SkillType::Mining);
        let tool_quality = world.get_tool_quality(miner, ToolType::Pickaxe);

        // Mining rate depends on skill and tool
        let rate = 0.1 * skill * tool_quality;

        if tick_progress >= 60 {
            let ore = world.deposits[deposit].extract(rate);
            world.add_to_inventory(miner, ore);

            // Tool wear
            world.degrade_tool(miner, ToolType::Pickaxe, 0.01);

            ExecutionResult::Completed
        } else {
            ExecutionResult::InProgress
        }
    }

    pub fn execute_craft(
        crafter: EntityId,
        recipe: RecipeId,
        world: &mut World,
        tick_progress: u32,
    ) -> ExecutionResult {
        let skill = world.get_skill(crafter, recipe.skill_type());
        let duration = recipe.base_duration() / skill;

        if tick_progress >= duration as u32 {
            // Consume materials
            if !world.consume_materials(crafter, &recipe.materials()) {
                return ExecutionResult::Failed(FailReason::MissingMaterials);
            }

            // Create item with quality based on skill
            let quality = calculate_craft_quality(skill, recipe.difficulty());
            let item = Item::new(recipe.output(), quality);
            world.add_to_inventory(crafter, item);

            // Skill improvement
            world.improve_skill(crafter, recipe.skill_type(), 0.01);

            ExecutionResult::Completed
        } else {
            ExecutionResult::InProgress
        }
    }
}
```

### Combat Actions

```rust
pub mod combat {
    pub fn execute_attack(
        attacker: EntityId,
        defender: EntityId,
        world: &mut World,
    ) -> ExecutionResult {
        // Get physical properties
        let weapon = world.get_equipped_weapon(attacker);
        let strength = world.get_strength(attacker);
        let skill = world.get_skill(attacker, SkillType::Combat);
        let fatigue = world.get_fatigue(attacker);

        // Calculate impact (see 14-PHYSICS-COMBAT-SPEC)
        let impact = calculate_impact(strength, weapon, skill, fatigue);

        // Resolve against defender
        let armor = world.get_equipped_armor(defender);
        let result = resolve_attack(impact, armor, world.rng());

        // Apply damage
        match result {
            AttackResult::Hit { damage, location } => {
                world.apply_damage(defender, damage, location);
                ExecutionResult::Completed
            }
            AttackResult::Blocked { blunt_damage } => {
                world.apply_blunt_damage(defender, blunt_damage);
                ExecutionResult::Completed
            }
            AttackResult::Miss => {
                ExecutionResult::Completed
            }
        }
    }

    pub fn execute_defend(
        defender: EntityId,
        world: &mut World,
    ) -> ExecutionResult {
        // Defensive stance: improved parry/dodge until next action
        world.set_stance(defender, Stance::Defensive);
        ExecutionResult::Completed
    }
}
```

---

## Action Selection Integration

Actions are selected by the cognitive pipeline, not hardcoded:

```rust
pub fn score_action(
    entity_idx: usize,
    action: ActionId,
    context: &ActionContext,
    world: &World,
) -> f32 {
    let needs = &world.humans.needs[entity_idx];
    let values = &world.humans.values[entity_idx];

    // Base score from need satisfaction
    let mut score = 0.0;
    for (need_type, satisfaction) in action.satisfies_needs() {
        let need_urgency = needs.get(need_type);
        score += need_urgency * satisfaction.abs();
    }

    // Value alignment bonus
    score += value_alignment(action, values, context);

    // Context modifiers
    if action == ActionId::Flee && context.threats.is_empty() {
        score *= 0.1;  // No point fleeing without threat
    }
    if action == ActionId::Attack && context.in_battle {
        score *= 1.5;  // Combat actions more relevant in battle
    }

    // Requirements check
    if !action.can_perform(&context.entity_state, world).is_allowed() {
        score = 0.0;
    }

    score
}

fn value_alignment(action: ActionId, values: &HumanValues, context: &ActionContext) -> f32 {
    match action {
        ActionId::Flee => values.safety * 0.5,
        ActionId::Rally => values.loyalty * 0.5 + values.honor * 0.3,
        ActionId::Comfort => values.love * 0.3 + values.loyalty * 0.2,
        ActionId::Pray => values.piety * 0.5,
        ActionId::Trade => values.ambition * 0.3,
        ActionId::Craft => values.beauty * 0.2 + values.ambition * 0.2,
        _ => 0.0,
    }
}
```

---

## Related Specifications

| Document | Relationship |
|----------|--------------|
| [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md) | Action selection pipeline |
| [05-BATTLE-SYSTEM-SPEC](05-BATTLE-SYSTEM-SPEC.md) | Combat action details |
| [14-PHYSICS-COMBAT-SPEC](14-PHYSICS-COMBAT-SPEC.md) | Attack physics |
| [16-RESOURCE-ECONOMY-SPEC](16-RESOURCE-ECONOMY-SPEC.md) | Work action outputs |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification created |
