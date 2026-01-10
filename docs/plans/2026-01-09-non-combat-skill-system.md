# Non-Combat Skill System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Expand the existing combat-focused chunking system to 6 non-combat skill domains (Craft, Social, Medicine, Leadership, Knowledge, Physical) with species modifiers, phenotype integration, and computed display stats.

**Architecture:**
- Extend flat `ChunkId` enum with ~70 new variants across 6 domains
- Add `ChunkDomain` enum and `domain()` method to ChunkId
- Add `Phenotype` struct to provide skill ceilings
- Integrate with action resolution for quality/success outcomes
- Compute display stats on-demand from underlying chunks

**Tech Stack:** Rust, serde, existing skills module infrastructure

---

## Task 1: Add ChunkDomain Enum

**Files:**
- Create: `src/skills/domain.rs`
- Modify: `src/skills/mod.rs:1-27`

**Step 1: Write the failing test**

Create `src/skills/domain.rs`:

```rust
//! Skill domains for chunk categorization

use serde::{Deserialize, Serialize};

/// Domain categories for skill chunks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChunkDomain {
    /// Combat: melee, ranged, defensive maneuvers
    Combat,
    /// Craft: smithing, carpentry, tailoring, etc.
    Craft,
    /// Social: persuasion, negotiation, deception
    Social,
    /// Medicine: wound care, surgery, herbalism
    Medicine,
    /// Leadership: command, tactics, morale
    Leadership,
    /// Knowledge: research, teaching, languages
    Knowledge,
    /// Physical: athletics, stealth, climbing
    Physical,
}

impl ChunkDomain {
    /// Get all domains
    pub fn all() -> &'static [ChunkDomain] {
        &[
            ChunkDomain::Combat,
            ChunkDomain::Craft,
            ChunkDomain::Social,
            ChunkDomain::Medicine,
            ChunkDomain::Leadership,
            ChunkDomain::Knowledge,
            ChunkDomain::Physical,
        ]
    }

    /// Human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            ChunkDomain::Combat => "Combat",
            ChunkDomain::Craft => "Craft",
            ChunkDomain::Social => "Social",
            ChunkDomain::Medicine => "Medicine",
            ChunkDomain::Leadership => "Leadership",
            ChunkDomain::Knowledge => "Knowledge",
            ChunkDomain::Physical => "Physical",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_domains() {
        assert_eq!(ChunkDomain::all().len(), 7);
    }

    #[test]
    fn test_domain_names() {
        assert_eq!(ChunkDomain::Combat.name(), "Combat");
        assert_eq!(ChunkDomain::Craft.name(), "Craft");
    }
}
```

**Step 2: Run test to verify it compiles and passes**

Run: `cargo test --lib skills::domain -- --nocapture`
Expected: PASS (2 tests)

**Step 3: Export from mod.rs**

Edit `src/skills/mod.rs` to add:

```rust
pub mod domain;
pub use domain::ChunkDomain;
```

**Step 4: Verify module exports**

