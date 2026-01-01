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
