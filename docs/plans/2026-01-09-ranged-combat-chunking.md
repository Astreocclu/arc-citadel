# Ranged Combat System with Hierarchical Chunking

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement ranged combat (bows, crossbows, thrown weapons) using the hierarchical chunking system, where skill differentiation emerges from attention costs and available chunks rather than stat bonuses.

**Architecture:** Extend the existing chunking system with ranged-specific ChunkIds and ContextTags. Add ranged weapon properties. Insert a `phase_ranged` into the battle execution loop between movement and melee combat. Bows have deep chunk trees with high fatigue costs; crossbows have shallow trees with easy floor but limited ceiling; thrown weapons bridge melee and ranged.

**Tech Stack:** Rust, existing `src/skills/` module, `src/battle/` module, `src/combat/` module

---

## Task 1: Add Ranged Context Tags

**Files:**
- Modify: `src/skills/context.rs:6-27`
- Test: `src/skills/context.rs` (inline tests)

**Step 1: Write the failing test**

Add to the existing test module in `src/skills/context.rs`:

```rust
#[test]
fn test_ranged_context_tags() {
    let ctx = CombatContext::new()
        .with_tag(ContextTag::AtRange)
        .with_tag(ContextTag::HasBow)
        .with_tag(ContextTag::TargetVisible);

    assert!(ctx.has(ContextTag::AtRange));
    assert!(ctx.has(ContextTag::HasBow));
    assert!(ctx.has(ContextTag::TargetVisible));
    assert!(!ctx.has(ContextTag::HasCrossbow));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_ranged_context_tags`
Expected: FAIL with "no variant named `HasBow`"

**Step 3: Write minimal implementation**

