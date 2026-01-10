# Arc Citadel MVP Analysis: City Builder & Tactical Combat

> Analysis Date: 2026-01-02
> Goal: Identify what's needed to reach MVP with a city layer resembling a city builder and a battle layer with realistic physics, delays, materials, and tactics.

## Executive Summary

Arc Citadel has **strong foundational systems** (90% ready for city layer, 50% for battle layer) but **both target layers are essentially stub code**. The existing perception→thought→action loop is production-ready and provides excellent emergent behavior. The gaps are primarily in:

1. **City Layer**: No buildings, construction, infrastructure, production chains, or economy
2. **Battle Layer**: No combat resolution, weapons, armor, formations, or order delays

This document provides a prioritized roadmap to MVP.

---

## Current State Assessment

### What's Production-Ready

| System | Status | Notes |
|--------|--------|-------|
| ECS World | 98/100 | SoA pattern, entity registry, species dispatch |
| Simulation Tick | 95/100 | 330 tests, parallel execution, all non-combat actions work |
| Spatial Queries | 95/100 | O(1) neighbor lookups, radius queries |
| Needs/Motivation | 98/100 | 5 universal needs, decay/satisfaction tuned |
| Social Memory | 90/100 | Dispositions, expectations, violation detection |
| Task Queue | 95/100 | Priority ordering, progress tracking |
| Thought Buffer | 90/100 | Decay, eviction, memory conversion |
| Perception | 85/100 | Distance-based, disposition-aware |
| Action Catalog | 80/100 | 18 actions defined, 14 fully implemented |
| Astronomical System | 100/100 | Seasons, dual moons, eclipses, founding modifiers |
| Aggregate Simulation | 85/100 | Polities, regions, expansion, population growth |

### What's Missing

| System | Status | Gap |
|--------|--------|-----|
| Buildings/Construction | 0/100 | No building types, no construction queue |
| Infrastructure | 0/100 | No roads, walls, workshops, farms |
| Production Chains | 0/100 | No crafting recipes, no resource conversion |
| Economy/Trade | 0/100 | No currency, markets, trade routes |
| Combat Resolution | 0/100 | Combat actions exist but do nothing |
| Weapons/Armor | 0/100 | Data structures not implemented |
| Battle Formations | 0/100 | No squad system, no formation bonuses |
| Order Delays | 0/100 | No courier system, no fog of war |

---

## Part 1: City Layer MVP

### Target Experience

A player should be able to:
- Found a settlement and watch it grow
- Designate buildings and see entities construct them
- Manage resource production and consumption
- Feel the pressure of supply/demand and population needs
- Experience emergent crises (famine, overcrowding, raids)

### Required Systems (Priority Order)

#### Phase C1: Buildings & Construction (Foundation)

**New Module:** `src/city/`

```
src/city/
├── mod.rs
├── building.rs      # Building types, requirements, effects
├── construction.rs  # Construction queue, progress, workforce
├── zone.rs          # Zoning system (residential, industrial, etc.)
└── infrastructure.rs # Roads, walls, utilities
```

**Building System** (building.rs)
| Component | Description | Effort |
|-----------|-------------|--------|
| `BuildingType` enum | House, Farm, Workshop, Granary, Barracks, Wall, Gate, Market | Medium |
| `Building` struct | Position, type, condition, workers, production | Medium |
| `BuildingEffect` | What the building does (housing, storage, production) | Medium |
| `BuildingRequirements` | Resources, terrain, adjacency rules | Low |

**Construction System** (construction.rs)
| Component | Description | Effort |
|-----------|-------------|--------|
| `ConstructionSite` | Blueprint, progress (0.0-1.0), assigned workers | Medium |
| `ConstructionQueue` | Per-settlement queue of planned buildings | Low |
| Worker assignment | Entities with Build task contribute progress | Medium |
| Resource consumption | Deduct materials as construction progresses | Medium |
| Completion callback | Spawn Building entity, assign effects | Medium |

**Estimated Effort:** 2 weeks foundation + 1 week integration

#### Phase C2: Resource Management

**Extend existing systems:**

