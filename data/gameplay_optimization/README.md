# Gameplay Optimization System

> LLM-driven self-improving optimization loop for Arc Citadel using LangChain for evaluation and DSPy for learnable proposal generation.

## Overview

This system creates a feedback loop that:
1. **Evaluates** simulation output against behavioral expectations (LangChain + Pydantic)
2. **Proposes** code fixes for failed expectations (DSPy ChainOfThought)
3. **Learns** from successful fixes to improve future proposals (DSPy BootstrapFewShot)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        OPTIMIZATION LOOP                                     │
│                                                                              │
│   ┌──────────┐    ┌───────────┐    ┌──────────┐    ┌───────────────────┐   │
│   │Simulation│───▶│ Evaluator │───▶│ Proposer │───▶│ Human Applies Fix │   │
│   │  (Rust)  │    │(LangChain)│    │  (DSPy)  │    │                   │   │
│   └──────────┘    └───────────┘    └──────────┘    └─────────┬─────────┘   │
│        ▲                                                      │             │
│        │                                                      │             │
│        └──────────────────────────────────────────────────────┘             │
│                              Re-run & iterate                               │
│                                                                              │
│                              ┌───────────┐                                  │
│                              │  Learner  │◀── changelog.json                │
│                              │  (DSPy)   │                                  │
│                              └─────┬─────┘                                  │
│                                    │                                        │
│                                    ▼                                        │
│                         compiled_proposer.json                              │
│                     (improves future proposals)                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Quick Start

```bash
# Install the tools
cd data/gameplay_optimization/tools
pip install -e .

# Run a simulation and capture output
cargo run 2>&1 | tee /tmp/sim_output.txt

# Evaluate against expectations
opt-gameplay evaluate \
  -f focuses/action-selection.json \
  -s /tmp/sim_output.txt \
  -o /tmp/eval.json

# Generate fix proposals
opt-gameplay propose \
  -e /tmp/eval.json \
  -f focuses/action-selection.json

# After 3+ successful fixes, train the proposer
opt-gameplay learn --changelog changelog.json
```

## Directory Structure

```
data/gameplay_optimization/
├── README.md                   # This file
├── changelog.json              # Training data: successful fixes with metrics
├── focuses/                    # Focus configs (expectations + tunables)
│   ├── action-selection.json
│   ├── need-satisfaction.json
│   ├── perception-filtering.json
│   ├── thought-quality.json
│   └── social-behavior.json
├── backups/                    # Code snapshots before changes
└── tools/                      # Python package
    ├── pyproject.toml
    ├── compiled_proposer.json  # Trained DSPy module (after learning)
    ├── .env                    # API keys (GOOGLE_API_KEY, DEEPSEEK_API_KEY)
    └── src/opt_gameplay/
        ├── schema.py           # Pydantic models
        ├── evaluator.py        # LangChain evaluation
        ├── proposer.py         # DSPy proposal generation
        ├── learner.py          # DSPy training
        └── cli.py              # Typer CLI
```

## Components

### Evaluator (LangChain)

Analyzes simulation output against behavioral expectations using structured parsing.

**Tech:** LangChain + PydanticOutputParser + Gemini 2.0 Flash

```python
# Returns structured verdicts
{
    "verdicts": [
        {
            "expectation_id": "E1",
            "verdict": "HIT",      # HIT | PARTIAL | MISS
            "observed": "5 different actions observed",
            "reasoning": "Variety requirement met",
            "severity": 3
        }
    ],
    "hit_rate": "5/7",
    "summary": "Core behaviors working, social needs underutilized",
    "worst_miss": "E6"
}
```

### Proposer (DSPy)

Generates fix hypotheses using DSPy's ChainOfThought for step-by-step reasoning.

**Tech:** DSPy 2.5+ with BootstrapFewShot optimization

**DSPy Signature:**
```python
class GenerateFix(dspy.Signature):
    # Inputs
    failed_expectation: InputField   # What failed and how
    tunable_constants: InputField    # Constants that can be adjusted
    system_files: InputField         # Relevant Rust files

    # Outputs (learned by DSPy)
    reasoning: OutputField           # Step-by-step diagnosis
    file_path: OutputField           # File to modify
    change_description: OutputField  # What to change
    code_snippet: OutputField        # Proposed code
    confidence: OutputField          # 0-100 confidence
```

**DSPy Module:**
```python
class FixProposer(dspy.Module):
    def __init__(self):
        self.generate = dspy.ChainOfThought(GenerateFix)

    def forward(self, failed_expectation, tunable_constants, system_files):
        return self.generate(...)
```

### Learner (DSPy)

Trains the proposer on successful fixes using BootstrapFewShot optimization.

