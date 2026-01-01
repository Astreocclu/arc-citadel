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
