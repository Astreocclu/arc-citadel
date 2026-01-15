# LangChain + DSPy Gameplay Optimization Tools

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create Python tools that Claude calls during `/optimize-gameplay` to evaluate simulation output with structured reasoning, propose code fixes, and self-improve prompts over time.

**Architecture:**
- `evaluator.py`: LangChain + Pydantic for structured verdict parsing (HIT/PARTIAL/MISS with reasoning)
- `proposer.py`: DSPy ChainOfThought for generating code change hypotheses
- `learner.py`: DSPy BootstrapFewShot that trains on successful fixes from changelog.json
- CLI via Typer, callable from Claude's bash commands

**Tech Stack:** Python 3.11+, LangChain, DSPy, Pydantic, Typer, Google Generative AI

---

## Task 1: Create Project Structure

**Files:**
- Create: `data/gameplay_optimization/tools/pyproject.toml`
- Create: `data/gameplay_optimization/tools/src/opt_gameplay/__init__.py`
- Create: `data/gameplay_optimization/tools/.env.example`

**Step 1: Create directory structure**

```bash
mkdir -p data/gameplay_optimization/tools/src/opt_gameplay
```

**Step 2: Create pyproject.toml**

Create `data/gameplay_optimization/tools/pyproject.toml`:

```toml
[project]
name = "opt-gameplay"
version = "0.1.0"
description = "LLM-driven gameplay optimization tools for Arc Citadel"
requires-python = ">=3.11"
dependencies = [
    "langchain>=0.3.0",
    "langchain-google-genai>=2.0.0",
    "dspy-ai>=2.5.0",
    "pydantic>=2.0.0",
    "typer>=0.12.0",
    "python-dotenv>=1.0.0",
]

[project.scripts]
opt-gameplay = "opt_gameplay.cli:app"

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"

[tool.hatch.build.targets.wheel]
packages = ["src/opt_gameplay"]
```

**Step 3: Create __init__.py**

Create `data/gameplay_optimization/tools/src/opt_gameplay/__init__.py`:

```python
"""LLM-driven gameplay optimization tools."""
__version__ = "0.1.0"
```

**Step 4: Create .env.example**

Create `data/gameplay_optimization/tools/.env.example`:

```bash
GOOGLE_API_KEY=your-api-key-here
```

**Step 5: Verify structure**

Run: `ls -la data/gameplay_optimization/tools/`
Expected: pyproject.toml, src/, .env.example

**Step 6: Commit**

```bash
git add data/gameplay_optimization/tools/
git commit -m "feat(opt-tools): scaffold Python project structure"
```

---

## Task 2: Define Schema Models

**Files:**
- Create: `data/gameplay_optimization/tools/src/opt_gameplay/schema.py`
- Test: Manual import test

**Step 1: Create schema.py with Pydantic models**

Create `data/gameplay_optimization/tools/src/opt_gameplay/schema.py`:

