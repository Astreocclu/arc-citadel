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
