"""Tag-based hex schemas for 100m scale composition."""

from enum import Enum
from typing import Optional

from pydantic import BaseModel, Field, field_validator


class EdgeType(str, Enum):
    """Edge connection types for hex borders."""
    TUNNEL = "tunnel"
    ROAD = "road"
    WATER = "water"
    WILDERNESS = "wilderness"
    ENTRANCE = "entrance"
    BLOCKED = "blocked"


class FoundingContext(BaseModel):
    """Founding conditions context for settlement hex generation.

    These conditions come from the astronomical state at founding time
    and influence what features/tags are preferred or avoided.
    """
    season: str = Field(description="Season at founding: spring, summer, autumn, winter, deep_winter")
    astronomical_event: Optional[str] = Field(default=None, description="Active celestial event if any")
    bias_tags: list[str] = Field(default_factory=list, description="Tags to bias toward")
    bias_against: list[str] = Field(default_factory=list, description="Tags to bias against")
    flavor: str = Field(default="", max_length=500, description="Narrative flavor text")

    # Numeric modifiers for downstream processing
    defensive_weight: float = Field(default=0.0, ge=-1.0, le=1.0)
    underground_preference: float = Field(default=0.0, ge=-1.0, le=1.0)
    martial_culture: float = Field(default=0.0, ge=-1.0, le=1.0)
    secrecy_trait: bool = Field(default=False)
    siege_mentality: bool = Field(default=False)


class TaggedHex(BaseModel):
    """A hex with tag-based composition (100m scale)."""
    q: int = Field(description="Axial coordinate q")
    r: int = Field(description="Axial coordinate r")
    name: str = Field(max_length=100, description="Location name")
    description: str = Field(max_length=500, description="100m-scale description")
    tags: list[str] = Field(min_length=1, max_length=10, description="Tags from hex_tags.toml")
    edge_types: list[EdgeType] = Field(min_length=6, max_length=6, description="6 edges clockwise from E")
    scale_score: Optional[float] = Field(default=None, ge=0.0, le=10.0, description="100m scale validation score")

    # Settlement/founding association
    founding_context: Optional[FoundingContext] = Field(
        default=None,
        description="Founding conditions if this hex is part of a settlement"
    )
    founding_cluster_id: Optional[int] = Field(
        default=None,
        description="Unique ID grouping hexes that share founding conditions"
    )

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
