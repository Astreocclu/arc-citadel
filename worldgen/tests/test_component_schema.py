"""Tests for Component schema."""

import pytest
from worldgen.schemas.base import Species, Terrain, SpeciesFitness
from worldgen.schemas.component import Component, ComponentCategory, ConnectionPoint


class TestComponent:
    def test_create_minimal_component(self):
        comp = Component(
            id="comp_dwarf_forge_abc12345_00001",
            category=ComponentCategory.DWARF_HOLD_FORGE,
            tags=["dwarf", "forge"],
            species=Species.DWARF,
            terrain=Terrain.UNDERGROUND,
            elevation=500.0,
            moisture=0.2,
            temperature=25.0,
            species_fitness=SpeciesFitness(human=0.3, dwarf=0.95, elf=0.1),
            name_fragment="Deep Forge",
            narrative_hook="Ancient hammers still ring in these halls.",
            quality_score=8.5,
        )
        assert comp.id == "comp_dwarf_forge_abc12345_00001"
        assert comp.category == ComponentCategory.DWARF_HOLD_FORGE

    def test_component_with_connections(self):
        conn = ConnectionPoint(
            direction="DOWN",
            compatible_categories=["dwarf_hold_tunnel"],
            required=True,
        )
        comp = Component(
            id="test_comp",
            category=ComponentCategory.DWARF_HOLD_ENTRANCE,
            tags=["dwarf", "entrance"],
            species=Species.DWARF,
            terrain=Terrain.MOUNTAINS,
            elevation=1500.0,
            moisture=0.15,
            temperature=5.0,
            species_fitness=SpeciesFitness(human=0.3, dwarf=0.9, elf=0.1),
            name_fragment="Iron Gate",
            narrative_hook="The gate has stood for a thousand years.",
            connection_points=[conn],
            quality_score=9.0,
        )
        assert len(comp.connection_points) == 1
        assert comp.connection_points[0].required is True
