"""Quality-focused generation with iterative improvement."""

import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

from worldgen import config
from worldgen.schemas import ComponentCategory
from .llm_client import DeepSeekClient


@dataclass
class GenerationStats:
    """Statistics for generation runs."""

    total_generated: int = 0
    total_iterations: int = 0
    total_candidates: int = 0
    avg_final_score: float = 0.0
    scores: list[float] = field(default_factory=list)

    def record_generation(self, score: float, iterations: int, candidates: int) -> None:
        """Record a completed generation."""
        self.total_generated += 1
        self.total_iterations += iterations
        self.total_candidates += candidates
        self.scores.append(score)
        self.avg_final_score = sum(self.scores) / len(self.scores)


@dataclass
class ScoringResult:
    """Result from scoring an asset."""

    strategic_score: float
    narrative_score: float
    authenticity_score: float
    sensory_score: float
    overall_score: float
    strengths: list[str]
    weaknesses: list[str]
    improvement_suggestions: list[str]

    @classmethod
    def from_dict(cls, data: dict) -> "ScoringResult":
        """Create from dictionary (LLM response)."""
        return cls(
            strategic_score=float(data.get("strategic_score", 0)),
            narrative_score=float(data.get("narrative_score", 0)),
            authenticity_score=float(data.get("authenticity_score", 0)),
            sensory_score=float(data.get("sensory_score", 0)),
            overall_score=float(data.get("overall_score", 0)),
            strengths=data.get("strengths", []),
            weaknesses=data.get("weaknesses", []),
            improvement_suggestions=data.get("improvement_suggestions", []),
        )


