# 18-SOCIAL-PRESSURE-MORALE-SPEC
> Group dynamics: morale, social pressure, and collective behavior emergence

## Overview

Entities exist within social contexts that influence their behavior. Morale is not a single number—it emerges from the interaction of individual state, group cohesion, leadership, and circumstances. This specification defines how social pressure and morale affect entity decisions and group behavior.

---

## Core Philosophy

**Morale emerges from physical and social conditions, not abstract modifiers.**

```rust
// ✅ CORRECT: Morale from conditions
let hunger_effect = entity.hunger.current / entity.hunger.max;
let wound_effect = entity.wounds.total_severity();
let group_effect = group.cohesion * nearby_allies_ratio;
let leadership_effect = leader_presence * leader_inspiration;

let morale = base_resolve
    - (hunger_effect * 0.3)
    - (wound_effect * 0.4)
    + (group_effect * 0.3)
    + (leadership_effect * 0.2);

// ❌ FORBIDDEN: Abstract morale modifiers
let morale = base_morale * morale_bonus * leadership_multiplier; // NEVER
```

---

## Individual Morale

### Morale State

```rust
/// Individual morale emerges from conditions
#[derive(Debug, Clone)]
pub struct MoraleState {
    // Base psychological state
    pub resolve: f32,              // 0.0 - 1.0: willingness to continue
    pub confidence: f32,           // 0.0 - 1.0: belief in success
    pub fear: f32,                 // 0.0 - 1.0: fear level

    // Combat-specific
    pub combat_fatigue: f32,       // 0.0 - 1.0: psychological exhaustion
    pub witnessed_deaths: u32,
    pub consecutive_failures: u32,

    // Recent events (decay over time)
    pub recent_victories: f32,     // decaying count
    pub recent_defeats: f32,
    pub recent_friendly_casualties: f32,

    // Thresholds
    pub panic_threshold: f32,      // fear level that triggers panic
    pub rout_threshold: f32,       // resolve level that triggers rout
}

impl MoraleState {
    /// Calculate effective morale from current state
    pub fn effective_morale(&self) -> f32 {
        let base = (self.resolve + self.confidence) / 2.0;
        let fear_penalty = self.fear * 0.5;
        let fatigue_penalty = self.combat_fatigue * 0.3;

        (base - fear_penalty - fatigue_penalty).clamp(0.0, 1.0)
    }

    /// Check if entity should panic
    pub fn check_panic(&self) -> bool {
        self.fear > self.panic_threshold
    }

    /// Check if entity should rout
    pub fn check_rout(&self) -> bool {
        self.resolve < self.rout_threshold
    }

    /// Process a morale event
    pub fn process_event(&mut self, event: MoraleEvent, personality: &Personality) {
        match event {
            MoraleEvent::AllyDeath { was_leader, was_close } => {
                self.witnessed_deaths += 1;
                self.recent_friendly_casualties += 1.0;

                // Fear increases
                let fear_increase = if was_leader { 0.3 } else { 0.1 }
                    + if was_close { 0.15 } else { 0.0 };
                self.fear += fear_increase * (1.0 - personality.courage);

                // Resolve tested
                self.resolve -= 0.1 * (1.0 - personality.loyalty);
            }

            MoraleEvent::EnemyDeath => {
                self.recent_victories += 0.5;
                self.confidence += 0.05;
                self.fear -= 0.03;
            }

            MoraleEvent::PersonalWound { severity } => {
                self.fear += severity * 0.3 * (1.0 - personality.courage);
                self.resolve -= severity * 0.2;
            }

            MoraleEvent::NearMiss => {
                self.fear += 0.1 * (1.0 - personality.courage);
            }

            MoraleEvent::VictoryAchieved { magnitude } => {
                self.recent_victories += magnitude;
                self.confidence += magnitude * 0.3;
                self.resolve = (self.resolve + 0.2).min(1.0);
                self.fear *= 0.7; // Victory reduces fear
            }

            MoraleEvent::DefeatSuffered { magnitude } => {
                self.recent_defeats += magnitude;
                self.consecutive_failures += 1;
                self.confidence -= magnitude * 0.4;
                self.resolve -= magnitude * 0.2;
            }

            MoraleEvent::LeaderSpeech { inspiration } => {
                self.resolve += inspiration * 0.3 * personality.loyalty;
                self.confidence += inspiration * 0.2;
                self.fear -= inspiration * 0.1;
            }

            MoraleEvent::Retreat => {
                self.combat_fatigue *= 0.8; // Rest helps
                self.fear *= 0.9;
            }

            MoraleEvent::Surrounded => {
                self.fear += 0.3;
                self.resolve -= 0.1;
            }

            MoraleEvent::ReinforcementsArrived => {
                self.confidence += 0.3;
                self.fear -= 0.2;
            }
        }

        // Clamp values
        self.resolve = self.resolve.clamp(0.0, 1.0);
        self.confidence = self.confidence.clamp(0.0, 1.0);
        self.fear = self.fear.clamp(0.0, 1.0);
    }

    /// Decay recent events over time
    pub fn decay(&mut self, dt_hours: f32) {
        let decay_rate = 0.1 * dt_hours;

        self.recent_victories *= 1.0 - decay_rate;
        self.recent_defeats *= 1.0 - decay_rate;
        self.recent_friendly_casualties *= 1.0 - decay_rate * 0.5;

        // Combat fatigue recovers slowly
        self.combat_fatigue *= 1.0 - (decay_rate * 0.3);

        // Fear naturally subsides
        self.fear *= 1.0 - (decay_rate * 0.5);

        // Confidence regression to mean
        let confidence_drift = (0.5 - self.confidence) * decay_rate * 0.2;
        self.confidence += confidence_drift;
    }
}

#[derive(Debug, Clone)]
pub enum MoraleEvent {
    AllyDeath { was_leader: bool, was_close: bool },
    EnemyDeath,
    PersonalWound { severity: f32 },
    NearMiss,
    VictoryAchieved { magnitude: f32 },
    DefeatSuffered { magnitude: f32 },
    LeaderSpeech { inspiration: f32 },
    Retreat,
    Surrounded,
    ReinforcementsArrived,
}

/// Personality traits affecting morale
#[derive(Debug, Clone)]
pub struct Personality {
    pub courage: f32,      // 0.0 - 1.0: resistance to fear
    pub loyalty: f32,      // 0.0 - 1.0: group attachment
    pub aggression: f32,   // 0.0 - 1.0: offensive tendency
    pub discipline: f32,   // 0.0 - 1.0: ability to follow orders under stress
}
```

