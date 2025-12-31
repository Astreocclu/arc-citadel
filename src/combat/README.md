# Combat Module

> Combat resolution, weapons, armor, wounds, and morale. Behavior emerges from property interactions.

## Module Structure

```
combat/
├── mod.rs          # Module exports
├── resolution.rs   # Combat resolution (stub)
├── weapons.rs      # Weapon properties (stub)
├── armor.rs        # Armor properties (stub)
├── wounds.rs       # Wound system (stub)
└── morale.rs       # Morale system (stub)
```

## Status: Stub Implementation

This module is planned but not yet implemented. The design follows Arc Citadel's core principle: **behavior emerges from property interactions**.

## Planned Design

### Combat Resolution Flow

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
│  Strength   │ ← Physical capability
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Impact    │ = weapon × strength × fatigue
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Armor     │ ← Coverage, material, condition
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Wound     │ ← Location, severity, type
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Morale    │ ← Pain, fear, resolve
└─────────────┘
```

### Property Interaction Examples

```rust
// Impact force from attack
fn calculate_impact(
    strength: f32,
    weapon_weight: f32,
    swing_momentum: f32,
    fatigue: f32,
) -> f32 {
    let base_force = strength * weapon_weight * swing_momentum;
    base_force * (1.0 - fatigue * 0.3)
}

// Armor penetration
fn calculate_penetration(
    impact: f32,
    armor_thickness: f32,
    armor_condition: f32,
    angle_of_impact: f32,
) -> f32 {
    let effective_armor = armor_thickness * armor_condition * angle_of_impact;
    (impact - effective_armor).max(0.0)
}

// Wound severity
fn calculate_wound(
    penetration: f32,
    body_part: BodyPart,
    weapon_type: WeaponType,
) -> Wound {
    // Severity emerges from interaction of factors
    // Different weapon types cause different wound types
}
```

### Morale System

Morale emerges from:
- Current wounds and pain
- Nearby allies and enemies
- Recent combat outcomes
- Entity values (courage, loyalty)

```rust
fn update_morale(
    entity: &Entity,
    nearby_allies: usize,
    nearby_enemies: usize,
    recent_wounds: &[Wound],
    values: &HumanValues,
) -> f32 {
    // Morale is a function of all these factors
    // Breaking point depends on values
}
```

## Integration Points

### With `entity/body.rs`
- Wounds affect BodyState
- Pain affects action capability
- Fatigue affects combat effectiveness

### With `entity/needs.rs`
- Combat increases safety need
- Wounds increase rest need
- Victory may satisfy purpose need

### With `simulation/action_select.rs`
- Low morale triggers Flee action
- High threat triggers defensive actions

## Future Implementation

When implementing this module:

1. **Start with weapons and armor** as data structures
2. **Implement resolution** as property interaction
3. **Add wounds** that affect body state
4. **Add morale** that affects action selection

## Testing Strategy

- Unit tests for each property interaction
- Integration tests for combat outcomes
- Property tests for edge cases (extreme values)