```python
"""Pydantic models for structured LLM outputs.

Why Pydantic: LangChain's PydanticOutputParser ensures LLM responses
match our expected structure, catching malformed outputs early.
"""

from enum import Enum
from typing import Optional
from pydantic import BaseModel, Field


class Verdict(str, Enum):
    """Evaluation outcome for an expectation."""
    HIT = "HIT"
    PARTIAL = "PARTIAL"
    MISS = "MISS"


class ExpectationVerdict(BaseModel):
    """Result of evaluating one expectation against simulation output.

    Why separate model: Each expectation gets independent analysis,
    making it easy to identify which specific behaviors need fixing.
    """
    expectation_id: str = Field(description="E1, E2, etc from focus config")
    verdict: Verdict
    observed: str = Field(description="What actually happened in the simulation")
    reasoning: str = Field(description="Why this verdict, 1-2 sentences")
    severity: int = Field(
        default=5,
        ge=1,
        le=10,
        description="Impact if not fixed: 1=minor, 10=game-breaking"
    )


class EvaluationReport(BaseModel):
    """Complete evaluation of simulation against all expectations.

    Why hit_rate as string: Preserves "5/7" format for human readability
    while still being parseable.
    """
    verdicts: list[ExpectationVerdict]
    hit_rate: str = Field(description="X/Y format, e.g. '5/7'")
    summary: str = Field(description="Overall assessment, 2-3 sentences")
    worst_miss: Optional[str] = Field(
        default=None,
        description="ID of most severe miss, if any"
    )


class Hypothesis(BaseModel):
    """A proposed code change to fix a missed expectation.

    Why include reasoning: DSPy can learn which reasoning patterns
    lead to successful fixes, improving future proposals.
    """
    target_expectation: str = Field(description="Which expectation this fixes")
    file_path: str = Field(description="Rust file to modify")
    change_description: str = Field(description="What to change, 1-2 sentences")
    code_snippet: str = Field(description="Suggested code or diff")
    reasoning: str = Field(description="Why this change should work")
    confidence: int = Field(
        ge=0,
        le=100,
        description="How confident the model is this will help"
    )


class ProposalReport(BaseModel):
    """Collection of hypotheses for fixing evaluation misses."""
    hypotheses: list[Hypothesis]
    recommended: Optional[str] = Field(
        default=None,
        description="ID of best hypothesis to try first"
    )
```

**Step 2: Verify models are valid**

Run:
```bash
cd data/gameplay_optimization/tools
python3 -c "from src.opt_gameplay.schema import EvaluationReport, Verdict; print('Schema OK')"
```
Expected: `Schema OK`

**Step 3: Commit**

```bash
git add data/gameplay_optimization/tools/src/opt_gameplay/schema.py
git commit -m "feat(opt-tools): add Pydantic schema models for structured outputs"
```

---

## Task 3: Implement Evaluator

**Files:**
- Create: `data/gameplay_optimization/tools/src/opt_gameplay/evaluator.py`
- Test: Integration test with mock data

**Step 1: Create evaluator.py**

Create `data/gameplay_optimization/tools/src/opt_gameplay/evaluator.py`:

```python
"""Evaluation engine using LangChain for structured verdict parsing.

Why LangChain: PydanticOutputParser guarantees structured JSON output,
and ChatPromptTemplate makes prompts maintainable and testable.
"""

import json
import os
from pathlib import Path

from dotenv import load_dotenv
from langchain_core.output_parsers import PydanticOutputParser
from langchain_core.prompts import ChatPromptTemplate
from langchain_google_genai import ChatGoogleGenerativeAI

from .schema import EvaluationReport, ExpectationVerdict, Verdict

load_dotenv()

# Prompt template explains the task and format to the LLM
EVAL_PROMPT = ChatPromptTemplate.from_template("""You are evaluating a game simulation against behavioral expectations.

## Expectations to Check
{expectations}

## Simulation Output
{sim_output}

## Your Task
For each expectation, determine if the simulation output demonstrates the expected behavior.

Verdicts:
- HIT: Behavior clearly present
- PARTIAL: Behavior somewhat present but not fully
- MISS: Behavior absent or opposite

Be concise in reasoning. Focus on observable evidence from the output.

{format_instructions}
""")


class GameplayEvaluator:
    """Evaluates simulation output against focus config expectations.

    Why separate class: Encapsulates LLM setup and provides clean
    interface for CLI. Can be mocked for testing.
    """

    def __init__(self, api_key: str | None = None):
        """Initialize with Gemini model.

        Args:
            api_key: Google API key. Falls back to GOOGLE_API_KEY env var.
        """
        key = api_key or os.getenv("GOOGLE_API_KEY")
        if not key:
            raise ValueError("GOOGLE_API_KEY required")

        # Temperature 0 for consistent, deterministic evaluation
        self.llm = ChatGoogleGenerativeAI(
            model="gemini-3-pro-preview",
            google_api_key=key,
            temperature=0,
        )
        self.parser = PydanticOutputParser(pydantic_object=EvaluationReport)

    def evaluate(
        self,
        focus_path: Path,
        sim_output_path: Path,
        max_output_chars: int = 30000,
    ) -> EvaluationReport:
        """Run evaluation and return structured report.

        Args:
            focus_path: Path to focus config JSON
            sim_output_path: Path to simulation output text
            max_output_chars: Truncate sim output to avoid context overflow

        Returns:
            EvaluationReport with verdicts for each expectation
        """
        # Load inputs
        with open(focus_path) as f:
            focus = json.load(f)
        with open(sim_output_path) as f:
            sim_output = f.read()[:max_output_chars]

        # Format expectations for prompt
        expectations_text = "\n".join(
            f"- {e['id']}: {e['behavior']} (eval hint: {e['eval_hint']})"
            for e in focus.get("expectations", [])
        )

        # Build and invoke chain
        chain = EVAL_PROMPT | self.llm | self.parser

        report = chain.invoke({
            "expectations": expectations_text,
            "sim_output": sim_output,
            "format_instructions": self.parser.get_format_instructions(),
        })

        # Calculate hit rate if not provided
        if not report.hit_rate:
            hits = sum(1 for v in report.verdicts if v.verdict == Verdict.HIT)
            report.hit_rate = f"{hits}/{len(report.verdicts)}"

        return report


def evaluate_simulation(
    focus_path: str,
    sim_output_path: str,
    api_key: str | None = None,
) -> dict:
    """Convenience function for CLI.

    Returns dict for easy JSON serialization.
    """
    evaluator = GameplayEvaluator(api_key=api_key)
    report = evaluator.evaluate(Path(focus_path), Path(sim_output_path))
    return report.model_dump()
```

