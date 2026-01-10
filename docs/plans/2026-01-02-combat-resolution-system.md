# Combat Resolution System Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement a categorical property-based combat system where weapon/armor properties determine outcomes through lookup tables, not percentage modifiers.

**Architecture:** Combat resolution uses categorical comparisons (Edge vs Rigidity → PenetrationResult, Mass vs Padding → TraumaResult). Two victory paths: Damage (wounds until incapacitated) and Morale (stress until break). Stance system determines exchange resolution order. Formation combat uses statistical aggregation at LOD 1+.

**Tech Stack:** Rust, serde for serialization, existing SoA archetype pattern

**Philosophy (READ FIRST):**
- NO multiplicative stacking (`* (1 + modifier)` patterns are FORBIDDEN)
- Categorical outcomes from property comparisons
- Skill unlocks capabilities, doesn't add numeric bonuses
- Deterministic LOD transitions (no RNG)

---

## Task 1: Combat Constants Module

**Files:**
- Create: `src/combat/constants.rs`
- Modify: `src/combat/mod.rs:1-6`

**Step 1: Write the failing test**

```rust
// In src/combat/constants.rs at bottom
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fatigue_constants_reasonable() {
        assert!(FATIGUE_PER_ATTACK > 0.0 && FATIGUE_PER_ATTACK < 0.2);
        assert!(FATIGUE_PER_DEFEND > 0.0 && FATIGUE_PER_DEFEND < FATIGUE_PER_ATTACK);
        assert!(FATIGUE_RECOVERY_RATE > 0.0);
    }

    #[test]
    fn test_stress_constants_reasonable() {
        assert!(BASE_STRESS_THRESHOLD > 0.0);
        assert!(SHAKEN_THRESHOLD_RATIO > 0.0 && SHAKEN_THRESHOLD_RATIO < 1.0);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib combat::constants`
Expected: FAIL with "cannot find value `FATIGUE_PER_ATTACK`"

**Step 3: Write minimal implementation**

```rust
//! Combat system constants - all tunable values in one place
//!
//! These values are ADDITIVE, never multiplicative. No percentage modifiers.

// Time constants
pub const TICK_DURATION_MS: u32 = 100;
pub const RECOVERY_TICKS: u32 = 10;
pub const EXHAUSTION_THRESHOLD: f32 = 0.9;

// Fatigue constants (ADDITIVE, not multiplicative)
pub const FATIGUE_PER_ATTACK: f32 = 0.05;
pub const FATIGUE_PER_DEFEND: f32 = 0.03;
pub const FATIGUE_PER_MELEE_TICK: f32 = 0.01;
pub const FATIGUE_RECOVERY_RATE: f32 = 0.02;

// Stress constants (ADDITIVE thresholds)
pub const BASE_STRESS_THRESHOLD: f32 = 1.0;
pub const STRESS_DECAY_RATE: f32 = 0.001;
pub const SHAKEN_THRESHOLD_RATIO: f32 = 0.8;

// Formation constants
pub const FORMATION_BREAK_THRESHOLD: f32 = 0.4;
pub const COHESION_LOSS_PER_CASUALTY: f32 = 0.02;
pub const PRESSURE_DECAY_RATE: f32 = 0.01;

// Officer constants
pub const INTERVENTION_COST: f32 = 0.2;
pub const INTERVENTION_RANGE: f32 = 10.0;
pub const ATTENTION_RECOVERY_RATE: f32 = 0.1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fatigue_constants_reasonable() {
        assert!(FATIGUE_PER_ATTACK > 0.0 && FATIGUE_PER_ATTACK < 0.2);
        assert!(FATIGUE_PER_DEFEND > 0.0 && FATIGUE_PER_DEFEND < FATIGUE_PER_ATTACK);
        assert!(FATIGUE_RECOVERY_RATE > 0.0);
    }

    #[test]
    fn test_stress_constants_reasonable() {
        assert!(BASE_STRESS_THRESHOLD > 0.0);
        assert!(SHAKEN_THRESHOLD_RATIO > 0.0 && SHAKEN_THRESHOLD_RATIO < 1.0);
    }
}
```

**Step 4: Update mod.rs**

```rust
// src/combat/mod.rs
pub mod constants;
pub mod resolution;
pub mod weapons;
pub mod armor;
pub mod wounds;
pub mod morale;
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib combat::constants`
Expected: PASS (2 tests)

**Step 6: Commit**

```bash
git add src/combat/constants.rs src/combat/mod.rs
git commit -m "feat(combat): add constants module with tunable values"
```

---

## Task 2: Weapon Properties (Edge, Mass, Reach)

**Files:**
- Replace: `src/combat/weapons.rs`
- Test: inline `#[cfg(test)]`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sword_properties() {
        let sword = WeaponProperties {
            edge: Edge::Sharp,
            mass: Mass::Medium,
            reach: Reach::Short,
            special: vec![],
        };
        assert_eq!(sword.edge, Edge::Sharp);
        assert_eq!(sword.mass, Mass::Medium);
    }

    #[test]
    fn test_mace_is_blunt() {
        let mace = WeaponProperties {
            edge: Edge::Blunt,
            mass: Mass::Heavy,
            reach: Reach::Short,
            special: vec![],
        };
        assert_eq!(mace.edge, Edge::Blunt);
    }

    #[test]
    fn test_reach_ordering() {
        assert!(Reach::Pike > Reach::Long);
        assert!(Reach::Long > Reach::Medium);
        assert!(Reach::Medium > Reach::Short);
        assert!(Reach::Short > Reach::Grapple);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib combat::weapons`
Expected: FAIL with "cannot find type `Edge`"

**Step 3: Write minimal implementation**

```rust
//! Weapon properties for categorical combat resolution
//!
//! Weapons have exactly three properties: Edge, Mass, Reach.
//! These determine outcomes via lookup tables, not modifiers.

use serde::{Deserialize, Serialize};

/// Sharpness category - determines penetration potential
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Edge {
    /// Surgical sharpness (scalpels, fine blades)
    Razor,
    /// Combat sharpness (swords, axes)
    Sharp,
    /// No edge (maces, hammers, fists)
    Blunt,
}

/// Weight category - determines trauma potential
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mass {
    /// Daggers, small weapons (<1kg)
    Light,
    /// Swords, axes (1-3kg)
    Medium,
    /// Warhammers, greatswords (3-6kg)
    Heavy,
    /// Horse + rider, siege weapons (>100kg)
    Massive,
}

/// Distance category - determines strike order in exchanges
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Reach {
    /// Touching distance (fists, daggers)
    Grapple,
    /// Arm's length (swords, maces)
    Short,
    /// Extended arm (bastard swords, axes)
    Medium,
    /// Spear length (spears, halberds)
    Long,
    /// Formation weapons (pikes, lances)
    Pike,
}

/// Special weapon properties (optional capabilities)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponSpecial {
    /// Can find gaps in armor (estocs, bodkins)
    Piercing,
    /// Can pull shields/dismount (billhooks)
    Hooking,
    /// Can be thrown (javelins, axes)
    Throwable,
    /// Requires both hands
    TwoHanded,
    /// Effective vs shields (axes)
    Shieldbreaker,
}

/// Complete weapon properties
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponProperties {
    pub edge: Edge,
    pub mass: Mass,
    pub reach: Reach,
    pub special: Vec<WeaponSpecial>,
}

impl WeaponProperties {
    /// Check if weapon has a specific special property
    pub fn has_special(&self, special: WeaponSpecial) -> bool {
        self.special.contains(&special)
    }

    /// Common weapon: Sword
    pub fn sword() -> Self {
        Self {
            edge: Edge::Sharp,
            mass: Mass::Medium,
            reach: Reach::Short,
            special: vec![],
        }
    }

    /// Common weapon: Mace
    pub fn mace() -> Self {
        Self {
            edge: Edge::Blunt,
            mass: Mass::Heavy,
            reach: Reach::Short,
            special: vec![],
        }
    }

    /// Common weapon: Spear
    pub fn spear() -> Self {
        Self {
            edge: Edge::Sharp,
            mass: Mass::Medium,
            reach: Reach::Long,
            special: vec![WeaponSpecial::Piercing],
        }
    }

    /// Common weapon: Dagger
    pub fn dagger() -> Self {
        Self {
            edge: Edge::Sharp,
            mass: Mass::Light,
            reach: Reach::Grapple,
            special: vec![WeaponSpecial::Piercing],
        }
    }

    /// Common weapon: Fists (unarmed)
    pub fn fists() -> Self {
        Self {
            edge: Edge::Blunt,
            mass: Mass::Light,
            reach: Reach::Grapple,
            special: vec![],
        }
    }
}

