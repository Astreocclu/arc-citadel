# 09-GAP-ANALYSIS
> Current implementation state vs. MVP requirements

## Overview

This document analyzes what exists in the Arc Citadel codebase versus what's needed for a playable MVP. It serves as the authoritative source for implementation priorities and identifies critical gaps.

**Analysis Date:** 2026-01-14 (updated from 2026-01-02 analysis)

---

## Executive Summary

Arc Citadel has **strong foundational systems** but **both target layers need substantial work**:

| Layer | Foundation Readiness | Target Layer Readiness |
|-------|---------------------|------------------------|
| City (Stronghold) | 90% | 10% |
| Battle (Tactical Combat) | 50% | 5% |

The existing perception → thought → action loop is **production-ready** and provides excellent emergent behavior. The gaps are in:

1. **City Layer**: No buildings, construction, infrastructure, production chains
2. **Battle Layer**: No combat resolution, weapons, armor, formations

---

## Production-Ready Systems

These systems are complete and can be relied upon for MVP:

| System | Score | Evidence | Specification |
|--------|-------|----------|---------------|
| **ECS World** | 98/100 | SoA pattern, entity registry, species dispatch | [02-IMPLEMENTATION-ARCHITECTURE](02-IMPLEMENTATION-ARCHITECTURE.md) |
| **Simulation Tick** | 95/100 | 330 tests, parallel execution | [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md) |
| **Spatial Queries** | 95/100 | O(1) neighbor lookups via sparse hash | [10-PERFORMANCE-ARCHITECTURE-SPEC](10-PERFORMANCE-ARCHITECTURE-SPEC.md) |
| **Needs/Motivation** | 98/100 | 5 universal needs, decay/satisfaction tuned | [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md) |
| **Social Memory** | 90/100 | Dispositions, expectations, violation detection | [17-SOCIAL-MEMORY-SPEC](17-SOCIAL-MEMORY-SPEC.md) |
| **Task Queue** | 95/100 | Priority ordering, progress tracking | [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md) |
| **Thought Buffer** | 90/100 | Decay, eviction, memory conversion | [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md) |
| **Perception** | 85/100 | Distance-based, disposition-aware | [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md) |
| **Action Catalog** | 80/100 | 18 actions defined, 14 implemented | [11-ACTION-CATALOG](11-ACTION-CATALOG.md) |
| **Astronomical System** | 100/100 | Seasons, moons, eclipses, founding modifiers | [01-GAME-DESIGN-DOCUMENT](01-GAME-DESIGN-DOCUMENT.md) |
| **Aggregate Simulation** | 85/100 | Polities, regions, expansion | [07-CAMPAIGN-MAP-SPEC](07-CAMPAIGN-MAP-SPEC.md) |

### Key Strengths

1. **Entity cognition loop works end-to-end**: Entities perceive, think, and act based on values/needs
2. **SoA architecture is cache-efficient**: Parallel iteration over entity arrays
3. **Social system is surprisingly deep**: Expectation violations create realistic relationship dynamics
4. **Action catalog is extensible**: Adding new actions follows established patterns

---

## Critical Gaps

### City Layer (Stronghold)

| System | Score | Gap Description | Priority |
|--------|-------|-----------------|----------|
| **Buildings** | 0/100 | No building types, blueprints, or placement | **P0** |
| **Construction** | 0/100 | No construction queue or progress tracking | **P0** |
| **Infrastructure** | 0/100 | No roads, walls, workshops | **P1** |
| **Production Chains** | 0/100 | No crafting recipes, resource conversion | **P1** |
| **Economy** | 0/100 | No currency, markets, trade | **P2** |
| **Housing** | 0/100 | No capacity limits, homelessness effects | **P1** |

**What "0/100" means**: These systems don't exist as code. The *design* exists in the genesis document, but there's no implementation.

### Battle Layer (Tactical Combat)

| System | Score | Gap Description | Priority |
|--------|-------|-----------------|----------|
| **Combat Resolution** | 0/100 | Attack actions exist but execute no logic | **P0** |
| **Weapons** | 0/100 | Data structures not implemented | **P0** |
| **Armor** | 0/100 | No penetration/protection mechanics | **P0** |
| **Battle Formations** | 0/100 | No squad system, formation bonuses | **P2** |
| **Order Delays** | 0/100 | No courier system, instant control | **P1** |
| **Terrain Effects** | 0/100 | No defense modifiers, movement costs | **P1** |

### Shared Systems