### Physical Conditions Affecting Morale

```rust
/// Calculate morale modifiers from physical state
pub fn physical_morale_effects(
    needs: &Needs,
    wounds: &WoundState,
    fatigue: &FatigueState,
) -> MoraleModifiers {
    // Hunger
    let hunger_ratio = needs.hunger.current / needs.hunger.max;
    let hunger_effect = if hunger_ratio < 0.3 {
        -0.2 * (1.0 - hunger_ratio / 0.3)
    } else {
        0.0
    };

    // Thirst (more critical)
    let thirst_ratio = needs.thirst.current / needs.thirst.max;
    let thirst_effect = if thirst_ratio < 0.3 {
        -0.3 * (1.0 - thirst_ratio / 0.3)
    } else {
        0.0
    };

    // Wounds
    let wound_effect = -wounds.total_severity() * 0.4;

    // Fatigue
    let fatigue_effect = -fatigue.exertion * 0.3;

    // Sleep deprivation
    let sleep_ratio = needs.sleep.current / needs.sleep.max;
    let sleep_effect = if sleep_ratio < 0.3 {
        -0.25 * (1.0 - sleep_ratio / 0.3)
    } else {
        0.0
    };

    MoraleModifiers {
        hunger: hunger_effect,
        thirst: thirst_effect,
        wounds: wound_effect,
        fatigue: fatigue_effect,
        sleep: sleep_effect,
        total: hunger_effect + thirst_effect + wound_effect + fatigue_effect + sleep_effect,
    }
}

#[derive(Debug, Clone)]
pub struct MoraleModifiers {
    pub hunger: f32,
    pub thirst: f32,
    pub wounds: f32,
    pub fatigue: f32,
    pub sleep: f32,
    pub total: f32,
}
```

---

## Group Morale

### Unit Cohesion

