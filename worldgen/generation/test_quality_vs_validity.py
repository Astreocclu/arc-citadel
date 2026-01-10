"""Test whether quality scoring implicitly enforces structural constraints.

Hypothesis: If structurally invalid outputs score low on quality, then
the quality loop is already doing constraint work and explicit constraints
are redundant.

Key metric: % of outputs that are INVALID but score HIGH (≥7)
If near-zero, skip constraints entirely.
"""

import json
import tomllib
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Optional
import csv

from .llm_client import DeepSeekClient
from .test_structural_validity import (
    TEST_CONTEXTS,
    TestContext,
    validate_output,
    ValidationResult,
)


def load_context_constraints() -> dict:
    """Load context constraints from TOML file."""
    constraints_path = Path(__file__).parent / "context_constraints.toml"
    with open(constraints_path, "rb") as f:
        return tomllib.load(f)


CONTEXT_CONSTRAINTS = load_context_constraints()


@dataclass
class QualityValidityResult:
    """Combined quality and validity result."""

    context_name: str
    feature_name: str

    # Structural validity
    elevation_valid: bool
    terrain_valid: bool
    temperature_valid: bool
    overall_valid: bool

    # Quality scores
    quality_score: float
    strategic_score: float
    narrative_score: float
    authenticity_score: float
    sensory_score: float
    physical_score: float

    # Raw data
    elevation_actual: Optional[float] = None
    terrain_actual: Optional[str] = None
    temperature_actual: Optional[float] = None
    error: Optional[str] = None


def build_scoring_prompt(context_name: str, context_description: str, location_json: str) -> str:
    """Build scoring prompt with context-specific constraint ranges."""

    constraints = CONTEXT_CONSTRAINTS.get(context_name, {})

    if constraints:
        constraint_block = f"""5. PHYSICAL COHERENCE - STRICT NUMERIC CHECK:

   STEP 1: Extract the numeric values from the location JSON above.
   STEP 2: Compare EACH value against these EXACT ranges:

   | Field       | Min    | Max    | Rule                                    |
   |-------------|--------|--------|-----------------------------------------|
   | elevation   | {constraints['elevation_min']:>6} | {constraints['elevation_max']:>6} | FAIL if outside [{constraints['elevation_min']}, {constraints['elevation_max']}] |
   | temperature | {constraints['temperature_min']:>6} | {constraints['temperature_max']:>6} | FAIL if outside [{constraints['temperature_min']}, {constraints['temperature_max']}] |
   | terrain     | must be one of: {constraints['terrain']}                    |

   STEP 3: Score based on STRICT numeric comparison:
   - If elevation < {constraints['elevation_min']} OR elevation > {constraints['elevation_max']}: FAIL → physical_score = 2
   - If temperature < {constraints['temperature_min']} OR temperature > {constraints['temperature_max']}: FAIL → physical_score = 2
   - If terrain not in {constraints['terrain']}: FAIL → physical_score = 2
   - If ANY check fails: overall_score CAPPED at 4 (cannot be higher)
   - If ALL checks pass: physical_score = 8-10"""
    else:
        constraint_block = """5. PHYSICAL COHERENCE: Are the numeric values physically sensible? (1-10)"""

    return f"""Rate this generated location from 1-10:

LOCATION:
{location_json}

CONTEXT: {context_description}

BEFORE scoring, do this NUMERIC CHECK for physical coherence:
{constraint_block}

Then rate on:
1. STRATEGIC VALUE: Does it create interesting gameplay choices? (1-10)
2. NARRATIVE POTENTIAL: Does it suggest emergent stories? (1-10)
3. AUTHENTICITY: Does it feel genuine to the setting? (1-10)
4. SENSORY RICHNESS: Can you feel being there? (1-10)

RULE: If physical_score <= 3, then overall_score MUST be <= 4. No exceptions.

Respond with ONLY JSON:
{{
  "strategic_score": <1-10>,
  "narrative_score": <1-10>,
  "authenticity_score": <1-10>,
  "sensory_score": <1-10>,
  "physical_score": <1-10>,
  "overall_score": <1-10>
}}"""


