# Hex Scale Anchoring and Tag-Based Composition System

**Created:** 2026-01-02
**Status:** Ready for implementation
**Confidence:** 95%

## Overview

This plan implements:
1. **100m scale anchoring** - Every hex is exactly 100m across with LLM scoring verification
2. **Tag taxonomy** - 5-category tag system replacing terrain enums
3. **Edge-based composition** - 6-edge connectivity with 3 configurable modes
4. **Adjacency validation** - Hard/soft constraint tiers
5. **20-hex cluster test** - Connected region generation with validation

## Architecture Summary

```
worldgen/
├── hex_generator.py      # MODIFY - add scale prompts, tag output
├── schemas.py            # MODIFY - add TaggedHex, EdgeType
├── generation/
│   ├── hex_tags.toml     # NEW - tag taxonomy
│   └── context_constraints.toml  # MODIFY - tag-based constraints
├── hex_coords.py         # NEW - axial coordinate utilities
├── adjacency.py          # NEW - adjacency validation engine
├── edge_handler.py       # NEW - 3 edge handling modes
├── scale_validator.py    # NEW - 100m scale scoring
├── cluster_generator.py  # NEW - connected cluster generation
└── tests/
    └── test_cluster_20.py  # NEW - 20-hex integration test
```

---

## Task 1: Create hex_tags.toml

**File:** `/home/astre/arc-citadel/worldgen/generation/hex_tags.toml`

### Test First (manual validation)

```bash
cd /home/astre/arc-citadel/worldgen
python -c "import tomllib; print(tomllib.load(open('generation/hex_tags.toml', 'rb')))"
```

**Expected:** Parses without error, shows all 5 categories

### Implementation

```toml
# Hex Tag Taxonomy v1.0
# Tags replace terrain enums as source of truth for hex composition

[meta]
version = "1.0"
scale = "100m"
description = "Every hex is 100m across - roughly what a party explores in 10-15 minutes"

[categories.TERRAIN]
description = "Physical environment - MUTUALLY EXCLUSIVE"
exclusive = true
values = ["underground", "surface", "underwater", "aerial"]

[categories.CULTURE]
description = "Dominant cultural influence"
exclusive = false
values = ["dwarf", "elf", "human", "wild", "ancient", "corrupted"]

[categories.FUNCTION]
description = "Primary purpose of location"
exclusive = false
values = ["residential", "military", "sacred", "industrial", "commercial", "passage"]

[categories.EDGES]
description = "Edge connection capabilities"
exclusive = false
values = ["connects_road", "connects_tunnel", "connects_water", "connects_wilderness", "entrance", "dead_end"]

[categories.ELEVATION_BAND]
description = "Vertical positioning - MUTUALLY EXCLUSIVE"
exclusive = true
values = ["deep", "shallow_under", "surface", "elevated", "peak"]

# Hard constraints - violations block placement
[constraints.hard]
# TERRAIN adjacency rules
incompatible_terrain = [
    ["underground", "aerial"],
    ["underwater", "aerial"],
    ["underwater", "peak"],
]

# EDGE matching rules
edge_must_match = [
    "connects_tunnel",
    "connects_road",
    "connects_water",
]

# Required pairings
requires = [
    {tag = "underground", requires_one_of = ["connects_tunnel", "entrance", "dead_end"]},
    {tag = "underwater", requires_one_of = ["connects_water", "dead_end"]},
    {tag = "deep", requires = "underground"},
    {tag = "shallow_under", requires = "underground"},
    {tag = "peak", requires_one_of = ["surface", "aerial"]},
]

# Soft constraints - generate warnings but allow
[constraints.soft]
# Culture adjacency warnings
culture_tensions = [
    ["dwarf", "elf"],
    ["corrupted", "sacred"],
]

# Function adjacency warnings
function_clashes = [
    ["industrial", "sacred"],
    ["military", "commercial"],
]
```

### Verify

```bash
python -c "
import tomllib
data = tomllib.load(open('generation/hex_tags.toml', 'rb'))
assert len(data['categories']) == 5
assert 'underground' in data['categories']['TERRAIN']['values']
print('hex_tags.toml: VALID')
"
```

---

## Task 2: Add Pydantic schemas for tags and edges

**File:** `/home/astre/arc-citadel/worldgen/schemas.py`

### Test First

Create test file first:

**File:** `/home/astre/arc-citadel/worldgen/tests/test_schemas.py`

```python
"""Tests for tag-based hex schemas."""
import pytest
from pydantic import ValidationError
from schemas import EdgeType, TaggedHex, HexCluster


class TestEdgeType:
    def test_valid_edge_types(self):
        assert EdgeType.TUNNEL == "tunnel"
        assert EdgeType.BLOCKED == "blocked"

    def test_all_edge_types_exist(self):
        expected = {"tunnel", "road", "water", "wilderness", "entrance", "blocked"}
        actual = {e.value for e in EdgeType}
        assert expected == actual


class TestTaggedHex:
    def test_valid_hex(self):
        hex = TaggedHex(
            q=0, r=0,
            name="Test Cavern",
            description="A small underground chamber",
            tags=["underground", "dwarf", "industrial"],
            edge_types=["tunnel", "tunnel", "blocked", "blocked", "blocked", "blocked"],
        )
        assert hex.q == 0
        assert len(hex.edge_types) == 6

    def test_requires_6_edges(self):
        with pytest.raises(ValidationError):
            TaggedHex(
                q=0, r=0,
                name="Bad Hex",
                description="Missing edges",
                tags=["surface"],
                edge_types=["road", "road"],  # Only 2 edges
            )

    def test_requires_at_least_one_tag(self):
        with pytest.raises(ValidationError):
            TaggedHex(
                q=0, r=0,
                name="No Tags",
                description="Empty tags",
                tags=[],
                edge_types=["blocked"] * 6,
            )

    def test_invalid_edge_type_rejected(self):
        with pytest.raises(ValidationError):
            TaggedHex(
                q=0, r=0,
                name="Bad Edge",
                description="Invalid edge type",
                tags=["surface"],
                edge_types=["tunnel", "road", "INVALID", "blocked", "blocked", "blocked"],
            )


class TestHexCluster:
    def test_cluster_requires_20_hexes(self):
        hexes = [
            TaggedHex(
                q=i, r=0,
                name=f"Hex {i}",
                description="Test hex",
                tags=["surface", "wild"],
                edge_types=["wilderness"] * 6,
            )
            for i in range(20)
        ]
        cluster = HexCluster(hexes=hexes)
        assert len(cluster.hexes) == 20

    def test_cluster_rejects_fewer_than_20(self):
        hexes = [
            TaggedHex(
                q=i, r=0,
                name=f"Hex {i}",
                description="Test hex",
                tags=["surface"],
                edge_types=["wilderness"] * 6,
            )
            for i in range(10)  # Only 10
        ]
        with pytest.raises(ValidationError):
            HexCluster(hexes=hexes)
```

### Run Test (expect FAIL)

```bash
cd /home/astre/arc-citadel/worldgen
python -m pytest tests/test_schemas.py -v
```

**Expected:** `ModuleNotFoundError` or `ImportError` for `EdgeType`, `TaggedHex`, `HexCluster`

