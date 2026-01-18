# 05-BATTLE-SYSTEM-SPEC
> Tactical combat layer with physics-based resolution and order delays

## Overview

The battle layer provides tactical real-time combat where physics determines outcomes, orders take time to reach units, and morale breaks before total casualties. This layer activates when forces engage on the campaign map or when the stronghold is attacked.

---

## Design Principles

### From Design Pillars

1. **Physics-Based Combat**: No percentage modifiers. Damage emerges from `force = mass × velocity`.
2. **Order Delays**: Commands travel via courier. Distance = delay.
3. **Emergent Balance**: Material differences matter because physics, not stat bonuses.
4. **Bottom-Up Emergence**: Units rout because morale breaks, not because `if losing then flee`.

### Target Experience

- Combat outcomes feel uncertain and physical
- Player feels tension from order delays
- Terrain and positioning provide tactical depth
- Battles end when morale breaks, not when everyone dies

---

## Battle Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                        BATTLE INITIATION                             │
│  Campaign: Armies meet on hex                                        │
│  Stronghold: Attackers reach settlement                              │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        BATTLE MAP SETUP                              │
│  ├── Generate terrain from campaign hex                              │
│  ├── Deploy forces at edges based on approach                        │
│  ├── Initialize courier system                                       │
│  └── Set initial morale from army state                              │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        BATTLE TICK LOOP                              │
│  ├── Deliver arrived orders                                          │
│  ├── Execute entity actions (movement, combat)                       │
│  ├── Resolve attacks (physics-based)                                 │
│  ├── Update morale                                                   │
│  ├── Check for rout/victory conditions                               │
│  └── Dispatch new courier orders                                     │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        BATTLE RESOLUTION                             │
│  ├── Tally casualties and survivors                                  │
│  ├── Update campaign state                                           │
│  ├── Apply wound effects to survivors                                │
│  └── Return to campaign or stronghold layer                          │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Battle Map

### Structure

```rust
pub struct BattleMap {
    pub width: u32,
    pub height: u32,
    pub terrain: Grid<TerrainType>,
    pub elevation: Grid<f32>,
    pub cover: Grid<CoverType>,
}

pub enum TerrainType {
    Open,       // No modifiers
    Forest,     // Defense bonus, visibility penalty
    Hill,       // Defense bonus, visibility bonus
    Marsh,      // Movement penalty, defense penalty
    River,      // Impassable except at fords
    Building,   // Defense bonus, blocks movement
    Road,       // Movement bonus
}

pub enum CoverType {
    None,
    Light,      // Bushes, low walls
    Heavy,      // Trees, stone walls
    Full,       // Inside buildings
}
```

### Terrain Effects

Terrain affects combat through physical properties, not percentage modifiers:

```rust
impl TerrainType {
    /// How much this terrain slows movement (multiplier on base speed)
    pub fn movement_cost(&self) -> f32 {
        match self {
            Self::Open => 1.0,
            Self::Forest => 1.5,   // Navigating obstacles
            Self::Hill => 1.3,    // Uphill effort
            Self::Marsh => 2.0,   // Sinking, pulling free
            Self::River => 999.0, // Impassable
            Self::Building => 999.0,
            Self::Road => 0.8,    // Faster on roads
        }
    }

    /// Bonus to effective armor from terrain (additive to armor value)
    pub fn cover_bonus(&self) -> f32 {
        match self {
            Self::Open => 0.0,
            Self::Forest => 5.0,   // Partial concealment
            Self::Hill => 3.0,     // Elevation advantage
            Self::Marsh => -2.0,   // Hard to dodge
            Self::River => 0.0,
            Self::Building => 10.0, // Solid cover
            Self::Road => 0.0,
        }
    }

    /// Visibility range multiplier
    pub fn visibility(&self) -> f32 {
        match self {
            Self::Open => 1.0,
            Self::Forest => 0.5,   // Obscured
            Self::Hill => 1.25,    // Better vantage
            Self::Marsh => 0.8,
            Self::River => 1.0,
            Self::Building => 0.3, // Very limited
            Self::Road => 1.0,
        }
    }
}
```

