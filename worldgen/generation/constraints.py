"""Structural constraints for component generation.

These are deterministic rules that the LLM must follow.
The LLM generates ONLY creative content; structure comes from here.
"""

from dataclasses import dataclass
from typing import Optional
from worldgen.schemas import (
    ComponentCategory,
    Species,
    Terrain,
    ResourceType,
    Abundance,
    FeatureType,
)


@dataclass
class StructuralConstraints:
    """Deterministic structural constraints for a component category."""

    terrain: Terrain
    elevation_min: float
    elevation_max: float
    moisture_min: float
    moisture_max: float
    temperature_min: float
    temperature_max: float
    allowed_resources: list[ResourceType]
    allowed_features: list[FeatureType]
    species: Species

    def to_prompt_section(self) -> str:
        """Generate the constraint section for prompts."""
        return f"""STRUCTURAL CONSTRAINTS (YOU MUST USE THESE EXACT VALUES):
terrain: "{self.terrain.value}"
elevation: between {self.elevation_min} and {self.elevation_max}
moisture: between {self.moisture_min} and {self.moisture_max}
temperature: between {self.temperature_min} and {self.temperature_max}
species: "{self.species.value}"

ALLOWED RESOURCES (pick 0-3):
{', '.join(r.value for r in self.allowed_resources)}

ALLOWED FEATURES (pick 0-2):
{', '.join(f.value for f in self.allowed_features)}"""


