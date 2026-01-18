# 06-BALANCE-COMPOSITION-SPEC
> Emergent balance philosophy: NO percentage modifiers, property composition only

## Overview

Arc Citadel achieves game balance through the **composition of physical properties**, not through percentage modifiers or flat bonuses. This specification defines the philosophy, implementation patterns, and examples of emergent balance.

---

## Core Principle

**All game effects emerge from the interaction of physical properties.**

```rust
// ✅ CORRECT: Physical property interaction
let impact_force = strength * weapon_mass * swing_velocity;
let penetration = calculate_penetration(impact_force, armor_thickness, material_hardness);

// ❌ FORBIDDEN: Percentage modifiers
let damage = base_damage * weapon_modifier * armor_reduction; // NEVER DO THIS
```

This isn't just an aesthetic choice—it produces fundamentally different gameplay:

| Percentage Systems | Property Composition |
|-------------------|---------------------|
| Predictable outcomes | Probabilistic outcomes |
| Flat power curves | Emergent power curves |
| Designer-tuned balance | Physics-emergent balance |
| "+20% damage" bonuses | Heavier weapon, faster swing |
| Stacking multipliers | Diminishing physical returns |

---

## Forbidden Patterns

### Pattern 1: Percentage Modifiers

```rust
// ❌ FORBIDDEN
struct Weapon {
    damage_multiplier: f32,  // NO
    crit_chance: f32,        // NO
    armor_penetration_percent: f32, // NO
}

// ✅ CORRECT
struct Weapon {
    mass: f32,           // kg - affects momentum
    length: f32,         // m - affects reach and leverage
    edge_angle: f32,     // degrees - affects cutting
    material: Material,  // determines hardness, density
}
```

### Pattern 2: Flat Stat Bonuses

```rust
// ❌ FORBIDDEN
impl Skill {
    fn bonus(&self) -> i32 {
        self.level * 5  // "+5 attack per level" - NO
    }
}

// ✅ CORRECT
impl Skill {
    /// Higher skill = more chunked sequences = faster execution
    fn action_chunks(&self) -> usize {
        match self.level {
            0..=2 => 5,   // Novice: 5 mental steps per action
            3..=5 => 3,   // Journeyman: 3 steps
            6..=8 => 2,   // Expert: 2 steps
            _ => 1,       // Master: single chunked action
        }
    }
}
```

### Pattern 3: Damage Reduction

```rust
// ❌ FORBIDDEN
fn apply_armor(damage: f32, armor: &Armor) -> f32 {
    damage * (1.0 - armor.reduction_percent)  // NO
}

// ✅ CORRECT
fn armor_interaction(impact: f32, armor: &Armor) -> ArmorResult {
    let penetration_prob = penetration_probability(impact, armor.effective_thickness());

    if rand::random::<f32>() < penetration_prob {
        ArmorResult::Penetrated {
            remaining_force: impact * (1.0 - armor.absorption_ratio())
        }
    } else {
        ArmorResult::Blocked {
            blunt_transfer: impact * armor.blunt_transfer_ratio()
        }
    }
}
```

---

## Property Composition Examples

### Example 1: Weapon Damage

Damage emerges from the physical properties of the weapon and wielder:

```rust
pub struct WeaponPhysics {
    pub mass: f32,           // kg
    pub length: f32,         // m (affects leverage)
    pub balance_point: f32,  // 0.0-1.0 (toward hilt vs blade)
    pub edge_geometry: EdgeGeometry,
    pub material: Material,
}

pub fn calculate_impact(
    weapon: &WeaponPhysics,
    wielder_strength: f32,
    wielder_skill: f32,
    swing_type: SwingType,
) -> f32 {
    // Moment of inertia affects how fast the weapon can swing
    let moment = weapon.mass * weapon.balance_point.powi(2) * weapon.length.powi(2);

    // Angular velocity depends on strength and weapon inertia
    let max_angular_velocity = wielder_strength / moment;

    // Skill affects how close to max velocity the wielder achieves
    let skill_efficiency = 0.3 + (wielder_skill * 0.07); // 30% base + 7% per skill level
    let actual_angular_velocity = max_angular_velocity * skill_efficiency.min(1.0);

    // Linear velocity at impact point
    let impact_velocity = actual_angular_velocity * weapon.length * (1.0 - weapon.balance_point);

    // Momentum at impact
    let effective_mass = weapon.mass * (1.0 - weapon.balance_point);
    let momentum = effective_mass * impact_velocity;

    // Edge geometry affects force concentration
    let edge_multiplier = weapon.edge_geometry.force_concentration();

    momentum * edge_multiplier
}
```