def run_quality_validity_test(
    contexts: list[TestContext],
    samples_per_context: int = 10,
    client: Optional[DeepSeekClient] = None,
) -> list[QualityValidityResult]:
    """Generate samples and measure both validity and quality."""

    if client is None:
        client = DeepSeekClient()

    results: list[QualityValidityResult] = []
    total = len(contexts) * samples_per_context
    current = 0

    for context in contexts:
        print(f"\n{'='*60}")
        print(f"Context: {context.name}")

        for i in range(samples_per_context):
            current += 1
            try:
                # Generate
                output = client.generate_json(
                    prompt=context.prompt_template,
                    temperature=1.0,
                    max_tokens=1000,
                )

                # Validate structure
                validity = validate_output(output, context)

                # Score quality with context-specific constraints
                score_prompt = build_scoring_prompt(
                    context_name=context.name,
                    context_description=context.description,
                    location_json=json.dumps(output, indent=2),
                )
                scores = client.generate_json(
                    prompt=score_prompt,
                    system_prompt="You are a harsh game design critic. Most things are 5-7.",
                    temperature=0.3,
                    max_tokens=300,
                )

                result = QualityValidityResult(
                    context_name=context.name,
                    feature_name=validity.feature_name,
                    elevation_valid=validity.elevation_valid,
                    terrain_valid=validity.terrain_valid,
                    temperature_valid=validity.temperature_valid,
                    overall_valid=validity.overall_valid,
                    quality_score=float(scores.get("overall_score", 0)),
                    strategic_score=float(scores.get("strategic_score", 0)),
                    narrative_score=float(scores.get("narrative_score", 0)),
                    authenticity_score=float(scores.get("authenticity_score", 0)),
                    sensory_score=float(scores.get("sensory_score", 0)),
                    physical_score=float(scores.get("physical_score", 0)),
                    elevation_actual=validity.elevation_actual,
                    terrain_actual=validity.terrain_actual,
                    temperature_actual=validity.temperature_actual,
                )

                status = "VALID" if result.overall_valid else "INVALID"
                print(f"  [{current}/{total}] {status} Q={result.quality_score:.1f}: {result.feature_name}")

            except Exception as e:
                result = QualityValidityResult(
                    context_name=context.name,
                    feature_name="ERROR",
                    elevation_valid=False,
                    terrain_valid=False,
                    temperature_valid=False,
                    overall_valid=False,
                    quality_score=0,
                    strategic_score=0,
                    narrative_score=0,
                    authenticity_score=0,
                    sensory_score=0,
                    physical_score=0,
                    error=str(e),
                )
                print(f"  [{current}/{total}] ERROR: {e}")

            results.append(result)

    return results


def analyze_results(results: list[QualityValidityResult]) -> dict:
    """Analyze the key question: invalid AND high quality?"""

    valid_results = [r for r in results if r.error is None]

    # Categorize
    valid_high = [r for r in valid_results if r.overall_valid and r.quality_score >= 7]
    valid_low = [r for r in valid_results if r.overall_valid and r.quality_score < 7]
    invalid_high = [r for r in valid_results if not r.overall_valid and r.quality_score >= 7]
    invalid_low = [r for r in valid_results if not r.overall_valid and r.quality_score < 7]

    total = len(valid_results)

    print("\n" + "=" * 70)
    print("QUALITY vs VALIDITY ANALYSIS")
    print("=" * 70)

    print(f"\nTotal samples (excluding errors): {total}")
    print(f"\n2x2 Matrix:")
    print(f"                    | Quality ≥ 7  | Quality < 7  |")
    print(f"  ------------------|--------------|--------------|")
    print(f"  Struct VALID      | {len(valid_high):4d} ({100*len(valid_high)/total:5.1f}%) | {len(valid_low):4d} ({100*len(valid_low)/total:5.1f}%) |")
    print(f"  Struct INVALID    | {len(invalid_high):4d} ({100*len(invalid_high)/total:5.1f}%) | {len(invalid_low):4d} ({100*len(invalid_low)/total:5.1f}%) |")

    print(f"\n" + "-" * 70)
    print("KEY METRIC: Invalid but High Quality")
    print("-" * 70)
    pct = 100 * len(invalid_high) / total if total > 0 else 0
    print(f"  Count: {len(invalid_high)}/{total}")
    print(f"  Percentage: {pct:.1f}%")

    if pct < 5:
        print(f"\n  CONCLUSION: Quality loop is implicitly enforcing constraints.")
        print(f"  RECOMMENDATION: Skip explicit constraints - they're redundant.")
    elif pct < 15:
        print(f"\n  CONCLUSION: Some leakage, but quality loop catches most issues.")
        print(f"  RECOMMENDATION: Consider lightweight constraints as safety net.")
    else:
        print(f"\n  CONCLUSION: Significant invalid-but-high-quality outputs.")
        print(f"  RECOMMENDATION: Explicit constraints needed.")

    # Show examples of invalid-high if any
    if invalid_high:
        print(f"\n" + "-" * 70)
        print("Examples of INVALID but HIGH QUALITY:")
        print("-" * 70)
        for r in invalid_high[:5]:
            print(f"\n  {r.feature_name} (Q={r.quality_score}, P={r.physical_score})")
            print(f"    Context: {r.context_name}")
            if not r.elevation_valid:
                print(f"    - Elevation: {r.elevation_actual}")
            if not r.terrain_valid:
                print(f"    - Terrain: {r.terrain_actual}")
            if not r.temperature_valid:
                print(f"    - Temperature: {r.temperature_actual}")

    # Quality distribution for valid vs invalid
    valid_scores = [r.quality_score for r in valid_results if r.overall_valid]
    invalid_scores = [r.quality_score for r in valid_results if not r.overall_valid]

    if valid_scores and invalid_scores:
        avg_valid = sum(valid_scores) / len(valid_scores)
        avg_invalid = sum(invalid_scores) / len(invalid_scores)

        print(f"\n" + "-" * 70)
        print("Overall Quality Score Distribution:")
        print("-" * 70)
        print(f"  Structurally VALID:   avg={avg_valid:.2f}, n={len(valid_scores)}")
        print(f"  Structurally INVALID: avg={avg_invalid:.2f}, n={len(invalid_scores)}")
        print(f"  Delta: {avg_valid - avg_invalid:+.2f}")

        if avg_valid - avg_invalid > 1.0:
            print(f"\n  Overall quality scores ARE correlated with structural validity.")
        else:
            print(f"\n  Overall quality scores NOT strongly correlated with structural validity.")

    # Physical score distribution
    valid_phys = [r.physical_score for r in valid_results if r.overall_valid]
    invalid_phys = [r.physical_score for r in valid_results if not r.overall_valid]

    if valid_phys and invalid_phys:
        avg_valid_phys = sum(valid_phys) / len(valid_phys)
        avg_invalid_phys = sum(invalid_phys) / len(invalid_phys)

        print(f"\n" + "-" * 70)
        print("Physical Score Distribution:")
        print("-" * 70)
        print(f"  Structurally VALID:   avg={avg_valid_phys:.2f}, n={len(valid_phys)}")
        print(f"  Structurally INVALID: avg={avg_invalid_phys:.2f}, n={len(invalid_phys)}")
        print(f"  Delta: {avg_valid_phys - avg_invalid_phys:+.2f}")

        if avg_valid_phys - avg_invalid_phys > 2.0:
            print(f"\n  Physical scores ARE detecting structural invalidity!")
        else:
            print(f"\n  Physical scores NOT detecting structural invalidity.")

    print("=" * 70)

    return {
        "total": total,
        "valid_high": len(valid_high),
        "valid_low": len(valid_low),
        "invalid_high": len(invalid_high),
        "invalid_low": len(invalid_low),
        "invalid_high_pct": pct,
    }