### Implementation

Add to `/home/astre/arc-citadel/worldgen/schemas.py`:

```python
# Add after existing imports
from typing import Literal

# Add after FeatureType enum (around line 46)

class EdgeType(str, Enum):
    """Edge connection types for hex borders."""
    TUNNEL = "tunnel"
    ROAD = "road"
    WATER = "water"
    WILDERNESS = "wilderness"
    ENTRANCE = "entrance"
    BLOCKED = "blocked"


class TaggedHex(BaseModel):
    """A hex with tag-based composition (100m scale)."""
    q: int = Field(description="Axial coordinate q")
    r: int = Field(description="Axial coordinate r")
    name: str = Field(max_length=100, description="Location name")
    description: str = Field(max_length=500, description="100m-scale description")
    tags: list[str] = Field(min_length=1, max_length=10, description="Tags from hex_tags.toml")
    edge_types: list[EdgeType] = Field(min_length=6, max_length=6, description="6 edges clockwise from E")
    scale_score: Optional[float] = Field(default=None, ge=0.0, le=10.0, description="100m scale validation score")

    @field_validator('edge_types')
    @classmethod
    def validate_edge_count(cls, v: list[EdgeType]) -> list[EdgeType]:
        if len(v) != 6:
            raise ValueError(f"Must have exactly 6 edges, got {len(v)}")
        return v


class HexCluster(BaseModel):
    """A connected cluster of 20 hexes for testing."""
    hexes: list[TaggedHex] = Field(min_length=20, max_length=20)
    adjacencies: list[tuple[int, int, int]] = Field(
        default_factory=list,
        description="List of (hex_a_idx, hex_b_idx, edge_from_a) connections"
    )

    @property
    def hex_count(self) -> int:
        return len(self.hexes)

    def get_hex_at(self, q: int, r: int) -> Optional[TaggedHex]:
        """Get hex at coordinates."""
        for h in self.hexes:
            if h.q == q and h.r == r:
                return h
        return None


class ScaleValidation(BaseModel):
    """Result of 100m scale validation."""
    hex_index: int
    score: float = Field(ge=0.0, le=10.0)
    passes: bool
    feedback: Optional[str] = None
```

### Verify

```bash
cd /home/astre/arc-citadel/worldgen
python -m pytest tests/test_schemas.py -v
```

**Expected:** All tests pass

---

## Task 3: Create hex coordinate utilities

**File:** `/home/astre/arc-citadel/worldgen/hex_coords.py`

### Test First

**File:** `/home/astre/arc-citadel/worldgen/tests/test_hex_coords.py`

```python
"""Tests for hex coordinate utilities."""
import pytest
from hex_coords import (
    HEX_DIRECTIONS,
    get_neighbor,
    get_opposite_edge,
    get_all_neighbors,
    distance,
)


class TestHexDirections:
    def test_six_directions(self):
        assert len(HEX_DIRECTIONS) == 6

    def test_direction_names(self):
        expected = {"E", "NE", "NW", "W", "SW", "SE"}
        assert set(HEX_DIRECTIONS.keys()) == expected


class TestGetNeighbor:
    def test_east_neighbor(self):
        q, r = get_neighbor(0, 0, 0)  # Edge 0 = East
        assert (q, r) == (1, 0)

    def test_west_neighbor(self):
        q, r = get_neighbor(0, 0, 3)  # Edge 3 = West
        assert (q, r) == (-1, 0)

    def test_all_neighbors_from_origin(self):
        neighbors = [get_neighbor(0, 0, edge) for edge in range(6)]
        expected = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)]
        assert neighbors == expected


class TestGetOppositeEdge:
    def test_opposite_of_east_is_west(self):
        assert get_opposite_edge(0) == 3  # E -> W

    def test_opposite_of_northeast_is_southwest(self):
        assert get_opposite_edge(1) == 4  # NE -> SW

    def test_all_opposites(self):
        for edge in range(6):
            opposite = get_opposite_edge(edge)
            assert get_opposite_edge(opposite) == edge  # Double opposite = original


class TestDistance:
    def test_same_hex_distance_zero(self):
        assert distance(0, 0, 0, 0) == 0

    def test_adjacent_hex_distance_one(self):
        assert distance(0, 0, 1, 0) == 1
        assert distance(0, 0, 0, 1) == 1

    def test_diagonal_distance(self):
        assert distance(0, 0, 2, -1) == 2
```

### Run Test (expect FAIL)

```bash
cd /home/astre/arc-citadel/worldgen
python -m pytest tests/test_hex_coords.py -v
```

### Implementation

**File:** `/home/astre/arc-citadel/worldgen/hex_coords.py`

```python
"""Axial hex coordinate utilities.

Hex edge numbering (clockwise from East):
    Edge 0: E   (+1,  0)
    Edge 1: NE  (+1, -1)
    Edge 2: NW  ( 0, -1)
    Edge 3: W   (-1,  0)
    Edge 4: SW  (-1, +1)
    Edge 5: SE  ( 0, +1)
"""

from typing import NamedTuple


class HexOffset(NamedTuple):
    """Offset for hex neighbor lookup."""
    dq: int
    dr: int


# Neighbor offsets indexed by edge number (clockwise from E)
HEX_NEIGHBOR_OFFSETS: list[HexOffset] = [
    HexOffset(+1,  0),  # Edge 0: E
    HexOffset(+1, -1),  # Edge 1: NE
    HexOffset( 0, -1),  # Edge 2: NW
    HexOffset(-1,  0),  # Edge 3: W
    HexOffset(-1, +1),  # Edge 4: SW
    HexOffset( 0, +1),  # Edge 5: SE
]

# Direction names for readability
HEX_DIRECTIONS: dict[str, int] = {
    "E": 0,
    "NE": 1,
    "NW": 2,
    "W": 3,
    "SW": 4,
    "SE": 5,
}


def get_neighbor(q: int, r: int, edge: int) -> tuple[int, int]:
    """Get coordinates of neighbor connected by given edge.

    Args:
        q: Axial q coordinate
        r: Axial r coordinate
        edge: Edge index 0-5 (clockwise from E)

    Returns:
        (q, r) of neighbor hex
    """
    offset = HEX_NEIGHBOR_OFFSETS[edge]
    return (q + offset.dq, r + offset.dr)


def get_opposite_edge(edge: int) -> int:
    """Get edge index on opposite side of hex.

    Edge 0 (E) opposite is Edge 3 (W), etc.
    """
    return (edge + 3) % 6


def get_all_neighbors(q: int, r: int) -> list[tuple[int, int, int]]:
    """Get all 6 neighbors with their connecting edge.

    Returns:
        List of (neighbor_q, neighbor_r, edge_from_center)
    """
    return [
        (q + offset.dq, r + offset.dr, edge)
        for edge, offset in enumerate(HEX_NEIGHBOR_OFFSETS)
    ]


def distance(q1: int, r1: int, q2: int, r2: int) -> int:
    """Calculate hex distance between two coordinates.

    Uses axial coordinate distance formula.
    """
    return (abs(q1 - q2) + abs(q1 + r1 - q2 - r2) + abs(r1 - r2)) // 2


def coords_to_key(q: int, r: int) -> str:
    """Convert coordinates to string key for dict lookups."""
    return f"{q},{r}"


def key_to_coords(key: str) -> tuple[int, int]:
    """Convert string key back to coordinates."""
    q, r = key.split(",")
    return (int(q), int(r))
```

