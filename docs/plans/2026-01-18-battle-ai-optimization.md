# Battle AI Optimization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable AI vs AI battles and create a battle focus config for the existing DSPy optimization loop.

**Architecture:** Add `friendly_ai` field to `BattleState`, update `phase_ai` to process both AIs, create a headless battle runner binary, and add a battle-ai focus config with behavioral expectations.

**Tech Stack:** Rust (battle system), Python (existing opt-gameplay tools), TOML (AI personalities)

---

## Task 1: Add friendly_ai to BattleState

**Files:**
- Modify: `src/battle/execution.rs:111-143`

**Step 1: Add the friendly_ai field**

In `BattleState` struct, add `friendly_ai` alongside `enemy_ai`:

```rust
/// Complete battle state
#[derive(Serialize, Deserialize)]
pub struct BattleState {
    // ... existing fields ...

    /// Enemy AI controller (None = player controlled)
    #[serde(skip)]
    pub enemy_ai: Option<Box<dyn BattleAI>>,

    /// Friendly AI controller (None = player controlled)
    #[serde(skip)]
    pub friendly_ai: Option<Box<dyn BattleAI>>,
}
```

**Step 2: Update Debug impl**

Find the Debug impl for BattleState and add friendly_ai:

```rust
impl std::fmt::Debug for BattleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BattleState")
            // ... existing fields ...
            .field(
                "enemy_ai",
                &self.enemy_ai.as_ref().map(|_| "<AI Controller>"),
            )
            .field(
                "friendly_ai",
                &self.friendly_ai.as_ref().map(|_| "<AI Controller>"),
            )
            .finish()
    }
}
```

**Step 3: Update new() and clone initialization**

Find places where `enemy_ai: None` is set and add `friendly_ai: None`:

```rust
// In BattleState::new()
friendly_ai: None,

// In Clone impl (if manual)
friendly_ai: None, // AI is not cloned - must be re-attached
```

**Step 4: Add setter method**

After `set_enemy_ai`, add:

```rust
pub fn set_friendly_ai(&mut self, ai: Option<Box<dyn BattleAI>>) {
    self.friendly_ai = ai;
}
```

**Step 5: Run cargo check**

Run: `cargo check --lib 2>&1 | grep "friendly_ai" | head -5`
Expected: No errors (warnings OK)

**Step 6: Commit**

```bash
git add src/battle/execution.rs
git commit -m "feat(battle): add friendly_ai field to BattleState"
```

---

## Task 2: Update phase_ai to process both AIs

**Files:**
- Modify: `src/battle/execution.rs` (find `fn phase_ai`)

**Step 1: Read current phase_ai implementation**

Find the `phase_ai` method to understand current structure.

**Step 2: Refactor to process both AIs**

Replace the single-AI logic with a loop over both:

```rust
fn phase_ai(&mut self, events: &mut BattleEventLog) {
    // Process enemy AI
    if let Some(ref mut ai) = self.enemy_ai {
        let context = DecisionContext::new(
            &self.enemy_army,
            &self.friendly_army,
            &self.enemy_visibility,
            self.tick,
            ai.ignores_fog_of_war(),
        );
        let orders = ai.process_tick(&context, self.tick, events);
        for order in orders {
            self.courier_system.dispatch_order(
                order,
                self.enemy_army.hq_position,
                self.tick,
                &self.enemy_army.courier_pool,
            );
        }
    }

    // Process friendly AI
    if let Some(ref mut ai) = self.friendly_ai {
        let context = DecisionContext::new(
            &self.friendly_army,
            &self.enemy_army,
            &self.friendly_visibility,
            self.tick,
            ai.ignores_fog_of_war(),
        );
        let orders = ai.process_tick(&context, self.tick, events);
        for order in orders {
            self.courier_system.dispatch_order(
                order,
                self.friendly_army.hq_position,
                self.tick,
                &self.friendly_army.courier_pool,
            );
        }
    }
}
```

**Step 3: Run cargo check**

Run: `cargo check --lib`
Expected: Compiles (warnings OK)

**Step 4: Commit**