def write_csv(results: list[QualityValidityResult], path: Optional[Path] = None) -> Path:
    """Write results to CSV."""
    if path is None:
        output_dir = Path(__file__).parent.parent / "output"
        output_dir.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        path = output_dir / f"quality_vs_validity_{timestamp}.csv"

    with open(path, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow([
            "context", "feature_name",
            "elevation_valid", "terrain_valid", "temp_valid", "overall_valid",
            "quality_score", "physical_score", "strategic", "narrative", "authenticity", "sensory",
            "elevation_actual", "terrain_actual", "temp_actual", "error"
        ])
        for r in results:
            writer.writerow([
                r.context_name, r.feature_name,
                1 if r.elevation_valid else 0,
                1 if r.terrain_valid else 0,
                1 if r.temperature_valid else 0,
                1 if r.overall_valid else 0,
                r.quality_score, r.physical_score, r.strategic_score, r.narrative_score,
                r.authenticity_score, r.sensory_score,
                r.elevation_actual or "", r.terrain_actual or "",
                r.temperature_actual or "", r.error or ""
            ])

    print(f"\nResults written to: {path}")
    return path


def main() -> int:
    """Run the quality vs validity test."""
    import argparse

    parser = argparse.ArgumentParser(
        description="Test if quality scoring implicitly enforces constraints"
    )
    parser.add_argument("--samples", "-n", type=int, default=10,
                        help="Samples per context (default: 10)")
    parser.add_argument("--contexts", type=int, default=None,
                        help="Number of contexts (default: all 10)")
    args = parser.parse_args()

    contexts = TEST_CONTEXTS[:args.contexts] if args.contexts else TEST_CONTEXTS
    total_calls = len(contexts) * args.samples * 2  # generate + score

    print(f"Testing {len(contexts)} contexts x {args.samples} samples = {len(contexts) * args.samples} generations")
    print(f"Total API calls: {total_calls} (generation + scoring)")

    results = run_quality_validity_test(contexts, samples_per_context=args.samples)
    write_csv(results)
    analysis = analyze_results(results)

    return 0


if __name__ == "__main__":
    import sys
    sys.exit(main())