### Verify

```bash
cd /home/astre/arc-citadel/worldgen
python -m pytest tests/test_hex_coords.py -v
```

---

## Task 4: Create adjacency validator

**File:** `/home/astre/arc-citadel/worldgen/adjacency.py`

### Test First

**File:** `/home/astre/arc-citadel/worldgen/tests/test_adjacency.py`

```python
"""Tests for adjacency validation."""
import pytest
from schemas import TaggedHex, EdgeType
from adjacency import AdjacencyValidator, ValidationResult


@pytest.fixture
def validator():
    return AdjacencyValidator("generation/hex_tags.toml")


@pytest.fixture
def underground_hex():
    return TaggedHex(
        q=0, r=0,
        name="Dwarf Mine",
        description="Underground mining chamber",
        tags=["underground", "dwarf", "industrial"],
        edge_types=["tunnel", "tunnel", "blocked", "blocked", "blocked", "blocked"],
    )


@pytest.fixture
def surface_hex():
    return TaggedHex(
        q=1, r=0,
        name="Forest Clearing",
        description="Surface clearing",
        tags=["surface", "wild"],
        edge_types=["wilderness", "wilderness", "wilderness", "wilderness", "wilderness", "wilderness"],
    )


@pytest.fixture
def aerial_hex():
    return TaggedHex(
        q=0, r=-1,
        name="Cloud Platform",
        description="Floating platform",
        tags=["aerial", "ancient"],
        edge_types=["blocked", "blocked", "blocked", "blocked", "blocked", "blocked"],
    )


class TestHardConstraints:
    def test_underground_cannot_adjoin_aerial(self, validator, underground_hex, aerial_hex):
        """Underground hex cannot be adjacent to aerial hex."""
        result = validator.validate_adjacency(underground_hex, aerial_hex, edge=1)
        assert not result.valid
        assert "TERRAIN" in result.errors[0]

    def test_edge_types_must_match(self, validator, underground_hex):
        """Tunnel edge must connect to tunnel edge."""
        mismatched = TaggedHex(
            q=1, r=0,
            name="Bad Neighbor",
            description="Has road where tunnel expected",
            tags=["underground", "dwarf"],
            edge_types=["road", "tunnel", "blocked", "blocked", "blocked", "blocked"],
        )
        result = validator.validate_adjacency(underground_hex, mismatched, edge=0)
        assert not result.valid
        assert "edge" in result.errors[0].lower()


class TestSoftConstraints:
    def test_dwarf_elf_adjacency_warns(self, validator):
        """Dwarf adjacent to elf generates warning, not error."""
        dwarf_hex = TaggedHex(
            q=0, r=0,
            name="Dwarf Hold",
            description="Dwarven area",
            tags=["underground", "dwarf", "residential"],
            edge_types=["tunnel"] * 6,
        )
        elf_hex = TaggedHex(
            q=1, r=0,
            name="Elf Outpost",
            description="Elven area",
            tags=["underground", "elf", "residential"],
            edge_types=["tunnel"] * 6,
        )
        result = validator.validate_adjacency(dwarf_hex, elf_hex, edge=0)
        assert result.valid  # Soft constraint doesn't block
        assert len(result.warnings) > 0
        assert "culture" in result.warnings[0].lower()


class TestValidAdjacencies:
    def test_matching_tunnels_valid(self, validator):
        """Two underground hexes with matching tunnel edges are valid."""
        hex_a = TaggedHex(
            q=0, r=0,
            name="Tunnel A",
            description="Tunnel section",
            tags=["underground", "passage"],
            edge_types=["tunnel", "blocked", "blocked", "blocked", "blocked", "blocked"],
        )
        hex_b = TaggedHex(
            q=1, r=0,
            name="Tunnel B",
            description="Tunnel section",
            tags=["underground", "passage"],
            edge_types=["blocked", "blocked", "blocked", "tunnel", "blocked", "blocked"],
        )
        result = validator.validate_adjacency(hex_a, hex_b, edge=0)
        assert result.valid
        assert len(result.errors) == 0
```

### Run Test (expect FAIL)

```bash
cd /home/astre/arc-citadel/worldgen
python -m pytest tests/test_adjacency.py -v
```

### Implementation

**File:** `/home/astre/arc-citadel/worldgen/adjacency.py`

```python
"""Adjacency validation for tag-based hex composition."""

import tomllib
from dataclasses import dataclass, field
from pathlib import Path

from schemas import TaggedHex, EdgeType
from hex_coords import get_opposite_edge


@dataclass
class ValidationResult:
    """Result of adjacency validation."""
    valid: bool
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)


class AdjacencyValidator:
    """Validates hex adjacencies using tag constraints."""

    # Edge types that must match between adjacent hexes
    MATCHING_EDGES = {EdgeType.TUNNEL, EdgeType.ROAD, EdgeType.WATER}

    def __init__(self, tags_path: str | Path):
        """Load constraints from hex_tags.toml."""
        with open(tags_path, "rb") as f:
            self.config = tomllib.load(f)

        self.hard_constraints = self.config.get("constraints", {}).get("hard", {})
        self.soft_constraints = self.config.get("constraints", {}).get("soft", {})

        # Build lookup sets for faster validation
        self.incompatible_terrain = {
            frozenset(pair) for pair in self.hard_constraints.get("incompatible_terrain", [])
        }
        self.culture_tensions = {
            frozenset(pair) for pair in self.soft_constraints.get("culture_tensions", [])
        }
        self.function_clashes = {
            frozenset(pair) for pair in self.soft_constraints.get("function_clashes", [])
        }

    def validate_adjacency(
        self,
        hex_a: TaggedHex,
        hex_b: TaggedHex,
        edge: int,
    ) -> ValidationResult:
        """Validate that two hexes can be adjacent.

        Args:
            hex_a: First hex
            hex_b: Second hex (adjacent to hex_a)
            edge: Edge index from hex_a to hex_b (0-5)

        Returns:
            ValidationResult with valid flag, errors, and warnings
        """
        errors: list[str] = []
        warnings: list[str] = []

        # 1. HARD: Check terrain compatibility
        terrain_a = self._get_terrain_tag(hex_a.tags)
        terrain_b = self._get_terrain_tag(hex_b.tags)

        if terrain_a and terrain_b:
            pair = frozenset([terrain_a, terrain_b])
            if pair in self.incompatible_terrain:
                errors.append(f"TERRAIN incompatible: {terrain_a} cannot adjoin {terrain_b}")

        # 2. HARD: Check edge type matching
        edge_from_a = hex_a.edge_types[edge]
        opposite_edge = get_opposite_edge(edge)
        edge_from_b = hex_b.edge_types[opposite_edge]

        if edge_from_a in self.MATCHING_EDGES or edge_from_b in self.MATCHING_EDGES:
            if edge_from_a != edge_from_b:
                errors.append(
                    f"Edge mismatch: hex_a edge {edge}={edge_from_a.value}, "
                    f"hex_b edge {opposite_edge}={edge_from_b.value}"
                )

        # 3. SOFT: Check culture tensions
        cultures_a = self._get_culture_tags(hex_a.tags)
        cultures_b = self._get_culture_tags(hex_b.tags)

        for ca in cultures_a:
            for cb in cultures_b:
                pair = frozenset([ca, cb])
                if pair in self.culture_tensions:
                    warnings.append(f"Culture tension: {ca} adjacent to {cb}")

        # 4. SOFT: Check function clashes
        functions_a = self._get_function_tags(hex_a.tags)
        functions_b = self._get_function_tags(hex_b.tags)

        for fa in functions_a:
            for fb in functions_b:
                pair = frozenset([fa, fb])
                if pair in self.function_clashes:
                    warnings.append(f"Function clash: {fa} adjacent to {fb}")

        return ValidationResult(
            valid=len(errors) == 0,
            errors=errors,
            warnings=warnings,
        )

    def _get_terrain_tag(self, tags: list[str]) -> str | None:
        """Extract TERRAIN tag from list."""
        terrain_values = set(self.config["categories"]["TERRAIN"]["values"])
        for tag in tags:
            if tag in terrain_values:
                return tag
        return None

    def _get_culture_tags(self, tags: list[str]) -> list[str]:
        """Extract CULTURE tags from list."""
        culture_values = set(self.config["categories"]["CULTURE"]["values"])
        return [tag for tag in tags if tag in culture_values]

    def _get_function_tags(self, tags: list[str]) -> list[str]:
        """Extract FUNCTION tags from list."""
        function_values = set(self.config["categories"]["FUNCTION"]["values"])
        return [tag for tag in tags if tag in function_values]

    def validate_hex_internal(self, hex: TaggedHex) -> ValidationResult:
        """Validate a hex's internal tag consistency.

        Checks required pairings (e.g., 'deep' requires 'underground').
        """
        errors: list[str] = []
        warnings: list[str] = []

        tag_set = set(hex.tags)

        for requirement in self.hard_constraints.get("requires", []):
            tag = requirement["tag"]
            if tag not in tag_set:
                continue

            if "requires" in requirement:
                required = requirement["requires"]
                if required not in tag_set:
                    errors.append(f"Tag '{tag}' requires '{required}'")

            if "requires_one_of" in requirement:
                options = requirement["requires_one_of"]
                if not any(opt in tag_set for opt in options):
                    errors.append(f"Tag '{tag}' requires one of {options}")

        return ValidationResult(
            valid=len(errors) == 0,
            errors=errors,
            warnings=warnings,
        )
```

