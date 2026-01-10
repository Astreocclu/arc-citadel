#!/usr/bin/env python3
"""
Placement Generator - Uses DeepSeek to generate object placements for hexes.

Takes tagged hex data and generates placement JSON for the Rust loader.
"""

import json
import os
import random
from dataclasses import dataclass, field
from typing import Optional
from pathlib import Path

try:
    import httpx
    HTTPX_AVAILABLE = True
except ImportError:
    HTTPX_AVAILABLE = False

# DeepSeek API configuration
DEEPSEEK_API_URL = "https://api.deepseek.com/v1/chat/completions"
DEEPSEEK_MODEL = "deepseek-chat"


@dataclass
class Placement:
    """A single object placement."""
    id: str
    template: str
    position: list[float]
    rotation_deg: Optional[float] = None
    placed_by: str | dict = "TerrainGen"
    state: Optional[str] = None
    damage_state: Optional[str] = None
    current_hp_ratio: Optional[float] = None
    construction_progress: Optional[float] = None
    parameters: dict = field(default_factory=dict)
    tags: list[str] = field(default_factory=list)

    def to_dict(self) -> dict:
        d = {
            "id": self.id,
            "template": self.template,
            "position": self.position,
            "placed_by": self.placed_by,
            "parameters": self.parameters,
        }
        if self.rotation_deg is not None:
            d["rotation_deg"] = self.rotation_deg
        if self.state is not None:
            d["state"] = self.state
        if self.damage_state is not None:
            d["damage_state"] = self.damage_state
        if self.current_hp_ratio is not None:
            d["current_hp_ratio"] = self.current_hp_ratio
        if self.construction_progress is not None:
            d["construction_progress"] = self.construction_progress
        if self.tags:
            d["tags"] = self.tags
        return d


# Available blueprints with their typical parameters
BLUEPRINTS = {
    "natural": {
        "oak_tree": {
            "height": (8.0, 25.0, 15.0),
            "canopy_radius": (3.0, 10.0, 5.0),
            "trunk_radius": (0.3, 1.5, 0.5),
        },
        "pine_tree": {
            "height": (10.0, 35.0, 20.0),
            "trunk_radius": (0.2, 1.0, 0.4),
        },
        "rock_outcrop": {
            "width": (1.0, 15.0, 4.0),
            "depth": (1.0, 10.0, 3.0),
            "height": (0.5, 6.0, 2.0),
        },
        "boulder": {
            "size": (1.0, 4.0, 2.0),
        },
    },
    "constructed": {
        "stone_wall": {
            "length": (5.0, 20.0, 8.0),  # Min length increased for stability
            "height": (1.0, 4.0, 2.5),
            "thickness": (0.5, 1.5, 0.6),
        },
        "wooden_house": {
            "width": (4.0, 12.0, 6.0),
            "depth": (4.0, 10.0, 5.0),
            "stories": (1, 2, 1),
        },
        "watchtower": {
            "base_width": (3.0, 8.0, 4.0),
            "height": (8.0, 20.0, 12.0),
        },
        "shrine": {
            "size": (2.0, 6.0, 3.0),
            "height": (2.0, 5.0, 3.0),
        },
        "well": {
            "diameter": (1.0, 3.0, 1.5),
            "depth": (5.0, 20.0, 10.0),
        },
    },
}


def random_param(param_range: tuple) -> float:
    """Generate a random parameter within range."""
    min_val, max_val, default = param_range
    # Bias toward default with some variance
    variance = (max_val - min_val) * 0.3
    value = default + random.gauss(0, variance)
    return max(min_val, min(max_val, value))


def generate_parameters(blueprint: str) -> dict:
    """Generate random parameters for a blueprint."""
    for category in BLUEPRINTS.values():
        if blueprint in category:
            params = {}
            for name, range_tuple in category[blueprint].items():
                val = random_param(range_tuple)
                # Round appropriately
                if name in ("stories",):
                    params[name] = int(round(val))
                else:
                    params[name] = round(val, 1)

            # Apply blueprint-specific constraints
            if blueprint == "stone_wall":
                # Constraint: length >= 2
                params["length"] = max(2.0, params["length"])
                # Constraint: height <= length * 0.8
                max_height = params["length"] * 0.8
                params["height"] = min(params["height"], max_height)
                params["height"] = round(max(1.0, params["height"]), 1)
                # Constraint: thickness >= height * 0.15
                min_thickness = params["height"] * 0.15
                params["thickness"] = max(params["thickness"], min_thickness)
                params["thickness"] = round(max(0.4, params["thickness"]), 1)

            return params
    return {}


def hex_to_world_pos(q: int, r: int, hex_size: float = 100.0) -> tuple[float, float]:
    """Convert hex coordinates to world position (center of hex)."""
    # Pointy-top hex layout
    x = hex_size * (3/2 * q)
    y = hex_size * (3**0.5 * (r + q/2))
    return (x, y)