**Step 2: Create .env file with API key**

Create `data/gameplay_optimization/tools/.env`:

```bash
GOOGLE_API_KEY=AIzaSyCqSGfQv_J2ta86gTJVYaN1xVWVQrE9zVg
```

**Step 3: Test evaluator with mock data**

Create temporary test files:
```bash
# Create mock sim output
echo "Tick 100: Marcus chose Rest (fatigue 0.9)
Tick 101: Elena chose Gather (food need 0.85)
Tick 102: Marcus chose TalkTo Elena (social need 0.6)
10 humans observed, 4 different actions chosen" > /tmp/test_sim.txt
```

Run:
```bash
cd data/gameplay_optimization/tools
python3 -c "
from src.opt_gameplay.evaluator import evaluate_simulation
import json
result = evaluate_simulation(
    '../focuses/action-selection.json',
    '/tmp/test_sim.txt'
)
print(json.dumps(result, indent=2))
"
```
Expected: JSON output with verdicts for E1-E7

**Step 4: Commit**

```bash
git add data/gameplay_optimization/tools/src/opt_gameplay/evaluator.py
git add data/gameplay_optimization/tools/.env
git commit -m "feat(opt-tools): implement LangChain evaluator with Gemini backend"
```

---

## Task 4: Implement Proposer (DSPy)

**Files:**
- Create: `data/gameplay_optimization/tools/src/opt_gameplay/proposer.py`

**Step 1: Create proposer.py**

Create `data/gameplay_optimization/tools/src/opt_gameplay/proposer.py`:

```python
"""Hypothesis generation using DSPy for learnable prompts.

Why DSPy: Unlike static prompts, DSPy can optimize its prompts
based on which proposals actually improved the simulation.
The ChainOfThought module encourages step-by-step reasoning.
"""

import json
import os
from pathlib import Path

import dspy
from dotenv import load_dotenv

from .schema import Hypothesis, ProposalReport

load_dotenv()


class GenerateFix(dspy.Signature):
    """Propose a code change to fix a failed gameplay expectation.

    Why this signature: Separates inputs (what failed, what can change)
    from outputs (what to change, why). DSPy learns the mapping.
    """

    failed_expectation = dspy.InputField(
        desc="The expectation that failed: ID, behavior, and what was observed"
    )
    tunable_constants = dspy.InputField(
        desc="Constants that can be adjusted: file, name, current value, valid range"
    )
    system_files = dspy.InputField(
        desc="Rust source files related to this behavior"
    )

    reasoning = dspy.OutputField(
        desc="Step-by-step reasoning about why the behavior failed and how to fix it"
    )
    file_path = dspy.OutputField(
        desc="The specific file to modify"
    )
    change_description = dspy.OutputField(
        desc="Concise description of the change"
    )
    code_snippet = dspy.OutputField(
        desc="The actual code change or constant adjustment"
    )
    confidence = dspy.OutputField(
        desc="Confidence 0-100 that this fix will help"
    )


class FixProposer(dspy.Module):
    """DSPy module that generates fix proposals.

    Why Module subclass: Allows DSPy to compile/optimize
    the internal prompts based on training examples.
    """

    def __init__(self):
        super().__init__()
        # ChainOfThought adds "let's think step by step" reasoning
        self.generate = dspy.ChainOfThought(GenerateFix)

    def forward(
        self,
        failed_expectation: str,
        tunable_constants: str,
        system_files: str,
    ) -> dspy.Prediction:
        """Generate a fix proposal.

        Returns DSPy Prediction with reasoning, file_path, etc.
        """
        return self.generate(
            failed_expectation=failed_expectation,
            tunable_constants=tunable_constants,
            system_files=system_files,
        )


def configure_dspy(api_key: str | None = None):
    """Set up DSPy with Gemini backend.

    Why separate function: Allows reconfiguration for different
    models or testing scenarios.
    """
    key = api_key or os.getenv("GOOGLE_API_KEY")
    if not key:
        raise ValueError("GOOGLE_API_KEY required")

    lm = dspy.LM(
        model="google/gemini-3-pro-preview",
        api_key=key,
        temperature=0.7,  # Slightly creative for proposals
    )
    dspy.configure(lm=lm)


def propose_fixes(
    eval_report_path: str,
    focus_path: str,
    api_key: str | None = None,
) -> dict:
    """Generate fix proposals for failed expectations.

    Args:
        eval_report_path: Path to evaluation report JSON
        focus_path: Path to focus config JSON
        api_key: Google API key (or from env)

    Returns:
        ProposalReport as dict
    """
    configure_dspy(api_key)

    # Load data
    with open(eval_report_path) as f:
        eval_report = json.load(f)
    with open(focus_path) as f:
        focus = json.load(f)

    # Find failed expectations
    misses = [
        v for v in eval_report.get("verdicts", [])
        if v.get("verdict") in ("MISS", "PARTIAL")
    ]

    if not misses:
        return ProposalReport(hypotheses=[], recommended=None).model_dump()

    # Format tunable constants
    tunables_text = "\n".join(
        f"- {t['file']}: {t['name']} = {t['current']} (range: {t['range']})"
        for t in focus.get("tunable_constants", [])
    )

    # Format system files
    systems_text = "\n".join(f"- {s}" for s in focus.get("systems", []))

    # Generate proposals
    proposer = FixProposer()
    hypotheses = []

    for miss in misses:
        # Format the failed expectation
        exp_text = f"ID: {miss['expectation_id']}\n"
        exp_text += f"Expected: {miss.get('expected', 'N/A')}\n"
        exp_text += f"Observed: {miss.get('observed', 'N/A')}\n"
        exp_text += f"Reasoning: {miss.get('reasoning', 'N/A')}"

        try:
            result = proposer(
                failed_expectation=exp_text,
                tunable_constants=tunables_text,
                system_files=systems_text,
            )

            hypotheses.append(Hypothesis(
                target_expectation=miss["expectation_id"],
                file_path=result.file_path,
                change_description=result.change_description,
                code_snippet=result.code_snippet,
                reasoning=result.reasoning,
                confidence=int(result.confidence) if result.confidence.isdigit() else 50,
            ))
        except Exception as e:
            # Log but continue with other misses
            print(f"Warning: Failed to generate proposal for {miss['expectation_id']}: {e}")

    # Sort by confidence, recommend highest
    hypotheses.sort(key=lambda h: h.confidence, reverse=True)
    recommended = hypotheses[0].target_expectation if hypotheses else None

    report = ProposalReport(hypotheses=hypotheses, recommended=recommended)
    return report.model_dump()
```

