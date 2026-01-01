# World Generation Asset Pipeline Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Python CLI pipeline that generates world assets using DeepSeek LLM, stores them in SQLite, and assembles worlds from seed files.

**Architecture:** DeepSeek generates a component library once (~$200 at scale). Cluster templates define assembly instructions. Seeds are tiny 2KB recipes. Assembly is pure procedural at runtime. MVP validates with ~100 components per category.

**Tech Stack:** Python 3.11+, Pydantic v2, SQLite, Click CLI, OpenAI-compatible DeepSeek API, PyYAML

---

## Project Structure

```
/worldgen/
  __init__.py
  config.py
  cli.py

  schemas/
    __init__.py
    base.py
    component.py
    template.py
    connector.py
    minor.py
    seed.py
    world.py

  generation/
    __init__.py
    llm_client.py
    quality_loop.py
    component_generator.py
    prompts/
      scoring.txt
      dwarf_hold_entrance.txt

  templates/
    __init__.py
    template_loader.py
    dwarf/
      hold_major.yaml

  seeds/
    __init__.py
    seed_generator.py

  assembly/
    __init__.py
    cluster_assembler.py
    layout_solver.py
    assembler.py

  storage/
    __init__.py
    database.py
```

---

## Task 1: Project Setup

**Files:**
- Create: `worldgen/pyproject.toml`
- Create: `worldgen/__init__.py`
- Create: `worldgen/config.py`

**Step 1: Create pyproject.toml**

```toml
[project]
name = "worldgen"
version = "0.1.0"
description = "Arc Citadel World Generation Pipeline"
requires-python = ">=3.11"
dependencies = [
    "pydantic>=2.0",
    "click>=8.0",
    "openai>=1.0",
    "pyyaml>=6.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.0",
    "pytest-cov>=4.0",
]

[project.scripts]
worldgen = "worldgen.cli:cli"

[build-system]
requires = ["setuptools>=61.0"]
build-backend = "setuptools.build_meta"
```

**Step 2: Create __init__.py**

```python
"""Arc Citadel World Generation Pipeline."""

__version__ = "0.1.0"
```

**Step 3: Create config.py**

```python
"""Configuration for worldgen pipeline."""

import os
from pathlib import Path

# Paths
WORLDGEN_ROOT = Path(__file__).parent
OUTPUT_DIR = WORLDGEN_ROOT / "output"
LIBRARIES_DIR = OUTPUT_DIR / "libraries"
SEEDS_DIR = OUTPUT_DIR / "seeds"
WORLDS_DIR = OUTPUT_DIR / "worlds"
TEMPLATES_DIR = WORLDGEN_ROOT / "templates"
PROMPTS_DIR = WORLDGEN_ROOT / "generation" / "prompts"

# Database
DATABASE_PATH = LIBRARIES_DIR / "assets.db"

# DeepSeek API
DEEPSEEK_API_KEY = os.environ.get("DEEPSEEK_API_KEY", "")
DEEPSEEK_BASE_URL = "https://api.deepseek.com"
DEEPSEEK_MODEL = "deepseek-chat"

# Generation
DEFAULT_TARGET_SCORE = 9.0
MAX_QUALITY_ITERATIONS = 10
CANDIDATES_PER_ROUND = 3

# MVP: Reduced counts for validation
MVP_COMPONENTS_PER_CATEGORY = 100
```

**Step 4: Install and verify**

Run:
```bash
cd /home/astre/arc-citadel/worldgen
pip install -e ".[dev]"
python -c "from worldgen import config; print(config.DATABASE_PATH)"
```

Expected: Path printed without errors

**Step 5: Commit**

```bash
git add worldgen/pyproject.toml worldgen/__init__.py worldgen/config.py
git commit -m "feat(worldgen): add project setup with config"
```

---

## Task 2: Base Schema

**Files:**
- Create: `worldgen/schemas/__init__.py`
- Create: `worldgen/schemas/base.py`
- Create: `worldgen/tests/__init__.py`
- Create: `worldgen/tests/test_schemas.py`

**Step 1: Write failing test for HexCoord**

Create `worldgen/tests/__init__.py`:
```python
"""Tests for worldgen."""
```

Create `worldgen/tests/test_schemas.py`:
```python
"""Tests for schema models."""

import pytest
from worldgen.schemas.base import HexCoord, Terrain, Species


class TestHexCoord:
    def test_distance_to_same_hex(self):
        a = HexCoord(q=0, r=0)
        assert a.distance_to(a) == 0

    def test_distance_to_adjacent(self):
        a = HexCoord(q=0, r=0)
        b = HexCoord(q=1, r=0)
        assert a.distance_to(b) == 1

    def test_distance_to_diagonal(self):
        a = HexCoord(q=0, r=0)
        b = HexCoord(q=2, r=-1)
        assert a.distance_to(b) == 2

    def test_distance_symmetric(self):
        a = HexCoord(q=3, r=-2)
        b = HexCoord(q=-1, r=4)
        assert a.distance_to(b) == b.distance_to(a)

    def test_hash_equality(self):
        a = HexCoord(q=5, r=3)
        b = HexCoord(q=5, r=3)
        assert hash(a) == hash(b)
        assert a == b


class TestEnums:
    def test_terrain_values(self):
        assert Terrain.MOUNTAINS.value == "mountains"
        assert Terrain.FOREST.value == "forest"

    def test_species_values(self):
        assert Species.DWARF.value == "dwarf"
        assert Species.ELF.value == "elf"
        assert Species.HUMAN.value == "human"
```

**Step 2: Run test to verify it fails**

Run: `cd /home/astre/arc-citadel/worldgen && python -m pytest tests/test_schemas.py -v`

Expected: ModuleNotFoundError for worldgen.schemas.base

**Step 3: Create schemas/__init__.py**

```python
"""Pydantic schemas for worldgen."""

from .base import (
    Terrain,
    ResourceType,
    Abundance,
    FeatureType,
    Species,
    Resource,
    Feature,
    SpeciesFitness,
    HexCoord,
    generate_stable_id,
)

__all__ = [
    "Terrain",
    "ResourceType",
    "Abundance",
    "FeatureType",
    "Species",
    "Resource",
    "Feature",
    "SpeciesFitness",
    "HexCoord",
    "generate_stable_id",
]
```

**Step 4: Create schemas/base.py**

```python
"""Base types and enums for worldgen schemas."""

from enum import Enum
from typing import Optional
import hashlib

from pydantic import BaseModel, Field


class Terrain(str, Enum):
    DEEP_WATER = "deep_water"
    SHALLOW_WATER = "shallow_water"
    MARSH = "marsh"
    PLAINS = "plains"
    HILLS = "hills"
    FOREST = "forest"
    DENSE_FOREST = "dense_forest"
    MOUNTAINS = "mountains"
    HIGH_MOUNTAINS = "high_mountains"
    DESERT = "desert"
    TUNDRA = "tundra"
    VOLCANIC = "volcanic"
    GLACIER = "glacier"
    UNDERGROUND = "underground"
    CAVERN = "cavern"


class ResourceType(str, Enum):
    # Metals
    IRON = "iron"
    COPPER = "copper"
    TIN = "tin"
    GOLD = "gold"
    SILVER = "silver"
    MITHRIL = "mithril"
    # Stone
    STONE = "stone"
    MARBLE = "marble"
    GEMS = "gems"
    OBSITE = "obsidite"
    # Organic
    TIMBER = "timber"
    HARDWOOD = "hardwood"
    GAME = "game"
    FISH = "fish"
    HERBS = "herbs"
    RARE_HERBS = "rare_herbs"
    # Agricultural
    FERTILE_SOIL = "fertile_soil"
    CLAY = "clay"
    SALT = "salt"
    # Special
    ANCIENT_ARTIFACT = "ancient_artifact"
    MAGICAL_RESIDUE = "magical_residue"


class Abundance(str, Enum):
    TRACE = "trace"
    SCARCE = "scarce"
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    RICH = "rich"
    LEGENDARY = "legendary"


class FeatureType(str, Enum):
    # Water
    RIVER = "river"
    RIVER_SOURCE = "river_source"
    RIVER_CONFLUENCE = "river_confluence"
    RIVER_MOUTH = "river_mouth"
    WATERFALL = "waterfall"
    FORD = "ford"
    RAPIDS = "rapids"
    LAKE = "lake"
    SPRING = "spring"
    HOT_SPRINGS = "hot_springs"
    # Terrain
    CLIFF = "cliff"
    GORGE = "gorge"
    PASS = "pass"
    CAVE_ENTRANCE = "cave_entrance"
    SINKHOLE = "sinkhole"
    VOLCANIC_VENT = "volcanic_vent"
    # Constructed
    BRIDGE = "bridge"
    BRIDGE_RUINS = "bridge_ruins"
    OLD_ROAD = "old_road"
    PAVED_ROAD = "paved_road"
    WALL = "wall"
    WALL_RUINS = "wall_ruins"
    GATE = "gate"
    TOWER = "tower"
    RUINS = "ruins"
    STANDING_STONES = "standing_stones"
    MONUMENT = "monument"
    STATUE = "statue"
    WELL = "well"
    MINE_ENTRANCE = "mine_entrance"
    # Natural
    CLEARING = "clearing"
    ANCIENT_TREE = "ancient_tree"
    GROVE = "grove"
    ROCK_FORMATION = "rock_formation"
    # Underground
    STALACTITES = "stalactites"
    UNDERGROUND_LAKE = "underground_lake"
    CRYSTAL_FORMATION = "crystal_formation"
    CARVED_HALL = "carved_hall"
    FORGE_CHAMBER = "forge_chamber"


class Species(str, Enum):
    HUMAN = "human"
    DWARF = "dwarf"
    ELF = "elf"
    NEUTRAL = "neutral"


class Resource(BaseModel):
    """A resource deposit."""

    type: ResourceType
    abundance: Abundance


class Feature(BaseModel):
    """A geographic or constructed feature."""

    type: FeatureType
    details: Optional[str] = None
    species_origin: Optional[Species] = None


class SpeciesFitness(BaseModel):
    """How suitable a location is for each species."""

    human: float = Field(ge=0.0, le=1.0)
    dwarf: float = Field(ge=0.0, le=1.0)
    elf: float = Field(ge=0.0, le=1.0)


class HexCoord(BaseModel):
    """Axial hex coordinates."""

    q: int
    r: int

    def __hash__(self) -> int:
        return hash((self.q, self.r))

    def __eq__(self, other: object) -> bool:
        if not isinstance(other, HexCoord):
            return False
        return self.q == other.q and self.r == other.r

    def distance_to(self, other: "HexCoord") -> int:
        """Calculate hex distance using axial coordinates."""
        dq = abs(self.q - other.q)
        dr = abs(self.r - other.r)
        ds = abs((self.q + self.r) - (other.q + other.r))
        return max(dq, dr, ds)


def generate_stable_id(
    category: str, subcategory: str, content: str, index: int
) -> str:
    """Generate stable ID that persists across library updates.

    Format: {category}_{subcategory}_{hash8}_{index:05d}
    Example: comp_dwarf_forge_a3f2b1c9_00142
    """
    hash_input = f"{category}:{subcategory}:{content}"
    hash_short = hashlib.sha256(hash_input.encode()).hexdigest()[:8]
    return f"{category}_{subcategory}_{hash_short}_{index:05d}"
```