### Elevation

Elevation provides advantages through physics:

```rust
pub fn elevation_advantage(attacker_elevation: f32, defender_elevation: f32) -> ElevationEffect {
    let diff = attacker_elevation - defender_elevation;

    ElevationEffect {
        // Higher ground = gravity assists attack
        attack_force_modifier: 1.0 + (diff * 0.05).clamp(-0.2, 0.3),

        // Lower ground = harder to hit vital areas
        hit_location_shift: if diff > 0.0 { -1 } else { 1 }, // Toward head/torso or legs

        // Ranged weapons benefit more from height
        range_modifier: 1.0 + (diff * 0.1).max(0.0),
    }
}
```

---

## Order System (Couriers)

### Core Concept

Player orders don't take effect instantly. A courier must carry the order to the unit.

```rust
pub struct Order {
    pub id: OrderId,
    pub order_type: OrderType,
    pub target: Option<OrderTarget>,
    pub formation: Option<FormationType>,
    pub issued_tick: Tick,
}

pub enum OrderType {
    Move(Vec2),
    Attack(EntityId),
    Defend,
    Retreat,
    FormUp(FormationType),
    Hold,
    Charge,
}

pub struct Courier {
    pub order: Order,
    pub from: Vec2,           // Commander position
    pub to: UnitId,           // Target unit
    pub position: Vec2,       // Current position
    pub speed: f32,           // Units per tick
    pub interception_risk: f32,
}
```

### Courier System

```rust
pub struct CourierSystem {
    pub in_transit: Vec<Courier>,
    pub delivered: Vec<(UnitId, Order, Tick)>,
}

impl CourierSystem {
    pub fn dispatch(&mut self, commander_pos: Vec2, unit: UnitId, order: Order, unit_pos: Vec2) {
        self.in_transit.push(Courier {
            order,
            from: commander_pos,
            to: unit,
            position: commander_pos,
            speed: 3.0, // Courier base speed
            interception_risk: 0.0,
        });
    }

    pub fn tick(&mut self, world: &World, battle_map: &BattleMap) {
        for courier in &mut self.in_transit {
            let target_pos = world.get_unit_center(courier.to);
            let direction = (target_pos - courier.position).normalize();

            // Terrain affects courier speed
            let terrain = battle_map.terrain.get(courier.position);
            let speed = courier.speed / terrain.movement_cost();

            courier.position += direction * speed;

            // Check for interception
            courier.interception_risk = calculate_interception_risk(
                courier.position,
                world,
                &battle_map,
            );
        }

        // Deliver arrived couriers
        let mut delivered = Vec::new();
        self.in_transit.retain(|c| {
            let target_pos = world.get_unit_center(c.to);
            if c.position.distance(target_pos) < 2.0 {
                delivered.push((c.to, c.order.clone(), world.current_tick));
                false
            } else {
                true
            }
        });

        self.delivered.extend(delivered);
    }
}

fn calculate_interception_risk(pos: Vec2, world: &World, map: &BattleMap) -> f32 {
    // Risk increases near enemy units
    let nearby_enemies = world.spatial_index
        .query_radius(pos, 20.0)
        .filter(|&id| world.is_enemy(id));

    nearby_enemies.count() as f32 * 0.1
}
```

### Order Delay Effects

| Distance | Approximate Delay | Gameplay Impact |
|----------|-------------------|-----------------|
| Close (< 50) | ~17 ticks (1 sec) | Near-instant |
| Medium (50-150) | ~50 ticks (3 sec) | Noticeable lag |
| Far (150-300) | ~100 ticks (6 sec) | Must plan ahead |
| Very Far (300+) | ~150+ ticks (10+ sec) | Strategic decisions only |