| System | Changes | Effort |
|--------|---------|--------|
| `ResourceZone` | Add type variants (Stone, Wood, Iron, etc.) | Low |
| `Inventory` component | Per-entity storage of materials | Medium |
| `Stockpile` building | Settlement-level storage with capacity | Medium |
| Hauling task | Move resources from zones to stockpiles | Medium |
| Consumption | Buildings consume resources for production | Medium |

**Resource Types:**
```rust
pub enum ResourceType {
    Food,       // From farms, fishing, hunting
    Wood,       // From forest zones
    Stone,      // From quarries
    Iron,       // From mines
    Gold,       // From mines (wealth/trade)
    Cloth,      // From workshops (wool→cloth)
    Tools,      // From smithy (iron→tools)
}
```

**Estimated Effort:** 2 weeks

#### Phase C3: Production Chains

**Production System:**
```rust
pub struct Recipe {
    pub inputs: Vec<(ResourceType, u32)>,
    pub output: (ResourceType, u32),
    pub duration_ticks: u32,
    pub skill_requirement: Option<SkillType>,
}

pub struct ProductionBuilding {
    pub building_id: BuildingId,
    pub recipes: Vec<Recipe>,
    pub active_recipe: Option<RecipeId>,
    pub progress: f32,
    pub workers: Vec<EntityId>,
}
```

**Example Chains:**
- Grain (farm) → Food
- Sheep (pasture) → Wool → Cloth (workshop)
- Iron ore (mine) → Iron bars (smelter) → Tools (smithy)
- Wood (forest) → Planks (sawmill) → Buildings

**Estimated Effort:** 2 weeks

#### Phase C4: Population & Housing

| System | Description | Effort |
|--------|-------------|--------|
| Housing capacity | Each house supports N entities | Low |
| Homelessness | Entities without housing have penalties | Low |
| Population growth | New entities spawn when food/housing available | Medium |
| Migration | Entities leave if needs unmet | Medium |

**Estimated Effort:** 1 week

#### Phase C5: Economy (Stretch)

| System | Description | Effort |
|--------|-------------|--------|
| Currency | Gold as universal exchange medium | Low |
| Markets | Buy/sell resources at prices | High |
| Trade routes | Connect settlements for resource flow | High |
| Wages | Pay workers, affect morale | Medium |

**Estimated Effort:** 3+ weeks (defer if needed)

### City Layer MVP Scope

**Must Have:**
- [ ] 5-8 building types (House, Farm, Granary, Workshop, Wall, Gate)
- [ ] Construction system with worker assignment
- [ ] 4-5 resource types (Food, Wood, Stone, Iron)
- [ ] Basic production (Farm→Food, Forest→Wood)
- [ ] Housing capacity affecting population
- [ ] Stockpile storage

**Nice to Have:**
- [ ] Production chains (multi-step crafting)
- [ ] Trade between settlements
- [ ] Building upgrades
- [ ] Defensive walls with combat integration

**Out of Scope for MVP:**
- Complex economy with markets
- Multiple settlement management
- Political/diplomatic city relations

---

## Part 2: Battle Layer MVP

### Target Experience

A player should be able to:
- Engage in tactical combat with physics-based damage
- Feel order delays (couriers, not instant control)
- See material differences matter (iron vs steel)
- Use formations and positioning tactically
- Experience fog of war and miscommunication

### Required Systems (Priority Order)

#### Phase B1: Combat Resolution (Foundation)

**Complete existing stubs:** `src/combat/`

**Weapons System** (weapons.rs)
```rust
pub struct Weapon {
    pub id: WeaponId,
    pub name: String,
    pub weapon_type: WeaponType,      // Sword, Axe, Spear, Bow, etc.
    pub material: Material,            // Iron, Steel, Bronze
    pub weight: f32,                   // Affects fatigue, damage
    pub reach: f32,                    // Engagement range
    pub speed: f32,                    // Attack frequency
    pub damage_type: DamageType,       // Cut, Pierce, Blunt
    pub condition: f32,                // 0.0-1.0 degradation
}

pub enum Material {
    Wood,
    Bronze,
    Iron,
    Steel,
    Mithril,
}

impl Material {
    pub fn damage_multiplier(&self) -> f32 {
        match self {
            Self::Wood => 0.5,
            Self::Bronze => 0.8,
            Self::Iron => 1.0,
            Self::Steel => 1.2,
            Self::Mithril => 1.5,
        }
    }

    pub fn durability(&self) -> f32 {
        match self {
            Self::Wood => 0.3,
            Self::Bronze => 0.7,
            Self::Iron => 1.0,
            Self::Steel => 1.3,
            Self::Mithril => 2.0,
        }
    }
}
```