```rust
/// Military unit morale and cohesion
#[derive(Debug)]
pub struct UnitMorale {
    pub unit_id: UnitId,

    // Composition
    pub members: Vec<EntityId>,
    pub leader: Option<EntityId>,
    pub original_strength: u32,

    // Collective state
    pub cohesion: f32,             // 0.0 - 1.0: unit coordination
    pub esprit_de_corps: f32,      // 0.0 - 1.0: unit pride
    pub discipline: f32,           // 0.0 - 1.0: ability to execute orders

    // Combat state
    pub casualties_taken: u32,
    pub casualties_inflicted: u32,
    pub ground_held: bool,
    pub objectives_achieved: u32,

    // Leadership
    pub leadership_presence: f32,  // 0.0 - 1.0
    pub orders_clarity: f32,       // 0.0 - 1.0

    // Supplies
    pub supply_state: SupplyState,
}

#[derive(Debug, Clone)]
pub struct SupplyState {
    pub food_days: f32,            // days of food remaining
    pub water_days: f32,
    pub ammunition_ratio: f32,     // 0.0 - 1.0
    pub medical_supplies: f32,     // 0.0 - 1.0
}

impl UnitMorale {
    /// Calculate unit's collective morale
    pub fn collective_morale(&self) -> f32 {
        // Base from cohesion and esprit
        let base = (self.cohesion + self.esprit_de_corps) / 2.0;

        // Casualty effect
        let casualty_ratio = self.casualties_taken as f32 / self.original_strength as f32;
        let casualty_effect = -casualty_ratio * 0.5;

        // Kill ratio effect
        let kill_ratio = if self.casualties_taken > 0 {
            self.casualties_inflicted as f32 / self.casualties_taken as f32
        } else if self.casualties_inflicted > 0 {
            2.0
        } else {
            1.0
        };
        let kill_effect = (kill_ratio - 1.0).clamp(-0.3, 0.3);

        // Supply effect
        let supply_effect = self.supply_morale_effect();

        // Leadership effect
        let leadership_effect = self.leadership_presence * 0.2;

        // Objectives effect
        let objectives_effect = (self.objectives_achieved as f32 * 0.1).min(0.3);

        (base + casualty_effect + kill_effect + supply_effect
            + leadership_effect + objectives_effect).clamp(0.0, 1.0)
    }

    fn supply_morale_effect(&self) -> f32 {
        let mut effect = 0.0;

        // Food
        if self.supply_state.food_days < 1.0 {
            effect -= 0.3;
        } else if self.supply_state.food_days < 3.0 {
            effect -= 0.15;
        }

        // Water (critical)
        if self.supply_state.water_days < 0.5 {
            effect -= 0.4;
        } else if self.supply_state.water_days < 1.0 {
            effect -= 0.2;
        }

        // Ammunition
        if self.supply_state.ammunition_ratio < 0.2 {
            effect -= 0.25;
        } else if self.supply_state.ammunition_ratio < 0.5 {
            effect -= 0.1;
        }

        effect
    }

    /// Check if unit should break
    pub fn check_unit_break(&self, avg_individual_morale: f32) -> UnitState {
        let collective = self.collective_morale();
        let combined = (collective + avg_individual_morale) / 2.0;

        // Casualty threshold check
        let casualty_ratio = self.casualties_taken as f32 / self.original_strength as f32;

        if casualty_ratio > 0.5 && combined < 0.3 {
            return UnitState::Routed;
        }

        if casualty_ratio > 0.3 && combined < 0.4 {
            return UnitState::Wavering;
        }

        if combined < 0.2 {
            return UnitState::Broken;
        }

        if combined < 0.4 {
            return UnitState::Shaken;
        }

        UnitState::Steady
    }

    /// Process a unit-level event
    pub fn process_unit_event(&mut self, event: UnitEvent) {
        match event {
            UnitEvent::LeaderKilled => {
                self.leader = None;
                self.leadership_presence = 0.0;
                self.cohesion -= 0.3;
                self.discipline -= 0.2;
            }

            UnitEvent::NewLeader { charisma } => {
                self.leadership_presence = charisma * 0.5; // New leader less effective initially
            }

            UnitEvent::OrdersReceived { clarity } => {
                self.orders_clarity = clarity;
                self.discipline += clarity * 0.1;
            }

            UnitEvent::SuccessfulManeuver => {
                self.cohesion += 0.1;
                self.esprit_de_corps += 0.05;
            }

            UnitEvent::FailedManeuver => {
                self.cohesion -= 0.1;
                self.discipline -= 0.05;
            }

            UnitEvent::TookCasualties { count } => {
                self.casualties_taken += count;
                self.cohesion -= count as f32 * 0.02;
            }

            UnitEvent::InflictedCasualties { count } => {
                self.casualties_inflicted += count;
                self.esprit_de_corps += count as f32 * 0.01;
            }

            UnitEvent::ResuppliedComplete => {
                self.supply_state.food_days = 7.0;
                self.supply_state.water_days = 3.0;
                self.supply_state.ammunition_ratio = 1.0;
                self.supply_state.medical_supplies = 1.0;
            }

            UnitEvent::ObjectiveAchieved => {
                self.objectives_achieved += 1;
                self.esprit_de_corps += 0.15;
                self.cohesion += 0.05;
            }
        }

        // Clamp values
        self.cohesion = self.cohesion.clamp(0.0, 1.0);
        self.esprit_de_corps = self.esprit_de_corps.clamp(0.0, 1.0);
        self.discipline = self.discipline.clamp(0.0, 1.0);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitState {
    Steady,       // Normal operation
    Shaken,       // Reduced effectiveness
    Wavering,     // May break soon
    Broken,       // Lost cohesion, individuals flee
    Routed,       // Complete collapse, full retreat
}

#[derive(Debug)]
pub enum UnitEvent {
    LeaderKilled,
    NewLeader { charisma: f32 },
    OrdersReceived { clarity: f32 },
    SuccessfulManeuver,
    FailedManeuver,
    TookCasualties { count: u32 },
    InflictedCasualties { count: u32 },
    ResuppliedComplete,
    ObjectiveAchieved,
}
```