### Verify

```bash
cd /home/astre/arc-citadel/worldgen
python -m pytest tests/test_adjacency.py -v
```

---

## Task 5: Create scale validator

**File:** `/home/astre/arc-citadel/worldgen/scale_validator.py`

### Test First

**File:** `/home/astre/arc-citadel/worldgen/tests/test_scale_validator.py`

```python
"""Tests for 100m scale validation."""
import pytest
from unittest.mock import Mock, patch
from schemas import TaggedHex, ScaleValidation
from scale_validator import ScaleValidator


@pytest.fixture
def validator():
    return ScaleValidator(threshold=7.0)


class TestScaleValidation:
    def test_good_scale_description_passes(self, validator):
        """Description fitting 100m scale should pass."""
        hex = TaggedHex(
            q=0, r=0,
            name="Forest Grove",
            description="A small clearing surrounded by ancient oaks. A moss-covered boulder sits at the center, with a spring bubbling nearby.",
            tags=["surface", "wild"],
            edge_types=["wilderness"] * 6,
        )

        with patch.object(validator, '_call_llm') as mock_llm:
            mock_llm.return_value = {"score": 9, "feedback": "Good 100m scale"}
            result = validator.validate(hex)

        assert result.passes
        assert result.score >= 7.0

    def test_too_large_description_fails(self, validator):
        """Description suggesting region-scale should fail."""
        hex = TaggedHex(
            q=0, r=0,
            name="The Great Forest",
            description="A vast forest stretching for miles, home to countless creatures and ancient secrets lost to time.",
            tags=["surface", "wild"],
            edge_types=["wilderness"] * 6,
        )

        with patch.object(validator, '_call_llm') as mock_llm:
            mock_llm.return_value = {"score": 3, "feedback": "Too large - describes miles"}
            result = validator.validate(hex)

        assert not result.passes
        assert result.score < 7.0

    def test_too_small_description_fails(self, validator):
        """Description suggesting room-scale should fail."""
        hex = TaggedHex(
            q=0, r=0,
            name="Storage Closet",
            description="A small closet with a few shelves holding dusty bottles.",
            tags=["underground", "dwarf", "residential"],
            edge_types=["tunnel", "blocked", "blocked", "blocked", "blocked", "blocked"],
        )

        with patch.object(validator, '_call_llm') as mock_llm:
            mock_llm.return_value = {"score": 2, "feedback": "Too small - a closet is not 100m"}
            result = validator.validate(hex)

        assert not result.passes


class TestScalePrompt:
    def test_prompt_includes_scale_context(self, validator):
        """Validation prompt should mention 100m scale."""
        hex = TaggedHex(
            q=0, r=0,
            name="Test",
            description="Test description",
            tags=["surface"],
            edge_types=["wilderness"] * 6,
        )

        prompt = validator._build_prompt(hex)
        assert "100m" in prompt or "100 meter" in prompt.lower()
        assert "10-15 minutes" in prompt
```

### Implementation

**File:** `/home/astre/arc-citadel/worldgen/scale_validator.py`