def random_pos_in_hex(hex_center: tuple[float, float], hex_size: float = 100.0) -> list[float]:
    """Generate a random position within a hex."""
    cx, cy = hex_center
    # Random offset within hex (simplified - treats as square)
    offset_x = random.uniform(-hex_size * 0.4, hex_size * 0.4)
    offset_y = random.uniform(-hex_size * 0.4, hex_size * 0.4)
    return [round(cx + offset_x, 1), round(cy + offset_y, 1)]


SYSTEM_PROMPT = """You are a world generator for a fantasy strategy game. Given a hex's description and tags,
you must decide what objects to place in it.

Available blueprints:
NATURAL (placed_by: "TerrainGen"):
- oak_tree: deciduous tree, good for forests
- pine_tree: conifer, good for highlands/cold areas
- rock_outcrop: large rock formation
- boulder: single large stone

CONSTRUCTED (placed_by: {"HistorySim": {"polity_id": <random 1-100>, "year": <-500 to -50>}}):
- stone_wall: defensive wall (use state: "complete" for ancient, or "under_construction" for recent)
- wooden_house: residential dwelling
- watchtower: observation/defense tower
- shrine: small sacred structure
- well: water source

For ANCIENT hexes, structures should be:
- state: "complete" with possible damage_state: "damaged" and current_hp_ratio: 0.3-0.8
- placed_by: {"HistorySim": {"polity_id": <number>, "year": <negative number like -300>}}

For WILD/NATURAL hexes:
- Mostly natural objects (trees, rocks)
- placed_by: "TerrainGen"

For RESIDENTIAL/MILITARY hexes:
- Mix of buildings and natural objects
- Buildings complete or under construction

Output JSON array of objects to place. Each object:
{
  "template": "<blueprint_name>",
  "count": <1-5>,
  "state": "complete" | "under_construction" | null,
  "damage_state": "intact" | "damaged" | "ruined" | null,
  "hp_ratio": <0.0-1.0 or null>,
  "origin": "natural" | "ancient" | "recent"
}

Keep it sparse - a 100m hex shouldn't have more than 5-15 objects total.
"""


def call_deepseek(hex_data: dict, api_key: str) -> list[dict]:
    """Call DeepSeek API to generate placements for a hex."""
    if not HTTPX_AVAILABLE:
        print("  httpx not available, using fallback")
        return []

    user_prompt = f"""Hex: {hex_data.get('name', 'Unknown')}
Description: {hex_data.get('description', 'No description')}
Tags: {', '.join(hex_data.get('tags', []))}
Terrain: {hex_data.get('terrain', 'unknown')}
Elevation: {hex_data.get('elevation', 0)}

What objects should be placed in this 100m x 100m hex? Return JSON array only."""

    try:
        response = httpx.post(
            DEEPSEEK_API_URL,
            headers={
                "Authorization": f"Bearer {api_key}",
                "Content-Type": "application/json",
            },
            json={
                "model": DEEPSEEK_MODEL,
                "messages": [
                    {"role": "system", "content": SYSTEM_PROMPT},
                    {"role": "user", "content": user_prompt},
                ],
                "temperature": 0.7,
                "max_tokens": 500,
            },
            timeout=30.0,
        )
        response.raise_for_status()
        content = response.json()["choices"][0]["message"]["content"]

        # Extract JSON from response
        if "```json" in content:
            content = content.split("```json")[1].split("```")[0]
        elif "```" in content:
            content = content.split("```")[1].split("```")[0]

        return json.loads(content.strip())
    except Exception as e:
        print(f"  DeepSeek error: {e}")
        return []


def fallback_generate(hex_data: dict) -> list[dict]:
    """Fallback generation without LLM."""
    tags = hex_data.get("tags", [])
    terrain = hex_data.get("terrain", "plains")
    objects = []

    # Natural objects based on terrain
    if terrain in ("forest", "plains", "hills"):
        tree_count = random.randint(2, 8) if "wild" in tags else random.randint(0, 3)
        for _ in range(tree_count):
            tree_type = "pine_tree" if hex_data.get("elevation", 0) > 300 else "oak_tree"
            objects.append({"template": tree_type, "count": 1, "origin": "natural"})

    # Rocks
    if terrain in ("hills", "mountain") or random.random() < 0.2:
        rock_count = random.randint(1, 3)
        for _ in range(rock_count):
            rock_type = "rock_outcrop" if random.random() < 0.3 else "boulder"
            objects.append({"template": rock_type, "count": 1, "origin": "natural"})

    # Buildings for non-wild hexes
    if "residential" in tags:
        objects.append({"template": "wooden_house", "count": random.randint(1, 3), "origin": "ancient", "state": "complete"})
        objects.append({"template": "well", "count": 1, "origin": "ancient", "state": "complete"})

    if "military" in tags:
        objects.append({"template": "watchtower", "count": 1, "origin": "ancient", "state": "complete", "damage_state": "damaged", "hp_ratio": 0.6})
        objects.append({"template": "stone_wall", "count": random.randint(2, 4), "origin": "ancient", "state": "complete"})

    if "sacred" in tags:
        objects.append({"template": "shrine", "count": 1, "origin": "ancient", "state": "complete"})

    if "ancient" in tags and not any(t in tags for t in ["residential", "military", "sacred"]):
        # Ancient ruins
        objects.append({"template": "stone_wall", "count": random.randint(1, 3), "origin": "ancient", "state": "complete", "damage_state": "damaged", "hp_ratio": 0.4})

    return objects


