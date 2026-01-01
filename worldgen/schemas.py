"""Pydantic schemas for hex generation data structures."""

from enum import Enum
from typing import Optional
from pydantic import BaseModel, Field, field_validator


class TerrainType(str, Enum):
    """Valid terrain types for hex tiles."""
    PLAINS = "plains"
    FOREST = "forest"
    HILLS = "hills"
    MOUNTAINS = "mountains"
    WATER = "water"
    SWAMP = "swamp"
    DESERT = "desert"
    TUNDRA = "tundra"


class ResourceType(str, Enum):
    """Valid resource types that can appear in hexes."""
    IRON = "iron"
    COPPER = "copper"
    GOLD = "gold"
    SILVER = "silver"
    COAL = "coal"
    GEMS = "gems"
    TIMBER = "timber"
    STONE = "stone"
    FERTILE_SOIL = "fertile_soil"
    FISH = "fish"
    GAME = "game"
    HERBS = "herbs"


class FeatureType(str, Enum):
    """Special features that can appear in hexes."""
    RUINS = "ruins"
    CAVE = "cave"
    SPRING = "spring"
    ANCIENT_TREE = "ancient_tree"
    STANDING_STONES = "standing_stones"
    RAVINE = "ravine"
    CLEARING = "clearing"
    OVERLOOK = "overlook"


class Resource(BaseModel):
    """A resource deposit within a hex."""
    type: ResourceType
    abundance: float = Field(ge=0.0, le=1.0, description="Resource abundance 0-1")

    @field_validator('abundance')
    @classmethod
    def round_abundance(cls, v: float) -> float:
        return round(v, 2)


class Feature(BaseModel):
    """A special feature within a hex."""
    type: FeatureType
    description: str = Field(max_length=200, description="Brief flavor text")


class HexTile(BaseModel):
    """A single hex tile in the world map."""
    q: int = Field(description="Axial coordinate q")
    r: int = Field(description="Axial coordinate r")
    terrain: TerrainType
    elevation: float = Field(ge=0.0, le=1.0, description="Elevation 0-1")
    moisture: float = Field(ge=0.0, le=1.0, description="Moisture level 0-1")
    resources: list[Resource] = Field(default_factory=list, max_length=3)
    features: list[Feature] = Field(default_factory=list, max_length=2)
    traversable: bool = Field(default=True, description="Can units cross this hex")
    defense_bonus: float = Field(default=0.0, ge=0.0, le=0.5, description="Defensive bonus 0-0.5")

    @field_validator('elevation', 'moisture')
    @classmethod
    def round_values(cls, v: float) -> float:
        return round(v, 2)


class HexRegion(BaseModel):
    """A collection of hexes forming a coherent region."""
    name: str = Field(max_length=50, description="Region name")
    description: str = Field(max_length=500, description="Region flavor description")
    hexes: list[HexTile] = Field(min_length=1, max_length=200)
    theme: str = Field(max_length=100, description="Overall region theme")

    @property
    def hex_count(self) -> int:
        return len(self.hexes)

    def get_hex(self, q: int, r: int) -> Optional[HexTile]:
        """Get hex at coordinates, or None if not found."""
        for h in self.hexes:
            if h.q == q and h.r == r:
                return h
        return None


class GenerationSeed(BaseModel):
    """Complete seed data for world generation."""
    seed_id: str = Field(description="Unique identifier for this seed")
    version: str = Field(default="1.0", description="Schema version")
    regions: list[HexRegion] = Field(min_length=1)
    generation_prompt: str = Field(description="The prompt used to generate this seed")

    @property
    def total_hexes(self) -> int:
        return sum(r.hex_count for r in self.regions)
