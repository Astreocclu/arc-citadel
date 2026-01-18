# 01-GAME-DESIGN-DOCUMENT
> Core vision, design pillars, and game loop for Arc Citadel

## Overview

Arc Citadel is a deep simulation strategy game where entity behavior emerges naturally from values, needs, and thoughts. Natural language commands flow through LLM parsing into structured game actions. The player experiences a living world where complexity arises from simple, interacting systems rather than scripted behaviors.

---

## Core Vision

**One Sentence**: A strategy game where you guide a settlement through natural language, watching behavior emerge from the interplay of entity psychology, physics, and circumstance.

**The Experience**: You speak to your settlement like a leader speaks to their people. "We need to fortify the eastern approach before winter." The game translates your intent into tasks, but how entities execute those tasks depends on their values, needs, relationships, and the physical realities they face. A coward doesn't flee because code says `if coward then flee`—they flee because their safety need overwhelms their duty, filtered through their values.

---

## Design Pillars

These are **inviolable constraints**. Every system must respect them.

### 1. Natural Language Command

**The Pillar**: Player intent flows through LLM parsing into structured game actions.

**What This Means**:
- Players type natural language: "Have the miners focus on iron this week"
- The LLM translates intent to structured `ParsedIntent`
- Game systems execute the intent through normal task/action flow
- The LLM is a **translator**, never a decision-maker for entities

**What This Forbids**:
- LLM controlling entity behavior directly
- LLM making tactical decisions for units
- Gameplay requiring LLM availability (graceful degradation required)

**Specification**: [04-LLM-INTERFACE-SPEC](04-LLM-INTERFACE-SPEC.md)

### 2. Species-Specific Cognition

**The Pillar**: Each species has its own value vocabulary as a **distinct type**.

**What This Means**:
- Human `honor` ≠ Dwarf `honor`—they are incompatible concepts
- A human prioritizing honor seeks social standing; a dwarf prioritizing clan honor fulfills ancestral debts
- Values are not weighted preferences on a universal scale; they are different mental frameworks

**Example Types**:

| Species | Type | Values |
|---------|------|--------|
| Human | `HumanValues` | honor, beauty, comfort, ambition, loyalty, love, justice, curiosity, safety, piety |
| Dwarf | `DwarfValues` | craft_truth, stone_debt, clan_weight, oath_chain, deep_memory, grudge_mark |
| Elf | `ElfValues` | pattern_beauty, slow_growth, star_longing, cycle_wisdom, tree_bond, fate_thread |

**What This Forbids**:
- Universal value scale across species
- `compare(human.honor, dwarf.honor)` ever compiling
- Cross-species value calculations

**Specification**: [08-GENETICS-SYSTEM-SPEC](08-GENETICS-SYSTEM-SPEC.md)

### 3. Bottom-Up Emergence

**The Pillar**: Behavior emerges from the interaction of values, needs, and context.

**What This Means**:
- Entities don't have behavior scripts
- No `if brave then charge` or `if hungry then eat` hardcoding
- Behavior emerges from: perception → thought → action selection
- The same entity in different contexts behaves differently

**The Flow**:
```
Perception (what entity notices)
  ↓ filtered by Values (what matters to this entity)
  ↓
Thoughts (emotional/cognitive reactions)
  ↓ weighted by Needs (what feels urgent)
  ↓
Action Selection (behavioral response)
```

**What This Forbids**:
- Personality-based behavior switches
- Hardcoded behavior patterns
- Direct value-to-action mappings

**Specification**: [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md)

### 4. Emergent Balance

**The Pillar**: NO PERCENTAGE MODIFIERS. Properties interact physically.

**What This Means**:
- A heavier weapon does more damage because `force = mass × acceleration`
- Not because it has `+20% damage`
- Armor doesn't "reduce damage by 30%"—it physically blocks or fails to block
- All balance emerges from physical property interaction