**Step 2: Verify DSPy import works**

Run:
```bash
cd data/gameplay_optimization/tools
python3 -c "from src.opt_gameplay.proposer import FixProposer; print('Proposer OK')"
```
Expected: `Proposer OK`

**Step 3: Commit**

```bash
git add data/gameplay_optimization/tools/src/opt_gameplay/proposer.py
git commit -m "feat(opt-tools): implement DSPy proposer with ChainOfThought"
```

---

## Task 5: Implement Learner (DSPy Optimization)

**Files:**
- Create: `data/gameplay_optimization/tools/src/opt_gameplay/learner.py`

**Step 1: Create learner.py**

Create `data/gameplay_optimization/tools/src/opt_gameplay/learner.py`:

```python
"""DSPy prompt optimization based on successful fixes.

Why self-optimization: Over time, the proposer learns which
types of reasoning and changes actually improve simulations,
making future proposals more effective.
"""

import json
import os
from pathlib import Path
from typing import Any

import dspy
from dspy.teleprompt import BootstrapFewShot
from dotenv import load_dotenv

from .proposer import FixProposer, GenerateFix, configure_dspy

load_dotenv()

# Where to save compiled DSPy programs
COMPILED_PROPOSER_PATH = Path(__file__).parent.parent.parent / "compiled_proposer.json"


def extract_training_examples(changelog_path: str) -> list[dict]:
    """Extract successful fixes from changelog as training data.

    Why changelog: It already tracks what changes improved metrics.
    Entries where after_hit_rate > before_hit_rate are successes.

    Returns:
        List of {input: ..., output: ...} dicts for DSPy training
    """
    with open(changelog_path) as f:
        changelog = json.load(f)

    examples = []

    for entry in changelog:
        # Parse hit rates like "5/7" -> 0.714
        def parse_rate(rate_str: str) -> float:
            if not rate_str or "/" not in rate_str:
                return 0.0
            num, denom = rate_str.split("/")
            return int(num) / int(denom) if int(denom) > 0 else 0.0

        before = parse_rate(entry.get("before_hit_rate", ""))
        after = parse_rate(entry.get("after_hit_rate", ""))

        # Only use entries where metrics improved
        if after > before:
            examples.append({
                "failed_expectation": f"Focus: {entry.get('focus', 'unknown')}",
                "hypothesis": entry.get("hypothesis", ""),
                "file": entry.get("file", ""),
                "change_summary": entry.get("change_summary", ""),
                "improvement": f"{before:.0%} -> {after:.0%}",
            })

    return examples


def create_dspy_examples(training_data: list[dict]) -> list[dspy.Example]:
    """Convert training data to DSPy Example format.

    Why Example objects: DSPy's optimizers expect this format
    to understand which outputs are "good".
    """
    examples = []

    for item in training_data:
        ex = dspy.Example(
            failed_expectation=item["failed_expectation"],
            tunable_constants="(from focus config)",
            system_files=item.get("file", ""),
            # Expected outputs
            reasoning=f"This change improved metrics: {item['improvement']}",
            file_path=item.get("file", ""),
            change_description=item.get("hypothesis", ""),
            code_snippet=item.get("change_summary", ""),
            confidence="75",
        ).with_inputs("failed_expectation", "tunable_constants", "system_files")

        examples.append(ex)

    return examples


def metric_fn(example: dspy.Example, prediction: dspy.Prediction) -> bool:
    """Evaluate if a prediction is good.

    Why simple metric: We can't run the actual simulation during
    training, so we check if the proposal is well-formed and
    references actual files.
    """
    # Check prediction has required fields
    if not prediction.file_path or not prediction.change_description:
        return False

    # Check file path looks like a Rust source file
    if not prediction.file_path.endswith(".rs"):
        return False

    # Check reasoning is substantive
    if len(prediction.reasoning) < 20:
        return False

    return True


def train_proposer(
    changelog_path: str,
    api_key: str | None = None,
    min_examples: int = 3,
) -> dict[str, Any]:
    """Train the proposer using successful fixes from changelog.

    Args:
        changelog_path: Path to changelog.json
        api_key: Google API key
        min_examples: Minimum examples needed to train

    Returns:
        Status dict with training results
    """
    configure_dspy(api_key)

    # Extract training data
    training_data = extract_training_examples(changelog_path)

    if len(training_data) < min_examples:
        return {
            "status": "skipped",
            "reason": f"Need {min_examples}+ successful fixes, have {len(training_data)}",
            "examples_found": len(training_data),
        }

    # Convert to DSPy examples
    examples = create_dspy_examples(training_data)

    # Split into train/dev
    split = max(1, len(examples) // 4)
    trainset = examples[:-split] if split < len(examples) else examples
    devset = examples[-split:] if split < len(examples) else examples[:1]

    # Create optimizer
    optimizer = BootstrapFewShot(
        metric=metric_fn,
        max_bootstrapped_demos=3,
        max_labeled_demos=3,
    )

    # Compile the proposer
    base_proposer = FixProposer()

    try:
        compiled_proposer = optimizer.compile(
            base_proposer,
            trainset=trainset,
        )

        # Save compiled program
        compiled_proposer.save(str(COMPILED_PROPOSER_PATH))

        return {
            "status": "success",
            "examples_used": len(trainset),
            "saved_to": str(COMPILED_PROPOSER_PATH),
        }

    except Exception as e:
        return {
            "status": "error",
            "error": str(e),
        }


def load_compiled_proposer() -> FixProposer | None:
    """Load a previously compiled proposer if available.

    Returns None if no compiled version exists.
    """
    if not COMPILED_PROPOSER_PATH.exists():
        return None

    try:
        proposer = FixProposer()
        proposer.load(str(COMPILED_PROPOSER_PATH))
        return proposer
    except Exception:
        return None
```

