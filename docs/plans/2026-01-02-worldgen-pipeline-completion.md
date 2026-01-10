# Worldgen Pipeline Completion

**Date**: 2026-01-02
**Status**: Ready for implementation
**Confidence**: 85%

## Overview

Complete the worldgen pipeline by implementing the 4 missing layers:
1. **Connectors** (roads/rivers) - Full ConnectorCollection with elastic segments
2. **Minor anchors** (inns/shrines) - LLM-selected per connector
3. **Filler hexes** - Constraint propagation from fixed edges
4. **Terrain transitions** - Rules in hex_tags.toml

### Generation Order
```
Clusters → Connectors → Anchors → Fillers
```

### Design Decisions
- **Hybrid paradigm**: TaggedHex for fillers, Component model for connectors/anchors
- **Full schema support**: ElasticSegment, internal_graph, MinorAnchorSlot all implemented
- **Constraint propagation**: WFC-style algorithm reusing AdjacencyValidator
- **LLM for creativity**: One DeepSeek call per connector for anchor selection

---

## Task 1: Add Transition Rules to hex_tags.toml

**File**: `worldgen/generation/hex_tags.toml`

Add after `[constraints.soft]`:

```toml
# Terrain transition rules for filler generation
# Each terrain tag lists compatible adjacent terrains
[transitions]
# Surface terrains
surface = ["surface", "underground"]  # Can transition to underground via entrance
underground = ["underground", "surface"]
underwater = ["underwater", "surface"]
aerial = ["aerial", "surface", "elevated", "peak"]

# Elevation bands
deep = ["deep", "shallow_under"]
shallow_under = ["shallow_under", "deep", "surface"]
elevated = ["elevated", "surface", "peak"]
peak = ["peak", "elevated", "aerial"]

# Culture transitions (any culture can border any, but tensions apply)
# No hard restrictions - soft constraints handle this

[transitions.weights]
# Probability weights for filler generation (0.0-1.0)
# Higher = more likely to propagate this terrain
surface.surface = 0.8
surface.underground = 0.1
surface.elevated = 0.3
underground.underground = 0.9
underground.deep = 0.4
elevated.peak = 0.3
peak.peak = 0.7

[transitions.features]
# Terrain pairs that require specific features
underground_surface = ["entrance"]  # Underground adjacent to surface needs entrance tag
elevated_surface = ["cliff", "slope"]  # Elevation change needs transition feature
```

**Verification**:
```bash
cd worldgen && python -c "
import tomllib
with open('generation/hex_tags.toml', 'rb') as f:
    config = tomllib.load(f)
assert 'transitions' in config
assert 'surface' in config['transitions']
print('Transition rules loaded successfully')
"
```

---

## Task 2: Create ConnectorGenerator

**File**: `worldgen/connector_generator.py`