```bash
git add src/battle/execution.rs
git commit -m "feat(battle): phase_ai processes both friendly and enemy AI"
```

---

## Task 3: Create headless battle runner binary

**Files:**
- Create: `src/bin/battle_runner.rs`

**Step 1: Create the binary with CLI parsing**

```rust
//! Headless battle runner for AI optimization
//!
//! Runs AI vs AI battles and outputs JSON scores for DSPy optimization.

use arc_citadel::battle::ai::{scoring, AiCommander, load_personality};
use arc_citadel::battle::battle_map::BattleMap;
use arc_citadel::battle::execution::{BattleOutcome, BattleState};
use arc_citadel::battle::units::{Army, ArmyId};
use arc_citadel::core::types::EntityId;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "battle_runner")]
#[command(about = "Run AI vs AI battles for optimization")]
struct Args {
    /// Friendly AI personality name (from data/ai_personalities/)
    #[arg(long, default_value = "default")]
    friendly: String,

    /// Enemy AI personality name
    #[arg(long, default_value = "default")]
    enemy: String,

    /// Map size (hexes across)
    #[arg(long, default_value = "30")]
    map_size: i32,

    /// Max ticks before timeout
    #[arg(long, default_value = "500")]
    max_ticks: u64,

    /// Random seed for reproducibility
    #[arg(long)]
    seed: Option<u64>,

    /// Output format: json or text
    #[arg(long, default_value = "json")]
    format: String,
}

#[derive(Serialize)]
struct BattleResult {
    outcome: String,
    ticks: u64,
    friendly_casualties_pct: f32,
    enemy_casualties_pct: f32,
    score: f32,
    friendly_personality: String,
    enemy_personality: String,
}

fn main() {
    let args = Args::parse();

    // Load personalities
    let friendly_personality = load_personality(&args.friendly)
        .unwrap_or_else(|e| panic!("Failed to load friendly personality '{}': {}", args.friendly, e));
    let enemy_personality = load_personality(&args.enemy)
        .unwrap_or_else(|e| panic!("Failed to load enemy personality '{}': {}", args.enemy, e));

    // Create map
    let map = BattleMap::new(args.map_size, args.map_size);

    // Create armies (minimal for now - can be expanded)
    let friendly_army = create_test_army("Friendly");
    let enemy_army = create_test_army("Enemy");

    // Create battle state
    let mut state = BattleState::new(map, friendly_army, enemy_army);

    // Attach AIs
    state.set_friendly_ai(Some(Box::new(AiCommander::new(friendly_personality.clone()))));
    state.set_enemy_ai(Some(Box::new(AiCommander::new(enemy_personality.clone()))));

    // Run battle
    while state.outcome == BattleOutcome::Undecided && state.tick < args.max_ticks {
        state.run_tick();
    }

    // Calculate score
    let weights = scoring::ScoreWeights::default();
    let battle_score = scoring::calculate_score(&state, &weights, args.max_ticks);

    // Output result
    let result = BattleResult {
        outcome: format!("{:?}", state.outcome),
        ticks: state.tick,
        friendly_casualties_pct: battle_score.friendly_casualties_percent * 100.0,
        enemy_casualties_pct: battle_score.enemy_casualties_percent * 100.0,
        score: battle_score.raw_score,
        friendly_personality: args.friendly,
        enemy_personality: args.enemy,
    };

    match args.format.as_str() {
        "json" => println!("{}", serde_json::to_string_pretty(&result).unwrap()),
        _ => {
            println!("Battle Result:");
            println!("  Outcome: {:?}", state.outcome);
            println!("  Ticks: {}", state.tick);
            println!("  Friendly casualties: {:.1}%", result.friendly_casualties_pct);
            println!("  Enemy casualties: {:.1}%", result.enemy_casualties_pct);
            println!("  Score: {:.2}", result.score);
        }
    }
}

fn create_test_army(name: &str) -> Army {
    use arc_citadel::battle::hex::BattleHexCoord;
    use arc_citadel::battle::units::*;
    use arc_citadel::battle::unit_type::UnitType;

    let mut army = Army::new(ArmyId::new(), EntityId::new());
    army.hq_position = if name == "Friendly" {
        BattleHexCoord::new(5, 15)
    } else {
        BattleHexCoord::new(25, 15)
    };

    // Create a formation with 3 infantry units
    let mut formation = BattleFormation::new(FormationId::new(), EntityId::new());

    for i in 0..3 {
        let mut unit = BattleUnit::new(UnitId::new(), UnitType::Infantry);
        unit.position = if name == "Friendly" {
            BattleHexCoord::new(8 + i, 14 + i)
        } else {
            BattleHexCoord::new(22 - i, 14 + i)
        };
        // Add 50 entities to each unit
        unit.elements.push(Element::new(vec![EntityId::new(); 50]));
        formation.units.push(unit);
    }

    army.formations.push(formation);
    army.courier_pool = vec![EntityId::new(); 3];
    army
}
```