**Step 2: Verify learner imports**

Run:
```bash
cd data/gameplay_optimization/tools
python3 -c "from src.opt_gameplay.learner import train_proposer; print('Learner OK')"
```
Expected: `Learner OK`

**Step 3: Commit**

```bash
git add data/gameplay_optimization/tools/src/opt_gameplay/learner.py
git commit -m "feat(opt-tools): implement DSPy learner with BootstrapFewShot"
```

---

## Task 6: Implement CLI

**Files:**
- Create: `data/gameplay_optimization/tools/src/opt_gameplay/cli.py`

**Step 1: Create cli.py**

Create `data/gameplay_optimization/tools/src/opt_gameplay/cli.py`:

```python
"""CLI interface for gameplay optimization tools.

Why Typer: Clean, type-safe CLI with automatic help generation.
Claude can call these commands directly from bash.
"""

import json
import sys
from pathlib import Path
from typing import Optional

import typer

from .evaluator import evaluate_simulation
from .learner import train_proposer
from .proposer import propose_fixes

app = typer.Typer(
    name="opt-gameplay",
    help="LLM-driven gameplay optimization tools for Arc Citadel",
)


@app.command()
def evaluate(
    focus: Path = typer.Option(..., "--focus", "-f", help="Path to focus config JSON"),
    sim_output: Path = typer.Option(..., "--sim-output", "-s", help="Path to simulation output"),
    output: Optional[Path] = typer.Option(None, "--output", "-o", help="Save report to file"),
):
    """Evaluate simulation output against focus expectations.

    Analyzes the simulation log and produces structured verdicts
    (HIT/PARTIAL/MISS) with reasoning for each expectation.

    Example:
        opt-gameplay evaluate -f focuses/action-selection.json -s /tmp/sim.txt
    """
    if not focus.exists():
        typer.echo(f"Error: Focus file not found: {focus}", err=True)
        raise typer.Exit(1)

    if not sim_output.exists():
        typer.echo(f"Error: Simulation output not found: {sim_output}", err=True)
        raise typer.Exit(1)

    typer.echo(f"Evaluating {sim_output.name} against {focus.name}...")

    try:
        report = evaluate_simulation(str(focus), str(sim_output))

        result_json = json.dumps(report, indent=2)

        if output:
            output.write_text(result_json)
            typer.echo(f"Report saved to {output}")
        else:
            typer.echo(result_json)

    except Exception as e:
        typer.echo(f"Error during evaluation: {e}", err=True)
        raise typer.Exit(1)


@app.command()
def propose(
    eval_result: Path = typer.Option(..., "--eval", "-e", help="Path to evaluation report JSON"),
    focus: Path = typer.Option(..., "--focus", "-f", help="Path to focus config JSON"),
    output: Optional[Path] = typer.Option(None, "--output", "-o", help="Save proposals to file"),
):
    """Generate fix proposals for failed expectations.

    Uses DSPy to reason about why expectations failed and
    propose concrete code changes to fix them.

    Example:
        opt-gameplay propose -e /tmp/eval.json -f focuses/action-selection.json
    """
    if not eval_result.exists():
        typer.echo(f"Error: Evaluation result not found: {eval_result}", err=True)
        raise typer.Exit(1)

    if not focus.exists():
        typer.echo(f"Error: Focus file not found: {focus}", err=True)
        raise typer.Exit(1)

    typer.echo("Generating fix proposals...")

    try:
        proposals = propose_fixes(str(eval_result), str(focus))

        result_json = json.dumps(proposals, indent=2)

        if output:
            output.write_text(result_json)
            typer.echo(f"Proposals saved to {output}")
        else:
            typer.echo(result_json)

    except Exception as e:
        typer.echo(f"Error generating proposals: {e}", err=True)
        raise typer.Exit(1)


@app.command()
def learn(
    changelog: Path = typer.Option(
        Path("data/gameplay_optimization/changelog.json"),
        "--changelog", "-c",
        help="Path to changelog JSON"
    ),
):
    """Train the proposer on successful fixes from changelog.

    Analyzes past optimization sessions to learn which types
    of proposals actually improved the simulation. Requires
    at least 3 successful fixes to train.

    Example:
        opt-gameplay learn -c changelog.json
    """
    if not changelog.exists():
        typer.echo(f"Error: Changelog not found: {changelog}", err=True)
        raise typer.Exit(1)

    typer.echo("Analyzing changelog for training examples...")

    try:
        result = train_proposer(str(changelog))

        if result["status"] == "success":
            typer.echo(f"Training complete! Used {result['examples_used']} examples.")
            typer.echo(f"Compiled proposer saved to: {result['saved_to']}")
        elif result["status"] == "skipped":
            typer.echo(f"Training skipped: {result['reason']}")
        else:
            typer.echo(f"Training failed: {result.get('error', 'Unknown error')}", err=True)
            raise typer.Exit(1)

    except Exception as e:
        typer.echo(f"Error during training: {e}", err=True)
        raise typer.Exit(1)


@app.command()
def diff(
    before: Path = typer.Option(..., "--before", "-b", help="Previous evaluation JSON"),
    after: Path = typer.Option(..., "--after", "-a", help="New evaluation JSON"),
):
    """Compare two evaluation reports to see improvement.

    Shows which expectations changed status and the overall
    hit rate delta. Useful for verifying a fix worked.

    Example:
        opt-gameplay diff -b /tmp/eval_v1.json -a /tmp/eval_v2.json
    """
    if not before.exists() or not after.exists():
        typer.echo("Error: Both evaluation files must exist", err=True)
        raise typer.Exit(1)

    with open(before) as f:
        before_data = json.load(f)
    with open(after) as f:
        after_data = json.load(f)

    # Compare hit rates
    before_rate = before_data.get("hit_rate", "?/?")
    after_rate = after_data.get("hit_rate", "?/?")

    typer.echo(f"Hit Rate: {before_rate} -> {after_rate}")
    typer.echo("")

    # Find changed verdicts
    before_verdicts = {v["expectation_id"]: v["verdict"] for v in before_data.get("verdicts", [])}
    after_verdicts = {v["expectation_id"]: v["verdict"] for v in after_data.get("verdicts", [])}

    changes = []
    for exp_id in set(before_verdicts.keys()) | set(after_verdicts.keys()):
        bv = before_verdicts.get(exp_id, "N/A")
        av = after_verdicts.get(exp_id, "N/A")
        if bv != av:
            changes.append(f"  {exp_id}: {bv} -> {av}")

    if changes:
        typer.echo("Changed verdicts:")
        for change in changes:
            typer.echo(change)
    else:
        typer.echo("No verdict changes")


if __name__ == "__main__":
    app()
```