**Step 5: Run tests to verify they pass**

Run: `cd /home/astre/arc-citadel/worldgen && python -m pytest tests/test_schemas.py -v`

Expected: All 7 tests pass

**Step 6: Commit**

```bash
git add worldgen/schemas/ worldgen/tests/
git commit -m "feat(worldgen): add base schema with HexCoord and enums"
```

---

## Task 3: Component Schema

**Files:**
- Create: `worldgen/schemas/component.py`
- Modify: `worldgen/schemas/__init__.py`
- Modify: `worldgen/tests/test_schemas.py`

**Step 1: Write failing test for Component**

Add to `worldgen/tests/test_schemas.py`:
```python
from worldgen.schemas.component import Component, ComponentCategory, ConnectionPoint


class TestComponent:
    def test_create_minimal_component(self):
        comp = Component(
            id="comp_dwarf_forge_abc12345_00001",
            category=ComponentCategory.DWARF_HOLD_FORGE,
            tags=["dwarf", "forge"],
            species=Species.DWARF,
            terrain=Terrain.UNDERGROUND,
            elevation=500.0,
            moisture=0.2,
            temperature=25.0,
            species_fitness=SpeciesFitness(human=0.3, dwarf=0.95, elf=0.1),
            name_fragment="Deep Forge",
            narrative_hook="Ancient hammers still ring in these halls.",
            quality_score=8.5,
        )
        assert comp.id == "comp_dwarf_forge_abc12345_00001"
        assert comp.category == ComponentCategory.DWARF_HOLD_FORGE

    def test_component_with_connections(self):
        conn = ConnectionPoint(
            direction="DOWN",
            compatible_categories=["dwarf_hold_tunnel"],
            required=True,
        )
        comp = Component(
            id="test_comp",
            category=ComponentCategory.DWARF_HOLD_ENTRANCE,
            tags=["dwarf", "entrance"],
            species=Species.DWARF,
            terrain=Terrain.MOUNTAINS,
            elevation=1500.0,
            moisture=0.15,
            temperature=5.0,
            species_fitness=SpeciesFitness(human=0.3, dwarf=0.9, elf=0.1),
            name_fragment="Iron Gate",
            narrative_hook="The gate has stood for a thousand years.",
            connection_points=[conn],
            quality_score=9.0,
        )
        assert len(comp.connection_points) == 1
        assert comp.connection_points[0].required is True
```

**Step 2: Run test to verify it fails**

Run: `cd /home/astre/arc-citadel/worldgen && python -m pytest tests/test_schemas.py::TestComponent -v`

Expected: ImportError for component module

**Step 3: Create schemas/component.py**

```python
"""Component schema for hex-sized building blocks."""

from enum import Enum
from typing import Optional

from pydantic import BaseModel, Field

from .base import Terrain, Species, Resource, Feature, SpeciesFitness


class ComponentCategory(str, Enum):
    # Dwarf
    DWARF_HOLD_ENTRANCE = "dwarf_hold_entrance"
    DWARF_HOLD_FORGE = "dwarf_hold_forge"
    DWARF_HOLD_TAVERN = "dwarf_hold_tavern"
    DWARF_HOLD_BARRACKS = "dwarf_hold_barracks"
    DWARF_HOLD_ANCESTOR_HALL = "dwarf_hold_ancestor_hall"
    DWARF_HOLD_MINE_SHAFT = "dwarf_hold_mine_shaft"
    DWARF_HOLD_TREASURY = "dwarf_hold_treasury"
    DWARF_HOLD_THRONE_ROOM = "dwarf_hold_throne_room"
    DWARF_HOLD_TUNNEL = "dwarf_hold_tunnel"
    DWARF_HOLD_DEFENSIVE = "dwarf_hold_defensive"
    DWARF_HOLD_STORAGE = "dwarf_hold_storage"
    DWARF_HOLD_LIVING_QUARTERS = "dwarf_hold_living_quarters"
    DWARF_HOLD_MUSHROOM_FARM = "dwarf_hold_mushroom_farm"
    # Elf
    ELF_GROVE_HEART_TREE = "elf_grove_heart_tree"
    ELF_GROVE_DWELLING = "elf_grove_dwelling"
    ELF_GROVE_COUNCIL_GLADE = "elf_grove_council_glade"
    ELF_GROVE_SANCTUARY = "elf_grove_sanctuary"
    ELF_GROVE_PATH = "elf_grove_path"
    ELF_GROVE_ARCHIVE = "elf_grove_archive"
    ELF_GROVE_TRAINING_GROUND = "elf_grove_training_ground"
    ELF_GROVE_MEMORIAL = "elf_grove_memorial"
    # Human
    HUMAN_CITY_GATE = "human_city_gate"
    HUMAN_CITY_MARKET = "human_city_market"
    HUMAN_CITY_TEMPLE = "human_city_temple"
    HUMAN_CITY_BARRACKS = "human_city_barracks"
    HUMAN_CITY_PALACE = "human_city_palace"
    HUMAN_CITY_GUILD_HALL = "human_city_guild_hall"
    HUMAN_CITY_SLUMS = "human_city_slums"
    HUMAN_CITY_WEALTHY_DISTRICT = "human_city_wealthy_district"
    HUMAN_CITY_STREET = "human_city_street"
    HUMAN_CITY_WALL = "human_city_wall"
    HUMAN_CITY_DOCK = "human_city_dock"
    HUMAN_CITY_WAREHOUSE = "human_city_warehouse"
    # Wilderness
    WILDERNESS_CAVE_ENTRANCE = "wilderness_cave_entrance"
    WILDERNESS_CAVE_CHAMBER = "wilderness_cave_chamber"
    WILDERNESS_CLEARING = "wilderness_clearing"
    WILDERNESS_SPRING = "wilderness_spring"
    WILDERNESS_ROCK_FORMATION = "wilderness_rock_formation"
    # Ruins
    RUIN_ENTRANCE = "ruin_entrance"
    RUIN_COLLAPSED_HALL = "ruin_collapsed_hall"
    RUIN_INTACT_CHAMBER = "ruin_intact_chamber"
    RUIN_TREASURE_ROOM = "ruin_treasure_room"
    RUIN_TRAPPED_CORRIDOR = "ruin_trapped_corridor"


class ConnectionPoint(BaseModel):
    """Where this component can connect to others."""

    direction: str  # "N", "NE", "SE", "S", "SW", "NW", "UP", "DOWN"
    compatible_categories: list[str]
    required: bool = False
    elevation_delta: int = 0


class Component(BaseModel):
    """A single hex-sized building block."""

    # Identity
    id: str
    version: int = 1

    # Classification
    category: ComponentCategory
    tags: list[str]
    species: Species

    # Hex data
    terrain: Terrain
    elevation: float = Field(ge=-500, le=5000)
    moisture: float = Field(ge=0.0, le=1.0)
    temperature: float = Field(ge=-40, le=50)
    resources: list[Resource] = Field(default_factory=list)
    features: list[Feature] = Field(default_factory=list)
    species_fitness: SpeciesFitness

    # Narrative
    name_fragment: str
    narrative_hook: str
    sensory_details: list[str] = Field(default_factory=list)

    # Connectivity
    connection_points: list[ConnectionPoint] = Field(default_factory=list)

    # Constraints for use in clusters
    depth_preference: tuple[int, int] = (0, 0)
    edge_allowed: bool = True
    centrality_preference: float = 0.5

    # Quality metadata
    quality_score: float = Field(ge=0.0, le=10.0)
    generation_notes: Optional[str] = None
```