Run: `cargo build --lib`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add src/skills/domain.rs src/skills/mod.rs
git commit -m "feat(skills): add ChunkDomain enum for skill categorization"
```

---

## Task 2: Add domain() Method to ChunkId

**Files:**
- Modify: `src/skills/chunk_id.rs:1-65`

**Step 1: Write the failing test**

Add to `src/skills/chunk_id.rs` tests:

```rust
#[test]
fn test_chunk_domains() {
    use crate::skills::ChunkDomain;

    // Combat chunks
    assert_eq!(ChunkId::BasicSwing.domain(), ChunkDomain::Combat);
    assert_eq!(ChunkId::HandleFlanking.domain(), ChunkDomain::Combat);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib skills::chunk_id::tests::test_chunk_domains`
Expected: FAIL with "no method named `domain`"

**Step 3: Implement domain() method**

Add to `ChunkId` impl block in `src/skills/chunk_id.rs`:

```rust
use crate::skills::ChunkDomain;

impl ChunkId {
    /// Get the domain this chunk belongs to
    pub const fn domain(&self) -> ChunkDomain {
        match self {
            // Combat domain (all existing chunks)
            Self::BasicSwing
            | Self::BasicBlock
            | Self::BasicStance
            | Self::AttackSequence
            | Self::DefendSequence
            | Self::Riposte
            | Self::EngageMelee
            | Self::HandleFlanking => ChunkDomain::Combat,
        }
    }

    // ... existing methods
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib skills::chunk_id::tests::test_chunk_domains`
Expected: PASS

**Step 5: Commit**

```bash
git add src/skills/chunk_id.rs
git commit -m "feat(skills): add domain() method to ChunkId"
```

---

## Task 3: Add Craft Domain Chunks

**Files:**
- Modify: `src/skills/chunk_id.rs:1-65`
- Modify: `src/skills/definitions.rs:1-174`

**Step 1: Write the failing test**

Add to `src/skills/chunk_id.rs` tests:

```rust
#[test]
fn test_craft_chunks_exist() {
    use crate::skills::ChunkDomain;

    // Level 1 craft chunks
    assert_eq!(ChunkId::CraftBasicHeatCycle.domain(), ChunkDomain::Craft);
    assert_eq!(ChunkId::CraftBasicHammerWork.domain(), ChunkDomain::Craft);
    assert_eq!(ChunkId::CraftBasicMeasure.domain(), ChunkDomain::Craft);

    // Level 2
    assert_eq!(ChunkId::CraftDrawOutMetal.domain(), ChunkDomain::Craft);

    // Level 3
    assert_eq!(ChunkId::CraftForgeKnife.domain(), ChunkDomain::Craft);

    // Level 4
    assert_eq!(ChunkId::CraftForgeSword.domain(), ChunkDomain::Craft);

    // Level 5
    assert_eq!(ChunkId::CraftForgeMasterwork.domain(), ChunkDomain::Craft);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib skills::chunk_id::tests::test_craft_chunks_exist`
Expected: FAIL with "no variant named `CraftBasicHeatCycle`"

**Step 3: Add Craft chunk variants to ChunkId enum**

Edit `src/skills/chunk_id.rs` enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChunkId {
    // === Combat Domain (existing) ===
    // Level 1
    BasicSwing,
    BasicBlock,
    BasicStance,
    // Level 2
    AttackSequence,
    DefendSequence,
    Riposte,
    // Level 3
    EngageMelee,
    HandleFlanking,

    // === Craft Domain ===
    // Level 1 - Micro-chunks
    CraftBasicHeatCycle,
    CraftBasicHammerWork,
    CraftBasicMeasure,
    CraftBasicCut,
    CraftBasicJoin,
    // Level 2 - Technique chunks
    CraftDrawOutMetal,
    CraftUpsetMetal,
    CraftBasicWeld,
    CraftShapeWood,
    CraftFinishSurface,
    // Level 3 - Product chunks
    CraftForgeKnife,
    CraftForgeToolHead,
    CraftBuildFurniture,
    CraftSewGarment,
    // Level 4 - Complex product chunks
    CraftForgeSword,
    CraftForgeArmor,
    CraftBuildStructure,
    CraftPatternWeld,
    // Level 5 - Mastery chunks
    CraftAssessAndExecute,
    CraftForgeMasterwork,
    CraftInnovativeTechnique,
}
```

**Step 4: Update domain() match**

```rust
pub const fn domain(&self) -> ChunkDomain {
    match self {
        // Combat domain
        Self::BasicSwing
        | Self::BasicBlock
        | Self::BasicStance
        | Self::AttackSequence
        | Self::DefendSequence
        | Self::Riposte
        | Self::EngageMelee
        | Self::HandleFlanking => ChunkDomain::Combat,

        // Craft domain
        Self::CraftBasicHeatCycle
        | Self::CraftBasicHammerWork
        | Self::CraftBasicMeasure
        | Self::CraftBasicCut
        | Self::CraftBasicJoin
        | Self::CraftDrawOutMetal
        | Self::CraftUpsetMetal
        | Self::CraftBasicWeld
        | Self::CraftShapeWood
        | Self::CraftFinishSurface
        | Self::CraftForgeKnife
        | Self::CraftForgeToolHead
        | Self::CraftBuildFurniture
        | Self::CraftSewGarment
        | Self::CraftForgeSword
        | Self::CraftForgeArmor
        | Self::CraftBuildStructure
        | Self::CraftPatternWeld
        | Self::CraftAssessAndExecute
        | Self::CraftForgeMasterwork
        | Self::CraftInnovativeTechnique => ChunkDomain::Craft,
    }
}
```

**Step 5: Update level() and name() methods**

```rust
pub fn level(&self) -> u8 {
    match self {
        // Combat
        Self::BasicSwing | Self::BasicBlock | Self::BasicStance => 1,
        Self::AttackSequence | Self::DefendSequence | Self::Riposte => 2,
        Self::EngageMelee | Self::HandleFlanking => 3,

        // Craft Level 1
        Self::CraftBasicHeatCycle
        | Self::CraftBasicHammerWork
        | Self::CraftBasicMeasure
        | Self::CraftBasicCut
        | Self::CraftBasicJoin => 1,
        // Craft Level 2
        Self::CraftDrawOutMetal
        | Self::CraftUpsetMetal
        | Self::CraftBasicWeld
        | Self::CraftShapeWood
        | Self::CraftFinishSurface => 2,
        // Craft Level 3
        Self::CraftForgeKnife
        | Self::CraftForgeToolHead
        | Self::CraftBuildFurniture
        | Self::CraftSewGarment => 3,
        // Craft Level 4
        Self::CraftForgeSword
        | Self::CraftForgeArmor
        | Self::CraftBuildStructure
        | Self::CraftPatternWeld => 4,
        // Craft Level 5
        Self::CraftAssessAndExecute
        | Self::CraftForgeMasterwork
        | Self::CraftInnovativeTechnique => 5,
    }
}

pub fn name(&self) -> &'static str {
    match self {
        // Combat
        Self::BasicSwing => "Basic Swing",
        Self::BasicBlock => "Basic Block",
        Self::BasicStance => "Basic Stance",
        Self::AttackSequence => "Attack Sequence",
        Self::DefendSequence => "Defend Sequence",
        Self::Riposte => "Riposte",
        Self::EngageMelee => "Engage Melee",
        Self::HandleFlanking => "Handle Flanking",

        // Craft
        Self::CraftBasicHeatCycle => "Basic Heat Cycle",
        Self::CraftBasicHammerWork => "Basic Hammer Work",
        Self::CraftBasicMeasure => "Basic Measure",
        Self::CraftBasicCut => "Basic Cut",
        Self::CraftBasicJoin => "Basic Join",
        Self::CraftDrawOutMetal => "Draw Out Metal",
        Self::CraftUpsetMetal => "Upset Metal",
        Self::CraftBasicWeld => "Basic Weld",
        Self::CraftShapeWood => "Shape Wood",
        Self::CraftFinishSurface => "Finish Surface",
        Self::CraftForgeKnife => "Forge Knife",
        Self::CraftForgeToolHead => "Forge Tool Head",
        Self::CraftBuildFurniture => "Build Furniture",
        Self::CraftSewGarment => "Sew Garment",
        Self::CraftForgeSword => "Forge Sword",
        Self::CraftForgeArmor => "Forge Armor",
        Self::CraftBuildStructure => "Build Structure",
        Self::CraftPatternWeld => "Pattern Weld",
        Self::CraftAssessAndExecute => "Assess and Execute",
        Self::CraftForgeMasterwork => "Forge Masterwork",
        Self::CraftInnovativeTechnique => "Innovative Technique",
    }
}
```

**Step 6: Run tests**

Run: `cargo test --lib skills::chunk_id`
Expected: PASS

**Step 7: Add Craft chunk definitions**

Add to `src/skills/definitions.rs` CHUNK_LIBRARY:

```rust
// === CRAFT DOMAIN ===
// Level 1 - Micro-chunks
ChunkDefinition {
    id: ChunkId::CraftBasicHeatCycle,
    name: "Basic Heat Cycle",
    level: 1,
    components: ChunkComponents::Atomic,
    context_requirements: &[],
    prerequisite_chunks: &[],
    base_repetitions: 15,
},
ChunkDefinition {
    id: ChunkId::CraftBasicHammerWork,
    name: "Basic Hammer Work",
    level: 1,
    components: ChunkComponents::Atomic,
    context_requirements: &[],
    prerequisite_chunks: &[],
    base_repetitions: 20,
},
ChunkDefinition {
    id: ChunkId::CraftBasicMeasure,
    name: "Basic Measure",
    level: 1,
    components: ChunkComponents::Atomic,
    context_requirements: &[],
    prerequisite_chunks: &[],
    base_repetitions: 10,
},
ChunkDefinition {
    id: ChunkId::CraftBasicCut,
    name: "Basic Cut",
    level: 1,
    components: ChunkComponents::Atomic,
    context_requirements: &[],
    prerequisite_chunks: &[],
    base_repetitions: 15,
},
ChunkDefinition {
    id: ChunkId::CraftBasicJoin,
    name: "Basic Join",
    level: 1,
    components: ChunkComponents::Atomic,
    context_requirements: &[],
    prerequisite_chunks: &[],
    base_repetitions: 20,
},
// Level 2 - Technique chunks
ChunkDefinition {
    id: ChunkId::CraftDrawOutMetal,
    name: "Draw Out Metal",
    level: 2,
    components: ChunkComponents::Composite(&[
        ChunkId::CraftBasicHeatCycle,
        ChunkId::CraftBasicHammerWork,
    ]),
    context_requirements: &[],
    prerequisite_chunks: &[ChunkId::CraftBasicHeatCycle, ChunkId::CraftBasicHammerWork],
    base_repetitions: 40,
},
ChunkDefinition {
    id: ChunkId::CraftUpsetMetal,
    name: "Upset Metal",
    level: 2,
    components: ChunkComponents::Composite(&[
        ChunkId::CraftBasicHeatCycle,
        ChunkId::CraftBasicHammerWork,
    ]),
    context_requirements: &[],
    prerequisite_chunks: &[ChunkId::CraftBasicHeatCycle, ChunkId::CraftBasicHammerWork],
    base_repetitions: 50,
},
ChunkDefinition {
    id: ChunkId::CraftBasicWeld,
    name: "Basic Weld",
    level: 2,
    components: ChunkComponents::Composite(&[
        ChunkId::CraftBasicHeatCycle,
        ChunkId::CraftBasicHammerWork,
    ]),
    context_requirements: &[],
    prerequisite_chunks: &[ChunkId::CraftBasicHeatCycle, ChunkId::CraftBasicHammerWork],
    base_repetitions: 60,
},
ChunkDefinition {
    id: ChunkId::CraftShapeWood,
    name: "Shape Wood",
    level: 2,
    components: ChunkComponents::Composite(&[
        ChunkId::CraftBasicCut,
        ChunkId::CraftBasicMeasure,
    ]),
    context_requirements: &[],
    prerequisite_chunks: &[ChunkId::CraftBasicCut, ChunkId::CraftBasicMeasure],
    base_repetitions: 35,
},
ChunkDefinition {
    id: ChunkId::CraftFinishSurface,
    name: "Finish Surface",
    level: 2,
    components: ChunkComponents::Composite(&[
        ChunkId::CraftBasicMeasure,
    ]),
    context_requirements: &[],
    prerequisite_chunks: &[ChunkId::CraftBasicMeasure],
    base_repetitions: 30,
},
// Level 3 - Product chunks
ChunkDefinition {
    id: ChunkId::CraftForgeKnife,
    name: "Forge Knife",
    level: 3,
    components: ChunkComponents::Composite(&[
        ChunkId::CraftDrawOutMetal,
        ChunkId::CraftBasicCut,
    ]),
    context_requirements: &[],
    prerequisite_chunks: &[ChunkId::CraftDrawOutMetal, ChunkId::CraftBasicCut],
    base_repetitions: 80,
},
ChunkDefinition {
    id: ChunkId::CraftForgeToolHead,
    name: "Forge Tool Head",
    level: 3,
    components: ChunkComponents::Composite(&[
        ChunkId::CraftUpsetMetal,
        ChunkId::CraftBasicHammerWork,
    ]),
    context_requirements: &[],
    prerequisite_chunks: &[ChunkId::CraftUpsetMetal],
    base_repetitions: 70,
},
ChunkDefinition {
    id: ChunkId::CraftBuildFurniture,
    name: "Build Furniture",
    level: 3,
    components: ChunkComponents::Composite(&[
        ChunkId::CraftShapeWood,
        ChunkId::CraftBasicJoin,
        ChunkId::CraftFinishSurface,
    ]),
    context_requirements: &[],
    prerequisite_chunks: &[ChunkId::CraftShapeWood, ChunkId::CraftBasicJoin],
    base_repetitions: 100,
},
ChunkDefinition {
    id: ChunkId::CraftSewGarment,
    name: "Sew Garment",
    level: 3,
    components: ChunkComponents::Composite(&[
        ChunkId::CraftBasicCut,
        ChunkId::CraftBasicJoin,
        ChunkId::CraftBasicMeasure,
    ]),
    context_requirements: &[],
    prerequisite_chunks: &[ChunkId::CraftBasicCut, ChunkId::CraftBasicJoin, ChunkId::CraftBasicMeasure],
    base_repetitions: 60,
},
// Level 4 - Complex product chunks
ChunkDefinition {
    id: ChunkId::CraftForgeSword,
    name: "Forge Sword",
    level: 4,
    components: ChunkComponents::Composite(&[
        ChunkId::CraftDrawOutMetal,
        ChunkId::CraftForgeKnife,
    ]),
    context_requirements: &[],
    prerequisite_chunks: &[ChunkId::CraftDrawOutMetal, ChunkId::CraftForgeKnife],
    base_repetitions: 150,
},
ChunkDefinition {
    id: ChunkId::CraftForgeArmor,
    name: "Forge Armor",
    level: 4,
    components: ChunkComponents::Composite(&[
        ChunkId::CraftDrawOutMetal,
        ChunkId::CraftBasicWeld,
    ]),
    context_requirements: &[],
    prerequisite_chunks: &[ChunkId::CraftDrawOutMetal, ChunkId::CraftBasicWeld],
    base_repetitions: 200,
},
ChunkDefinition {
    id: ChunkId::CraftBuildStructure,
    name: "Build Structure",
    level: 4,
    components: ChunkComponents::Composite(&[
        ChunkId::CraftBuildFurniture,
        ChunkId::CraftShapeWood,
    ]),
    context_requirements: &[],
    prerequisite_chunks: &[ChunkId::CraftBuildFurniture],
    base_repetitions: 250,
},
ChunkDefinition {
    id: ChunkId::CraftPatternWeld,
    name: "Pattern Weld",
    level: 4,
    components: ChunkComponents::Composite(&[
        ChunkId::CraftBasicWeld,
        ChunkId::CraftDrawOutMetal,
    ]),
    context_requirements: &[],
    prerequisite_chunks: &[ChunkId::CraftBasicWeld, ChunkId::CraftDrawOutMetal],
    base_repetitions: 300,
},
// Level 5 - Mastery chunks
ChunkDefinition {
    id: ChunkId::CraftAssessAndExecute,
    name: "Assess and Execute",
    level: 5,
    components: ChunkComponents::Composite(&[
        ChunkId::CraftForgeKnife,
        ChunkId::CraftForgeSword,
    ]),
    context_requirements: &[],
    prerequisite_chunks: &[ChunkId::CraftForgeSword],
    base_repetitions: 500,
},
ChunkDefinition {
    id: ChunkId::CraftForgeMasterwork,
    name: "Forge Masterwork",
    level: 5,
    components: ChunkComponents::Composite(&[
        ChunkId::CraftForgeSword,
        ChunkId::CraftPatternWeld,
    ]),
    context_requirements: &[],
    prerequisite_chunks: &[ChunkId::CraftForgeSword, ChunkId::CraftPatternWeld],
    base_repetitions: 800,
},
ChunkDefinition {
    id: ChunkId::CraftInnovativeTechnique,
    name: "Innovative Technique",
    level: 5,
    components: ChunkComponents::Composite(&[
        ChunkId::CraftAssessAndExecute,
    ]),
    context_requirements: &[],
    prerequisite_chunks: &[ChunkId::CraftAssessAndExecute],
    base_repetitions: 1000,
},
```

**Step 8: Run full test suite**

Run: `cargo test --lib skills`
Expected: PASS (all tests including new definition tests)

**Step 9: Commit**

```bash
git add src/skills/chunk_id.rs src/skills/definitions.rs
git commit -m "feat(skills): add Craft domain chunks (21 chunks, levels 1-5)"
```

---

## Task 4: Add Social Domain Chunks

**Files:**
- Modify: `src/skills/chunk_id.rs`
- Modify: `src/skills/definitions.rs`

**Step 1: Write the failing test**

Add to `src/skills/chunk_id.rs` tests:

```rust
#[test]
fn test_social_chunks_exist() {
    use crate::skills::ChunkDomain;

    assert_eq!(ChunkId::SocialActiveListening.domain(), ChunkDomain::Social);
    assert_eq!(ChunkId::SocialBuildRapport.domain(), ChunkDomain::Social);
    assert_eq!(ChunkId::SocialNegotiateTerms.domain(), ChunkDomain::Social);
    assert_eq!(ChunkId::SocialWorkRoom.domain(), ChunkDomain::Social);
    assert_eq!(ChunkId::SocialManipulateDynamics.domain(), ChunkDomain::Social);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib skills::chunk_id::tests::test_social_chunks_exist`
Expected: FAIL

**Step 3: Add Social chunk variants**

Add to ChunkId enum:

```rust
// === Social Domain ===
// Level 1 - Micro-chunks
SocialActiveListening,
SocialProjectConfidence,
SocialEmpathicMirror,
SocialCreateTension,
// Level 2 - Technique chunks
SocialBuildRapport,
SocialProjectAuthority,
SocialReadReaction,
SocialDeflectInquiry,
SocialEmotionalAppeal,
// Level 3 - Tactical chunks
SocialNegotiateTerms,
SocialIntimidate,
SocialPersuade,
SocialDeceive,
SocialInspire,
// Level 4 - Strategic chunks
SocialWorkRoom,
SocialPoliticalManeuver,
SocialLeadGroup,
SocialMediateConflict,
// Level 5 - Mastery chunks
SocialOmniscience,
SocialManipulateDynamics,
SocialCultOfPersonality,
```

**Step 4: Update domain(), level(), name() methods**

Add Social domain cases to each match block. Pattern follows Craft domain implementation.

**Step 5: Add Social chunk definitions**

Add to CHUNK_LIBRARY following the same pattern as Craft. Use base_repetitions:
- Level 1: 15-25
- Level 2: 40-60
- Level 3: 80-150
- Level 4: 200-350
- Level 5: 500-1000

**Step 6: Run tests**

Run: `cargo test --lib skills`
Expected: PASS

**Step 7: Commit**

```bash
git add src/skills/chunk_id.rs src/skills/definitions.rs
git commit -m "feat(skills): add Social domain chunks (21 chunks, levels 1-5)"
```

---

## Task 5: Add Medicine Domain Chunks

**Files:**
- Modify: `src/skills/chunk_id.rs`
- Modify: `src/skills/definitions.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_medicine_chunks_exist() {
    use crate::skills::ChunkDomain;

    assert_eq!(ChunkId::MedWoundAssessment.domain(), ChunkDomain::Medicine);
    assert_eq!(ChunkId::MedTreatLaceration.domain(), ChunkDomain::Medicine);
    assert_eq!(ChunkId::MedFieldSurgery.domain(), ChunkDomain::Medicine);
    assert_eq!(ChunkId::MedBattlefieldTriage.domain(), ChunkDomain::Medicine);
    assert_eq!(ChunkId::MedDiagnosticIntuition.domain(), ChunkDomain::Medicine);
}
```

**Step 2-7: Follow same pattern as Social domain**

Add chunks:
```rust
// === Medicine Domain ===
// Level 1
MedWoundAssessment,
MedBasicCleaning,
MedBasicSuture,
MedVitalCheck,
// Level 2
MedTreatLaceration,
MedSetFracture,
MedPreparePoultice,
MedDiagnoseIllness,
MedPainManagement,
// Level 3
MedFieldSurgery,
MedTreatInfection,
MedDeliverBaby,
MedAmputation,
// Level 4
MedBattlefieldTriage,
MedComplexSurgery,
MedEpidemicResponse,
// Level 5
MedDiagnosticIntuition,
MedSurgicalExcellence,
MedHolisticTreatment,
```

**Step 8: Commit**

```bash
git add src/skills/chunk_id.rs src/skills/definitions.rs
git commit -m "feat(skills): add Medicine domain chunks (19 chunks, levels 1-5)"
```

---

## Task 6: Add Leadership Domain Chunks

**Files:**
- Modify: `src/skills/chunk_id.rs`
- Modify: `src/skills/definitions.rs`

**Pattern: Same as previous domains**

Add chunks:
```rust
// === Leadership Domain ===
// Level 1
LeadCommandPresence,
LeadClearOrder,
LeadSituationalRead,
// Level 2
LeadIssueCommand,
LeadAssessUnitState,
LeadDelegateTask,
LeadMaintainCalm,
// Level 3
LeadDirectFormation,
LeadRespondToCrisis,
LeadRallyWavering,
LeadCoordinateUnits,
// Level 4
LeadBattleManagement,
LeadCampaignPlanning,
LeadOrganizationBuilding,
// Level 5
LeadReadBattleFlow,
LeadInspireArmy,
LeadStrategicIntuition,
```

**Commit message:** `feat(skills): add Leadership domain chunks (17 chunks, levels 1-5)`

---

## Task 7: Add Knowledge Domain Chunks

**Files:**
- Modify: `src/skills/chunk_id.rs`
- Modify: `src/skills/definitions.rs`

Add chunks:
```rust
// === Knowledge Domain ===
// Level 1
KnowFluentReading,
KnowFluentWriting,
KnowArithmetic,
KnowMemorization,
// Level 2
KnowResearchSource,
KnowComposeDocument,
KnowMathematicalProof,
KnowTeachConcept,
KnowTranslateText,
// Level 3
KnowAnalyzeText,
KnowSynthesizeSources,
KnowFormalArgument,
KnowInstructStudent,
// Level 4
KnowOriginalResearch,
KnowComprehensiveTreatise,
KnowCurriculumDesign,
// Level 5
KnowParadigmIntegration,
KnowIntellectualLegacy,
```

**Commit message:** `feat(skills): add Knowledge domain chunks (18 chunks, levels 1-5)`

---

## Task 8: Add Physical Domain Chunks

**Files:**
- Modify: `src/skills/chunk_id.rs`
- Modify: `src/skills/definitions.rs`

Add chunks:
```rust
// === Physical Domain ===
// Level 1
PhysEfficientGait,
PhysQuietMovement,
PhysPowerStance,
PhysClimbGrip,
// Level 2
PhysDistanceRunning,
PhysHeavyLifting,
PhysSilentApproach,
PhysRockClimbing,
PhysHorseControl,
// Level 3
PhysSustainedLabor,
PhysInfiltration,
PhysRoughTerrainTravel,
PhysCavalryRiding,
PhysSwimming,
// Level 4
PhysLaborLeadership,
PhysScoutMission,
PhysMountedCombat,
PhysSurvivalTravel,
// Level 5
PhysTirelessEndurance,
PhysShadowMovement,
PhysCentaurUnity,
```

**Commit message:** `feat(skills): add Physical domain chunks (21 chunks, levels 1-5)`

---

## Task 9: Add Phenotype Struct

**Files:**
- Modify: `src/genetics/phenotype.rs:1-2`
- Modify: `src/genetics/mod.rs:1-5`

**Step 1: Write the failing test**

Replace `src/genetics/phenotype.rs`:

```rust
//! Phenotype - physical traits affecting skill ceilings
//!
//! Phenotype provides the physical/cognitive baseline that chunks operate within.
//! A weak entity can become a skilled smith, but their output speed is capped by strength.

use serde::{Deserialize, Serialize};

/// Physical and cognitive traits that affect skill performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phenotype {
    /// Physical strength (0.5-1.5, default 1.0)
    /// Affects: Craft speed ceiling, melee damage, labor capacity
    pub strength: f32,

    /// Physical endurance (0.5-1.5, default 1.0)
    /// Affects: Duration before fatigue, sustained work capacity
    pub endurance: f32,

    /// Fine motor control and speed (0.5-1.5, default 1.0)
    /// Affects: Craft precision ceiling, surgery, ranged combat
    pub agility: f32,

    /// Sensory acuity (0.5-1.5, default 1.0)
    /// Affects: Diagnostic accuracy, social reading, stealth detection
    pub perception: f32,

    /// Voice quality and projection (0.5-1.5, default 1.0)
    /// Affects: All voice-based social chunks, command range
    pub voice_quality: f32,

    /// Cognitive learning speed (0.5-1.5, default 1.0)
    /// Affects: Chunk formation rate multiplier
    pub learning_rate: f32,
}

impl Default for Phenotype {
    fn default() -> Self {
        Self {
            strength: 1.0,
            endurance: 1.0,
            agility: 1.0,
            perception: 1.0,
            voice_quality: 1.0,
            learning_rate: 1.0,
        }
    }
}

impl Phenotype {
    /// Create a phenotype with random variance around defaults
    pub fn with_variance(variance: f32) -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        let vary = |base: f32| -> f32 {
            let delta = rng.gen_range(-variance..=variance);
            (base + delta).clamp(0.5, 1.5)
        };

        Self {
            strength: vary(1.0),
            endurance: vary(1.0),
            agility: vary(1.0),
            perception: vary(1.0),
            voice_quality: vary(1.0),
            learning_rate: vary(1.0),
        }
    }

    /// Get the ceiling for a specific domain based on relevant traits
    pub fn domain_ceiling(&self, domain: crate::skills::ChunkDomain) -> f32 {
        use crate::skills::ChunkDomain;

        match domain {
            ChunkDomain::Combat => (self.strength + self.agility) / 2.0,
            ChunkDomain::Craft => (self.strength.min(self.agility) + self.perception) / 2.0,
            ChunkDomain::Social => (self.voice_quality + self.perception) / 2.0,
            ChunkDomain::Medicine => (self.agility + self.perception) / 2.0,
            ChunkDomain::Leadership => (self.voice_quality + self.perception) / 2.0,
            ChunkDomain::Knowledge => self.perception,
            ChunkDomain::Physical => (self.strength + self.endurance + self.agility) / 3.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::ChunkDomain;

    #[test]
    fn test_default_phenotype() {
        let p = Phenotype::default();
        assert_eq!(p.strength, 1.0);
        assert_eq!(p.learning_rate, 1.0);
    }

    #[test]
    fn test_phenotype_variance() {
        let p = Phenotype::with_variance(0.2);
        assert!(p.strength >= 0.5 && p.strength <= 1.5);
        assert!(p.agility >= 0.5 && p.agility <= 1.5);
    }

    #[test]
    fn test_domain_ceiling() {
        let p = Phenotype {
            strength: 1.2,
            endurance: 1.0,
            agility: 0.8,
            perception: 1.1,
            voice_quality: 1.0,
            learning_rate: 1.0,
        };

        // Combat ceiling = (strength + agility) / 2 = (1.2 + 0.8) / 2 = 1.0
        assert!((p.domain_ceiling(ChunkDomain::Combat) - 1.0).abs() < 0.01);

        // Craft ceiling = (min(strength, agility) + perception) / 2 = (0.8 + 1.1) / 2 = 0.95
        assert!((p.domain_ceiling(ChunkDomain::Craft) - 0.95).abs() < 0.01);
    }
}
```

**Step 2: Run test to verify**

Run: `cargo test --lib genetics::phenotype`
Expected: PASS (3 tests)

**Step 3: Export from mod.rs**

Edit `src/genetics/mod.rs`:

```rust
pub mod genome;
pub mod personality;
pub mod phenotype;
pub mod values;

pub use phenotype::Phenotype;
```

**Step 4: Verify build**

Run: `cargo build --lib`
Expected: Compiles successfully

**Step 5: Commit**

```bash
git add src/genetics/phenotype.rs src/genetics/mod.rs
git commit -m "feat(genetics): implement Phenotype struct with domain ceilings"
```

---

## Task 10: Add Phenotype to HumanArchetype

**Files:**
- Modify: `src/entity/species/human.rs:1-222`

**Step 1: Write the failing test**

Add to `src/entity/species/human.rs` tests:

```rust
#[test]
fn test_human_has_phenotype() {
    let mut archetype = HumanArchetype::new();
    let id = EntityId::new();
    archetype.spawn(id, "Test".into(), 0);

    assert_eq!(archetype.phenotypes.len(), 1);
    assert_eq!(archetype.phenotypes[0].strength, 1.0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib entity::species::human::tests::test_human_has_phenotype`
Expected: FAIL with "no field `phenotypes`"

**Step 3: Add phenotypes field to HumanArchetype**

Add to struct fields:

```rust
use crate::genetics::Phenotype;

pub struct HumanArchetype {
    // ... existing fields ...
    /// Physical/cognitive traits for each entity
    pub phenotypes: Vec<Phenotype>,
}
```

Update `new()`:
```rust
pub fn new() -> Self {
    Self {
        // ... existing fields ...
        phenotypes: Vec::new(),
    }
}
```

Update `spawn()`:
```rust
pub fn spawn(&mut self, id: EntityId, name: String, tick: Tick) {
    // ... existing pushes ...
    self.phenotypes.push(Phenotype::with_variance(0.1)); // 10% variance
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib entity::species::human::tests::test_human_has_phenotype`
Expected: PASS

**Step 5: Commit**

```bash
git add src/entity/species/human.rs
git commit -m "feat(entity): add Phenotype to HumanArchetype"
```

---

## Task 11: Add domain_summary() to ChunkLibrary

**Files:**
- Modify: `src/skills/library.rs:1-294`

**Step 1: Write the failing test**

Add to `src/skills/library.rs` tests:

```rust
use crate::skills::ChunkDomain;

#[test]
fn test_domain_summary_empty() {
    let lib = ChunkLibrary::new();
    let summary = lib.domain_summary(ChunkDomain::Combat);

    assert_eq!(summary.chunk_count, 0);
    assert_eq!(summary.total_encoding, 0.0);
    assert!(summary.best_chunk.is_none());
}

#[test]
fn test_domain_summary_with_chunks() {
    let lib = ChunkLibrary::trained_soldier(1000);
    let summary = lib.domain_summary(ChunkDomain::Combat);

    assert!(summary.chunk_count >= 3);
    assert!(summary.total_encoding > 0.0);
    assert!(summary.best_chunk.is_some());
    assert!(summary.average_encoding() > 0.0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib skills::library::tests::test_domain_summary_empty`
Expected: FAIL with "no method named `domain_summary`"

**Step 3: Add DomainSummary struct and method**

Add to `src/skills/library.rs`:

```rust
use crate::skills::ChunkDomain;

/// Summary of chunks in a specific domain
#[derive(Debug, Clone)]
pub struct DomainSummary {
    pub domain: ChunkDomain,
    pub chunk_count: usize,
    pub total_encoding: f32,
    pub best_chunk: Option<(ChunkId, f32)>,
    pub highest_level: u8,
}

impl DomainSummary {
    /// Average encoding depth (0.0 if no chunks)
    pub fn average_encoding(&self) -> f32 {
        if self.chunk_count == 0 {
            0.0
        } else {
            self.total_encoding / self.chunk_count as f32
        }
    }
}

impl ChunkLibrary {
    /// Get summary of chunks in a specific domain
    pub fn domain_summary(&self, domain: ChunkDomain) -> DomainSummary {
        let domain_chunks: Vec<_> = self.chunks
            .iter()
            .filter(|(id, _)| id.domain() == domain)
            .collect();

        let chunk_count = domain_chunks.len();
        let total_encoding: f32 = domain_chunks.iter()
            .map(|(_, state)| state.encoding_depth)
            .sum();

        let best_chunk = domain_chunks.iter()
            .max_by(|a, b| {
                let score_a = a.0.level() as f32 * 10.0 + a.1.encoding_depth * 5.0;
                let score_b = b.0.level() as f32 * 10.0 + b.1.encoding_depth * 5.0;
                score_a.partial_cmp(&score_b).unwrap()
            })
            .map(|(id, state)| (**id, state.encoding_depth));

        let highest_level = domain_chunks.iter()
            .map(|(id, _)| id.level())
            .max()
            .unwrap_or(0);

        DomainSummary {
            domain,
            chunk_count,
            total_encoding,
            best_chunk,
            highest_level,
        }
    }

    // ... existing methods ...
}
```

**Step 4: Run tests**

Run: `cargo test --lib skills::library::tests::test_domain_summary`
Expected: PASS (both tests)

**Step 5: Export DomainSummary from mod.rs**

```rust
pub use library::{ChunkLibrary, DomainSummary, Experience, PersonalChunkState};
```

**Step 6: Commit**

```bash
git add src/skills/library.rs src/skills/mod.rs
git commit -m "feat(skills): add domain_summary() method to ChunkLibrary"
```

---

## Task 12: Add Display Stats Module

**Files:**
- Create: `src/skills/display.rs`
- Modify: `src/skills/mod.rs`

**Step 1: Write the failing test**

Create `src/skills/display.rs`:

```rust
//! Computed display stats for player UI
//!
//! These stats are computed on-demand from underlying chunks and phenotype.
//! No caching - they're cheap to compute and always current.

use crate::genetics::Phenotype;
use crate::skills::{ChunkDomain, ChunkLibrary};
use serde::{Deserialize, Serialize};

/// Skill level for display purposes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillLevel {
    Untrained,  // No relevant chunks, or only L0
    Novice,     // Some L1 chunks, low encoding
    Trained,    // L2 chunks present, moderate encoding
    Veteran,    // L3 chunks, good encoding
    Expert,     // L4 chunks, high encoding
    Master,     // L5 chunks, near-compiled
    Legend,     // L5 chunks at 0.99+ encoding
}

impl SkillLevel {
    pub fn name(&self) -> &'static str {
        match self {
            SkillLevel::Untrained => "Untrained",
            SkillLevel::Novice => "Novice",
            SkillLevel::Trained => "Trained",
            SkillLevel::Veteran => "Veteran",
            SkillLevel::Expert => "Expert",
            SkillLevel::Master => "Master",
            SkillLevel::Legend => "Legend",
        }
    }
}

/// A displayable skill stat
#[derive(Debug, Clone)]
pub struct DisplayStat {
    pub name: &'static str,
    pub level: SkillLevel,
    pub bar_fill: f32,  // 0.0 to 1.0 for visual bar
}

impl DisplayStat {
    pub fn new(name: &'static str, level: SkillLevel, bar_fill: f32) -> Self {
        Self {
            name,
            level,
            bar_fill: bar_fill.clamp(0.0, 1.0),
        }
    }
}

/// Compute skill level from highest chunk level and average encoding
fn compute_level(highest_chunk_level: u8, avg_encoding: f32) -> SkillLevel {
    match highest_chunk_level {
        0 => SkillLevel::Untrained,
        1 => {
            if avg_encoding < 0.3 { SkillLevel::Untrained }
            else { SkillLevel::Novice }
        }
        2 => {
            if avg_encoding < 0.5 { SkillLevel::Novice }
            else { SkillLevel::Trained }
        }
        3 => {
            if avg_encoding < 0.6 { SkillLevel::Trained }
            else { SkillLevel::Veteran }
        }
        4 => {
            if avg_encoding < 0.7 { SkillLevel::Veteran }
            else { SkillLevel::Expert }
        }
        5 => {
            if avg_encoding >= 0.99 { SkillLevel::Legend }
            else if avg_encoding >= 0.85 { SkillLevel::Master }
            else { SkillLevel::Expert }
        }
        _ => SkillLevel::Legend,
    }
}

/// Compute bar fill from domain summary
fn compute_bar(highest_level: u8, avg_encoding: f32) -> f32 {
    // Bar represents progress: level contributes 0.15 each, encoding fills the rest
    let level_contribution = (highest_level as f32) * 0.15;
    let encoding_contribution = avg_encoding * 0.25;
    (level_contribution + encoding_contribution).clamp(0.0, 1.0)
}

/// Compute craftsmanship stat (Craft domain)
pub fn compute_craftsmanship(library: &ChunkLibrary, phenotype: &Phenotype) -> DisplayStat {
    let summary = library.domain_summary(ChunkDomain::Craft);
    let ceiling = phenotype.domain_ceiling(ChunkDomain::Craft);

    let avg_encoding = summary.average_encoding() * ceiling;
    let level = compute_level(summary.highest_level, avg_encoding);
    let bar = compute_bar(summary.highest_level, avg_encoding);

    DisplayStat::new("Craftsmanship", level, bar)
}

/// Compute medicine stat
pub fn compute_medicine(library: &ChunkLibrary, phenotype: &Phenotype) -> DisplayStat {
    let summary = library.domain_summary(ChunkDomain::Medicine);
    let ceiling = phenotype.domain_ceiling(ChunkDomain::Medicine);

    let avg_encoding = summary.average_encoding() * ceiling;
    let level = compute_level(summary.highest_level, avg_encoding);
    let bar = compute_bar(summary.highest_level, avg_encoding);

    DisplayStat::new("Medicine", level, bar)
}

/// Compute leadership stat
pub fn compute_leadership(library: &ChunkLibrary, phenotype: &Phenotype) -> DisplayStat {
    let summary = library.domain_summary(ChunkDomain::Leadership);
    let ceiling = phenotype.domain_ceiling(ChunkDomain::Leadership);

    let avg_encoding = summary.average_encoding() * ceiling;
    let level = compute_level(summary.highest_level, avg_encoding);
    let bar = compute_bar(summary.highest_level, avg_encoding);

    DisplayStat::new("Leadership", level, bar)
}

/// Compute scholarship stat (Knowledge domain)
pub fn compute_scholarship(library: &ChunkLibrary, phenotype: &Phenotype) -> DisplayStat {
    let summary = library.domain_summary(ChunkDomain::Knowledge);
    let ceiling = phenotype.domain_ceiling(ChunkDomain::Knowledge);

    let avg_encoding = summary.average_encoding() * ceiling;
    let level = compute_level(summary.highest_level, avg_encoding);
    let bar = compute_bar(summary.highest_level, avg_encoding);

    DisplayStat::new("Scholarship", level, bar)
}

/// Compute athleticism stat (Physical domain)
pub fn compute_athleticism(library: &ChunkLibrary, phenotype: &Phenotype) -> DisplayStat {
    let summary = library.domain_summary(ChunkDomain::Physical);
    let ceiling = phenotype.domain_ceiling(ChunkDomain::Physical);

    let avg_encoding = summary.average_encoding() * ceiling;
    let level = compute_level(summary.highest_level, avg_encoding);
    let bar = compute_bar(summary.highest_level, avg_encoding);

    DisplayStat::new("Athleticism", level, bar)
}

/// Compute charisma stat (Social domain + phenotype)
/// Charisma is special: it includes appearance/voice phenotype contributions
pub fn compute_charisma(library: &ChunkLibrary, phenotype: &Phenotype) -> DisplayStat {
    let summary = library.domain_summary(ChunkDomain::Social);

    // Charisma has multiple components
    let chunk_contribution = summary.average_encoding() * 0.5;  // 50% from chunks
    let appearance_contribution = phenotype.voice_quality * 0.3;  // 30% from voice
    let perception_contribution = phenotype.perception * 0.2;  // 20% from perception

    let combined = chunk_contribution + appearance_contribution + perception_contribution;
    let level = compute_level(summary.highest_level, combined);
    let bar = compute_bar(summary.highest_level, combined);

    DisplayStat::new("Charisma", level, bar)
}

/// Compute combat stat
pub fn compute_combat(library: &ChunkLibrary, phenotype: &Phenotype) -> DisplayStat {
    let summary = library.domain_summary(ChunkDomain::Combat);
    let ceiling = phenotype.domain_ceiling(ChunkDomain::Combat);

    let avg_encoding = summary.average_encoding() * ceiling;
    let level = compute_level(summary.highest_level, avg_encoding);
    let bar = compute_bar(summary.highest_level, avg_encoding);

    DisplayStat::new("Combat", level, bar)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_level_names() {
        assert_eq!(SkillLevel::Untrained.name(), "Untrained");
        assert_eq!(SkillLevel::Master.name(), "Master");
    }

    #[test]
    fn test_compute_level() {
        assert_eq!(compute_level(0, 0.0), SkillLevel::Untrained);
        assert_eq!(compute_level(1, 0.5), SkillLevel::Novice);
        assert_eq!(compute_level(3, 0.7), SkillLevel::Veteran);
        assert_eq!(compute_level(5, 0.99), SkillLevel::Legend);
    }

    #[test]
    fn test_compute_craftsmanship_empty() {
        let library = ChunkLibrary::new();
        let phenotype = Phenotype::default();

        let stat = compute_craftsmanship(&library, &phenotype);

        assert_eq!(stat.name, "Craftsmanship");
        assert_eq!(stat.level, SkillLevel::Untrained);
        assert_eq!(stat.bar_fill, 0.0);
    }

    #[test]
    fn test_compute_combat_trained() {
        let library = ChunkLibrary::trained_soldier(1000);
        let phenotype = Phenotype::default();

        let stat = compute_combat(&library, &phenotype);

        assert_eq!(stat.name, "Combat");
        assert!(stat.level == SkillLevel::Novice || stat.level == SkillLevel::Trained);
        assert!(stat.bar_fill > 0.0);
    }
}
```

**Step 2: Run tests**

Run: `cargo test --lib skills::display`
Expected: PASS (4 tests)

**Step 3: Export from mod.rs**

Add to `src/skills/mod.rs`:

```rust
pub mod display;
pub use display::{
    compute_athleticism, compute_charisma, compute_combat, compute_craftsmanship,
    compute_leadership, compute_medicine, compute_scholarship, DisplayStat, SkillLevel,
};
```

**Step 4: Commit**

```bash
git add src/skills/display.rs src/skills/mod.rs
git commit -m "feat(skills): add display stats computation (7 stat types)"
```

---

## Task 13: Add Species Chunk Modifiers

**Files:**
- Create: `src/skills/species_mods.rs`
- Modify: `src/skills/mod.rs`

**Step 1: Create species modifiers struct**

Create `src/skills/species_mods.rs`:

```rust
//! Species-specific chunk formation and decay modifiers
//!
//! Different species learn different domains at different rates.

use crate::skills::ChunkDomain;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Modifiers for chunk formation in a specific domain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainModifier {
    /// Multiplier for chunk formation rate (1.0 = normal)
    pub formation_rate: f32,
    /// Multiplier for rust/decay rate (1.0 = normal, 0.0 = no decay)
    pub decay_rate: f32,
    /// Maximum encoding depth achievable (0.95-0.995)
    pub max_encoding: f32,
}

impl Default for DomainModifier {
    fn default() -> Self {
        Self {
            formation_rate: 1.0,
            decay_rate: 1.0,
            max_encoding: 0.95,
        }
    }
}

/// Species-specific skill modifiers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpeciesChunkModifiers {
    /// Per-domain modifiers
    pub domains: HashMap<ChunkDomain, DomainModifier>,
    /// Global learning rate multiplier
    pub base_learning_rate: f32,
    /// Cross-species social penalty (0.0-1.0, where 1.0 = no penalty)
    pub cross_species_social: f32,
}

impl SpeciesChunkModifiers {
    /// Create default human modifiers (baseline)
    pub fn human() -> Self {
        Self {
            domains: HashMap::new(), // All defaults
            base_learning_rate: 1.0,
            cross_species_social: 0.8, // 20% penalty with non-humans
        }
    }

    /// Create dwarf modifiers (craft-focused)
    pub fn dwarf() -> Self {
        let mut domains = HashMap::new();

        // Dwarves excel at crafting
        domains.insert(ChunkDomain::Craft, DomainModifier {
            formation_rate: 1.5,  // 50% faster craft learning
            decay_rate: 0.3,      // Craft skills barely rust
            max_encoding: 0.99,   // Higher ceiling
        });

        // Good at physical labor
        domains.insert(ChunkDomain::Physical, DomainModifier {
            formation_rate: 1.2,
            decay_rate: 0.5,
            max_encoding: 0.95,
        });

        // Slower socially
        domains.insert(ChunkDomain::Social, DomainModifier {
            formation_rate: 0.8,
            decay_rate: 1.0,
            max_encoding: 0.9,
        });

        Self {
            domains,
            base_learning_rate: 1.0,
            cross_species_social: 0.7, // 30% penalty with non-dwarves
        }
    }

    /// Create elf modifiers (long-lived, slow but deep)
    pub fn elf() -> Self {
        let mut domains = HashMap::new();

        // Elves learn slowly but deeply in all domains
        for domain in ChunkDomain::all() {
            domains.insert(*domain, DomainModifier {
                formation_rate: 0.6,   // Slower learning
                decay_rate: 0.05,      // Almost no rust
                max_encoding: 0.995,   // Higher ceiling
            });
        }

        Self {
            domains,
            base_learning_rate: 0.8,
            cross_species_social: 0.6, // 40% penalty with non-elves
        }
    }

    /// Create orc modifiers (combat-focused)
    pub fn orc() -> Self {
        let mut domains = HashMap::new();

        // Orcs excel at combat
        domains.insert(ChunkDomain::Combat, DomainModifier {
            formation_rate: 1.4,
            decay_rate: 0.6,
            max_encoding: 0.95,
        });

        // Physical strength
        domains.insert(ChunkDomain::Physical, DomainModifier {
            formation_rate: 1.3,
            decay_rate: 0.7,
            max_encoding: 0.95,
        });

        // Weaker at scholarly pursuits
        domains.insert(ChunkDomain::Knowledge, DomainModifier {
            formation_rate: 0.6,
            decay_rate: 1.5,
            max_encoding: 0.8,
        });

        Self {
            domains,
            base_learning_rate: 1.0,
            cross_species_social: 0.6,
        }
    }

    /// Get modifier for a specific domain (defaults if not set)
    pub fn get_domain(&self, domain: ChunkDomain) -> DomainModifier {
        self.domains.get(&domain).cloned().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_human_baseline() {
        let mods = SpeciesChunkModifiers::human();
        let craft = mods.get_domain(ChunkDomain::Craft);

        assert_eq!(craft.formation_rate, 1.0);
        assert_eq!(craft.decay_rate, 1.0);
    }

    #[test]
    fn test_dwarf_craft_bonus() {
        let mods = SpeciesChunkModifiers::dwarf();
        let craft = mods.get_domain(ChunkDomain::Craft);

        assert_eq!(craft.formation_rate, 1.5);
        assert_eq!(craft.decay_rate, 0.3);
    }

    #[test]
    fn test_elf_slow_learning() {
        let mods = SpeciesChunkModifiers::elf();
        let combat = mods.get_domain(ChunkDomain::Combat);

        assert_eq!(combat.formation_rate, 0.6);
        assert_eq!(combat.decay_rate, 0.05);
        assert_eq!(combat.max_encoding, 0.995);
    }
}
```

**Step 2: Run tests**

Run: `cargo test --lib skills::species_mods`
Expected: PASS (3 tests)

**Step 3: Export from mod.rs**

Add to `src/skills/mod.rs`:

```rust
pub mod species_mods;
pub use species_mods::{DomainModifier, SpeciesChunkModifiers};
```

**Step 4: Commit**

```bash
git add src/skills/species_mods.rs src/skills/mod.rs
git commit -m "feat(skills): add species chunk modifiers (human, dwarf, elf, orc)"
```

---

## Task 14: Integrate Species Modifiers into Learning

**Files:**
- Modify: `src/skills/learning.rs:1-213`

**Step 1: Write the failing test**

Add to `src/skills/learning.rs` tests:

```rust
use crate::skills::{ChunkDomain, SpeciesChunkModifiers};

#[test]
fn test_species_modifier_affects_learning() {
    let mut lib = ChunkLibrary::new();
    let dwarf_mods = SpeciesChunkModifiers::dwarf();

    // Add a craft chunk
    lib.set_chunk(ChunkId::CraftBasicHeatCycle, PersonalChunkState {
        encoding_depth: 0.3,
        repetition_count: 30,
        last_used_tick: 0,
        formation_tick: 0,
    });

    // Record successful experience
    lib.record_experience(Experience {
        chunk_id: ChunkId::CraftBasicHeatCycle,
        success: true,
        tick: 100,
    });

    // Process with dwarf modifiers
    process_learning_with_modifiers(&mut lib, 100, &dwarf_mods);

    let depth = lib.get_chunk(ChunkId::CraftBasicHeatCycle).unwrap().encoding_depth;

    // Dwarf should learn faster (1.5x formation rate)
    // At 31 reps with 1.5x rate, effective reps = 46.5
    // Standard depth at 31 reps: ~0.236
    // Dwarf depth should be higher
    assert!(depth > 0.3); // Improved from experience
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib skills::learning::tests::test_species_modifier_affects_learning`
Expected: FAIL with "cannot find function `process_learning_with_modifiers`"

**Step 3: Add process_learning_with_modifiers function**

Add to `src/skills/learning.rs`:

```rust
use crate::skills::SpeciesChunkModifiers;

/// Process learning with species-specific modifiers
pub fn process_learning_with_modifiers(
    library: &mut ChunkLibrary,
    tick: u64,
    modifiers: &SpeciesChunkModifiers,
) {
    // 1. Consolidate experiences with species modifiers
    for exp in library.pending_experiences().to_vec() {
        if let Some(state) = library.get_chunk_mut(exp.chunk_id) {
            if exp.success {
                let domain = exp.chunk_id.domain();
                let domain_mod = modifiers.get_domain(domain);

                state.repetition_count += 1;

                // Apply formation rate modifier
                let effective_reps = (state.repetition_count as f32
                    * domain_mod.formation_rate
                    * modifiers.base_learning_rate) as u32;

                let base_depth = calculate_encoding_depth(effective_reps);
                state.encoding_depth = base_depth.min(domain_mod.max_encoding);
            }
            state.last_used_tick = exp.tick;
        } else {
            check_chunk_formation(library, exp.chunk_id, tick);
        }
    }

    library.clear_experiences();
    check_all_formations(library, tick);
    apply_rust_decay_with_modifiers(library, tick, modifiers);
}

/// Apply rust decay with species-specific modifiers
fn apply_rust_decay_with_modifiers(
    library: &mut ChunkLibrary,
    tick: u64,
    modifiers: &SpeciesChunkModifiers,
) {
    for (chunk_id, state) in library.chunks_mut().iter_mut() {
        let ticks_since_use = tick.saturating_sub(state.last_used_tick);

        if ticks_since_use > RUST_THRESHOLD {
            let domain = chunk_id.domain();
            let domain_mod = modifiers.get_domain(domain);

            let decay_ticks = ticks_since_use - RUST_THRESHOLD;
            let decay = decay_ticks as f32 * RUST_RATE * domain_mod.decay_rate;
            state.encoding_depth = (state.encoding_depth - decay).max(MIN_ENCODING);
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib skills::learning`
Expected: PASS (all tests)

**Step 5: Export new function**

Update `src/skills/mod.rs`:

```rust
pub use learning::{calculate_encoding_depth, process_learning, process_learning_with_modifiers};
```

**Step 6: Commit**

```bash
git add src/skills/learning.rs src/skills/mod.rs
git commit -m "feat(skills): integrate species modifiers into learning system"
```

---

## Task 15: Integration Test - Skill Differentiation

**Files:**
- Create: `tests/skill_differentiation.rs`

**Step 1: Write the integration test**

Create `tests/skill_differentiation.rs`:

```rust
//! Integration test: Skill Differentiation
//!
//! Tests that novice and master entities produce different outcomes
//! when performing the same action.

use arc_citadel::genetics::Phenotype;
use arc_citadel::skills::{
    calculate_encoding_depth, compute_craftsmanship, ChunkId, ChunkLibrary,
    PersonalChunkState, SkillLevel,
};

/// Create a novice crafter (no chunks)
fn create_novice() -> (ChunkLibrary, Phenotype) {
    (ChunkLibrary::new(), Phenotype::default())
}

/// Create a master crafter (deep L5 chunks)
fn create_master(tick: u64) -> (ChunkLibrary, Phenotype) {
    let mut lib = ChunkLibrary::new();

    // Add all craft chunks with deep encoding
    let craft_chunks = [
        (ChunkId::CraftBasicHeatCycle, 500),
        (ChunkId::CraftBasicHammerWork, 500),
        (ChunkId::CraftBasicMeasure, 400),
        (ChunkId::CraftBasicCut, 400),
        (ChunkId::CraftBasicJoin, 400),
        (ChunkId::CraftDrawOutMetal, 300),
        (ChunkId::CraftUpsetMetal, 300),
        (ChunkId::CraftBasicWeld, 250),
        (ChunkId::CraftForgeKnife, 200),
        (ChunkId::CraftForgeToolHead, 200),
        (ChunkId::CraftForgeSword, 150),
        (ChunkId::CraftForgeArmor, 150),
        (ChunkId::CraftPatternWeld, 100),
        (ChunkId::CraftAssessAndExecute, 80),
        (ChunkId::CraftForgeMasterwork, 50),
    ];

    for (chunk_id, reps) in craft_chunks {
        lib.set_chunk(chunk_id, PersonalChunkState {
            encoding_depth: calculate_encoding_depth(reps),
            repetition_count: reps,
            last_used_tick: tick,
            formation_tick: tick.saturating_sub(10000),
        });
    }

    (lib, Phenotype::default())
}

#[test]
fn test_novice_vs_master_display_stats() {
    let (novice_lib, novice_pheno) = create_novice();
    let (master_lib, master_pheno) = create_master(1000);

    let novice_stat = compute_craftsmanship(&novice_lib, &novice_pheno);
    let master_stat = compute_craftsmanship(&master_lib, &master_pheno);

    // Novice should be Untrained
    assert_eq!(novice_stat.level, SkillLevel::Untrained);
    assert_eq!(novice_stat.bar_fill, 0.0);

    // Master should be Expert or Master (has L5 chunks)
    assert!(
        master_stat.level == SkillLevel::Expert || master_stat.level == SkillLevel::Master,
        "Master should be Expert or Master, got {:?}",
        master_stat.level
    );
    assert!(master_stat.bar_fill > 0.5);
}

#[test]
fn test_novice_high_attention_cost() {
    let (novice_lib, _) = create_novice();
    let (master_lib, _) = create_master(1000);

    // Novice has no chunks - would need to use atomics (high cost)
    // Master has deep chunks - low attention cost

    // Check attention cost for a craft action
    // Novice: no chunk = ~1.0 attention cost
    // Master: deep chunk = ~0.1-0.2 attention cost

    let novice_best = novice_lib.domain_summary(arc_citadel::skills::ChunkDomain::Craft);
    let master_best = master_lib.domain_summary(arc_citadel::skills::ChunkDomain::Craft);

    // Novice has no craft chunks
    assert_eq!(novice_best.chunk_count, 0);

    // Master has many craft chunks
    assert!(master_best.chunk_count >= 10);
    assert!(master_best.average_encoding() > 0.7);
}

#[test]
fn test_master_attention_remaining_for_parallel_tasks() {
    let (_, _) = create_novice();
    let (mut master_lib, _) = create_master(1000);

    // Master can execute a craft chunk and still have attention for other things
    master_lib.attention_budget = 1.0;
    master_lib.attention_spent = 0.0;

    // Simulate using a deep chunk (encoding 0.85 = attention cost 0.15)
    let chunk_cost = 1.0 - 0.85; // 0.15
    assert!(master_lib.spend_attention(chunk_cost));

    // Master still has attention for quality assessment, teaching, etc.
    assert!(master_lib.attention_remaining() > 0.8);

    // Novice would need full attention just for basic operations
    // (No chunks = attention cost ~1.0 per atomic action)
}
```

**Step 2: Run test**

Run: `cargo test --test skill_differentiation`
Expected: PASS (3 tests)

**Step 3: Commit**

```bash
git add tests/skill_differentiation.rs
git commit -m "test: add skill differentiation integration tests"
```

---

## Task 16: Final Verification

**Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests PASS

**Step 2: Check for warnings**

Run: `cargo build --lib 2>&1 | grep -i warning`
Expected: No new warnings (or address any that appear)

**Step 3: Verify documentation**

Run: `cargo doc --lib --no-deps`
Expected: Documentation builds successfully

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat(skills): complete non-combat skill system implementation

Adds 6 non-combat skill domains:
- Craft (21 chunks)
- Social (21 chunks)
- Medicine (19 chunks)
- Leadership (17 chunks)
- Knowledge (18 chunks)
- Physical (21 chunks)

New features:
- ChunkDomain enum for categorization
- Phenotype struct with domain ceilings
- Species chunk modifiers (human, dwarf, elf, orc)
- Display stat computation (7 stat types)
- Domain summary for ChunkLibrary

Integration:
- Added Phenotype to HumanArchetype
- Species-aware learning with formation/decay modifiers
- Integration tests for skill differentiation"
```

---

## Summary

**Total tasks:** 16
**Estimated chunks added:** ~117 new ChunkId variants
**New files:** 4 (domain.rs, display.rs, species_mods.rs, skill_differentiation.rs test)
**Modified files:** 7

**Key integration points:**
- `src/skills/chunk_id.rs` - Extended enum with domain() method
- `src/skills/library.rs` - Added domain_summary()
- `src/skills/learning.rs` - Added species-aware learning
- `src/genetics/phenotype.rs` - New Phenotype struct
- `src/entity/species/human.rs` - Added phenotypes Vec

**Testing strategy:**
- Unit tests in each new module
- Integration test validating novice vs master differentiation
- Existing tests should continue to pass (combat chunks unchanged)
