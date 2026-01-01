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
