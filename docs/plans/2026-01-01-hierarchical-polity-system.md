# Hierarchical Polity System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a hierarchical polity system where political entities (empires, kingdoms, duchies, counties, baronies) form parent-child relationships, with full character rulers who own diplomatic opinions.

**Architecture:** Polities have a `parent: Option<PolityId>` field creating a tree structure. Rulers are stored separately with `RulerId` and linked to polities via `rulers: Vec<RulerId>`. Opinions belong to rulers (not polities) as `HashMap<PolityId, Opinion>`. Location.controller uses PolityId as source of truth for territory.

**Tech Stack:** Rust, serde for serialization, existing Arc Citadel aggregate module

---

## Phase 1: Core Types

### Task 1.1: Add PolityId Newtype

**Files:**
- Modify: `src/core/types.rs`
- Test: `src/core/types.rs` (inline test)

**Step 1: Write the failing test**

Add at the bottom of `src/core/types.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polity_id_equality() {
        let a = PolityId(1);
        let b = PolityId(1);
        let c = PolityId(2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn test_polity_id_hash() {
        use std::collections::HashMap;
        let mut map: HashMap<PolityId, &str> = HashMap::new();
        map.insert(PolityId(1), "empire");
        assert_eq!(map.get(&PolityId(1)), Some(&"empire"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib test_polity_id -- --nocapture`
Expected: FAIL with "cannot find value `PolityId`"

**Step 3: Write minimal implementation**

Add before the tests in `src/core/types.rs`:

```rust
/// Unique identifier for polities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PolityId(pub u32);

impl PolityId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib test_polity_id -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/core/types.rs
git commit -m "feat(core): add PolityId newtype"
```

---

### Task 1.2: Add RulerId Newtype

**Files:**
- Modify: `src/core/types.rs`

**Step 1: Write the failing test**

Add to the tests module in `src/core/types.rs`:

```rust
    #[test]
    fn test_ruler_id_equality() {
        let a = RulerId(1);
        let b = RulerId(1);
        let c = RulerId(2);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib test_ruler_id -- --nocapture`
Expected: FAIL with "cannot find value `RulerId`"

**Step 3: Write minimal implementation**

Add after PolityId in `src/core/types.rs`:

```rust
/// Unique identifier for rulers (characters who lead polities)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RulerId(pub u32);

impl RulerId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib test_ruler_id -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/core/types.rs
git commit -m "feat(core): add RulerId newtype"
```

---

### Task 1.3: Add PolityTier Enum

**Files:**
- Modify: `src/core/types.rs`

**Step 1: Write the failing test**

Add to the tests module:

```rust
    #[test]
    fn test_polity_tier_ordering() {
        // Empire > Kingdom > Duchy > County > Barony
        assert!(PolityTier::Empire as u8 > PolityTier::Kingdom as u8);
        assert!(PolityTier::Kingdom as u8 > PolityTier::Duchy as u8);
        assert!(PolityTier::Duchy as u8 > PolityTier::County as u8);
        assert!(PolityTier::County as u8 > PolityTier::Barony as u8);
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib test_polity_tier -- --nocapture`
Expected: FAIL with "cannot find type `PolityTier`"

**Step 3: Write minimal implementation**

Add after RulerId:

```rust
/// Hierarchy tier for polities (political rank, not cultural type)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum PolityTier {
    Barony = 1,
    County = 2,
    Duchy = 3,
    Kingdom = 4,
    Empire = 5,
}

impl PolityTier {
    /// Returns true if this tier outranks the other
    pub fn outranks(&self, other: &PolityTier) -> bool {
        (*self as u8) > (*other as u8)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib test_polity_tier -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/core/types.rs
git commit -m "feat(core): add PolityTier enum with hierarchy ordering"
```

---

### Task 1.4: Add GovernmentType Enum

**Files:**
- Modify: `src/core/types.rs`

**Step 1: Write the failing test**

Add to the tests module:

```rust
    #[test]
    fn test_government_type() {
        let autocracy = GovernmentType::Autocracy;
        let council = GovernmentType::Council;
        assert_ne!(autocracy, council);
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib test_government_type -- --nocapture`
Expected: FAIL with "cannot find type `GovernmentType`"

**Step 3: Write minimal implementation**

Add after PolityTier:

```rust
/// Type of government (affects decision-making)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum GovernmentType {
    #[default]
    Autocracy,  // Single ruler makes decisions
    Council,    // Multiple rulers vote on decisions
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib test_government_type -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/core/types.rs
git commit -m "feat(core): add GovernmentType enum"
```

---

## Phase 2: Ruler Aggregate

### Task 2.1: Create Ruler Module Structure

**Files:**
- Create: `src/aggregate/ruler.rs`
- Modify: `src/aggregate/mod.rs`

**Step 1: Create the module file**

Create `src/aggregate/ruler.rs`:

```rust
//! Ruler - characters who lead polities
//!
//! Rulers are the decision-makers in the aggregate simulation.
//! They have personalities, skills, opinions, and family relationships.
//! Opinions of other polities belong to rulers, not to polities.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::core::types::{PolityId, RulerId, Species};

// Placeholder - will be filled in next tasks
```

**Step 2: Add to mod.rs**

In `src/aggregate/mod.rs`, add the module declaration after the existing ones:

```rust
pub mod ruler;
```

And add to the pub use section:

```rust
pub use ruler::Ruler;
```

**Step 3: Verify it compiles**

Run: `cargo build --lib`
Expected: Warning about unused import, but no errors

**Step 4: Commit**

```bash
git add src/aggregate/ruler.rs src/aggregate/mod.rs
git commit -m "feat(aggregate): add ruler module structure"
```

---

### Task 2.2: Add PersonalityTrait Enum

**Files:**
- Modify: `src/aggregate/ruler.rs`

**Step 1: Write the failing test**

Add at the bottom of `src/aggregate/ruler.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_trait_affects_behavior() {
        let ambitious = PersonalityTrait::Ambitious;
        let cautious = PersonalityTrait::Cautious;

        // Ambitious increases war likelihood
        assert!(ambitious.war_modifier() > 0);
        // Cautious decreases war likelihood
        assert!(cautious.war_modifier() < 0);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib aggregate::ruler::tests::test_personality_trait -- --nocapture`
Expected: FAIL with "cannot find type `PersonalityTrait`"