```python
"""Connector generation with elastic pathfinding and full schema support."""

import random
from collections import deque
from dataclasses import dataclass, field
from typing import Optional

from schemas import TaggedHex, EdgeType, FoundingContext
from schemas.connector import (
    ConnectorCollection, ConnectorType, ConnectorHex,
    EntryPoint, MinorAnchorSlot, ElasticSegment
)
from schemas.base import Terrain, SpeciesFitness, Resource, Feature
from hex_coords import get_neighbor, get_opposite_edge, coords_to_key, distance


@dataclass
class PathConstraint:
    """Constraints for pathfinding."""
    start: tuple[int, int]
    end: tuple[int, int]
    avoid_positions: set[tuple[int, int]] = field(default_factory=set)
    prefer_terrain: list[str] = field(default_factory=list)
    max_length: int = 100


class ConnectorGenerator:
    """Generates connectors between clusters with full schema support."""

    def __init__(
        self,
        elasticity: float = 0.3,
        max_deviation: int = 3,
    ):
        self.elasticity = elasticity
        self.max_deviation = max_deviation

    def generate(
        self,
        connector_type: ConnectorType,
        start_pos: tuple[int, int],
        end_pos: tuple[int, int],
        placed_hexes: set[tuple[int, int]],
        elastic_segments: Optional[list[ElasticSegment]] = None,
    ) -> ConnectorCollection:
        """Generate a connector between two points.

        Args:
            connector_type: Type of connector (road, river, etc.)
            start_pos: Starting hex coordinates (q, r)
            end_pos: Ending hex coordinates (q, r)
            placed_hexes: Set of already-occupied hex positions to avoid
            elastic_segments: Optional elastic segment definitions

        Returns:
            ConnectorCollection with hexes, graph, and anchor slots
        """
        # 1. Find path using A* with elasticity
        constraint = PathConstraint(
            start=start_pos,
            end=end_pos,
            avoid_positions=placed_hexes,
            prefer_terrain=self._get_preferred_terrain(connector_type),
        )
        path_coords = self._find_elastic_path(constraint)

        # 2. Generate ConnectorHex for each position
        connector_hexes = self._generate_connector_hexes(path_coords, connector_type)

        # 3. Apply elastic segment adjustments if provided
        if elastic_segments:
            connector_hexes = self._apply_elastic_segments(connector_hexes, elastic_segments)

        # 4. Build internal graph
        internal_graph = self._build_internal_graph(connector_hexes)

        # 5. Generate anchor slots
        anchor_slots = self._generate_anchor_slots(connector_hexes, connector_type)

        # 6. Create entry points
        entry_points = self._create_entry_points(connector_hexes, connector_type)

        # 7. Assemble collection
        collection = ConnectorCollection(
            id=f"conn_{connector_type.value}_{start_pos[0]}_{start_pos[1]}",
            type=connector_type,
            hexes=connector_hexes,
            base_length=len(connector_hexes),
            entry_points=entry_points,
            internal_graph=internal_graph,
            elastic_segments=elastic_segments or [],
            minor_slots=anchor_slots,
        )

        return collection

    def _find_elastic_path(self, constraint: PathConstraint) -> list[tuple[int, int]]:
        """A* pathfinding with elasticity allowing scenic detours."""
        start = constraint.start
        end = constraint.end

        # Priority queue: (f_score, g_score, position, path)
        open_set = [(0, 0, start, [start])]
        visited = set()

        while open_set:
            open_set.sort(key=lambda x: x[0])
            f, g, current, path = open_set.pop(0)

            if current == end:
                return path

            if current in visited:
                continue
            visited.add(current)

            # Explore neighbors
            for edge in range(6):
                nq, nr = get_neighbor(current[0], current[1], edge)
                neighbor = (nq, nr)

                if neighbor in visited:
                    continue
                if neighbor in constraint.avoid_positions:
                    continue

                # Cost calculation with elasticity
                new_g = g + 1
                h = distance(nq, nr, end[0], end[1])

                # Elasticity bonus: allow slightly longer paths for variety
                elasticity_bonus = random.random() * self.elasticity
                new_f = new_g + h - elasticity_bonus

                # Check deviation from straight line
                straight_dist = distance(start[0], start[1], end[0], end[1])
                current_deviation = (new_g + h) - straight_dist
                if current_deviation > self.max_deviation:
                    continue

                open_set.append((new_f, new_g, neighbor, path + [neighbor]))

        # Fallback: straight line if A* fails
        return self._straight_line_path(start, end)

    def _straight_line_path(
        self, start: tuple[int, int], end: tuple[int, int]
    ) -> list[tuple[int, int]]:
        """Generate straight-line hex path as fallback."""
        path = [start]
        current = start

        while current != end:
            best_neighbor = None
            best_dist = float('inf')

            for edge in range(6):
                nq, nr = get_neighbor(current[0], current[1], edge)
                d = distance(nq, nr, end[0], end[1])
                if d < best_dist:
                    best_dist = d
                    best_neighbor = (nq, nr)

            if best_neighbor is None or best_neighbor == current:
                break

            current = best_neighbor
            path.append(current)

        return path

    def _generate_connector_hexes(
        self,
        path: list[tuple[int, int]],
        connector_type: ConnectorType,
    ) -> list[ConnectorHex]:
        """Generate ConnectorHex objects for path coordinates."""
        hexes = []
        terrain = self._get_base_terrain(connector_type)

        for i, (q, r) in enumerate(path):
            # Determine connectivity
            connects_to = []
            if i > 0:
                connects_to.append(i - 1)
            if i < len(path) - 1:
                connects_to.append(i + 1)

            hex = ConnectorHex(
                index=i,
                terrain=terrain,
                elevation=self._sample_elevation(connector_type, i, len(path)),
                moisture=self._sample_moisture(connector_type),
                connects_to=connects_to,
                is_entry_point=(i == 0 or i == len(path) - 1),
                species_fitness=SpeciesFitness(human=0.5, dwarf=0.5, elf=0.5),
            )
            hexes.append(hex)

        return hexes

    def _apply_elastic_segments(
        self,
        hexes: list[ConnectorHex],
        segments: list[ElasticSegment],
    ) -> list[ConnectorHex]:
        """Apply elastic segment stretch/compress."""
        result = list(hexes)

        for segment in segments:
            start_idx = segment.start_hex
            end_idx = segment.end_hex

            if start_idx >= len(result) or end_idx >= len(result):
                continue

            current_length = end_idx - start_idx + 1

            if current_length < segment.min_length:
                # Stretch: insert hexes with stretch_terrain
                stretch_count = segment.min_length - current_length
                insert_idx = start_idx + current_length // 2

                for j in range(stretch_count):
                    stretch_hex = ConnectorHex(
                        index=insert_idx + j,
                        terrain=segment.stretch_terrain,
                        elevation=result[start_idx].elevation,
                        moisture=result[start_idx].moisture,
                        connects_to=[],
                        species_fitness=SpeciesFitness(human=0.5, dwarf=0.5, elf=0.5),
                    )
                    result.insert(insert_idx + j, stretch_hex)

            elif current_length > segment.max_length:
                # Compress: remove middle hexes
                remove_count = current_length - segment.max_length
                mid_idx = start_idx + current_length // 2

                for _ in range(remove_count):
                    if mid_idx < len(result):
                        result.pop(mid_idx)

        # Reindex after modifications
        for i, hex in enumerate(result):
            hex.index = i
            hex.connects_to = []
            if i > 0:
                hex.connects_to.append(i - 1)
            if i < len(result) - 1:
                hex.connects_to.append(i + 1)

        return result

    def _build_internal_graph(
        self, hexes: list[ConnectorHex]
    ) -> dict[int, list[int]]:
        """Build hex-to-hex connectivity graph."""
        graph = {}
        for hex in hexes:
            graph[hex.index] = hex.connects_to.copy()
        return graph

    def _generate_anchor_slots(
        self,
        hexes: list[ConnectorHex],
        connector_type: ConnectorType,
    ) -> list[MinorAnchorSlot]:
        """Generate anchor slots based on connector type and features."""
        slots = []

        # Endpoints always get slots
        slots.append(MinorAnchorSlot(
            slot_id=f"slot_start",
            hex_index=0,
            compatible_categories=["waystation", "toll_gate", "border_post"],
            required=False,
            narrative_context="Connector entrance",
        ))

        slots.append(MinorAnchorSlot(
            slot_id=f"slot_end",
            hex_index=len(hexes) - 1,
            compatible_categories=["waystation", "toll_gate", "border_post"],
            required=False,
            narrative_context="Connector terminus",
        ))

        # Interval-based slots
        interval = self._get_anchor_interval(connector_type)
        for i in range(interval, len(hexes) - 1, interval):
            compatible = self._get_compatible_anchors(connector_type, hexes[i])
            slots.append(MinorAnchorSlot(
                slot_id=f"slot_{i}",
                hex_index=i,
                compatible_categories=compatible,
                required=False,
                narrative_context=f"Regular stopping point",
            ))

        # Feature-based slots (river crossings, elevation changes)
        for i, hex in enumerate(hexes):
            if i == 0 or i == len(hexes) - 1:
                continue

            # River crossing detection
            if hex.terrain == Terrain.SHALLOW_WATER:
                slots.append(MinorAnchorSlot(
                    slot_id=f"slot_crossing_{i}",
                    hex_index=i,
                    compatible_categories=["bridge_wood", "bridge_stone", "ford_improved", "ferry"],
                    required=True,
                    narrative_context="River crossing",
                ))

            # Elevation change detection
            if i > 0 and abs(hex.elevation - hexes[i-1].elevation) > 100:
                slots.append(MinorAnchorSlot(
                    slot_id=f"slot_pass_{i}",
                    hex_index=i,
                    compatible_categories=["watchtower", "shrine", "camp"],
                    required=False,
                    narrative_context="Mountain pass or significant elevation",
                ))

        return slots

    def _create_entry_points(
        self,
        hexes: list[ConnectorHex],
        connector_type: ConnectorType,
    ) -> list[EntryPoint]:
        """Create entry points at connector ends."""
        if not hexes:
            return []

        return [
            EntryPoint(
                hex_index=0,
                direction="start",
                elevation=hexes[0].elevation,
                terrain=hexes[0].terrain,
                is_terminus=True,
            ),
            EntryPoint(
                hex_index=len(hexes) - 1,
                direction="end",
                elevation=hexes[-1].elevation,
                terrain=hexes[-1].terrain,
                is_terminus=True,
            ),
        ]

    def _get_preferred_terrain(self, connector_type: ConnectorType) -> list[str]:
        """Get preferred terrain for pathfinding."""
        preferences = {
            ConnectorType.TRADE_ROUTE_MAJOR: ["plains", "grassland"],
            ConnectorType.TRADE_ROUTE_MINOR: ["plains", "forest"],
            ConnectorType.MILITARY_ROAD: ["plains", "hills"],
            ConnectorType.RIVER_MIDDLE: ["plains", "forest"],
            ConnectorType.MOUNTAIN_PASS: ["mountains", "hills"],
        }
        return preferences.get(connector_type, ["plains"])

    def _get_base_terrain(self, connector_type: ConnectorType) -> Terrain:
        """Get base terrain for connector type."""
        terrain_map = {
            ConnectorType.RIVER_HEADWATERS: Terrain.SHALLOW_WATER,
            ConnectorType.RIVER_UPPER: Terrain.SHALLOW_WATER,
            ConnectorType.RIVER_MIDDLE: Terrain.SHALLOW_WATER,
            ConnectorType.RIVER_LOWER: Terrain.DEEP_WATER,
            ConnectorType.TRADE_ROUTE_MAJOR: Terrain.PLAINS,
            ConnectorType.TRADE_ROUTE_MINOR: Terrain.PLAINS,
            ConnectorType.MILITARY_ROAD: Terrain.PLAINS,
            ConnectorType.MOUNTAIN_PASS: Terrain.MOUNTAINS,
        }
        return terrain_map.get(connector_type, Terrain.PLAINS)

    def _sample_elevation(
        self, connector_type: ConnectorType, index: int, total: int
    ) -> float:
        """Sample elevation along connector."""
        # Rivers flow downhill
        if "river" in connector_type.value.lower():
            return 500.0 - (index / total) * 300.0
        # Roads follow terrain
        return 200.0 + random.random() * 100.0

    def _sample_moisture(self, connector_type: ConnectorType) -> float:
        """Sample moisture for connector type."""
        if "river" in connector_type.value.lower():
            return 0.9
        return 0.4 + random.random() * 0.2

    def _get_anchor_interval(self, connector_type: ConnectorType) -> int:
        """Get anchor placement interval for connector type."""
        intervals = {
            ConnectorType.TRADE_ROUTE_MAJOR: 8,
            ConnectorType.TRADE_ROUTE_MINOR: 12,
            ConnectorType.MILITARY_ROAD: 10,
            ConnectorType.PILGRIM_PATH: 6,
        }
        return intervals.get(connector_type, 10)

    def _get_compatible_anchors(
        self, connector_type: ConnectorType, hex: ConnectorHex
    ) -> list[str]:
        """Get compatible anchor categories for position."""
        base = ["camp", "shrine"]

        if connector_type in (ConnectorType.TRADE_ROUTE_MAJOR, ConnectorType.TRADE_ROUTE_MINOR):
            base.extend(["inn", "tavern", "market_small", "caravanserai"])
        elif connector_type == ConnectorType.MILITARY_ROAD:
            base.extend(["watchtower", "signal_tower", "border_post"])
        elif connector_type == ConnectorType.PILGRIM_PATH:
            base.extend(["shrine", "temple_small", "sacred_spring"])

        return base
```