**Step 4: Update schemas/__init__.py**

```python
"""Pydantic schemas for worldgen."""

from .base import (
    Terrain,
    ResourceType,
    Abundance,
    FeatureType,
    Species,
    Resource,
    Feature,
    SpeciesFitness,
    HexCoord,
    generate_stable_id,
)
from .component import Component, ComponentCategory, ConnectionPoint

__all__ = [
    # base
    "Terrain",
    "ResourceType",
    "Abundance",
    "FeatureType",
    "Species",
    "Resource",
    "Feature",
    "SpeciesFitness",
    "HexCoord",
    "generate_stable_id",
    # component
    "Component",
    "ComponentCategory",
    "ConnectionPoint",
]
```

**Step 5: Run tests**

Run: `cd /home/astre/arc-citadel/worldgen && python -m pytest tests/test_schemas.py -v`

Expected: All tests pass

**Step 6: Commit**

```bash
git add worldgen/schemas/
git commit -m "feat(worldgen): add Component schema with categories"
```

---

## Task 4: Template, Connector, Minor, Seed Schemas

**Files:**
- Create: `worldgen/schemas/template.py`
- Create: `worldgen/schemas/connector.py`
- Create: `worldgen/schemas/minor.py`
- Create: `worldgen/schemas/seed.py`
- Create: `worldgen/schemas/world.py`
- Modify: `worldgen/schemas/__init__.py`

**Step 1: Create schemas/template.py**

```python
"""Cluster template schemas."""

from typing import Optional

from pydantic import BaseModel, Field

from .base import Species
from .component import ComponentCategory


class SlotCount(BaseModel):
    min: int
    max: int


class ClusterSlot(BaseModel):
    """A slot in a cluster template that gets filled with a component."""

    slot_id: str
    component_category: ComponentCategory
    required: bool
    count: SlotCount

    # Spatial constraints
    adjacent_to: list[str] = Field(default_factory=list)
    not_adjacent_to: list[str] = Field(default_factory=list)
    depth_range: tuple[int, int] = (0, 999)
    edge_allowed: bool = True
    center_preference: float = 0.5

    # Selection criteria
    required_tags: list[str] = Field(default_factory=list)
    preferred_tags: list[str] = Field(default_factory=list)
    excluded_tags: list[str] = Field(default_factory=list)


class InternalConnection(BaseModel):
    """Connection between slots within a cluster."""

    from_slot: str
    to_slot: str
    connector_category: str
    required: bool = True
    max_length: int = 5


class ExternalPort(BaseModel):
    """Where external connectors can attach to cluster."""

    port_id: str
    attached_slot: str
    direction_preference: list[str] = Field(default_factory=list)
    compatible_connectors: list[str] = Field(default_factory=list)
    required: bool = False


class ClusterTemplate(BaseModel):
    """Template defining how to assemble a location from components."""

    id: str
    name: str
    version: int = 1

    species: Species
    tags: list[str] = Field(default_factory=list)

    slots: list[ClusterSlot]
    internal_connections: list[InternalConnection] = Field(default_factory=list)
    external_ports: list[ExternalPort] = Field(default_factory=list)

    min_footprint: int
    max_footprint: int

    terrain_requirements: list[str] = Field(default_factory=list)
    min_distance_same_type: int = 20

    description: str
    history_seeds: list[str] = Field(default_factory=list)


class AssembledCluster(BaseModel):
    """A cluster after components have been selected and arranged."""

    template_id: str
    instance_id: str

    components: dict[str, list[str]]  # slot_id -> component IDs
    layout: dict[str, tuple[int, int]]  # component instance -> hex offset
    internal_connectors: list[dict] = Field(default_factory=list)
    port_positions: dict[str, tuple[int, int]] = Field(default_factory=dict)
    footprint: list[tuple[int, int]] = Field(default_factory=list)

    def get_component_at(self, offset: tuple[int, int]) -> Optional[str]:
        """Get component ID at given offset, or None."""
        for comp_id, comp_offset in self.layout.items():
            if comp_offset == offset:
                return comp_id
        return None
```

**Step 2: Create schemas/connector.py**

```python
"""Connector collection schemas for geographic features."""

from enum import Enum
from typing import Optional

from pydantic import BaseModel, Field

from .base import Terrain, Resource, Feature, SpeciesFitness


class ConnectorType(str, Enum):
    # Water
    RIVER_HEADWATERS = "river_headwaters"
    RIVER_UPPER = "river_upper"
    RIVER_MIDDLE = "river_middle"
    RIVER_LOWER = "river_lower"
    RIVER_DELTA = "river_delta"
    RIVER_FULL = "river_full"
    LAKE_SHORE = "lake_shore"
    COASTLINE = "coastline"
    # Mountains
    MOUNTAIN_RANGE_MAJOR = "mountain_range_major"
    MOUNTAIN_RANGE_MINOR = "mountain_range_minor"
    MOUNTAIN_SPUR = "mountain_spur"
    HIGHLAND = "highland"
    # Forest
    FOREST_BAND_DENSE = "forest_band_dense"
    FOREST_BAND_LIGHT = "forest_band_light"
    ANCIENT_FOREST = "ancient_forest"
    FOREST_EDGE = "forest_edge"
    # Routes
    TRADE_ROUTE_MAJOR = "trade_route_major"
    TRADE_ROUTE_MINOR = "trade_route_minor"
    MILITARY_ROAD = "military_road"
    PILGRIM_PATH = "pilgrim_path"
    MOUNTAIN_PASS = "mountain_pass"
    # Transitions
    PLAINS_BAND = "plains_band"
    MARSH_SYSTEM = "marsh_system"
    DESERT_EDGE = "desert_edge"
    TUNDRA_BAND = "tundra_band"


class EntryPoint(BaseModel):
    """Where other assets connect to this collection."""

    hex_index: int
    direction: str
    compatible_tags: list[str] = Field(default_factory=list)
    elevation: float
    terrain: Terrain
    is_terminus: bool = False


class MinorAnchorSlot(BaseModel):
    """A slot where a minor anchor can be placed."""

    slot_id: str
    hex_index: int
    compatible_categories: list[str] = Field(default_factory=list)
    required: bool = False
    narrative_context: str = ""


class ElasticSegment(BaseModel):
    """A segment that can stretch or compress."""

    start_hex: int
    end_hex: int
    base_length: int
    min_length: int
    max_length: int
    stretch_terrain: Terrain
    stretch_features: list[Feature] = Field(default_factory=list)


class ConnectorHex(BaseModel):
    """A single hex within a connector collection."""

    index: int
    terrain: Terrain
    elevation: float
    moisture: float
    temperature: float = 15.0
    resources: list[Resource] = Field(default_factory=list)
    features: list[Feature] = Field(default_factory=list)
    species_fitness: SpeciesFitness

    connects_to: list[int] = Field(default_factory=list)
    is_entry_point: bool = False
    is_slot: bool = False


class ConnectorCollection(BaseModel):
    """A geographic feature that connects clusters."""

    id: str
    version: int = 1

    type: ConnectorType
    tags: list[str] = Field(default_factory=list)

    hexes: list[ConnectorHex] = Field(default_factory=list)
    base_length: int = 0

    entry_points: list[EntryPoint] = Field(default_factory=list)
    internal_graph: dict[int, list[int]] = Field(default_factory=dict)

    elastic_segments: list[ElasticSegment] = Field(default_factory=list)
    min_total_length: int = 1
    max_total_length: int = 100

    minor_slots: list[MinorAnchorSlot] = Field(default_factory=list)

    requires_terrain_nearby: list[Terrain] = Field(default_factory=list)
    provides_terrain: list[Terrain] = Field(default_factory=list)
    elevation_range: tuple[float, float] = (0, 5000)

    name_fragment: str = ""
    narrative_hooks: list[str] = Field(default_factory=list)

    quality_score: float = Field(ge=0.0, le=10.0, default=0.0)
```

**Step 3: Create schemas/minor.py**