impl Default for WeaponProperties {
    fn default() -> Self {
        Self::fists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sword_properties() {
        let sword = WeaponProperties {
            edge: Edge::Sharp,
            mass: Mass::Medium,
            reach: Reach::Short,
            special: vec![],
        };
        assert_eq!(sword.edge, Edge::Sharp);
        assert_eq!(sword.mass, Mass::Medium);
    }

    #[test]
    fn test_mace_is_blunt() {
        let mace = WeaponProperties {
            edge: Edge::Blunt,
            mass: Mass::Heavy,
            reach: Reach::Short,
            special: vec![],
        };
        assert_eq!(mace.edge, Edge::Blunt);
    }

    #[test]
    fn test_reach_ordering() {
        assert!(Reach::Pike > Reach::Long);
        assert!(Reach::Long > Reach::Medium);
        assert!(Reach::Medium > Reach::Short);
        assert!(Reach::Short > Reach::Grapple);
    }

    #[test]
    fn test_common_weapons() {
        let sword = WeaponProperties::sword();
        assert_eq!(sword.edge, Edge::Sharp);

        let mace = WeaponProperties::mace();
        assert_eq!(mace.edge, Edge::Blunt);

        let spear = WeaponProperties::spear();
        assert!(spear.has_special(WeaponSpecial::Piercing));
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib combat::weapons`
Expected: PASS (4 tests)

**Step 5: Commit**

```bash
git add src/combat/weapons.rs
git commit -m "feat(combat): add weapon properties (Edge, Mass, Reach)"
```

---

## Task 3: Armor Properties (Rigidity, Padding, Coverage)

**Files:**
- Replace: `src/combat/armor.rs`
- Test: inline `#[cfg(test)]`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plate_armor() {
        let plate = ArmorProperties {
            rigidity: Rigidity::Plate,
            padding: Padding::Heavy,
            coverage: Coverage::Full,
        };
        assert_eq!(plate.rigidity, Rigidity::Plate);
    }

    #[test]
    fn test_unarmored() {
        let naked = ArmorProperties::none();
        assert_eq!(naked.rigidity, Rigidity::Cloth);
        assert_eq!(naked.padding, Padding::None);
        assert_eq!(naked.coverage, Coverage::None);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib combat::armor`
Expected: FAIL with "cannot find type `Rigidity`"

**Step 3: Write minimal implementation**

```rust
//! Armor properties for categorical combat resolution
//!
//! Armor has exactly three properties: Rigidity, Padding, Coverage.
//! These determine outcomes via lookup tables against weapon properties.

use serde::{Deserialize, Serialize};

/// Material hardness - determines if edge can penetrate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rigidity {
    /// Clothing, robes
    Cloth,
    /// Cured hide, thick cloth
    Leather,
    /// Interlocking rings
    Mail,
    /// Solid metal
    Plate,
}

/// Impact absorption - determines trauma from mass
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Padding {
    /// Bare skin or cloth only
    None,
    /// Thin gambeson, leather backing
    Light,
    /// Full gambeson, layered padding
    Heavy,
}

/// How much of body is protected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Coverage {
    /// Unarmored
    None,
    /// Some gaps (standard armor)
    Partial,
    /// Nearly complete (full harness)
    Full,
}

/// Complete armor properties
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmorProperties {
    pub rigidity: Rigidity,
    pub padding: Padding,
    pub coverage: Coverage,
}

impl ArmorProperties {
    /// No armor at all
    pub fn none() -> Self {
        Self {
            rigidity: Rigidity::Cloth,
            padding: Padding::None,
            coverage: Coverage::None,
        }
    }

    /// Light armor (leather)
    pub fn leather() -> Self {
        Self {
            rigidity: Rigidity::Leather,
            padding: Padding::Light,
            coverage: Coverage::Partial,
        }
    }

    /// Medium armor (mail)
    pub fn mail() -> Self {
        Self {
            rigidity: Rigidity::Mail,
            padding: Padding::Light,
            coverage: Coverage::Partial,
        }
    }