---

## Social Pressure

### Conformity Pressure

```rust
/// Social pressure from group norms
#[derive(Debug)]
pub struct SocialPressure {
    // Group context
    pub group_action: Option<ActionId>,       // what is the group doing
    pub group_movement: Option<Vec2>,         // where is the group going
    pub group_stance: GroupStance,

    // Pressure sources
    pub peer_pressure: f32,                   // pressure from equals
    pub authority_pressure: f32,              // pressure from leaders
    pub crowd_pressure: f32,                  // pressure from numbers

    // Observation
    pub allies_fighting: u32,
    pub allies_fleeing: u32,
    pub allies_total: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum GroupStance {
    Aggressive,
    Defensive,
    Retreating,
    Routing,
    Idle,
}

impl SocialPressure {
    /// Calculate pressure to conform to group action
    pub fn conformity_pressure(&self, proposed_action: ActionId) -> f32 {
        let Some(group_action) = self.group_action else {
            return 0.0;
        };

        if proposed_action == group_action {
            // Conforming - positive pressure
            return self.peer_pressure * 0.5 + self.authority_pressure * 0.3;
        }

        // Non-conforming - negative pressure (resistance)
        -(self.peer_pressure * 0.3 + self.authority_pressure * 0.4)
    }

    /// Calculate cascade effects (fleeing spreads)
    pub fn panic_cascade(&self) -> f32 {
        if self.allies_total == 0 {
            return 0.0;
        }

        let fleeing_ratio = self.allies_fleeing as f32 / self.allies_total as f32;

        // Non-linear: panic spreads faster as more flee
        // Sigmoid curve centered at 30% fleeing
        let panic_spread = 1.0 / (1.0 + (-10.0 * (fleeing_ratio - 0.3)).exp());

        panic_spread * self.crowd_pressure
    }

    /// Calculate rally effects (courage spreads)
    pub fn courage_cascade(&self) -> f32 {
        if self.allies_total == 0 {
            return 0.0;
        }

        let fighting_ratio = self.allies_fighting as f32 / self.allies_total as f32;

        // Courage spreads linearly (easier to break than to rally)
        fighting_ratio * self.peer_pressure * 0.5
    }
}

/// Apply social pressure to action selection
pub fn social_pressure_modifier(
    entity: EntityId,
    proposed_action: ActionId,
    social_context: &SocialPressure,
    personality: &Personality,
) -> f32 {
    let conformity = social_context.conformity_pressure(proposed_action);

    // Personality affects susceptibility
    let susceptibility = 1.0 - (personality.courage * 0.3 + personality.discipline * 0.3);

    // Panic cascade
    let panic = social_context.panic_cascade();
    let panic_effect = if proposed_action == ActionId::Flee {
        panic * susceptibility
    } else {
        -panic * susceptibility * 0.5
    };

    // Courage cascade
    let courage = social_context.courage_cascade();
    let courage_effect = if proposed_action == ActionId::Attack || proposed_action == ActionId::Hold {
        courage * (1.0 - susceptibility)
    } else {
        0.0
    };

    conformity + panic_effect + courage_effect
}
```