```python
"""Minor anchor schemas for small features."""

from enum import Enum
from typing import Optional

from pydantic import BaseModel, Field

from .base import Terrain, Species, Resource, Feature


class MinorCategory(str, Enum):
    # Rest/services
    INN = "inn"
    TAVERN = "tavern"
    WAYSTATION = "waystation"
    CAMP = "camp"
    CARAVANSERAI = "caravanserai"
    # Crossings
    BRIDGE_WOOD = "bridge_wood"
    BRIDGE_STONE = "bridge_stone"
    BRIDGE_DWARF = "bridge_dwarf"
    FORD_IMPROVED = "ford_improved"
    FERRY = "ferry"
    TUNNEL = "tunnel"
    # Sacred
    SHRINE = "shrine"
    TEMPLE_SMALL = "temple_small"
    STANDING_STONE = "standing_stone"
    SACRED_SPRING = "sacred_spring"
    MEMORIAL = "memorial"
    # Military
    WATCHTOWER = "watchtower"
    TOLL_GATE = "toll_gate"
    BORDER_POST = "border_post"
    SIGNAL_TOWER = "signal_tower"
    # Economic
    MARKET_SMALL = "market_small"
    MILL = "mill"
    MINE_ENTRANCE = "mine_entrance"
    LUMBER_CAMP = "lumber_camp"
    FISHING_VILLAGE = "fishing_village"
    # Mysterious
    HERMIT_HUT = "hermit_hut"
    WITCH_COTTAGE = "witch_cottage"
    ABANDONED_CAMP = "abandoned_camp"
    UNMARKED_GRAVES = "unmarked_graves"
    ANCIENT_MARKER = "ancient_marker"


class MinorAnchor(BaseModel):
    """A small feature that slots into connector collections."""

    id: str
    version: int = 1

    category: MinorCategory
    tags: list[str] = Field(default_factory=list)

    slot_contexts: list[str] = Field(default_factory=list)
    compatible_terrain: list[Terrain] = Field(default_factory=list)
    species_preference: Optional[Species] = None

    name_fragment: str
    narrative_hook: str
    features_added: list[Feature] = Field(default_factory=list)
    resources_added: list[Resource] = Field(default_factory=list)

    provides_rest: bool = False
    provides_trade: bool = False
    provides_information: bool = False
    blocks_passage: bool = False
    danger_level: float = 0.0

    quality_score: float = Field(ge=0.0, le=10.0, default=0.0)
```

**Step 4: Create schemas/seed.py**

```python
"""World seed schemas."""

from typing import Optional

from pydantic import BaseModel, Field


class ClusterPlacement(BaseModel):
    """A cluster to place in the world."""

    template_id: str
    instance_id: str

    region_hint: Optional[str] = None
    terrain_context: Optional[str] = None

    component_overrides: dict[str, list[str]] = Field(default_factory=dict)
    name_override: Optional[str] = None


class ConnectorAssignment(BaseModel):
    """How a connector is used in a seed."""

    collection_id: str
    instance_id: str

    start_cluster: Optional[str] = None
    start_port: Optional[str] = None
    end_cluster: Optional[str] = None
    end_port: Optional[str] = None

    region_hint: Optional[str] = None

    slot_assignments: dict[str, str] = Field(default_factory=dict)


class LayoutHint(BaseModel):
    """Additional placement guidance."""

    cluster_id: str
    hints: list[str] = Field(default_factory=list)


class WorldSeed(BaseModel):
    """A recipe for assembling a world."""

    seed_id: int
    version: int = 1
    name: Optional[str] = None

    clusters: list[ClusterPlacement] = Field(default_factory=list)
    connectors: list[ConnectorAssignment] = Field(default_factory=list)

    layout_hints: list[LayoutHint] = Field(default_factory=list)

    world_radius: int = 150
    climate_band: str = "temperate"

    theme_tags: list[str] = Field(default_factory=list)
```

**Step 5: Create schemas/world.py**

```python
"""World assembly output schemas."""

from typing import Optional

from pydantic import BaseModel, Field

from .base import Terrain, Resource, Feature, SpeciesFitness, HexCoord
from .template import AssembledCluster


class WorldHex(BaseModel):
    """A single hex in the assembled world."""

    coord: HexCoord
    terrain: Terrain
    elevation: float
    moisture: float
    temperature: float
    resources: list[Resource] = Field(default_factory=list)
    features: list[Feature] = Field(default_factory=list)
    species_fitness: SpeciesFitness
    cluster_id: Optional[str] = None
    component_id: Optional[str] = None


class HexMap(BaseModel):
    """The complete hex map of an assembled world."""

    seed_id: int
    world_radius: int
    hexes: dict[str, WorldHex] = Field(default_factory=dict)  # "q,r" -> WorldHex
    clusters: dict[str, AssembledCluster] = Field(default_factory=dict)
    cluster_positions: dict[str, tuple[int, int]] = Field(default_factory=dict)


class AssembledWorld(BaseModel):
    """Complete assembled world with metadata."""

    seed_id: int
    name: Optional[str] = None
    hex_map: HexMap
    generation_time_ms: int = 0
    total_hexes: int = 0
    total_clusters: int = 0
```

**Step 6: Update schemas/__init__.py with all exports**

```python
"""Pydantic schemas for worldgen."""

from .base import (
    Terrain,
    ResourceType,
    Abundance,
    FeatureType,
    Species,
    Resource,
    Feature,
    SpeciesFitness,
    HexCoord,
    generate_stable_id,
)
from .component import Component, ComponentCategory, ConnectionPoint
from .template import (
    SlotCount,
    ClusterSlot,
    InternalConnection,
    ExternalPort,
    ClusterTemplate,
    AssembledCluster,
)
from .connector import (
    ConnectorType,
    EntryPoint,
    MinorAnchorSlot,
    ElasticSegment,
    ConnectorHex,
    ConnectorCollection,
)
from .minor import MinorCategory, MinorAnchor
from .seed import ClusterPlacement, ConnectorAssignment, LayoutHint, WorldSeed
from .world import WorldHex, HexMap, AssembledWorld

__all__ = [
    # base
    "Terrain",
    "ResourceType",
    "Abundance",
    "FeatureType",
    "Species",
    "Resource",
    "Feature",
    "SpeciesFitness",
    "HexCoord",
    "generate_stable_id",
    # component
    "Component",
    "ComponentCategory",
    "ConnectionPoint",
    # template
    "SlotCount",
    "ClusterSlot",
    "InternalConnection",
    "ExternalPort",
    "ClusterTemplate",
    "AssembledCluster",
    # connector
    "ConnectorType",
    "EntryPoint",
    "MinorAnchorSlot",
    "ElasticSegment",
    "ConnectorHex",
    "ConnectorCollection",
    # minor
    "MinorCategory",
    "MinorAnchor",
    # seed
    "ClusterPlacement",
    "ConnectorAssignment",
    "LayoutHint",
    "WorldSeed",
    # world
    "WorldHex",
    "HexMap",
    "AssembledWorld",
]
```

**Step 7: Run tests**

Run: `cd /home/astre/arc-citadel/worldgen && python -m pytest tests/ -v`

Expected: All tests pass

**Step 8: Commit**

```bash
git add worldgen/schemas/
git commit -m "feat(worldgen): add template, connector, minor, seed, world schemas"
```

---

## Task 5: SQLite Database

**Files:**
- Create: `worldgen/storage/__init__.py`
- Create: `worldgen/storage/database.py`
- Create: `worldgen/tests/test_storage.py`

**Step 1: Write failing test for database**

Create `worldgen/tests/test_storage.py`:
```python
"""Tests for SQLite storage."""

import tempfile
from pathlib import Path

import pytest

from worldgen.storage.database import Database
from worldgen.schemas import (
    Component,
    ComponentCategory,
    Species,
    Terrain,
    SpeciesFitness,
)


@pytest.fixture
def temp_db():
    """Create a temporary database."""
    with tempfile.TemporaryDirectory() as tmpdir:
        db_path = Path(tmpdir) / "test.db"
        db = Database(db_path)
        db.init()
        yield db


class TestDatabase:
    def test_init_creates_tables(self, temp_db):
        # Tables should exist after init
        tables = temp_db.list_tables()
        assert "components" in tables
        assert "connectors" in tables
        assert "minors" in tables

    def test_insert_and_get_component(self, temp_db):
        comp = Component(
            id="test_comp_001",
            category=ComponentCategory.DWARF_HOLD_FORGE,
            tags=["dwarf", "forge", "test"],
            species=Species.DWARF,
            terrain=Terrain.UNDERGROUND,
            elevation=500.0,
            moisture=0.2,
            temperature=25.0,
            species_fitness=SpeciesFitness(human=0.3, dwarf=0.95, elf=0.1),
            name_fragment="Test Forge",
            narrative_hook="A test forge.",
            quality_score=8.0,
        )
        temp_db.insert_component(comp)

        retrieved = temp_db.get_component("test_comp_001")
        assert retrieved is not None
        assert retrieved.id == "test_comp_001"
        assert retrieved.category == ComponentCategory.DWARF_HOLD_FORGE

    def test_query_components_by_category(self, temp_db):
        # Insert two components
        for i in range(2):
            comp = Component(
                id=f"forge_{i}",
                category=ComponentCategory.DWARF_HOLD_FORGE,
                tags=["dwarf"],
                species=Species.DWARF,
                terrain=Terrain.UNDERGROUND,
                elevation=500.0,
                moisture=0.2,
                temperature=25.0,
                species_fitness=SpeciesFitness(human=0.3, dwarf=0.9, elf=0.1),
                name_fragment=f"Forge {i}",
                narrative_hook="A forge.",
                quality_score=8.0,
            )
            temp_db.insert_component(comp)

        results = temp_db.query_components(category=ComponentCategory.DWARF_HOLD_FORGE)
        assert len(results) == 2

    def test_get_stats(self, temp_db):
        stats = temp_db.get_stats()
        assert "components" in stats
        assert stats["components"] == 0
```