**Training Data:** `changelog.json` entries where `after_hit_rate > before_hit_rate`

**Process:**
1. Extract successful fixes from changelog
2. Convert to DSPy Examples with input/output pairs
3. Enrich with real focus file data (tunable constants, system files)
4. Optimize using BootstrapFewShot (creates few-shot demos)
5. Save compiled module to `compiled_proposer.json`

**Metric Function:**
```python
def metric_fn(example, prediction, trace=None) -> bool:
    # Validates proposal quality:
    # - Has required fields (file_path, change_description)
    # - File path ends with .rs (Rust file)
    # - Reasoning is substantive (>20 chars)
    return all_checks_pass
```

## Focus Configs

Each focus defines an optimization target:

```json
{
  "name": "action-selection",
  "description": "How entities choose actions based on needs and values",
  "expectations": [
    {
      "id": "E1",
      "behavior": "Entities perform at least 3 different action types",
      "eval_hint": "Count unique ActionId values in log"
    }
  ],
  "tunable_constants": [
    {
      "file": "src/entity/needs.rs",
      "name": "HAS_CRITICAL_THRESHOLD",
      "current": "0.8",
      "range": "0.5-0.95"
    }
  ],
  "systems": [
    "src/simulation/action_select.rs",
    "src/actions/catalog.rs"
  ]
}
```

**Available Focuses:**
| Focus | Expectations | Description |
|-------|-------------|-------------|
| action-selection | 7 | Action variety, critical needs, idle behavior |
| need-satisfaction | 7 | Decay rates, proactive/reactive, safety |
| perception-filtering | 7 | Threat perception, value-driven noticing |
| thought-quality | 8 | Generation, intensity, decay, buffer |
| social-behavior | 8 | Relationships, memory, social actions |

## CLI Commands

```bash
# Evaluate simulation against focus expectations
opt-gameplay evaluate -f <focus.json> -s <sim_output.txt> [-o output.json]

# Generate fix proposals for failed expectations
opt-gameplay propose -e <eval.json> -f <focus.json> [-o proposals.json]

# Train proposer on successful fixes (requires 3+ examples)
opt-gameplay learn [-c changelog.json]

# Compare two evaluations
opt-gameplay diff -b <before.json> -a <after.json>
```

## Training Data Format

`changelog.json` tracks optimization history:

```json
[
  {
    "date": "2026-01-15",
    "focus": "action-selection",
    "hypothesis": "Allow critical needs to interrupt idle tasks",
    "file": "src/simulation/tick.rs",
    "change_summary": "Modified parallel/sequential processing paths to check for critical needs before skipping action selection",
    "before_hit_rate": "1/7",
    "after_hit_rate": "3/5",
    "notes": "E4, E7 untestable (no combat). E6 missed (social decay too slow)."
  }
]
```

## Environment Variables

```bash
# Required: LLM API keys
GOOGLE_API_KEY=...       # For Gemini (default)
DEEPSEEK_API_KEY=...     # For DeepSeek (alternative)

# Optional: Model selection
DSPY_MODEL=gemini        # or "deepseek"
```

## Integration with Arc Citadel

The optimization tools are invoked via the `/optimize-gameplay` Claude command:

1. User runs `/optimize-gameplay` with a focus area
2. Claude runs simulation and captures output
3. Evaluator produces structured verdicts
4. Proposer generates hypotheses for misses
5. User selects fix to apply
6. Claude applies fix and re-runs
7. Improvement tracked in changelog
8. After 3+ successes, proposer is trained

## Current Status

| Component | Status | Notes |
|-----------|--------|-------|
| Schema | Complete | Pydantic models for all outputs |
| Evaluator | Complete | LangChain + Gemini 2.0 Flash |
| Proposer | Complete | DSPy ChainOfThought |
| Learner | Complete | BootstrapFewShot optimizer |
| CLI | Complete | evaluate, propose, learn, diff |
| Training Data | 3 entries | First training run complete |

## Recent Results

**2026-01-15 Action Selection Session:**
- Before: 1/7 hit rate (14%) - entities stuck in idle tasks
- After: 3/5 testable hit rate (60%) - critical needs now interrupt idle
- Root cause: `IdleWander`/`IdleObserve` never complete, blocked action selection
- Fix: Allow critical needs (>0.8) to clear idle tasks before pushing response

## Limitations

- **Context limits:** Evaluator truncates simulation output to 30KB
- **Proxy metric:** Learner can't run actual simulation during training
- **Manual changelog:** Entries require manual JSON editing after successful fixes
- **Confidence parsing:** Proposer may output non-numeric confidence (falls back to 50)