```python
"""100m scale validation for hex descriptions."""

import json
import os
from typing import Optional

import httpx

from schemas import TaggedHex, ScaleValidation


SCALE_VALIDATION_PROMPT = """Analyze this hex description for 100-meter scale consistency.

SCALE RULE: Each hex represents approximately 100m × 100m (about 1 hectare).
- APPROPRIATE (100m scale): A grove of trees, a hilltop with ruins, a market square, a mine entrance area, a small lake
- TOO LARGE (region scale): "vast forest", "mountain range", "sprawling city", anything described as "miles" or "leagues"
- TOO SMALL (room scale): "a closet", "a single room", "a small chamber", anything that fits in a building

REFERENCE: A party of adventurers should be able to thoroughly explore this hex in 10-15 minutes.

HEX DATA:
Name: {name}
Description: {description}
Tags: {tags}

ANALYSIS:
1. Can all described features realistically fit in 100m × 100m?
2. Would exploring this area take roughly 10-15 minutes?
3. Are any features described that are too large OR too small?

OUTPUT JSON:
{{
  "score": <0-10 integer>,
  "feedback": "<brief explanation of score>"
}}

Score guide:
- 9-10: Perfect 100m scale fit
- 7-8: Acceptable, minor scale issues
- 4-6: Questionable scale, needs revision
- 1-3: Wrong scale entirely
"""


class ScaleValidator:
    """Validates hex descriptions for 100m scale consistency."""

    def __init__(
        self,
        threshold: float = 7.0,
        api_key: Optional[str] = None,
        model: str = "deepseek-chat",
    ):
        self.threshold = threshold
        self.api_key = api_key or os.environ.get("DEEPSEEK_API_KEY")
        self.model = model
        self.client = httpx.Client(timeout=60.0)

    def validate(self, hex: TaggedHex) -> ScaleValidation:
        """Validate a hex's description for 100m scale.

        Returns:
            ScaleValidation with score, passes flag, and feedback
        """
        prompt = self._build_prompt(hex)
        response = self._call_llm(prompt)

        score = float(response.get("score", 0))
        feedback = response.get("feedback", "")

        return ScaleValidation(
            hex_index=0,  # Will be set by caller
            score=score,
            passes=score >= self.threshold,
            feedback=feedback,
        )

    def validate_batch(self, hexes: list[TaggedHex]) -> list[ScaleValidation]:
        """Validate multiple hexes."""
        results = []
        for idx, hex in enumerate(hexes):
            result = self.validate(hex)
            result.hex_index = idx
            results.append(result)
        return results

    def _build_prompt(self, hex: TaggedHex) -> str:
        """Build validation prompt for a hex."""
        return SCALE_VALIDATION_PROMPT.format(
            name=hex.name,
            description=hex.description,
            tags=", ".join(hex.tags),
        )

    def _call_llm(self, prompt: str) -> dict:
        """Call LLM API for scale validation."""
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
                    {"role": "system", "content": "You are a scale validation assistant. Output JSON only."},
                    {"role": "user", "content": prompt},
                ],
                "temperature": 0.3,  # Low temp for consistent scoring
                "max_tokens": 256,
                "response_format": {"type": "json_object"},
            },
        )
        response.raise_for_status()
        content = response.json()["choices"][0]["message"]["content"]
        return json.loads(content)

    def close(self):
        self.client.close()

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
```

### Verify

```bash
cd /home/astre/arc-citadel/worldgen
python -m pytest tests/test_scale_validator.py -v
```

---

## Task 6: Create edge handler with 3 modes

**File:** `/home/astre/arc-citadel/worldgen/edge_handler.py`

### Test First

**File:** `/home/astre/arc-citadel/worldgen/tests/test_edge_handler.py`

```python
"""Tests for edge handling modes."""
import pytest
from schemas import TaggedHex, EdgeType
from edge_handler import EdgeHandler, EdgeMode


@pytest.fixture
def explicit_handler():
    return EdgeHandler(mode=EdgeMode.EXPLICIT)


@pytest.fixture
def derived_handler():
    return EdgeHandler(mode=EdgeMode.DERIVED)


@pytest.fixture
def hybrid_handler():
    return EdgeHandler(mode=EdgeMode.HYBRID)


class TestExplicitMode:
    def test_explicit_passes_through_unchanged(self, explicit_handler):
        """Explicit mode returns edges as-is from LLM."""
        hex = TaggedHex(
            q=0, r=0,
            name="Test",
            description="Test",
            tags=["underground", "passage"],
            edge_types=["tunnel", "road", "blocked", "tunnel", "blocked", "blocked"],
        )
        result = explicit_handler.process(hex)
        assert result.edge_types == hex.edge_types


class TestDerivedMode:
    def test_underground_gets_tunnels(self, derived_handler):
        """Underground hex derives tunnel edges."""
        hex = TaggedHex(
            q=0, r=0,
            name="Cavern",
            description="Underground cavern",
            tags=["underground", "wild"],
            edge_types=["blocked"] * 6,  # Will be overwritten
        )
        result = derived_handler.process(hex)
        # Should have at least some tunnel edges for underground
        assert EdgeType.TUNNEL in result.edge_types

    def test_surface_wild_gets_wilderness(self, derived_handler):
        """Surface wild hex derives wilderness edges."""
        hex = TaggedHex(
            q=0, r=0,
            name="Forest",
            description="Forest clearing",
            tags=["surface", "wild"],
            edge_types=["blocked"] * 6,
        )
        result = derived_handler.process(hex)
        assert EdgeType.WILDERNESS in result.edge_types

    def test_passage_gets_multiple_open_edges(self, derived_handler):
        """Passage tag should have multiple non-blocked edges."""
        hex = TaggedHex(
            q=0, r=0,
            name="Tunnel Junction",
            description="Junction point",
            tags=["underground", "passage"],
            edge_types=["blocked"] * 6,
        )
        result = derived_handler.process(hex)
        non_blocked = [e for e in result.edge_types if e != EdgeType.BLOCKED]
        assert len(non_blocked) >= 2


class TestHybridMode:
    def test_heals_mismatched_edges(self, hybrid_handler):
        """Hybrid mode should fix obviously wrong edges."""
        hex = TaggedHex(
            q=0, r=0,
            name="Underground Road",
            description="Underground passage",
            tags=["underground", "passage"],
            edge_types=["road", "road", "road", "road", "road", "road"],  # Wrong for underground
        )
        result = hybrid_handler.process(hex)
        # Should convert roads to tunnels for underground
        assert EdgeType.TUNNEL in result.edge_types
```

### Implementation

**File:** `/home/astre/arc-citadel/worldgen/edge_handler.py`

```python
"""Edge handling modes for hex composition."""

from enum import Enum
from copy import deepcopy

from schemas import TaggedHex, EdgeType


class EdgeMode(str, Enum):
    """Edge handling strategy."""
    EXPLICIT = "explicit"  # Use LLM output as-is
    DERIVED = "derived"    # Infer edges from tags
    HYBRID = "hybrid"      # LLM suggests, system heals conflicts


class EdgeHandler:
    """Handles hex edge processing based on selected mode."""

    # Derivation rules: tag -> default edge type
    TAG_EDGE_DEFAULTS = {
        "underground": EdgeType.TUNNEL,
        "surface": EdgeType.WILDERNESS,
        "underwater": EdgeType.WATER,
        "aerial": EdgeType.BLOCKED,
        "passage": None,  # Special handling
        "dead_end": EdgeType.BLOCKED,
        "entrance": EdgeType.ENTRANCE,
    }

    # Tags that should NOT have certain edge types
    TAG_EDGE_CONFLICTS = {
        "underground": {EdgeType.ROAD, EdgeType.WILDERNESS},
        "underwater": {EdgeType.ROAD, EdgeType.TUNNEL},
        "aerial": {EdgeType.TUNNEL, EdgeType.WATER},
    }

    def __init__(self, mode: EdgeMode = EdgeMode.EXPLICIT):
        self.mode = mode

    def process(self, hex: TaggedHex) -> TaggedHex:
        """Process hex edges according to mode.

        Args:
            hex: Input hex (may be modified in DERIVED/HYBRID modes)

        Returns:
            TaggedHex with processed edges
        """
        if self.mode == EdgeMode.EXPLICIT:
            return hex
        elif self.mode == EdgeMode.DERIVED:
            return self._derive_edges(hex)
        elif self.mode == EdgeMode.HYBRID:
            return self._heal_edges(hex)
        else:
            raise ValueError(f"Unknown mode: {self.mode}")

    def _derive_edges(self, hex: TaggedHex) -> TaggedHex:
        """Infer edges entirely from tags."""
        result = deepcopy(hex)
        tag_set = set(hex.tags)

        # Determine base edge type from terrain
        base_edge = EdgeType.WILDERNESS
        if "underground" in tag_set:
            base_edge = EdgeType.TUNNEL
        elif "underwater" in tag_set:
            base_edge = EdgeType.WATER
        elif "aerial" in tag_set:
            base_edge = EdgeType.BLOCKED

        # Start with base edges
        new_edges = [base_edge] * 6

        # Apply function modifiers
        if "passage" in tag_set:
            # Passage: at least 2 open edges on opposite sides
            pass  # Keep base edges (allows passage)
        elif "dead_end" in tag_set:
            # Dead end: only 1 non-blocked edge
            new_edges = [EdgeType.BLOCKED] * 6
            new_edges[0] = base_edge

        if "entrance" in tag_set:
            # Entrance: one edge is entrance type
            new_edges[0] = EdgeType.ENTRANCE

        result.edge_types = new_edges
        return result

    def _heal_edges(self, hex: TaggedHex) -> TaggedHex:
        """Heal edges that conflict with tags."""
        result = deepcopy(hex)
        tag_set = set(hex.tags)

        # Find conflicting edge types for this hex's tags
        forbidden_edges: set[EdgeType] = set()
        for tag in tag_set:
            if tag in self.TAG_EDGE_CONFLICTS:
                forbidden_edges.update(self.TAG_EDGE_CONFLICTS[tag])

        if not forbidden_edges:
            return result

        # Determine replacement edge type
        replacement = EdgeType.BLOCKED
        if "underground" in tag_set:
            replacement = EdgeType.TUNNEL
        elif "underwater" in tag_set:
            replacement = EdgeType.WATER
        elif "surface" in tag_set:
            replacement = EdgeType.WILDERNESS

        # Replace forbidden edges
        new_edges = []
        for edge in result.edge_types:
            if edge in forbidden_edges:
                new_edges.append(replacement)
            else:
                new_edges.append(edge)

        result.edge_types = new_edges
        return result
```

