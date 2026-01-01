#!/usr/bin/env python3
"""Batch runner for overnight hex generation."""

import argparse
import json
import logging
import random
import string
import sys
import time
from datetime import datetime
from pathlib import Path

from hex_generator import HexGenerator
from validator import HexValidator
from schemas import GenerationSeed


logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    handlers=[
        logging.StreamHandler(),
        logging.FileHandler("batch_run.log"),
    ],
)
logger = logging.getLogger(__name__)


REGION_THEMES = [
    ("Verdant Valley", "lush river valley with fertile farmland"),
    ("Iron Peaks", "rugged mountain range rich in ore deposits"),
    ("Twilight Marsh", "misty swampland with ancient secrets"),
    ("Golden Sands", "desert expanse with hidden oases"),
    ("Frozen North", "tundra wasteland with frozen lakes"),
    ("Whispering Woods", "dense forest with mystical clearings"),
    ("Coastal Reach", "seaside cliffs and fishing coves"),
    ("Volcanic Wastes", "scorched earth near dormant volcanoes"),
    ("Highland Moors", "rolling hills with standing stones"),
    ("River Delta", "wetlands where great rivers meet the sea"),
]


def generate_seed_id() -> str:
    """Generate a unique seed identifier."""
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    suffix = "".join(random.choices(string.ascii_lowercase, k=4))
    return f"seed_{timestamp}_{suffix}"


def pick_regions(count: int = 3) -> list[dict]:
    """Pick random region configurations."""
    selected = random.sample(REGION_THEMES, min(count, len(REGION_THEMES)))
    return [
        {
            "region_name": name,
            "region_theme": theme,
            "hex_count": random.randint(12, 25),
        }
        for name, theme in selected
    ]


def run_batch(
    target_count: int,
    prompt_path: Path,
    output_dir: Path,
    delay_between: float = 2.0,
    max_failures: int = 10,
):
    """Run batch generation until target count reached or max failures."""
    valid_dir = output_dir / "valid"
    invalid_dir = output_dir / "invalid"
    valid_dir.mkdir(parents=True, exist_ok=True)
    invalid_dir.mkdir(parents=True, exist_ok=True)

    validator = HexValidator()
    valid_count = 0
    failure_count = 0
    total_attempts = 0

    logger.info(f"Starting batch run: target={target_count} seeds")
    logger.info(f"Prompt: {prompt_path}")
    logger.info(f"Output: {output_dir}")

    with HexGenerator() as generator:
        while valid_count < target_count and failure_count < max_failures:
            total_attempts += 1
            seed_id = generate_seed_id()
            regions_config = pick_regions(count=random.randint(2, 4))

            logger.info(f"Attempt {total_attempts}: Generating {seed_id}")
            logger.info(f"  Regions: {[r['region_name'] for r in regions_config]}")

            try:
                seed = generator.generate_seed(
                    prompt_path=prompt_path,
                    seed_id=seed_id,
                    regions_config=regions_config,
                )

                if seed is None:
                    logger.warning(f"  Generation returned None")
                    failure_count += 1
                    continue

                result = validator.validate_seed(seed)

                seed_data = seed.model_dump()
                seed_data["validation"] = {
                    "valid": result.valid,
                    "errors": result.errors,
                    "warnings": result.warnings,
                }

                if result.valid:
                    out_path = valid_dir / f"{seed_id}.json"
                    out_path.write_text(json.dumps(seed_data, indent=2))
                    valid_count += 1
                    logger.info(f"  Valid seed saved: {out_path.name}")
                    logger.info(f"  Progress: {valid_count}/{target_count}")
                    failure_count = 0
                else:
                    out_path = invalid_dir / f"{seed_id}.json"
                    out_path.write_text(json.dumps(seed_data, indent=2))
                    logger.warning(f"  Invalid seed: {result.errors[:3]}")
                    failure_count += 1

                if result.warnings:
                    logger.info(f"  Warnings: {result.warnings[:2]}")

            except Exception as e:
                logger.error(f"  Error: {e}")
                failure_count += 1

            if valid_count < target_count:
                time.sleep(delay_between)

    logger.info("=" * 50)
    logger.info(f"Batch complete:")
    logger.info(f"  Valid seeds: {valid_count}")
    logger.info(f"  Total attempts: {total_attempts}")
    logger.info(f"  Success rate: {valid_count/max(total_attempts,1)*100:.1f}%")

    return valid_count


def main():
    parser = argparse.ArgumentParser(description="Batch hex generation runner")
    parser.add_argument(
        "-n", "--count",
        type=int,
        default=100,
        help="Target number of valid seeds (default: 100)",
    )
    parser.add_argument(
        "-p", "--prompt",
        type=Path,
        default=Path(__file__).parent / "prompts" / "hex_prompt.txt",
        help="Path to prompt template",
    )
    parser.add_argument(
        "-o", "--output",
        type=Path,
        default=Path(__file__).parent / "output",
        help="Output directory",
    )
    parser.add_argument(
        "-d", "--delay",
        type=float,
        default=2.0,
        help="Delay between API calls in seconds",
    )
    parser.add_argument(
        "--max-failures",
        type=int,
        default=10,
        help="Stop after this many consecutive failures",
    )

    args = parser.parse_args()

    if not args.prompt.exists():
        logger.error(f"Prompt file not found: {args.prompt}")
        sys.exit(1)

    valid_count = run_batch(
        target_count=args.count,
        prompt_path=args.prompt,
        output_dir=args.output,
        delay_between=args.delay,
        max_failures=args.max_failures,
    )

    sys.exit(0 if valid_count >= args.count else 1)


if __name__ == "__main__":
    main()