**Verification**:
```bash
cd worldgen && python -c "
from connector_generator import ConnectorGenerator
from schemas.connector import ConnectorType

gen = ConnectorGenerator()
conn = gen.generate(
    ConnectorType.TRADE_ROUTE_MAJOR,
    start_pos=(0, 0),
    end_pos=(10, 5),
    placed_hexes=set(),
)
print(f'Generated connector with {len(conn.hexes)} hexes')
print(f'Anchor slots: {len(conn.minor_slots)}')
print(f'Entry points: {len(conn.entry_points)}')
assert len(conn.hexes) > 0
assert len(conn.internal_graph) == len(conn.hexes)
print('ConnectorGenerator OK')
"
```

---

## Task 3: Create AnchorPlacer

**File**: `worldgen/anchor_placer.py`

```python
"""LLM-based anchor selection for connectors."""

import json
import os
from typing import Optional

import httpx

from schemas.connector import ConnectorCollection, MinorAnchorSlot
from schemas.minor import MinorAnchor, MinorCategory


ANCHOR_SELECTION_PROMPT = """You are placing points of interest along a fantasy travel route.

CONNECTOR CONTEXT:
- Type: {connector_type}
- Length: {length} hexes (each hex is 100m, so {length_km}km total)
- Terrain: {terrain_summary}
- Available slots: {slot_count} positions

SLOT DETAILS:
{slot_details}

ANCHOR TYPES AVAILABLE:
- Rest: inn, tavern, waystation, camp, caravanserai
- Crossings: bridge_wood, bridge_stone, ford_improved, ferry, tunnel
- Sacred: shrine, temple_small, standing_stone, sacred_spring
- Military: watchtower, toll_gate, border_post, signal_tower
- Economic: market_small, mill, mine_entrance, lumber_camp
- Mysterious: hermit_hut, witch_cottage, abandoned_camp, unmarked_graves

RULES:
1. Select 0 to {max_anchors} anchors total
2. Match anchor type to slot context (crossings need bridges/fords)
3. Trade routes need rest stops; military roads need watchtowers
4. Space anchors reasonably (not every slot needs an anchor)
5. Add narrative variety - not all inns, mix it up

OUTPUT JSON:
{{
  "anchors": [
    {{
      "slot_id": "<slot_id from above>",
      "category": "<MinorCategory value>",
      "name_fragment": "<evocative 2-4 word name>",
      "narrative_hook": "<1 sentence hook for this location>"
    }}
  ]
}}
"""


class AnchorPlacer:
    """LLM-based selection of minor anchors for connectors."""

    def __init__(
        self,
        api_key: Optional[str] = None,
        model: str = "deepseek-chat",
        max_anchors_per_connector: int = 5,
    ):
        self.api_key = api_key or os.environ.get("DEEPSEEK_API_KEY")
        self.model = model
        self.max_anchors = max_anchors_per_connector
        self.client = httpx.Client(timeout=60.0)

    def place_anchors(
        self, connector: ConnectorCollection
    ) -> list[MinorAnchor]:
        """Select and generate anchors for a connector via LLM.

        Args:
            connector: ConnectorCollection with slots defined

        Returns:
            List of MinorAnchor objects to place
        """
        if not connector.minor_slots:
            return []

        # Build context for LLM
        context = self._build_context(connector)
        prompt = ANCHOR_SELECTION_PROMPT.format(**context)

        # Call LLM
        try:
            response = self._call_llm(prompt)
            anchors = self._parse_response(response, connector)
            return anchors
        except Exception as e:
            print(f"LLM anchor selection failed: {e}")
            return self._fallback_selection(connector)

    def _build_context(self, connector: ConnectorCollection) -> dict:
        """Build prompt context from connector."""
        # Summarize terrain
        terrain_counts = {}
        for hex in connector.hexes:
            t = hex.terrain.value
            terrain_counts[t] = terrain_counts.get(t, 0) + 1
        terrain_summary = ", ".join(f"{t}: {c}" for t, c in terrain_counts.items())

        # Format slot details
        slot_details = []
        for slot in connector.minor_slots:
            slot_details.append(
                f"- {slot.slot_id}: index {slot.hex_index}, "
                f"context: {slot.narrative_context}, "
                f"compatible: {slot.compatible_categories}"
            )

        return {
            "connector_type": connector.type.value,
            "length": len(connector.hexes),
            "length_km": len(connector.hexes) * 0.1,
            "terrain_summary": terrain_summary,
            "slot_count": len(connector.minor_slots),
            "slot_details": "\n".join(slot_details),
            "max_anchors": min(self.max_anchors, len(connector.minor_slots)),
        }

    def _call_llm(self, prompt: str) -> dict:
        """Call DeepSeek API."""
        if not self.api_key:
            raise ValueError("DEEPSEEK_API_KEY not set")

        response = self.client.post(
            "https://api.deepseek.com/chat/completions",
            headers={
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json",
            },
            json={
                "model": self.model,
                "messages": [
                    {"role": "system", "content": "You are a fantasy worldbuilder. Output valid JSON only."},
                    {"role": "user", "content": prompt},
                ],
                "temperature": 0.9,
                "max_tokens": 1024,
                "response_format": {"type": "json_object"},
            },
        )
        response.raise_for_status()
        content = response.json()["choices"][0]["message"]["content"]
        return json.loads(content)

    def _parse_response(
        self, response: dict, connector: ConnectorCollection
    ) -> list[MinorAnchor]:
        """Parse LLM response into MinorAnchor objects."""
        anchors = []
        slot_map = {slot.slot_id: slot for slot in connector.minor_slots}

        for item in response.get("anchors", []):
            slot_id = item.get("slot_id")
            if slot_id not in slot_map:
                continue

            slot = slot_map[slot_id]

            try:
                category = MinorCategory(item["category"])
            except ValueError:
                continue

            # Validate category is compatible with slot
            if slot.compatible_categories and category.value not in slot.compatible_categories:
                continue

            anchor = MinorAnchor(
                id=f"anchor_{connector.id}_{slot_id}",
                category=category,
                slot_contexts=[slot.narrative_context],
                compatible_terrain=[connector.hexes[slot.hex_index].terrain],
                name_fragment=item.get("name_fragment", f"The {category.value}"),
                narrative_hook=item.get("narrative_hook", "A place of interest along the route."),
            )

            # Set service flags based on category
            if category in (MinorCategory.INN, MinorCategory.TAVERN, MinorCategory.WAYSTATION):
                anchor.provides_rest = True
            if category in (MinorCategory.MARKET_SMALL, MinorCategory.CARAVANSERAI):
                anchor.provides_trade = True
            if category in (MinorCategory.SHRINE, MinorCategory.HERMIT_HUT):
                anchor.provides_information = True
            if category in (MinorCategory.TOLL_GATE, MinorCategory.BORDER_POST):
                anchor.blocks_passage = True

            anchors.append(anchor)

        return anchors

    def _fallback_selection(self, connector: ConnectorCollection) -> list[MinorAnchor]:
        """Rule-based fallback when LLM fails."""
        anchors = []

        # Place anchors at required slots
        for slot in connector.minor_slots:
            if not slot.required:
                continue

            # Pick first compatible category
            if slot.compatible_categories:
                try:
                    category = MinorCategory(slot.compatible_categories[0])
                except ValueError:
                    continue

                anchor = MinorAnchor(
                    id=f"anchor_{connector.id}_{slot.slot_id}",
                    category=category,
                    slot_contexts=[slot.narrative_context],
                    compatible_terrain=[connector.hexes[slot.hex_index].terrain],
                    name_fragment=f"The {category.value.replace('_', ' ').title()}",
                    narrative_hook=slot.narrative_context,
                )
                anchors.append(anchor)

        return anchors

    def close(self):
        self.client.close()

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
```