**Step 3: Write minimal implementation**

Add to `src/aggregate/ruler.rs`:

```rust
/// Personality traits that affect ruler behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PersonalityTrait {
    Ambitious,    // More likely to expand, claim titles
    Cautious,     // Less likely to declare war
    Charismatic,  // Better diplomacy
    Deceitful,    // More likely to break agreements
    Honorable,    // Keeps alliances, less likely to betray
    Warlike,      // More likely to declare war
    Greedy,       // Focus on economy
    Zealous,      // Religious/ideological focus
}

impl PersonalityTrait {
    /// Modifier to war declaration likelihood (-10 to +10)
    pub fn war_modifier(&self) -> i8 {
        match self {
            Self::Ambitious => 3,
            Self::Cautious => -5,
            Self::Charismatic => 0,
            Self::Deceitful => 1,
            Self::Honorable => -2,
            Self::Warlike => 8,
            Self::Greedy => -1,
            Self::Zealous => 4,
        }
    }

    /// Modifier to diplomatic opinion formation (-10 to +10)
    pub fn diplomacy_modifier(&self) -> i8 {
        match self {
            Self::Ambitious => -2,
            Self::Cautious => 1,
            Self::Charismatic => 5,
            Self::Deceitful => -3,
            Self::Honorable => 3,
            Self::Warlike => -4,
            Self::Greedy => 0,
            Self::Zealous => -2,
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib aggregate::ruler::tests::test_personality_trait -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/aggregate/ruler.rs
git commit -m "feat(aggregate): add PersonalityTrait enum with behavior modifiers"
```

---

### Task 2.3: Add Skills Struct

**Files:**
- Modify: `src/aggregate/ruler.rs`

**Step 1: Write the failing test**

Add to the tests module:

```rust
    #[test]
    fn test_skills_default() {
        let skills = Skills::default();
        assert_eq!(skills.diplomacy, 0);
        assert_eq!(skills.martial, 0);
        assert_eq!(skills.stewardship, 0);
        assert_eq!(skills.intrigue, 0);
    }

    #[test]
    fn test_skills_clamped() {
        let skills = Skills::new(15, -15, 5, -5);
        // Values should be clamped to -10..=10
        assert_eq!(skills.diplomacy, 10);
        assert_eq!(skills.martial, -10);
        assert_eq!(skills.stewardship, 5);
        assert_eq!(skills.intrigue, -5);
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib aggregate::ruler::tests::test_skills -- --nocapture`
Expected: FAIL with "cannot find type `Skills`"

**Step 3: Write minimal implementation**

Add to `src/aggregate/ruler.rs`:

```rust
/// Ruler skills affecting governance and war
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Skills {
    pub diplomacy: i8,    // -10 to +10: affects opinion formation, alliance success
    pub martial: i8,      // -10 to +10: affects military effectiveness
    pub stewardship: i8,  // -10 to +10: affects economic growth
    pub intrigue: i8,     // -10 to +10: affects espionage, plot success
}

impl Skills {
    /// Create new skills, clamping values to valid range
    pub fn new(diplomacy: i8, martial: i8, stewardship: i8, intrigue: i8) -> Self {
        Self {
            diplomacy: diplomacy.clamp(-10, 10),
            martial: martial.clamp(-10, 10),
            stewardship: stewardship.clamp(-10, 10),
            intrigue: intrigue.clamp(-10, 10),
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib aggregate::ruler::tests::test_skills -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/aggregate/ruler.rs
git commit -m "feat(aggregate): add Skills struct with clamping"
```

---

### Task 2.4: Add Opinion Struct

**Files:**
- Modify: `src/aggregate/ruler.rs`

**Step 1: Write the failing test**

Add to the tests module:

```rust
    #[test]
    fn test_opinion_effective_value() {
        let mut opinion = Opinion::new(-20);
        assert_eq!(opinion.effective_value(), -20);

        // Add a positive modifier
        opinion.add_modifier("trade_agreement", 15, 10);
        assert_eq!(opinion.effective_value(), -5); // -20 + 15 = -5
    }

    #[test]
    fn test_opinion_decay_modifiers() {
        let mut opinion = Opinion::new(0);
        opinion.add_modifier("recent_gift", 10, 2);

        opinion.decay_modifiers();
        assert_eq!(opinion.modifiers.len(), 1);
        assert_eq!(opinion.modifiers[0].turns_remaining, 1);

        opinion.decay_modifiers();
        assert_eq!(opinion.modifiers.len(), 0); // Expired
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib aggregate::ruler::tests::test_opinion -- --nocapture`
Expected: FAIL with "cannot find type `Opinion`"

**Step 3: Write minimal implementation**

Add to `src/aggregate/ruler.rs`:

```rust
/// Opinion of a ruler toward another polity
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Opinion {
    pub base_value: i16,                  // -100 to +100
    pub trust: i8,                        // -10 to +10 (separate from liking)
    pub modifiers: Vec<OpinionModifier>,  // Temporary modifiers
}

/// Temporary modifier to opinion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpinionModifier {
    pub reason: String,
    pub value: i8,           // -50 to +50
    pub turns_remaining: u8,
}

impl Opinion {
    pub fn new(base_value: i16) -> Self {
        Self {
            base_value: base_value.clamp(-100, 100),
            trust: 0,
            modifiers: Vec::new(),
        }
    }

    /// Calculate effective opinion including all modifiers
    pub fn effective_value(&self) -> i16 {
        let modifier_sum: i16 = self.modifiers.iter().map(|m| m.value as i16).sum();
        (self.base_value + modifier_sum).clamp(-100, 100)
    }

    /// Add a temporary modifier
    pub fn add_modifier(&mut self, reason: &str, value: i8, turns: u8) {
        self.modifiers.push(OpinionModifier {
            reason: reason.to_string(),
            value,
            turns_remaining: turns,
        });
    }

    /// Decay modifiers by one turn, removing expired ones
    pub fn decay_modifiers(&mut self) {
        for modifier in &mut self.modifiers {
            modifier.turns_remaining = modifier.turns_remaining.saturating_sub(1);
        }
        self.modifiers.retain(|m| m.turns_remaining > 0);
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib aggregate::ruler::tests::test_opinion -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/aggregate/ruler.rs
git commit -m "feat(aggregate): add Opinion struct with modifiers and decay"
```