### Verify

```bash
cd /home/astre/arc-citadel/worldgen
python -m pytest tests/test_edge_handler.py -v
```

---

## Task 7: Create cluster generator

**File:** `/home/astre/arc-citadel/worldgen/cluster_generator.py`

### Test First

**File:** `/home/astre/arc-citadel/worldgen/tests/test_cluster_generator.py`

```python
"""Tests for connected cluster generation."""
import pytest
from unittest.mock import Mock, patch, AsyncMock
from cluster_generator import ClusterGenerator
from schemas import TaggedHex, HexCluster


@pytest.fixture
def generator():
    return ClusterGenerator(api_key="test-key")


class TestClusterConnectivity:
    def test_generates_20_hexes(self, generator):
        """Cluster must have exactly 20 hexes."""
        with patch.object(generator, '_generate_single_hex') as mock_gen:
            mock_gen.return_value = TaggedHex(
                q=0, r=0,
                name="Test Hex",
                description="Test description for 100m area",
                tags=["surface", "wild"],
                edge_types=["wilderness"] * 6,
            )

            cluster = generator.generate(size=20)

        assert len(cluster.hexes) == 20

    def test_all_hexes_connected(self, generator):
        """All hexes in cluster must be connected."""
        with patch.object(generator, '_generate_single_hex') as mock_gen:
            mock_gen.return_value = TaggedHex(
                q=0, r=0,
                name="Test Hex",
                description="Test description",
                tags=["surface", "wild"],
                edge_types=["wilderness"] * 6,
            )

            cluster = generator.generate(size=20)

        # Verify connectivity via BFS
        if len(cluster.hexes) > 0:
            visited = set()
            coord_to_idx = {(h.q, h.r): i for i, h in enumerate(cluster.hexes)}

            def bfs(start_idx):
                from collections import deque
                from hex_coords import get_neighbor

                queue = deque([start_idx])
                visited.add(start_idx)

                while queue:
                    idx = queue.popleft()
                    hex = cluster.hexes[idx]

                    for edge in range(6):
                        nq, nr = get_neighbor(hex.q, hex.r, edge)
                        neighbor_idx = coord_to_idx.get((nq, nr))
                        if neighbor_idx is not None and neighbor_idx not in visited:
                            visited.add(neighbor_idx)
                            queue.append(neighbor_idx)

            bfs(0)
            assert len(visited) == len(cluster.hexes), "Not all hexes connected"


class TestAdjacencyContext:
    def test_context_includes_neighbors(self, generator):
        """Generation context should include existing neighbor info."""
        existing_hex = TaggedHex(
            q=0, r=0,
            name="Origin",
            description="Starting point",
            tags=["underground", "dwarf"],
            edge_types=["tunnel"] * 6,
        )

        context = generator._build_adjacency_context(
            existing_hexes={f"{existing_hex.q},{existing_hex.r}": existing_hex},
            new_q=1,
            new_r=0,
        )

        assert len(context["neighbors"]) >= 1
        assert context["neighbors"][0]["tags"] == ["underground", "dwarf"]
```

### Implementation

**File:** `/home/astre/arc-citadel/worldgen/cluster_generator.py`

