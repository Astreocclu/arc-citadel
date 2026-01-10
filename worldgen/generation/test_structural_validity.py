"""Structural validity test harness for freeform LLM generation.

Tests whether the LLM can generate structurally valid outputs (correct elevation,
terrain, temperature) without explicit constraints. High violation rates indicate
that constrained generation is necessary.

Usage:
    python -m generation.test_structural_validity --samples 5
    python -m generation.test_structural_validity --contexts 3 --samples 2  # Quick test

Output:
    - CSV with per-sample results
    - Summary statistics with violation rates
"""

import csv
from collections import defaultdict
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Optional

from .llm_client import DeepSeekClient


# =============================================================================
# Test Context Definitions
# =============================================================================


@dataclass
class TestContext:
    """A test context with expected structural constraints."""

    name: str
    description: str

    # Expected elevation (meters, real-world scale matching constraints.py)
    elevation_min: float
    elevation_max: float

    # Expected terrain types (any match = pass)
    expected_terrains: list[str]

    # Expected temperature range (Celsius)
    temperature_min: float
    temperature_max: float

    # Generation prompt for the LLM
    prompt_template: str


TEST_CONTEXTS: list[TestContext] = [
    # 1. Underground dwarven forge
    TestContext(
        name="underground_dwarf_forge",
        description="Deep underground dwarven forge",
        elevation_min=-800,
        elevation_max=-50,
        expected_terrains=["underground", "cavern"],
        temperature_min=20,
        temperature_max=70,  # Forges are hot
        prompt_template="""Generate a dwarven forge deep underground.
Output JSON with: name, terrain, elevation, temperature, resources, features, narrative_hook

terrain must be one of: underground, cavern
elevation should be negative (underground)
temperature in Celsius""",
    ),
    # 2. Surface human settlement
    TestContext(
        name="surface_human_settlement",
        description="Temperate surface human city district",
        elevation_min=0,
        elevation_max=500,
        expected_terrains=["plains", "hills"],
        temperature_min=5,
        temperature_max=35,
        prompt_template="""Generate a human city market square.
Output JSON with: name, terrain, elevation, temperature, resources, features, narrative_hook

terrain must be one of: plains, hills
elevation in meters above sea level
temperature in Celsius""",
    ),
    # 3. Elven forest
    TestContext(
        name="elven_forest",
        description="Ancient elven forest grove",
        elevation_min=0,
        elevation_max=600,
        expected_terrains=["forest", "dense_forest"],
        temperature_min=5,
        temperature_max=30,
        prompt_template="""Generate an ancient elven forest grove with a heart tree.
Output JSON with: name, terrain, elevation, temperature, resources, features, narrative_hook

terrain must be one of: forest, dense_forest
elevation in meters above sea level
temperature in Celsius""",
    ),
    # 4. Mountain pass
    TestContext(
        name="mountain_pass",
        description="High mountain pass between peaks",
        elevation_min=800,
        elevation_max=3000,
        expected_terrains=["mountains", "high_mountains"],
        temperature_min=-20,
        temperature_max=15,
        prompt_template="""Generate a treacherous mountain pass used by traders.
Output JSON with: name, terrain, elevation, temperature, resources, features, narrative_hook

terrain must be one of: mountains, high_mountains
elevation in meters (high altitude)
temperature in Celsius (cold at altitude)""",
    ),
    # 5. Coastal
    TestContext(
        name="coastal_harbor",
        description="Coastal area near sea level",
        elevation_min=-10,
        elevation_max=50,
        expected_terrains=["shallow_water", "plains", "marsh"],
        temperature_min=10,
        temperature_max=35,
        prompt_template="""Generate a coastal harbor area.
Output JSON with: name, terrain, elevation, temperature, resources, features, narrative_hook

terrain must be one of: shallow_water, plains, marsh
elevation near sea level (0m)
temperature in Celsius""",
    ),
    # 6. Desert oasis
    TestContext(
        name="desert_oasis",
        description="Oasis in deep desert",
        elevation_min=0,
        elevation_max=400,
        expected_terrains=["desert"],
        temperature_min=25,
        temperature_max=50,  # Deserts are hot
        prompt_template="""Generate a hidden desert oasis.
Output JSON with: name, terrain, elevation, temperature, resources, features, narrative_hook

terrain must be: desert
elevation in meters
temperature in Celsius (hot desert climate)""",
    ),
    # 7. Frozen tundra
    TestContext(
        name="frozen_tundra",
        description="Frozen northern wasteland",
        elevation_min=0,
        elevation_max=300,
        expected_terrains=["tundra", "glacier"],
        temperature_min=-40,
        temperature_max=5,  # Cold!
        prompt_template="""Generate a frozen tundra with ancient burial mounds.
Output JSON with: name, terrain, elevation, temperature, resources, features, narrative_hook

terrain must be one of: tundra, glacier
elevation in meters
temperature in Celsius (arctic cold)""",
    ),
    # 8. Deep cavern
    TestContext(
        name="deep_cavern",
        description="Natural deep cavern system",
        elevation_min=-500,
        elevation_max=-100,
        expected_terrains=["cavern", "underground"],
        temperature_min=10,
        temperature_max=20,  # Stable underground temp
        prompt_template="""Generate a vast natural cavern with underground lake.
Output JSON with: name, terrain, elevation, temperature, resources, features, narrative_hook

terrain must be one of: cavern, underground
elevation negative (deep underground)
temperature in Celsius (stable cave temperature)""",
    ),
    # 9. Volcanic region
    TestContext(
        name="volcanic_region",
        description="Active volcanic highlands",
        elevation_min=500,
        elevation_max=2000,
        expected_terrains=["volcanic", "mountains"],
        temperature_min=30,
        temperature_max=80,  # Near lava
        prompt_template="""Generate volcanic highlands with lava flows.
Output JSON with: name, terrain, elevation, temperature, resources, features, narrative_hook

terrain must be one of: volcanic, mountains
elevation in meters (volcanic peak)
temperature in Celsius (hot from volcanic activity)""",
    ),
    # 10. Swampland
    TestContext(
        name="swampland",
        description="Murky swamp with ruins",
        elevation_min=-20,
        elevation_max=50,
        expected_terrains=["marsh", "shallow_water"],
        temperature_min=15,
        temperature_max=35,
        prompt_template="""Generate a murky swamp with ancient ruins.
Output JSON with: name, terrain, elevation, temperature, resources, features, narrative_hook

terrain must be one of: marsh, shallow_water
elevation near sea level
temperature in Celsius""",
    ),
]