---

### Task 2.5: Add Family Struct

**Files:**
- Modify: `src/aggregate/ruler.rs`

**Step 1: Write the failing test**

Add to the tests module:

```rust
    #[test]
    fn test_family_founder() {
        let family = Family::founder(1);
        assert!(family.father.is_none());
        assert!(family.mother.is_none());
        assert!(family.spouse.is_none());
        assert!(family.children.is_empty());
        assert_eq!(family.dynasty_id, 1);
    }

    #[test]
    fn test_family_add_child() {
        let mut family = Family::founder(1);
        family.add_child(RulerId(2));
        family.add_child(RulerId(3));
        assert_eq!(family.children.len(), 2);
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib aggregate::ruler::tests::test_family -- --nocapture`
Expected: FAIL with "cannot find type `Family`"

**Step 3: Write minimal implementation**

Add to `src/aggregate/ruler.rs`:

```rust
/// Family relationships for a ruler
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Family {
    pub father: Option<RulerId>,
    pub mother: Option<RulerId>,
    pub spouse: Option<RulerId>,
    pub children: Vec<RulerId>,
    pub dynasty_id: u32,  // For dynasty-wide mechanics
}

impl Family {
    /// Create a family for a dynasty founder (no parents)
    pub fn founder(dynasty_id: u32) -> Self {
        Self {
            father: None,
            mother: None,
            spouse: None,
            children: Vec::new(),
            dynasty_id,
        }
    }

    /// Create a family with known parents
    pub fn with_parents(father: RulerId, mother: RulerId, dynasty_id: u32) -> Self {
        Self {
            father: Some(father),
            mother: Some(mother),
            spouse: None,
            children: Vec::new(),
            dynasty_id,
        }
    }

    pub fn add_child(&mut self, child: RulerId) {
        if !self.children.contains(&child) {
            self.children.push(child);
        }
    }

    pub fn set_spouse(&mut self, spouse: RulerId) {
        self.spouse = Some(spouse);
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib aggregate::ruler::tests::test_family -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/aggregate/ruler.rs
git commit -m "feat(aggregate): add Family struct for dynasty tracking"
```

---

### Task 2.6: Add Complete Ruler Struct

**Files:**
- Modify: `src/aggregate/ruler.rs`

**Step 1: Write the failing test**

Add to the tests module:

```rust
    #[test]
    fn test_ruler_creation() {
        let ruler = Ruler::new(
            RulerId(1),
            "King Aldric".to_string(),
            Species::Human,
            42,
            vec![PersonalityTrait::Ambitious, PersonalityTrait::Warlike],
            Skills::new(3, 8, 2, -1),
            Family::founder(1),
        );

        assert_eq!(ruler.id, RulerId(1));
        assert_eq!(ruler.name, "King Aldric");
        assert_eq!(ruler.age, 42);
        assert_eq!(ruler.health, 5); // Default healthy
        assert!(ruler.opinions.is_empty()); // No opinions yet
        assert!(ruler.claims.is_empty());
    }

    #[test]
    fn test_ruler_get_opinion() {
        let mut ruler = Ruler::new(
            RulerId(1),
            "King Aldric".to_string(),
            Species::Human,
            42,
            vec![PersonalityTrait::Honorable],
            Skills::default(),
            Family::founder(1),
        );

        // No opinion yet
        assert!(ruler.get_opinion(PolityId(2)).is_none());

        // Set opinion
        ruler.set_opinion(PolityId(2), Opinion::new(50));
        assert_eq!(ruler.get_opinion(PolityId(2)).unwrap().base_value, 50);
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib aggregate::ruler::tests::test_ruler -- --nocapture`
Expected: FAIL with "cannot find type `Ruler`"

**Step 3: Write minimal implementation**

Add to `src/aggregate/ruler.rs`:

```rust
/// A ruler (character who leads a polity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ruler {
    pub id: RulerId,
    pub name: String,
    pub species: Species,
    pub age: u8,
    pub health: i8,                           // -10 (dying) to +10 (excellent)
    pub personality: Vec<PersonalityTrait>,   // Usually 2-3 traits
    pub skills: Skills,
    pub claims: Vec<PolityId>,                // Polities this ruler claims to rule
    pub family: Family,
    pub opinions: HashMap<PolityId, Opinion>, // Sparse - only known polities
    pub alive: bool,
}

impl Ruler {
    pub fn new(
        id: RulerId,
        name: String,
        species: Species,
        age: u8,
        personality: Vec<PersonalityTrait>,
        skills: Skills,
        family: Family,
    ) -> Self {
        Self {
            id,
            name,
            species,
            age,
            health: 5, // Default healthy
            personality,
            skills,
            claims: Vec::new(),
            family,
            opinions: HashMap::new(),
            alive: true,
        }
    }

    /// Get opinion of another polity (if any)
    pub fn get_opinion(&self, polity: PolityId) -> Option<&Opinion> {
        self.opinions.get(&polity)
    }

    /// Get mutable opinion of another polity (if any)
    pub fn get_opinion_mut(&mut self, polity: PolityId) -> Option<&mut Opinion> {
        self.opinions.get_mut(&polity)
    }

    /// Set or create opinion of another polity
    pub fn set_opinion(&mut self, polity: PolityId, opinion: Opinion) {
        self.opinions.insert(polity, opinion);
    }

    /// Calculate total war modifier from personality
    pub fn war_modifier(&self) -> i8 {
        self.personality.iter().map(|t| t.war_modifier()).sum()
    }

    /// Calculate total diplomacy modifier from personality and skills
    pub fn diplomacy_modifier(&self) -> i8 {
        let personality: i8 = self.personality.iter().map(|t| t.diplomacy_modifier()).sum();
        personality.saturating_add(self.skills.diplomacy)
    }

    /// Add a claim to a polity
    pub fn add_claim(&mut self, polity: PolityId) {
        if !self.claims.contains(&polity) {
            self.claims.push(polity);
        }
    }

    /// Check if ruler has a claim to a polity
    pub fn has_claim(&self, polity: PolityId) -> bool {
        self.claims.contains(&polity)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib aggregate::ruler::tests::test_ruler -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/aggregate/ruler.rs
git commit -m "feat(aggregate): add complete Ruler struct"
```

---

## Phase 3: Update Polity Struct

### Task 3.1: Add New Fields to Polity