Update the `ContextTag` enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextTag {
    // Spatial
    InMelee,
    AtRange,
    Flanked,
    Flanking,

    // Equipment - Melee
    HasSword,
    HasShield,
    HasPolearm,
    Armored,

    // Equipment - Ranged
    HasBow,
    HasCrossbow,
    HasThrown,
    AmmoAvailable,
    CrossbowLoaded,

    // Target
    EnemyVisible,
    TargetVisible,
    TargetInCover,
    MultipleEnemies,

    // State
    Fresh,
    Fatigued,
    HighGround,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_ranged_context_tags`
Expected: PASS

**Step 5: Commit**

```bash
git add src/skills/context.rs
git commit -m "$(cat <<'EOF'
feat(skills): add ranged context tags

Add ContextTags for ranged combat: HasBow, HasCrossbow, HasThrown,
AmmoAvailable, CrossbowLoaded, TargetVisible, TargetInCover, HighGround.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 2: Add Ranged Chunk IDs

**Files:**
- Modify: `src/skills/chunk_id.rs:6-21`
- Modify: `src/skills/chunk_id.rs:23-46` (impl block)

**Step 1: Write the failing test**

Add to existing tests in `src/skills/chunk_id.rs`:

```rust
#[test]
fn test_ranged_chunk_levels() {
    // Level 1 ranged
    assert_eq!(ChunkId::DrawBow.level(), 1);
    assert_eq!(ChunkId::LoadCrossbow.level(), 1);
    assert_eq!(ChunkId::BasicAim.level(), 1);
    assert_eq!(ChunkId::BasicThrow.level(), 1);

    // Level 2 ranged
    assert_eq!(ChunkId::LooseArrow.level(), 2);
    assert_eq!(ChunkId::CrossbowShot.level(), 2);
    assert_eq!(ChunkId::AimedThrow.level(), 2);

    // Level 3 ranged
    assert_eq!(ChunkId::RapidFire.level(), 3);
    assert_eq!(ChunkId::SniperShot.level(), 3);
    assert_eq!(ChunkId::VolleyFire.level(), 3);
}

#[test]
fn test_ranged_chunk_names() {
    assert_eq!(ChunkId::DrawBow.name(), "Draw Bow");
    assert_eq!(ChunkId::LooseArrow.name(), "Loose Arrow");
    assert_eq!(ChunkId::RapidFire.name(), "Rapid Fire");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_ranged_chunk_levels`
Expected: FAIL with "no variant named `DrawBow`"

**Step 3: Write minimal implementation**

Update `ChunkId` enum and impl:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChunkId {
    // === MELEE ===
    // Level 1 - Micro-chunks (first learning)
    BasicSwing,
    BasicBlock,
    BasicStance,

    // Level 2 - Action chunks (competent soldier)
    AttackSequence,
    DefendSequence,
    Riposte,

    // Level 3 - Tactical chunks (veteran)
    EngageMelee,
    HandleFlanking,

    // === RANGED ===
    // Level 1 - Micro-chunks
    DrawBow,        // Physical act of drawing bowstring
    LoadCrossbow,   // Spanning/winding crossbow mechanism
    BasicAim,       // Visual focus on target
    BasicThrow,     // Throwing motion fundamentals

    // Level 2 - Action chunks
    LooseArrow,     // Draw + Aim + Release (standard bow shot)
    CrossbowShot,   // Aim + Trigger (crossbow shot when loaded)
    AimedThrow,     // Aim + Throw (accurate thrown weapon)
    SnapShot,       // Quick bow shot, less accurate

    // Level 3 - Tactical chunks
    RapidFire,      // Multiple arrows in quick succession
    SniperShot,     // Maximum precision, high cost
    VolleyFire,     // Coordinated area fire
    PartingShot,    // Fire while retreating (horse archers)
}

impl ChunkId {
    pub fn level(&self) -> u8 {
        match self {
            // Melee Level 1
            Self::BasicSwing | Self::BasicBlock | Self::BasicStance => 1,
            // Melee Level 2
            Self::AttackSequence | Self::DefendSequence | Self::Riposte => 2,
            // Melee Level 3
            Self::EngageMelee | Self::HandleFlanking => 3,

            // Ranged Level 1
            Self::DrawBow | Self::LoadCrossbow | Self::BasicAim | Self::BasicThrow => 1,
            // Ranged Level 2
            Self::LooseArrow | Self::CrossbowShot | Self::AimedThrow | Self::SnapShot => 2,
            // Ranged Level 3
            Self::RapidFire | Self::SniperShot | Self::VolleyFire | Self::PartingShot => 3,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            // Melee
            Self::BasicSwing => "Basic Swing",
            Self::BasicBlock => "Basic Block",
            Self::BasicStance => "Basic Stance",
            Self::AttackSequence => "Attack Sequence",
            Self::DefendSequence => "Defend Sequence",
            Self::Riposte => "Riposte",
            Self::EngageMelee => "Engage Melee",
            Self::HandleFlanking => "Handle Flanking",

            // Ranged
            Self::DrawBow => "Draw Bow",
            Self::LoadCrossbow => "Load Crossbow",
            Self::BasicAim => "Basic Aim",
            Self::BasicThrow => "Basic Throw",
            Self::LooseArrow => "Loose Arrow",
            Self::CrossbowShot => "Crossbow Shot",
            Self::AimedThrow => "Aimed Throw",
            Self::SnapShot => "Snap Shot",
            Self::RapidFire => "Rapid Fire",
            Self::SniperShot => "Sniper Shot",
            Self::VolleyFire => "Volley Fire",
            Self::PartingShot => "Parting Shot",
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_ranged_chunk_levels && cargo test test_ranged_chunk_names`
Expected: PASS

**Step 5: Commit**

```bash
git add src/skills/chunk_id.rs
git commit -m "$(cat <<'EOF'
feat(skills): add ranged combat chunk IDs

Level 1: DrawBow, LoadCrossbow, BasicAim, BasicThrow
Level 2: LooseArrow, CrossbowShot, AimedThrow, SnapShot
Level 3: RapidFire, SniperShot, VolleyFire, PartingShot

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 3: Add Ranged Chunk Definitions

**Files:**
- Modify: `src/skills/definitions.rs:28-120`

**Step 1: Write the failing test**

Add to existing tests in `src/skills/definitions.rs`:

```rust
#[test]
fn test_ranged_chunks_exist() {
    // All ranged chunks should have definitions
    assert!(get_chunk_definition(ChunkId::DrawBow).is_some());
    assert!(get_chunk_definition(ChunkId::LoadCrossbow).is_some());
    assert!(get_chunk_definition(ChunkId::BasicAim).is_some());
    assert!(get_chunk_definition(ChunkId::BasicThrow).is_some());
    assert!(get_chunk_definition(ChunkId::LooseArrow).is_some());
    assert!(get_chunk_definition(ChunkId::CrossbowShot).is_some());
    assert!(get_chunk_definition(ChunkId::AimedThrow).is_some());
    assert!(get_chunk_definition(ChunkId::SnapShot).is_some());
    assert!(get_chunk_definition(ChunkId::RapidFire).is_some());
    assert!(get_chunk_definition(ChunkId::SniperShot).is_some());
    assert!(get_chunk_definition(ChunkId::VolleyFire).is_some());
    assert!(get_chunk_definition(ChunkId::PartingShot).is_some());
}

#[test]
fn test_ranged_prerequisites() {
    // LooseArrow requires DrawBow and BasicAim
    let def = get_chunk_definition(ChunkId::LooseArrow).unwrap();
    assert!(def.prerequisite_chunks.contains(&ChunkId::DrawBow));
    assert!(def.prerequisite_chunks.contains(&ChunkId::BasicAim));

    // CrossbowShot only requires BasicAim (loading is separate)
    let def = get_chunk_definition(ChunkId::CrossbowShot).unwrap();
    assert!(def.prerequisite_chunks.contains(&ChunkId::BasicAim));
    assert!(!def.prerequisite_chunks.contains(&ChunkId::LoadCrossbow));
}

#[test]
fn test_bow_requires_more_reps_than_crossbow() {
    let bow = get_chunk_definition(ChunkId::LooseArrow).unwrap();
    let crossbow = get_chunk_definition(ChunkId::CrossbowShot).unwrap();

    // Bows are harder to master
    assert!(bow.base_repetitions > crossbow.base_repetitions);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_ranged_chunks_exist`
Expected: FAIL with "assertion failed" (definitions don't exist yet)

**Step 3: Write minimal implementation**

Add to `CHUNK_LIBRARY` static slice:

```rust
pub static CHUNK_LIBRARY: &[ChunkDefinition] = &[
    // ... existing melee definitions ...

    // === RANGED Level 1 - Micro-chunks ===
    ChunkDefinition {
        id: ChunkId::DrawBow,
        name: "Draw Bow",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[ContextTag::HasBow, ContextTag::AtRange],
        prerequisite_chunks: &[],
        base_repetitions: 30, // Bow drawing requires muscle memory
    },
    ChunkDefinition {
        id: ChunkId::LoadCrossbow,
        name: "Load Crossbow",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[ContextTag::HasCrossbow],
        prerequisite_chunks: &[],
        base_repetitions: 15, // Mechanical, easier to learn
    },
    ChunkDefinition {
        id: ChunkId::BasicAim,
        name: "Basic Aim",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[ContextTag::TargetVisible],
        prerequisite_chunks: &[],
        base_repetitions: 20,
    },
    ChunkDefinition {
        id: ChunkId::BasicThrow,
        name: "Basic Throw",
        level: 1,
        components: ChunkComponents::Atomic,
        context_requirements: &[ContextTag::HasThrown, ContextTag::AtRange],
        prerequisite_chunks: &[],
        base_repetitions: 15,
    },

    // === RANGED Level 2 - Action chunks ===
    ChunkDefinition {
        id: ChunkId::LooseArrow,
        name: "Loose Arrow",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::DrawBow,
            ChunkId::BasicAim,
        ]),
        context_requirements: &[ContextTag::HasBow, ContextTag::AtRange, ContextTag::TargetVisible, ContextTag::AmmoAvailable],
        prerequisite_chunks: &[ChunkId::DrawBow, ChunkId::BasicAim],
        base_repetitions: 80,
    },
    ChunkDefinition {
        id: ChunkId::CrossbowShot,
        name: "Crossbow Shot",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::BasicAim,
        ]),
        context_requirements: &[ContextTag::HasCrossbow, ContextTag::CrossbowLoaded, ContextTag::AtRange, ContextTag::TargetVisible],
        prerequisite_chunks: &[ChunkId::BasicAim],
        base_repetitions: 40, // Easy - point and shoot
    },
    ChunkDefinition {
        id: ChunkId::AimedThrow,
        name: "Aimed Throw",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::BasicThrow,
            ChunkId::BasicAim,
        ]),
        context_requirements: &[ContextTag::HasThrown, ContextTag::AtRange, ContextTag::TargetVisible],
        prerequisite_chunks: &[ChunkId::BasicThrow, ChunkId::BasicAim],
        base_repetitions: 50,
    },
    ChunkDefinition {
        id: ChunkId::SnapShot,
        name: "Snap Shot",
        level: 2,
        components: ChunkComponents::Composite(&[
            ChunkId::DrawBow,
        ]),
        context_requirements: &[ContextTag::HasBow, ContextTag::AtRange, ContextTag::AmmoAvailable],
        prerequisite_chunks: &[ChunkId::DrawBow],
        base_repetitions: 60, // Quick but less precise
    },

    // === RANGED Level 3 - Tactical chunks ===
    ChunkDefinition {
        id: ChunkId::RapidFire,
        name: "Rapid Fire",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::LooseArrow,
            ChunkId::SnapShot,
        ]),
        context_requirements: &[ContextTag::HasBow, ContextTag::AtRange, ContextTag::AmmoAvailable],
        prerequisite_chunks: &[ChunkId::LooseArrow, ChunkId::SnapShot],
        base_repetitions: 200,
    },
    ChunkDefinition {
        id: ChunkId::SniperShot,
        name: "Sniper Shot",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::LooseArrow,
        ]),
        context_requirements: &[ContextTag::HasBow, ContextTag::AtRange, ContextTag::TargetVisible, ContextTag::AmmoAvailable],
        prerequisite_chunks: &[ChunkId::LooseArrow],
        base_repetitions: 250, // Mastery of precision
    },
    ChunkDefinition {
        id: ChunkId::VolleyFire,
        name: "Volley Fire",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::LooseArrow,
        ]),
        context_requirements: &[ContextTag::AtRange, ContextTag::AmmoAvailable],
        prerequisite_chunks: &[ChunkId::LooseArrow],
        base_repetitions: 150, // Coordinated fire
    },
    ChunkDefinition {
        id: ChunkId::PartingShot,
        name: "Parting Shot",
        level: 3,
        components: ChunkComponents::Composite(&[
            ChunkId::SnapShot,
        ]),
        context_requirements: &[ContextTag::HasBow, ContextTag::AtRange, ContextTag::AmmoAvailable],
        prerequisite_chunks: &[ChunkId::SnapShot],
        base_repetitions: 180, // Fire while retreating
    },
];
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_ranged_chunks_exist && cargo test test_ranged_prerequisites && cargo test test_bow_requires_more_reps`
Expected: PASS

**Step 5: Commit**

```bash
git add src/skills/definitions.rs
git commit -m "$(cat <<'EOF'
feat(skills): add ranged chunk definitions

Bow chunks: DrawBow -> LooseArrow/SnapShot -> RapidFire/SniperShot/VolleyFire
Crossbow chunks: LoadCrossbow, BasicAim -> CrossbowShot (shallow tree)
Thrown chunks: BasicThrow -> AimedThrow

Bows require more repetitions (high ceiling), crossbows are easier (low ceiling).

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 4: Add Ranged Resolution Functions

**Files:**
- Modify: `src/skills/resolution.rs:84-105`
- Modify: `src/skills/mod.rs:23-26`

**Step 1: Write the failing test**

Add to existing tests in `src/skills/resolution.rs`:

```rust
#[test]
fn test_resolve_ranged_bow_attack() {
    let mut lib = ChunkLibrary::new();
    lib.attention_budget = 1.0;

    // Add bow chunks for a trained archer
    lib.set_chunk(ChunkId::DrawBow, PersonalChunkState {
        encoding_depth: 0.5,
        repetition_count: 30,
        last_used_tick: 0,
        formation_tick: 0,
    });
    lib.set_chunk(ChunkId::BasicAim, PersonalChunkState {
        encoding_depth: 0.5,
        repetition_count: 20,
        last_used_tick: 0,
        formation_tick: 0,
    });
    lib.set_chunk(ChunkId::LooseArrow, PersonalChunkState {
        encoding_depth: 0.4,
        repetition_count: 40,
        last_used_tick: 0,
        formation_tick: 0,
    });

    let ctx = CombatContext::new()
        .with_tag(ContextTag::HasBow)
        .with_tag(ContextTag::AtRange)
        .with_tag(ContextTag::TargetVisible)
        .with_tag(ContextTag::AmmoAvailable);

    let result = resolve_ranged_attack(&mut lib, &ctx, 100);
    assert!(result.is_success());
}

#[test]
fn test_resolve_crossbow_requires_loaded() {
    let mut lib = ChunkLibrary::new();
    lib.attention_budget = 1.0;

    lib.set_chunk(ChunkId::BasicAim, PersonalChunkState {
        encoding_depth: 0.5,
        repetition_count: 20,
        last_used_tick: 0,
        formation_tick: 0,
    });
    lib.set_chunk(ChunkId::CrossbowShot, PersonalChunkState {
        encoding_depth: 0.5,
        repetition_count: 40,
        last_used_tick: 0,
        formation_tick: 0,
    });

    // Without CrossbowLoaded tag, should use fallback
    let ctx = CombatContext::new()
        .with_tag(ContextTag::HasCrossbow)
        .with_tag(ContextTag::AtRange)
        .with_tag(ContextTag::TargetVisible);

    let result = resolve_ranged_attack(&mut lib, &ctx, 100);
    // Should still succeed but with worse skill modifier (no CrossbowShot match)
    assert!(result.is_success());
    assert!(result.skill_modifier() < 0.5);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_resolve_ranged_bow_attack`
Expected: FAIL with "cannot find function `resolve_ranged_attack`"

**Step 3: Write minimal implementation**

Add to `src/skills/resolution.rs`:

```rust
/// Chunks applicable for bow attacks
pub const BOW_ATTACK_CHUNKS: &[ChunkId] = &[
    ChunkId::DrawBow,
    ChunkId::LooseArrow,
    ChunkId::SnapShot,
    ChunkId::RapidFire,
    ChunkId::SniperShot,
    ChunkId::VolleyFire,
    ChunkId::PartingShot,
];

/// Chunks applicable for crossbow attacks
pub const CROSSBOW_ATTACK_CHUNKS: &[ChunkId] = &[
    ChunkId::BasicAim,
    ChunkId::CrossbowShot,
];

/// Chunks applicable for thrown weapon attacks
pub const THROWN_ATTACK_CHUNKS: &[ChunkId] = &[
    ChunkId::BasicThrow,
    ChunkId::AimedThrow,
];

/// All ranged attack chunks (for generic ranged resolution)
pub const RANGED_ATTACK_CHUNKS: &[ChunkId] = &[
    ChunkId::DrawBow,
    ChunkId::BasicAim,
    ChunkId::BasicThrow,
    ChunkId::LooseArrow,
    ChunkId::CrossbowShot,
    ChunkId::AimedThrow,
    ChunkId::SnapShot,
    ChunkId::RapidFire,
    ChunkId::SniperShot,
    ChunkId::VolleyFire,
    ChunkId::PartingShot,
];

/// Resolve a ranged attack action (bow, crossbow, or thrown)
pub fn resolve_ranged_attack(
    library: &mut ChunkLibrary,
    context: &CombatContext,
    tick: u64,
) -> ActionResult {
    // Select appropriate chunk set based on context
    let chunks = if context.has(ContextTag::HasBow) {
        BOW_ATTACK_CHUNKS
    } else if context.has(ContextTag::HasCrossbow) {
        CROSSBOW_ATTACK_CHUNKS
    } else if context.has(ContextTag::HasThrown) {
        THROWN_ATTACK_CHUNKS
    } else {
        RANGED_ATTACK_CHUNKS
    };

    resolve_action(library, chunks, context, tick)
}

/// Resolve a crossbow reload action
pub fn resolve_crossbow_reload(
    library: &mut ChunkLibrary,
    context: &CombatContext,
    tick: u64,
) -> ActionResult {
    resolve_action(library, &[ChunkId::LoadCrossbow], context, tick)
}
```

Update `src/skills/mod.rs` to export new functions:

```rust
pub use resolution::{
    find_best_chunk, resolve_attack, resolve_defense, resolve_riposte,
    resolve_ranged_attack, resolve_crossbow_reload, ActionResult,
    ATTACK_CHUNKS, DEFENSE_CHUNKS, RIPOSTE_CHUNKS,
    BOW_ATTACK_CHUNKS, CROSSBOW_ATTACK_CHUNKS, THROWN_ATTACK_CHUNKS, RANGED_ATTACK_CHUNKS,
};
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_resolve_ranged_bow_attack && cargo test test_resolve_crossbow_requires_loaded`
Expected: PASS

**Step 5: Commit**

```bash
git add src/skills/resolution.rs src/skills/mod.rs
git commit -m "$(cat <<'EOF'
feat(skills): add ranged attack resolution functions

- resolve_ranged_attack() auto-selects chunk set by weapon type
- resolve_crossbow_reload() for reload action
- BOW_ATTACK_CHUNKS, CROSSBOW_ATTACK_CHUNKS, THROWN_ATTACK_CHUNKS constants

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 5: Add Ranged Weapon Properties

**Files:**
- Modify: `src/combat/weapons.rs:48-68`
- Modify: `src/combat/mod.rs` (if needed for exports)

**Step 1: Write the failing test**

Add to `src/combat/weapons.rs` tests:

```rust
#[test]
fn test_ranged_weapon_properties() {
    let longbow = RangedWeaponProperties {
        range: RangeCategory::Long,
        draw_strength: Mass::Heavy,
        reload_ticks: 0, // No reload for bows
        ammo_per_shot: 1,
    };
    assert_eq!(longbow.range, RangeCategory::Long);

    let crossbow = RangedWeaponProperties {
        range: RangeCategory::Medium,
        draw_strength: Mass::Light, // Mechanical advantage
        reload_ticks: 3,
        ammo_per_shot: 1,
    };
    assert_eq!(crossbow.reload_ticks, 3);
}

#[test]
fn test_range_category_ordering() {
    assert!(RangeCategory::Long > RangeCategory::Medium);
    assert!(RangeCategory::Medium > RangeCategory::Close);
}

#[test]
fn test_common_ranged_weapons() {
    let bow = RangedWeaponProperties::shortbow();
    assert_eq!(bow.range, RangeCategory::Medium);

    let crossbow = RangedWeaponProperties::light_crossbow();
    assert!(crossbow.reload_ticks > 0);

    let javelin = RangedWeaponProperties::javelin();
    assert_eq!(javelin.range, RangeCategory::Close);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_ranged_weapon_properties`
Expected: FAIL with "cannot find type `RangedWeaponProperties`"

**Step 3: Write minimal implementation**

Add to `src/combat/weapons.rs`:

```rust
/// Range category for ranged weapons
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RangeCategory {
    /// Thrown weapons (3-5 hexes effective)
    Close,
    /// Shortbows, light crossbows (8-12 hexes)
    Medium,
    /// Longbows, heavy crossbows (15-20+ hexes)
    Long,
}

/// Properties specific to ranged weapons
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangedWeaponProperties {
    /// Effective range category
    pub range: RangeCategory,
    /// Physical strength required to draw/throw (affects fatigue)
    pub draw_strength: Mass,
    /// Ticks required to reload (0 for bows, >0 for crossbows)
    pub reload_ticks: u8,
    /// Ammunition consumed per shot
    pub ammo_per_shot: u8,
}

impl RangedWeaponProperties {
    /// Shortbow - medium range, moderate draw
    pub fn shortbow() -> Self {
        Self {
            range: RangeCategory::Medium,
            draw_strength: Mass::Medium,
            reload_ticks: 0,
            ammo_per_shot: 1,
        }
    }

    /// Longbow - long range, heavy draw
    pub fn longbow() -> Self {
        Self {
            range: RangeCategory::Long,
            draw_strength: Mass::Heavy,
            reload_ticks: 0,
            ammo_per_shot: 1,
        }
    }

    /// Light crossbow - medium range, easy to use, slow reload
    pub fn light_crossbow() -> Self {
        Self {
            range: RangeCategory::Medium,
            draw_strength: Mass::Light,
            reload_ticks: 2,
            ammo_per_shot: 1,
        }
    }

    /// Heavy crossbow - long range, powerful, very slow reload
    pub fn heavy_crossbow() -> Self {
        Self {
            range: RangeCategory::Long,
            draw_strength: Mass::Light, // Mechanical advantage
            reload_ticks: 4,
            ammo_per_shot: 1,
        }
    }

    /// Javelin - close range, throwable
    pub fn javelin() -> Self {
        Self {
            range: RangeCategory::Close,
            draw_strength: Mass::Medium,
            reload_ticks: 0,
            ammo_per_shot: 1,
        }
    }

    /// Throwing axe - close range, heavy
    pub fn throwing_axe() -> Self {
        Self {
            range: RangeCategory::Close,
            draw_strength: Mass::Heavy,
            reload_ticks: 0,
            ammo_per_shot: 1,
        }
    }

    /// Sling - medium range, light
    pub fn sling() -> Self {
        Self {
            range: RangeCategory::Medium,
            draw_strength: Mass::Light,
            reload_ticks: 1,
            ammo_per_shot: 1,
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_ranged_weapon_properties && cargo test test_range_category_ordering && cargo test test_common_ranged_weapons`
Expected: PASS

**Step 5: Commit**

```bash
git add src/combat/weapons.rs
git commit -m "$(cat <<'EOF'
feat(combat): add ranged weapon properties

RangeCategory: Close, Medium, Long
RangedWeaponProperties: range, draw_strength, reload_ticks, ammo_per_shot

Presets: shortbow, longbow, light_crossbow, heavy_crossbow, javelin,
throwing_axe, sling

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 6: Add Archer/Crossbowman Chunk Libraries

**Files:**
- Modify: `src/skills/library.rs`

**Step 1: Write the failing test**

Add to `src/skills/library.rs` tests:

```rust
#[test]
fn test_trained_archer_library() {
    let lib = ChunkLibrary::trained_archer(0);

    // Should have bow fundamentals
    assert!(lib.has_chunk(ChunkId::DrawBow));
    assert!(lib.has_chunk(ChunkId::BasicAim));
    assert!(lib.has_chunk(ChunkId::LooseArrow));

    // Should NOT have crossbow chunks
    assert!(!lib.has_chunk(ChunkId::LoadCrossbow));
    assert!(!lib.has_chunk(ChunkId::CrossbowShot));
}

#[test]
fn test_trained_crossbowman_library() {
    let lib = ChunkLibrary::trained_crossbowman(0);

    // Should have crossbow fundamentals
    assert!(lib.has_chunk(ChunkId::LoadCrossbow));
    assert!(lib.has_chunk(ChunkId::BasicAim));
    assert!(lib.has_chunk(ChunkId::CrossbowShot));

    // Should NOT have bow chunks
    assert!(!lib.has_chunk(ChunkId::DrawBow));
    assert!(!lib.has_chunk(ChunkId::LooseArrow));
}

#[test]
fn test_veteran_archer_has_advanced_chunks() {
    let lib = ChunkLibrary::veteran_archer(0);

    // Should have advanced bow chunks
    assert!(lib.has_chunk(ChunkId::RapidFire));
    assert!(lib.has_chunk(ChunkId::SniperShot));

    // With high encoding depth
    assert!(lib.get_chunk(ChunkId::LooseArrow).unwrap().encoding_depth > 0.7);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_trained_archer_library`
Expected: FAIL with "no method named `trained_archer`"

**Step 3: Write minimal implementation**

Add to `impl ChunkLibrary` in `src/skills/library.rs`:

```rust
/// Create a trained archer's chunk library
pub fn trained_archer(formation_tick: u64) -> Self {
    let mut lib = Self::new();

    // Level 1 fundamentals
    lib.set_chunk(ChunkId::DrawBow, PersonalChunkState {
        encoding_depth: 0.5,
        repetition_count: 50,
        last_used_tick: formation_tick,
        formation_tick,
    });
    lib.set_chunk(ChunkId::BasicAim, PersonalChunkState {
        encoding_depth: 0.5,
        repetition_count: 40,
        last_used_tick: formation_tick,
        formation_tick,
    });

    // Level 2 - standard shot
    lib.set_chunk(ChunkId::LooseArrow, PersonalChunkState {
        encoding_depth: 0.4,
        repetition_count: 60,
        last_used_tick: formation_tick,
        formation_tick,
    });
    lib.set_chunk(ChunkId::SnapShot, PersonalChunkState {
        encoding_depth: 0.3,
        repetition_count: 30,
        last_used_tick: formation_tick,
        formation_tick,
    });

    lib
}

/// Create a veteran archer's chunk library
pub fn veteran_archer(formation_tick: u64) -> Self {
    let mut lib = Self::trained_archer(formation_tick);

    // Upgrade existing chunks
    lib.set_chunk(ChunkId::DrawBow, PersonalChunkState {
        encoding_depth: 0.85,
        repetition_count: 300,
        last_used_tick: formation_tick,
        formation_tick,
    });
    lib.set_chunk(ChunkId::BasicAim, PersonalChunkState {
        encoding_depth: 0.8,
        repetition_count: 250,
        last_used_tick: formation_tick,
        formation_tick,
    });
    lib.set_chunk(ChunkId::LooseArrow, PersonalChunkState {
        encoding_depth: 0.75,
        repetition_count: 200,
        last_used_tick: formation_tick,
        formation_tick,
    });
    lib.set_chunk(ChunkId::SnapShot, PersonalChunkState {
        encoding_depth: 0.7,
        repetition_count: 150,
        last_used_tick: formation_tick,
        formation_tick,
    });

    // Level 3 - advanced techniques
    lib.set_chunk(ChunkId::RapidFire, PersonalChunkState {
        encoding_depth: 0.5,
        repetition_count: 100,
        last_used_tick: formation_tick,
        formation_tick,
    });
    lib.set_chunk(ChunkId::SniperShot, PersonalChunkState {
        encoding_depth: 0.5,
        repetition_count: 100,
        last_used_tick: formation_tick,
        formation_tick,
    });

    lib
}

/// Create a trained crossbowman's chunk library
pub fn trained_crossbowman(formation_tick: u64) -> Self {
    let mut lib = Self::new();

    // Level 1 fundamentals
    lib.set_chunk(ChunkId::LoadCrossbow, PersonalChunkState {
        encoding_depth: 0.5,
        repetition_count: 30,
        last_used_tick: formation_tick,
        formation_tick,
    });
    lib.set_chunk(ChunkId::BasicAim, PersonalChunkState {
        encoding_depth: 0.5,
        repetition_count: 40,
        last_used_tick: formation_tick,
        formation_tick,
    });

    // Level 2 - crossbow shot (easy ceiling)
    lib.set_chunk(ChunkId::CrossbowShot, PersonalChunkState {
        encoding_depth: 0.6, // Higher floor than bow
        repetition_count: 50,
        last_used_tick: formation_tick,
        formation_tick,
    });

    lib
}

/// Create a veteran crossbowman's chunk library
pub fn veteran_crossbowman(formation_tick: u64) -> Self {
    let mut lib = Self::trained_crossbowman(formation_tick);

    // Crossbows have lower ceiling - max out faster
    lib.set_chunk(ChunkId::LoadCrossbow, PersonalChunkState {
        encoding_depth: 0.8, // Not as high as bow mastery
        repetition_count: 100,
        last_used_tick: formation_tick,
        formation_tick,
    });
    lib.set_chunk(ChunkId::BasicAim, PersonalChunkState {
        encoding_depth: 0.75,
        repetition_count: 120,
        last_used_tick: formation_tick,
        formation_tick,
    });
    lib.set_chunk(ChunkId::CrossbowShot, PersonalChunkState {
        encoding_depth: 0.75, // Lower ceiling than veteran archer
        repetition_count: 100,
        last_used_tick: formation_tick,
        formation_tick,
    });

    lib
}

/// Create a trained thrower's chunk library (javelins, axes)
pub fn trained_thrower(formation_tick: u64) -> Self {
    let mut lib = Self::new();

    lib.set_chunk(ChunkId::BasicThrow, PersonalChunkState {
        encoding_depth: 0.5,
        repetition_count: 30,
        last_used_tick: formation_tick,
        formation_tick,
    });
    lib.set_chunk(ChunkId::BasicAim, PersonalChunkState {
        encoding_depth: 0.4,
        repetition_count: 25,
        last_used_tick: formation_tick,
        formation_tick,
    });
    lib.set_chunk(ChunkId::AimedThrow, PersonalChunkState {
        encoding_depth: 0.4,
        repetition_count: 40,
        last_used_tick: formation_tick,
        formation_tick,
    });

    lib
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test test_trained_archer_library && cargo test test_trained_crossbowman_library && cargo test test_veteran_archer_has_advanced_chunks`
Expected: PASS

**Step 5: Commit**

```bash
git add src/skills/library.rs
git commit -m "$(cat <<'EOF'
feat(skills): add archer/crossbowman chunk libraries

- trained_archer / veteran_archer
- trained_crossbowman / veteran_crossbowman
- trained_thrower

Crossbowmen have higher floor but lower ceiling than archers.

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 7: Create Ranged Phase Module

**Files:**
- Create: `src/battle/ranged.rs`
- Modify: `src/battle/mod.rs`

**Step 1: Write the failing test**

Create file `src/battle/ranged.rs` with test:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::hex::BattleHexCoord;
    use crate::battle::units::{BattleUnit, UnitId};
    use crate::battle::unit_type::UnitType;

    #[test]
    fn test_can_shoot_at_target_in_range() {
        let shooter_pos = BattleHexCoord::new(0, 0);
        let target_pos = BattleHexCoord::new(5, 0);

        assert!(can_shoot(shooter_pos, target_pos, RangeCategory::Medium));
        assert!(!can_shoot(shooter_pos, target_pos, RangeCategory::Close));
    }

    #[test]
    fn test_range_category_to_hex_distance() {
        assert_eq!(max_range_hexes(RangeCategory::Close), 5);
        assert_eq!(max_range_hexes(RangeCategory::Medium), 12);
        assert_eq!(max_range_hexes(RangeCategory::Long), 20);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_can_shoot_at_target_in_range`
Expected: FAIL with "cannot find function `can_shoot`"

**Step 3: Write minimal implementation**

Create `src/battle/ranged.rs`:

```rust
//! Ranged combat phase for battle system
//!
//! Handles bow, crossbow, and thrown weapon attacks using the chunking skill system.

use crate::battle::hex::BattleHexCoord;
use crate::combat::weapons::RangeCategory;

/// Maximum effective range in hexes for each range category
pub fn max_range_hexes(range: RangeCategory) -> u32 {
    match range {
        RangeCategory::Close => 5,
        RangeCategory::Medium => 12,
        RangeCategory::Long => 20,
    }
}

/// Minimum effective range in hexes (can't shoot point-blank)
pub fn min_range_hexes(range: RangeCategory) -> u32 {
    match range {
        RangeCategory::Close => 2,
        RangeCategory::Medium => 3,
        RangeCategory::Long => 5,
    }
}

/// Check if a shooter can hit a target at this distance
pub fn can_shoot(shooter: BattleHexCoord, target: BattleHexCoord, range: RangeCategory) -> bool {
    let distance = shooter.distance(&target);
    distance >= min_range_hexes(range) && distance <= max_range_hexes(range)
}

/// Result of a ranged attack
#[derive(Debug, Clone)]
pub struct RangedAttackResult {
    /// Did the projectile hit?
    pub hit: bool,
    /// Casualties inflicted (if hit)
    pub casualties: u32,
    /// Stress inflicted on target (even on miss - suppression)
    pub stress_inflicted: f32,
    /// Fatigue cost to shooter
    pub fatigue_cost: f32,
    /// Ammo consumed
    pub ammo_consumed: u32,
}

impl Default for RangedAttackResult {
    fn default() -> Self {
        Self {
            hit: false,
            casualties: 0,
            stress_inflicted: 0.0,
            fatigue_cost: 0.0,
            ammo_consumed: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_shoot_at_target_in_range() {
        let shooter_pos = BattleHexCoord::new(0, 0);
        let target_pos = BattleHexCoord::new(5, 0);

        assert!(can_shoot(shooter_pos, target_pos, RangeCategory::Medium));
        assert!(!can_shoot(shooter_pos, target_pos, RangeCategory::Close));
    }

    #[test]
    fn test_range_category_to_hex_distance() {
        assert_eq!(max_range_hexes(RangeCategory::Close), 5);
        assert_eq!(max_range_hexes(RangeCategory::Medium), 12);
        assert_eq!(max_range_hexes(RangeCategory::Long), 20);
    }

    #[test]
    fn test_minimum_range() {
        let shooter = BattleHexCoord::new(0, 0);
        let too_close = BattleHexCoord::new(1, 0);

        // Can't shoot longbow at adjacent hex
        assert!(!can_shoot(shooter, too_close, RangeCategory::Long));
    }
}
```

Update `src/battle/mod.rs` to include the new module:

```rust
pub mod ranged;
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p arc_citadel --lib battle::ranged`
Expected: PASS

**Step 5: Commit**

```bash
git add src/battle/ranged.rs src/battle/mod.rs
git commit -m "$(cat <<'EOF'
feat(battle): add ranged combat module skeleton

- Range categories map to hex distances (Close=5, Medium=12, Long=20)
- Minimum ranges prevent point-blank shooting
- RangedAttackResult struct for attack outcomes

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 8: Implement Ranged Attack Resolution

**Files:**
- Modify: `src/battle/ranged.rs`

**Step 1: Write the failing test**

Add to `src/battle/ranged.rs` tests:

```rust
#[test]
fn test_resolve_unit_ranged_attack() {
    use crate::battle::unit_type::UnitType;
    use crate::battle::units::{BattleUnit, Element, UnitId};
    use crate::core::types::EntityId;

    // Create archer unit
    let mut archer = BattleUnit::new(UnitId::new(), UnitType::Archers);
    archer.position = BattleHexCoord::new(0, 0);
    archer.elements.push(Element::new(vec![EntityId::new(); 20]));

    // Create target infantry
    let mut target = BattleUnit::new(UnitId::new(), UnitType::Infantry);
    target.position = BattleHexCoord::new(8, 0);
    target.elements.push(Element::new(vec![EntityId::new(); 50]));

    let result = resolve_unit_ranged_attack(&archer, &target, 0, false);

    // Should have attempted attack
    assert!(result.ammo_consumed > 0);
    // Should cause some stress even if miss
    assert!(result.stress_inflicted >= 0.0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_resolve_unit_ranged_attack`
Expected: FAIL with "cannot find function `resolve_unit_ranged_attack`"

**Step 3: Write minimal implementation**

Add to `src/battle/ranged.rs`:

```rust
use crate::battle::battle_map::BattleMap;
use crate::battle::unit_type::UnitType;
use crate::battle::units::BattleUnit;
use crate::combat::weapons::{RangeCategory, RangedWeaponProperties};

/// Get ranged weapon properties for a unit type
pub fn unit_ranged_weapon(unit_type: UnitType) -> Option<RangedWeaponProperties> {
    match unit_type {
        UnitType::Archers => Some(RangedWeaponProperties::shortbow()),
        UnitType::Crossbowmen => Some(RangedWeaponProperties::light_crossbow()),
        UnitType::HorseArchers => Some(RangedWeaponProperties::shortbow()),
        _ => None,
    }
}

/// Resolve a ranged attack from one unit to another
///
/// Returns attack result. Does NOT mutate units - caller applies results.
pub fn resolve_unit_ranged_attack(
    attacker: &BattleUnit,
    defender: &BattleUnit,
    tick: u64,
    has_los: bool,
) -> RangedAttackResult {
    let mut result = RangedAttackResult::default();

    // Get weapon properties
    let weapon = match unit_ranged_weapon(attacker.unit_type) {
        Some(w) => w,
        None => return result, // Not a ranged unit
    };

    // Check range
    if !can_shoot(attacker.position, defender.position, weapon.range) {
        result.ammo_consumed = 0;
        return result;
    }

    // Base hit chance depends on skill (simplified - use encoding depth later)
    let base_hit_chance = 0.4;

    // Distance penalty
    let distance = attacker.position.distance(&defender.position);
    let max_range = max_range_hexes(weapon.range);
    let distance_penalty = (distance as f32 / max_range as f32) * 0.3;

    // Cover bonus for defender (would come from terrain)
    let cover_bonus = 0.0; // TODO: terrain lookup

    // LOS penalty
    let los_penalty = if has_los { 0.0 } else { 0.5 };

    // Final hit chance
    let hit_chance = (base_hit_chance - distance_penalty - cover_bonus - los_penalty).max(0.05);

    // Roll for hit (simplified - use RNG properly in real impl)
    let roll: f32 = rand::random();
    result.hit = roll < hit_chance;

    // Casualties if hit
    if result.hit {
        // Base casualties from ranged fire
        let effective_strength = attacker.effective_strength();
        let base_casualties = (effective_strength as f32 * 0.02).ceil() as u32;
        result.casualties = base_casualties.max(1);
    }

    // Stress inflicted (even misses cause suppression)
    result.stress_inflicted = if result.hit { 0.03 } else { 0.01 };

    // Fatigue cost based on draw strength
    result.fatigue_cost = match weapon.draw_strength {
        crate::combat::weapons::Mass::Light => 0.01,
        crate::combat::weapons::Mass::Medium => 0.02,
        crate::combat::weapons::Mass::Heavy => 0.03,
        crate::combat::weapons::Mass::Massive => 0.05,
    };

    result
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_resolve_unit_ranged_attack`
Expected: PASS

**Step 5: Commit**

```bash
git add src/battle/ranged.rs
git commit -m "$(cat <<'EOF'
feat(battle): implement ranged attack resolution

resolve_unit_ranged_attack computes:
- Range check
- Hit chance with distance/cover/LOS penalties
- Casualties on hit
- Suppression stress (even on miss)
- Fatigue cost based on draw strength

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 9: Integrate Ranged Phase into Battle Loop

**Files:**
- Modify: `src/battle/execution.rs:276-306`

**Step 1: Write the failing test**

Add to `src/battle/execution.rs` tests:

```rust
#[test]
fn test_ranged_phase_fires_before_melee() {
    use crate::battle::hex::BattleHexCoord;
    use crate::battle::unit_type::UnitType;
    use crate::battle::units::{Army, ArmyId, BattleFormation, BattleUnit, Element, FormationId, UnitId};
    use crate::core::types::EntityId;

    let map = BattleMap::new(30, 30);

    // Friendly archers
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    let mut f_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    let mut archers = BattleUnit::new(UnitId::new(), UnitType::Archers);
    archers.position = BattleHexCoord::new(5, 5);
    archers.elements.push(Element::new(vec![EntityId::new(); 30]));
    f_formation.units.push(archers);
    friendly.formations.push(f_formation);

    // Enemy infantry at range
    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut e_formation = BattleFormation::new(FormationId::new(), EntityId::new());
    let mut infantry = BattleUnit::new(UnitId::new(), UnitType::Infantry);
    infantry.position = BattleHexCoord::new(12, 5); // 7 hexes away - in shortbow range
    infantry.elements.push(Element::new(vec![EntityId::new(); 50]));
    e_formation.units.push(infantry);
    enemy.formations.push(e_formation);

    let mut state = BattleState::new(map, friendly, enemy);
    state.start_battle();

    // Run several ticks
    for _ in 0..5 {
        state.run_tick();
    }

    // Enemy should have taken stress from ranged fire
    let enemy_unit = state.enemy_army.formations[0].units.first().unwrap();
    assert!(enemy_unit.stress > 0.0, "Enemy should have stress from ranged fire");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_ranged_phase_fires_before_melee`
Expected: FAIL (enemy stress is 0 because ranged phase doesn't exist)

**Step 3: Write minimal implementation**

Add `phase_ranged` method to `BattleState` and call it from `run_tick`:

In `src/battle/execution.rs`, add the phase method:

```rust
fn phase_ranged(&mut self, events: &mut BattleEventLog) {
    use crate::battle::ranged::{can_shoot, resolve_unit_ranged_attack, unit_ranged_weapon};

    // Collect ranged units and their targets
    let mut ranged_attacks: Vec<(UnitId, UnitId)> = Vec::new();

    // Friendly ranged units targeting enemies
    for formation in &self.friendly_army.formations {
        for unit in &formation.units {
            if !unit.unit_type.is_ranged() || unit.is_broken() {
                continue;
            }

            // Find closest visible enemy in range
            if let Some(weapon) = unit_ranged_weapon(unit.unit_type) {
                let mut best_target: Option<(UnitId, u32)> = None;

                for e_formation in &self.enemy_army.formations {
                    for enemy in &e_formation.units {
                        if enemy.is_broken() {
                            continue;
                        }

                        let distance = unit.position.distance(&enemy.position);
                        if can_shoot(unit.position, enemy.position, weapon.range) {
                            if best_target.map_or(true, |(_, d)| distance < d) {
                                best_target = Some((enemy.id, distance));
                            }
                        }
                    }
                }

                if let Some((target_id, _)) = best_target {
                    ranged_attacks.push((unit.id, target_id));
                }
            }
        }
    }

    // Enemy ranged units targeting friendlies
    for formation in &self.enemy_army.formations {
        for unit in &formation.units {
            if !unit.unit_type.is_ranged() || unit.is_broken() {
                continue;
            }

            if let Some(weapon) = unit_ranged_weapon(unit.unit_type) {
                let mut best_target: Option<(UnitId, u32)> = None;

                for f_formation in &self.friendly_army.formations {
                    for friendly in &f_formation.units {
                        if friendly.is_broken() {
                            continue;
                        }

                        let distance = unit.position.distance(&friendly.position);
                        if can_shoot(unit.position, friendly.position, weapon.range) {
                            if best_target.map_or(true, |(_, d)| distance < d) {
                                best_target = Some((friendly.id, distance));
                            }
                        }
                    }
                }

                if let Some((target_id, _)) = best_target {
                    ranged_attacks.push((unit.id, target_id));
                }
            }
        }
    }

    // Resolve attacks
    for (attacker_id, defender_id) in ranged_attacks {
        let attacker = self.get_unit(attacker_id);
        let defender = self.get_unit(defender_id);

        if let (Some(attacker), Some(defender)) = (attacker, defender) {
            // Check LOS (simplified - would use map.has_los)
            let has_los = true; // TODO: proper LOS check

            let result = resolve_unit_ranged_attack(attacker, defender, self.tick, has_los);

            // Apply results to defender
            if let Some(target) = self.get_unit_mut(defender_id) {
                target.casualties += result.casualties;
                target.stress += result.stress_inflicted;
            }

            // Apply fatigue to attacker
            if let Some(shooter) = self.get_unit_mut(attacker_id) {
                shooter.fatigue = (shooter.fatigue + result.fatigue_cost).min(1.0);
            }
        }
    }
}
```

Update `run_tick` to call the new phase between movement and combat:

```rust
pub fn run_tick(&mut self) -> BattleEventLog {
    let mut events = BattleEventLog::new();

    if self.is_finished() {
        return events;
    }

    // ===== PHASE 0: AI DECISIONS =====
    self.phase_ai(&mut events);

    // ===== PHASE 1: PRE-TICK =====
    self.phase_pre_tick(&mut events);

    // ===== PHASE 2: MOVEMENT =====
    self.phase_movement(&mut events);

    // ===== PHASE 3: RANGED =====  <-- NEW
    self.phase_ranged(&mut events);

    // ===== PHASE 4: COMBAT (MELEE) =====
    self.phase_combat(&mut events);

    // ===== PHASE 5: MORALE =====
    self.phase_morale(&mut events);

    // ===== PHASE 6: ROUT =====
    self.phase_rout(&mut events);

    // ===== PHASE 7: POST-TICK =====
    self.phase_post_tick(&mut events);

    events
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_ranged_phase_fires_before_melee`
Expected: PASS

**Step 5: Commit**

```bash
git add src/battle/execution.rs
git commit -m "$(cat <<'EOF'
feat(battle): integrate ranged phase into battle loop

phase_ranged runs after movement, before melee combat:
- Finds ranged units with targets in range
- Resolves attacks, applies casualties and stress
- Applies fatigue to shooters

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Task 10: Integration Test - Full Ranged Combat

**Files:**
- Create: `tests/ranged_combat_integration.rs`

**Step 1: Write the failing test**

Create `tests/ranged_combat_integration.rs`:

```rust
//! Integration tests for ranged combat using hierarchical chunking

use arc_citadel::battle::battle_map::BattleMap;
use arc_citadel::battle::execution::BattleState;
use arc_citadel::battle::hex::BattleHexCoord;
use arc_citadel::battle::unit_type::UnitType;
use arc_citadel::battle::units::{Army, ArmyId, BattleFormation, BattleUnit, Element, FormationId, UnitId};
use arc_citadel::core::types::EntityId;
use arc_citadel::skills::{
    resolve_ranged_attack, ChunkId, ChunkLibrary, CombatContext, ContextTag,
};

/// Test that veteran archers outperform conscript archers
#[test]
fn test_veteran_archer_vs_conscript() {
    // Create contexts
    let bow_context = CombatContext::new()
        .with_tag(ContextTag::HasBow)
        .with_tag(ContextTag::AtRange)
        .with_tag(ContextTag::TargetVisible)
        .with_tag(ContextTag::AmmoAvailable);

    // Veteran archer
    let mut veteran = ChunkLibrary::veteran_archer(0);
    veteran.attention_budget = 1.0;
    let vet_result = resolve_ranged_attack(&mut veteran, &bow_context, 100);
    let vet_attention = veteran.attention_remaining();

    // Conscript (no archery training)
    let mut conscript = ChunkLibrary::new();
    conscript.attention_budget = 1.0;
    let con_result = resolve_ranged_attack(&mut conscript, &bow_context, 100);
    let con_attention = conscript.attention_remaining();

    // Veteran should have more attention remaining
    assert!(vet_attention > con_attention + 0.3,
        "Veteran attention {} should be much higher than conscript {}",
        vet_attention, con_attention);

    // Veteran should have higher skill modifier
    assert!(vet_result.skill_modifier() > con_result.skill_modifier(),
        "Veteran skill {} should exceed conscript {}",
        vet_result.skill_modifier(), con_result.skill_modifier());
}

/// Test that crossbowmen have easier time than archers at basic shooting
#[test]
fn test_crossbow_easier_than_bow() {
    let crossbow_context = CombatContext::new()
        .with_tag(ContextTag::HasCrossbow)
        .with_tag(ContextTag::CrossbowLoaded)
        .with_tag(ContextTag::AtRange)
        .with_tag(ContextTag::TargetVisible);

    let bow_context = CombatContext::new()
        .with_tag(ContextTag::HasBow)
        .with_tag(ContextTag::AtRange)
        .with_tag(ContextTag::TargetVisible)
        .with_tag(ContextTag::AmmoAvailable);

    // Trained crossbowman
    let mut crossbowman = ChunkLibrary::trained_crossbowman(0);
    crossbowman.attention_budget = 1.0;
    let xbow_result = resolve_ranged_attack(&mut crossbowman, &crossbow_context, 100);
    let xbow_attention = crossbowman.attention_remaining();

    // Trained archer (same training level)
    let mut archer = ChunkLibrary::trained_archer(0);
    archer.attention_budget = 1.0;
    let bow_result = resolve_ranged_attack(&mut archer, &bow_context, 100);
    let bow_attention = archer.attention_remaining();

    // At same training level, crossbow should be easier (more attention remaining)
    assert!(xbow_attention >= bow_attention,
        "Crossbow attention {} should be >= bow attention {}",
        xbow_attention, bow_attention);
}

/// Test full battle with ranged units
#[test]
fn test_battle_with_ranged_units() {
    let map = BattleMap::new(30, 30);

    // Friendly: archers behind infantry
    let mut friendly = Army::new(ArmyId::new(), EntityId::new());
    let mut f_formation = BattleFormation::new(FormationId::new(), EntityId::new());

    let mut archers = BattleUnit::new(UnitId::new(), UnitType::Archers);
    archers.position = BattleHexCoord::new(5, 5);
    archers.elements.push(Element::new(vec![EntityId::new(); 30]));
    f_formation.units.push(archers);

    let mut infantry = BattleUnit::new(UnitId::new(), UnitType::Infantry);
    infantry.position = BattleHexCoord::new(10, 5);
    infantry.elements.push(Element::new(vec![EntityId::new(); 50]));
    f_formation.units.push(infantry);

    friendly.formations.push(f_formation);

    // Enemy: infantry advancing
    let mut enemy = Army::new(ArmyId::new(), EntityId::new());
    let mut e_formation = BattleFormation::new(FormationId::new(), EntityId::new());

    let mut enemy_infantry = BattleUnit::new(UnitId::new(), UnitType::Infantry);
    enemy_infantry.position = BattleHexCoord::new(15, 5);
    enemy_infantry.elements.push(Element::new(vec![EntityId::new(); 80]));
    e_formation.units.push(enemy_infantry);

    enemy.formations.push(e_formation);

    let mut state = BattleState::new(map, friendly, enemy);
    state.start_battle();

    // Run 20 ticks
    for _ in 0..20 {
        state.run_tick();
    }

    // Enemy should have taken casualties from ranged fire
    let enemy_casualties: u32 = state.enemy_army
        .formations.iter()
        .flat_map(|f| f.units.iter())
        .map(|u| u.casualties)
        .sum();

    assert!(enemy_casualties > 0, "Enemy should have ranged casualties");

    // Enemy should have stress from suppression
    let enemy_stress: f32 = state.enemy_army
        .formations.iter()
        .flat_map(|f| f.units.iter())
        .map(|u| u.stress)
        .sum();

    assert!(enemy_stress > 0.0, "Enemy should have suppression stress");
}
```

**Step 2: Run test to verify it compiles and runs**

Run: `cargo test --test ranged_combat_integration`
Expected: Some tests may fail if earlier tasks not complete

**Step 3: Fix any issues**

Ensure all imports are correct and previous tasks are completed.

**Step 4: Run all tests to verify full integration**

Run: `cargo test`
Expected: ALL PASS

**Step 5: Commit**

```bash
git add tests/ranged_combat_integration.rs
git commit -m "$(cat <<'EOF'
test: add ranged combat integration tests

- test_veteran_archer_vs_conscript: skill differentiation
- test_crossbow_easier_than_bow: weapon differentiation
- test_battle_with_ranged_units: full battle integration

Co-Authored-By: Claude Opus 4.5 <noreply@anthropic.com>
EOF
)"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Add ranged context tags | `src/skills/context.rs` |
| 2 | Add ranged chunk IDs | `src/skills/chunk_id.rs` |
| 3 | Add ranged chunk definitions | `src/skills/definitions.rs` |
| 4 | Add ranged resolution functions | `src/skills/resolution.rs`, `mod.rs` |
| 5 | Add ranged weapon properties | `src/combat/weapons.rs` |
| 6 | Add archer/crossbowman libraries | `src/skills/library.rs` |
| 7 | Create ranged phase module | `src/battle/ranged.rs`, `mod.rs` |
| 8 | Implement ranged attack resolution | `src/battle/ranged.rs` |
| 9 | Integrate into battle loop | `src/battle/execution.rs` |
| 10 | Full integration tests | `tests/ranged_combat_integration.rs` |

**Verification command after all tasks:**
```bash
cargo test && cargo build
```

**Expected output:** All tests pass, build succeeds.