```python
"""Connected hex cluster generation."""

import json
import os
import random
from collections import deque
from typing import Optional

import httpx

from schemas import TaggedHex, HexCluster, EdgeType
from hex_coords import get_neighbor, get_opposite_edge, coords_to_key, HEX_NEIGHBOR_OFFSETS
from adjacency import AdjacencyValidator


HEX_GENERATION_PROMPT = """Generate a fantasy location that fits within a 100-meter hex.

CRITICAL SCALE CONTEXT:
- This location fills a 100-meter hex (100m × 100m, about 1 hectare)
- A party can thoroughly explore this area in 10-15 minutes
- Examples of 100m scale: a forest grove, a hilltop, a mine entrance area, a market square
- NOT 100m scale: a single room (too small), a vast forest (too large)

LOCATION COORDINATES: ({q}, {r})

ADJACENT HEXES (must be compatible):
{adjacency_context}

TAG OPTIONS:
- TERRAIN (pick 1): underground, surface, underwater, aerial
- CULTURE (0-2): dwarf, elf, human, wild, ancient, corrupted
- FUNCTION (0-2): residential, military, sacred, industrial, commercial, passage
- ELEVATION (pick 1): deep, shallow_under, surface, elevated, peak

EDGE TYPES (exactly 6, clockwise from East):
- tunnel: underground passage
- road: surface road/path
- water: water connection
- wilderness: open terrain
- entrance: transition point
- blocked: impassable

{constraint_hints}

OUTPUT JSON:
{{
  "q": {q},
  "r": {r},
  "name": "<evocative 2-4 word name>",
  "description": "<2-3 sentences describing what exists in this 100m space>",
  "tags": ["<terrain>", "<culture>", "<function>", "<elevation>"],
  "edge_types": ["<edge0>", "<edge1>", "<edge2>", "<edge3>", "<edge4>", "<edge5>"]
}}
"""


class ClusterGenerator:
    """Generates connected clusters of tagged hexes."""

    def __init__(
        self,
        api_key: Optional[str] = None,
        model: str = "deepseek-chat",
        tags_path: str = "generation/hex_tags.toml",
    ):
        self.api_key = api_key or os.environ.get("DEEPSEEK_API_KEY")
        self.model = model
        self.client = httpx.Client(timeout=120.0)
        self.validator = AdjacencyValidator(tags_path)

    def generate(
        self,
        size: int = 20,
        seed_tags: Optional[list[str]] = None,
    ) -> HexCluster:
        """Generate a connected cluster of hexes.

        Args:
            size: Number of hexes to generate (default 20)
            seed_tags: Optional tags for the seed hex

        Returns:
            HexCluster with connected hexes
        """
        # Initialize with seed hex at origin
        seed_hex = self._generate_seed_hex(seed_tags)
        hexes: dict[str, TaggedHex] = {coords_to_key(0, 0): seed_hex}

        # BFS expansion frontier
        frontier: deque[tuple[int, int]] = deque([(0, 0)])

        while len(hexes) < size and frontier:
            q, r = frontier.popleft()
            current_hex = hexes[coords_to_key(q, r)]

            # Find available expansion edges
            available_edges = []
            for edge in range(6):
                nq, nr = get_neighbor(q, r, edge)
                if coords_to_key(nq, nr) not in hexes:
                    # Check if this edge allows expansion
                    if current_hex.edge_types[edge] != EdgeType.BLOCKED:
                        available_edges.append((edge, nq, nr))

            if not available_edges:
                continue

            # Add 1-2 new hexes from this position
            num_to_add = min(random.randint(1, 2), size - len(hexes), len(available_edges))
            edges_to_expand = random.sample(available_edges, num_to_add)

            for edge, nq, nr in edges_to_expand:
                if len(hexes) >= size:
                    break

                # Build context for new hex
                context = self._build_adjacency_context(hexes, nq, nr)

                # Generate new hex with context
                try:
                    new_hex = self._generate_single_hex(nq, nr, context)

                    # Validate adjacency
                    validation = self.validator.validate_adjacency(
                        current_hex, new_hex, edge
                    )

                    if validation.valid:
                        hexes[coords_to_key(nq, nr)] = new_hex
                        frontier.append((nq, nr))
                    else:
                        print(f"Adjacency invalid at ({nq},{nr}): {validation.errors}")
                        # Retry with stricter constraints
                        new_hex = self._generate_single_hex(
                            nq, nr, context,
                            constraint_hints=f"MUST FIX: {validation.errors}"
                        )
                        hexes[coords_to_key(nq, nr)] = new_hex
                        frontier.append((nq, nr))

                except Exception as e:
                    print(f"Generation failed at ({nq},{nr}): {e}")

        # Build adjacency list
        adjacencies = self._compute_adjacencies(hexes)

        return HexCluster(
            hexes=list(hexes.values()),
            adjacencies=adjacencies,
        )

    def _generate_seed_hex(self, tags: Optional[list[str]] = None) -> TaggedHex:
        """Generate the starting hex at origin."""
        default_tags = tags or ["surface", "wild", "passage", "surface"]

        return TaggedHex(
            q=0,
            r=0,
            name="Origin Point",
            description="A natural crossroads where several paths converge. Worn stones mark the intersection.",
            tags=default_tags,
            edge_types=[EdgeType.WILDERNESS] * 6,
        )

    def _build_adjacency_context(
        self,
        existing_hexes: dict[str, TaggedHex],
        new_q: int,
        new_r: int,
    ) -> dict:
        """Build context about adjacent hexes for LLM."""
        neighbors = []

        for edge in range(6):
            # Check if there's a hex in the opposite direction
            nq, nr = get_neighbor(new_q, new_r, edge)
            key = coords_to_key(nq, nr)

            if key in existing_hexes:
                neighbor = existing_hexes[key]
                opposite = get_opposite_edge(edge)
                neighbors.append({
                    "direction": edge,
                    "direction_name": ["E", "NE", "NW", "W", "SW", "SE"][edge],
                    "tags": neighbor.tags,
                    "their_edge_type": neighbor.edge_types[opposite],
                    "hint": f"Edge {edge} must match their edge {opposite} ({neighbor.edge_types[opposite]})",
                })

        return {"neighbors": neighbors}

    def _generate_single_hex(
        self,
        q: int,
        r: int,
        adjacency_context: dict,
        constraint_hints: str = "",
    ) -> TaggedHex:
        """Generate a single hex with LLM."""
        # Format adjacency context for prompt
        if adjacency_context["neighbors"]:
            context_str = json.dumps(adjacency_context["neighbors"], indent=2)
        else:
            context_str = "None - this is an edge hex"

        prompt = HEX_GENERATION_PROMPT.format(
            q=q,
            r=r,
            adjacency_context=context_str,
            constraint_hints=constraint_hints,
        )

        response = self._call_llm(prompt)

        # Parse and validate
        return TaggedHex(
            q=response["q"],
            r=response["r"],
            name=response["name"],
            description=response["description"],
            tags=response["tags"],
            edge_types=[EdgeType(e) for e in response["edge_types"]],
        )

    def _call_llm(self, prompt: str) -> dict:
        """Call LLM API."""
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
                    {"role": "system", "content": "You are a fantasy world generator. Output valid JSON only."},
                    {"role": "user", "content": prompt},
                ],
                "temperature": 0.8,
                "max_tokens": 1024,
                "response_format": {"type": "json_object"},
            },
        )
        response.raise_for_status()
        content = response.json()["choices"][0]["message"]["content"]
        return json.loads(content)

    def _compute_adjacencies(
        self,
        hexes: dict[str, TaggedHex],
    ) -> list[tuple[int, int, int]]:
        """Compute adjacency list from hex positions."""
        hex_list = list(hexes.values())
        coord_to_idx = {(h.q, h.r): i for i, h in enumerate(hex_list)}

        adjacencies = []
        for idx, hex in enumerate(hex_list):
            for edge in range(6):
                nq, nr = get_neighbor(hex.q, hex.r, edge)
                neighbor_idx = coord_to_idx.get((nq, nr))
                if neighbor_idx is not None and neighbor_idx > idx:
                    adjacencies.append((idx, neighbor_idx, edge))

        return adjacencies

    def close(self):
        self.client.close()

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
```

### Verify

```bash
cd /home/astre/arc-citadel/worldgen
python -m pytest tests/test_cluster_generator.py -v
```

---

## Task 8: Integration test - 20 hex cluster

**File:** `/home/astre/arc-citadel/worldgen/tests/test_cluster_20.py`

