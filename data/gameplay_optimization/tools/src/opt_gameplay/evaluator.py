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
            model="gemini-2.0-flash",
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