# =============================================================================
# Validation Functions
# =============================================================================


@dataclass
class ValidationResult:
    """Result of validating one generated feature."""

    context_name: str
    feature_name: str
    raw_output: dict

    # Binary pass/fail
    elevation_valid: bool
    terrain_valid: bool
    temperature_valid: bool

    # Details for debugging
    elevation_actual: Optional[float] = None
    terrain_actual: Optional[str] = None
    temperature_actual: Optional[float] = None
    error: Optional[str] = None

    @property
    def overall_valid(self) -> bool:
        """All dimensions must pass."""
        return self.elevation_valid and self.terrain_valid and self.temperature_valid


def validate_elevation(
    output: dict, context: TestContext
) -> tuple[bool, Optional[float]]:
    """Check if elevation matches context expectations.

    Returns (is_valid, actual_value).
    """
    elev = output.get("elevation")
    if elev is None:
        return False, None

    try:
        elev = float(elev)
    except (ValueError, TypeError):
        return False, None

    valid = context.elevation_min <= elev <= context.elevation_max
    return valid, elev


def validate_terrain(
    output: dict, context: TestContext
) -> tuple[bool, Optional[str]]:
    """Check if terrain matches context expectations.

    Returns (is_valid, actual_value).
    """
    terrain = output.get("terrain")
    if terrain is None:
        return False, None

    terrain = str(terrain).lower()
    valid = terrain in context.expected_terrains
    return valid, terrain


def validate_temperature(
    output: dict, context: TestContext
) -> tuple[bool, Optional[float]]:
    """Check if temperature makes physical sense for context.

    Returns (is_valid, actual_value).
    """
    temp = output.get("temperature")
    if temp is None:
        return False, None

    try:
        temp = float(temp)
    except (ValueError, TypeError):
        return False, None

    valid = context.temperature_min <= temp <= context.temperature_max
    return valid, temp


def validate_output(output: dict, context: TestContext) -> ValidationResult:
    """Validate a single generated output against context expectations."""
    feature_name = output.get("name", output.get("name_fragment", "unknown"))

    elev_valid, elev_actual = validate_elevation(output, context)
    terrain_valid, terrain_actual = validate_terrain(output, context)
    temp_valid, temp_actual = validate_temperature(output, context)

    return ValidationResult(
        context_name=context.name,
        feature_name=str(feature_name),
        raw_output=output,
        elevation_valid=elev_valid,
        terrain_valid=terrain_valid,
        temperature_valid=temp_valid,
        elevation_actual=elev_actual,
        terrain_actual=terrain_actual,
        temperature_actual=temp_actual,
    )