### Leadership Effects

```rust
/// Leader's effect on followers
#[derive(Debug)]
pub struct LeadershipEffect {
    pub leader_id: EntityId,

    // Leader qualities
    pub charisma: f32,             // 0.0 - 1.0: personal magnetism
    pub tactical_skill: f32,       // 0.0 - 1.0: perceived competence
    pub reputation: f32,           // 0.0 - 1.0: track record

    // Presence
    pub visibility: f32,           // 0.0 - 1.0: can followers see leader
    pub distance: f32,             // meters from followers
    pub leading_from_front: bool,

    // Recent actions
    pub last_speech: Option<SimTime>,
    pub personal_kills: u32,
    pub wounded_but_fighting: bool,
}

impl LeadershipEffect {
    /// Calculate inspiration provided to followers
    pub fn inspiration_radius(&self) -> f32 {
        // Base radius from charisma
        let base_radius = 20.0 + self.charisma * 30.0;

        // Visibility extends reach
        base_radius * (0.5 + self.visibility * 0.5)
    }

    /// Calculate morale bonus for a follower
    pub fn morale_bonus(&self, follower_distance: f32) -> f32 {
        let in_range = follower_distance <= self.inspiration_radius();
        if !in_range {
            return 0.0;
        }

        let mut bonus = 0.0;

        // Base inspiration from qualities
        bonus += self.charisma * 0.15;
        bonus += self.tactical_skill * 0.1;
        bonus += self.reputation * 0.1;

        // Leading from front is inspiring
        if self.leading_from_front {
            bonus += 0.1;
        }

        // Wounded but fighting is very inspiring
        if self.wounded_but_fighting {
            bonus += 0.15;
        }

        // Personal kills inspire
        bonus += (self.personal_kills as f32 * 0.02).min(0.1);

        // Distance falloff
        let distance_factor = 1.0 - (follower_distance / self.inspiration_radius());
        bonus * distance_factor
    }

    /// Calculate fear reduction from leader presence
    pub fn fear_reduction(&self, follower_distance: f32) -> f32 {
        let bonus = self.morale_bonus(follower_distance);

        // Presence reduces fear
        bonus * 0.5
    }

    /// Give a speech to boost morale
    pub fn give_speech(&self, audience: &[EntityId]) -> SpeechEffect {
        // Speech effectiveness
        let effectiveness = self.charisma * 0.5
            + self.reputation * 0.3
            + self.tactical_skill * 0.2;

        SpeechEffect {
            inspiration: effectiveness,
            duration_hours: 1.0 + effectiveness * 2.0,
            affected_entities: audience.to_vec(),
        }
    }
}

#[derive(Debug)]
pub struct SpeechEffect {
    pub inspiration: f32,
    pub duration_hours: f32,
    pub affected_entities: Vec<EntityId>,
}
```

---

## Settlement Morale

### Civilian Morale