**Step 2: Install package in dev mode**

Run:
```bash
cd data/gameplay_optimization/tools
pip install -e .
```
Expected: Successfully installed opt-gameplay

**Step 3: Test CLI help**

Run:
```bash
opt-gameplay --help
```
Expected: Shows commands: evaluate, propose, learn, diff

**Step 4: Commit**

```bash
git add data/gameplay_optimization/tools/src/opt_gameplay/cli.py
git commit -m "feat(opt-tools): implement Typer CLI with evaluate/propose/learn/diff commands"
```

---

## Task 7: Update optimize-gameplay.md Command

**Files:**
- Modify: `.claude/commands/optimize-gameplay.md`

**Step 1: Add tool integration to Step 3**

In `.claude/commands/optimize-gameplay.md`, replace Step 3 content (around line 47-60) with:

```markdown
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
```

**Step 2: Add tool integration to Step 4**

Replace Step 4 content with:

```markdown
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
```

**Step 3: Add diff check to Step 7**

In Step 7 (If Applying), add before the changelog append:

```markdown
After re-running simulation and evaluation, compare results:

```bash
opt-gameplay diff \
  --before /tmp/prev_eval.json \
  --after /tmp/current_eval.json
```

Copy current to prev for next iteration:
```bash
cp /tmp/current_eval.json /tmp/prev_eval.json
```
```

