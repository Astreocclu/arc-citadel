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