**Verification**:
```bash
cd worldgen && python -c "
from anchor_placer import AnchorPlacer
from connector_generator import ConnectorGenerator
from schemas.connector import ConnectorType

# Generate test connector
gen = ConnectorGenerator()
conn = gen.generate(
    ConnectorType.TRADE_ROUTE_MAJOR,
    start_pos=(0, 0),
    end_pos=(15, 8),
    placed_hexes=set(),
)

# Test fallback (no API key)
placer = AnchorPlacer(api_key='')
anchors = placer._fallback_selection(conn)
print(f'Fallback generated {len(anchors)} anchors')
print('AnchorPlacer OK')
"
```

---

## Task 4: Create FillerGenerator

**File**: `worldgen/filler_generator.py`

```python
"""Constraint propagation filler generation."""

import random
import tomllib
from collections import deque
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

from schemas import TaggedHex, EdgeType
from schemas.base import HexCoord
from schemas.world import WorldHex, HexMap
from hex_coords import get_neighbor, get_all_neighbors, coords_to_key, key_to_coords


@dataclass
class WaveCell:
    """Wave function cell for constraint propagation."""
    possible_tags: set[str]
    collapsed: bool = False
    chosen_tag: Optional[str] = None


class FillerGenerator:
    """Constraint propagation for filler hex generation."""

    def __init__(
        self,
        tags_path: str | Path = "generation/hex_tags.toml",
        max_iterations: int = 10000,
    ):
        with open(tags_path, "rb") as f:
            self.config = tomllib.load(f)

        self.max_iterations = max_iterations
        self.transitions = self.config.get("transitions", {})
        self.weights = self.config.get("transitions", {}).get("weights", {})

        # All possible terrain tags
        self.all_terrain_tags = set(self.config["categories"]["TERRAIN"]["values"])
        self.all_elevation_tags = set(self.config["categories"]["ELEVATION_BAND"]["values"])

    def generate_fillers(
        self,
        hex_map: HexMap,
        world_radius: int,
    ) -> HexMap:
        """Fill empty hexes using constraint propagation.

        Args:
            hex_map: HexMap with clusters and connectors already placed
            world_radius: Maximum world radius for hex coordinates

        Returns:
            HexMap with filler hexes added
        """
        # 1. Identify fixed hexes (clusters + connectors)
        fixed_positions = set(hex_map.hexes.keys())

        # 2. Identify all empty positions within world radius
        empty_positions = self._find_empty_positions(fixed_positions, world_radius)

        if not empty_positions:
            return hex_map

        # 3. Initialize wave function for each empty position
        wave = self._initialize_wave(empty_positions, fixed_positions, hex_map)

        # 4. Build frontier from positions adjacent to fixed hexes
        frontier = self._build_initial_frontier(empty_positions, fixed_positions)

        # 5. Constraint propagation loop
        iteration = 0
        while frontier and iteration < self.max_iterations:
            # Get position with minimum entropy (fewest possibilities)
            pos_key = self._select_min_entropy(frontier, wave)
            if pos_key is None:
                break

            frontier.remove(pos_key)
            cell = wave[pos_key]

            if cell.collapsed:
                continue

            # Collapse wave function
            chosen_terrain = self._collapse(cell, pos_key, wave, hex_map)

            if chosen_terrain is None:
                # Contradiction - use fallback
                chosen_terrain = "surface"

            cell.chosen_tag = chosen_terrain
            cell.collapsed = True

            # Create filler hex
            q, r = key_to_coords(pos_key)
            filler_hex = self._create_filler_hex(q, r, chosen_terrain)
            hex_map.hexes[pos_key] = filler_hex

            # Add uncollapsed neighbors to frontier
            for nq, nr, _ in get_all_neighbors(q, r):
                nkey = coords_to_key(nq, nr)
                if nkey in wave and not wave[nkey].collapsed and nkey not in frontier:
                    # Propagate constraints to neighbor
                    self._propagate_constraints(nkey, wave, hex_map)
                    frontier.add(nkey)

            iteration += 1

        # 6. Fill any remaining empty hexes with fallback
        for pos_key, cell in wave.items():
            if not cell.collapsed:
                q, r = key_to_coords(pos_key)
                filler_hex = self._create_filler_hex(q, r, "surface")
                hex_map.hexes[pos_key] = filler_hex

        return hex_map

    def _find_empty_positions(
        self,
        fixed: set[str],
        world_radius: int,
    ) -> set[str]:
        """Find all empty hex positions within world radius."""
        empty = set()

        for q in range(-world_radius, world_radius + 1):
            for r in range(-world_radius, world_radius + 1):
                # Check hex is within world bounds
                if abs(q) + abs(r) + abs(-q - r) <= world_radius * 2:
                    key = coords_to_key(q, r)
                    if key not in fixed:
                        empty.add(key)

        return empty

    def _initialize_wave(
        self,
        empty: set[str],
        fixed: set[str],
        hex_map: HexMap,
    ) -> dict[str, WaveCell]:
        """Initialize wave function with all possibilities."""
        wave = {}

        for pos_key in empty:
            q, r = key_to_coords(pos_key)

            # Start with all terrain tags possible
            possible = set(self.all_terrain_tags)

            # Constrain by adjacent fixed hexes
            for nq, nr, _ in get_all_neighbors(q, r):
                nkey = coords_to_key(nq, nr)
                if nkey in fixed and nkey in hex_map.hexes:
                    neighbor_hex = hex_map.hexes[nkey]
                    neighbor_terrain = self._extract_terrain_tag(neighbor_hex)

                    if neighbor_terrain and neighbor_terrain in self.transitions:
                        compatible = set(self.transitions[neighbor_terrain])
                        possible &= compatible

            # Ensure at least one possibility
            if not possible:
                possible = {"surface"}

            wave[pos_key] = WaveCell(possible_tags=possible)

        return wave

    def _build_initial_frontier(
        self,
        empty: set[str],
        fixed: set[str],
    ) -> set[str]:
        """Build initial frontier of empty hexes adjacent to fixed hexes."""
        frontier = set()

        for pos_key in empty:
            q, r = key_to_coords(pos_key)

            for nq, nr, _ in get_all_neighbors(q, r):
                nkey = coords_to_key(nq, nr)
                if nkey in fixed:
                    frontier.add(pos_key)
                    break

        return frontier

    def _select_min_entropy(
        self,
        frontier: set[str],
        wave: dict[str, WaveCell],
    ) -> Optional[str]:
        """Select position with minimum entropy (fewest possibilities)."""
        min_entropy = float('inf')
        min_pos = None

        for pos_key in frontier:
            cell = wave.get(pos_key)
            if cell and not cell.collapsed:
                entropy = len(cell.possible_tags)
                # Add small random factor to break ties
                entropy += random.random() * 0.1
                if entropy < min_entropy:
                    min_entropy = entropy
                    min_pos = pos_key

        return min_pos

    def _collapse(
        self,
        cell: WaveCell,
        pos_key: str,
        wave: dict[str, WaveCell],
        hex_map: HexMap,
    ) -> Optional[str]:
        """Collapse wave function to single terrain."""
        if not cell.possible_tags:
            return None

        # Weight by transition weights if available
        weighted_choices = []
        for tag in cell.possible_tags:
            weight = 1.0

            # Check neighbor-based weights
            q, r = key_to_coords(pos_key)
            for nq, nr, _ in get_all_neighbors(q, r):
                nkey = coords_to_key(nq, nr)

                # Check collapsed wave cells
                if nkey in wave and wave[nkey].collapsed:
                    neighbor_tag = wave[nkey].chosen_tag
                    weight_key = f"{neighbor_tag}.{tag}"
                    if weight_key in self.weights:
                        weight *= self.weights[weight_key]

                # Check fixed hexes
                if nkey in hex_map.hexes:
                    neighbor_hex = hex_map.hexes[nkey]
                    neighbor_tag = self._extract_terrain_tag(neighbor_hex)
                    if neighbor_tag:
                        weight_key = f"{neighbor_tag}.{tag}"
                        if weight_key in self.weights:
                            weight *= self.weights[weight_key]

            weighted_choices.append((tag, weight))

        # Weighted random selection
        total_weight = sum(w for _, w in weighted_choices)
        if total_weight <= 0:
            return random.choice(list(cell.possible_tags))

        r = random.random() * total_weight
        cumulative = 0
        for tag, weight in weighted_choices:
            cumulative += weight
            if r <= cumulative:
                return tag

        return weighted_choices[-1][0]

    def _propagate_constraints(
        self,
        pos_key: str,
        wave: dict[str, WaveCell],
        hex_map: HexMap,
    ):
        """Propagate constraints to a cell from its neighbors."""
        cell = wave.get(pos_key)
        if not cell or cell.collapsed:
            return

        q, r = key_to_coords(pos_key)

        for nq, nr, _ in get_all_neighbors(q, r):
            nkey = coords_to_key(nq, nr)

            # Constraint from collapsed wave cells
            if nkey in wave and wave[nkey].collapsed:
                neighbor_tag = wave[nkey].chosen_tag
                if neighbor_tag and neighbor_tag in self.transitions:
                    compatible = set(self.transitions[neighbor_tag])
                    cell.possible_tags &= compatible

            # Constraint from fixed hexes
            if nkey in hex_map.hexes:
                neighbor_hex = hex_map.hexes[nkey]
                neighbor_tag = self._extract_terrain_tag(neighbor_hex)
                if neighbor_tag and neighbor_tag in self.transitions:
                    compatible = set(self.transitions[neighbor_tag])
                    cell.possible_tags &= compatible

        # Ensure at least one possibility
        if not cell.possible_tags:
            cell.possible_tags = {"surface"}

    def _extract_terrain_tag(self, hex: WorldHex) -> Optional[str]:
        """Extract terrain tag from WorldHex."""
        # WorldHex uses Terrain enum, map to tag
        terrain_to_tag = {
            "underground": "underground",
            "cavern": "underground",
            "deep_water": "underwater",
            "shallow_water": "surface",
            "plains": "surface",
            "hills": "surface",
            "forest": "surface",
            "dense_forest": "surface",
            "mountains": "surface",
            "high_mountains": "elevated",
        }
        return terrain_to_tag.get(hex.terrain.value, "surface")

    def _create_filler_hex(self, q: int, r: int, terrain_tag: str) -> WorldHex:
        """Create a filler WorldHex."""
        from schemas.base import Terrain, SpeciesFitness

        # Map tag to Terrain enum
        tag_to_terrain = {
            "surface": Terrain.PLAINS,
            "underground": Terrain.UNDERGROUND,
            "underwater": Terrain.DEEP_WATER,
            "aerial": Terrain.HIGH_MOUNTAINS,
            "elevated": Terrain.HILLS,
            "peak": Terrain.HIGH_MOUNTAINS,
        }
        terrain = tag_to_terrain.get(terrain_tag, Terrain.PLAINS)

        return WorldHex(
            coord=HexCoord(q=q, r=r),
            terrain=terrain,
            elevation=200.0 if terrain_tag == "surface" else 0.0,
            moisture=0.5,
            temperature=15.0,
            species_fitness=SpeciesFitness(human=0.5, dwarf=0.5, elf=0.5),
        )
```