**Armor System** (armor.rs)
```rust
pub struct Armor {
    pub coverage: ArmorCoverage,       // Head, Torso, Arms, Legs
    pub material: Material,
    pub thickness: f32,                // Penetration resistance
    pub condition: f32,                // Degradation
    pub weight: f32,                   // Movement/fatigue penalty
}

impl Armor {
    pub fn protection(&self) -> f32 {
        self.thickness * self.material.damage_multiplier() * self.condition
    }
}
```

**Estimated Effort:** 1 week

#### Phase B2: Damage Calculation

**Combat Resolution** (resolution.rs)
```rust
pub fn resolve_attack(
    attacker: &CombatStats,
    defender: &CombatStats,
    weapon: &Weapon,
    armor: Option<&Armor>,
) -> CombatResult {
    // 1. Calculate raw impact
    let swing_force = attacker.strength * weapon.weight;
    let fatigue_penalty = 1.0 - (attacker.fatigue * 0.4);
    let impact = swing_force * fatigue_penalty * weapon.material.damage_multiplier();

    // 2. Check armor penetration
    let armor_protection = armor.map(|a| a.protection()).unwrap_or(0.0);
    let penetration = (impact - armor_protection).max(0.0);

    // 3. Determine hit location
    let body_part = roll_hit_location(weapon.weapon_type);

    // 4. Calculate wound severity
    let severity = (penetration / 100.0).min(1.0);

    // 5. Create wound if penetrated
    if severity > 0.05 {
        CombatResult::Wound {
            body_part,
            wound_type: weapon.damage_type.into(),
            severity,
        }
    } else {
        CombatResult::Blocked
    }
}
```

**Key Physics Properties:**
| Property | Affects | Formula Contribution |
|----------|---------|---------------------|
| Strength | Impact force | `strength * weapon_weight` |
| Weapon weight | Impact + fatigue | Higher = more damage, more tiring |
| Fatigue | All combat | `1.0 - fatigue * penalty_rate` |
| Armor thickness | Damage reduction | `thickness * material_mult * condition` |
| Weapon reach | Who can attack | Must be in range to strike |
| Weapon speed | Attack frequency | Faster = more attacks per round |

**Estimated Effort:** 2 weeks

#### Phase B3: Wound Effects & Health

**Extend existing body.rs:**
```rust
impl Wound {
    pub fn effects(&self) -> WoundEffects {
        let mut effects = WoundEffects::default();

        match self.body_part {
            BodyPart::Head => {
                effects.consciousness_penalty = self.severity * 0.5;
                effects.perception_penalty = self.severity * 0.3;
            }
            BodyPart::Torso => {
                effects.health_drain = self.severity * 0.02; // per tick
                effects.fatigue_increase = self.severity * 0.1;
            }
            BodyPart::LeftArm | BodyPart::RightArm => {
                effects.attack_penalty = self.severity * 0.4;
            }
            BodyPart::LeftLeg | BodyPart::RightLeg => {
                effects.movement_penalty = self.severity * 0.5;
            }
        }

        if self.infected {
            effects.health_drain += 0.01;
            effects.fever_chance = 0.1;
        }

        effects
    }
}
```

**Estimated Effort:** 1 week

#### Phase B4: Order Delays (Courier System)