**Step 2: Run cargo check**

Run: `cargo check --bin battle_runner`
Expected: Compiles (may need import fixes)

**Step 3: Fix any import issues**

Adjust imports based on actual module structure. Common fixes:
- `use arc_citadel::battle::ai::scoring` may need `use arc_citadel::battle::ai::scoring::*`
- Check if `AiCommander::new` takes personality by value or reference

**Step 4: Run cargo build**

Run: `cargo build --bin battle_runner`
Expected: Binary builds successfully

**Step 5: Test the binary**

Run: `cargo run --bin battle_runner -- --format text`
Expected: Output showing battle result

**Step 6: Commit**

```bash
git add src/bin/battle_runner.rs
git commit -m "feat(battle): add headless battle_runner binary for AI optimization"
```

---

## Task 4: Create battle-ai focus config

**Files:**
- Create: `data/gameplay_optimization/focuses/battle-ai.json`

**Step 1: Create the focus config**

```json
{
  "name": "battle-ai",
  "description": "How the battle AI makes tactical decisions",
  "expectations": [
    {
      "id": "B1",
      "behavior": "AI retreats when strength ratio drops below retreat_threshold",
      "eval_hint": "Check for retreat orders when friendly/enemy strength ratio < 0.3"
    },
    {
      "id": "B2",
      "behavior": "AI prioritizes attacking weak/damaged enemy units",
      "eval_hint": "Attack orders should target units with lowest effective_strength"
    },
    {
      "id": "B3",
      "behavior": "AI exploits flanking opportunities when available",
      "eval_hint": "Units should maneuver to attack from multiple directions"
    },
    {
      "id": "B4",
      "behavior": "AI preserves reserves until main assault phase",
      "eval_hint": "Reserve units should not engage in opening phase"
    },
    {
      "id": "B5",
      "behavior": "AI responds to routing units by attempting rally",
      "eval_hint": "Rally orders issued for broken units when commander nearby"
    },
    {
      "id": "B6",
      "behavior": "Aggressive personality attacks more frequently than cautious",
      "eval_hint": "Compare attack order frequency between personality types"
    },
    {
      "id": "B7",
      "behavior": "AI uses couriers efficiently (doesn't spam orders)",
      "eval_hint": "Order frequency should match re_evaluation_interval"
    }
  ],
  "tunable_constants": [
    {
      "file": "src/battle/ai/personality.rs",
      "name": "default aggression",
      "current": "0.5",
      "range": "0.0-1.0"
    },
    {
      "file": "src/battle/ai/personality.rs",
      "name": "default caution",
      "current": "0.5",
      "range": "0.0-1.0"
    },
    {
      "file": "src/battle/ai/personality.rs",
      "name": "retreat_threshold",
      "current": "0.3",
      "range": "0.1-0.5"
    },
    {
      "file": "src/battle/ai/personality.rs",
      "name": "casualty_threshold",
      "current": "0.5",
      "range": "0.3-0.7"
    },
    {
      "file": "src/battle/ai/personality.rs",
      "name": "flanking_value",
      "current": "1.2",
      "range": "1.0-2.0"
    },
    {
      "file": "src/battle/ai/personality.rs",
      "name": "attack_value",
      "current": "1.0",
      "range": "0.5-2.0"
    },
    {
      "file": "src/battle/ai/personality.rs",
      "name": "defense_value",
      "current": "1.0",
      "range": "0.5-2.0"
    },
    {
      "file": "src/battle/ai/personality.rs",
      "name": "reserve_value",
      "current": "0.8",
      "range": "0.3-1.5"
    }
  ],
  "systems": [
    "src/battle/ai/commander.rs",
    "src/battle/ai/personality.rs",
    "src/battle/ai/decision_context.rs",
    "src/battle/ai/phase_plans.rs",
    "src/battle/execution.rs"
  ]
}
```