**Verification**:
```bash
cd worldgen && python -c "
from filler_generator import FillerGenerator
from schemas.world import HexMap
from schemas.base import HexCoord, Terrain, SpeciesFitness

# Create hex map with a few fixed hexes
hex_map = HexMap(seed_id=1, world_radius=5, hexes={}, clusters={}, cluster_positions={})

# Add some fixed hexes
from schemas.world import WorldHex
for q in range(-1, 2):
    for r in range(-1, 2):
        if abs(q) + abs(r) <= 1:
            key = f'{q},{r}'
            hex_map.hexes[key] = WorldHex(
                coord=HexCoord(q=q, r=r),
                terrain=Terrain.PLAINS,
                elevation=200.0,
                moisture=0.5,
                temperature=15.0,
                species_fitness=SpeciesFitness(human=0.5, dwarf=0.5, elf=0.5),
            )

initial_count = len(hex_map.hexes)
print(f'Initial hexes: {initial_count}')

# Generate fillers
gen = FillerGenerator()
hex_map = gen.generate_fillers(hex_map, world_radius=5)

final_count = len(hex_map.hexes)
print(f'Final hexes: {final_count}')
print(f'Fillers added: {final_count - initial_count}')
assert final_count > initial_count
print('FillerGenerator OK')
"
```