**Example**:
```rust
// CORRECT: Physical interaction
let impact_force = strength * weapon_mass * velocity;
let penetration = calculate_penetration(impact_force, armor_thickness, material_hardness);

// FORBIDDEN: Percentage modifiers
let damage = base_damage * weapon_modifier * armor_reduction; // NO
```

**What This Forbids**:
- Percentage-based damage modifiers
- Flat stat bonuses (`+10 attack`)
- Multiplier stacking
- Any system that doesn't reduce to physical properties

**Specification**: [06-BALANCE-COMPOSITION-SPEC](06-BALANCE-COMPOSITION-SPEC.md), [14-PHYSICS-COMBAT-SPEC](14-PHYSICS-COMBAT-SPEC.md)

### 5. LLM-Generated Modules

**The Pillar**: Content can be generated at runtime following strict schemas.

**What This Means**:
- Items, creatures, scenarios can be LLM-generated
- Generation follows rigid schemas (validated at runtime)
- Generated content integrates seamlessly with handcrafted content
- The game can create novel situations within defined constraints

**What This Forbids**:
- Arbitrary LLM output
- Unvalidated generated content
- Generation that violates game rules

**Specification**: [13-MODULE-SCHEMA-SPEC](13-MODULE-SCHEMA-SPEC.md)

### 6. Distributed Architecture

**The Pillar**: The game runs as distributed services.

**What This Means**:
- Rust simulation core (tick execution, entity management)
- PostgreSQL persistence (world state, entity data)
- LLM service (command parsing, content generation)
- Web UI (player interaction, visualization)

**What This Forbids**:
- Monolithic architecture
- In-memory-only state
- Single point of failure

**Specification**: [02-IMPLEMENTATION-ARCHITECTURE](02-IMPLEMENTATION-ARCHITECTURE.md)

### 7. Physics-Based Combat

**The Pillar**: No guaranteed outcomes. Combat uses probability from physics.

**What This Means**:
- Penetration uses sigmoid probability curves
- Armor fails probabilistically based on thickness vs. impact
- Hit location is random within weapon constraints
- The same attack can kill or bounce off

**The Penetration Curve**:
```rust
fn penetration_probability(impact: f32, armor: f32) -> f32 {
    // Sigmoid curve: ~0 when impact << armor, ~1 when impact >> armor
    let ratio = impact / armor;
    1.0 / (1.0 + (-10.0 * (ratio - 1.0)).exp())
}
```

**What This Forbids**:
- Guaranteed damage
- Deterministic combat outcomes
- "You need X damage to beat Y armor" certainties

**Specification**: [14-PHYSICS-COMBAT-SPEC](14-PHYSICS-COMBAT-SPEC.md)

### 8. Hierarchical Chunking Skills

**The Pillar**: Skills abstract multi-step sequences into single actions.

**What This Means**:
- A novice swordsman thinks: "raise weapon, step, swing, recover"
- A master thinks: "attack"
- Higher skill = fewer cognitive steps = faster execution
- Skills are chunked sequences, not percentage bonuses

**What This Forbids**:
- Skill = percentage modifier
- Flat execution speed
- Skills as damage multipliers

**Specification**: [19-HIERARCHICAL-CHUNKING-SPEC](19-HIERARCHICAL-CHUNKING-SPEC.md)

---

## Game Loop

Arc Citadel operates on three interconnected layers:

```
┌─────────────────────────────────────────────────────────────────────┐
│                         STRONGHOLD LAYER                             │
│  Base building, resource management, entity spawning, daily life     │
│                                                                      │
│  Player: "Build defenses" → Entities construct, gather, craft        │
│  Time scale: Days to weeks                                           │
└──────────────────────────────┬──────────────────────────────────────┘
                               │ Threats emerge / Player deploys
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         CAMPAIGN LAYER                               │
│  Strategic hex map, army movement, scouting, supply lines            │
│                                                                      │
│  Player: "Scout the eastern forest" → Units move, explore, report    │
│  Time scale: Weeks to months                                         │
└──────────────────────────────┬──────────────────────────────────────┘
                               │ Armies meet / Player engages
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          BATTLE LAYER                                │
│  Tactical real-time combat, physics-based resolution, morale         │
│                                                                      │
│  Player: "Hold the high ground" → Units position, fight, rout        │
│  Time scale: Minutes to hours                                        │
└─────────────────────────────────────────────────────────────────────┘
```

