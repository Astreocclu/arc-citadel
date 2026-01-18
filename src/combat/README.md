# Combat Module

> Combat resolution, weapons, armor, wounds, and morale. Behavior emerges from property interactions.

## Module Structure (2128 LOC total)

```
combat/
├── mod.rs          # Module exports (39 re-exported items)
├── resolution.rs   # Combat resolution - resolve_exchange() (6821 LOC)
├── weapons.rs      # Weapon properties and types
├── armor.rs        # Armor properties and coverage
├── wounds.rs       # Wound system and severity
├── trauma.rs       # Trauma calculation
├── penetration.rs  # Armor penetration mechanics
├── morale.rs       # Morale system
├── body_zone.rs    # Body part targeting
├── formation.rs    # Combat formation effects
├── skill.rs        # Combat skill levels
├── stance.rs       # Combat stances (aggressive, defensive, etc.)
├── state.rs        # Combat state tracking
├── constants.rs    # Combat constants
├── adapter.rs      # Integration with entity system
└── equipment.rs    # Equipment properties
```

## Status: COMPLETE IMPLEMENTATION

All combat subsystems are implemented and wired into the simulation tick.

## Core Functions

### Combat Resolution

```rust
pub fn resolve_exchange(attacker: &Combatant, defender: &Combatant) -> ExchangeResult
```

Called from `simulation/tick.rs` (line ~2280). Resolves a combat exchange between two combatants.

### Supporting Functions

```rust
pub fn resolve_penetration(impact: f32, armor: &ArmorProperties) -> PenetrationResult
pub fn resolve_trauma(penetration: f32, zone: BodyZone) -> TraumaResult
pub fn combine_results(penetration: PenetrationResult, trauma: TraumaResult) -> Wound
```

## Key Types

### Combatant

```rust
pub struct Combatant {
    pub skill: CombatSkill,
    pub stance: CombatStance,
    pub weapon: WeaponProperties,
    pub armor: ArmorProperties,
    pub fatigue: f32,
}
```

### ExchangeResult

```rust
pub struct ExchangeResult {
    pub attacker_wound: Option<Wound>,
    pub defender_wound: Option<Wound>,
}
```

### Wound

```rust
pub struct Wound {
    pub zone: BodyZone,
    pub severity: WoundSeverity,
    pub bleed_rate: f32,
}
```

## Combat Resolution Flow

```
Attacker Intent
      │
      ▼
┌─────────────┐
│   Weapon    │ ← Weight, reach, speed
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Skill +    │ ← Combat skill level
│  Stance     │ ← Aggressive/Defensive
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Impact    │ = weapon × skill × fatigue
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ Penetration │ ← resolve_penetration()
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Trauma    │ ← resolve_trauma()
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Wound     │ ← combine_results()
└─────────────┘
```

## Integration with Simulation

In `simulation/tick.rs` (around line 2280):

```rust
let exchange = resolve_exchange(&attacker, &defender);

// Apply wounds to defender
if let Some(wound) = &exchange.defender_wound {
    if wound.severity != WoundSeverity::None {
        let fatigue_increase = match wound.severity {
            WoundSeverity::Light => 0.1,
            WoundSeverity::Moderate => 0.2,
            WoundSeverity::Severe => 0.4,
            WoundSeverity::Critical => 0.6,
            WoundSeverity::None => 0.0,
        };
        world.humans.body_states[defender_idx].fatigue =
            (world.humans.body_states[defender_idx].fatigue + fatigue_increase).min(1.0);
    }
}
```

## Enums

### WoundSeverity
`None`, `Light`, `Moderate`, `Severe`, `Critical`

### BodyZone
`Head`, `Torso`, `LeftArm`, `RightArm`, `LeftLeg`, `RightLeg`

### CombatStance
`Aggressive`, `Balanced`, `Defensive`, `Reckless`

### CombatSkill
`Untrained`, `Novice`, `Competent`, `Skilled`, `Expert`, `Master`

## Testing

```bash
cargo test --lib combat::
```

Tests cover:
- Individual combat resolution
- Wound severity calculation
- Armor penetration mechanics
- Stance modifiers
