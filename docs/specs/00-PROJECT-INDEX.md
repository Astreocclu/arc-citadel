# 00-PROJECT-INDEX
> Master index of all Arc Citadel design specifications

## Document Suite

| # | Document | Purpose | Status |
|---|----------|---------|--------|
| 00 | PROJECT-INDEX | This document - master navigation | Active |
| 01 | [GAME-DESIGN-DOCUMENT](01-GAME-DESIGN-DOCUMENT.md) | Core vision, pillars, game loop | Complete |
| 02 | [IMPLEMENTATION-ARCHITECTURE](02-IMPLEMENTATION-ARCHITECTURE.md) | Technical stack, module structure | Complete |
| 03 | [ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md) | Entity model, cognition, behavior | Complete |
| 04 | [LLM-INTERFACE-SPEC](04-LLM-INTERFACE-SPEC.md) | Natural language command parsing | Complete |
| 05 | [BATTLE-SYSTEM-SPEC](05-BATTLE-SYSTEM-SPEC.md) | Tactical combat layer | Complete |
| 06 | [BALANCE-COMPOSITION-SPEC](06-BALANCE-COMPOSITION-SPEC.md) | Emergent balance philosophy | Complete |
| 07 | [CAMPAIGN-MAP-SPEC](07-CAMPAIGN-MAP-SPEC.md) | Strategic hex map layer | Complete |
| 08 | [GENETICS-SYSTEM-SPEC](08-GENETICS-SYSTEM-SPEC.md) | Genome, phenotype, inheritance | Complete |
| 09 | [GAP-ANALYSIS](09-GAP-ANALYSIS.md) | Current state vs. MVP requirements | Complete |
| 10 | [PERFORMANCE-ARCHITECTURE-SPEC](10-PERFORMANCE-ARCHITECTURE-SPEC.md) | SoA, caching, optimization | Complete |
| 11 | [ACTION-CATALOG](11-ACTION-CATALOG.md) | Complete action definitions | Complete |
| 12 | [BATTLE-PLANNING-TERRAIN-SPEC](12-BATTLE-PLANNING-TERRAIN-SPEC.md) | Terrain effects, tactical planning | Complete |
| 13 | [MODULE-SCHEMA-SPEC](13-MODULE-SCHEMA-SPEC.md) | LLM-generated module format | Complete |
| 14 | [PHYSICS-COMBAT-SPEC](14-PHYSICS-COMBAT-SPEC.md) | Penetration, damage, wounds | Complete |
| 15 | [WORLD-GENERATION-SPEC](15-WORLD-GENERATION-SPEC.md) | Procedural generation systems | Complete |
| 16 | [RESOURCE-ECONOMY-SPEC](16-RESOURCE-ECONOMY-SPEC.md) | Resources, production, trade | Complete |
| 17 | [SOCIAL-MEMORY-SPEC](17-SOCIAL-MEMORY-SPEC.md) | Dunbar limits, relationship tracking | Complete |
| 18 | [SOCIAL-PRESSURE-MORALE-SPEC](18-SOCIAL-PRESSURE-MORALE-SPEC.md) | Group dynamics, morale systems | Complete |
| 19 | [HIERARCHICAL-CHUNKING-SPEC](19-HIERARCHICAL-CHUNKING-SPEC.md) | Skill abstraction and expertise | Complete |

---

## Core Design Pillars

These pillars are **inviolable constraints** that all specifications must respect:

### 1. Natural Language Command
Player intent flows through LLM parsing into structured game actions. The LLM is a translator, never a decision-maker for entities.

### 2. Species-Specific Cognition
Each species has its own value vocabulary as a **distinct type**. Human honor ≠ Dwarf honor - they are incompatible concepts.

### 3. Bottom-Up Emergence
Behavior emerges from the interaction of values, needs, and context. No scripted behaviors like `if coward then flee`.

### 4. Emergent Balance
**NO PERCENTAGE MODIFIERS.** Properties interact physically. A heavier weapon does more damage because `force = mass × acceleration`, not because it has `+20% damage`.

### 5. LLM-Generated Modules
Content (items, creatures, scenarios) can be generated at runtime by the LLM following strict schemas.

### 6. Distributed Architecture
The game runs as distributed services: Rust simulation, PostgreSQL persistence, LLM parsing, web UI.

### 7. Physics-Based Combat
No guaranteed outcomes. Penetration uses sigmoid probability curves. Armor doesn't "reduce damage by X%" - it physically blocks or fails.

### 8. Hierarchical Chunking Skills
Skills abstract multi-step sequences into single actions. Experts execute chunked routines; novices process each step.

---

## Game Loop Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         STRONGHOLD LAYER                             │
│  Base building, resource management, entity spawning, daily life     │
│  See: 01-GDD, 03-ENTITY, 16-RESOURCE-ECONOMY                        │
└──────────────────────────────┬──────────────────────────────────────┘
                               │ Threats emerge / Player deploys
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         CAMPAIGN LAYER                               │
│  Strategic hex map, army movement, scouting, supply lines            │
│  See: 07-CAMPAIGN-MAP, 12-BATTLE-PLANNING-TERRAIN                   │
└──────────────────────────────┬──────────────────────────────────────┘
                               │ Armies meet / Player engages
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                          BATTLE LAYER                                │
│  Tactical real-time combat, physics-based resolution, morale         │
│  See: 05-BATTLE-SYSTEM, 14-PHYSICS-COMBAT, 18-SOCIAL-PRESSURE       │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Entity Model Summary