### Stronghold Layer

**Purpose**: Economic foundation, entity development, preparation

**Systems Active**:
- Building construction and maintenance
- Resource gathering and production chains
- Entity needs satisfaction (hunger, rest, social)
- Population growth and migration
- Skill development through practice

**Player Interaction**:
- Designate buildings and zones
- Set production priorities
- Manage workforce allocation
- Respond to entity concerns

**Specifications**:
- [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md)
- [16-RESOURCE-ECONOMY-SPEC](16-RESOURCE-ECONOMY-SPEC.md)

### Campaign Layer

**Purpose**: Strategic movement, intelligence, preparation for battle

**Systems Active**:
- Hex-based map navigation
- Supply line management
- Scouting and fog of war
- Army composition and deployment
- Diplomatic interactions

**Player Interaction**:
- Deploy units to regions
- Set strategic objectives
- Manage supply chains
- Negotiate with factions

**Specifications**:
- [07-CAMPAIGN-MAP-SPEC](07-CAMPAIGN-MAP-SPEC.md)
- [12-BATTLE-PLANNING-TERRAIN-SPEC](12-BATTLE-PLANNING-TERRAIN-SPEC.md)

### Battle Layer

**Purpose**: Tactical combat resolution

**Systems Active**:
- Real-time entity movement
- Physics-based combat resolution
- Order delays (courier system)
- Morale and routing
- Terrain effects

**Player Interaction**:
- Issue tactical orders (with delay)
- Set formation and positioning
- React to battlefield changes
- Decide when to retreat

**Specifications**:
- [05-BATTLE-SYSTEM-SPEC](05-BATTLE-SYSTEM-SPEC.md)
- [14-PHYSICS-COMBAT-SPEC](14-PHYSICS-COMBAT-SPEC.md)
- [18-SOCIAL-PRESSURE-MORALE-SPEC](18-SOCIAL-PRESSURE-MORALE-SPEC.md)

---

## Entity Model

Every entity in Arc Citadel follows this structure:

```
┌──────────────────────────────────────────────────────────────────┐
│                         ENTITY                                    │
├──────────────────────────────────────────────────────────────────┤
│  Identity                                                        │
│  ├── EntityId: u64                                               │
│  ├── Name: "Marcus the Bold"                                     │
│  └── Species: Human                                              │
├──────────────────────────────────────────────────────────────────┤
│  Body (Genome → Phenotype)                                       │
│  ├── Physical: strength, endurance, speed, size                  │
│  ├── Mental: intelligence, perception, willpower                 │
│  └── State: health, fatigue, wounds, equipment                   │
├──────────────────────────────────────────────────────────────────┤
│  Mind (Phenotype → Personality → Values)                         │
│  ├── Values: species-specific (HumanValues, DwarfValues, etc.)   │
│  ├── Needs: universal (hunger, thirst, rest, safety, social)     │
│  ├── Thoughts: [perception-generated, value-filtered buffer]     │
│  └── Tasks: [current action queue]                               │
├──────────────────────────────────────────────────────────────────┤
│  Relationships (Social Memory)                                   │
│  ├── ~150 slots (Dunbar limit)                                   │
│  ├── Per-relationship: disposition, expectations, history        │
│  └── Violations: tracked for grudge/gratitude                    │
└──────────────────────────────────────────────────────────────────┘
```

**Full Specification**: [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md)

---

## Simulation Tick

Each simulation tick follows this order:

