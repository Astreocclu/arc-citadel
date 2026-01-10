#!/usr/bin/env python3
"""Fix constraint violations in placements JSON."""

import json
import sys

def fix_stone_wall(params):
    """Apply stone_wall constraints."""
    # Constraint: length >= 2
    params["length"] = max(2.0, params.get("length", 5.0))
    # Constraint: height <= length * 0.8
    max_height = params["length"] * 0.8
    params["height"] = min(params.get("height", 2.5), max_height)
    params["height"] = round(max(1.0, params["height"]), 1)
    # Constraint: thickness >= height * 0.15
    min_thickness = params["height"] * 0.15
    params["thickness"] = max(params.get("thickness", 0.6), min_thickness)
    params["thickness"] = round(max(0.4, params["thickness"]), 1)
    return params

def main():
    input_file = sys.argv[1] if len(sys.argv) > 1 else "worldgen/placements_deepseek.json"
    output_file = sys.argv[2] if len(sys.argv) > 2 else "worldgen/placements.json"

    with open(input_file) as f:
        data = json.load(f)

    fixed = 0
    for placement in data["placements"]:
        if placement["template"] == "stone_wall":
            placement["parameters"] = fix_stone_wall(placement["parameters"])
            fixed += 1

    with open(output_file, "w") as f:
        json.dump(data, f, indent=2)

    print(f"Fixed {fixed} stone_wall placements")
    print(f"Output written to {output_file}")

if __name__ == "__main__":
    main()
