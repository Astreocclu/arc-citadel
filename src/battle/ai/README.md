# Battle AI Module

Enemy AI system for battle decision-making with configurable personalities.

## Overview

**Architecture: Trait + Data Hybrid**

The AI system combines behavioral configuration with a trait-based interface:

- `trait BattleAI` - Defines the interface for swappable AI implementations
- `AiPersonality` - TOML-loaded configuration holding weights and preferences
- `DecisionContext` - Provides fog-of-war-filtered view of battle state
- `PhasePlanManager` - Handles multi-phase battle strategies with automatic transitions

```
                    ┌─────────────────┐
                    │   AiCommander   │
                    │ (BattleAI impl) │
                    └────────┬────────┘
                             │
         ┌───────────────────┼───────────────────┐
         ▼                   ▼                   ▼
┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐
│  AiPersonality  │ │ DecisionContext │ │PhasePlanManager │
│   (TOML data)   │ │ (filtered view) │ │  (transitions)  │
└─────────────────┘ └─────────────────┘ └─────────────────┘
```

## Quick Start

```rust
use arc_citadel::battle::ai::{load_personality, AiCommander};

// Load personality from TOML file
let personality = load_personality("aggressive")?;

// Create AI commander
let ai = AiCommander::new(personality);

// Attach to battle state
battle_state.set_enemy_ai(Some(Box::new(ai)));
```

## Personality Configuration

Personalities are TOML files stored in `data/ai_personalities/`. Each personality defines four configuration sections:

### Full TOML Structure

```toml
# Behavioral tendencies (0.0 to 1.0)
[behavior]
aggression = 0.8    # 0.0 = defensive, 1.0 = aggressive
caution = 0.2       # 0.0 = reckless, 1.0 = cautious
initiative = 0.9    # 0.0 = reactive, 1.0 = proactive
cunning = 0.4       # Tendency to use deception (feints, fake retreats)

# Decision weights for evaluating options
[weights]
attack_value = 1.5      # Priority for attacking weak/vulnerable units
defense_value = 0.6     # Priority for defending key positions
flanking_value = 1.4    # Priority for flanking opportunities
reserve_value = 0.4     # Priority for preserving reserves
retreat_threshold = 0.15 # Strength ratio below which to retreat
casualty_threshold = 0.7 # Casualty percentage to trigger withdrawal

# Tactical preferences
[preferences]
preferred_range = "close"   # "close", "medium", or "ranged"
reserve_usage = "early"     # "early", "conservative", or "desperate"
re_evaluation_interval = 5  # Ticks between tactical re-evaluations

# Difficulty modifiers
[difficulty]
ignores_fog_of_war = false  # If true, AI sees all enemy units
reaction_delay = 1          # Ticks before reacting (0 = instant)
mistake_chance = 0.15       # Probability of skipping an order (0.0-1.0)
```

## Available Presets

| Preset | Style | Difficulty | Key Traits |
|--------|-------|------------|------------|
| `default` | Balanced | Medium | Standard all-around commander |
| `aggressive` | Attack-focused | Medium | High aggression, low caution, early reserves |
| `cautious` | Defense-focused | Medium | High caution, ranged preference, preserves forces |
| `cunning` | Flanking/Exploit | Medium | High flanking priority, exploits weaknesses |
| `easy` | Slow/Predictable | Easy | Slow reactions, frequent mistakes, low initiative |
| `hard` | Fast/Precise | Hard | Ignores fog, instant reactions, rare mistakes |

Load any preset by name:
```rust
let personality = load_personality("cunning")?;
```

## Fog of War

By default, AI respects fog of war. The `DecisionContext` filters enemy unit visibility:

```rust
// AI only sees enemies in hexes visible to its army
let visible_enemies = context.visible_enemy_units();

// Check if a specific position is visible
if context.is_visible(target_position) {
    // Can act on information at this position
}

// Strength ratio calculated from VISIBLE enemies only
let ratio = context.strength_ratio();
```

### Cheating AI

Set `ignores_fog_of_war = true` in the difficulty config to create an AI that sees all enemy units regardless of visibility. This is useful for harder difficulty levels:

```toml
[difficulty]
ignores_fog_of_war = true  # Sees through fog of war
```

## Phase Plans

AI can execute multi-phase battle strategies that automatically transition based on conditions:

```rust
use arc_citadel::battle::ai::{PhasePlan, PhasePlanManager, PhaseTransition};

let mut manager = PhasePlanManager::new();

// Opening phase: probe and position
manager.add_phase(PhasePlan {
    name: "Probe".to_string(),
    reserve_commitment: 0.0,
    aggression_modifier: -0.2,  // More cautious initially
    priority_targets: vec![],
    transition: PhaseTransition::TimeElapsed(50),  // After 50 ticks
});

// Main assault phase
manager.add_phase(PhasePlan {
    name: "Main Assault".to_string(),
    reserve_commitment: 0.7,   // Commit 70% of reserves
    aggression_modifier: 0.3,  // More aggressive
    priority_targets: vec![objective_hex],
    transition: PhaseTransition::CasualtiesExceed(0.4),  // If 40% casualties
});

// Withdrawal phase
manager.add_phase(PhasePlan {
    name: "Withdrawal".to_string(),
    reserve_commitment: 0.0,
    aggression_modifier: -0.5,  // Defensive
    priority_targets: vec![],
    transition: PhaseTransition::Never,  // Final phase
});

commander.set_phase_manager(manager);
```

### Phase Transition Types

| Transition | Description |
|------------|-------------|
| `TimeElapsed(ticks)` | Transition after N ticks in current phase |
| `StrengthRatioBelow(threshold)` | When own/enemy strength ratio drops below threshold |
| `CasualtiesExceed(percentage)` | When casualty percentage exceeds threshold |
| `Manual` | Only transition via explicit command |
| `Never` | Final phase - never transitions |

## Integration

The AI runs in `phase_ai()` at the start of each tick, before movement and combat:

```
run_tick()
  |
  +-> phase_ai()          <- AI evaluates situation, dispatches orders via courier
  |
  +-> phase_pre_tick()    <- Pre-tick housekeeping
  |
  +-> phase_movement()    <- Couriers advance, orders delivered, units move
  |
  +-> phase_combat()      <- Combat resolution
  |
  +-> phase_morale()      <- Morale checks
  |
  +-> phase_rout()        <- Routing unit movement
  |
  +-> phase_post_tick()   <- Cleanup and victory checks
```

The AI issues orders through the courier system, so there is a realistic delay between AI decisions and unit responses based on courier travel time.

## Testing

Run AI-specific tests:

```bash
# Run all AI module tests
cargo test --lib battle::ai

# Run specific submodule tests
cargo test --lib battle::ai::personality
cargo test --lib battle::ai::decision_context
cargo test --lib battle::ai::phase_plans
cargo test --lib battle::ai::commander

# Run AI integration tests
cargo test --test battle_integration test_ai
```

## Module Structure

```
src/battle/ai/
  mod.rs              # Module root, BattleAI trait definition
  personality.rs      # AiPersonality, TOML loading
  decision_context.rs # Fog-of-war filtered battle view
  phase_plans.rs      # Multi-phase battle planning
  commander.rs        # AiCommander implementation
  README.md           # This documentation

data/ai_personalities/
  default.toml        # Balanced preset
  aggressive.toml     # Attack-focused preset
  cautious.toml       # Defense-focused preset
  cunning.toml        # Flanking/exploitation preset
  easy.toml           # Easy difficulty preset
  hard.toml           # Hard difficulty preset
```