```rust
pub fn tick(world: &mut World) {
    // 1. Needs decay over time
    update_needs(world);

    // 2. Entities perceive their environment
    let spatial_index = build_spatial_index(world);
    let perceptions = run_perception(world, &spatial_index);

    // 3. Perceptions generate thoughts (filtered by values)
    generate_thoughts(world, &perceptions);

    // 4. Old thoughts fade
    decay_thoughts(world);

    // 5. Idle entities choose actions (weighted by needs)
    select_actions(world);

    // 6. Current tasks make progress
    execute_tasks(world);

    // 7. Combat resolution (if applicable)
    resolve_combat(world);

    // 8. Production tick (buildings produce)
    run_production(world);

    // 9. Advance world time
    world.advance_tick();
}
```

**Full Specification**: [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md)

---

## Player Command Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│  Player Input: "Have the miners focus on iron this week"            │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│  LLM Parser                                                         │
│  ├── Extract intent: resource prioritization                        │
│  ├── Identify subjects: entities with Miner role                    │
│  ├── Identify target: Iron resource zones                           │
│  └── Determine duration: 1 week                                     │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│  ParsedIntent                                                       │
│  {                                                                  │
│    action: SetPriority,                                             │
│    subjects: [role:Miner],                                          │
│    target: ResourceType::Iron,                                      │
│    duration: Duration::Weeks(1),                                    │
│  }                                                                  │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│  Game Systems                                                       │
│  ├── Find all Miner entities                                        │
│  ├── Adjust task priority weights for Iron gathering                │
│  └── Set duration flag (revert after 1 week)                        │
└─────────────────────────────────────────────────────────────────────┘
```

**Full Specification**: [04-LLM-INTERFACE-SPEC](04-LLM-INTERFACE-SPEC.md)

---

## Astronomy & Time

The world has its own astronomical system affecting gameplay:

| Element | Effect |
|---------|--------|
| **Seasons** | Resource availability, temperature, morale |
| **Dual Moons** | Tide effects, lunar calendar, omens |
| **Eclipses** | Rare events with founding modifiers |
| **Day/Night** | Vision range, fatigue, creature activity |

**Founding Modifiers**: Settlements founded during eclipses or special conjunctions receive permanent bonuses/penalties based on astronomical conditions.

---

## Content Generation

The game can generate content following strict schemas:

| Content Type | Schema | Generation Trigger |
|--------------|--------|-------------------|
| Items | [13-MODULE-SCHEMA-SPEC](13-MODULE-SCHEMA-SPEC.md) | Crafting, loot, trade |
| Creatures | [13-MODULE-SCHEMA-SPEC](13-MODULE-SCHEMA-SPEC.md) | Spawning, migration |
| Scenarios | [13-MODULE-SCHEMA-SPEC](13-MODULE-SCHEMA-SPEC.md) | Events, quests |
| Dialogue | Context-specific | NPC interaction |

All generated content is **validated against schemas** before integration.

---

## Target Experience Metrics

### Stronghold Layer Success

- Player feels like a leader giving direction, not a micromanager
- Entities pursue their own satisfaction within player priorities
- Crises emerge from system interactions, not scripted events
- Settlement can sustain itself without constant player intervention

### Campaign Layer Success

- Strategic decisions have meaningful consequences
- Information is uncertain and must be gathered
- Supply and logistics create interesting constraints
- Diplomacy and alliances feel earned

### Battle Layer Success

- Combat outcomes feel physical and uncertain
- Orders have realistic delays creating tension
- Terrain and formation provide tactical depth
- Morale breaks before total casualties

---

## Related Specifications

| Document | Relationship |
|----------|--------------|
| [02-IMPLEMENTATION-ARCHITECTURE](02-IMPLEMENTATION-ARCHITECTURE.md) | Technical realization |
| [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md) | Entity behavior details |
| [04-LLM-INTERFACE-SPEC](04-LLM-INTERFACE-SPEC.md) | Command parsing |
| [05-BATTLE-SYSTEM-SPEC](05-BATTLE-SYSTEM-SPEC.md) | Tactical combat |
| [06-BALANCE-COMPOSITION-SPEC](06-BALANCE-COMPOSITION-SPEC.md) | Emergence philosophy |
| [09-GAP-ANALYSIS](09-GAP-ANALYSIS.md) | Implementation status |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification created |
