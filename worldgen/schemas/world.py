"""World assembly output schemas."""

from typing import Optional

from pydantic import BaseModel, Field

from .base import Terrain, Resource, Feature, SpeciesFitness, HexCoord
from .template import AssembledCluster
from .tagged import EdgeType


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

    # Tagged hex content (100m scale, LLM-generated)
    name: Optional[str] = Field(default=None, max_length=100, description="Location name")
    description: Optional[str] = Field(default=None, max_length=500, description="100m-scale description")
    tags: list[str] = Field(default_factory=list, description="Tags: culture, function, etc.")
    edge_types: list[EdgeType] = Field(default_factory=list, description="6 edges if tagged")


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