```
┌──────────────────────────────────────────────────────────────────┐
│                         ENTITY                                    │
├──────────────────────────────────────────────────────────────────┤
│  Name: "Marcus the Bold"                                         │
│  Species: Human                                                  │
├──────────────────────────────────────────────────────────────────┤
│  GENOME → PHENOTYPE → PERSONALITY → VALUES                       │
│  (DNA)    (body)      (traits)       (motivations)               │
├──────────────────────────────────────────────────────────────────┤
│  Needs: hunger, thirst, rest, safety, social, purpose            │
│  Thoughts: [perception-generated, value-filtered, need-weighted] │
│  Tasks: [current action queue]                                   │
│  Social Memory: ~150 relationship slots (Dunbar limit)           │
└──────────────────────────────────────────────────────────────────┘
```

**Full specification:** [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md)

---

## Species Value Types

Values are **type-incompatible** across species. You cannot compare human honor to dwarf honor.

| Species | Value Type | Example Values |
|---------|------------|----------------|
| Human | `HumanValues` | honor, beauty, comfort, ambition, loyalty, love, justice, curiosity, safety, piety |
| Dwarf | `DwarfValues` | craft_truth, stone_debt, clan_weight, oath_chain, deep_memory, grudge_mark |
| Elf | `ElfValues` | pattern_beauty, slow_growth, star_longing, cycle_wisdom, tree_bond, fate_thread |

**Full specification:** [08-GENETICS-SYSTEM-SPEC](08-GENETICS-SYSTEM-SPEC.md)

---

## Technical Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Simulation | Rust 2021 | Core game logic, tick execution |
| Data Layout | SoA (Structure of Arrays) | Cache-efficient entity iteration |
| ECS | Custom Archetype-based | Entity management without OOP overhead |
| Persistence | PostgreSQL | World state, entity data |
| Async | Tokio | Non-blocking I/O, LLM calls |
| LLM | OpenAI API | Natural language parsing |
| Spatial | Sparse Hash Grid | O(1) neighbor queries |

**Full specification:** [02-IMPLEMENTATION-ARCHITECTURE](02-IMPLEMENTATION-ARCHITECTURE.md)

---

## Cross-References

### How Documents Relate

```
01-GDD ─────────────────────┬─────────────────────────────────────┐
   │                        │                                     │
   ▼                        ▼                                     ▼
03-ENTITY              05-BATTLE                           07-CAMPAIGN
   │                        │                                     │
   ├── 08-GENETICS          ├── 14-PHYSICS-COMBAT                 │
   ├── 17-SOCIAL-MEMORY     ├── 18-SOCIAL-PRESSURE                │
   ├── 19-CHUNKING          └── 12-TERRAIN                        │
   │                                                              │
   └────────────────────────┬─────────────────────────────────────┘
                            │
                            ▼
                      02-ARCHITECTURE
                            │
           ┌────────────────┼────────────────┐
           ▼                ▼                ▼
    10-PERFORMANCE    04-LLM-INTERFACE    13-MODULE-SCHEMA
                            │
                            ▼
                      06-BALANCE
```

### Reading Order

**For game designers:**
1. 01-GDD → 03-ENTITY → 05-BATTLE → 06-BALANCE

**For implementers:**
1. 02-ARCHITECTURE → 10-PERFORMANCE → 09-GAP-ANALYSIS → 03-ENTITY

**For content creators:**
1. 13-MODULE-SCHEMA → 11-ACTION-CATALOG → 08-GENETICS

---

## Conventions

### Document Structure

Each specification follows this template:

```markdown
# [NUMBER]-[TITLE]
> One-line summary

## Overview
What this system does and why it exists.

## Core Concepts
Key abstractions and mental models.

## Design Decisions
Why we chose this approach, alternatives considered.

## Technical Specification
Rust types, data structures, algorithms.

## Interactions
How this system connects to others.

## Implementation Notes
Practical guidance, gotchas, edge cases.

## Status
Current implementation state, known gaps.
```

### Terminology

| Term | Definition |
|------|------------|
| **Entity** | A game object with identity, components, and behavior |
| **Archetype** | A species-specific entity storage container (SoA layout) |
| **Value** | A species-specific motivation type (not a number, a concept) |
| **Need** | A universal physiological/psychological requirement |
| **Thought** | A perception-generated, value-filtered cognitive event |
| **Task** | A queued action being executed over time |
| **Tick** | One simulation update cycle |

---

## Implementation Status

See [09-GAP-ANALYSIS](09-GAP-ANALYSIS.md) for current implementation state.

**Summary (as of 2025-12-31 analysis):**
- City layer foundation: ~90% ready
- Battle layer foundation: ~50% ready
- Critical gaps: Buildings, Combat Resolution, Weapons/Armor, AI Opponents

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification suite created |