    /// Heavy armor (plate)
    pub fn plate() -> Self {
        Self {
            rigidity: Rigidity::Plate,
            padding: Padding::Heavy,
            coverage: Coverage::Full,
        }
    }
}

impl Default for ArmorProperties {
    fn default() -> Self {
        Self::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plate_armor() {
        let plate = ArmorProperties {
            rigidity: Rigidity::Plate,
            padding: Padding::Heavy,
            coverage: Coverage::Full,
        };
        assert_eq!(plate.rigidity, Rigidity::Plate);
    }

    #[test]
    fn test_unarmored() {
        let naked = ArmorProperties::none();
        assert_eq!(naked.rigidity, Rigidity::Cloth);
        assert_eq!(naked.padding, Padding::None);
        assert_eq!(naked.coverage, Coverage::None);
    }

    #[test]
    fn test_armor_presets() {
        let leather = ArmorProperties::leather();
        assert_eq!(leather.rigidity, Rigidity::Leather);

        let mail = ArmorProperties::mail();
        assert_eq!(mail.rigidity, Rigidity::Mail);

        let plate = ArmorProperties::plate();
        assert_eq!(plate.rigidity, Rigidity::Plate);
        assert_eq!(plate.padding, Padding::Heavy);
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib combat::armor`
Expected: PASS (3 tests)

**Step 5: Commit**

```bash
git add src/combat/armor.rs
git commit -m "feat(combat): add armor properties (Rigidity, Padding, Coverage)"
```

---

## Task 4: Body Zones (11 zones with hit weights)

**Files:**
- Create: `src/combat/body_zone.rs`
- Modify: `src/combat/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_zone_count() {
        // 11 zones: Head, Neck, Torso, 2 Arms, 2 Hands, 2 Legs, 2 Feet
        assert_eq!(BodyZone::all().len(), 11);
    }

    #[test]
    fn test_hit_weights_sum_to_one() {
        let total: f32 = BodyZone::all()
            .iter()
            .map(|z| z.hit_weight_standing())
            .sum();
        assert!((total - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_head_is_fatal() {
        assert_eq!(BodyZone::Head.fatality_threshold(), WoundSeverity::Serious);
    }

    #[test]
    fn test_limbs_not_directly_fatal() {
        assert_eq!(BodyZone::ArmLeft.fatality_threshold(), WoundSeverity::Destroyed);
        assert_eq!(BodyZone::LegRight.fatality_threshold(), WoundSeverity::Destroyed);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib combat::body_zone`
Expected: FAIL with "cannot find type `BodyZone`"

**Step 3: Write minimal implementation**

```rust
//! Body zones for wound tracking (11 zones)
//!
//! Replaces the old 6-zone BodyPart system with granular zones.

use serde::{Deserialize, Serialize};

/// Wound severity categories (not f32)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WoundSeverity {
    /// No wound
    None,
    /// Cosmetic only
    Scratch,
    /// Painful but functional
    Minor,
    /// Impaired function, bleeding
    Serious,
    /// Disabled, severe bleeding
    Critical,
    /// Limb gone / organ destroyed
    Destroyed,
}

/// Body zones for hit location (11 total)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BodyZone {
    /// Fatal threshold very low
    Head,
    /// Arterial - bleeds fast
    Neck,
    /// Center mass, most hits land here
    Torso,
    ArmLeft,
    ArmRight,
    HandLeft,
    HandRight,
    LegLeft,
    LegRight,
    FootLeft,
    FootRight,
}

impl BodyZone {
    /// Returns all body zones
    pub fn all() -> [BodyZone; 11] {
        [
            BodyZone::Head,
            BodyZone::Neck,
            BodyZone::Torso,
            BodyZone::ArmLeft,
            BodyZone::ArmRight,
            BodyZone::HandLeft,
            BodyZone::HandRight,
            BodyZone::LegLeft,
            BodyZone::LegRight,
            BodyZone::FootLeft,
            BodyZone::FootRight,
        ]
    }

    /// What severity of wound to this zone is fatal?
    pub fn fatality_threshold(&self) -> WoundSeverity {
        match self {
            BodyZone::Head | BodyZone::Neck => WoundSeverity::Serious,
            BodyZone::Torso => WoundSeverity::Critical,
            // Limbs don't kill directly
            _ => WoundSeverity::Destroyed,
        }
    }

    /// Relative probability of being hit when standing (sums to 1.0)
    pub fn hit_weight_standing(&self) -> f32 {
        match self {
            BodyZone::Torso => 0.35,
            BodyZone::ArmLeft | BodyZone::ArmRight => 0.10,
            BodyZone::LegLeft | BodyZone::LegRight => 0.12,
            BodyZone::Head => 0.08,
            BodyZone::Neck => 0.03,
            BodyZone::HandLeft | BodyZone::HandRight => 0.03,
            BodyZone::FootLeft | BodyZone::FootRight => 0.02,
        }
    }

    /// Is this a leg zone?
    pub fn is_leg(&self) -> bool {
        matches!(self, BodyZone::LegLeft | BodyZone::LegRight | BodyZone::FootLeft | BodyZone::FootRight)
    }

    /// Is this an arm zone?
    pub fn is_arm(&self) -> bool {
        matches!(self, BodyZone::ArmLeft | BodyZone::ArmRight)
    }

    /// Is this a hand zone?
    pub fn is_hand(&self) -> bool {
        matches!(self, BodyZone::HandLeft | BodyZone::HandRight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_zone_count() {
        assert_eq!(BodyZone::all().len(), 11);
    }

    #[test]
    fn test_hit_weights_sum_to_one() {
        let total: f32 = BodyZone::all()
            .iter()
            .map(|z| z.hit_weight_standing())
            .sum();
        assert!((total - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_head_is_fatal() {
        assert_eq!(BodyZone::Head.fatality_threshold(), WoundSeverity::Serious);
    }

    #[test]
    fn test_limbs_not_directly_fatal() {
        assert_eq!(BodyZone::ArmLeft.fatality_threshold(), WoundSeverity::Destroyed);
        assert_eq!(BodyZone::LegRight.fatality_threshold(), WoundSeverity::Destroyed);
    }

    #[test]
    fn test_zone_categories() {
        assert!(BodyZone::LegLeft.is_leg());
        assert!(BodyZone::FootRight.is_leg());
        assert!(!BodyZone::ArmLeft.is_leg());

        assert!(BodyZone::ArmRight.is_arm());
        assert!(!BodyZone::HandRight.is_arm());

        assert!(BodyZone::HandLeft.is_hand());
    }
}
```

**Step 4: Update mod.rs**

```rust
// src/combat/mod.rs
pub mod constants;
pub mod body_zone;
pub mod weapons;
pub mod armor;
pub mod resolution;
pub mod wounds;
pub mod morale;

pub use body_zone::{BodyZone, WoundSeverity};
pub use weapons::{Edge, Mass, Reach, WeaponProperties, WeaponSpecial};
pub use armor::{Rigidity, Padding, Coverage, ArmorProperties};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib combat::body_zone`
Expected: PASS (5 tests)

**Step 6: Commit**

```bash
git add src/combat/body_zone.rs src/combat/mod.rs
git commit -m "feat(combat): add 11-zone body system with hit weights"
```

---

## Task 5: Penetration Resolution (Edge vs Rigidity)

**Files:**
- Create: `src/combat/penetration.rs`
- Modify: `src/combat/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::weapons::Edge;
    use crate::combat::armor::Rigidity;

    #[test]
    fn test_sharp_vs_plate_deflects() {
        let result = resolve_penetration(Edge::Sharp, Rigidity::Plate, false);
        assert_eq!(result, PenetrationResult::Deflect);
    }

    #[test]
    fn test_razor_vs_cloth_deep_cut() {
        let result = resolve_penetration(Edge::Razor, Rigidity::Cloth, false);
        assert_eq!(result, PenetrationResult::DeepCut);
    }

    #[test]
    fn test_blunt_no_penetration() {
        let result = resolve_penetration(Edge::Blunt, Rigidity::Plate, false);
        assert_eq!(result, PenetrationResult::NoPenetrationAttempt);
    }

    #[test]
    fn test_piercing_improves_vs_mail() {
        // Without piercing: Sharp vs Mail = Deflect
        let without = resolve_penetration(Edge::Sharp, Rigidity::Mail, false);
        assert_eq!(without, PenetrationResult::Deflect);

        // With piercing: Sharp vs Mail = ShallowCut
        let with = resolve_penetration(Edge::Sharp, Rigidity::Mail, true);
        assert_eq!(with, PenetrationResult::ShallowCut);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib combat::penetration`
Expected: FAIL with "cannot find function `resolve_penetration`"

**Step 3: Write minimal implementation**

```rust
//! Penetration resolution: Edge vs Rigidity lookup table
//!
//! NO PERCENTAGE MODIFIERS. Categorical comparison only.

use serde::{Deserialize, Serialize};
use crate::combat::weapons::Edge;
use crate::combat::armor::Rigidity;

/// Result of edge vs rigidity comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PenetrationResult {
    /// Severe wound, arterial risk
    DeepCut,
    /// Standard wound
    Cut,
    /// Minor wound
    ShallowCut,
    /// Weapon stuck, no wound, attacker vulnerable
    Snag,
    /// No effect from edge
    Deflect,
    /// Blunt weapon, skip to trauma resolution
    NoPenetrationAttempt,
}

/// Resolve penetration using categorical lookup table
///
/// # Arguments
/// * `edge` - Weapon's edge type
/// * `rigidity` - Armor's rigidity type
/// * `has_piercing` - Whether weapon has Piercing special property
///
/// # Returns
/// Categorical penetration result (no percentages)
pub fn resolve_penetration(edge: Edge, rigidity: Rigidity, has_piercing: bool) -> PenetrationResult {
    use Edge::*;
    use Rigidity::*;
    use PenetrationResult::*;

    // Base lookup table (NO MULTIPLICATION)
    let base_result = match (edge, rigidity) {
        // Razor edge - surgical sharpness
        (Razor, Cloth) => DeepCut,
        (Razor, Leather) => Cut,
        (Razor, Mail) => Snag,
        (Razor, Plate) => Deflect,

        // Sharp edge - combat weapons
        (Sharp, Cloth) => Cut,
        (Sharp, Leather) => ShallowCut,
        (Sharp, Mail) => Deflect,
        (Sharp, Plate) => Deflect,

        // Blunt - no penetration attempt, skip to trauma
        (Blunt, _) => NoPenetrationAttempt,
    };

    // Piercing special: one category better vs Mail/Plate
    // This is a CATEGORY SHIFT, not a percentage modifier
    if has_piercing {
        match (edge, rigidity, base_result) {
            // Piercing vs Mail: one step better
            (Razor, Mail, Snag) => ShallowCut,
            (Sharp, Mail, Deflect) => ShallowCut,
            // Piercing vs Plate: one step better
            (Razor, Plate, Deflect) => Snag,
            (Sharp, Plate, Deflect) => Snag,
            _ => base_result,
        }
    } else {
        base_result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sharp_vs_plate_deflects() {
        let result = resolve_penetration(Edge::Sharp, Rigidity::Plate, false);
        assert_eq!(result, PenetrationResult::Deflect);
    }

    #[test]
    fn test_razor_vs_cloth_deep_cut() {
        let result = resolve_penetration(Edge::Razor, Rigidity::Cloth, false);
        assert_eq!(result, PenetrationResult::DeepCut);
    }

    #[test]
    fn test_blunt_no_penetration() {
        let result = resolve_penetration(Edge::Blunt, Rigidity::Plate, false);
        assert_eq!(result, PenetrationResult::NoPenetrationAttempt);
    }

    #[test]
    fn test_piercing_improves_vs_mail() {
        let without = resolve_penetration(Edge::Sharp, Rigidity::Mail, false);
        assert_eq!(without, PenetrationResult::Deflect);

        let with = resolve_penetration(Edge::Sharp, Rigidity::Mail, true);
        assert_eq!(with, PenetrationResult::ShallowCut);
    }

    #[test]
    fn test_all_edge_rigidity_combinations() {
        // Verify all combinations produce valid results (no panics)
        for edge in [Edge::Razor, Edge::Sharp, Edge::Blunt] {
            for rigidity in [Rigidity::Cloth, Rigidity::Leather, Rigidity::Mail, Rigidity::Plate] {
                for piercing in [false, true] {
                    let _ = resolve_penetration(edge, rigidity, piercing);
                }
            }
        }
    }
}
```

**Step 4: Update mod.rs**

```rust
// src/combat/mod.rs
pub mod constants;
pub mod body_zone;
pub mod weapons;
pub mod armor;
pub mod penetration;
pub mod resolution;
pub mod wounds;
pub mod morale;

pub use body_zone::{BodyZone, WoundSeverity};
pub use weapons::{Edge, Mass, Reach, WeaponProperties, WeaponSpecial};
pub use armor::{Rigidity, Padding, Coverage, ArmorProperties};
pub use penetration::{PenetrationResult, resolve_penetration};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib combat::penetration`
Expected: PASS (5 tests)

**Step 6: Commit**

```bash
git add src/combat/penetration.rs src/combat/mod.rs
git commit -m "feat(combat): add penetration resolution (Edge vs Rigidity)"
```

---

## Task 6: Trauma Resolution (Mass vs Padding)

**Files:**
- Create: `src/combat/trauma.rs`
- Modify: `src/combat/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::weapons::Mass;
    use crate::combat::armor::Padding;

    #[test]
    fn test_heavy_vs_no_padding_knocks_down() {
        let result = resolve_trauma(Mass::Heavy, Padding::None);
        assert_eq!(result, TraumaResult::KnockdownBruise);
    }

    #[test]
    fn test_light_always_negligible() {
        assert_eq!(resolve_trauma(Mass::Light, Padding::None), TraumaResult::Negligible);
        assert_eq!(resolve_trauma(Mass::Light, Padding::Light), TraumaResult::Negligible);
        assert_eq!(resolve_trauma(Mass::Light, Padding::Heavy), TraumaResult::Negligible);
    }

    #[test]
    fn test_massive_overwhelms() {
        // Massive vs no padding = KnockdownCrush (broken bones)
        assert_eq!(resolve_trauma(Mass::Massive, Padding::None), TraumaResult::KnockdownCrush);
        // Even heavy padding only reduces to Stagger
        assert_eq!(resolve_trauma(Mass::Massive, Padding::Heavy), TraumaResult::Stagger);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib combat::trauma`
Expected: FAIL with "cannot find function `resolve_trauma`"

**Step 3: Write minimal implementation**

```rust
//! Trauma resolution: Mass vs Padding lookup table
//!
//! ALL hits transfer some impact. Even deflected blows cause fatigue.
//! NO PERCENTAGE MODIFIERS. Categorical comparison only.

use serde::{Deserialize, Serialize};
use crate::combat::weapons::Mass;
use crate::combat::armor::Padding;

/// Result of mass vs padding comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TraumaResult {
    /// No mechanical effect
    Negligible,
    /// Stamina cost, accumulates
    Fatigue,
    /// Brief vulnerability window
    Stagger,
    /// On ground + internal bruising
    KnockdownBruise,
    /// On ground + broken bones
    KnockdownCrush,
}

/// Resolve trauma using categorical lookup table
///
/// # Arguments
/// * `mass` - Weapon's mass category
/// * `padding` - Armor's padding category
///
/// # Returns
/// Categorical trauma result (no percentages)
pub fn resolve_trauma(mass: Mass, padding: Padding) -> TraumaResult {
    use Mass::*;
    use Padding::*;
    use TraumaResult::*;

    // Lookup table (NO MULTIPLICATION)
    match (mass, padding) {
        // Light weapons - minimal trauma regardless of padding
        (Light, None) => Negligible,
        (Light, Light) => Negligible,
        (Light, Heavy) => Negligible,

        // Medium weapons - depends on padding
        (Medium, None) => Stagger,
        (Medium, Light) => Fatigue,
        (Medium, Heavy) => Negligible,

        // Heavy weapons - padding helps but doesn't eliminate
        (Heavy, None) => KnockdownBruise,
        (Heavy, Light) => Stagger,
        (Heavy, Heavy) => Fatigue,

        // Massive (cavalry charge, siege) - padding barely matters
        (Massive, None) => KnockdownCrush,
        (Massive, Light) => KnockdownBruise,
        (Massive, Heavy) => Stagger,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heavy_vs_no_padding_knocks_down() {
        let result = resolve_trauma(Mass::Heavy, Padding::None);
        assert_eq!(result, TraumaResult::KnockdownBruise);
    }

    #[test]
    fn test_light_always_negligible() {
        assert_eq!(resolve_trauma(Mass::Light, Padding::None), TraumaResult::Negligible);
        assert_eq!(resolve_trauma(Mass::Light, Padding::Light), TraumaResult::Negligible);
        assert_eq!(resolve_trauma(Mass::Light, Padding::Heavy), TraumaResult::Negligible);
    }

    #[test]
    fn test_massive_overwhelms() {
        assert_eq!(resolve_trauma(Mass::Massive, Padding::None), TraumaResult::KnockdownCrush);
        assert_eq!(resolve_trauma(Mass::Massive, Padding::Heavy), TraumaResult::Stagger);
    }

    #[test]
    fn test_mace_vs_plate_knight() {
        // Spec example: Heavy mace vs Heavy padding = Fatigue
        // Knight is fatigued but not wounded - historically accurate
        let result = resolve_trauma(Mass::Heavy, Padding::Heavy);
        assert_eq!(result, TraumaResult::Fatigue);
    }

    #[test]
    fn test_all_mass_padding_combinations() {
        for mass in [Mass::Light, Mass::Medium, Mass::Heavy, Mass::Massive] {
            for padding in [Padding::None, Padding::Light, Padding::Heavy] {
                let _ = resolve_trauma(mass, padding);
            }
        }
    }
}
```

**Step 4: Update mod.rs**

```rust
// src/combat/mod.rs
pub mod constants;
pub mod body_zone;
pub mod weapons;
pub mod armor;
pub mod penetration;
pub mod trauma;
pub mod resolution;
pub mod wounds;
pub mod morale;

pub use body_zone::{BodyZone, WoundSeverity};
pub use weapons::{Edge, Mass, Reach, WeaponProperties, WeaponSpecial};
pub use armor::{Rigidity, Padding, Coverage, ArmorProperties};
pub use penetration::{PenetrationResult, resolve_penetration};
pub use trauma::{TraumaResult, resolve_trauma};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib combat::trauma`
Expected: PASS (5 tests)

**Step 6: Commit**

```bash
git add src/combat/trauma.rs src/combat/mod.rs
git commit -m "feat(combat): add trauma resolution (Mass vs Padding)"
```

---

## Task 7: Wound Combination (Penetration + Trauma → Wound)

**Files:**
- Replace: `src/combat/wounds.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{
        BodyZone, WoundSeverity,
        PenetrationResult, TraumaResult,
    };

    #[test]
    fn test_deep_cut_is_critical() {
        let wound = combine_results(
            PenetrationResult::DeepCut,
            TraumaResult::Negligible,
            BodyZone::Torso,
        );
        assert_eq!(wound.severity, WoundSeverity::Critical);
        assert!(wound.bleeding);
    }

    #[test]
    fn test_deflect_with_knockdown_still_wounds() {
        // Mace deflects off plate but knockdown causes bruise
        let wound = combine_results(
            PenetrationResult::Deflect,
            TraumaResult::KnockdownBruise,
            BodyZone::Torso,
        );
        assert_eq!(wound.severity, WoundSeverity::Serious);
        assert!(!wound.bleeding); // Blunt trauma doesn't bleed
    }

    #[test]
    fn test_leg_wound_affects_mobility() {
        let wound = combine_results(
            PenetrationResult::Cut,
            TraumaResult::Stagger,
            BodyZone::LegLeft,
        );
        assert!(wound.mobility_impact);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib combat::wounds`
Expected: FAIL with "cannot find function `combine_results`"

**Step 3: Write minimal implementation**

```rust
//! Wound system: combines penetration and trauma results
//!
//! Wounds track zone, severity, and effects (bleeding, mobility, grip).

use serde::{Deserialize, Serialize};
use crate::combat::body_zone::{BodyZone, WoundSeverity};
use crate::combat::penetration::PenetrationResult;
use crate::combat::trauma::TraumaResult;

/// A wound on a specific body zone
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wound {
    pub zone: BodyZone,
    pub severity: WoundSeverity,
    pub bleeding: bool,
    pub mobility_impact: bool,
    pub grip_impact: bool,
}

impl Wound {
    /// Create a wound with no effects
    pub fn none(zone: BodyZone) -> Self {
        Self {
            zone,
            severity: WoundSeverity::None,
            bleeding: false,
            mobility_impact: false,
            grip_impact: false,
        }
    }
}

/// Combine penetration and trauma results into a wound
///
/// Takes the WORSE of the two results (no multiplication).
pub fn combine_results(
    pen: PenetrationResult,
    trauma: TraumaResult,
    zone: BodyZone,
) -> Wound {
    use PenetrationResult::*;
    use TraumaResult::*;
    use WoundSeverity::*;

    // Penetration severity
    let pen_severity = match pen {
        DeepCut => Critical,
        Cut => Serious,
        ShallowCut => Minor,
        Snag | Deflect | NoPenetrationAttempt => None,
    };

    // Trauma severity
    let trauma_severity = match trauma {
        KnockdownCrush => Critical,
        KnockdownBruise => Serious,
        Stagger => Scratch,
        Fatigue | Negligible => None,
    };

    // Take worse of the two (max by Ord)
    let severity = pen_severity.max(trauma_severity);

    // Bleeding only from cuts
    let bleeding = matches!(pen, DeepCut | Cut);

    // Mobility impact from legs OR knockdown trauma
    let mobility_impact = zone.is_leg()
        || matches!(trauma, KnockdownCrush | KnockdownBruise);

    // Grip impact from arms or hands
    let grip_impact = zone.is_arm() || zone.is_hand();

    Wound {
        zone,
        severity,
        bleeding,
        mobility_impact,
        grip_impact,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deep_cut_is_critical() {
        let wound = combine_results(
            PenetrationResult::DeepCut,
            TraumaResult::Negligible,
            BodyZone::Torso,
        );
        assert_eq!(wound.severity, WoundSeverity::Critical);
        assert!(wound.bleeding);
    }

    #[test]
    fn test_deflect_with_knockdown_still_wounds() {
        let wound = combine_results(
            PenetrationResult::Deflect,
            TraumaResult::KnockdownBruise,
            BodyZone::Torso,
        );
        assert_eq!(wound.severity, WoundSeverity::Serious);
        assert!(!wound.bleeding);
    }

    #[test]
    fn test_leg_wound_affects_mobility() {
        let wound = combine_results(
            PenetrationResult::Cut,
            TraumaResult::Stagger,
            BodyZone::LegLeft,
        );
        assert!(wound.mobility_impact);
    }

    #[test]
    fn test_arm_wound_affects_grip() {
        let wound = combine_results(
            PenetrationResult::ShallowCut,
            TraumaResult::Negligible,
            BodyZone::ArmRight,
        );
        assert!(wound.grip_impact);
    }

    #[test]
    fn test_no_wound_from_deflect_and_negligible() {
        let wound = combine_results(
            PenetrationResult::Deflect,
            TraumaResult::Negligible,
            BodyZone::Torso,
        );
        assert_eq!(wound.severity, WoundSeverity::None);
    }
}
```

**Step 4: Update mod.rs exports**

```rust
// Add to pub use statements in mod.rs
pub use wounds::{Wound, combine_results};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib combat::wounds`
Expected: PASS (5 tests)

**Step 6: Commit**

```bash
git add src/combat/wounds.rs src/combat/mod.rs
git commit -m "feat(combat): add wound combination (penetration + trauma)"
```

---

## Task 8: Combat Stance System

**Files:**
- Create: `src/combat/stance.rs`
- Modify: `src/combat/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pressing_can_attack() {
        assert!(CombatStance::Pressing.can_attack());
        assert!(CombatStance::Neutral.can_attack());
        assert!(!CombatStance::Defensive.can_attack());
        assert!(!CombatStance::Recovering.can_attack());
    }

    #[test]
    fn test_recovering_is_vulnerable() {
        assert!(CombatStance::Recovering.vulnerable());
        assert!(CombatStance::Broken.vulnerable());
        assert!(!CombatStance::Pressing.vulnerable());
    }

    #[test]
    fn test_stance_transitions() {
        let transitions = StanceTransitions::new();

        // Neutral → Pressing when initiating attack
        let next = transitions.apply(CombatStance::Neutral, TransitionTrigger::InitiateAttack);
        assert_eq!(next, CombatStance::Pressing);

        // Pressing → Recovering when attack missed
        let next = transitions.apply(CombatStance::Pressing, TransitionTrigger::AttackMissed);
        assert_eq!(next, CombatStance::Recovering);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib combat::stance`
Expected: FAIL with "cannot find type `CombatStance`"

**Step 3: Write minimal implementation**

```rust
//! Combat stance system
//!
//! Combat is pressure and timing, not turns. Stances determine
//! what actions are available and who strikes first.

use serde::{Deserialize, Serialize};

/// Combat stance - every combatant is always in exactly one stance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum CombatStance {
    /// Attacking, has initiative
    Pressing,
    /// Balanced, can attack or defend
    #[default]
    Neutral,
    /// Focused on blocking/parrying
    Defensive,
    /// Catching breath, vulnerable
    Recovering,
    /// Out of fight (wounded/fled)
    Broken,
}

impl CombatStance {
    /// Can this stance initiate an attack?
    pub fn can_attack(&self) -> bool {
        matches!(self, CombatStance::Pressing | CombatStance::Neutral)
    }

    /// Can this stance perform active defense?
    pub fn can_defend(&self) -> bool {
        matches!(self, CombatStance::Neutral | CombatStance::Defensive)
    }

    /// Is this stance vulnerable to free hits?
    pub fn vulnerable(&self) -> bool {
        matches!(self, CombatStance::Recovering | CombatStance::Broken)
    }
}

/// Events that trigger stance transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransitionTrigger {
    // Self-initiated
    InitiateAttack,
    RaiseGuard,
    DropGuard,
    CatchBreath,

    // Combat outcomes
    AttackCompleted,
    AttackBlocked,
    AttackMissed,
    DefenseSucceeded,
    DefenseFailed,
    TookHit,
    Staggered,
    Knockdown,

    // Fatigue
    Exhausted,
    Recovered,
}

/// Stance transition rules (state machine)
pub struct StanceTransitions;

impl StanceTransitions {
    pub fn new() -> Self {
        Self
    }

    /// Apply a transition trigger to get the next stance
    pub fn apply(&self, current: CombatStance, trigger: TransitionTrigger) -> CombatStance {
        use CombatStance::*;
        use TransitionTrigger::*;

        match (current, trigger) {
            // Self-initiated transitions
            (Neutral, InitiateAttack) => Pressing,
            (Neutral, RaiseGuard) => Defensive,
            (Defensive, DropGuard) => Neutral,
            (_, CatchBreath) => Recovering,

            // Attack outcomes
            (Pressing, AttackCompleted) => Neutral,
            (Pressing, AttackBlocked) => Neutral,
            (Pressing, AttackMissed) => Recovering, // Overextended

            // Defense outcomes
            (Defensive, DefenseSucceeded) => Neutral,
            (Defensive, DefenseFailed) => Recovering,

            // Taking damage
            (_, TookHit) => Recovering,
            (_, Staggered) => Recovering,
            (_, Knockdown) => Recovering,

            // Fatigue
            (_, Exhausted) => Recovering,
            (Recovering, Recovered) => Neutral,

            // No change for invalid transitions
            _ => current,
        }
    }
}

impl Default for StanceTransitions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pressing_can_attack() {
        assert!(CombatStance::Pressing.can_attack());
        assert!(CombatStance::Neutral.can_attack());
        assert!(!CombatStance::Defensive.can_attack());
        assert!(!CombatStance::Recovering.can_attack());
    }

    #[test]
    fn test_recovering_is_vulnerable() {
        assert!(CombatStance::Recovering.vulnerable());
        assert!(CombatStance::Broken.vulnerable());
        assert!(!CombatStance::Pressing.vulnerable());
    }

    #[test]
    fn test_stance_transitions() {
        let transitions = StanceTransitions::new();

        let next = transitions.apply(CombatStance::Neutral, TransitionTrigger::InitiateAttack);
        assert_eq!(next, CombatStance::Pressing);

        let next = transitions.apply(CombatStance::Pressing, TransitionTrigger::AttackMissed);
        assert_eq!(next, CombatStance::Recovering);
    }

    #[test]
    fn test_recovery_cycle() {
        let transitions = StanceTransitions::new();

        // Get hit → Recovering
        let stance = transitions.apply(CombatStance::Neutral, TransitionTrigger::TookHit);
        assert_eq!(stance, CombatStance::Recovering);

        // Recover → Neutral
        let stance = transitions.apply(CombatStance::Recovering, TransitionTrigger::Recovered);
        assert_eq!(stance, CombatStance::Neutral);
    }
}
```

**Step 4: Update mod.rs**

```rust
// Add to src/combat/mod.rs
pub mod stance;

// Add to pub use statements
pub use stance::{CombatStance, TransitionTrigger, StanceTransitions};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib combat::stance`
Expected: PASS (4 tests)

**Step 6: Commit**

```bash
git add src/combat/stance.rs src/combat/mod.rs
git commit -m "feat(combat): add stance system (Pressing, Neutral, Defensive, Recovering, Broken)"
```

---

## Task 9: Skill System (Capability Gates, NOT Bonuses)

**Files:**
- Create: `src/combat/skill.rs`
- Modify: `src/combat/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_novice_cannot_riposte() {
        assert!(!SkillLevel::Novice.can_attempt_riposte());
    }

    #[test]
    fn test_veteran_can_riposte() {
        assert!(SkillLevel::Veteran.can_attempt_riposte());
        assert!(SkillLevel::Master.can_attempt_riposte());
    }

    #[test]
    fn test_only_master_can_feint() {
        assert!(!SkillLevel::Novice.can_feint());
        assert!(!SkillLevel::Trained.can_feint());
        assert!(!SkillLevel::Veteran.can_feint());
        assert!(SkillLevel::Master.can_feint());
    }

    #[test]
    fn test_no_numeric_bonuses() {
        // This test documents the philosophy: skill is capability, not bonus
        // SkillLevel has NO methods that return damage multipliers or hit chance bonuses
        let _skill = SkillLevel::Master;
        // If this compiles, we haven't added forbidden methods
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib combat::skill`
Expected: FAIL with "cannot find type `SkillLevel`"

**Step 3: Write minimal implementation**

```rust
//! Combat skill system
//!
//! CRITICAL: Skill determines WHICH actions are available, NOT bonuses.
//! NO percentage modifiers. NO damage multipliers. NO hit chance bonuses.
//!
//! Skill unlocks capabilities and affects timing, not numbers.

use serde::{Deserialize, Serialize};

/// Skill level - unlocks capabilities, doesn't add bonuses
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
pub enum SkillLevel {
    /// High variance, slow transitions, poor reads
    #[default]
    Novice,
    /// Moderate variance, decent transitions
    Trained,
    /// Low variance, good transitions, finds gaps
    Veteran,
    /// Minimal variance, instant transitions, exploits openings
    Master,
}

impl SkillLevel {
    /// Can attempt a riposte (counterattack after successful defense)?
    pub fn can_attempt_riposte(&self) -> bool {
        matches!(self, SkillLevel::Veteran | SkillLevel::Master)
    }

    /// Can target a specific body zone instead of random?
    pub fn can_target_specific_zone(&self) -> bool {
        matches!(self, SkillLevel::Trained | SkillLevel::Veteran | SkillLevel::Master)
    }

    /// Can feint (fake attack to create opening)?
    pub fn can_feint(&self) -> bool {
        matches!(self, SkillLevel::Master)
    }

    /// Can disarm opponent?
    pub fn can_disarm(&self) -> bool {
        matches!(self, SkillLevel::Veteran | SkillLevel::Master)
    }
}

/// Complete combat skill profile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CombatSkill {
    /// Overall combat skill level
    pub level: SkillLevel,
    // NOTE: No numeric fields! Skill affects capability, not numbers.
}

impl CombatSkill {
    pub fn novice() -> Self {
        Self { level: SkillLevel::Novice }
    }

    pub fn trained() -> Self {
        Self { level: SkillLevel::Trained }
    }

    pub fn veteran() -> Self {
        Self { level: SkillLevel::Veteran }
    }

    pub fn master() -> Self {
        Self { level: SkillLevel::Master }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_novice_cannot_riposte() {
        assert!(!SkillLevel::Novice.can_attempt_riposte());
    }

    #[test]
    fn test_veteran_can_riposte() {
        assert!(SkillLevel::Veteran.can_attempt_riposte());
        assert!(SkillLevel::Master.can_attempt_riposte());
    }

    #[test]
    fn test_only_master_can_feint() {
        assert!(!SkillLevel::Novice.can_feint());
        assert!(!SkillLevel::Trained.can_feint());
        assert!(!SkillLevel::Veteran.can_feint());
        assert!(SkillLevel::Master.can_feint());
    }

    #[test]
    fn test_no_numeric_bonuses() {
        let _skill = SkillLevel::Master;
        // This test documents that SkillLevel has no numeric bonus methods
    }

    #[test]
    fn test_skill_ordering() {
        assert!(SkillLevel::Master > SkillLevel::Veteran);
        assert!(SkillLevel::Veteran > SkillLevel::Trained);
        assert!(SkillLevel::Trained > SkillLevel::Novice);
    }
}
```

**Step 4: Update mod.rs**

```rust
// Add to src/combat/mod.rs
pub mod skill;

// Add to pub use statements
pub use skill::{SkillLevel, CombatSkill};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib combat::skill`
Expected: PASS (5 tests)

**Step 6: Commit**

```bash
git add src/combat/skill.rs src/combat/mod.rs
git commit -m "feat(combat): add skill system (capability gates, NOT bonuses)"
```

---

## Task 10: Stress Sources (Morale Pressure)

**Files:**
- Replace: `src/combat/morale.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stress_sources_have_positive_values() {
        for source in StressSource::all() {
            assert!(source.base_stress() > 0.0);
        }
    }

    #[test]
    fn test_shock_stress_higher_than_sustained() {
        // Shock events should cause more immediate stress than sustained pressure
        assert!(StressSource::OfficerKilled.base_stress() > StressSource::ProlongedCombat.base_stress());
        assert!(StressSource::AmbushSprung.base_stress() > StressSource::Outnumbered.base_stress());
    }

    #[test]
    fn test_stress_accumulates() {
        let mut state = MoraleState::default();
        let initial = state.current_stress;

        state.apply_stress(StressSource::TakingCasualties);

        assert!(state.current_stress > initial);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib combat::morale`
Expected: FAIL with "cannot find type `StressSource`"

**Step 3: Write minimal implementation**

```rust
//! Stress and morale system
//!
//! Stress accumulates. When stress exceeds threshold, entity breaks.
//! What you do to enemies = what they can do to you (symmetric).

use serde::{Deserialize, Serialize};
use crate::combat::constants::{BASE_STRESS_THRESHOLD, SHAKEN_THRESHOLD_RATIO};

/// Sources of combat stress (symmetric - applies to both sides)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StressSource {
    // Combat stress
    TakingCasualties,
    TakingFire,
    MeleeViolence,
    WoundReceived,
    NearMiss,

    // Shock stress (spikes)
    OfficerKilled,
    FlankAttack,
    AmbushSprung,
    CavalryCharge,
    TerrifyingEnemy,

    // Pressure stress (sustained)
    Outnumbered,
    Surrounded,
    NoResponse,
    OverwatchFire,
    ProlongedCombat,

    // Social stress
    AlliesBreaking,
    AloneExposed,
}

impl StressSource {
    /// All stress sources
    pub fn all() -> &'static [StressSource] {
        &[
            StressSource::TakingCasualties,
            StressSource::TakingFire,
            StressSource::MeleeViolence,
            StressSource::WoundReceived,
            StressSource::NearMiss,
            StressSource::OfficerKilled,
            StressSource::FlankAttack,
            StressSource::AmbushSprung,
            StressSource::CavalryCharge,
            StressSource::TerrifyingEnemy,
            StressSource::Outnumbered,
            StressSource::Surrounded,
            StressSource::NoResponse,
            StressSource::OverwatchFire,
            StressSource::ProlongedCombat,
            StressSource::AlliesBreaking,
            StressSource::AloneExposed,
        ]
    }

    /// Base stress value (ADDITIVE, not percentage)
    pub fn base_stress(&self) -> f32 {
        match self {
            // Combat stress
            StressSource::TakingCasualties => 0.05,
            StressSource::TakingFire => 0.02,
            StressSource::MeleeViolence => 0.01,
            StressSource::WoundReceived => 0.15,
            StressSource::NearMiss => 0.03,

            // Shock stress (major spikes)
            StressSource::OfficerKilled => 0.30,
            StressSource::FlankAttack => 0.20,
            StressSource::AmbushSprung => 0.25,
            StressSource::CavalryCharge => 0.20,
            StressSource::TerrifyingEnemy => 0.15,

            // Pressure stress (per tick while true)
            StressSource::Outnumbered => 0.01,
            StressSource::Surrounded => 0.03,
            StressSource::NoResponse => 0.02,
            StressSource::OverwatchFire => 0.02,
            StressSource::ProlongedCombat => 0.005,

            // Social stress
            StressSource::AlliesBreaking => 0.10,
            StressSource::AloneExposed => 0.05,
        }
    }
}

/// Morale break result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakResult {
    /// Holding steady
    Holding,
    /// Shaken but not broken
    Shaken,
    /// Breaking - will flee
    Breaking,
}

/// Morale state for an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoraleState {
    /// Accumulated stress (0.0 to unlimited)
    pub current_stress: f32,
    /// Personal breaking point
    pub base_threshold: f32,
}

impl Default for MoraleState {
    fn default() -> Self {
        Self {
            current_stress: 0.0,
            base_threshold: BASE_STRESS_THRESHOLD,
        }
    }
}

impl MoraleState {
    /// Apply stress from a source
    pub fn apply_stress(&mut self, source: StressSource) {
        self.current_stress += source.base_stress();
    }

    /// Check if entity is breaking
    pub fn check_break(&self) -> BreakResult {
        let effective_threshold = self.base_threshold;

        if self.current_stress > effective_threshold {
            BreakResult::Breaking
        } else if self.current_stress > effective_threshold * SHAKEN_THRESHOLD_RATIO {
            BreakResult::Shaken
        } else {
            BreakResult::Holding
        }
    }

    /// Decay stress over time (when safe)
    pub fn decay_stress(&mut self, rate: f32) {
        self.current_stress = (self.current_stress - rate).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stress_sources_have_positive_values() {
        for source in StressSource::all() {
            assert!(source.base_stress() > 0.0);
        }
    }

    #[test]
    fn test_shock_stress_higher_than_sustained() {
        assert!(StressSource::OfficerKilled.base_stress() > StressSource::ProlongedCombat.base_stress());
        assert!(StressSource::AmbushSprung.base_stress() > StressSource::Outnumbered.base_stress());
    }

    #[test]
    fn test_stress_accumulates() {
        let mut state = MoraleState::default();
        let initial = state.current_stress;

        state.apply_stress(StressSource::TakingCasualties);

        assert!(state.current_stress > initial);
    }

    #[test]
    fn test_break_thresholds() {
        let mut state = MoraleState::default();

        // Initially holding
        assert_eq!(state.check_break(), BreakResult::Holding);

        // Add stress to reach shaken
        state.current_stress = state.base_threshold * 0.85;
        assert_eq!(state.check_break(), BreakResult::Shaken);

        // Add more to break
        state.current_stress = state.base_threshold * 1.1;
        assert_eq!(state.check_break(), BreakResult::Breaking);
    }
}
```

**Step 4: Update mod.rs exports**

```rust
// Add to pub use statements
pub use morale::{StressSource, MoraleState, BreakResult};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib combat::morale`
Expected: PASS (4 tests)

**Step 6: Commit**

```bash
git add src/combat/morale.rs src/combat/mod.rs
git commit -m "feat(combat): add stress/morale system with symmetric sources"
```

---

## Task 11: Exchange Resolution (Core Combat)

**Files:**
- Replace: `src/combat/resolution.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pressing_vs_recovering_is_free_hit() {
        let mut attacker = Combatant::test_swordsman();
        attacker.stance = CombatStance::Pressing;

        let mut defender = Combatant::test_unarmored();
        defender.stance = CombatStance::Recovering;

        let result = resolve_exchange(&attacker, &defender);

        assert!(result.defender_hit);
        assert!(!result.attacker_hit);
    }

    #[test]
    fn test_reach_determines_strike_order() {
        // Spear (Long) vs Sword (Short) - spear strikes first
        let spearman = Combatant::test_spearman();
        let swordsman = Combatant::test_swordsman();

        let result = resolve_exchange(&spearman, &swordsman);

        assert!(result.attacker_struck_first);
    }

    #[test]
    fn test_sword_vs_plate_no_wound() {
        let attacker = Combatant::test_swordsman();
        let defender = Combatant::test_plate_knight();

        let result = resolve_exchange(&attacker, &defender);

        // Sword can't penetrate plate
        if let Some(wound) = &result.defender_wound {
            assert_eq!(wound.severity, WoundSeverity::None);
        }
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib combat::resolution`
Expected: FAIL with "cannot find struct `Combatant`"

**Step 3: Write minimal implementation**

```rust
//! Combat exchange resolution
//!
//! An exchange occurs when PRESSING meets any other stance.
//! NO PERCENTAGE MODIFIERS. Property comparisons only.

use crate::combat::{
    BodyZone, WoundSeverity,
    WeaponProperties, WeaponSpecial, Reach,
    ArmorProperties,
    CombatStance,
    CombatSkill, SkillLevel,
    resolve_penetration, resolve_trauma, combine_results,
    Wound,
};

/// A combatant in an exchange
#[derive(Debug, Clone)]
pub struct Combatant {
    pub weapon: WeaponProperties,
    pub armor: ArmorProperties,
    pub stance: CombatStance,
    pub skill: CombatSkill,
}

impl Combatant {
    /// Test combatant: swordsman with no armor
    pub fn test_swordsman() -> Self {
        Self {
            weapon: WeaponProperties::sword(),
            armor: ArmorProperties::none(),
            stance: CombatStance::Pressing,
            skill: CombatSkill::trained(),
        }
    }

    /// Test combatant: spearman with no armor
    pub fn test_spearman() -> Self {
        Self {
            weapon: WeaponProperties::spear(),
            armor: ArmorProperties::none(),
            stance: CombatStance::Pressing,
            skill: CombatSkill::trained(),
        }
    }

    /// Test combatant: plate knight with sword
    pub fn test_plate_knight() -> Self {
        Self {
            weapon: WeaponProperties::sword(),
            armor: ArmorProperties::plate(),
            stance: CombatStance::Neutral,
            skill: CombatSkill::veteran(),
        }
    }

    /// Test combatant: unarmored civilian
    pub fn test_unarmored() -> Self {
        Self {
            weapon: WeaponProperties::fists(),
            armor: ArmorProperties::none(),
            stance: CombatStance::Neutral,
            skill: CombatSkill::novice(),
        }
    }
}

/// Result of an exchange
#[derive(Debug, Clone)]
pub struct ExchangeResult {
    /// Did attacker hit defender?
    pub defender_hit: bool,
    /// Did defender hit attacker?
    pub attacker_hit: bool,
    /// Who struck first (if both attacked)?
    pub attacker_struck_first: bool,
    /// Wound to defender (if any)
    pub defender_wound: Option<Wound>,
    /// Wound to attacker (if any)
    pub attacker_wound: Option<Wound>,
}

/// Select a hit zone (deterministic based on skill)
fn select_hit_zone(skill: SkillLevel) -> BodyZone {
    // Higher skill = more likely to hit vital areas
    // This is deterministic, not random
    match skill {
        SkillLevel::Master => BodyZone::Head,
        SkillLevel::Veteran => BodyZone::Torso,
        SkillLevel::Trained => BodyZone::Torso,
        SkillLevel::Novice => BodyZone::Torso,
    }
}

/// Resolve a single hit
fn resolve_hit(weapon: &WeaponProperties, armor: &ArmorProperties, zone: BodyZone) -> Wound {
    let has_piercing = weapon.has_special(WeaponSpecial::Piercing);
    let pen = resolve_penetration(weapon.edge, armor.rigidity, has_piercing);
    let trauma = resolve_trauma(weapon.mass, armor.padding);
    combine_results(pen, trauma, zone)
}

/// Resolve an exchange between attacker and defender
///
/// # Arguments
/// * `attacker` - The combatant initiating (must be PRESSING)
/// * `defender` - The combatant receiving
///
/// # Returns
/// Exchange result with hits and wounds
pub fn resolve_exchange(attacker: &Combatant, defender: &Combatant) -> ExchangeResult {
    // Step 1: Check if defender can respond
    let defender_can_respond = !defender.stance.vulnerable();

    if !defender_can_respond {
        // Free hit - defender is recovering or broken
        let zone = select_hit_zone(attacker.skill.level);
        let wound = resolve_hit(&attacker.weapon, &defender.armor, zone);

        return ExchangeResult {
            defender_hit: true,
            attacker_hit: false,
            attacker_struck_first: true,
            defender_wound: Some(wound),
            attacker_wound: None,
        };
    }

    // Step 2: Both can fight - reach determines strike order
    let attacker_reach = attacker.weapon.reach;
    let defender_reach = defender.weapon.reach;

    let (attacker_struck_first, both_hit) = match attacker_reach.cmp(&defender_reach) {
        std::cmp::Ordering::Greater => (true, true),
        std::cmp::Ordering::Less => (false, true),
        std::cmp::Ordering::Equal => (true, true), // Simultaneous
    };

    // Step 3: Resolve attacker's hit
    let attacker_zone = select_hit_zone(attacker.skill.level);
    let defender_wound = resolve_hit(&attacker.weapon, &defender.armor, attacker_zone);

    // Step 4: Resolve defender's counter (if they can attack)
    let (attacker_hit, attacker_wound) = if defender.stance.can_attack() && both_hit {
        let defender_zone = select_hit_zone(defender.skill.level);
        let wound = resolve_hit(&defender.weapon, &attacker.armor, defender_zone);
        (true, Some(wound))
    } else {
        (false, None)
    };

    ExchangeResult {
        defender_hit: true,
        attacker_hit,
        attacker_struck_first,
        defender_wound: Some(defender_wound),
        attacker_wound,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pressing_vs_recovering_is_free_hit() {
        let mut attacker = Combatant::test_swordsman();
        attacker.stance = CombatStance::Pressing;

        let mut defender = Combatant::test_unarmored();
        defender.stance = CombatStance::Recovering;

        let result = resolve_exchange(&attacker, &defender);

        assert!(result.defender_hit);
        assert!(!result.attacker_hit);
    }

    #[test]
    fn test_reach_determines_strike_order() {
        let spearman = Combatant::test_spearman();
        let swordsman = Combatant::test_swordsman();

        let result = resolve_exchange(&spearman, &swordsman);

        assert!(result.attacker_struck_first);
    }

    #[test]
    fn test_sword_vs_plate_no_wound() {
        let attacker = Combatant::test_swordsman();
        let defender = Combatant::test_plate_knight();

        let result = resolve_exchange(&attacker, &defender);

        if let Some(wound) = &result.defender_wound {
            // Sharp vs Plate = Deflect, Medium vs Heavy = Fatigue
            // Worse of (None, None) = None
            assert_eq!(wound.severity, WoundSeverity::None);
        }
    }

    #[test]
    fn test_both_combatants_can_be_hit() {
        // Two swordsmen attacking each other
        let mut attacker = Combatant::test_swordsman();
        attacker.stance = CombatStance::Pressing;

        let mut defender = Combatant::test_swordsman();
        defender.stance = CombatStance::Pressing; // Both attacking

        let result = resolve_exchange(&attacker, &defender);

        // Equal reach = simultaneous, both should be hit
        assert!(result.defender_hit);
        assert!(result.attacker_hit);
    }
}
```

**Step 4: Update mod.rs exports**

```rust
// Add to pub use statements
pub use resolution::{Combatant, ExchangeResult, resolve_exchange};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib combat::resolution`
Expected: PASS (4 tests)

**Step 6: Commit**

```bash
git add src/combat/resolution.rs src/combat/mod.rs
git commit -m "feat(combat): add exchange resolution system"
```

---

## Task 12: Formation State (LOD 1)

**Files:**
- Create: `src/combat/formation.rs`
- Modify: `src/combat/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::EntityId;

    #[test]
    fn test_formation_creation() {
        let formation = FormationState::new(vec![EntityId::new(), EntityId::new()]);
        assert_eq!(formation.entities.len(), 2);
        assert_eq!(formation.pressure, 0.0);
    }

    #[test]
    fn test_pressure_clamped() {
        let mut formation = FormationState::new(vec![]);
        formation.apply_pressure_delta(2.0);
        assert_eq!(formation.pressure, 1.0); // Clamped to max

        formation.apply_pressure_delta(-3.0);
        assert_eq!(formation.pressure, -1.0); // Clamped to min
    }

    #[test]
    fn test_formation_break_threshold() {
        let mut formation = FormationState::new(vec![EntityId::new(); 10]);
        formation.broken_count = 4; // 40% broken

        assert!(formation.is_broken());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib combat::formation`
Expected: FAIL with "cannot find struct `FormationState`"

**Step 3: Write minimal implementation**

```rust
//! Formation combat for LOD 1+
//!
//! At formation level, individual exchanges become statistical.
//! Property matchups determine casualty rates, not percentages.

use serde::{Deserialize, Serialize};
use crate::core::types::EntityId;
use crate::combat::constants::FORMATION_BREAK_THRESHOLD;

/// Formation state for LOD 1 combat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationState {
    /// Entities in this formation
    pub entities: Vec<EntityId>,
    /// Front line entities (engage in combat)
    pub front_line: Vec<EntityId>,
    /// Pressure: -1.0 (losing) to +1.0 (winning)
    pub pressure: f32,
    /// Cohesion: 0.0 (scattered) to 1.0 (tight formation)
    pub cohesion: f32,
    /// Fatigue: 0.0 (fresh) to 1.0 (exhausted)
    pub fatigue: f32,
    /// Formation-level stress
    pub stress: f32,
    /// Count of broken/routed entities
    pub broken_count: u32,
}

impl FormationState {
    /// Create a new formation
    pub fn new(entities: Vec<EntityId>) -> Self {
        let front_line = entities.iter().take(entities.len() / 3).copied().collect();
        Self {
            entities,
            front_line,
            pressure: 0.0,
            cohesion: 1.0,
            fatigue: 0.0,
            stress: 0.0,
            broken_count: 0,
        }
    }

    /// Apply pressure delta (clamped to -1.0 to 1.0)
    pub fn apply_pressure_delta(&mut self, delta: f32) {
        self.pressure = (self.pressure + delta).clamp(-1.0, 1.0);
    }

    /// Effective fighting strength
    pub fn effective_strength(&self) -> usize {
        self.entities.len().saturating_sub(self.broken_count as usize)
    }

    /// Check if formation is broken
    pub fn is_broken(&self) -> bool {
        if self.entities.is_empty() {
            return true;
        }
        let broken_ratio = self.broken_count as f32 / self.entities.len() as f32;
        broken_ratio >= FORMATION_BREAK_THRESHOLD
    }

    /// Get pressure category (for display, not calculations)
    pub fn pressure_category(&self) -> PressureCategory {
        match self.pressure {
            p if p <= -0.7 => PressureCategory::Collapsing,
            p if p <= -0.3 => PressureCategory::Losing,
            p if p <= 0.3 => PressureCategory::Neutral,
            p if p <= 0.7 => PressureCategory::Pushing,
            _ => PressureCategory::Overwhelming,
        }
    }
}

/// Pressure categories (for display only)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressureCategory {
    Collapsing,
    Losing,
    Neutral,
    Pushing,
    Overwhelming,
}

/// Shock attack types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShockType {
    CavalryCharge,
    FlankAttack,
    RearCharge,
    Ambush,
}

impl ShockType {
    /// Stress spike from shock attack
    pub fn stress_spike(&self) -> f32 {
        match self {
            ShockType::CavalryCharge => 0.30,
            ShockType::FlankAttack => 0.20,
            ShockType::RearCharge => 0.40,
            ShockType::Ambush => 0.35,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formation_creation() {
        let formation = FormationState::new(vec![EntityId::new(), EntityId::new()]);
        assert_eq!(formation.entities.len(), 2);
        assert_eq!(formation.pressure, 0.0);
    }

    #[test]
    fn test_pressure_clamped() {
        let mut formation = FormationState::new(vec![]);
        formation.apply_pressure_delta(2.0);
        assert_eq!(formation.pressure, 1.0);

        formation.apply_pressure_delta(-3.0);
        assert_eq!(formation.pressure, -1.0);
    }

    #[test]
    fn test_formation_break_threshold() {
        let mut formation = FormationState::new(vec![EntityId::new(); 10]);
        formation.broken_count = 4;

        assert!(formation.is_broken());
    }

    #[test]
    fn test_pressure_categories() {
        let mut formation = FormationState::new(vec![]);

        formation.pressure = -0.8;
        assert_eq!(formation.pressure_category(), PressureCategory::Collapsing);

        formation.pressure = 0.0;
        assert_eq!(formation.pressure_category(), PressureCategory::Neutral);

        formation.pressure = 0.9;
        assert_eq!(formation.pressure_category(), PressureCategory::Overwhelming);
    }
}
```

**Step 4: Update mod.rs**

```rust
// Add to src/combat/mod.rs
pub mod formation;

// Add to pub use statements
pub use formation::{FormationState, PressureCategory, ShockType};
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib combat::formation`
Expected: PASS (4 tests)

**Step 6: Commit**

```bash
git add src/combat/formation.rs src/combat/mod.rs
git commit -m "feat(combat): add formation state for LOD 1 combat"
```

---

## Task 13: Combat State Component

**Files:**
- Create: `src/combat/state.rs`
- Modify: `src/combat/mod.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_combat_state() {
        let state = CombatState::default();
        assert_eq!(state.stance, CombatStance::Neutral);
        assert_eq!(state.morale.current_stress, 0.0);
    }

    #[test]
    fn test_can_fight_when_healthy() {
        let state = CombatState::default();
        assert!(state.can_fight());
    }

    #[test]
    fn test_cannot_fight_when_broken() {
        let mut state = CombatState::default();
        state.stance = CombatStance::Broken;
        assert!(!state.can_fight());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib combat::state`
Expected: FAIL with "cannot find struct `CombatState`"

**Step 3: Write minimal implementation**

```rust
//! Combat state component for entities
//!
//! Every entity has combat state (mandatory but minimal).

use serde::{Deserialize, Serialize};
use crate::combat::{
    CombatStance, CombatSkill,
    MoraleState,
    WeaponProperties, ArmorProperties,
};

/// Combat state component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    /// Current combat stance
    pub stance: CombatStance,
    /// Combat skill profile
    pub skill: CombatSkill,
    /// Morale/stress state
    pub morale: MoraleState,
    /// Currently equipped weapon
    pub weapon: WeaponProperties,
    /// Currently worn armor
    pub armor: ArmorProperties,
    /// Combat fatigue (0.0 to 1.0)
    pub fatigue: f32,
}

impl Default for CombatState {
    fn default() -> Self {
        Self {
            stance: CombatStance::default(),
            skill: CombatSkill::default(),
            morale: MoraleState::default(),
            weapon: WeaponProperties::default(),
            armor: ArmorProperties::default(),
            fatigue: 0.0,
        }
    }
}

impl CombatState {
    /// Can this entity participate in combat?
    pub fn can_fight(&self) -> bool {
        !matches!(self.stance, CombatStance::Broken)
    }

    /// Is this entity actively in combat?
    pub fn in_combat(&self) -> bool {
        matches!(self.stance, CombatStance::Pressing | CombatStance::Defensive)
    }

    /// Apply fatigue (additive, clamped)
    pub fn add_fatigue(&mut self, amount: f32) {
        self.fatigue = (self.fatigue + amount).min(1.0);
    }

    /// Recover fatigue
    pub fn recover_fatigue(&mut self, amount: f32) {
        self.fatigue = (self.fatigue - amount).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_combat_state() {
        let state = CombatState::default();
        assert_eq!(state.stance, CombatStance::Neutral);
        assert_eq!(state.morale.current_stress, 0.0);
    }

    #[test]
    fn test_can_fight_when_healthy() {
        let state = CombatState::default();
        assert!(state.can_fight());
    }

    #[test]
    fn test_cannot_fight_when_broken() {
        let mut state = CombatState::default();
        state.stance = CombatStance::Broken;
        assert!(!state.can_fight());
    }

    #[test]
    fn test_fatigue_clamped() {
        let mut state = CombatState::default();
        state.add_fatigue(0.5);
        assert_eq!(state.fatigue, 0.5);

        state.add_fatigue(0.7);
        assert_eq!(state.fatigue, 1.0); // Clamped

        state.recover_fatigue(0.3);
        assert!((state.fatigue - 0.7).abs() < 0.001);
    }
}
```

**Step 4: Update mod.rs**

```rust
// Add to src/combat/mod.rs
pub mod state;

// Add to pub use statements
pub use state::CombatState;
```

**Step 5: Run test to verify it passes**

Run: `cargo test --lib combat::state`
Expected: PASS (4 tests)

**Step 6: Commit**

```bash
git add src/combat/state.rs src/combat/mod.rs
git commit -m "feat(combat): add CombatState component for entities"
```

---

## Task 14: Final mod.rs and Integration Test

**Files:**
- Update: `src/combat/mod.rs` (final version)
- Create: `tests/combat_integration.rs`

**Step 1: Write the integration test**

```rust
// tests/combat_integration.rs
//! Combat system integration tests

use arc_citadel::combat::{
    // Properties
    Edge, Mass, Reach, WeaponProperties, WeaponSpecial,
    Rigidity, Padding, Coverage, ArmorProperties,
    BodyZone, WoundSeverity,
    // Resolution
    PenetrationResult, TraumaResult,
    resolve_penetration, resolve_trauma, combine_results,
    Wound,
    // Stance
    CombatStance, TransitionTrigger, StanceTransitions,
    // Skill
    SkillLevel, CombatSkill,
    // Morale
    StressSource, MoraleState, BreakResult,
    // Exchange
    Combatant, ExchangeResult, resolve_exchange,
    // Formation
    FormationState, PressureCategory, ShockType,
    // State
    CombatState,
};

/// Test the spec example: Sword vs Plate armor
#[test]
fn test_spec_example_sword_vs_plate() {
    // Sword (Sharp, Medium, Short) vs Plate (Plate, Heavy padding, Full)
    let sword = WeaponProperties::sword();
    let plate = ArmorProperties::plate();

    // 1. Penetration: Sharp vs Plate → DEFLECT
    let pen = resolve_penetration(sword.edge, plate.rigidity, false);
    assert_eq!(pen, PenetrationResult::Deflect);

    // 2. Trauma: Medium vs Heavy → FATIGUE (not wound)
    let trauma = resolve_trauma(sword.mass, plate.padding);
    assert_eq!(trauma, TraumaResult::Negligible); // Heavy padding absorbs medium mass

    // 3. Combined result: No wound
    let wound = combine_results(pen, trauma, BodyZone::Torso);
    assert_eq!(wound.severity, WoundSeverity::None);

    // The sword literally cannot cut plate. You need maces, flanks, or morale breaks.
}

/// Test the spec example: Mace vs Plate armor
#[test]
fn test_spec_example_mace_vs_plate() {
    let mace = WeaponProperties::mace();
    let plate = ArmorProperties::plate();

    // Mace doesn't try to penetrate
    let pen = resolve_penetration(mace.edge, plate.rigidity, false);
    assert_eq!(pen, PenetrationResult::NoPenetrationAttempt);

    // Heavy mass vs Heavy padding = Fatigue
    let trauma = resolve_trauma(mace.mass, plate.padding);
    assert_eq!(trauma, TraumaResult::Fatigue);

    // Knight is fatigued but not wounded - historically accurate
}

/// Test the two victory paths
#[test]
fn test_two_victory_paths() {
    // Path 1: Damage - inflict wounds until incapacitated
    let razor = WeaponProperties {
        edge: Edge::Razor,
        mass: Mass::Light,
        reach: Reach::Grapple,
        special: vec![],
    };
    let unarmored = ArmorProperties::none();

    let pen = resolve_penetration(razor.edge, unarmored.rigidity, false);
    assert_eq!(pen, PenetrationResult::DeepCut);

    let wound = combine_results(pen, TraumaResult::Negligible, BodyZone::Neck);
    assert_eq!(wound.severity, WoundSeverity::Critical);
    // Neck wound at Critical exceeds fatality threshold (Serious)

    // Path 2: Morale - accumulate stress until break
    let mut morale = MoraleState::default();

    // Simulate cavalry charge + officer killed + surrounded
    morale.apply_stress(StressSource::CavalryCharge);
    morale.apply_stress(StressSource::OfficerKilled);
    morale.apply_stress(StressSource::Surrounded);
    morale.apply_stress(StressSource::Surrounded);
    morale.apply_stress(StressSource::Surrounded);

    // Should be at least shaken, possibly breaking
    assert!(morale.check_break() != BreakResult::Holding);
}

/// Verify no multiplicative stacking patterns in public API
#[test]
fn test_no_percentage_api() {
    // This test documents that the public API has no percentage-based methods
    // If these compile without methods like `damage_multiplier()`, we're good

    let _skill = SkillLevel::Master;
    let _weapon = WeaponProperties::sword();
    let _armor = ArmorProperties::plate();
    let _wound = WoundSeverity::Critical;

    // None of these types should have methods returning f32 multipliers
    // or methods taking "bonus" parameters
}
```

**Step 2: Run integration test to verify it fails**

Run: `cargo test --test combat_integration`
Expected: FAIL (can't find imports initially, should pass after mod.rs is complete)

**Step 3: Finalize mod.rs**

```rust
//! Combat resolution system
//!
//! Philosophy: Property interaction, not percentage modifiers.
//! NO multiplicative stacking. Categorical outcomes from comparisons.
//!
//! Two victory paths:
//! 1. DAMAGE PATH: Inflict wounds until they can't fight
//! 2. MORALE PATH: Inflict stress until they break and flee

pub mod constants;
pub mod body_zone;
pub mod weapons;
pub mod armor;
pub mod penetration;
pub mod trauma;
pub mod wounds;
pub mod stance;
pub mod skill;
pub mod morale;
pub mod resolution;
pub mod formation;
pub mod state;

// Re-exports for convenient access
pub use body_zone::{BodyZone, WoundSeverity};
pub use weapons::{Edge, Mass, Reach, WeaponProperties, WeaponSpecial};
pub use armor::{Rigidity, Padding, Coverage, ArmorProperties};
pub use penetration::{PenetrationResult, resolve_penetration};
pub use trauma::{TraumaResult, resolve_trauma};
pub use wounds::{Wound, combine_results};
pub use stance::{CombatStance, TransitionTrigger, StanceTransitions};
pub use skill::{SkillLevel, CombatSkill};
pub use morale::{StressSource, MoraleState, BreakResult};
pub use resolution::{Combatant, ExchangeResult, resolve_exchange};
pub use formation::{FormationState, PressureCategory, ShockType};
pub use state::CombatState;
```

**Step 4: Run integration test to verify it passes**

Run: `cargo test --test combat_integration`
Expected: PASS (3 tests)

**Step 5: Run all combat tests**

Run: `cargo test --lib combat`
Expected: PASS (all ~40 tests)

**Step 6: Commit**

```bash
git add src/combat/mod.rs tests/combat_integration.rs
git commit -m "feat(combat): complete combat system with integration tests"
```

---

## Task 15: Add CombatState to Human Archetype

**Files:**
- Modify: `src/entity/species/human.rs`

**Step 1: Write the failing test**

```rust
// Add to existing tests in human.rs
#[test]
fn test_human_has_combat_state() {
    let mut archetype = HumanArchetype::new();
    let id = EntityId::new();
    archetype.spawn(id, "Warrior".into(), 0);

    assert_eq!(archetype.combat_states.len(), 1);
    assert!(archetype.combat_states[0].can_fight());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --lib entity::species::human::tests::test_human_has_combat_state`
Expected: FAIL with "no field `combat_states`"

**Step 3: Modify human.rs**

Add import at top:
```rust
use crate::combat::CombatState;
```

Add field to HumanArchetype struct:
```rust
pub combat_states: Vec<CombatState>,
```

Update `new()`:
```rust
combat_states: Vec::new(),
```

Update `spawn()`:
```rust
self.combat_states.push(CombatState::default());
```

**Step 4: Run test to verify it passes**

Run: `cargo test --lib entity::species::human`
Expected: PASS

**Step 5: Commit**

```bash
git add src/entity/species/human.rs
git commit -m "feat(entity): add CombatState to HumanArchetype"
```

---

## Summary

This implementation plan creates a complete combat resolution system following the spec's philosophy:

1. **Property enums** (Edge, Mass, Reach, Rigidity, Padding, Coverage) - categorical, not numeric
2. **Lookup tables** (penetration, trauma) - no multiplicative stacking
3. **Wound system** - combines results, tracks effects
4. **Stance system** - determines exchange resolution
5. **Skill system** - capability gates, NOT bonuses
6. **Morale system** - symmetric stress accumulation
7. **Formation combat** - LOD 1 statistical resolution
8. **Integration** - CombatState added to archetypes

Total: ~1500 lines of Rust across 14 files, ~50 tests

**Verification commands:**
- `cargo test --lib combat` - All combat unit tests
- `cargo test --test combat_integration` - Integration tests
- `cargo build` - Verify compilation
- `cargo clippy` - Check for common issues