**Design Intent**: Players must think ahead. Reactive micro-management is limited by order delays.

---

## Combat Resolution

### Attack Flow

```rust
pub fn resolve_attack(
    attacker: &CombatStats,
    defender: &CombatStats,
    weapon: &Weapon,
    armor: Option<&Armor>,
    terrain: &TerrainType,
    elevation_diff: f32,
) -> CombatResult {
    // 1. Calculate swing force
    let base_force = attacker.strength * weapon.weight;
    let fatigue_modifier = 1.0 - (attacker.fatigue * 0.4);
    let elevation_modifier = 1.0 + (elevation_diff * 0.05).clamp(-0.2, 0.3);
    let impact = base_force * fatigue_modifier * elevation_modifier;

    // 2. Determine hit location
    let body_part = roll_hit_location(&weapon.weapon_type, elevation_diff);

    // 3. Calculate armor protection at that location
    let armor_value = armor
        .filter(|a| a.covers(body_part))
        .map(|a| a.protection() + terrain.cover_bonus())
        .unwrap_or(terrain.cover_bonus());

    // 4. Determine penetration (probabilistic)
    let penetration_prob = penetration_probability(impact, armor_value);

    if rand::random::<f32>() < penetration_prob {
        // Penetrated - calculate wound
        let wound_severity = calculate_wound_severity(impact, armor_value, weapon.damage_type);

        CombatResult::Wound {
            body_part,
            wound_type: weapon.damage_type.into(),
            severity: wound_severity,
            penetrated: true,
        }
    } else {
        // Blocked - possible blunt trauma
        let blunt_transfer = calculate_blunt_transfer(impact, armor_value);

        if blunt_transfer > 0.1 {
            CombatResult::Wound {
                body_part,
                wound_type: WoundType::Blunt,
                severity: blunt_transfer,
                penetrated: false,
            }
        } else {
            CombatResult::Blocked
        }
    }
}

/// Sigmoid probability curve for armor penetration
fn penetration_probability(impact: f32, armor: f32) -> f32 {
    if armor <= 0.0 {
        return 1.0;
    }

    let ratio = impact / armor;
    // Steep sigmoid: ~0 when impact << armor, ~1 when impact >> armor
    // Crossover at ratio = 1.0
    1.0 / (1.0 + (-10.0 * (ratio - 1.0)).exp())
}
```

### Penetration Probability Curve

```
Probability
    1.0 ─────────────────────────────────────●●●●●
        │                               ●●●●
    0.8 │                            ●●●
        │                          ●●
    0.6 │                        ●●
        │                       ●
    0.5 ─────────────────────●─────────────────────  (crossover)
        │                   ●
    0.4 │                  ●
        │                ●●
    0.2 │              ●●
        │           ●●●
    0.0 ●●●●●●●●●●●●───────────────────────────────
        0.0       0.5       1.0       1.5       2.0
                      Impact/Armor Ratio
```

**Key Points**:
- Ratio < 0.5: Nearly impossible to penetrate
- Ratio = 1.0: 50% chance
- Ratio > 1.5: Nearly guaranteed penetration
- There's always uncertainty

### Wound Severity

```rust
fn calculate_wound_severity(impact: f32, armor: f32, damage_type: DamageType) -> f32 {
    let penetration_margin = (impact - armor).max(0.0);

    match damage_type {
        DamageType::Cut => {
            // Cuts: severity scales with margin, caps at 1.0
            (penetration_margin / 50.0).min(1.0)
        }
        DamageType::Pierce => {
            // Pierce: deep wounds even with small margin
            (penetration_margin / 30.0).min(1.0)
        }
        DamageType::Blunt => {
            // Blunt: wide impact, less severe per point
            (penetration_margin / 70.0).min(0.8)
        }
    }
}

fn calculate_blunt_transfer(impact: f32, armor: f32) -> f32 {
    // Even when armor blocks, some force transfers
    // Better armor disperses force more effectively
    let blocked_force = impact.min(armor);
    let transfer_rate = 0.3 * (1.0 - (armor / 100.0).min(0.5));
    blocked_force * transfer_rate / 100.0
}
```

