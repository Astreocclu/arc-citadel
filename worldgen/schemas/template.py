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