```rust
/// Settlement-wide morale
#[derive(Debug)]
pub struct SettlementMorale {
    pub settlement_id: SettlementId,

    // Base conditions
    pub food_security: f32,        // 0.0 - 1.0
    pub safety: f32,               // 0.0 - 1.0
    pub prosperity: f32,           // 0.0 - 1.0

    // Social factors
    pub governance_approval: f32,  // 0.0 - 1.0
    pub social_cohesion: f32,      // 0.0 - 1.0
    pub cultural_vitality: f32,    // 0.0 - 1.0

    // Recent events
    pub recent_events: Vec<SettlementEvent>,

    // Unrest tracking
    pub unrest_level: f32,         // 0.0 - 1.0
    pub unrest_causes: Vec<UnrestCause>,
}

#[derive(Debug, Clone)]
pub struct SettlementEvent {
    pub event_type: SettlementEventType,
    pub timestamp: SimTime,
    pub morale_impact: f32,
    pub decay_rate: f32,
}

#[derive(Debug, Clone, Copy)]
pub enum SettlementEventType {
    // Positive
    Festival,
    Victory,
    GoodHarvest,
    TradeSuccess,
    BuildingCompleted,

    // Negative
    Raid,
    Famine,
    Plague,
    TaxIncrease,
    LeaderDeath,
    Defeat,

    // Neutral/Mixed
    NewLaw,
    Migration,
    Weather,
}

#[derive(Debug, Clone)]
pub struct UnrestCause {
    pub cause_type: UnrestType,
    pub severity: f32,
    pub affected_population: f32,  // fraction of population affected
}

#[derive(Debug, Clone, Copy)]
pub enum UnrestType {
    Hunger,
    Taxation,
    Oppression,
    Injustice,
    Religious,
    Ethnic,
    Economic,
}

impl SettlementMorale {
    /// Calculate overall morale
    pub fn overall_morale(&self) -> f32 {
        // Base from conditions
        let conditions = (self.food_security * 0.3
            + self.safety * 0.3
            + self.prosperity * 0.2) * 0.6;

        // Social factors
        let social = (self.governance_approval * 0.3
            + self.social_cohesion * 0.4
            + self.cultural_vitality * 0.3) * 0.4;

        // Event effects
        let event_effect: f32 = self.recent_events.iter()
            .map(|e| e.morale_impact)
            .sum();

        (conditions + social + event_effect).clamp(0.0, 1.0)
    }

    /// Calculate risk of unrest
    pub fn unrest_risk(&self) -> f32 {
        let base_risk = 1.0 - self.overall_morale();

        // Unrest causes compound
        let cause_severity: f32 = self.unrest_causes.iter()
            .map(|c| c.severity * c.affected_population)
            .sum();

        (base_risk + cause_severity * 0.5).clamp(0.0, 1.0)
    }

    /// Process daily settlement morale
    pub fn daily_tick(&mut self, settlement: &Settlement, current_time: SimTime) {
        // Update base conditions
        self.food_security = self.calculate_food_security(settlement);
        self.safety = self.calculate_safety(settlement);
        self.prosperity = self.calculate_prosperity(settlement);

        // Decay event effects
        for event in &mut self.recent_events {
            event.morale_impact *= 1.0 - event.decay_rate;
        }
        self.recent_events.retain(|e| e.morale_impact.abs() > 0.01);

        // Update unrest
        self.update_unrest(settlement);
    }

    fn calculate_food_security(&self, settlement: &Settlement) -> f32 {
        let days_of_food = settlement.economy.stockpile_days
            .get(&ResourceId::Food)
            .copied()
            .unwrap_or(0.0);

        // 30 days of food = full security
        (days_of_food / 30.0).min(1.0)
    }

    fn calculate_safety(&self, settlement: &Settlement) -> f32 {
        // Based on garrison, walls, recent attacks
        let garrison_ratio = settlement.garrison_strength / settlement.population as f32;
        let wall_bonus = if settlement.has_walls { 0.3 } else { 0.0 };
        let recent_attacks = settlement.recent_attack_count;

        let base_safety = 0.5 + garrison_ratio * 2.0 + wall_bonus;
        let attack_penalty = recent_attacks as f32 * 0.15;

        (base_safety - attack_penalty).clamp(0.0, 1.0)
    }

    fn calculate_prosperity(&self, settlement: &Settlement) -> f32 {
        // Based on trade, employment, wealth
        let employment = 1.0 - (settlement.economy.unemployment as f32
            / settlement.population as f32);
        let trade_activity = settlement.trade_volume_ratio;

        (employment * 0.6 + trade_activity * 0.4).clamp(0.0, 1.0)
    }

    fn update_unrest(&mut self, settlement: &Settlement) {
        self.unrest_causes.clear();

        // Check hunger
        if self.food_security < 0.3 {
            self.unrest_causes.push(UnrestCause {
                cause_type: UnrestType::Hunger,
                severity: 1.0 - self.food_security / 0.3,
                affected_population: 1.0 - self.food_security,
            });
        }

        // Check taxation (placeholder)
        // Check oppression (placeholder)

        // Calculate unrest level
        let risk = self.unrest_risk();
        let unrest_change = if risk > self.unrest_level {
            (risk - self.unrest_level) * 0.1
        } else {
            (risk - self.unrest_level) * 0.05
        };

        self.unrest_level = (self.unrest_level + unrest_change).clamp(0.0, 1.0);
    }

    /// Add a morale event
    pub fn add_event(&mut self, event_type: SettlementEventType, current_time: SimTime) {
        let (impact, decay) = match event_type {
            SettlementEventType::Festival => (0.15, 0.1),
            SettlementEventType::Victory => (0.2, 0.05),
            SettlementEventType::GoodHarvest => (0.1, 0.03),
            SettlementEventType::Raid => (-0.3, 0.05),
            SettlementEventType::Famine => (-0.4, 0.02),
            SettlementEventType::Plague => (-0.5, 0.03),
            SettlementEventType::TaxIncrease => (-0.15, 0.02),
            SettlementEventType::Defeat => (-0.25, 0.05),
            _ => (0.0, 0.1),
        };

        self.recent_events.push(SettlementEvent {
            event_type,
            timestamp: current_time,
            morale_impact: impact,
            decay_rate: decay,
        });
    }
}
```