**Step 2: Run test to verify it fails**

Run: `cd /home/astre/arc-citadel/worldgen && python -m pytest tests/test_storage.py -v`

Expected: ModuleNotFoundError

**Step 3: Create storage/__init__.py**

```python
"""Storage layer for worldgen assets."""

from .database import Database

__all__ = ["Database"]
```

**Step 4: Create storage/database.py**

```python
"""SQLite database for asset storage."""

import json
import sqlite3
from pathlib import Path
from typing import Optional

from worldgen.schemas import (
    Component,
    ComponentCategory,
    ConnectorCollection,
    ConnectorType,
    MinorAnchor,
    MinorCategory,
)


class Database:
    """SQLite database for worldgen assets."""

    def __init__(self, db_path: Path):
        self.db_path = db_path
        self._conn: Optional[sqlite3.Connection] = None

    @property
    def conn(self) -> sqlite3.Connection:
        if self._conn is None:
            self.db_path.parent.mkdir(parents=True, exist_ok=True)
            self._conn = sqlite3.connect(self.db_path)
            self._conn.row_factory = sqlite3.Row
        return self._conn

    def init(self) -> None:
        """Initialize database schema."""
        self.conn.executescript("""
            CREATE TABLE IF NOT EXISTS components (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL,
                species TEXT NOT NULL,
                tags TEXT NOT NULL,
                data TEXT NOT NULL,
                quality_score REAL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS connectors (
                id TEXT PRIMARY KEY,
                type TEXT NOT NULL,
                tags TEXT NOT NULL,
                data TEXT NOT NULL,
                quality_score REAL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS minors (
                id TEXT PRIMARY KEY,
                category TEXT NOT NULL,
                tags TEXT NOT NULL,
                data TEXT NOT NULL,
                quality_score REAL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_comp_category ON components(category);
            CREATE INDEX IF NOT EXISTS idx_comp_species ON components(species);
            CREATE INDEX IF NOT EXISTS idx_conn_type ON connectors(type);
            CREATE INDEX IF NOT EXISTS idx_minor_category ON minors(category);
        """)
        self.conn.commit()

    def list_tables(self) -> list[str]:
        """List all tables in the database."""
        cursor = self.conn.execute(
            "SELECT name FROM sqlite_master WHERE type='table'"
        )
        return [row["name"] for row in cursor.fetchall()]

    def insert_component(self, component: Component) -> None:
        """Insert a component into the database."""
        self.conn.execute(
            """
            INSERT OR REPLACE INTO components (id, category, species, tags, data, quality_score)
            VALUES (?, ?, ?, ?, ?, ?)
            """,
            (
                component.id,
                component.category.value,
                component.species.value,
                json.dumps(component.tags),
                component.model_dump_json(),
                component.quality_score,
            ),
        )
        self.conn.commit()

    def get_component(self, component_id: str) -> Optional[Component]:
        """Get a component by ID."""
        cursor = self.conn.execute(
            "SELECT data FROM components WHERE id = ?", (component_id,)
        )
        row = cursor.fetchone()
        if row:
            return Component.model_validate_json(row["data"])
        return None

    def query_components(
        self,
        category: Optional[ComponentCategory] = None,
        species: Optional[str] = None,
        min_quality: Optional[float] = None,
        limit: int = 100,
    ) -> list[Component]:
        """Query components with optional filters."""
        query = "SELECT data FROM components WHERE 1=1"
        params: list = []

        if category:
            query += " AND category = ?"
            params.append(category.value)
        if species:
            query += " AND species = ?"
            params.append(species)
        if min_quality:
            query += " AND quality_score >= ?"
            params.append(min_quality)

        query += " LIMIT ?"
        params.append(limit)

        cursor = self.conn.execute(query, params)
        return [Component.model_validate_json(row["data"]) for row in cursor.fetchall()]

    def insert_connector(self, connector: ConnectorCollection) -> None:
        """Insert a connector into the database."""
        self.conn.execute(
            """
            INSERT OR REPLACE INTO connectors (id, type, tags, data, quality_score)
            VALUES (?, ?, ?, ?, ?)
            """,
            (
                connector.id,
                connector.type.value,
                json.dumps(connector.tags),
                connector.model_dump_json(),
                connector.quality_score,
            ),
        )
        self.conn.commit()

    def get_connector(self, connector_id: str) -> Optional[ConnectorCollection]:
        """Get a connector by ID."""
        cursor = self.conn.execute(
            "SELECT data FROM connectors WHERE id = ?", (connector_id,)
        )
        row = cursor.fetchone()
        if row:
            return ConnectorCollection.model_validate_json(row["data"])
        return None

    def insert_minor(self, minor: MinorAnchor) -> None:
        """Insert a minor anchor into the database."""
        self.conn.execute(
            """
            INSERT OR REPLACE INTO minors (id, category, tags, data, quality_score)
            VALUES (?, ?, ?, ?, ?)
            """,
            (
                minor.id,
                minor.category.value,
                json.dumps(minor.tags),
                minor.model_dump_json(),
                minor.quality_score,
            ),
        )
        self.conn.commit()

    def get_minor(self, minor_id: str) -> Optional[MinorAnchor]:
        """Get a minor anchor by ID."""
        cursor = self.conn.execute(
            "SELECT data FROM minors WHERE id = ?", (minor_id,)
        )
        row = cursor.fetchone()
        if row:
            return MinorAnchor.model_validate_json(row["data"])
        return None

    def get_stats(self) -> dict:
        """Get database statistics."""
        stats = {}
        for table in ["components", "connectors", "minors"]:
            cursor = self.conn.execute(f"SELECT COUNT(*) as count FROM {table}")
            stats[table] = cursor.fetchone()["count"]
        return stats

    def close(self) -> None:
        """Close database connection."""
        if self._conn:
            self._conn.close()
            self._conn = None
```

**Step 5: Run tests**

Run: `cd /home/astre/arc-citadel/worldgen && python -m pytest tests/test_storage.py -v`

Expected: All 4 tests pass

**Step 6: Commit**

```bash
git add worldgen/storage/
git commit -m "feat(worldgen): add SQLite database storage"
```

---

## Task 6: LLM Client and Quality Loop

**Files:**
- Create: `worldgen/generation/__init__.py`
- Create: `worldgen/generation/llm_client.py`
- Create: `worldgen/generation/quality_loop.py`
- Create: `worldgen/generation/prompts/` directory
- Create: `worldgen/generation/prompts/scoring.txt`

**Step 1: Create generation/__init__.py**

```python
"""Asset generation using LLM."""

from .llm_client import LLMClient
from .quality_loop import QualityGenerator

__all__ = ["LLMClient", "QualityGenerator"]
```

**Step 2: Create generation/llm_client.py**

```python
"""DeepSeek LLM client wrapper."""

import json
from typing import Optional

from openai import OpenAI

from worldgen import config


class LLMClient:
    """Wrapper for DeepSeek API via OpenAI-compatible client."""

    def __init__(
        self,
        api_key: Optional[str] = None,
        base_url: Optional[str] = None,
        model: Optional[str] = None,
    ):
        self.api_key = api_key or config.DEEPSEEK_API_KEY
        self.base_url = base_url or config.DEEPSEEK_BASE_URL
        self.model = model or config.DEEPSEEK_MODEL

        if not self.api_key:
            raise ValueError("DEEPSEEK_API_KEY not set")

        self.client = OpenAI(api_key=self.api_key, base_url=self.base_url)

    def generate(
        self,
        prompt: str,
        system_prompt: str = "You are a world generator for Arc Citadel. Output ONLY valid JSON, no markdown.",
        temperature: float = 0.9,
        max_tokens: int = 2000,
    ) -> str:
        """Generate content from prompt."""
        response = self.client.chat.completions.create(
            model=self.model,
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": prompt},
            ],
            temperature=temperature,
            max_tokens=max_tokens,
        )
        return response.choices[0].message.content or ""

    def generate_json(
        self,
        prompt: str,
        system_prompt: str = "You are a world generator for Arc Citadel. Output ONLY valid JSON, no markdown.",
        temperature: float = 0.9,
        max_tokens: int = 2000,
    ) -> dict:
        """Generate and parse JSON response."""
        content = self.generate(prompt, system_prompt, temperature, max_tokens)
        return self._clean_and_parse_json(content)

    def _clean_and_parse_json(self, content: str) -> dict:
        """Clean up common JSON issues from LLM output."""
        content = content.strip()

        # Remove markdown code blocks
        if content.startswith("```"):
            lines = content.split("\n")
            if lines[-1].strip() == "```":
                lines = lines[1:-1]
            else:
                lines = lines[1:]
            content = "\n".join(lines)

        # Remove json prefix
        if content.startswith("json"):
            content = content[4:]

        content = content.strip()
        return json.loads(content)
```

**Step 3: Create generation/quality_loop.py**

```python
"""Quality-focused generation with iterative improvement."""