**Files:**
- Modify: `src/aggregate/polity.rs`
- Modify: `src/core/types.rs` (if CouncilRole needed)

**Step 1: Write the failing test**

Add to tests in `src/aggregate/polity.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{PolityId, RulerId, PolityTier, GovernmentType};

    #[test]
    fn test_polity_has_new_fields() {
        let polity = Polity {
            id: PolityId(1),
            name: "Kingdom of Aldoria".to_string(),
            species: Species::Human,
            polity_type: PolityType::Kingdom,
            tier: PolityTier::Kingdom,
            government: GovernmentType::Autocracy,
            parent: None, // Sovereign
            rulers: vec![RulerId(1)],
            council_roles: HashMap::new(),
            capital: 0,
            population: 10000,
            military_strength: 100.0,
            economic_strength: 100.0,
            cultural_drift: CulturalDrift::default(),
            relations: HashMap::new(),
            species_state: SpeciesState::Human(HumanState::default()),
            alive: true,
        };

        assert!(polity.is_sovereign());
        assert_eq!(polity.tier, PolityTier::Kingdom);
        assert_eq!(polity.rulers.len(), 1);
    }

    #[test]
    fn test_polity_is_vassal() {
        let polity = Polity {
            id: PolityId(2),
            name: "Duchy of Valheim".to_string(),
            species: Species::Human,
            polity_type: PolityType::Kingdom, // Cultural type
            tier: PolityTier::Duchy,          // Hierarchy rank
            government: GovernmentType::Autocracy,
            parent: Some(PolityId(1)),        // Vassal of polity 1
            rulers: vec![RulerId(2)],
            council_roles: HashMap::new(),
            capital: 1,
            population: 5000,
            military_strength: 50.0,
            economic_strength: 50.0,
            cultural_drift: CulturalDrift::default(),
            relations: HashMap::new(),
            species_state: SpeciesState::Human(HumanState::default()),
            alive: true,
        };

        assert!(!polity.is_sovereign());
        assert_eq!(polity.parent, Some(PolityId(1)));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib aggregate::polity::tests -- --nocapture`
Expected: FAIL with missing fields/methods

**Step 3: Update Polity struct**

Modify `src/aggregate/polity.rs`:

First, add imports at the top:
```rust
use crate::core::types::{PolityId, RulerId, PolityTier, GovernmentType};
```

Then update the Polity struct (replace id: u32 with id: PolityId, remove territory, add new fields):

```rust
/// A polity (nation, tribe, hold, grove, etc.)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Polity {
    pub id: PolityId,
    pub name: String,
    pub species: Species,
    pub polity_type: PolityType,

    // NEW: Hierarchy fields
    pub tier: PolityTier,
    pub government: GovernmentType,
    pub parent: Option<PolityId>,  // None = sovereign
    pub rulers: Vec<RulerId>,       // len=1 for autocracy, len=N for council
    pub council_roles: HashMap<CouncilRole, RulerId>,

    // Physical state (territory removed - Location.controller is source of truth)
    pub population: u32,
    pub capital: u32,  // Region ID
    pub military_strength: f32,
    pub economic_strength: f32,

    // Cultural drift from species baseline
    pub cultural_drift: CulturalDrift,

    // Relations with other polities (treaties, not opinions)
    pub relations: HashMap<u32, Relation>,

    // Species-specific state
    pub species_state: SpeciesState,

    // Alive status
    pub alive: bool,
}

/// Council roles for government
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CouncilRole {
    Chancellor,   // Diplomacy
    Marshal,      // Military
    Steward,      // Economy
    Spymaster,    // Intrigue
}

impl Polity {
    /// Returns true if this polity has no parent (is sovereign)
    pub fn is_sovereign(&self) -> bool {
        self.parent.is_none()
    }

    /// Get the liege (immediate parent) if any
    pub fn liege(&self) -> Option<PolityId> {
        self.parent
    }

    /// Get the primary ruler (first in rulers list)
    pub fn primary_ruler(&self) -> Option<RulerId> {
        self.rulers.first().copied()
    }

    /// Check if a ruler is part of this polity's leadership
    pub fn has_ruler(&self, ruler: RulerId) -> bool {
        self.rulers.contains(&ruler)
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib aggregate::polity::tests -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/aggregate/polity.rs
git commit -m "feat(aggregate): update Polity with hierarchy fields, remove territory"
```

---

### Task 3.2: Remove TreatyTerms::Vassalage

**Files:**
- Modify: `src/aggregate/polity.rs`

**Step 1: Identify current usage**

Search for Vassalage references:
```bash
grep -r "Vassalage" src/
```

**Step 2: Remove the variant**

In `src/aggregate/polity.rs`, update TreatyTerms enum to remove Vassalage:

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TreatyTerms {
    Peace,
    Trade,
    MilitaryAccess,
    Tribute { from: u32, to: u32, amount: u32 },
    // Removed: Vassalage - now represented by Polity.parent field
}
```

**Step 3: Fix any compilation errors**

Run: `cargo build --lib`
Fix any code that referenced TreatyTerms::Vassalage

**Step 4: Commit**

```bash
git add src/aggregate/polity.rs
git commit -m "refactor(aggregate): remove TreatyTerms::Vassalage (replaced by parent field)"
```

---

## Phase 4: Hierarchy Operations

### Task 4.1: Create Hierarchy Module

**Files:**
- Create: `src/aggregate/hierarchy.rs`
- Modify: `src/aggregate/mod.rs`

**Step 1: Create module and add to mod.rs**

Create `src/aggregate/hierarchy.rs`:

```rust
//! Hierarchy operations for polity parent-child relationships
//!
//! Provides queries for traversing the polity hierarchy tree.
//! All polities form a forest (collection of trees) where each tree
//! is rooted at a sovereign polity.

