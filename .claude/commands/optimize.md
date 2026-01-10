---
description: Iterative optimization loop for game system quality and feel
argument-hint: <system-name> e.g. "chunk system for combat"
---

# Optimize System

Interactive optimization loop for gameplay quality and emergent behavior in a 10k+ entity simulation.

## Input

System to optimize: $ARGUMENTS

## The Loop

### Step 1: Run and Observe

```bash
cd /home/astre/arc-citadel
cargo run 2>&1 | tee /tmp/sim_output.txt
```

Watch the emergent behavior. Look at aggregate patterns, not individuals.

### Step 2: Analyze

Compare actual vs intended:
- Read design docs in `docs/`
- Check if emergent behavior matches design intent
- Identify what feels wrong at the macro level

Write observations:
```
OBSERVED: [What's happening across the population]
INTENDED: [What the design docs say should happen]
GAP: [The disconnect]
HYPOTHESIS: [Why the system produces this]
```

### Step 3: Propose Changes

Propose a coherent set of changes that address the gap. This may touch multiple files, constants, or systems. Group related changes together.

```
CHANGES:
1. [file]: [change] - [why]
2. [file]: [change] - [why]
3. [file]: [change] - [why]

EXPECTED EMERGENT EFFECT: [What population behavior should shift]
```

### Step 4: Ask Player

Use AskUserQuestion:
- "Apply these changes and run again"
- "I'll test it myself first"
- "Show me the current implementation before deciding"
- "Different approach"
- "Feels good, stop"

### Step 5: If Player Tests

```
To test: cargo run
Watch for: <specific behavior to observe>
Tell me: Does it feel right now? What's still off?
```

Wait for feedback.

### Step 6: If Applying

1. Backup touched files to `data/optimization_backups/`
2. Apply all changes
3. Log iteration to `data/optimization_backups/changelog.json`
4. Return to Step 1

## Changelog

```json
{
  "timestamp": "...",
  "system": "<system>",
  "iteration": 1,
  "observation": "<what felt wrong>",
  "changes": ["<file>: <summary>", ...],
  "player_feedback": "<after testing>"
}
```

## Exit

Player says it feels right, or player says stop. No other exit condition.