import json
from pathlib import Path
from typing import Optional

from worldgen import config
from worldgen.schemas import ComponentCategory
from .llm_client import LLMClient


class QualityGenerator:
    """Generate assets with iterative quality improvement."""

    def __init__(
        self,
        target_score: float = config.DEFAULT_TARGET_SCORE,
        max_iterations: int = config.MAX_QUALITY_ITERATIONS,
        candidates_per_round: int = config.CANDIDATES_PER_ROUND,
        client: Optional[LLMClient] = None,
    ):
        self.target_score = target_score
        self.max_iterations = max_iterations
        self.candidates_per_round = candidates_per_round
        self.client = client or LLMClient()

        self.stats = {
            "total_generated": 0,
            "total_iterations": 0,
            "avg_final_score": 0.0,
        }

    def _load_scoring_prompt(self) -> str:
        """Load the universal scoring prompt."""
        scoring_path = config.PROMPTS_DIR / "scoring.txt"
        if scoring_path.exists():
            return scoring_path.read_text()
        return self._default_scoring_prompt()

    def _default_scoring_prompt(self) -> str:
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
    ) -> dict:
        """Score an asset using DeepSeek."""
        prompt_template = self._load_scoring_prompt()
        prompt = prompt_template.format(
            asset_type=asset_type,
            species=species,
            asset_json=json.dumps(asset, indent=2),
        )

        return self.client.generate_json(
            prompt=prompt,
            system_prompt="You are a harsh but fair game design critic. A 9/10 is genuinely excellent. Most things are 5-7.",
            temperature=0.3,
            max_tokens=500,
        )

    def generate_with_quality(
        self,
        prompt_template: str,
        asset_type: str,
        species: str,
    ) -> tuple[Optional[dict], Optional[dict]]:
        """Generate candidates until one reaches target quality."""
        best: Optional[dict] = None
        best_score_data: Optional[dict] = None
        best_score = 0.0

        current_prompt = prompt_template

        for iteration in range(self.max_iterations):
            # Generate candidates
            candidates = []
            for _ in range(self.candidates_per_round):
                try:
                    candidate = self.client.generate_json(current_prompt)
                    score_data = self.score_asset(candidate, asset_type, species)
                    score = score_data.get("overall_score", 0)
                    candidates.append((candidate, score_data, score))
                except Exception as e:
                    print(f"    Generation failed: {e}")
                    continue

            if not candidates:
                continue

            # Find best
            round_best = max(candidates, key=lambda x: x[2])

            if round_best[2] > best_score:
                best, best_score_data, best_score = round_best

            # Check if target reached
            if best_score >= self.target_score:
                self.stats["total_generated"] += 1
                self.stats["total_iterations"] += iteration + 1
                return best, best_score_data

            # Build improvement prompt
            current_prompt = self._improvement_prompt(
                original_prompt=prompt_template,
                score=best_score,
                score_data=best_score_data,
            )

        # Return best we got
        self.stats["total_generated"] += 1
        self.stats["total_iterations"] += self.max_iterations
        return best, best_score_data

    def _improvement_prompt(
        self, original_prompt: str, score: float, score_data: dict
    ) -> str:
        """Build prompt for improvement iteration."""
        return f"""The previous version scored {score}/10.

FEEDBACK:
Strengths: {", ".join(score_data.get("strengths", []))}
Weaknesses: {", ".join(score_data.get("weaknesses", []))}
Suggestions: {", ".join(score_data.get("improvement_suggestions", []))}

Generate an IMPROVED version that:
1. Keeps all the strengths
2. Fixes all the weaknesses
3. Implements the suggestions
4. Targets 9/10 or higher

{original_prompt}"""

    def generate_component(
        self, category: ComponentCategory, prompt_template: str, index: int
    ) -> Optional[dict]:
        """Generate a single component with quality iteration."""
        species = category.value.split("_")[0]
        asset_type = f"{species} {category.value.replace('_', ' ')}"

        result, score_data = self.generate_with_quality(
            prompt_template=prompt_template,
            asset_type=asset_type,
            species=species,
        )

        if result and score_data:
            result["quality_score"] = score_data.get("overall_score", 0)
            result["generation_notes"] = f"Strengths: {score_data.get('strengths', [])}"

        return result
```

**Step 4: Create prompts directory and scoring.txt**

```bash
mkdir -p /home/astre/arc-citadel/worldgen/generation/prompts
```

Create `worldgen/generation/prompts/scoring.txt`:
```
Rate this {asset_type} from 1-10. Be HARSH. A 7 is good. A 9 is excellent.

STRATEGIC VALUE (1-10):
- Does it create interesting choices?
- Is it defensible/contestable/valuable?
- Does it have multiple viable uses?
- Score 1-3 if generic, 4-6 if solid, 7-9 if excellent

NARRATIVE POTENTIAL (1-10):
- Does it imply history beyond what's stated?
- Does it create questions players want answered?
- Is it memorable and distinctive?
- Score 1-3 if generic, 4-6 if interesting, 7-9 if compelling

SPECIES AUTHENTICITY (1-10):
- Does it feel GENUINELY {species}?
- NOT a human with cosmetic differences?
- Does it reflect the species' actual values?
  - Dwarf: CRAFT-TRUTH, STONE-DEBT, GRUDGE-BALANCE, OATH-CHAIN
  - Elf: PATTERN-BEAUTY, SLOW-GROWTH, MEMORY-WEIGHT, CHANGE-GRIEF
  - Human: HONOR, AMBITION, LOYALTY, PIETY
- Score 1-3 if human-with-hat, 4-6 if somewhat alien, 7-9 if genuinely different

SENSORY RICHNESS (1-10):
- Can you FEEL being there?
- Specific sights, sounds, smells?
- Not just "it's dark" but "the air tastes of iron and old smoke"?
- Score 1-3 if vague, 4-6 if decent, 7-9 if immersive

ASSET:
{asset_json}

Respond with ONLY this JSON:
{{
  "strategic_score": <1-10>,
  "narrative_score": <1-10>,
  "authenticity_score": <1-10>,
  "sensory_score": <1-10>,
  "overall_score": <1-10>,
  "strengths": ["strength 1", "strength 2"],
  "weaknesses": ["weakness 1", "weakness 2"],
  "improvement_suggestions": ["suggestion 1", "suggestion 2"]
}}
```

**Step 5: Commit**

```bash
git add worldgen/generation/
git commit -m "feat(worldgen): add LLM client and quality loop"
```

---

## Task 7: Template Loader

**Files:**
- Create: `worldgen/templates/__init__.py`
- Create: `worldgen/templates/template_loader.py`
- Create: `worldgen/templates/dwarf/hold_major.yaml`
- Create: `worldgen/tests/test_templates.py`

**Step 1: Write failing test**

Create `worldgen/tests/test_templates.py`:
```python
"""Tests for template loading."""

from pathlib import Path
import tempfile

import pytest

from worldgen.templates.template_loader import TemplateLoader
from worldgen.schemas import ClusterTemplate, Species


class TestTemplateLoader:
    def test_load_single_template(self):
        loader = TemplateLoader()
        template = loader.load_template("dwarf/hold_major")
        assert template is not None
        assert template.id == "dwarf_hold_major"
        assert template.species == Species.DWARF

    def test_load_all_templates(self):
        loader = TemplateLoader()
        templates = loader.load_all()
        assert len(templates) >= 1
        assert "dwarf_hold_major" in templates
```

**Step 2: Run test to verify it fails**

Run: `cd /home/astre/arc-citadel/worldgen && python -m pytest tests/test_templates.py -v`

Expected: ModuleNotFoundError

**Step 3: Create templates/__init__.py**

```python
"""Cluster template management."""

from .template_loader import TemplateLoader

__all__ = ["TemplateLoader"]
```

**Step 4: Create templates/template_loader.py**

```python
"""Load and validate cluster templates from YAML."""

from pathlib import Path
from typing import Optional

import yaml

from worldgen import config
from worldgen.schemas import ClusterTemplate


class TemplateLoader:
    """Load cluster templates from YAML files."""

    def __init__(self, templates_dir: Optional[Path] = None):
        self.templates_dir = templates_dir or config.TEMPLATES_DIR

    def load_template(self, template_path: str) -> ClusterTemplate:
        """Load a single template by path (e.g., 'dwarf/hold_major')."""
        yaml_path = self.templates_dir / f"{template_path}.yaml"
        if not yaml_path.exists():
            raise FileNotFoundError(f"Template not found: {yaml_path}")

        with open(yaml_path) as f:
            data = yaml.safe_load(f)

        return ClusterTemplate.model_validate(data)

    def load_all(self) -> dict[str, ClusterTemplate]:
        """Load all templates from the templates directory."""
        templates = {}

        for yaml_file in self.templates_dir.rglob("*.yaml"):
            try:
                with open(yaml_file) as f:
                    data = yaml.safe_load(f)

                template = ClusterTemplate.model_validate(data)
                templates[template.id] = template
            except Exception as e:
                print(f"Failed to load {yaml_file}: {e}")

        return templates

    def validate_template(
        self, template: ClusterTemplate, component_categories: set[str]
    ) -> list[str]:
        """Validate a template against available component categories."""
        errors = []

        for slot in template.slots:
            if slot.component_category.value not in component_categories:
                errors.append(
                    f"Unknown component category: {slot.component_category.value}"
                )

        slot_ids = {s.slot_id for s in template.slots}
        for conn in template.internal_connections:
            if conn.from_slot not in slot_ids:
                errors.append(f"Unknown slot in connection: {conn.from_slot}")
            if conn.to_slot not in slot_ids:
                errors.append(f"Unknown slot in connection: {conn.to_slot}")

        return errors