---

## Morale in Combat

### Combat Morale Flow

```rust
/// Combat morale manager
#[derive(Debug)]
pub struct CombatMoraleManager {
    pub battle_id: BattleId,
    pub unit_morale: HashMap<UnitId, UnitMorale>,
    pub entity_morale: HashMap<EntityId, MoraleState>,
    pub social_contexts: HashMap<EntityId, SocialPressure>,
}

impl CombatMoraleManager {
    /// Update morale for a combat tick
    pub fn tick(&mut self, battle: &Battle, dt: f32) {
        // Update social contexts
        self.update_social_contexts(battle);

        // Process morale events from combat
        for event in &battle.recent_events {
            self.process_combat_event(event);
        }

        // Check for breaks and routs
        self.check_morale_failures(battle);

        // Decay over time
        for (_, morale) in &mut self.entity_morale {
            morale.decay(dt / 3600.0); // dt is in seconds, decay expects hours
        }
    }

    fn update_social_contexts(&mut self, battle: &Battle) {
        for entity_id in battle.all_combatants() {
            let unit = battle.unit_of(entity_id);
            let position = battle.position_of(entity_id);

            // Count nearby allies
            let nearby_allies: Vec<EntityId> = battle.allies_near(entity_id, 20.0);
            let allies_fighting = nearby_allies.iter()
                .filter(|id| battle.is_fighting(**id))
                .count() as u32;
            let allies_fleeing = nearby_allies.iter()
                .filter(|id| battle.is_fleeing(**id))
                .count() as u32;

            // Determine group stance
            let group_stance = if allies_fleeing > allies_fighting {
                GroupStance::Retreating
            } else if battle.is_unit_attacking(unit) {
                GroupStance::Aggressive
            } else {
                GroupStance::Defensive
            };

            // Calculate pressures
            let peer_pressure = (nearby_allies.len() as f32 / 10.0).min(1.0);
            let authority_pressure = if battle.leader_visible(entity_id, unit) {
                0.8
            } else {
                0.3
            };

            self.social_contexts.insert(entity_id, SocialPressure {
                group_action: battle.unit_current_action(unit),
                group_movement: battle.unit_movement_direction(unit),
                group_stance,
                peer_pressure,
                authority_pressure,
                crowd_pressure: peer_pressure,
                allies_fighting,
                allies_fleeing,
                allies_total: nearby_allies.len() as u32,
            });
        }
    }

    fn process_combat_event(&mut self, event: &CombatEvent) {
        match event {
            CombatEvent::Death { entity, killer, witnesses } => {
                // Killer gains confidence
                if let Some(morale) = self.entity_morale.get_mut(killer) {
                    morale.process_event(
                        MoraleEvent::EnemyDeath,
                        &Personality::default(),
                    );
                }

                // Witnesses may be affected
                for witness in witnesses {
                    if let Some(morale) = self.entity_morale.get_mut(witness) {
                        let was_ally = true; // Determine from battle
                        if was_ally {
                            morale.process_event(
                                MoraleEvent::AllyDeath {
                                    was_leader: false,
                                    was_close: false,
                                },
                                &Personality::default(),
                            );
                        } else {
                            morale.process_event(
                                MoraleEvent::EnemyDeath,
                                &Personality::default(),
                            );
                        }
                    }
                }
            }

            CombatEvent::Wound { entity, severity, .. } => {
                if let Some(morale) = self.entity_morale.get_mut(entity) {
                    morale.process_event(
                        MoraleEvent::PersonalWound { severity: *severity },
                        &Personality::default(),
                    );
                }
            }

            CombatEvent::NearMiss { target } => {
                if let Some(morale) = self.entity_morale.get_mut(target) {
                    morale.process_event(
                        MoraleEvent::NearMiss,
                        &Personality::default(),
                    );
                }
            }

            _ => {}
        }
    }

    fn check_morale_failures(&mut self, battle: &Battle) {
        // Check individual panic/rout
        for (entity_id, morale) in &self.entity_morale {
            if morale.check_panic() {
                // Entity panics
                // Trigger flee behavior
            }
            if morale.check_rout() {
                // Entity routs
                // Trigger full retreat
            }
        }

        // Check unit breaks
        for (unit_id, unit_morale) in &self.unit_morale {
            let avg_individual = self.average_individual_morale(*unit_id);
            let state = unit_morale.check_unit_break(avg_individual);

            match state {
                UnitState::Routed => {
                    // Whole unit routs
                }
                UnitState::Broken => {
                    // Unit loses cohesion
                }
                UnitState::Wavering => {
                    // Unit may break soon
                }
                _ => {}
            }
        }
    }

    fn average_individual_morale(&self, unit_id: UnitId) -> f32 {
        let unit_morale = &self.unit_morale[&unit_id];
        let morale_sum: f32 = unit_morale.members.iter()
            .filter_map(|id| self.entity_morale.get(id))
            .map(|m| m.effective_morale())
            .sum();

        morale_sum / unit_morale.members.len() as f32
    }
}
```