**Step 4: Add learn command to end**

Add new section after Success Criteria:

```markdown
## Periodic Training

After 3+ successful optimization sessions, train the proposer:

```bash
opt-gameplay learn --changelog data/gameplay_optimization/changelog.json
```

This improves future proposal quality by learning from what worked.
```

**Step 5: Commit**

```bash
git add .claude/commands/optimize-gameplay.md
git commit -m "feat(optimize-gameplay): integrate LangChain/DSPy tools into command loop"
```

---

## Task 8: End-to-End Test

**Files:**
- Test: Full workflow

**Step 1: Run simulation and capture output**

```bash
cd /home/astre/arc-citadel
source ~/.cargo/env
cargo run 2>&1 | head -200 > /tmp/sim_output.txt
```

**Step 2: Run evaluation**

```bash
opt-gameplay evaluate \
  -f data/gameplay_optimization/focuses/action-selection.json \
  -s /tmp/sim_output.txt \
  -o /tmp/eval.json
```
Expected: JSON report with verdicts

**Step 3: Run proposal generation**

```bash
opt-gameplay propose \
  -e /tmp/eval.json \
  -f data/gameplay_optimization/focuses/action-selection.json
```
Expected: Proposals for any misses

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat(opt-tools): complete LangChain + DSPy gameplay optimization integration

- Evaluator: LangChain structured output for verdicts
- Proposer: DSPy ChainOfThought for fix hypotheses
- Learner: BootstrapFewShot trains on successful fixes
- CLI: opt-gameplay evaluate|propose|learn|diff
- Integrated into /optimize-gameplay command"
```

---

## Summary

| Task | Component | Purpose |
|------|-----------|---------|
| 1 | Project scaffold | pyproject.toml, directories |
| 2 | Schema | Pydantic models for structured outputs |
| 3 | Evaluator | LangChain verdict parsing |
| 4 | Proposer | DSPy hypothesis generation |
| 5 | Learner | DSPy prompt optimization |
| 6 | CLI | Typer commands |
| 7 | Integration | Update optimize-gameplay.md |
| 8 | E2E Test | Verify full workflow |