# =============================================================================
# Generation Runner
# =============================================================================


def run_generation_test(
    contexts: list[TestContext],
    samples_per_context: int = 5,
    client: Optional[DeepSeekClient] = None,
) -> list[ValidationResult]:
    """Generate samples for each context and validate structural validity.

    Args:
        contexts: List of test contexts to generate for.
        samples_per_context: Number of samples to generate per context.
        client: DeepSeek client (creates new if None).

    Returns:
        List of validation results.
    """
    if client is None:
        client = DeepSeekClient()

    results: list[ValidationResult] = []

    for context in contexts:
        print(f"\n{'='*60}")
        print(f"Testing: {context.name}")
        print(f"  Expected elevation: [{context.elevation_min}, {context.elevation_max}]")
        print(f"  Expected terrain: {context.expected_terrains}")
        print(f"  Expected temp: [{context.temperature_min}, {context.temperature_max}]")

        for i in range(samples_per_context):
            try:
                output = client.generate_json(
                    prompt=context.prompt_template,
                    temperature=1.0,  # High temp for freeform
                    max_tokens=1000,
                )
                result = validate_output(output, context)

                status = "PASS" if result.overall_valid else "FAIL"
                print(f"  [{i+1}/{samples_per_context}] {status}: {result.feature_name}")
                if not result.overall_valid:
                    if not result.elevation_valid:
                        print(
                            f"    - elevation: {result.elevation_actual} "
                            f"(expected {context.elevation_min} to {context.elevation_max})"
                        )
                    if not result.terrain_valid:
                        print(
                            f"    - terrain: {result.terrain_actual} "
                            f"(expected {context.expected_terrains})"
                        )
                    if not result.temperature_valid:
                        print(
                            f"    - temperature: {result.temperature_actual} "
                            f"(expected {context.temperature_min} to {context.temperature_max})"
                        )

            except Exception as e:
                result = ValidationResult(
                    context_name=context.name,
                    feature_name="ERROR",
                    raw_output={},
                    elevation_valid=False,
                    terrain_valid=False,
                    temperature_valid=False,
                    error=str(e),
                )
                print(f"  [{i+1}/{samples_per_context}] ERROR: {e}")

            results.append(result)

    return results


# =============================================================================
# CSV Output
# =============================================================================


def write_results_csv(
    results: list[ValidationResult],
    output_path: Optional[Path] = None,
) -> Path:
    """Write validation results to CSV.

    Args:
        results: Validation results to write.
        output_path: Where to write CSV. Defaults to output/structural_validity_TIMESTAMP.csv

    Returns:
        Path to the written CSV file.
    """
    if output_path is None:
        output_dir = Path(__file__).parent.parent / "output"
        output_dir.mkdir(parents=True, exist_ok=True)
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        output_path = output_dir / f"structural_validity_{timestamp}.csv"

    with open(output_path, "w", newline="") as f:
        writer = csv.writer(f)

        # Header
        writer.writerow(
            [
                "context",
                "feature_name",
                "elevation_valid",
                "terrain_valid",
                "temp_valid",
                "overall_valid",
                "elevation_actual",
                "terrain_actual",
                "temp_actual",
                "error",
            ]
        )

        # Data rows
        for r in results:
            writer.writerow(
                [
                    r.context_name,
                    r.feature_name,
                    1 if r.elevation_valid else 0,
                    1 if r.terrain_valid else 0,
                    1 if r.temperature_valid else 0,
                    1 if r.overall_valid else 0,
                    r.elevation_actual if r.elevation_actual is not None else "",
                    r.terrain_actual if r.terrain_actual is not None else "",
                    r.temperature_actual if r.temperature_actual is not None else "",
                    r.error if r.error else "",
                ]
            )

    print(f"\nResults written to: {output_path}")
    return output_path


# =============================================================================
# Summary Statistics
# =============================================================================