| System | Score | Gap Description | Priority |
|--------|-------|-----------------|----------|
| **Equipment** | 0/100 | Entities have no weapon/armor slots | **P0** |
| **Morale** | 0/100 | No morale component or effects | **P1** |
| **Resource Inventory** | 20/100 | `ResourceZone` exists, entity inventory missing | **P1** |

---

## Partial Systems

These exist but need extension:

### Action Catalog (80/100)

**Implemented:**
- Movement (Walk, Run)
- Social (Talk, Fight, Flee)
- Survival (Eat, Drink, Sleep, Rest)
- Work (Build, Gather, Craft, Repair)
- Observation (LookAround)

**Not Implemented:**
- Combat execution (Attack exists, does nothing)
- Build execution (Build exists, no buildings to build)
- Trade actions
- Formation commands

### Perception (85/100)

**Implemented:**
- Distance-based awareness
- Disposition filtering
- Visual range calculation

**Missing:**
- Equipment awareness (notice armed entities)
- Building awareness (notice structures)
- Fog of war (battle layer)

### Resource Zones (20/100)

**Implemented:**
- `ResourceZone` struct exists
- Basic zone types (Stone, Wood hints in design)

**Missing:**
- `ResourceType` enum with variants
- Zone depletion mechanics
- Hauling task execution

---

## File Structure: What Exists vs. What's Needed

### Existing Structure

```
src/
├── core/           ✅ Complete (types, errors, astronomy)
├── ecs/            ✅ Complete (world, entity management)
├── spatial/        ✅ Complete (sparse hash, grid)
├── entity/         ⚠️  Partial (needs equipment, morale)
│   ├── needs.rs    ✅
│   ├── thoughts.rs ✅
│   ├── tasks.rs    ✅
│   └── species/    ✅
├── simulation/     ✅ Complete (tick, perception, action_select)
├── actions/        ⚠️  Catalog exists, execution stubs
├── combat/         ❌ Stubs only
├── campaign/       ❌ Stubs only
├── battle/         ❌ Stubs only
├── genetics/       ❌ Stubs only
├── llm/            ✅ Complete (client, parser, context)
└── ui/             ❌ Stubs only
```

### Required New Modules

```
src/
├── city/                   🆕 Entire module needed
│   ├── building.rs         BuildingType, Building
│   ├── construction.rs     ConstructionSite, Queue
│   ├── zone.rs             Zone management
│   ├── production.rs       Recipe, ProductionBuilding
│   └── stockpile.rs        Storage, capacity
│
├── combat/                 🔧 Complete existing stubs
│   ├── weapons.rs          Weapon, WeaponType, Material
│   ├── armor.rs            Armor, ArmorCoverage
│   ├── resolution.rs       resolve_attack(), physics
│   └── morale.rs           Morale, modifiers
│
├── battle/                 🔧 Complete existing stubs
│   ├── battle_map.rs       BattleMap, TerrainType
│   ├── courier.rs          Order delays
│   ├── formation.rs        Formation, bonuses
│   └── execution.rs        tick_battle()
│
├── entity/
│   └── equipment.rs        🆕 Equipment component
```

---

## MVP Definition

### City Layer MVP (Minimum)

- [ ] 5-8 building types (House, Farm, Granary, Workshop, Wall, Gate)
- [ ] Construction system with worker assignment
- [ ] 4-5 resource types (Food, Wood, Stone, Iron)
- [ ] Basic production (Farm → Food, Forest → Wood)
- [ ] Housing capacity affecting population
- [ ] Stockpile storage

### Battle Layer MVP (Minimum)

- [ ] Weapons with material differences (Iron < Steel)
- [ ] Armor with penetration mechanics
- [ ] Damage calculation based on physics
- [ ] Wound system with location-specific effects
- [ ] Order delays via courier system
- [ ] Basic terrain effects (hills, forests)

### Integration MVP (Minimum)

- [ ] Equipment slots on entities
- [ ] Weapons flow from city → battle
- [ ] Battle outcomes affect city population

### Species Priority

| Tier | Species | Status | Notes |
|------|---------|--------|-------|
| **MVP** | Human | Implemented | Primary focus, all features tested here first |
| **MVP** | Elf | Stub | Add after Human systems stable |
| **MVP** | Dwarf | Stub | Add after Human systems stable |
| Post-MVP | Orc, Kobold, Goblin, etc. | Implemented | Monster species, polished later |

**Rule**: All gameplay optimization and testing focuses on MVP species (Human, Elf, Dwarf). Monster species (Orc, Kobold, etc.) have basic TOML rules but are **not** optimization targets until post-MVP.

---

## Recommended Implementation Order

Based on dependencies and risk:

### Phase 1: Combat Foundation (Weeks 1-3)

**Goal**: Two entities can fight, wounds accumulate, one dies

1. `Weapon` and `Armor` data structures
2. `Equipment` component on entities
3. `resolve_attack()` physics implementation
4. Wire attack actions to resolution
5. Wound effects (movement, combat penalties)

**Exit Criteria**:
```rust
// This test should pass
#[test]
fn two_entities_can_fight_to_death() {
    let mut world = setup_combat_test();
    let attacker = spawn_armed_entity(&mut world);
    let defender = spawn_armed_entity(&mut world);

    // Run combat until one dies
    while world.both_alive(attacker, defender) {
        world.tick();
    }

    assert!(world.one_dead(attacker, defender));
}
```

### Phase 2: City Foundation (Weeks 4-6)

**Goal**: Entities can build a house from gathered wood

1. `BuildingType` enum and `Building` struct
2. `ConstructionSite` with progress tracking
3. Wire `Task::Build` to construction system
4. Resource gathering to stockpiles
5. Building completion spawns Building entity

**Exit Criteria**:
```rust
#[test]
fn entity_builds_house_from_wood() {
    let mut world = setup_city_test();
    let builder = spawn_entity(&mut world);
    let wood_zone = spawn_resource_zone(&mut world, ResourceType::Wood);

    // Queue build task
    world.queue_task(builder, Task::Build {
        building_type: BuildingType::House,
        position: Vec2::new(10.0, 10.0),
    });

    // Run until house built
    for _ in 0..1000 {
        world.tick();
    }

    assert!(world.building_exists_at(Vec2::new(10.0, 10.0)));
}
```

### Phase 3: Battle Tactics (Weeks 7-9)

**Goal**: Orders take time, terrain matters

1. `CourierSystem` with order delivery
2. `BattleMap` with terrain types
3. Terrain modifiers on combat
4. Basic formations (stretch)

### Phase 4: City Production (Weeks 10-11)

**Goal**: Farm produces food, houses enable growth

1. `Recipe` and `ProductionBuilding`
2. Production tick integration
3. Housing capacity limits
4. Population growth when conditions met

### Phase 5: Integration (Week 12)

**Goal**: Equipment flows from city to battle

1. Equipment crafted in city workshops
2. Entities equip before battle
3. Morale component shared between layers
4. Battle casualties affect city population

---

## Risk Assessment

### Technical Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Combat balance | High | Tunable constants, extensive testing |
| Order delay UX | Medium | Adjust courier speed, visual feedback |
| Resource spirals | Medium | Emergency actions, AI safety nets |
| Performance | Low | Already parallel-ready, existing spatial index |

### Scope Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Building feature creep | High | Lock MVP list (5-8 types) |
| Complex combat | High | Ship simple, iterate |
| UI requirements | Medium | Defer UI, use terminal/egui basic |

### Recommended Cuts If Behind

1. **Economy/Trade** - Use simple stockpiles
2. **Formations** - Entities fight individually
3. **Production chains** - Single-step only (ore → weapon)
4. **Weather effects** - Post-MVP

---

## Code to Leverage

### For City Layer

| Existing | Usage |
|----------|-------|
| `ResourceZone` | Extend with type variants |
| `Task::Build` | Already exists, wire to construction |
| `Task::Gather` | Works, add inventory deposit |
| `ActionId::Build/Craft/Repair` | Catalog entries ready |

### For Battle Layer

| Existing | Usage |
|----------|-------|
| `BodyState` | Add equipment effects |
| `Wound`, `BodyPart` | Defined, add combat integration |
| `ActionId::Attack/Defend` | Wire to resolution |
| `SparseHashGrid` | Combat range queries |
| `Grid<T>` | Battle map terrain |

---

## Success Metrics

### When City Layer MVP Is Complete

- [ ] Player designates 5+ building types
- [ ] Entities construct buildings over time
- [ ] Resources gathered from zones, stored in stockpiles
- [ ] At least one production chain works
- [ ] Population grows when conditions met
- [ ] Settlement self-sustains without intervention

### When Battle Layer MVP Is Complete

- [ ] Two groups engage in combat
- [ ] Damage depends on weapon, armor, fatigue
- [ ] Materials produce different outcomes
- [ ] Orders take time proportional to distance
- [ ] Terrain affects defense
- [ ] Entities die from wounds

### When Integration Is Complete

- [ ] Weapons forged in city equip entities
- [ ] Battle outcomes affect population
- [ ] Morale flows between layers

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Converted from analysis doc to formal specification |
| 2026-01-02 | Original MVP analysis created |