**Emergent Consequences**:
- Heavy weapons hit harder but swing slower
- Well-balanced weapons (balance_point ~0.5) swing faster
- Longer weapons have more reach but more inertia
- Strong wielders get more from heavy weapons than weak wielders
- Skilled wielders extract more of a weapon's potential

### Example 2: Armor Penetration

Armor doesn't "reduce damage by X%"—it physically blocks or fails to block:

```rust
pub struct ArmorPhysics {
    pub thickness: f32,      // mm
    pub material: Material,
    pub coverage: f32,       // 0.0-1.0 (how much body covered)
    pub condition: f32,      // 0.0-1.0 (wear reduces effectiveness)
}

pub fn penetration_probability(impact_force: f32, armor: &ArmorPhysics) -> f32 {
    let effective_thickness = armor.thickness
        * armor.material.hardness()
        * armor.condition;

    // Sigmoid curve: gradual transition from "bounces off" to "punches through"
    let ratio = impact_force / effective_thickness;
    let steepness = 10.0; // Tunable: higher = sharper transition

    1.0 / (1.0 + (-steepness * (ratio - 1.0)).exp())
}

pub fn resolve_hit(impact: f32, armor: &ArmorPhysics) -> HitResult {
    if rand::random::<f32>() >= armor.coverage {
        // Hit unarmored area
        return HitResult::DirectHit { force: impact };
    }

    let pen_prob = penetration_probability(impact, armor);

    if rand::random::<f32>() < pen_prob {
        // Penetrated
        let remaining = impact * (1.0 - armor.material.absorption());
        HitResult::Penetrated { force: remaining }
    } else {
        // Blocked - but blunt force transfers
        let blunt = impact * armor.material.blunt_transfer();
        HitResult::Blocked { blunt_force: blunt }
    }
}
```

**Emergent Consequences**:
- The same attack might penetrate or bounce off
- Worn armor is less protective
- Partial coverage means some hits bypass armor entirely
- Even blocked hits transfer some force (bruising, knockback)
- Heavy armor with good coverage is very protective but has tradeoffs (fatigue, mobility)

### Example 3: Fatigue Effects

Fatigue doesn't "reduce stats by X%"—it affects physical capabilities:

```rust
pub struct FatigueState {
    pub exertion: f32,       // 0.0-1.0 (how tired)
    pub recovery_rate: f32,  // Based on endurance
}

impl FatigueState {
    /// Returns the force multiplier from fatigue
    /// Not a percentage reduction—models muscle exhaustion
    pub fn strength_available(&self) -> f32 {
        // Exponential decay: small fatigue has small effect,
        // high fatigue has dramatic effect
        (-2.0 * self.exertion).exp()
    }

    /// Returns reaction time multiplier
    /// Fatigue slows reactions non-linearly
    pub fn reaction_multiplier(&self) -> f32 {
        1.0 + (self.exertion * self.exertion * 2.0)
    }

    /// Accumulate fatigue from exertion
    pub fn exert(&mut self, effort: f32, endurance: f32) {
        let fatigue_gain = effort / endurance;
        self.exertion = (self.exertion + fatigue_gain).min(1.0);
    }

    /// Recover fatigue over time
    pub fn recover(&mut self, dt: f32) {
        let recovery = self.recovery_rate * dt;
        self.exertion = (self.exertion - recovery).max(0.0);
    }
}
```

**Emergent Consequences**:
- Fresh combatants hit much harder than exhausted ones
- Heavy weapons exhaust wielders faster
- Endurance stat affects sustainability, not direct combat power
- Resting mid-combat is tactically valuable
- Prolonged fights favor the more enduring side

---

## Material System

Materials have physical properties that compose into item behavior:

```rust
#[derive(Clone, Copy)]
pub struct Material {
    pub density: f32,      // kg/m³
    pub hardness: f32,     // 0.0-1.0 (resistance to deformation)
    pub toughness: f32,    // 0.0-1.0 (resistance to fracture)
    pub edge_retention: f32, // 0.0-1.0 (how long edge stays sharp)
}

impl Material {
    pub const IRON: Material = Material {
        density: 7874.0,
        hardness: 0.4,
        toughness: 0.6,
        edge_retention: 0.3,
    };

    pub const STEEL: Material = Material {
        density: 7850.0,
        hardness: 0.7,
        toughness: 0.7,
        edge_retention: 0.6,
    };

    pub const BRONZE: Material = Material {
        density: 8800.0,
        hardness: 0.35,
        toughness: 0.5,
        edge_retention: 0.4,
    };

    pub const LEATHER: Material = Material {
        density: 860.0,
        hardness: 0.1,
        toughness: 0.8,
        edge_retention: 0.0,
    };

    /// How much force is absorbed on impact
    pub fn absorption(&self) -> f32 {
        self.toughness * 0.3
    }

    /// How much blunt force transfers through
    pub fn blunt_transfer(&self) -> f32 {
        1.0 - (self.hardness * 0.5)
    }
}
```