```

**Step 5: Create templates/dwarf/ directory and hold_major.yaml**

```bash
mkdir -p /home/astre/arc-citadel/worldgen/templates/dwarf
```

Create `worldgen/templates/dwarf/hold_major.yaml`:
```yaml
id: dwarf_hold_major
name: Major Dwarf Hold
version: 1
species: dwarf
tags:
  - settlement
  - military
  - trade
  - underground
description: |
  A great dwarven hold carved deep into the mountain. These are the seats of
  dwarf clans, containing forges, ancestor halls, treasuries, and extensive
  mine works.

slots:
  - slot_id: entrance_main
    component_category: dwarf_hold_entrance
    required: true
    count:
      min: 1
      max: 2
    depth_range: [0, 0]
    edge_allowed: true
    center_preference: 0.0
    required_tags:
      - main_gate

  - slot_id: forge_complex
    component_category: dwarf_hold_forge
    required: true
    count:
      min: 2
      max: 4
    depth_range: [2, 5]
    center_preference: 0.7

  - slot_id: tavern
    component_category: dwarf_hold_tavern
    required: true
    count:
      min: 1
      max: 3
    depth_range: [1, 3]
    adjacent_to:
      - forge_complex

  - slot_id: living_quarters
    component_category: dwarf_hold_living_quarters
    required: true
    count:
      min: 3
      max: 6
    depth_range: [2, 5]

internal_connections:
  - from_slot: entrance_main
    to_slot: tavern
    connector_category: dwarf_hold_tunnel
    required: true
    max_length: 4

  - from_slot: tavern
    to_slot: forge_complex
    connector_category: dwarf_hold_tunnel
    required: true
    max_length: 3

external_ports:
  - port_id: main_gate
    attached_slot: entrance_main
    direction_preference:
      - S
      - SE
      - SW
    compatible_connectors:
      - trade_route
      - military_road
    required: true

min_footprint: 25
max_footprint: 60

terrain_requirements:
  - requires_mountains

min_distance_same_type: 30

history_seeds:
  - "Founded after a great migration from a fallen hold"
  - "Built atop an ancient mine that predates the current clan"
  - "Withstood a siege that lasted seven years"
```

**Step 6: Run tests**

Run: `cd /home/astre/arc-citadel/worldgen && python -m pytest tests/test_templates.py -v`

Expected: All tests pass

**Step 7: Commit**

```bash
git add worldgen/templates/
git commit -m "feat(worldgen): add template loader with dwarf hold template"
```

---

## Task 8: CLI Interface

**Files:**
- Create: `worldgen/cli.py`
- Create: `worldgen/tests/test_cli.py`

**Step 1: Write failing test**

Create `worldgen/tests/test_cli.py`:
```python
"""Tests for CLI interface."""

from click.testing import CliRunner

from worldgen.cli import cli


class TestCLI:
    def test_cli_exists(self):
        runner = CliRunner()
        result = runner.invoke(cli, ["--help"])
        assert result.exit_code == 0
        assert "Arc Citadel World Generation Pipeline" in result.output

    def test_init_command(self):
        runner = CliRunner()
        with runner.isolated_filesystem():
            result = runner.invoke(cli, ["init", "--output", "test_output"])
            assert result.exit_code == 0
            assert "Initialized" in result.output

    def test_stats_command(self):
        runner = CliRunner()
        with runner.isolated_filesystem():
            # Init first
            runner.invoke(cli, ["init", "--output", "test_output"])
            result = runner.invoke(cli, ["stats", "--db", "test_output/libraries/assets.db"])
            assert result.exit_code == 0
```

**Step 2: Run test to verify it fails**

Run: `cd /home/astre/arc-citadel/worldgen && python -m pytest tests/test_cli.py -v`

Expected: ModuleNotFoundError or ImportError

**Step 3: Create cli.py**

```python
"""CLI interface for worldgen pipeline."""

from pathlib import Path

import click

from worldgen import config
from worldgen.storage import Database


@click.group()
def cli():
    """Arc Citadel World Generation Pipeline"""
    pass


@cli.command()
@click.option("--output", default="output", help="Output directory")
def init(output: str):
    """Initialize output directory structure and database."""
    output_path = Path(output)

    dirs = [
        output_path / "libraries",
        output_path / "seeds",
        output_path / "worlds",
        output_path / "logs",
    ]

    for d in dirs:
        d.mkdir(parents=True, exist_ok=True)
        click.echo(f"Created {d}")

    # Initialize database
    db_path = output_path / "libraries" / "assets.db"
    db = Database(db_path)
    db.init()
    db.close()
    click.echo(f"Initialized database at {db_path}")

    click.echo("Initialized output directories")


@cli.command()
@click.option("--db", default=None, help="Database path")
def stats(db: str):
    """Show library statistics."""
    if db:
        db_path = Path(db)
    else:
        db_path = config.DATABASE_PATH

    if not db_path.exists():
        click.echo(f"Database not found at {db_path}")
        click.echo("Run 'worldgen init' first")
        return

    database = Database(db_path)
    stats_data = database.get_stats()
    database.close()

    click.echo("Asset Library Statistics:")
    click.echo(f"  Components: {stats_data.get('components', 0)}")
    click.echo(f"  Connectors: {stats_data.get('connectors', 0)}")
    click.echo(f"  Minor anchors: {stats_data.get('minors', 0)}")


@cli.group()
def generate():
    """Generate asset libraries."""
    pass


@generate.command("components")
@click.option("--target-score", default=9.0, help="Minimum quality score")
@click.option("--count", default=100, help="Components per category")
@click.option("--category", default=None, help="Specific category to generate")
@click.option("--db", default=None, help="Database path")
def generate_components(target_score: float, count: int, category: str, db: str):
    """Generate component library using DeepSeek."""
    click.echo(f"Generating components (target score: {target_score})")
    click.echo("Note: Requires DEEPSEEK_API_KEY environment variable")

    if db:
        db_path = Path(db)
    else:
        db_path = config.DATABASE_PATH

    if not db_path.exists():
        click.echo(f"Database not found at {db_path}")
        click.echo("Run 'worldgen init' first")
        return

    # TODO: Implement actual generation
    click.echo("Component generation not yet implemented")
    click.echo(f"Would generate {count} components per category")


if __name__ == "__main__":
    cli()
```

**Step 4: Run tests**

Run: `cd /home/astre/arc-citadel/worldgen && python -m pytest tests/test_cli.py -v`

Expected: All tests pass

**Step 5: Commit**

```bash
git add worldgen/cli.py worldgen/tests/test_cli.py
git commit -m "feat(worldgen): add CLI with init and stats commands"
```

---

## Task 9: Seed Generator (Stub)

**Files:**
- Create: `worldgen/seeds/__init__.py`
- Create: `worldgen/seeds/seed_generator.py`

**Step 1: Create seeds/__init__.py**

```python
"""Seed generation for world assembly."""

from .seed_generator import SeedGenerator

__all__ = ["SeedGenerator"]
```

**Step 2: Create seeds/seed_generator.py**

```python
"""Generate valid world seeds."""

import random
from typing import Optional

from worldgen.schemas import (
    WorldSeed,
    ClusterPlacement,
    ConnectorAssignment,
    LayoutHint,
)


class SeedGenerator:
    """Generate valid world seeds from templates."""

    def __init__(self, available_templates: list[str]):
        self.available_templates = available_templates

    def generate_seed(
        self,
        seed_id: int,
        num_dwarf_holds: int = 3,
        num_elf_groves: int = 3,
        num_human_cities: int = 5,
        world_radius: int = 150,
    ) -> Optional[WorldSeed]:
        """Generate a single valid seed."""
        rng = random.Random(seed_id)

        clusters = []

        # Dwarf holds
        for i in range(num_dwarf_holds):
            template_id = "dwarf_hold_major"  # Use available template
            if template_id in self.available_templates:
                clusters.append(
                    ClusterPlacement(
                        template_id=template_id,
                        instance_id=f"dwarf_{i}",
                        region_hint=rng.choice(["N", "NE", "NW"]),
                    )
                )

        # Generate layout hints
        layout_hints = []
        for c in clusters:
            if c.template_id.startswith("dwarf_"):
                layout_hints.append(
                    LayoutHint(
                        cluster_id=c.instance_id,
                        hints=["terrain:mountains", "region:north"],
                    )
                )

        return WorldSeed(
            seed_id=seed_id,
            clusters=clusters,
            connectors=[],
            layout_hints=layout_hints,
            world_radius=world_radius,
        )