**Full Specification**: [14-PHYSICS-COMBAT-SPEC](14-PHYSICS-COMBAT-SPEC.md)

---

## Morale System

### Morale Components

```rust
pub struct BattleMorale {
    pub value: f32,                    // 0.0-1.0, below 0.2 = rout
    pub modifiers: Vec<MoraleModifier>,
    pub rout_threshold: f32,           // Default 0.2
}

pub struct MoraleModifier {
    pub source: MoraleSource,
    pub amount: f32,
    pub duration: Option<u32>,
}

pub enum MoraleSource {
    // Positive
    LeaderNearby,
    WinningBattle,
    FreshReinforcements,
    HoldingGround,
    EnemyRouting,

    // Negative
    LeaderFallen,
    LosingBattle,
    HeavyCasualties,
    Flanked,
    Surrounded,
    AllyRouting,
    RecentWound,
}
```

### Morale Calculation

```rust
impl BattleMorale {
    pub fn update(&mut self, battle_state: &BattleState, entity: &Entity) {
        // Base morale from battle situation
        let casualties_ratio = battle_state.friendly_casualties as f32
            / battle_state.friendly_starting as f32;
        let enemy_casualties_ratio = battle_state.enemy_casualties as f32
            / battle_state.enemy_starting as f32;

        // Losing = morale drain
        if casualties_ratio > enemy_casualties_ratio {
            self.apply_modifier(MoraleSource::LosingBattle, -(casualties_ratio - enemy_casualties_ratio) * 0.1);
        } else {
            self.apply_modifier(MoraleSource::WinningBattle, (enemy_casualties_ratio - casualties_ratio) * 0.05);
        }

        // Heavy casualties
        if casualties_ratio > 0.3 {
            self.apply_modifier(MoraleSource::HeavyCasualties, -0.2);
        }

        // Leader effects
        if battle_state.leader_nearby(entity.id) {
            self.apply_modifier(MoraleSource::LeaderNearby, 0.1);
        }
        if battle_state.leader_fallen {
            self.apply_modifier(MoraleSource::LeaderFallen, -0.3);
        }

        // Tactical situation
        if battle_state.is_flanked(entity.id) {
            self.apply_modifier(MoraleSource::Flanked, -0.15);
        }
        if battle_state.is_surrounded(entity.id) {
            self.apply_modifier(MoraleSource::Surrounded, -0.25);
        }

        // Calculate final value
        self.value = 1.0 + self.modifiers.iter().map(|m| m.amount).sum::<f32>();
        self.value = self.value.clamp(0.0, 1.0);
    }

    pub fn check_rout(&self) -> bool {
        self.value < self.rout_threshold
    }
}
```

### Rout Behavior

When morale drops below threshold:

```rust
pub fn handle_rout(world: &mut World, entity_id: EntityId) {
    // Clear current tasks
    world.clear_tasks(entity_id);

    // Force retreat action
    let retreat_direction = calculate_retreat_direction(world, entity_id);
    world.queue_task(entity_id, Task {
        action: ActionId::Flee,
        target: Some(TaskTarget::Direction(retreat_direction)),
        priority: TaskPriority::Critical,
        // ...
    });

    // Routing units ignore orders until they rally or leave battlefield
    world.set_routing(entity_id, true);
}

fn calculate_retreat_direction(world: &World, entity_id: EntityId) -> Vec2 {
    let pos = world.get_position(entity_id);
    let enemy_center = world.enemy_center_of_mass(entity_id);

    // Run away from enemies
    let away = (pos - enemy_center).normalize();

    // Bias toward friendly rear
    let friendly_rear = world.friendly_spawn_point(entity_id);
    let to_rear = (friendly_rear - pos).normalize();

    (away + to_rear * 0.5).normalize()
}
```

