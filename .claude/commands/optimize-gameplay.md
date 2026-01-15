---
description: LLM-driven gameplay optimization loop using predefined focus configs
argument-hint: <focus-name> e.g. "combat-urgency", "social-behavior"
---

# Optimize Gameplay Systems

LLM-driven optimization loop for Arc Citadel's emergent systems.

## Focus Area

Target: $ARGUMENTS

**Available focuses:**
```bash
ls data/gameplay_optimization/focuses/
```

If no argument or invalid, list available focuses and ask user to pick one.

## The Loop

### Step 1: Load Focus Config

```bash
cat data/gameplay_optimization/focuses/<focus-name>.json
```

Display to user:
- Description
- Systems that may be tuned
- Expectations to evaluate against

If expectations are all PLACEHOLDER, tell user: "This focus needs expectations defined. Rant at me about what good [focus] looks like and I'll populate it."

### Step 2: Run Scenario

Run the setup command from the focus config (or default `cargo run`):

```bash
cd /home/astre/arc-citadel
<setup.command> 2>&1 | tee /tmp/sim_output.txt
```

Capture output relevant to the expectations.

### Step 3: Evaluate Against Expectations

Run automated evaluation:

```bash
cd /home/astre/arc-citadel
opt-gameplay evaluate \
  --focus data/gameplay_optimization/focuses/<focus-name>.json \
  --sim-output /tmp/sim_output.txt \
  --output /tmp/current_eval.json
```

Display the report to user, highlighting:
- Hit rate (X/Y)
- Each verdict with reasoning
- Worst miss (if any)

If hit rate >= 80%, congratulate and ask if user wants to continue optimizing or stop.

### Step 4: Generate Fix Proposals

For MISS or PARTIAL verdicts, generate proposals:

```bash
opt-gameplay propose \
  --eval /tmp/current_eval.json \
  --focus data/gameplay_optimization/focuses/<focus-name>.json \
  --output /tmp/proposals.json
```

Display proposals ranked by confidence:

```
HYPOTHESIS [id]: [change_description]
FILE: [file_path]
REASONING: [reasoning]
CONFIDENCE: [X/100]
CODE: [code_snippet]
```

If multiple hypotheses exist, present them as a table:

| Option | Change | Expected Effect | Rating |
|--------|--------|-----------------|--------|
| A | [change] | [effect] | X/100 |
| B | [change] | [effect] | X/100 |

### Step 6: Ask User

Use AskUserQuestion:
- "Apply and re-run"
- "Show me the current code first"
- "Try different hypothesis"
- "Stop, good enough"

### Step 7: If Applying

1. Save current eval for comparison: `cp /tmp/current_eval.json /tmp/prev_eval.json`
2. Backup: `cp <file> data/gameplay_optimization/backups/$(date +%Y%m%d%H%M%S)_$(basename <file>)`
3. Apply the change
4. Return to Step 2 (run simulation and evaluate)
5. After re-running simulation and evaluation, compare results:

```bash
opt-gameplay diff \
  --before /tmp/prev_eval.json \
  --after /tmp/current_eval.json
```

6. Log to changelog:
```bash
# Append to data/gameplay_optimization/changelog.json
{
  "timestamp": "<now>",
  "focus": "<focus-name>",
  "iteration": <n>,
  "before_hit_rate": "X/Y",
  "hypothesis": "<hypothesis>",
  "file": "<file>",
  "change_summary": "<what changed>",
  "after_hit_rate": "X/Y"
}
```

## Populating Expectations

When user rants about what good gameplay looks like, parse their rant into structured expectations:

```json
{
  "id": "E1",
  "behavior": "<concrete observable behavior>",
  "eval_hint": "<what to look for in simulation output>",
  "weight": 1.0
}
```

Update the focus file with new expectations. Confirm with user before saving.

## Success Criteria

- 80%+ hit rate on expectations
- OR user says stop
- OR 10 iterations without improvement

## Periodic Training

After 3+ successful optimization sessions, train the proposer:

```bash
opt-gameplay learn --changelog data/gameplay_optimization/changelog.json
```

This improves future proposal quality by learning from what worked.

## Key Rules

1. **One change per iteration** - isolate effects
2. **Stay within focus config** - don't wander to other systems
3. **Backup before modifying** - always
4. **Revert if worse** - restore from backup immediately
5. **Log everything** - changelog is the memory