# Constraint definitions per category
CATEGORY_CONSTRAINTS: dict[ComponentCategory, StructuralConstraints] = {
    # ===== DWARF CATEGORIES =====
    ComponentCategory.DWARF_HOLD_ENTRANCE: StructuralConstraints(
        terrain=Terrain.MOUNTAINS,
        elevation_min=800,
        elevation_max=1500,
        moisture_min=0.1,
        moisture_max=0.4,
        temperature_min=-10,
        temperature_max=15,
        allowed_resources=[
            ResourceType.IRON, ResourceType.COPPER, ResourceType.STONE,
            ResourceType.GOLD, ResourceType.SILVER,
        ],
        allowed_features=[
            FeatureType.GATE, FeatureType.FORTIFICATION, FeatureType.STATUE,
            FeatureType.RUNE_CARVED, FeatureType.WATCHTOWER,
        ],
        species=Species.DWARF,
    ),
    ComponentCategory.DWARF_HOLD_FORGE: StructuralConstraints(
        terrain=Terrain.UNDERGROUND,
        elevation_min=-500,
        elevation_max=-50,
        moisture_min=0.05,
        moisture_max=0.2,
        temperature_min=30,
        temperature_max=60,
        allowed_resources=[
            ResourceType.IRON, ResourceType.COPPER, ResourceType.COAL,
            ResourceType.GOLD, ResourceType.SILVER, ResourceType.GEMS,
            ResourceType.ADAMANTINE, ResourceType.MITHRIL,
        ],
        allowed_features=[
            FeatureType.FORGE, FeatureType.ANVIL, FeatureType.SMELTER,
            FeatureType.RUNE_CARVED, FeatureType.LAVA_CHANNEL,
        ],
        species=Species.DWARF,
    ),
    ComponentCategory.DWARF_HOLD_TAVERN: StructuralConstraints(
        terrain=Terrain.UNDERGROUND,
        elevation_min=-200,
        elevation_max=-20,
        moisture_min=0.2,
        moisture_max=0.5,
        temperature_min=15,
        temperature_max=25,
        allowed_resources=[
            ResourceType.GRAIN, ResourceType.MUSHROOM, ResourceType.WATER,
        ],
        allowed_features=[
            FeatureType.HEARTH, FeatureType.BREWING_VAT, FeatureType.STAGE,
            FeatureType.RUNE_CARVED,
        ],
        species=Species.DWARF,
    ),
    ComponentCategory.DWARF_HOLD_BARRACKS: StructuralConstraints(
        terrain=Terrain.UNDERGROUND,
        elevation_min=-300,
        elevation_max=-50,
        moisture_min=0.1,
        moisture_max=0.3,
        temperature_min=10,
        temperature_max=20,
        allowed_resources=[
            ResourceType.IRON, ResourceType.LEATHER, ResourceType.STONE,
        ],
        allowed_features=[
            FeatureType.ARMORY, FeatureType.TRAINING_GROUND, FeatureType.STATUE,
            FeatureType.RUNE_CARVED,
        ],
        species=Species.DWARF,
    ),
    ComponentCategory.DWARF_HOLD_ANCESTOR_HALL: StructuralConstraints(
        terrain=Terrain.UNDERGROUND,
        elevation_min=-400,
        elevation_max=-100,
        moisture_min=0.05,
        moisture_max=0.15,
        temperature_min=5,
        temperature_max=15,
        allowed_resources=[
            ResourceType.GOLD, ResourceType.GEMS, ResourceType.STONE,
        ],
        allowed_features=[
            FeatureType.TOMB, FeatureType.STATUE, FeatureType.SHRINE,
            FeatureType.RUNE_CARVED, FeatureType.ECHO_CHAMBER,
        ],
        species=Species.DWARF,
    ),
    ComponentCategory.DWARF_HOLD_TREASURY: StructuralConstraints(
        terrain=Terrain.UNDERGROUND,
        elevation_min=-600,
        elevation_max=-200,
        moisture_min=0.0,
        moisture_max=0.1,
        temperature_min=5,
        temperature_max=15,
        allowed_resources=[
            ResourceType.GOLD, ResourceType.SILVER, ResourceType.GEMS,
            ResourceType.MITHRIL, ResourceType.ADAMANTINE,
        ],
        allowed_features=[
            FeatureType.VAULT, FeatureType.RUNE_CARVED, FeatureType.TRAP,
        ],
        species=Species.DWARF,
    ),
    ComponentCategory.DWARF_HOLD_MINE_SHAFT: StructuralConstraints(
        terrain=Terrain.UNDERGROUND,
        elevation_min=-800,
        elevation_max=-100,
        moisture_min=0.1,
        moisture_max=0.4,
        temperature_min=15,
        temperature_max=35,
        allowed_resources=[
            ResourceType.IRON, ResourceType.COPPER, ResourceType.COAL,
            ResourceType.GOLD, ResourceType.SILVER, ResourceType.GEMS,
            ResourceType.ADAMANTINE, ResourceType.MITHRIL, ResourceType.STONE,
        ],
        allowed_features=[
            FeatureType.MINE_SHAFT, FeatureType.ORE_VEIN, FeatureType.CART_TRACK,
            FeatureType.SUPPORT_PILLAR,
        ],
        species=Species.DWARF,
    ),
    ComponentCategory.DWARF_HOLD_TUNNEL: StructuralConstraints(
        terrain=Terrain.UNDERGROUND,
        elevation_min=-500,
        elevation_max=-10,
        moisture_min=0.1,
        moisture_max=0.3,
        temperature_min=10,
        temperature_max=20,
        allowed_resources=[ResourceType.STONE],
        allowed_features=[
            FeatureType.RUNE_CARVED, FeatureType.CART_TRACK,
            FeatureType.SUPPORT_PILLAR, FeatureType.VENTILATION,
        ],
        species=Species.DWARF,
    ),
    ComponentCategory.DWARF_HOLD_LIVING_QUARTERS: StructuralConstraints(
        terrain=Terrain.UNDERGROUND,
        elevation_min=-300,
        elevation_max=-30,
        moisture_min=0.15,
        moisture_max=0.35,
        temperature_min=15,
        temperature_max=22,
        allowed_resources=[
            ResourceType.STONE, ResourceType.WATER, ResourceType.MUSHROOM,
        ],
        allowed_features=[
            FeatureType.HEARTH, FeatureType.CISTERN, FeatureType.RUNE_CARVED,
            FeatureType.FUNGAL_GARDEN,
        ],
        species=Species.DWARF,
    ),

    # ===== HUMAN CATEGORIES =====
    ComponentCategory.HUMAN_CITY_GATE: StructuralConstraints(
        terrain=Terrain.PLAINS,
        elevation_min=50,
        elevation_max=300,
        moisture_min=0.3,
        moisture_max=0.6,
        temperature_min=5,
        temperature_max=30,
        allowed_resources=[ResourceType.STONE, ResourceType.TIMBER],
        allowed_features=[
            FeatureType.GATE, FeatureType.FORTIFICATION, FeatureType.WATCHTOWER,
            FeatureType.GUARDHOUSE,
        ],
        species=Species.HUMAN,
    ),
    ComponentCategory.HUMAN_CITY_MARKET: StructuralConstraints(
        terrain=Terrain.PLAINS,
        elevation_min=50,
        elevation_max=200,
        moisture_min=0.3,
        moisture_max=0.5,
        temperature_min=10,
        temperature_max=30,
        allowed_resources=[
            ResourceType.GRAIN, ResourceType.LIVESTOCK, ResourceType.TIMBER,
            ResourceType.CLOTH, ResourceType.SPICES,
        ],
        allowed_features=[
            FeatureType.MARKET_STALL, FeatureType.FOUNTAIN, FeatureType.WELL,
            FeatureType.WAREHOUSE,
        ],
        species=Species.HUMAN,
    ),
    ComponentCategory.HUMAN_CITY_TEMPLE: StructuralConstraints(
        terrain=Terrain.PLAINS,
        elevation_min=80,
        elevation_max=250,
        moisture_min=0.3,
        moisture_max=0.5,
        temperature_min=10,
        temperature_max=25,
        allowed_resources=[
            ResourceType.STONE, ResourceType.GOLD, ResourceType.INCENSE,
        ],
        allowed_features=[
            FeatureType.SHRINE, FeatureType.ALTAR, FeatureType.STATUE,
            FeatureType.BELL_TOWER, FeatureType.SACRED_POOL,
        ],
        species=Species.HUMAN,
    ),
    ComponentCategory.HUMAN_CITY_PALACE: StructuralConstraints(
        terrain=Terrain.PLAINS,
        elevation_min=100,
        elevation_max=300,
        moisture_min=0.3,
        moisture_max=0.5,
        temperature_min=10,
        temperature_max=25,
        allowed_resources=[
            ResourceType.STONE, ResourceType.GOLD, ResourceType.GEMS,
            ResourceType.MARBLE,
        ],
        allowed_features=[
            FeatureType.THRONE_ROOM, FeatureType.COURTYARD, FeatureType.GARDEN,
            FeatureType.FORTIFICATION, FeatureType.STATUE,
        ],
        species=Species.HUMAN,
    ),

    # ===== ELF CATEGORIES =====
    ComponentCategory.ELF_GROVE_HEART_TREE: StructuralConstraints(
        terrain=Terrain.DENSE_FOREST,
        elevation_min=100,
        elevation_max=500,
        moisture_min=0.6,
        moisture_max=0.9,
        temperature_min=10,
        temperature_max=25,
        allowed_resources=[
            ResourceType.TIMBER, ResourceType.HERBS, ResourceType.CRYSTAL,
            ResourceType.MOONWELL_WATER,
        ],
        allowed_features=[
            FeatureType.ANCIENT_TREE, FeatureType.SACRED_POOL, FeatureType.SHRINE,
            FeatureType.SPIRIT_WARD,
        ],
        species=Species.ELF,
    ),
    ComponentCategory.ELF_GROVE_DWELLING: StructuralConstraints(
        terrain=Terrain.FOREST,
        elevation_min=50,
        elevation_max=400,
        moisture_min=0.5,
        moisture_max=0.8,
        temperature_min=10,
        temperature_max=25,
        allowed_resources=[
            ResourceType.TIMBER, ResourceType.HERBS, ResourceType.SILK,
        ],
        allowed_features=[
            FeatureType.TREEHOUSE, FeatureType.HANGING_BRIDGE, FeatureType.GARDEN,
            FeatureType.SPIRIT_WARD,
        ],
        species=Species.ELF,
    ),
    ComponentCategory.ELF_GROVE_COUNCIL_GLADE: StructuralConstraints(
        terrain=Terrain.FOREST,
        elevation_min=80,
        elevation_max=300,
        moisture_min=0.5,
        moisture_max=0.7,
        temperature_min=12,
        temperature_max=22,
        allowed_resources=[ResourceType.TIMBER, ResourceType.CRYSTAL],
        allowed_features=[
            FeatureType.STANDING_STONES, FeatureType.ANCIENT_TREE,
            FeatureType.SPIRIT_WARD, FeatureType.MOONWELL,
        ],
        species=Species.ELF,
    ),
}


