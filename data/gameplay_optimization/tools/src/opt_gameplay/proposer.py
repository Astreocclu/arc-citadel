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
        model="gemini/gemini-2.0-flash",
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