use std::collections::HashMap;
use crate::core::types::PolityId;
use crate::aggregate::polity::Polity;
```

Add to `src/aggregate/mod.rs`:
```rust
pub mod hierarchy;
pub use hierarchy::*;
```

**Step 2: Commit**

```bash
git add src/aggregate/hierarchy.rs src/aggregate/mod.rs
git commit -m "feat(aggregate): add hierarchy module structure"
```

---

### Task 4.2: Implement get_sovereign

**Files:**
- Modify: `src/aggregate/hierarchy.rs`

**Step 1: Write the failing test**

Add to `src/aggregate/hierarchy.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{PolityTier, GovernmentType, RulerId};
    use crate::aggregate::polity::{PolityType, CulturalDrift, SpeciesState, HumanState};
    use crate::core::types::Species;

    fn make_polity(id: u32, parent: Option<u32>) -> Polity {
        Polity {
            id: PolityId(id),
            name: format!("Polity {}", id),
            species: Species::Human,
            polity_type: PolityType::Kingdom,
            tier: PolityTier::Kingdom,
            government: GovernmentType::Autocracy,
            parent: parent.map(PolityId),
            rulers: vec![RulerId(id)],
            council_roles: HashMap::new(),
            capital: 0,
            population: 1000,
            military_strength: 10.0,
            economic_strength: 10.0,
            cultural_drift: CulturalDrift::default(),
            relations: HashMap::new(),
            species_state: SpeciesState::Human(HumanState::default()),
            alive: true,
        }
    }

    fn make_polity_map(polities: Vec<Polity>) -> HashMap<PolityId, Polity> {
        polities.into_iter().map(|p| (p.id, p)).collect()
    }

    #[test]
    fn test_get_sovereign_self() {
        // Sovereign polity returns itself
        let polities = make_polity_map(vec![make_polity(1, None)]);
        assert_eq!(get_sovereign(PolityId(1), &polities), Some(PolityId(1)));
    }

    #[test]
    fn test_get_sovereign_chain() {
        // Empire(1) -> Kingdom(2) -> Duchy(3)
        let polities = make_polity_map(vec![
            make_polity(1, None),      // Empire (sovereign)
            make_polity(2, Some(1)),   // Kingdom under Empire
            make_polity(3, Some(2)),   // Duchy under Kingdom
        ]);

        assert_eq!(get_sovereign(PolityId(1), &polities), Some(PolityId(1)));
        assert_eq!(get_sovereign(PolityId(2), &polities), Some(PolityId(1)));
        assert_eq!(get_sovereign(PolityId(3), &polities), Some(PolityId(1)));
    }

    #[test]
    fn test_get_sovereign_missing() {
        let polities = make_polity_map(vec![make_polity(1, None)]);
        assert_eq!(get_sovereign(PolityId(999), &polities), None);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib aggregate::hierarchy::tests::test_get_sovereign -- --nocapture`
Expected: FAIL with "cannot find function `get_sovereign`"

**Step 3: Write minimal implementation**

Add to `src/aggregate/hierarchy.rs`:

```rust
/// Get the sovereign (root) polity for a given polity.
/// Returns None if the polity doesn't exist.
/// A sovereign polity returns itself.
pub fn get_sovereign(polity_id: PolityId, polities: &HashMap<PolityId, Polity>) -> Option<PolityId> {
    let mut current = polity_id;
    let mut visited = std::collections::HashSet::new();

    loop {
        // Prevent infinite loops from corrupted data
        if !visited.insert(current) {
            return None; // Cycle detected
        }

        let polity = polities.get(&current)?;

        match polity.parent {
            None => return Some(current), // Found sovereign
            Some(parent_id) => current = parent_id,
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib aggregate::hierarchy::tests::test_get_sovereign -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/aggregate/hierarchy.rs
git commit -m "feat(aggregate): implement get_sovereign with cycle detection"
```

---

### Task 4.3: Implement get_vassals and get_all_vassals

**Files:**
- Modify: `src/aggregate/hierarchy.rs`

**Step 1: Write the failing tests**

Add to tests module:

```rust
    #[test]
    fn test_get_vassals_direct() {
        // Empire(1) has vassals Kingdom(2) and Kingdom(3)
        let polities = make_polity_map(vec![
            make_polity(1, None),
            make_polity(2, Some(1)),
            make_polity(3, Some(1)),
            make_polity(4, Some(2)), // Vassal of 2, not 1
        ]);

        let vassals = get_vassals(PolityId(1), &polities);
        assert_eq!(vassals.len(), 2);
        assert!(vassals.contains(&PolityId(2)));
        assert!(vassals.contains(&PolityId(3)));
        assert!(!vassals.contains(&PolityId(4))); // Not direct vassal
    }

    #[test]
    fn test_get_all_vassals_recursive() {
        // Empire(1) -> Kingdom(2) -> Duchy(4)
        //           -> Kingdom(3)
        let polities = make_polity_map(vec![
            make_polity(1, None),
            make_polity(2, Some(1)),
            make_polity(3, Some(1)),
            make_polity(4, Some(2)),
        ]);

        let all_vassals = get_all_vassals(PolityId(1), &polities);
        assert_eq!(all_vassals.len(), 3);
        assert!(all_vassals.contains(&PolityId(2)));
        assert!(all_vassals.contains(&PolityId(3)));
        assert!(all_vassals.contains(&PolityId(4)));
    }

    #[test]
    fn test_get_vassals_none() {
        let polities = make_polity_map(vec![make_polity(1, None)]);
        let vassals = get_vassals(PolityId(1), &polities);
        assert!(vassals.is_empty());
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib aggregate::hierarchy::tests::test_get_vassals -- --nocapture`
Expected: FAIL with "cannot find function `get_vassals`"

**Step 3: Write minimal implementation**

Add to `src/aggregate/hierarchy.rs`:

```rust
/// Get direct vassals of a polity (immediate children only)
pub fn get_vassals(polity_id: PolityId, polities: &HashMap<PolityId, Polity>) -> Vec<PolityId> {
    polities
        .values()
        .filter(|p| p.parent == Some(polity_id))
        .map(|p| p.id)
        .collect()
}

/// Get all vassals recursively (all descendants)
pub fn get_all_vassals(polity_id: PolityId, polities: &HashMap<PolityId, Polity>) -> Vec<PolityId> {
    let mut result = Vec::new();
    let mut stack = get_vassals(polity_id, polities);

    while let Some(vassal_id) = stack.pop() {
        result.push(vassal_id);
        stack.extend(get_vassals(vassal_id, polities));
    }

    result
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib aggregate::hierarchy::tests::test_get_vassals -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/aggregate/hierarchy.rs
git commit -m "feat(aggregate): implement get_vassals and get_all_vassals"
```

---

### Task 4.4: Implement is_vassal_of and same_realm

**Files:**
- Modify: `src/aggregate/hierarchy.rs`

**Step 1: Write the failing tests**

Add to tests module:

```rust
    #[test]
    fn test_is_vassal_of_direct() {
        let polities = make_polity_map(vec![
            make_polity(1, None),
            make_polity(2, Some(1)),
        ]);

        assert!(is_vassal_of(PolityId(2), PolityId(1), &polities));
        assert!(!is_vassal_of(PolityId(1), PolityId(2), &polities));
    }

    #[test]
    fn test_is_vassal_of_indirect() {
        // Empire(1) -> Kingdom(2) -> Duchy(3)
        let polities = make_polity_map(vec![
            make_polity(1, None),
            make_polity(2, Some(1)),
            make_polity(3, Some(2)),
        ]);

        // Duchy is vassal of both Kingdom and Empire
        assert!(is_vassal_of(PolityId(3), PolityId(2), &polities));
        assert!(is_vassal_of(PolityId(3), PolityId(1), &polities));
    }

    #[test]
    fn test_same_realm() {
        // Empire(1) -> Kingdom(2) -> Duchy(3)
        //           -> Kingdom(4)
        let polities = make_polity_map(vec![
            make_polity(1, None),
            make_polity(2, Some(1)),
            make_polity(3, Some(2)),
            make_polity(4, Some(1)),
            make_polity(5, None), // Different sovereign
        ]);

        assert!(same_realm(PolityId(2), PolityId(3), &polities));
        assert!(same_realm(PolityId(2), PolityId(4), &polities));
        assert!(same_realm(PolityId(3), PolityId(4), &polities));
        assert!(!same_realm(PolityId(2), PolityId(5), &polities));
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib aggregate::hierarchy::tests::test_is_vassal -- --nocapture`
Run: `cargo test --lib aggregate::hierarchy::tests::test_same_realm -- --nocapture`
Expected: FAIL with "cannot find function"

**Step 3: Write minimal implementation**

Add to `src/aggregate/hierarchy.rs`:

```rust
/// Check if subject is a vassal of lord (at any level)
pub fn is_vassal_of(subject: PolityId, lord: PolityId, polities: &HashMap<PolityId, Polity>) -> bool {
    if subject == lord {
        return false; // Not a vassal of yourself
    }

    let mut current = subject;
    let mut visited = std::collections::HashSet::new();

    while let Some(polity) = polities.get(&current) {
        if !visited.insert(current) {
            return false; // Cycle detected
        }

        match polity.parent {
            Some(parent_id) if parent_id == lord => return true,
            Some(parent_id) => current = parent_id,
            None => return false, // Reached sovereign without finding lord
        }
    }

    false
}

/// Check if two polities are in the same realm (share a sovereign)
pub fn same_realm(a: PolityId, b: PolityId, polities: &HashMap<PolityId, Polity>) -> bool {
    match (get_sovereign(a, polities), get_sovereign(b, polities)) {
        (Some(sov_a), Some(sov_b)) => sov_a == sov_b,
        _ => false,
    }
}

/// Get the liege (immediate parent) of a polity
pub fn get_liege(polity_id: PolityId, polities: &HashMap<PolityId, Polity>) -> Option<PolityId> {
    polities.get(&polity_id).and_then(|p| p.parent)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib aggregate::hierarchy::tests -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/aggregate/hierarchy.rs
git commit -m "feat(aggregate): implement is_vassal_of, same_realm, get_liege"
```

---

## Phase 5: World Storage Updates

### Task 5.1: Update AggregateWorld

**Files:**
- Modify: `src/aggregate/world.rs`

**Step 1: Write the failing test**

Add tests to `src/aggregate/world.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{PolityId, RulerId};
    use crate::aggregate::ruler::Ruler;

    #[test]
    fn test_world_has_rulers() {
        let world = AggregateWorld::new(vec![], vec![], ChaCha8Rng::seed_from_u64(42));
        assert!(world.rulers.is_empty());
    }

    #[test]
    fn test_world_polity_lookup_by_id() {
        // This test verifies we can look up polities by PolityId
        let world = AggregateWorld::new(vec![], vec![], ChaCha8Rng::seed_from_u64(42));
        assert!(world.get_polity_by_id(PolityId(1)).is_none());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib aggregate::world::tests -- --nocapture`
Expected: FAIL with missing field/method

**Step 3: Update AggregateWorld**

Modify `src/aggregate/world.rs`:

Add imports:
```rust
use std::collections::HashMap;
use crate::core::types::{PolityId, RulerId};
use crate::aggregate::ruler::Ruler;
```

Update struct:
```rust
/// The aggregate world state for history simulation
pub struct AggregateWorld {
    /// All regions (pseudo-nodes) in the world
    pub regions: Vec<Region>,
    /// All polities (nations/tribes/holds/groves)
    pub polities: Vec<Polity>,
    /// All rulers (characters who lead polities)
    pub rulers: HashMap<RulerId, Ruler>,
    /// Currently active wars
    pub active_wars: Vec<War>,
    /// Current simulation year
    pub year: u32,
    /// Random number generator (deterministic)
    pub rng: ChaCha8Rng,
    /// Next polity ID to assign
    next_polity_id: u32,
    /// Next ruler ID to assign
    next_ruler_id: u32,
}

impl AggregateWorld {
    pub fn new(regions: Vec<Region>, polities: Vec<Polity>, rng: ChaCha8Rng) -> Self {
        let next_polity_id = polities.iter()
            .map(|p| p.id.0)
            .max()
            .unwrap_or(0) + 1;

        Self {
            regions,
            polities,
            rulers: HashMap::new(),
            active_wars: Vec::new(),
            year: 0,
            rng,
            next_polity_id,
            next_ruler_id: 1,
        }
    }

    /// Get polity by PolityId
    pub fn get_polity_by_id(&self, id: PolityId) -> Option<&Polity> {
        self.polities.iter().find(|p| p.id == id)
    }

    /// Get mutable polity by PolityId
    pub fn get_polity_by_id_mut(&mut self, id: PolityId) -> Option<&mut Polity> {
        self.polities.iter_mut().find(|p| p.id == id)
    }

    /// Get ruler by RulerId
    pub fn get_ruler(&self, id: RulerId) -> Option<&Ruler> {
        self.rulers.get(&id)
    }

    /// Get mutable ruler by RulerId
    pub fn get_ruler_mut(&mut self, id: RulerId) -> Option<&mut Ruler> {
        self.rulers.get_mut(&id)
    }

    /// Add a new ruler
    pub fn add_ruler(&mut self, ruler: Ruler) {
        self.rulers.insert(ruler.id, ruler);
    }

    /// Generate next polity ID
    pub fn next_polity_id(&mut self) -> PolityId {
        let id = PolityId(self.next_polity_id);
        self.next_polity_id += 1;
        id
    }

    /// Generate next ruler ID
    pub fn next_ruler_id(&mut self) -> RulerId {
        let id = RulerId(self.next_ruler_id);
        self.next_ruler_id += 1;
        id
    }

    /// Build a map of PolityId -> Polity for hierarchy queries
    pub fn polity_map(&self) -> HashMap<PolityId, Polity> {
        self.polities.iter().map(|p| (p.id, p.clone())).collect()
    }

    // Keep existing methods...
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib aggregate::world::tests -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add src/aggregate/world.rs
git commit -m "feat(aggregate): update AggregateWorld with rulers storage"
```

---

## Phase 6: Location Controller

### Task 6.1: Add Location Struct with PolityId Controller

**Files:**
- Modify: `src/campaign/location.rs`
- Modify: `src/campaign/mod.rs`

**Step 1: Implement Location struct**

Replace contents of `src/campaign/location.rs`:

```rust
//! Location - a place on the campaign map
//!
//! Locations are the nodes in the campaign map graph.
//! They have a controller (polity that owns them) and various properties.

use serde::{Deserialize, Serialize};
use crate::core::types::{LocationId, PolityId};

/// A location on the campaign map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: LocationId,
    pub name: String,
    pub controller: Option<PolityId>,  // Which polity controls this location
    pub population: u32,
    pub fortification: u8,  // 0-10, affects siege difficulty
}

impl Location {
    pub fn new(id: LocationId, name: String) -> Self {
        Self {
            id,
            name,
            controller: None,
            population: 0,
            fortification: 0,
        }
    }

    /// Transfer control to a new polity
    pub fn transfer_control(&mut self, new_controller: Option<PolityId>) {
        self.controller = new_controller;
    }

    /// Check if controlled by a specific polity
    pub fn is_controlled_by(&self, polity: PolityId) -> bool {
        self.controller == Some(polity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_controller() {
        let mut loc = Location::new(LocationId(1), "Castle".to_string());
        assert!(loc.controller.is_none());

        loc.transfer_control(Some(PolityId(1)));
        assert!(loc.is_controlled_by(PolityId(1)));

        loc.transfer_control(Some(PolityId(2)));
        assert!(!loc.is_controlled_by(PolityId(1)));
        assert!(loc.is_controlled_by(PolityId(2)));
    }
}
```

**Step 2: Update mod.rs**

Ensure `src/campaign/mod.rs` exports Location:
```rust
pub mod location;
pub use location::Location;
```

**Step 3: Run tests**

Run: `cargo test --lib campaign::location -- --nocapture`
Expected: PASS

**Step 4: Commit**

```bash
git add src/campaign/location.rs src/campaign/mod.rs
git commit -m "feat(campaign): add Location with PolityId controller"
```

---

## Phase 7: Integration and Final Tests

### Task 7.1: Full Integration Test

**Files:**
- Create: `tests/hierarchy_integration.rs`

**Step 1: Write integration test**

Create `tests/hierarchy_integration.rs`:

```rust
//! Integration tests for hierarchical polity system

use std::collections::HashMap;

use arc_citadel::core::types::{PolityId, RulerId, PolityTier, GovernmentType, Species, LocationId};
use arc_citadel::aggregate::polity::{Polity, PolityType, CulturalDrift, SpeciesState, HumanState, CouncilRole};
use arc_citadel::aggregate::ruler::{Ruler, PersonalityTrait, Skills, Family, Opinion};
use arc_citadel::aggregate::hierarchy::{get_sovereign, get_vassals, get_all_vassals, is_vassal_of, same_realm};
use arc_citadel::campaign::location::Location;

fn create_test_hierarchy() -> (HashMap<PolityId, Polity>, HashMap<RulerId, Ruler>) {
    // Create: Empire(1) -> Kingdom(2) -> Duchy(3)
    //                   -> Kingdom(4)

    let mut polities = HashMap::new();
    let mut rulers = HashMap::new();

    // Empire
    let emperor = Ruler::new(
        RulerId(1),
        "Emperor Magnus".to_string(),
        Species::Human,
        55,
        vec![PersonalityTrait::Ambitious, PersonalityTrait::Charismatic],
        Skills::new(7, 5, 6, 3),
        Family::founder(1),
    );
    rulers.insert(emperor.id, emperor);

    polities.insert(PolityId(1), Polity {
        id: PolityId(1),
        name: "Empire of Aldoria".to_string(),
        species: Species::Human,
        polity_type: PolityType::Kingdom,
        tier: PolityTier::Empire,
        government: GovernmentType::Autocracy,
        parent: None,
        rulers: vec![RulerId(1)],
        council_roles: HashMap::new(),
        capital: 0,
        population: 100000,
        military_strength: 1000.0,
        economic_strength: 1000.0,
        cultural_drift: CulturalDrift::default(),
        relations: HashMap::new(),
        species_state: SpeciesState::Human(HumanState::default()),
        alive: true,
    });

    // Kingdom under Empire
    let king = Ruler::new(
        RulerId(2),
        "King Aldric".to_string(),
        Species::Human,
        42,
        vec![PersonalityTrait::Honorable, PersonalityTrait::Cautious],
        Skills::new(5, 8, 4, 2),
        Family::founder(2),
    );
    rulers.insert(king.id, king);

    polities.insert(PolityId(2), Polity {
        id: PolityId(2),
        name: "Kingdom of Valheim".to_string(),
        species: Species::Human,
        polity_type: PolityType::Kingdom,
        tier: PolityTier::Kingdom,
        government: GovernmentType::Autocracy,
        parent: Some(PolityId(1)),
        rulers: vec![RulerId(2)],
        council_roles: HashMap::new(),
        capital: 1,
        population: 50000,
        military_strength: 500.0,
        economic_strength: 500.0,
        cultural_drift: CulturalDrift::default(),
        relations: HashMap::new(),
        species_state: SpeciesState::Human(HumanState::default()),
        alive: true,
    });

    // Duchy under Kingdom
    let duke = Ruler::new(
        RulerId(3),
        "Duke Rodric".to_string(),
        Species::Human,
        35,
        vec![PersonalityTrait::Warlike],
        Skills::new(3, 9, 2, 1),
        Family::founder(3),
    );
    rulers.insert(duke.id, duke);

    polities.insert(PolityId(3), Polity {
        id: PolityId(3),
        name: "Duchy of Ironhold".to_string(),
        species: Species::Human,
        polity_type: PolityType::Kingdom,
        tier: PolityTier::Duchy,
        government: GovernmentType::Autocracy,
        parent: Some(PolityId(2)),
        rulers: vec![RulerId(3)],
        council_roles: HashMap::new(),
        capital: 2,
        population: 20000,
        military_strength: 200.0,
        economic_strength: 200.0,
        cultural_drift: CulturalDrift::default(),
        relations: HashMap::new(),
        species_state: SpeciesState::Human(HumanState::default()),
        alive: true,
    });

    // Another Kingdom under Empire
    polities.insert(PolityId(4), Polity {
        id: PolityId(4),
        name: "Kingdom of Eastmarch".to_string(),
        species: Species::Human,
        polity_type: PolityType::Kingdom,
        tier: PolityTier::Kingdom,
        government: GovernmentType::Autocracy,
        parent: Some(PolityId(1)),
        rulers: vec![RulerId(4)],
        council_roles: HashMap::new(),
        capital: 3,
        population: 40000,
        military_strength: 400.0,
        economic_strength: 400.0,
        cultural_drift: CulturalDrift::default(),
        relations: HashMap::new(),
        species_state: SpeciesState::Human(HumanState::default()),
        alive: true,
    });

    (polities, rulers)
}

#[test]
fn test_hierarchy_queries() {
    let (polities, _rulers) = create_test_hierarchy();

    // All should trace to Empire
    assert_eq!(get_sovereign(PolityId(1), &polities), Some(PolityId(1)));
    assert_eq!(get_sovereign(PolityId(2), &polities), Some(PolityId(1)));
    assert_eq!(get_sovereign(PolityId(3), &polities), Some(PolityId(1)));
    assert_eq!(get_sovereign(PolityId(4), &polities), Some(PolityId(1)));

    // Direct vassals of Empire
    let empire_vassals = get_vassals(PolityId(1), &polities);
    assert_eq!(empire_vassals.len(), 2);

    // All vassals of Empire
    let all_empire_vassals = get_all_vassals(PolityId(1), &polities);
    assert_eq!(all_empire_vassals.len(), 3);

    // Vassal relationships
    assert!(is_vassal_of(PolityId(3), PolityId(1), &polities)); // Duchy vassal of Empire
    assert!(is_vassal_of(PolityId(3), PolityId(2), &polities)); // Duchy vassal of Kingdom
    assert!(!is_vassal_of(PolityId(2), PolityId(3), &polities)); // Kingdom not vassal of Duchy

    // Same realm
    assert!(same_realm(PolityId(2), PolityId(4), &polities)); // Both under Empire
    assert!(same_realm(PolityId(3), PolityId(4), &polities)); // Duchy and other Kingdom
}

#[test]
fn test_ruler_opinions() {
    let (_polities, mut rulers) = create_test_hierarchy();

    // Emperor forms opinions of vassal kingdoms
    let emperor = rulers.get_mut(&RulerId(1)).unwrap();
    emperor.set_opinion(PolityId(2), Opinion::new(50));  // Likes Kingdom
    emperor.set_opinion(PolityId(4), Opinion::new(-20)); // Dislikes other Kingdom

    assert_eq!(emperor.get_opinion(PolityId(2)).unwrap().effective_value(), 50);
    assert_eq!(emperor.get_opinion(PolityId(4)).unwrap().effective_value(), -20);

    // King forms opinion of liege
    let king = rulers.get_mut(&RulerId(2)).unwrap();
    king.set_opinion(PolityId(1), Opinion::new(30)); // Respects Emperor

    // War modifier from personalities
    let duke = rulers.get(&RulerId(3)).unwrap();
    assert!(duke.war_modifier() > 0); // Warlike trait increases war likelihood
}

#[test]
fn test_location_controller() {
    let mut castle = Location::new(LocationId(1), "Ironhold Castle".to_string());

    // Initially uncontrolled
    assert!(castle.controller.is_none());

    // Transfer to duchy
    castle.transfer_control(Some(PolityId(3)));
    assert!(castle.is_controlled_by(PolityId(3)));

    // Conquered by kingdom
    castle.transfer_control(Some(PolityId(2)));
    assert!(!castle.is_controlled_by(PolityId(3)));
    assert!(castle.is_controlled_by(PolityId(2)));
}
```

**Step 2: Run integration tests**

Run: `cargo test --test hierarchy_integration -- --nocapture`
Expected: PASS

**Step 3: Commit**

```bash
git add tests/hierarchy_integration.rs
git commit -m "test: add integration tests for hierarchical polity system"
```

---

### Task 7.2: Run Full Test Suite

**Step 1: Run all tests**

Run: `cargo test`
Expected: All tests PASS

**Step 2: Run clippy**

Run: `cargo clippy -- -D warnings`
Expected: No warnings

**Step 3: Final commit**

```bash
git add -A
git commit -m "feat: complete hierarchical polity system implementation"
```

---

## Summary

This implementation plan delivers:

1. **Core Types**: `PolityId`, `RulerId`, `PolityTier`, `GovernmentType` in `core/types.rs`
2. **Ruler System**: Full character model with personality, skills, claims, family, opinions in `aggregate/ruler.rs`
3. **Updated Polity**: Hierarchy fields (`parent`, `tier`, `rulers`), removed `territory` in `aggregate/polity.rs`
4. **Hierarchy Operations**: `get_sovereign`, `get_vassals`, `is_vassal_of`, `same_realm` in `aggregate/hierarchy.rs`
5. **Location Controller**: `Location.controller: Option<PolityId>` in `campaign/location.rs`
6. **Storage Updates**: Rulers HashMap and ID generation in `aggregate/world.rs`

**Not included (future work):**
- Succession mechanics
- Council decision aggregation beyond basic structure
- Opinion decay over simulation time
- Claim inheritance/validation
- Event system integration