def expand_placements(hex_data: dict, objects: list[dict], placement_id_counter: int) -> tuple[list[Placement], int]:
    """Expand object specifications into individual placements."""
    q = hex_data["coord"]["q"]
    r = hex_data["coord"]["r"]
    hex_center = hex_to_world_pos(q, r)

    placements = []

    for obj in objects:
        template = obj.get("template")
        if not template:
            continue

        count = obj.get("count", 1)
        origin = obj.get("origin", "natural")
        state = obj.get("state")
        damage_state = obj.get("damage_state")
        hp_ratio = obj.get("hp_ratio")

        for i in range(count):
            placement_id_counter += 1

            # Determine placed_by
            if origin == "natural":
                placed_by = "TerrainGen"
            elif origin == "ancient":
                placed_by = {
                    "HistorySim": {
                        "polity_id": random.randint(1, 100),
                        "year": random.randint(-500, -50),
                    }
                }
            else:
                placed_by = {"Gameplay": {"tick": 0}}

            placement = Placement(
                id=f"{template}_{placement_id_counter:05d}",
                template=template,
                position=random_pos_in_hex(hex_center),
                rotation_deg=random.uniform(0, 360) if template not in ("well",) else 0,
                placed_by=placed_by,
                state=state,
                damage_state=damage_state,
                current_hp_ratio=hp_ratio,
                parameters=generate_parameters(template),
                tags=[f"hex_{q}_{r}"],
            )
            placements.append(placement)

    return placements, placement_id_counter


def generate_world_placements(
    hex_file: str,
    output_file: str,
    use_deepseek: bool = False,
    api_key: Optional[str] = None,
    max_hexes: Optional[int] = None,
) -> None:
    """Generate placements for all hexes in a world file."""

    print(f"Loading hex data from {hex_file}...")
    with open(hex_file) as f:
        world_data = json.load(f)

    hexes = world_data.get("hex_map", {}).get("hexes", {})
    print(f"Found {len(hexes)} hexes")

    if max_hexes:
        # Take a sample
        hex_keys = list(hexes.keys())[:max_hexes]
        hexes = {k: hexes[k] for k in hex_keys}
        print(f"Processing {len(hexes)} hexes (limited)")

    all_placements = []
    placement_id_counter = 0

    for i, (hex_key, hex_data) in enumerate(hexes.items()):
        if i % 50 == 0:
            print(f"Processing hex {i+1}/{len(hexes)}...")

        # Generate objects for this hex
        if use_deepseek and api_key:
            objects = call_deepseek(hex_data, api_key)
            if not objects:
                objects = fallback_generate(hex_data)
        else:
            objects = fallback_generate(hex_data)

        # Expand into placements
        placements, placement_id_counter = expand_placements(
            hex_data, objects, placement_id_counter
        )
        all_placements.extend(placements)

    # Build output
    output = {
        "version": 1,
        "metadata": {
            "name": world_data.get("name", "Generated World"),
            "description": f"Auto-generated placements for {len(hexes)} hexes",
            "created_by": "placement_generator.py",
        },
        "placements": [p.to_dict() for p in all_placements],
    }

    print(f"Writing {len(all_placements)} placements to {output_file}...")
    with open(output_file, "w") as f:
        json.dump(output, f, indent=2)

    print("Done!")


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Generate object placements for hex world")
    parser.add_argument("--input", "-i", default="worldgen/tagged_world.json", help="Input hex file")
    parser.add_argument("--output", "-o", default="worldgen/placements.json", help="Output placements file")
    parser.add_argument("--deepseek", action="store_true", help="Use DeepSeek API")
    parser.add_argument("--max-hexes", "-n", type=int, help="Limit number of hexes to process")

    args = parser.parse_args()

    api_key = os.environ.get("DEEPSEEK_API_KEY") if args.deepseek else None

    generate_world_placements(
        hex_file=args.input,
        output_file=args.output,
        use_deepseek=args.deepseek,
        api_key=api_key,
        max_hexes=args.max_hexes,
    )
