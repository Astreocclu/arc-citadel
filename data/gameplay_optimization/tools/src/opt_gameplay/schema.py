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