**Emergent Material Behaviors**:
- Steel: Hard enough to block well, tough enough not to shatter
- Iron: Cheaper but dulls quickly, less hard
- Bronze: Heavier (more momentum) but softer (dulls faster)
- Leather: Light, flexible, but only stops glancing blows

---

## Skill as Chunking

Skills don't provide bonuses—they reduce cognitive load:

```rust
pub struct SkillLevel(pub u8); // 0-10

impl SkillLevel {
    /// How many mental "chunks" needed to perform an action
    /// Masters have automated sequences into single thoughts
    pub fn chunks_required(&self, action: &ActionType) -> usize {
        let base_chunks = action.base_complexity();

        match self.0 {
            0 => base_chunks * 2,      // Novice: everything is conscious
            1..=3 => base_chunks,      // Learning: normal complexity
            4..=6 => base_chunks / 2,  // Competent: some automation
            7..=9 => base_chunks / 4,  // Expert: highly automated
            10 => 1,                   // Master: single chunked action
        }
    }

    /// Time per chunk (reaction + execution)
    pub fn chunk_time(&self) -> f32 {
        match self.0 {
            0 => 0.5,      // Novice: slow, deliberate
            1..=3 => 0.3,  // Learning: still thinking
            4..=6 => 0.2,  // Competent: smoother
            7..=9 => 0.15, // Expert: fluid
            10 => 0.1,     // Master: instantaneous
        }
    }

    /// Total action time
    pub fn action_time(&self, action: &ActionType) -> f32 {
        self.chunks_required(action) as f32 * self.chunk_time()
    }
}
```

**Example: Sword Attack**

| Skill Level | Chunks | Time/Chunk | Total Time |
|-------------|--------|------------|------------|
| Novice (0) | 10 | 0.5s | 5.0s |
| Learning (2) | 5 | 0.3s | 1.5s |
| Competent (5) | 2 | 0.2s | 0.4s |
| Master (10) | 1 | 0.1s | 0.1s |

The master's attack is 50x faster—not because of a "+50x speed bonus" but because they've chunked 10 steps into 1.

---

## Balance Emergence

### Why This Works

Traditional balance requires designer tuning:
```
"Weapon A does too much damage" → reduce damage_multiplier
"Armor B is too strong" → reduce damage_reduction
```

Property composition self-balances:
```
Heavy weapon: More damage, but slower, more tiring
Thick armor: More protection, but heavier, more fatiguing
High strength: More damage, but doesn't help defense
High endurance: Sustainable, but doesn't hit harder
```

Every advantage has a cost that emerges from physics.

### Emergent Counter-Play

| Strategy | Counter | Why (Physics) |
|----------|---------|---------------|
| Heavy armor | Blunt weapons | Transfer force even when blocked |
| Heavy weapons | Fast opponents | Can't hit what you can't catch |
| High endurance | Burst damage | Can't outlast if dead first |
| Light armor + speed | Area denial | Can't dodge everything |

These counters aren't designed—they emerge from how properties interact.

---

## Implementation Checklist

When implementing any game system, verify:

1. **No percentage modifiers**: All effects come from property changes
2. **No flat bonuses**: Stats don't add, they compose
3. **Physical grounding**: Can you explain the effect in physics terms?
4. **Emergent tradeoffs**: Does every advantage have a cost?
5. **Composition clarity**: Are property interactions understandable?

### Review Questions

For any proposed mechanic, ask:

- "What physical properties cause this effect?"
- "How would this scale at extremes?"
- "What's the tradeoff for this advantage?"
- "Can a player understand why this happened?"

---

## Related Specifications

| Document | Relationship |
|----------|--------------|
| [01-GAME-DESIGN-DOCUMENT](01-GAME-DESIGN-DOCUMENT.md) | Emergent Balance pillar |
| [05-BATTLE-SYSTEM-SPEC](05-BATTLE-SYSTEM-SPEC.md) | Combat property composition |
| [14-PHYSICS-COMBAT-SPEC](14-PHYSICS-COMBAT-SPEC.md) | Detailed physics formulas |
| [19-HIERARCHICAL-CHUNKING-SPEC](19-HIERARCHICAL-CHUNKING-SPEC.md) | Skill system details |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification created |