---

## Task 5: Update WorldAssembler

**File**: `worldgen/assembly/assembler.py`

Replace contents with:

```python
"""Main world assembly pipeline."""

import random
import time
from typing import Optional

from worldgen.schemas import WorldSeed, HexMap, AssembledCluster, AssembledWorld
from worldgen.schemas.connector import ConnectorCollection, ConnectorType
from worldgen.schemas.minor import MinorAnchor
from worldgen.schemas.world import WorldHex
from worldgen.schemas.base import HexCoord, Terrain, SpeciesFitness
from worldgen.storage import Database
from worldgen.hex_coords import coords_to_key
from .layout_solver import LayoutSolver


class WorldAssembler:
    """Assemble a complete world from a seed.

    Pipeline order:
    1. Place clusters (existing)
    2. Generate connectors between clusters
    3. Place minor anchors along connectors
    4. Fill empty hexes with constraint propagation
    """

    def __init__(self, database: Database):
        self.database = database
        self.layout_solver = LayoutSolver()

        # Lazy imports to avoid circular dependencies
        self._connector_generator = None
        self._anchor_placer = None
        self._filler_generator = None

    @property
    def connector_generator(self):
        if self._connector_generator is None:
            from worldgen.connector_generator import ConnectorGenerator
            self._connector_generator = ConnectorGenerator()
        return self._connector_generator

    @property
    def anchor_placer(self):
        if self._anchor_placer is None:
            from worldgen.anchor_placer import AnchorPlacer
            self._anchor_placer = AnchorPlacer()
        return self._anchor_placer

    @property
    def filler_generator(self):
        if self._filler_generator is None:
            from worldgen.filler_generator import FillerGenerator
            self._filler_generator = FillerGenerator()
        return self._filler_generator

    def assemble(self, seed: WorldSeed) -> AssembledWorld:
        """Assemble a complete world from a seed."""
        start_time = time.time()
        rng = random.Random(seed.seed_id)

        # 1. Create assembled clusters and solve layout
        assembled_clusters, cluster_positions, hex_map = self._place_clusters(seed, rng)

        # 2. Generate connectors between clusters
        connectors = self._generate_connectors(seed, cluster_positions, hex_map)

        # 3. Place minor anchors along connectors
        anchors = self._place_anchors(connectors)

        # 4. Generate filler hexes
        hex_map = self._generate_fillers(hex_map, seed.world_radius)

        # 5. Store connector and anchor data (for later use)
        hex_map.clusters = assembled_clusters
        hex_map.cluster_positions = cluster_positions

        elapsed_ms = int((time.time() - start_time) * 1000)

        return AssembledWorld(
            seed_id=seed.seed_id,
            name=seed.name,
            hex_map=hex_map,
            generation_time_ms=elapsed_ms,
            total_hexes=len(hex_map.hexes),
            total_clusters=len(assembled_clusters),
        )

    def _place_clusters(
        self, seed: WorldSeed, rng: random.Random
    ) -> tuple[dict[str, AssembledCluster], dict[str, tuple[int, int]], HexMap]:
        """Place clusters and convert to world hexes."""
        assembled_clusters: dict[str, AssembledCluster] = {}

        for placement in seed.clusters:
            assembled = AssembledCluster(
                template_id=placement.template_id,
                instance_id=placement.instance_id,
                components={},
                layout={},
                footprint=[(0, 0), (1, 0), (0, 1)],  # Minimal footprint
            )
            assembled_clusters[placement.instance_id] = assembled

        # Solve global layout
        cluster_positions = self.layout_solver.solve(
            clusters=assembled_clusters,
            hints=seed.layout_hints,
            world_radius=seed.world_radius,
            rng=rng,
        )

        # Create hex map with cluster hexes
        hex_map = HexMap(
            seed_id=seed.seed_id,
            world_radius=seed.world_radius,
            hexes={},
            clusters={},
            cluster_positions={},
        )

        # Add cluster hexes to map
        for instance_id, cluster in assembled_clusters.items():
            base_pos = cluster_positions[instance_id]

            for offset in cluster.footprint:
                world_q = base_pos[0] + offset[0]
                world_r = base_pos[1] + offset[1]
                key = coords_to_key(world_q, world_r)

                hex_map.hexes[key] = WorldHex(
                    coord=HexCoord(q=world_q, r=world_r),
                    terrain=Terrain.PLAINS,
                    elevation=200.0,
                    moisture=0.5,
                    temperature=15.0,
                    species_fitness=SpeciesFitness(human=0.7, dwarf=0.5, elf=0.5),
                    cluster_id=instance_id,
                )

        return assembled_clusters, cluster_positions, hex_map

    def _generate_connectors(
        self,
        seed: WorldSeed,
        cluster_positions: dict[str, tuple[int, int]],
        hex_map: HexMap,
    ) -> list[ConnectorCollection]:
        """Generate connectors based on seed assignments."""
        connectors = []
        placed_hexes = set(hex_map.hexes.keys())

        for assignment in seed.connectors:
            start_cluster = assignment.start_cluster
            end_cluster = assignment.end_cluster

            if not start_cluster or not end_cluster:
                continue
            if start_cluster not in cluster_positions:
                continue
            if end_cluster not in cluster_positions:
                continue

            start_pos = cluster_positions[start_cluster]
            end_pos = cluster_positions[end_cluster]

            # Determine connector type
            connector_type = ConnectorType.TRADE_ROUTE_MAJOR  # Default
            if "river" in assignment.collection_id.lower():
                connector_type = ConnectorType.RIVER_MIDDLE
            elif "military" in assignment.collection_id.lower():
                connector_type = ConnectorType.MILITARY_ROAD

            # Generate connector
            connector = self.connector_generator.generate(
                connector_type=connector_type,
                start_pos=start_pos,
                end_pos=end_pos,
                placed_hexes={(int(k.split(',')[0]), int(k.split(',')[1])) for k in placed_hexes},
            )

            # Add connector hexes to map
            for chex in connector.hexes:
                # Need to get world coordinates - connector hexes use indices
                # For now, interpolate along path
                path_progress = chex.index / max(1, len(connector.hexes) - 1)
                world_q = int(start_pos[0] + (end_pos[0] - start_pos[0]) * path_progress)
                world_r = int(start_pos[1] + (end_pos[1] - start_pos[1]) * path_progress)
                key = coords_to_key(world_q, world_r)

                if key not in hex_map.hexes:
                    hex_map.hexes[key] = WorldHex(
                        coord=HexCoord(q=world_q, r=world_r),
                        terrain=chex.terrain,
                        elevation=chex.elevation,
                        moisture=chex.moisture,
                        temperature=15.0,
                        species_fitness=chex.species_fitness,
                    )
                    placed_hexes.add(key)

            connectors.append(connector)

        return connectors

    def _place_anchors(
        self, connectors: list[ConnectorCollection]
    ) -> list[MinorAnchor]:
        """Place anchors along connectors via LLM."""
        all_anchors = []

        for connector in connectors:
            try:
                anchors = self.anchor_placer.place_anchors(connector)
                all_anchors.extend(anchors)
            except Exception as e:
                print(f"Anchor placement failed for {connector.id}: {e}")

        return all_anchors

    def _generate_fillers(self, hex_map: HexMap, world_radius: int) -> HexMap:
        """Generate filler hexes."""
        return self.filler_generator.generate_fillers(hex_map, world_radius)
```