```python
"""Integration test: Generate and validate 20-hex cluster."""
import os
import pytest
from cluster_generator import ClusterGenerator
from scale_validator import ScaleValidator
from adjacency import AdjacencyValidator
from hex_coords import get_neighbor, get_opposite_edge


# Skip if no API key
pytestmark = pytest.mark.skipif(
    not os.environ.get("DEEPSEEK_API_KEY"),
    reason="DEEPSEEK_API_KEY not set"
)


class TestCluster20Integration:
    """Full integration test for 20-hex cluster generation."""

    @pytest.fixture
    def cluster(self):
        """Generate a 20-hex cluster (cached for test session)."""
        with ClusterGenerator() as gen:
            return gen.generate(size=20, seed_tags=["underground", "dwarf", "passage", "shallow_under"])

    def test_cluster_has_20_hexes(self, cluster):
        """Cluster must have exactly 20 hexes."""
        assert len(cluster.hexes) == 20

    def test_all_hexes_have_valid_tags(self, cluster):
        """All hexes must have at least one tag."""
        for hex in cluster.hexes:
            assert len(hex.tags) >= 1
            # Must have a TERRAIN tag
            terrain_tags = {"underground", "surface", "underwater", "aerial"}
            assert any(t in terrain_tags for t in hex.tags), f"Hex {hex.name} missing TERRAIN tag"

    def test_all_hexes_have_6_edges(self, cluster):
        """All hexes must have exactly 6 edges."""
        for hex in cluster.hexes:
            assert len(hex.edge_types) == 6

    def test_cluster_is_connected(self, cluster):
        """All hexes must be reachable from the origin."""
        coord_to_hex = {(h.q, h.r): h for h in cluster.hexes}
        visited = set()

        def dfs(q, r):
            if (q, r) in visited:
                return
            if (q, r) not in coord_to_hex:
                return
            visited.add((q, r))
            for edge in range(6):
                nq, nr = get_neighbor(q, r, edge)
                dfs(nq, nr)

        # Start from first hex
        first = cluster.hexes[0]
        dfs(first.q, first.r)

        assert len(visited) == 20, f"Only {len(visited)} hexes connected, expected 20"

    def test_adjacencies_are_valid(self, cluster):
        """All adjacent hexes must pass adjacency validation."""
        validator = AdjacencyValidator("generation/hex_tags.toml")
        coord_to_hex = {(h.q, h.r): h for h in cluster.hexes}

        errors = []
        for hex in cluster.hexes:
            for edge in range(6):
                nq, nr = get_neighbor(hex.q, hex.r, edge)
                neighbor = coord_to_hex.get((nq, nr))
                if neighbor:
                    result = validator.validate_adjacency(hex, neighbor, edge)
                    if not result.valid:
                        errors.append(f"({hex.q},{hex.r}) -> ({nq},{nr}): {result.errors}")

        assert len(errors) == 0, f"Adjacency errors:\n" + "\n".join(errors)

    def test_scale_validation_passes(self, cluster):
        """All hexes must pass 100m scale validation (score >= 7)."""
        with ScaleValidator(threshold=7.0) as validator:
            results = validator.validate_batch(cluster.hexes)

        failures = [r for r in results if not r.passes]

        # Allow up to 2 failures (LLM isn't perfect)
        assert len(failures) <= 2, f"{len(failures)} hexes failed scale validation:\n" + \
            "\n".join(f"  Hex {r.hex_index}: score={r.score}, {r.feedback}" for r in failures)

    def test_edge_types_match_at_borders(self, cluster):
        """Matching edge types must be equal between adjacent hexes."""
        coord_to_hex = {(h.q, h.r): h for h in cluster.hexes}
        matching_types = {"tunnel", "road", "water"}

        errors = []
        for hex in cluster.hexes:
            for edge in range(6):
                nq, nr = get_neighbor(hex.q, hex.r, edge)
                neighbor = coord_to_hex.get((nq, nr))
                if neighbor:
                    my_edge = hex.edge_types[edge].value
                    their_edge = neighbor.edge_types[get_opposite_edge(edge)].value

                    if my_edge in matching_types or their_edge in matching_types:
                        if my_edge != their_edge:
                            errors.append(
                                f"({hex.q},{hex.r}) edge {edge}={my_edge} != "
                                f"({nq},{nr}) edge {get_opposite_edge(edge)}={their_edge}"
                            )

        assert len(errors) == 0, f"Edge mismatches:\n" + "\n".join(errors)


class TestEdgeModeComparison:
    """Compare the 3 edge handling modes."""

    @pytest.fixture(params=["explicit", "derived", "hybrid"])
    def edge_mode(self, request):
        return request.param

    def test_each_mode_produces_valid_cluster(self, edge_mode):
        """Each edge mode should produce a valid 20-hex cluster."""
        from edge_handler import EdgeHandler, EdgeMode

        handler = EdgeHandler(mode=EdgeMode(edge_mode))

        with ClusterGenerator() as gen:
            cluster = gen.generate(size=20)

        # Process edges with handler
        processed_hexes = [handler.process(h) for h in cluster.hexes]

        # Basic validation
        assert len(processed_hexes) == 20
        for hex in processed_hexes:
            assert len(hex.edge_types) == 6
```

### Run Full Integration Test

```bash
cd /home/astre/arc-citadel/worldgen
export DEEPSEEK_API_KEY="your-key-here"
python -m pytest tests/test_cluster_20.py -v --tb=short
```

**Expected output:**
```
tests/test_cluster_20.py::TestCluster20Integration::test_cluster_has_20_hexes PASSED
tests/test_cluster_20.py::TestCluster20Integration::test_all_hexes_have_valid_tags PASSED
tests/test_cluster_20.py::TestCluster20Integration::test_all_hexes_have_6_edges PASSED
tests/test_cluster_20.py::TestCluster20Integration::test_cluster_is_connected PASSED
tests/test_cluster_20.py::TestCluster20Integration::test_adjacencies_are_valid PASSED
tests/test_cluster_20.py::TestCluster20Integration::test_scale_validation_passes PASSED
tests/test_cluster_20.py::TestCluster20Integration::test_edge_types_match_at_borders PASSED
```

---

## Task Summary

| Task | File | Type | Est. Lines |
|------|------|------|------------|
| 1 | generation/hex_tags.toml | NEW | 60 |
| 2 | schemas.py | MODIFY | +50 |
| 3 | hex_coords.py | NEW | 70 |
| 4 | adjacency.py | NEW | 130 |
| 5 | scale_validator.py | NEW | 100 |
| 6 | edge_handler.py | NEW | 100 |
| 7 | cluster_generator.py | NEW | 200 |
| 8 | tests/test_cluster_20.py | NEW | 130 |

**Total: ~840 lines of code + tests**

## Execution Order

1. **Task 1** - hex_tags.toml (no dependencies)
2. **Task 2** - schemas.py (no dependencies)
3. **Task 3** - hex_coords.py (no dependencies)
4. **Task 4** - adjacency.py (depends on 1, 2, 3)
5. **Task 5** - scale_validator.py (depends on 2)
6. **Task 6** - edge_handler.py (depends on 2)
7. **Task 7** - cluster_generator.py (depends on all above)
8. **Task 8** - integration test (depends on all above)

## Verification Checklist

- [ ] `python -m pytest tests/test_schemas.py -v` passes
- [ ] `python -m pytest tests/test_hex_coords.py -v` passes
- [ ] `python -m pytest tests/test_adjacency.py -v` passes
- [ ] `python -m pytest tests/test_scale_validator.py -v` passes
- [ ] `python -m pytest tests/test_edge_handler.py -v` passes
- [ ] `python -m pytest tests/test_cluster_generator.py -v` passes
- [ ] `python -m pytest tests/test_cluster_20.py -v` passes (requires API key)
- [ ] Generated cluster has 20 connected hexes
- [ ] All hexes pass scale validation (score >= 7)
- [ ] All adjacencies pass hard constraints