**New:** `src/battle/courier.rs`
```rust
pub struct Order {
    pub order_type: OrderType,
    pub target: Option<Vec2>,
    pub formation: Option<FormationType>,
}

pub struct Courier {
    pub order: Order,
    pub from: EntityId,           // Commander
    pub to: UnitId,               // Squad/unit
    pub sent_tick: Tick,
    pub position: Vec2,
    pub speed: f32,               // Units per tick
}

pub struct CourierSystem {
    pub in_transit: Vec<Courier>,
    pub delivered: Vec<(UnitId, Order, Tick)>,
}

impl CourierSystem {
    pub fn send_order(&mut self, commander: EntityId, unit: UnitId, order: Order, world: &World) {
        let commander_pos = world.get_position(commander);
        let unit_pos = world.get_unit_position(unit);

        self.in_transit.push(Courier {
            order,
            from: commander,
            to: unit,
            sent_tick: world.current_tick,
            position: commander_pos,
            speed: 2.0, // Courier speed
        });
    }

    pub fn tick(&mut self, world: &World) {
        for courier in &mut self.in_transit {
            let target_pos = world.get_unit_position(courier.to);
            let direction = (target_pos - courier.position).normalize();
            courier.position += direction * courier.speed;

            // Check if arrived
            if courier.position.distance(target_pos) < 1.0 {
                self.delivered.push((courier.to, courier.order.clone(), world.current_tick));
            }
        }

        // Remove delivered
        self.in_transit.retain(|c| {
            let target = world.get_unit_position(c.to);
            c.position.distance(target) >= 1.0
        });
    }
}
```

**Order Delay Effects:**
| Distance | Approximate Delay | Gameplay Impact |
|----------|-------------------|-----------------|
| Close (< 50 units) | 25 ticks | Near-instant response |
| Medium (50-200) | 100+ ticks | Noticeable lag |
| Far (> 200) | 250+ ticks | Strategic planning required |

**Risk Factors:**
- Courier intercepted (enemy near path)
- Courier killed (combat zone)
- Order outdated (situation changed)

**Estimated Effort:** 2 weeks

#### Phase B5: Formations & Tactics

**New:** `src/battle/formation.rs`
```rust
pub enum FormationType {
    Line,       // Wide front, good for defense
    Column,     // Deep, good for marching
    Wedge,      // Offensive, penetration
    Square,     // All-around defense
    Skirmish,   // Loose, good for ranged
}

pub struct Formation {
    pub formation_type: FormationType,
    pub members: Vec<EntityId>,
    pub center: Vec2,
    pub facing: Vec2,
    pub cohesion: f32,          // 0.0-1.0, affects bonuses
}

impl FormationType {
    pub fn bonuses(&self) -> FormationBonuses {
        match self {
            Self::Line => FormationBonuses {
                defense_front: 0.2,
                defense_flank: -0.1,
                attack: 0.0,
                morale: 0.1,
            },
            Self::Wedge => FormationBonuses {
                defense_front: -0.1,
                defense_flank: -0.2,
                attack: 0.3,
                morale: 0.15,
            },
            Self::Square => FormationBonuses {
                defense_front: 0.15,
                defense_flank: 0.15,
                attack: -0.1,
                morale: 0.2,
            },
            // ...
        }
    }
}
```

**Estimated Effort:** 2 weeks

#### Phase B6: Battle Map & Terrain

**New:** `src/battle/battle_map.rs`
```rust
pub struct BattleMap {
    pub terrain: Grid<TerrainType>,
    pub elevation: Grid<f32>,
    pub cover: Grid<CoverType>,
}

pub enum TerrainType {
    Open,       // No modifiers
    Forest,     // +20% defense, -50% visibility
    Hill,       // +15% defense, +25% visibility
    Marsh,      // -30% movement, -10% defense
    River,      // Impassable except at fords
    Building,   // +30% defense, blocks movement
}

impl TerrainType {
    pub fn movement_cost(&self) -> f32 {
        match self {
            Self::Open => 1.0,
            Self::Forest => 1.5,
            Self::Hill => 1.3,
            Self::Marsh => 2.0,
            Self::River => 999.0,
            Self::Building => 999.0,
        }
    }

    pub fn defense_modifier(&self) -> f32 {
        match self {
            Self::Open => 0.0,
            Self::Forest => 0.2,
            Self::Hill => 0.15,
            Self::Marsh => -0.1,
            Self::River => 0.0,
            Self::Building => 0.3,
        }
    }
}
```

**Estimated Effort:** 1 week

### Battle Layer MVP Scope