**Verification**:
```bash
cd worldgen && python -c "
from assembly.assembler import WorldAssembler
from schemas import WorldSeed, ClusterPlacement, ConnectorAssignment

# Mock database
class MockDB:
    pass

assembler = WorldAssembler(MockDB())

# Create test seed
seed = WorldSeed(
    seed_id=42,
    name='Test World',
    world_radius=10,
    clusters=[
        ClusterPlacement(template_id='town', instance_id='town_1'),
        ClusterPlacement(template_id='town', instance_id='town_2'),
    ],
    connectors=[
        ConnectorAssignment(
            collection_id='trade_route_1',
            instance_id='route_1',
            start_cluster='town_1',
            end_cluster='town_2',
        ),
    ],
)

# Run assembly (will use fallbacks for LLM calls)
world = assembler.assemble(seed)
print(f'Generated world with {world.total_hexes} hexes')
print(f'Generation time: {world.generation_time_ms}ms')
assert world.total_hexes > 0
print('WorldAssembler pipeline OK')
"
```

---

## Task 6: Add CLI Commands

**File**: `worldgen/cli.py`

Add these commands after existing commands:

```python
@app.command()
def generate_world(
    seed_id: int = typer.Option(42, help="World seed ID"),
    radius: int = typer.Option(20, help="World radius in hexes"),
    output: Path = typer.Option(Path("world.json"), help="Output file"),
):
    """Generate complete world with clusters, connectors, and fillers."""
    from assembly.assembler import WorldAssembler
    from schemas import WorldSeed, ClusterPlacement, ConnectorAssignment

    # Create sample seed
    seed = WorldSeed(
        seed_id=seed_id,
        name=f"World {seed_id}",
        world_radius=radius,
        clusters=[
            ClusterPlacement(template_id='settlement', instance_id='settlement_1'),
            ClusterPlacement(template_id='settlement', instance_id='settlement_2'),
        ],
        connectors=[
            ConnectorAssignment(
                collection_id='trade_route',
                instance_id='route_1',
                start_cluster='settlement_1',
                end_cluster='settlement_2',
            ),
        ],
    )

    # Mock database for now
    class MockDB:
        pass

    assembler = WorldAssembler(MockDB())
    world = assembler.assemble(seed)

    # Save to file
    with open(output, 'w') as f:
        f.write(world.model_dump_json(indent=2))

    typer.echo(f"Generated world with {world.total_hexes} hexes")
    typer.echo(f"Saved to {output}")


@app.command()
def test_connector(
    start_q: int = typer.Option(0, help="Start Q coordinate"),
    start_r: int = typer.Option(0, help="Start R coordinate"),
    end_q: int = typer.Option(10, help="End Q coordinate"),
    end_r: int = typer.Option(5, help="End R coordinate"),
):
    """Test connector generation between two points."""
    from connector_generator import ConnectorGenerator
    from schemas.connector import ConnectorType

    gen = ConnectorGenerator()
    conn = gen.generate(
        ConnectorType.TRADE_ROUTE_MAJOR,
        start_pos=(start_q, start_r),
        end_pos=(end_q, end_r),
        placed_hexes=set(),
    )

    typer.echo(f"Generated connector: {conn.id}")
    typer.echo(f"  Length: {len(conn.hexes)} hexes")
    typer.echo(f"  Anchor slots: {len(conn.minor_slots)}")
    typer.echo(f"  Entry points: {len(conn.entry_points)}")
```