class QualityGenerator:
    """Generate assets with iterative quality improvement.

    The quality loop works as follows:
    1. Generate N candidates per round
    2. Score each candidate
    3. Pick the best scoring candidate
    4. If score >= target, return it
    5. Otherwise, create improvement prompt with feedback and iterate
    6. Repeat until target score reached or max iterations exhausted
    """

    def __init__(
        self,
        target_score: float = config.DEFAULT_TARGET_SCORE,
        max_iterations: int = config.MAX_QUALITY_ITERATIONS,
        candidates_per_round: int = config.CANDIDATES_PER_ROUND,
        client: Optional[DeepSeekClient] = None,
    ):
        """Initialize the quality generator.

        Args:
            target_score: Minimum score to accept (1-10 scale).
            max_iterations: Maximum improvement iterations.
            candidates_per_round: Number of candidates to generate per round.
            client: DeepSeek client instance. If None, creates new one.
        """
        self.target_score = target_score
        self.max_iterations = max_iterations
        self.candidates_per_round = candidates_per_round
        self.client = client

        self.stats = GenerationStats()

    def _get_client(self) -> DeepSeekClient:
        """Get or create the DeepSeek client."""
        if self.client is None:
            self.client = DeepSeekClient()
        return self.client

    def _load_scoring_prompt(self) -> str:
        """Load the universal scoring prompt from file."""
        scoring_path = config.PROMPTS_DIR / "scoring.txt"
        if scoring_path.exists():
            return scoring_path.read_text()
        return self._default_scoring_prompt()

    def _default_scoring_prompt(self) -> str:
        """Return the default scoring prompt if file not found."""
        return """Rate this {asset_type} from 1-10 on:
STRATEGIC VALUE: Does it create interesting choices? (1-10)
NARRATIVE POTENTIAL: Does it suggest stories? (1-10)
SPECIES AUTHENTICITY: Does it feel genuinely {species}? (1-10)
SENSORY RICHNESS: Can you feel being there? (1-10)

ASSET:
{asset_json}

Respond with ONLY JSON:
{{
  "strategic_score": <1-10>,
  "narrative_score": <1-10>,
  "authenticity_score": <1-10>,
  "sensory_score": <1-10>,
  "overall_score": <1-10>,
  "strengths": ["...", "..."],
  "weaknesses": ["...", "..."],
  "improvement_suggestions": ["...", "..."]
}}"""

    def score_asset(
        self, asset: dict, asset_type: str, species: str = "neutral"
    ) -> ScoringResult:
        """Score an asset using DeepSeek.

        Args:
            asset: The asset data as a dictionary.
            asset_type: Description of the asset type (e.g., "dwarf forge").
            species: The species this asset belongs to.

        Returns:
            ScoringResult with all scoring dimensions.
        """
        client = self._get_client()
        prompt_template = self._load_scoring_prompt()
        prompt = prompt_template.format(
            asset_type=asset_type,
            species=species,
            asset_json=json.dumps(asset, indent=2),
        )

        result = client.generate_json(
            prompt=prompt,
            system_prompt="You are a harsh but fair game design critic. A 9/10 is genuinely excellent. Most things are 5-7.",
            temperature=0.3,
            max_tokens=500,
        )

        return ScoringResult.from_dict(result)

    def generate_with_quality(
        self,
        prompt_template: str,
        asset_type: str,
        species: str,
    ) -> tuple[Optional[dict], Optional[ScoringResult]]:
        """Generate candidates until one reaches target quality.

        Args:
            prompt_template: The base prompt for generation.
            asset_type: Description of the asset type.
            species: The species this asset belongs to.

        Returns:
            Tuple of (best_asset, scoring_result) or (None, None) if failed.
        """
        client = self._get_client()
        best: Optional[dict] = None
        best_score_data: Optional[ScoringResult] = None
        best_score = 0.0

        current_prompt = prompt_template
        total_candidates = 0

        for iteration in range(self.max_iterations):
            # Generate candidates
            candidates: list[tuple[dict, ScoringResult, float]] = []

            for _ in range(self.candidates_per_round):
                try:
                    candidate = client.generate_json(current_prompt)
                    score_data = self.score_asset(candidate, asset_type, species)
                    score = score_data.overall_score
                    candidates.append((candidate, score_data, score))
                    total_candidates += 1
                except Exception as e:
                    # Log but continue - some generations may fail
                    print(f"    Generation failed: {e}")
                    continue

            if not candidates:
                continue

            # Find best from this round
            round_best = max(candidates, key=lambda x: x[2])

            if round_best[2] > best_score:
                best, best_score_data, best_score = round_best

            # Check if target reached
            if best_score >= self.target_score:
                self.stats.record_generation(
                    score=best_score,
                    iterations=iteration + 1,
                    candidates=total_candidates,
                )
                return best, best_score_data

            # Build improvement prompt for next iteration
            if best_score_data is not None:
                current_prompt = self._improvement_prompt(
                    original_prompt=prompt_template,
                    score=best_score,
                    score_data=best_score_data,
                )

        # Return best we got after max iterations
        if best is not None:
            self.stats.record_generation(
                score=best_score,
                iterations=self.max_iterations,
                candidates=total_candidates,
            )
        return best, best_score_data

    def _improvement_prompt(
        self, original_prompt: str, score: float, score_data: ScoringResult
    ) -> str:
        """Build prompt for improvement iteration.

        Args:
            original_prompt: The original generation prompt.
            score: The score achieved by the previous best.
            score_data: The scoring details for feedback.

        Returns:
            New prompt incorporating feedback for improvement.
        """
        return f"""The previous version scored {score}/10.

FEEDBACK:
Strengths: {", ".join(score_data.strengths)}
Weaknesses: {", ".join(score_data.weaknesses)}
Suggestions: {", ".join(score_data.improvement_suggestions)}

Generate an IMPROVED version that:
1. Keeps all the strengths
2. Fixes all the weaknesses
3. Implements the suggestions
4. Targets 9/10 or higher

{original_prompt}"""

    def generate_component(
        self, category: ComponentCategory, prompt_template: str, index: int
    ) -> Optional[dict]:
        """Generate a single component with quality iteration.

        Args:
            category: The component category to generate.
            prompt_template: The prompt template for generation.
            index: Index for ID generation.

        Returns:
            The generated component dict with quality_score added, or None.
        """
        species = category.value.split("_")[0]
        asset_type = f"{species} {category.value.replace('_', ' ')}"

        result, score_data = self.generate_with_quality(
            prompt_template=prompt_template,
            asset_type=asset_type,
            species=species,
        )

        if result and score_data:
            result["quality_score"] = score_data.overall_score
            result["generation_notes"] = f"Strengths: {score_data.strengths}"

        return result
