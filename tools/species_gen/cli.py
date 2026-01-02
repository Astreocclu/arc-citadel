#!/usr/bin/env python3
"""Species generator CLI."""

import argparse
import sys
from pathlib import Path

# Add parent to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from tools.species_gen.generator import SpeciesGenerator
from tools.species_gen.patcher import SpeciesPatcher


def main():
    parser = argparse.ArgumentParser(
        description="Generate Rust code for new species from TOML definitions"
    )
    parser.add_argument(
        "spec",
        type=Path,
        help="Path to species TOML specification file"
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be generated without writing files"
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Overwrite existing files"
    )
    parser.add_argument(
        "--verify-markers",
        action="store_true",
        help="Check that all required markers exist in codebase"
    )
    parser.add_argument(
        "--project-root",
        type=Path,
        default=Path(__file__).parent.parent.parent,
        help="Project root directory (default: auto-detect)"
    )

    args = parser.parse_args()

    generator = SpeciesGenerator(args.project_root)
    patcher = SpeciesPatcher(args.project_root)

    # Load spec
    try:
        spec = generator.load_spec(args.spec)
    except Exception as e:
        print(f"Error loading spec: {e}", file=sys.stderr)
        sys.exit(1)

    name = spec["metadata"]["name"]
    module_name = spec["metadata"]["module_name"]
    print(f"Generating species: {name} (module: {module_name})")

    # Generate patches
    patches = patcher.generate_patches(spec)

    # Verify markers if requested
    if args.verify_markers:
        missing = patcher.verify_markers_exist(patches)
        if missing:
            print("Missing markers:", file=sys.stderr)
            for m in missing:
                print(f"  - {m}", file=sys.stderr)
            sys.exit(1)
        print("All markers present!")
        return

    # Check markers exist before proceeding
    missing = patcher.verify_markers_exist(patches)
    if missing:
        print("ERROR: Missing markers in codebase. Add markers first.", file=sys.stderr)
        print("Missing markers:", file=sys.stderr)
        for m in missing:
            print(f"  - {m}", file=sys.stderr)
        sys.exit(1)

    # Generate new files
    print("\n=== Generated Files ===")
    generated = generator.generate_all(args.spec, dry_run=args.dry_run, force=args.force)
    for filepath in generated:
        status = "[DRY RUN]" if args.dry_run else "[CREATED]"
        print(f"{status} {filepath}")

    if not generated:
        print("(No new files - entity archetype already exists. Use --force to overwrite)")

    # Apply patches
    print("\n=== Patched Files ===")
    patched = patcher.apply_patches(patches, dry_run=args.dry_run)
    for filepath, (original, modified) in patched.items():
        if original != modified:
            status = "[DRY RUN]" if args.dry_run else "[PATCHED]"
            print(f"{status} {filepath}")

    if args.dry_run:
        print("\nDry run complete. No files modified.")
    else:
        print("\nGeneration complete. Run 'cargo check' to verify.")


if __name__ == "__main__":
    main()
