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
# Path from tools/src/opt_gameplay/learner.py:
# .parent = opt_gameplay/, .parent.parent = src/, .parent.parent.parent = tools/
COMPILED_PROPOSER_PATH = Path(__file__).parent.parent.parent / "compiled_proposer.json"
# .parent.parent.parent.parent = data/gameplay_optimization/
FOCUSES_DIR = Path(__file__).parent.parent.parent.parent / "focuses"


def load_focus_context(focus_name: str) -> tuple[str, str]:
    """Load tunable constants and system files from a focus file.

    Returns (tunable_constants_text, systems_text).
    """
    focus_path = FOCUSES_DIR / f"{focus_name}.json"
    if not focus_path.exists():
        return "(no focus file found)", "(no systems listed)"

    with open(focus_path) as f:
        focus = json.load(f)

    tunables = focus.get("tunable_constants", [])
    tunables_text = "\n".join(
        f"- {t['file']}: {t['name']} = {t['current']} (range: {t['range']})"
        for t in tunables
    ) or "(no tunable constants)"

    systems = focus.get("systems", [])
    systems_text = "\n".join(f"- {s}" for s in systems) or "(no systems listed)"

    return tunables_text, systems_text


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
                "focus": entry.get("focus", "unknown"),
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
        # Load real focus data for richer training context
        focus_name = item.get("focus", "unknown")
        tunables_text, systems_text = load_focus_context(focus_name)

        ex = dspy.Example(
            failed_expectation=item["failed_expectation"],
            tunable_constants=tunables_text,
            system_files=systems_text,
            # Expected outputs
            reasoning=f"This change improved metrics: {item['improvement']}",
            file_path=item.get("file", ""),
            change_description=item.get("hypothesis", ""),
            code_snippet=item.get("change_summary", ""),
            confidence="75",
        ).with_inputs("failed_expectation", "tunable_constants", "system_files")

        examples.append(ex)

    return examples


def metric_fn(example: dspy.Example, prediction: dspy.Prediction, trace=None) -> bool:
    """Evaluate if a prediction is good.

    Why simple metric: We can't run the actual simulation during
    training, so we check if the proposal is well-formed and
    references actual files.

    Args:
        example: The input example
        prediction: The model's prediction
        trace: Optional trace info from DSPy (unused but required by newer DSPy versions)
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

    # Split into train/dev (use all examples for both if we have few)
    if len(examples) <= 2:
        # With 1-2 examples, use all for training (no meaningful split possible)
        trainset = examples
        devset = examples
    else:
        # Reserve ~25% for dev, but at least 1
        split = max(1, len(examples) // 4)
        trainset = examples[:-split]
        devset = examples[-split:]

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