**Step 2: Verify JSON is valid**

Run: `python3 -c "import json; json.load(open('data/gameplay_optimization/focuses/battle-ai.json'))"`
Expected: No output (success)

**Step 3: Commit**

```bash
git add data/gameplay_optimization/focuses/battle-ai.json
git commit -m "feat(optimize): add battle-ai focus config for AI optimization"
```

---

## Task 5: Add verbose logging to battle_runner for evaluation

**Files:**
- Modify: `src/bin/battle_runner.rs`

**Step 1: Add --verbose flag**

Add to Args struct:

```rust
/// Enable verbose battle logging for evaluation
#[arg(long, short)]
verbose: bool,
```

**Step 2: Add logging during battle loop**

Replace the battle loop with verbose logging:

```rust
// Run battle
while state.outcome == BattleOutcome::Undecided && state.tick < args.max_ticks {
    if args.verbose {
        eprintln!("=== Tick {} ===", state.tick);
        eprintln!("Friendly strength: {}/{}",
            state.friendly_army.effective_strength(),
            state.friendly_army.total_strength());
        eprintln!("Enemy strength: {}/{}",
            state.enemy_army.effective_strength(),
            state.enemy_army.total_strength());
    }

    let events_before = state.battle_log.len();
    state.run_tick();

    if args.verbose {
        // Print new events
        for event in state.battle_log.iter().skip(events_before) {
            eprintln!("  [{}] {:?}: {}", event.tick, event.event_type, event.description);
        }
    }
}
```

**Step 3: Test verbose output**

Run: `cargo run --bin battle_runner -- --verbose --format text 2>&1 | head -50`
Expected: Tick-by-tick battle log visible

**Step 4: Commit**

```bash
git add src/bin/battle_runner.rs
git commit -m "feat(battle): add verbose logging to battle_runner for optimization"
```

---

## Task 6: Test full optimization loop

**Step 1: Run a battle and capture output**

```bash
cargo run --bin battle_runner -- --verbose --friendly aggressive --enemy cautious 2>&1 | tee /tmp/battle_output.txt
```

**Step 2: Evaluate against battle-ai focus**

```bash
cd data/gameplay_optimization/tools
source .venv/bin/activate
opt-gameplay evaluate -f ../focuses/battle-ai.json -s /tmp/battle_output.txt -o /tmp/battle_eval.json
```

**Step 3: Review evaluation results**

```bash
cat /tmp/battle_eval.json | python3 -m json.tool
```

Expected: Structured verdicts for each expectation (B1-B7)

**Step 4: Generate proposals for any misses**

```bash
opt-gameplay propose -e /tmp/battle_eval.json -f ../focuses/battle-ai.json
```

**Step 5: Document results**

Add session notes to this plan or create a new session doc.

---

## Summary

| Task | Description | Key File |
|------|-------------|----------|
| 1 | Add friendly_ai field | `src/battle/execution.rs` |
| 2 | Update phase_ai for both AIs | `src/battle/execution.rs` |
| 3 | Create battle_runner binary | `src/bin/battle_runner.rs` |
| 4 | Create battle-ai focus config | `data/gameplay_optimization/focuses/battle-ai.json` |
| 5 | Add verbose logging | `src/bin/battle_runner.rs` |
| 6 | Test full loop | Integration test |

**DSPy Optimization Targets (10 floats):**
- `BehaviorConfig`: aggression, caution, initiative, cunning
- `WeightConfig`: attack_value, defense_value, flanking_value, reserve_value, retreat_threshold, casualty_threshold
