# Skills Module

> Hierarchical chunking skill system - practiced actions combine into larger chunks that execute with lower attention cost.

## Philosophy

A novice conscript thinks: "Grip. Stance. Swing. Don't drop it."
A master thinks: "Handle this flank." - thousands of micro-actions automatic.

## Module Structure (7209 LOC total)

```
skills/
├── mod.rs              # Module exports (22 re-exported items)
├── chunk_id.rs         # Chunk identifier system (36106 LOC)
├── definitions.rs      # Chunk definitions library (62870 LOC)
├── spawn_loadouts.rs   # Role-based skill generation (33245 LOC)
├── history.rs          # Experience and history (36656 LOC)
├── library.rs          # Personal chunk library (10427 LOC)
├── integration.rs      # Skill check integration (10277 LOC)
├── resolution.rs       # Attack/defense resolution
├── learning.rs         # Skill learning mechanics
├── attention.rs        # Attention budget system
├── action_mapping.rs   # Action to skill mapping
├── context.rs          # Combat context tags
├── domain.rs           # Chunk domains
├── display.rs          # (orphaned - needs domain_summary)
└── species_mods.rs     # (orphaned - needs genetics)
```

## Status: COMPLETE IMPLEMENTATION

The skill system is fully implemented with:
- Hierarchical chunk-based mastery
- Attention budget mechanics
- Role-based skill generation
- Combat resolution integration

## Core Concept: Chunks

Skills are composed of "chunks" - automated sequences of micro-actions:

```rust
pub struct ChunkDefinition {
    pub id: ChunkId,
    pub name: String,
    pub domain: ChunkDomain,
    pub components: ChunkComponents,
    pub attention_cost: f32,
}
```

Higher-level chunks incorporate lower-level ones:
- `BasicParry` (attention: 0.3)
- `ParryRiposte` = `BasicParry` + `QuickThrust` (attention: 0.5)
- `CounterAttackSequence` = `ParryRiposte` + positioning (attention: 0.7)

## Attention Budget

Every entity has limited attention each tick:

```rust
pub fn calculate_attention_budget(entity: &Entity) -> f32

pub fn can_afford_attention(budget: f32, cost: f32) -> bool

pub fn risks_fumble(budget: f32, cost: f32) -> bool

pub const FUMBLE_ATTENTION_THRESHOLD: f32 = 0.1;
```

When attention is depleted, complex actions fail or fumble.

## Combat Resolution Integration

```rust
// Find best available chunk for an action
pub fn find_best_chunk(library: &ChunkLibrary, action: ActionType) -> Option<ChunkId>

// Resolve attack using skill chunks
pub fn resolve_attack(attacker: &ChunkLibrary, context: &CombatContext) -> ActionResult

// Resolve defense using skill chunks
pub fn resolve_defense(defender: &ChunkLibrary, context: &CombatContext) -> ActionResult

// Riposte after successful defense
pub fn resolve_riposte(defender: &ChunkLibrary, context: &CombatContext) -> ActionResult
```

## Experience and Learning

```rust
pub struct LifeExperience {
    pub role: Role,
    pub activities: Vec<ActivityType>,
    pub years: f32,
}

pub enum ActivityType {
    MilitaryTraining { years: f32 },
    CombatExperience { battles_fought: u32 },
    CraftApprenticeship { specialty: CraftSpecialty },
    // ... more activity types
}

// Generate chunks from life history
pub fn generate_chunks_from_history(history: &[LifeExperience]) -> ChunkLibrary
```

## Spawn Loadouts

Generate skills based on entity role:

```rust
pub fn generate_spawn_chunks(role: Role, unit_type: UnitType) -> ChunkLibrary
```

Role examples:
- `Soldier` - combat chunks
- `Craftsman { specialty }` - craft-specific chunks
- `Scholar` - knowledge chunks
- `Farmer` - agricultural chunks

## Key Exports

```rust
pub use action_mapping::{action_requires_skill, get_chunks_for_action};
pub use attention::{
    calculate_attention_budget, can_afford_attention, risks_fumble, FUMBLE_ATTENTION_THRESHOLD,
};
pub use chunk_id::ChunkId;
pub use context::{CombatContext, ContextTag};
pub use definitions::{get_chunk_definition, ChunkComponents, ChunkDefinition, CHUNK_LIBRARY};
pub use domain::ChunkDomain;
pub use history::{
    calculate_experience_contribution, combine_encoding, estimate_repetitions,
    generate_chunks_from_history, generate_history_for_role, get_chunks_for_activity, ActivityType,
    CraftSpecialty, LifeExperience, Role, UnitType,
};
pub use integration::{
    record_action_experience, refresh_attention, skill_check, spend_attention, SkillCheckResult,
    SkillFailure,
};
pub use learning::{calculate_encoding_depth, process_learning};
pub use library::{ChunkLibrary, Experience, PersonalChunkState};
pub use resolution::{
    find_best_chunk, resolve_attack, resolve_defense, resolve_riposte, ActionResult, ATTACK_CHUNKS,
    DEFENSE_CHUNKS, RIPOSTE_CHUNKS,
};
pub use spawn_loadouts::generate_spawn_chunks;
```

## Integration Points

### With `combat/`
- Combat resolution uses skill checks
- Weapon proficiency affects damage
- Fatigue increases attention costs

### With `battle/`
- Unit-level skill aggregation
- Formation coordination bonuses

### With `entity/`
- Skills stored per entity
- Experience tracking

## Orphaned Files

Two files exist but are not compiled:

1. `display.rs` - Depends on `ChunkLibrary::domain_summary()` (not implemented)
2. `species_mods.rs` - Depends on `genetics::Phenotype` (genetics module is stub)

These will be integrated when their dependencies are implemented.

## Testing

```bash
cargo test --lib skills::
```