**Full Specification**: [18-SOCIAL-PRESSURE-MORALE-SPEC](18-SOCIAL-PRESSURE-MORALE-SPEC.md)

---

## Formations

### Formation Types

```rust
pub enum FormationType {
    Line,       // Wide front, good for ranged/defense
    Column,     // Deep, good for marching
    Wedge,      // Offensive penetration
    Square,     // All-around defense
    Skirmish,   // Loose, good for ranged
}

pub struct Formation {
    pub formation_type: FormationType,
    pub members: Vec<EntityId>,
    pub center: Vec2,
    pub facing: Vec2,
    pub cohesion: f32,       // 0.0-1.0, affects bonuses
    pub spacing: f32,        // Distance between members
}
```

### Formation Effects

Formations provide bonuses through positioning, not flat modifiers:

```rust
impl Formation {
    pub fn position_for_member(&self, index: usize, total: usize) -> Vec2 {
        match self.formation_type {
            FormationType::Line => {
                let width = (total as f32 - 1.0) * self.spacing;
                let x_offset = (index as f32 / (total - 1).max(1) as f32 - 0.5) * width;
                self.center + self.facing.perpendicular() * x_offset
            }
            FormationType::Wedge => {
                let row = (index as f32).sqrt() as usize;
                let col = index - row * row;
                let row_width = (row as f32 + 1.0) * self.spacing;
                let x_offset = (col as f32 / row.max(1) as f32 - 0.5) * row_width;
                self.center - self.facing * (row as f32 * self.spacing) + self.facing.perpendicular() * x_offset
            }
            // ... other formations
        }
    }

    /// Cohesion bonus based on how well members maintain positions
    pub fn update_cohesion(&mut self, world: &World) {
        let mut total_deviation = 0.0;

        for (i, &member) in self.members.iter().enumerate() {
            let actual_pos = world.get_position(member);
            let ideal_pos = self.position_for_member(i, self.members.len());
            total_deviation += actual_pos.distance(ideal_pos);
        }

        let avg_deviation = total_deviation / self.members.len() as f32;
        self.cohesion = (1.0 - avg_deviation / (self.spacing * 2.0)).clamp(0.0, 1.0);
    }
}
```

### Formation Combat Effects

```rust
pub struct FormationCombatEffect {
    pub front_protection: f32,    // Effective armor bonus from shields
    pub flank_vulnerability: f32, // Penalty when attacked from flank
    pub charge_resistance: f32,   // Resistance to being pushed
}

impl FormationType {
    pub fn combat_effect(&self, cohesion: f32) -> FormationCombatEffect {
        let base = match self {
            Self::Line => FormationCombatEffect {
                front_protection: 5.0,
                flank_vulnerability: 0.3,
                charge_resistance: 0.6,
            },
            Self::Square => FormationCombatEffect {
                front_protection: 3.0,
                flank_vulnerability: 0.0,
                charge_resistance: 0.8,
            },
            Self::Wedge => FormationCombatEffect {
                front_protection: 2.0,
                flank_vulnerability: 0.4,
                charge_resistance: 0.3, // Moves with charge
            },
            Self::Skirmish => FormationCombatEffect {
                front_protection: 0.0,
                flank_vulnerability: 0.1,
                charge_resistance: 0.1, // Disperses
            },
            Self::Column => FormationCombatEffect {
                front_protection: 1.0,
                flank_vulnerability: 0.5,
                charge_resistance: 0.4,
            },
        };

        // Cohesion scales effectiveness
        FormationCombatEffect {
            front_protection: base.front_protection * cohesion,
            flank_vulnerability: base.flank_vulnerability * (2.0 - cohesion),
            charge_resistance: base.charge_resistance * cohesion,
        }
    }
}
```

---

## Battle Tick

### Tick Structure