def print_summary(results: list[ValidationResult]) -> dict:
    """Print summary statistics and return as dict.

    Returns:
        Dict with violation rates per context and overall.
    """
    # Group by context
    by_context: dict[str, list[ValidationResult]] = defaultdict(list)
    for r in results:
        by_context[r.context_name].append(r)

    print("\n" + "=" * 70)
    print("STRUCTURAL VALIDITY SUMMARY")
    print("=" * 70)

    summary: dict = {
        "by_context": {},
        "overall": {},
    }

    total_samples = 0
    total_valid = 0
    total_elev_violations = 0
    total_terrain_violations = 0
    total_temp_violations = 0

    for context_name, context_results in sorted(by_context.items()):
        n = len(context_results)
        valid = sum(1 for r in context_results if r.overall_valid)
        elev_fail = sum(1 for r in context_results if not r.elevation_valid)
        terrain_fail = sum(1 for r in context_results if not r.terrain_valid)
        temp_fail = sum(1 for r in context_results if not r.temperature_valid)

        violation_rate = 1 - (valid / n) if n > 0 else 0

        summary["by_context"][context_name] = {
            "samples": n,
            "valid": valid,
            "violation_rate": violation_rate,
            "elevation_failures": elev_fail,
            "terrain_failures": terrain_fail,
            "temperature_failures": temp_fail,
        }

        print(f"\n{context_name}:")
        print(f"  Samples: {n}")
        print(f"  Valid: {valid}/{n} ({100*valid/n:.1f}%)")
        print(f"  Violation rate: {100*violation_rate:.1f}%")
        print(f"  Failures by type:")
        print(f"    - elevation: {elev_fail}/{n} ({100*elev_fail/n:.1f}%)")
        print(f"    - terrain: {terrain_fail}/{n} ({100*terrain_fail/n:.1f}%)")
        print(f"    - temperature: {temp_fail}/{n} ({100*temp_fail/n:.1f}%)")

        total_samples += n
        total_valid += valid
        total_elev_violations += elev_fail
        total_terrain_violations += terrain_fail
        total_temp_violations += temp_fail

    overall_violation_rate = (
        1 - (total_valid / total_samples) if total_samples > 0 else 0
    )

    summary["overall"] = {
        "total_samples": total_samples,
        "total_valid": total_valid,
        "violation_rate": overall_violation_rate,
        "elevation_failure_rate": (
            total_elev_violations / total_samples if total_samples > 0 else 0
        ),
        "terrain_failure_rate": (
            total_terrain_violations / total_samples if total_samples > 0 else 0
        ),
        "temperature_failure_rate": (
            total_temp_violations / total_samples if total_samples > 0 else 0
        ),
    }

    print("\n" + "-" * 70)
    print("OVERALL:")
    print(f"  Total samples: {total_samples}")
    print(
        f"  Total valid: {total_valid}/{total_samples} "
        f"({100*total_valid/total_samples:.1f}%)"
    )
    print(f"  Overall violation rate: {100*overall_violation_rate:.1f}%")
    print(f"  Failure breakdown:")
    print(f"    - elevation: {100*summary['overall']['elevation_failure_rate']:.1f}%")
    print(f"    - terrain: {100*summary['overall']['terrain_failure_rate']:.1f}%")
    print(
        f"    - temperature: {100*summary['overall']['temperature_failure_rate']:.1f}%"
    )
    print("=" * 70)

    return summary


# =============================================================================
# Main Entry Point
# =============================================================================


def main() -> int:
    """Run the structural validity test harness."""
    import argparse

    parser = argparse.ArgumentParser(
        description="Test structural validity of freeform LLM generation"
    )
    parser.add_argument(
        "--samples",
        "-n",
        type=int,
        default=5,
        help="Samples per context (default: 5)",
    )
    parser.add_argument(
        "--contexts",
        type=int,
        default=None,
        help="Number of contexts to test (default: all 10)",
    )
    parser.add_argument(
        "--output",
        "-o",
        type=Path,
        default=None,
        help="Output CSV path (default: auto-generated)",
    )
    args = parser.parse_args()

    contexts = TEST_CONTEXTS[: args.contexts] if args.contexts else TEST_CONTEXTS

    print(f"Testing {len(contexts)} contexts with {args.samples} samples each")
    print(f"Total API calls: {len(contexts) * args.samples}")

    results = run_generation_test(contexts, samples_per_context=args.samples)

    csv_path = write_results_csv(results, args.output)

    summary = print_summary(results)

    # Exit with error code if violation rate > 50%
    if summary["overall"]["violation_rate"] > 0.5:
        print(
            "\nWARNING: High violation rate indicates freeform generation needs constraints!"
        )
        return 1
    return 0


if __name__ == "__main__":
    import sys

    sys.exit(main())