```

**Step 3: Commit**

```bash
git add worldgen/seeds/
git commit -m "feat(worldgen): add seed generator stub"
```

---

## Task 10: Assembly Stubs

**Files:**
- Create: `worldgen/assembly/__init__.py`
- Create: `worldgen/assembly/assembler.py`
- Create: `worldgen/assembly/layout_solver.py`

**Step 1: Create assembly/__init__.py**

```python
"""World assembly from seeds."""

from .assembler import WorldAssembler

__all__ = ["WorldAssembler"]
```

**Step 2: Create assembly/layout_solver.py**

```python
"""Solve cluster placement on hex grid."""

import random
from typing import Optional

from worldgen.schemas import AssembledCluster, LayoutHint


class LayoutSolver:
    """Solve cluster placement via constraint satisfaction."""

    def __init__(self):
        self.max_attempts = 1000

    def solve(
        self,
        clusters: dict[str, AssembledCluster],
        hints: list[LayoutHint],
        world_radius: int,
        rng: random.Random,
    ) -> dict[str, tuple[int, int]]:
        """Find valid positions for all clusters."""
        positions = {}
        placed_footprints: set[tuple[int, int]] = set()

        for instance_id, cluster in clusters.items():
            hint = next((h for h in hints if h.cluster_id == instance_id), None)

            position = self._find_valid_position(
                cluster=cluster,
                hint=hint,
                placed=placed_footprints,
                world_radius=world_radius,
                rng=rng,
            )

            if position is None:
                raise ValueError(f"Could not place cluster {instance_id}")

            positions[instance_id] = position

            # Add footprint to placed set
            for offset in cluster.footprint:
                placed_footprints.add((position[0] + offset[0], position[1] + offset[1]))

        return positions

    def _find_valid_position(
        self,
        cluster: AssembledCluster,
        hint: Optional[LayoutHint],
        placed: set[tuple[int, int]],
        world_radius: int,
        rng: random.Random,
    ) -> Optional[tuple[int, int]]:
        """Find a valid position for a cluster."""
        for _ in range(self.max_attempts):
            q = rng.randint(-world_radius, world_radius)
            r = rng.randint(-world_radius, world_radius)
            if abs(q + r) <= world_radius:
                if self._is_valid_position(cluster, (q, r), placed, world_radius):
                    return (q, r)
        return None

    def _is_valid_position(
        self,
        cluster: AssembledCluster,
        position: tuple[int, int],
        placed: set[tuple[int, int]],
        world_radius: int,
    ) -> bool:
        """Check if a position is valid for a cluster."""
        for offset in cluster.footprint:
            world_q = position[0] + offset[0]
            world_r = position[1] + offset[1]

            if abs(world_q) + abs(world_r) > world_radius:
                return False

            if (world_q, world_r) in placed:
                return False

        return True
```

**Step 3: Create assembly/assembler.py**

```python
"""Main world assembly pipeline."""

import random

from worldgen.schemas import WorldSeed, HexMap, AssembledCluster
from worldgen.storage import Database
from .layout_solver import LayoutSolver


class WorldAssembler:
    """Assemble a complete world from a seed."""

    def __init__(self, database: Database):
        self.database = database
        self.layout_solver = LayoutSolver()

    def assemble(self, seed: WorldSeed) -> HexMap:
        """Assemble a complete world from a seed."""
        rng = random.Random(seed.seed_id)

        # 1. Create assembled clusters (stub - just empty footprints for now)
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

        # 2. Solve global layout
        cluster_positions = self.layout_solver.solve(
            clusters=assembled_clusters,
            hints=seed.layout_hints,
            world_radius=seed.world_radius,
            rng=rng,
        )

        return HexMap(
            seed_id=seed.seed_id,
            world_radius=seed.world_radius,
            hexes={},
            clusters=assembled_clusters,
            cluster_positions=cluster_positions,
        )
```

**Step 4: Commit**

```bash
git add worldgen/assembly/
git commit -m "feat(worldgen): add assembly stubs with layout solver"
```

---

## Task 11: Integration Test

**Files:**
- Create: `worldgen/tests/test_integration.py`

**Step 1: Write integration test**

Create `worldgen/tests/test_integration.py`:
```python
"""Integration tests for end-to-end pipeline."""

import tempfile
from pathlib import Path

import pytest

from worldgen.storage import Database
from worldgen.templates import TemplateLoader
from worldgen.seeds import SeedGenerator
from worldgen.assembly import WorldAssembler
from worldgen.schemas import WorldSeed


class TestEndToEndPipeline:
    def test_seed_to_world_assembly(self):
        """Test complete pipeline: templates -> seed -> assembly."""
        with tempfile.TemporaryDirectory() as tmpdir:
            db_path = Path(tmpdir) / "test.db"

            # 1. Initialize database
            db = Database(db_path)
            db.init()

            # 2. Load templates
            loader = TemplateLoader()
            templates = loader.load_all()
            assert "dwarf_hold_major" in templates

            # 3. Generate seed
            generator = SeedGenerator(list(templates.keys()))
            seed = generator.generate_seed(
                seed_id=42,
                num_dwarf_holds=2,
                num_elf_groves=0,
                num_human_cities=0,
                world_radius=50,
            )

            assert seed is not None
            assert seed.seed_id == 42
            assert len(seed.clusters) == 2

            # 4. Assemble world
            assembler = WorldAssembler(db)
            hex_map = assembler.assemble(seed)

            assert hex_map.seed_id == 42
            assert len(hex_map.clusters) == 2
            assert len(hex_map.cluster_positions) == 2

            # 5. Verify determinism
            hex_map_2 = assembler.assemble(seed)
            assert hex_map.cluster_positions == hex_map_2.cluster_positions

            db.close()

    def test_database_roundtrip(self):
        """Test storing and retrieving components."""
        from worldgen.schemas import (
            Component,
            ComponentCategory,
            Species,
            Terrain,
            SpeciesFitness,
        )

        with tempfile.TemporaryDirectory() as tmpdir:
            db_path = Path(tmpdir) / "test.db"
            db = Database(db_path)
            db.init()

            # Create and store component
            comp = Component(
                id="integration_test_001",
                category=ComponentCategory.DWARF_HOLD_FORGE,
                tags=["dwarf", "forge", "integration"],
                species=Species.DWARF,
                terrain=Terrain.UNDERGROUND,
                elevation=500.0,
                moisture=0.2,
                temperature=25.0,
                species_fitness=SpeciesFitness(human=0.3, dwarf=0.95, elf=0.1),
                name_fragment="Test Forge",
                narrative_hook="Integration test forge.",
                quality_score=8.5,
            )
            db.insert_component(comp)

            # Retrieve and verify
            retrieved = db.get_component("integration_test_001")
            assert retrieved is not None
            assert retrieved.name_fragment == "Test Forge"
            assert retrieved.quality_score == 8.5

            # Query by category
            results = db.query_components(category=ComponentCategory.DWARF_HOLD_FORGE)
            assert len(results) == 1

            db.close()
```

**Step 2: Run integration tests**

Run: `cd /home/astre/arc-citadel/worldgen && python -m pytest tests/test_integration.py -v`

Expected: All tests pass

**Step 3: Run all tests**

Run: `cd /home/astre/arc-citadel/worldgen && python -m pytest tests/ -v`

Expected: All tests pass

**Step 4: Commit**

```bash
git add worldgen/tests/test_integration.py
git commit -m "test(worldgen): add integration tests for pipeline"
```

---

## Task 12: Final Verification

**Step 1: Run full test suite**

```bash
cd /home/astre/arc-citadel/worldgen
python -m pytest tests/ -v --tb=short
```

Expected: All tests pass

**Step 2: Test CLI manually**

```bash
cd /home/astre/arc-citadel/worldgen
python -m worldgen.cli init --output test_output
python -m worldgen.cli stats --db test_output/libraries/assets.db
```

Expected: Commands complete without error

**Step 3: Verify imports work**

```bash
cd /home/astre/arc-citadel/worldgen
python -c "
from worldgen.schemas import *
from worldgen.storage import Database
from worldgen.templates import TemplateLoader
from worldgen.seeds import SeedGenerator
from worldgen.assembly import WorldAssembler
print('All imports successful')
"
```

Expected: "All imports successful"

**Step 4: Final commit**

```bash
git add .
git commit -m "feat(worldgen): complete MVP pipeline structure"
```

---

## Summary

This plan implements the worldgen MVP with:

1. **Schemas**: All Pydantic models for components, templates, connectors, minors, seeds, worlds
2. **Storage**: SQLite database with JSON data storage
3. **Generation**: LLM client and quality loop (ready for DeepSeek integration)
4. **Templates**: YAML loader with dwarf_hold_major template
5. **Seeds**: Basic seed generator
6. **Assembly**: Layout solver and assembler stubs
7. **CLI**: init, stats, generate commands

**Next steps after MVP validation:**
- Implement actual component generation with DeepSeek
- Add more templates (elf, human, ruins)
- Implement connector and minor generation
- Complete assembly logic with component placement
- Add seed batch generation
- Scale to full 100k component library

---

**Plan complete and saved to `docs/plans/2026-01-01-worldgen-asset-pipeline.md`.**

Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

Which approach?