```rust
pub fn tick_battle(world: &mut World, battle: &mut BattleState) {
    // 1. Deliver orders that have arrived
    let delivered = battle.courier_system.take_delivered();
    for (unit_id, order, _tick) in delivered {
        execute_order(world, unit_id, order);
    }

    // 2. Update courier positions
    battle.courier_system.tick(world, &battle.map);

    // 3. Entity AI for unordered entities
    for entity_id in battle.participants.iter() {
        if !battle.has_pending_order(entity_id) && !world.is_routing(entity_id) {
            let action = battle_ai_select_action(world, battle, entity_id);
            world.queue_task(entity_id, action);
        }
    }

    // 4. Execute entity tasks (movement, attacks)
    for entity_id in battle.participants.iter() {
        execute_battle_task(world, battle, entity_id);
    }

    // 5. Resolve queued attacks
    let attacks = battle.take_queued_attacks();
    for attack in attacks {
        let result = resolve_attack(
            &world.get_combat_stats(attack.attacker),
            &world.get_combat_stats(attack.defender),
            &world.get_weapon(attack.attacker),
            world.get_armor(attack.defender).as_ref(),
            &battle.map.terrain.get(attack.position),
            world.get_elevation(attack.attacker) - world.get_elevation(attack.defender),
        );

        apply_combat_result(world, attack.defender, result);
    }

    // 6. Update morale
    update_battle_morale(world, battle);

    // 7. Check victory/rout conditions
    if let Some(outcome) = check_battle_outcome(world, battle) {
        battle.outcome = Some(outcome);
    }
}
```

---

## Victory Conditions

```rust
pub enum BattleOutcome {
    Victory { winner: Faction, survivors: Vec<EntityId> },
    Rout { routing: Faction, pursuing: Faction },
    Mutual { status: String },
    Withdrawal { withdrawing: Faction },
}

fn check_battle_outcome(world: &World, battle: &BattleState) -> Option<BattleOutcome> {
    let friendly_active = battle.friendly_active_count(world);
    let enemy_active = battle.enemy_active_count(world);
    let friendly_routing = battle.friendly_routing_count(world);
    let enemy_routing = battle.enemy_routing_count(world);

    // Total rout
    if friendly_routing > friendly_active * 2 {
        return Some(BattleOutcome::Rout {
            routing: Faction::Player,
            pursuing: Faction::Enemy,
        });
    }
    if enemy_routing > enemy_active * 2 {
        return Some(BattleOutcome::Rout {
            routing: Faction::Enemy,
            pursuing: Faction::Player,
        });
    }

    // Annihilation
    if friendly_active == 0 {
        return Some(BattleOutcome::Victory {
            winner: Faction::Enemy,
            survivors: battle.enemy_survivors(world),
        });
    }
    if enemy_active == 0 {
        return Some(BattleOutcome::Victory {
            winner: Faction::Player,
            survivors: battle.friendly_survivors(world),
        });
    }

    None
}
```

---

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| BattleMap | Partial | Terrain types defined, elevation pending |
| Courier System | Not started | Core architecture defined |
| Combat Resolution | Not started | Design complete |
| Morale | Not started | Design complete |
| Formations | Not started | Basic types defined |
| Battle Tick | Not started | Structure defined |

---

## Related Specifications

| Document | Relationship |
|----------|--------------|
| [07-CAMPAIGN-MAP-SPEC](07-CAMPAIGN-MAP-SPEC.md) | Battle initiation |
| [12-BATTLE-PLANNING-TERRAIN-SPEC](12-BATTLE-PLANNING-TERRAIN-SPEC.md) | Terrain details |
| [14-PHYSICS-COMBAT-SPEC](14-PHYSICS-COMBAT-SPEC.md) | Damage calculation |
| [18-SOCIAL-PRESSURE-MORALE-SPEC](18-SOCIAL-PRESSURE-MORALE-SPEC.md) | Morale system |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification created |