---

## Morale Recovery

### Recovery Conditions

```rust
/// Morale recovery over time
pub fn process_morale_recovery(
    morale: &mut MoraleState,
    conditions: &RecoveryConditions,
    dt_hours: f32,
) {
    // Base recovery rate
    let mut recovery_rate = 0.05 * dt_hours;

    // Conditions affect recovery
    if conditions.is_resting {
        recovery_rate *= 2.0;
    }

    if conditions.is_fed {
        recovery_rate *= 1.5;
    }

    if conditions.is_safe {
        recovery_rate *= 1.5;
    }

    if conditions.with_comrades {
        recovery_rate *= 1.3;
    }

    if conditions.received_medical_care {
        recovery_rate *= 1.2;
    }

    // Apply recovery
    morale.resolve += recovery_rate;
    morale.confidence += recovery_rate * 0.5;
    morale.combat_fatigue -= recovery_rate * 2.0;

    // Fear recovery (slower)
    morale.fear -= recovery_rate * 0.3;

    // Clamp values
    morale.resolve = morale.resolve.clamp(0.0, 1.0);
    morale.confidence = morale.confidence.clamp(0.0, 1.0);
    morale.combat_fatigue = morale.combat_fatigue.clamp(0.0, 1.0);
    morale.fear = morale.fear.clamp(0.0, 1.0);
}

#[derive(Debug)]
pub struct RecoveryConditions {
    pub is_resting: bool,
    pub is_fed: bool,
    pub is_safe: bool,
    pub with_comrades: bool,
    pub received_medical_care: bool,
}
```

---

## Related Specifications

| Document | Relationship |
|----------|--------------|
| [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md) | Entity needs and state |
| [05-BATTLE-SYSTEM-SPEC](05-BATTLE-SYSTEM-SPEC.md) | Combat system |
| [06-BALANCE-COMPOSITION-SPEC](06-BALANCE-COMPOSITION-SPEC.md) | Emergent behavior |
| [16-RESOURCE-ECONOMY-SPEC](16-RESOURCE-ECONOMY-SPEC.md) | Supply effects |
| [17-SOCIAL-MEMORY-SPEC](17-SOCIAL-MEMORY-SPEC.md) | Memory and relationships |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification created |