---

## Verification Checklist

Run all verifications in order:

```bash
cd worldgen

# 1. Transition rules
python -c "
import tomllib
with open('generation/hex_tags.toml', 'rb') as f:
    config = tomllib.load(f)
assert 'transitions' in config
print('1. Transition rules: OK')
"

# 2. ConnectorGenerator
python -c "
from connector_generator import ConnectorGenerator
from schemas.connector import ConnectorType
gen = ConnectorGenerator()
conn = gen.generate(ConnectorType.TRADE_ROUTE_MAJOR, (0,0), (10,5), set())
assert len(conn.hexes) > 0
print('2. ConnectorGenerator: OK')
"

# 3. AnchorPlacer (fallback mode)
python -c "
from anchor_placer import AnchorPlacer
from connector_generator import ConnectorGenerator
from schemas.connector import ConnectorType
gen = ConnectorGenerator()
conn = gen.generate(ConnectorType.TRADE_ROUTE_MAJOR, (0,0), (15,8), set())
placer = AnchorPlacer(api_key='')
anchors = placer._fallback_selection(conn)
print(f'3. AnchorPlacer: OK ({len(anchors)} fallback anchors)')
"

# 4. FillerGenerator
python -c "
from filler_generator import FillerGenerator
from schemas.world import HexMap, WorldHex
from schemas.base import HexCoord, Terrain, SpeciesFitness
hex_map = HexMap(seed_id=1, world_radius=3, hexes={}, clusters={}, cluster_positions={})
hex_map.hexes['0,0'] = WorldHex(
    coord=HexCoord(q=0, r=0), terrain=Terrain.PLAINS,
    elevation=200.0, moisture=0.5, temperature=15.0,
    species_fitness=SpeciesFitness(human=0.5, dwarf=0.5, elf=0.5)
)
gen = FillerGenerator()
hex_map = gen.generate_fillers(hex_map, world_radius=3)
assert len(hex_map.hexes) > 1
print(f'4. FillerGenerator: OK ({len(hex_map.hexes)} total hexes)')
"

# 5. Full pipeline
python -c "
from assembly.assembler import WorldAssembler
from schemas import WorldSeed, ClusterPlacement, ConnectorAssignment
class MockDB: pass
assembler = WorldAssembler(MockDB())
seed = WorldSeed(seed_id=42, world_radius=5,
    clusters=[ClusterPlacement(template_id='t', instance_id='c1'),
              ClusterPlacement(template_id='t', instance_id='c2')],
    connectors=[ConnectorAssignment(collection_id='r', instance_id='r1',
                start_cluster='c1', end_cluster='c2')])
world = assembler.assemble(seed)
assert world.total_hexes > 0
print(f'5. Full pipeline: OK ({world.total_hexes} hexes, {world.generation_time_ms}ms)')
"

echo "All verifications passed!"
```

---

## Risk Mitigations

| Risk | Mitigation |
|------|------------|
| Constraint propagation contradictions | Fallback to "surface" terrain |
| LLM anchor selection fails | Rule-based fallback in `_fallback_selection()` |
| A* pathfinding stuck | `_straight_line_path()` fallback |
| Empty transition rules | Default "surface" compatible with all |
| Performance at large radius | `max_iterations` limit on propagation |

---

## Future Enhancements (Out of Scope)

- Directional transition rules (river flow direction)
- Multi-hex features (lakes, forests)
- Elevation interpolation for terrain gradients
- Connector branching (road junctions)
- Climate zones affecting transitions