def get_constraints(category: ComponentCategory) -> Optional[StructuralConstraints]:
    """Get structural constraints for a category.

    Returns None if category has no defined constraints yet.
    """
    return CATEGORY_CONSTRAINTS.get(category)


def validate_against_constraints(
    component_data: dict, constraints: StructuralConstraints
) -> list[str]:
    """Validate component data against structural constraints.

    Returns list of validation errors (empty if valid).
    This is deterministic, instant, and free.
    """
    errors = []

    # Terrain must match exactly
    if component_data.get("terrain") != constraints.terrain.value:
        errors.append(
            f"terrain must be '{constraints.terrain.value}', "
            f"got '{component_data.get('terrain')}'"
        )

    # Elevation must be in range
    elev = component_data.get("elevation", 0)
    if not (constraints.elevation_min <= elev <= constraints.elevation_max):
        errors.append(
            f"elevation {elev} outside range "
            f"[{constraints.elevation_min}, {constraints.elevation_max}]"
        )

    # Moisture must be in range
    moist = component_data.get("moisture", 0)
    if not (constraints.moisture_min <= moist <= constraints.moisture_max):
        errors.append(
            f"moisture {moist} outside range "
            f"[{constraints.moisture_min}, {constraints.moisture_max}]"
        )

    # Temperature must be in range
    temp = component_data.get("temperature", 0)
    if not (constraints.temperature_min <= temp <= constraints.temperature_max):
        errors.append(
            f"temperature {temp} outside range "
            f"[{constraints.temperature_min}, {constraints.temperature_max}]"
        )

    # Resources must be from allowed list
    allowed_resource_values = {r.value for r in constraints.allowed_resources}
    for res in component_data.get("resources", []):
        res_type = res.get("type") if isinstance(res, dict) else res
        if res_type not in allowed_resource_values:
            errors.append(f"resource '{res_type}' not allowed for this category")

    # Features must be from allowed list
    allowed_feature_values = {f.value for f in constraints.allowed_features}
    for feat in component_data.get("features", []):
        feat_type = feat.get("type") if isinstance(feat, dict) else feat
        if feat_type not in allowed_feature_values:
            errors.append(f"feature '{feat_type}' not allowed for this category")

    return errors