**Must Have:**
- [ ] Weapons with material differences (Iron < Steel < Mithril)
- [ ] Armor with penetration mechanics
- [ ] Damage calculation based on physics (strength, weight, fatigue)
- [ ] Wound system with location-specific effects
- [ ] Order delays via courier system
- [ ] Basic terrain effects (hills, forests)

**Nice to Have:**
- [ ] Formations with cohesion bonuses
- [ ] Morale system with rout mechanics
- [ ] Fog of war from limited visibility
- [ ] Intercepted orders

**Out of Scope for MVP:**
- Siege mechanics
- Naval combat
- Magic/special abilities
- Detailed unit experience system

---

## Part 3: Integration & Shared Systems

### Entity Equipment

Both layers need equipment on entities:

```rust
// Add to human archetype (and other species)
pub struct Equipment {
    pub main_hand: Option<WeaponId>,
    pub off_hand: Option<WeaponId>,  // Shield or second weapon
    pub armor: Vec<ArmorId>,          // Can have multiple pieces
    pub inventory: Vec<ItemId>,       // Carried items
}
```

**Required Changes:**
- Extend `HumanArchetype` with `equipment: Vec<Equipment>`
- Add equipment to perception (notice armed entities)
- Add equipment to action selection (can't attack without weapon)

### Resource Flow: City → Battle

```
CITY LAYER                          BATTLE LAYER
-----------                         ------------
Iron Ore (mine)
    ↓
Iron Bars (smelter)
    ↓
Weapons (smithy) ──────────────────→ Entity Equipment
    ↓
Armory (storage) ──────────────────→ Battle Supplies
```

### Morale Integration

Morale affects both layers:
- **City**: Low morale → productivity drops, entities leave
- **Battle**: Low morale → rout, refuse orders

```rust
pub struct Morale {
    pub value: f32,                   // 0.0-1.0
    pub modifiers: Vec<MoraleModifier>,
}

pub struct MoraleModifier {
    pub source: MoraleSource,
    pub amount: f32,
    pub duration: Option<u32>,        // Ticks until expires
}

pub enum MoraleSource {
    // City
    WellFed,
    Housed,
    SafeFromRaids,

    // Battle
    NearbyAllies,
    NearbyEnemies,
    RecentWound,
    LeaderPresent,
    WinningBattle,
    LosingBattle,
}
```

---

## Part 4: Implementation Roadmap

### Phase 1: Combat Foundation (Weeks 1-3)

| Week | Deliverable | Rating |
|------|-------------|--------|
| 1 | Weapons + Armor data structures | 90/100 |
| 2 | Damage calculation + wound application | 85/100 |
| 3 | Combat action execution in tick system | 85/100 |

**Exit Criteria:** Two entities can fight, wounds accumulate, one dies

### Phase 2: City Foundation (Weeks 4-6)

| Week | Deliverable | Rating |
|------|-------------|--------|
| 4 | Building types + construction sites | 85/100 |
| 5 | Construction task + worker assignment | 85/100 |
| 6 | Resource zones + stockpiles | 80/100 |

**Exit Criteria:** Entities can build a house from gathered wood

### Phase 3: Battle Tactics (Weeks 7-9)

| Week | Deliverable | Rating |
|------|-------------|--------|
| 7 | Courier system + order delays | 90/100 |
| 8 | Terrain + positioning | 85/100 |
| 9 | Basic formations | 75/100 |

**Exit Criteria:** Orders take time to reach units, terrain affects combat

### Phase 4: City Production (Weeks 10-11)

| Week | Deliverable | Rating |
|------|-------------|--------|
| 10 | Production buildings + recipes | 80/100 |
| 11 | Housing + population | 85/100 |

**Exit Criteria:** Farm produces food, houses enable population growth

### Phase 5: Integration (Week 12)

| Week | Deliverable | Rating |
|------|-------------|--------|
| 12 | Equipment flow + morale | 75/100 |

**Exit Criteria:** Smithy-produced weapons equip entities for battle

---

## Part 5: Risk Assessment

### Technical Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Combat balance issues | High | Extensive testing, tunable constants |
| Order delay feels bad | Medium | Adjust courier speed, provide feedback |
| Resource starvation spirals | Medium | Add emergency actions, safety nets |
| Performance with many entities | Medium | Already parallel-ready, profile early |

### Scope Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Feature creep in buildings | High | Lock MVP building list (5-8 types) |
| Over-complex combat | High | Start simple, add complexity iteratively |
| UI requirements | Medium | Defer UI to post-MVP, use terminal |

### Recommended Cuts if Behind Schedule

1. **Economy/Trade** - Defer entirely, use simple resource stockpiles
2. **Formations** - Defer, entities fight individually
3. **Production chains** - Simplify to single-step (ore→weapon)
4. **Weather effects** - Defer to post-MVP

---

## Part 6: Success Metrics

### City Layer MVP Complete When:

- [ ] Player can designate 5+ building types
- [ ] Entities construct buildings over time
- [ ] Resources gathered from zones, stored in stockpiles
- [ ] At least one production chain works (farm→food)
- [ ] Population grows when food+housing available
- [ ] Settlement can sustain itself without player intervention

### Battle Layer MVP Complete When:

- [ ] Two groups can engage in combat
- [ ] Damage depends on weapon, armor, and fatigue
- [ ] Different materials produce different outcomes
- [ ] Orders take time proportional to distance
- [ ] Terrain affects defense values
- [ ] Entities can die from accumulated wounds

### Integration Complete When:

- [ ] Weapons forged in city equip entities for battle
- [ ] Battle outcomes affect city population
- [ ] Morale flows between both layers

---

## Appendix A: File Structure for New Systems

```
src/
├── city/                       # NEW MODULE
│   ├── mod.rs
│   ├── building.rs             # BuildingType, Building, BuildingEffect
│   ├── construction.rs         # ConstructionSite, ConstructionQueue
│   ├── zone.rs                 # ZoneType, Zone (residential, industrial)
│   ├── production.rs           # Recipe, ProductionBuilding
│   ├── stockpile.rs            # Stockpile, StorageCapacity
│   └── README.md
│
├── combat/                     # COMPLETE STUBS
│   ├── mod.rs
│   ├── weapons.rs              # Weapon, WeaponType, Material
│   ├── armor.rs                # Armor, ArmorCoverage
│   ├── resolution.rs           # resolve_attack(), CombatResult
│   ├── wounds.rs               # Extend existing, add effects
│   ├── morale.rs               # Morale, MoraleModifier
│   └── README.md
│
├── battle/                     # COMPLETE STUBS
│   ├── mod.rs
│   ├── battle_map.rs           # BattleMap, TerrainType
│   ├── courier.rs              # Courier, CourierSystem, Order
│   ├── formation.rs            # Formation, FormationType
│   ├── planning.rs             # BattlePlan, Objective
│   ├── execution.rs            # BattleExecutor, tick_battle()
│   └── README.md
│
├── entity/
│   └── equipment.rs            # NEW: Equipment component
```

---

## Appendix B: Existing Code to Leverage

### For City Layer

| Existing Code | How to Use |
|---------------|------------|
| `ResourceZone` | Extend with type variants |
| `Task::Build` | Already exists, wire to construction |
| `Task::Gather` | Already works, add inventory |
| `ActionId::Build/Craft/Repair` | Catalog entries exist |
| `FoundingModifiers` | Apply when settlement created |

### For Battle Layer

| Existing Code | How to Use |
|---------------|------------|
| `BodyState` | Add equipment effects |
| `Wound`, `BodyPart` | Already defined, add effects |
| `ActionId::Attack/Defend` | Wire to resolution |
| `SparseHashGrid` | Range queries for combat |
| `Grid<T>` | Battle map terrain |

---

## Conclusion

Arc Citadel has an excellent foundation with mature perception, thoughts, needs, and action systems. The MVP requires building two major new layers:

1. **City Layer** (~6 weeks): Buildings, construction, resources, production
2. **Battle Layer** (~6 weeks): Weapons, armor, damage physics, order delays

The recommended approach is to alternate between layers, validating each works before deep integration:

1. Combat foundation → entities can fight
2. City foundation → entities can build
3. Battle tactics → orders have delays, terrain matters
4. City production → resource chains work
5. Integration → equipment flows from city to battle

Total estimated timeline: **12 weeks to MVP** with the scope defined above.
