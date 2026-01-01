"""Tests for SQLite storage."""

import tempfile
from pathlib import Path

import pytest

from worldgen.storage.database import AssetDatabase
from worldgen.schemas import (
    Component,
    ComponentCategory,
    Species,
    Terrain,
    SpeciesFitness,
    ConnectorCollection,
    ConnectorType,
    MinorAnchor,
    MinorCategory,
)


@pytest.fixture
def temp_db():
    """Create a temporary database."""
    with tempfile.TemporaryDirectory() as tmpdir:
        db_path = Path(tmpdir) / "test.db"
        db = AssetDatabase(db_path)
        db.init()
        yield db
        db.close()


class TestDatabaseInit:
    def test_init_creates_tables(self, temp_db):
        """Tables should exist after init."""
        tables = temp_db.list_tables()
        assert "components" in tables
        assert "connectors" in tables
        assert "minors" in tables


class TestComponentOperations:
    def test_save_and_get_component(self, temp_db):
        """Test inserting and retrieving a component."""
        comp = Component(
            id="test_comp_001",
            category=ComponentCategory.DWARF_HOLD_FORGE,
            tags=["dwarf", "forge", "test"],
            species=Species.DWARF,
            terrain=Terrain.UNDERGROUND,
            elevation=500.0,
            moisture=0.2,
            temperature=25.0,
            species_fitness=SpeciesFitness(human=0.3, dwarf=0.95, elf=0.1),
            name_fragment="Test Forge",
            narrative_hook="A test forge.",
            quality_score=8.0,
        )
        temp_db.save_component(comp)

        retrieved = temp_db.get_component("test_comp_001")
        assert retrieved is not None
        assert retrieved.id == "test_comp_001"
        assert retrieved.category == ComponentCategory.DWARF_HOLD_FORGE
        assert retrieved.name_fragment == "Test Forge"
        assert retrieved.quality_score == 8.0

    def test_get_nonexistent_component(self, temp_db):
        """Getting a nonexistent component returns None."""
        result = temp_db.get_component("nonexistent")
        assert result is None

    def test_list_components_by_category(self, temp_db):
        """Test listing components filtered by category."""
        # Insert two forge components
        for i in range(2):
            comp = Component(
                id=f"forge_{i}",
                category=ComponentCategory.DWARF_HOLD_FORGE,
                tags=["dwarf"],
                species=Species.DWARF,
                terrain=Terrain.UNDERGROUND,
                elevation=500.0,
                moisture=0.2,
                temperature=25.0,
                species_fitness=SpeciesFitness(human=0.3, dwarf=0.9, elf=0.1),
                name_fragment=f"Forge {i}",
                narrative_hook="A forge.",
                quality_score=8.0,
            )
            temp_db.save_component(comp)

        # Insert one entrance component
        entrance = Component(
            id="entrance_0",
            category=ComponentCategory.DWARF_HOLD_ENTRANCE,
            tags=["dwarf"],
            species=Species.DWARF,
            terrain=Terrain.MOUNTAINS,
            elevation=1500.0,
            moisture=0.15,
            temperature=5.0,
            species_fitness=SpeciesFitness(human=0.3, dwarf=0.9, elf=0.1),
            name_fragment="Main Entrance",
            narrative_hook="An entrance.",
            quality_score=7.5,
        )
        temp_db.save_component(entrance)

        # Query forges only
        results = temp_db.list_components(category=ComponentCategory.DWARF_HOLD_FORGE)
        assert len(results) == 2

        # Query entrances only
        results = temp_db.list_components(category=ComponentCategory.DWARF_HOLD_ENTRANCE)
        assert len(results) == 1

    def test_list_components_by_min_quality(self, temp_db):
        """Test listing components filtered by minimum quality."""
        # Insert components with different quality scores
        for i, score in enumerate([6.0, 7.0, 8.0, 9.0]):
            comp = Component(
                id=f"quality_test_{i}",
                category=ComponentCategory.DWARF_HOLD_FORGE,
                tags=["dwarf"],
                species=Species.DWARF,
                terrain=Terrain.UNDERGROUND,
                elevation=500.0,
                moisture=0.2,
                temperature=25.0,
                species_fitness=SpeciesFitness(human=0.3, dwarf=0.9, elf=0.1),
                name_fragment=f"Forge {i}",
                narrative_hook="A forge.",
                quality_score=score,
            )
            temp_db.save_component(comp)

        results = temp_db.list_components(min_quality=8.0)
        assert len(results) == 2  # Should get the 8.0 and 9.0 scored ones

    def test_delete_component(self, temp_db):
        """Test deleting a component."""
        comp = Component(
            id="to_delete",
            category=ComponentCategory.DWARF_HOLD_FORGE,
            tags=["dwarf"],
            species=Species.DWARF,
            terrain=Terrain.UNDERGROUND,
            elevation=500.0,
            moisture=0.2,
            temperature=25.0,
            species_fitness=SpeciesFitness(human=0.3, dwarf=0.9, elf=0.1),
            name_fragment="Delete Me",
            narrative_hook="A forge.",
            quality_score=8.0,
        )
        temp_db.save_component(comp)
        assert temp_db.get_component("to_delete") is not None

        deleted = temp_db.delete_component("to_delete")
        assert deleted is True
        assert temp_db.get_component("to_delete") is None

    def test_delete_nonexistent_component(self, temp_db):
        """Deleting a nonexistent component returns False."""
        deleted = temp_db.delete_component("nonexistent")
        assert deleted is False

    def test_save_component_replaces_existing(self, temp_db):
        """Saving a component with the same ID replaces it."""
        comp1 = Component(
            id="replace_test",
            category=ComponentCategory.DWARF_HOLD_FORGE,
            tags=["dwarf"],
            species=Species.DWARF,
            terrain=Terrain.UNDERGROUND,
            elevation=500.0,
            moisture=0.2,
            temperature=25.0,
            species_fitness=SpeciesFitness(human=0.3, dwarf=0.9, elf=0.1),
            name_fragment="Original Name",
            narrative_hook="Original hook.",
            quality_score=7.0,
        )
        temp_db.save_component(comp1)

        comp2 = Component(
            id="replace_test",
            category=ComponentCategory.DWARF_HOLD_FORGE,
            tags=["dwarf", "updated"],
            species=Species.DWARF,
            terrain=Terrain.UNDERGROUND,
            elevation=500.0,
            moisture=0.2,
            temperature=25.0,
            species_fitness=SpeciesFitness(human=0.3, dwarf=0.9, elf=0.1),
            name_fragment="Updated Name",
            narrative_hook="Updated hook.",
            quality_score=9.0,
        )
        temp_db.save_component(comp2)

        retrieved = temp_db.get_component("replace_test")
        assert retrieved.name_fragment == "Updated Name"
        assert retrieved.quality_score == 9.0
        assert "updated" in retrieved.tags


class TestConnectorOperations:
    def test_save_and_get_connector(self, temp_db):
        """Test inserting and retrieving a connector."""
        connector = ConnectorCollection(
            id="river_001",
            type=ConnectorType.RIVER_FULL,
            tags=["water", "major"],
            name_fragment="Silverbrook River",
            quality_score=8.5,
        )
        temp_db.save_connector(connector)

        retrieved = temp_db.get_connector("river_001")
        assert retrieved is not None
        assert retrieved.id == "river_001"
        assert retrieved.type == ConnectorType.RIVER_FULL
        assert retrieved.name_fragment == "Silverbrook River"

    def test_get_nonexistent_connector(self, temp_db):
        """Getting a nonexistent connector returns None."""
        result = temp_db.get_connector("nonexistent")
        assert result is None

    def test_list_connectors_by_type(self, temp_db):
        """Test listing connectors filtered by type."""
        # Insert rivers
        for i in range(2):
            connector = ConnectorCollection(
                id=f"river_{i}",
                type=ConnectorType.RIVER_FULL,
                tags=["water"],
                quality_score=8.0,
            )
            temp_db.save_connector(connector)

        # Insert a trade route
        route = ConnectorCollection(
            id="route_0",
            type=ConnectorType.TRADE_ROUTE_MAJOR,
            tags=["trade"],
            quality_score=7.5,
        )
        temp_db.save_connector(route)

        results = temp_db.list_connectors(connector_type=ConnectorType.RIVER_FULL)
        assert len(results) == 2

    def test_delete_connector(self, temp_db):
        """Test deleting a connector."""
        connector = ConnectorCollection(
            id="to_delete",
            type=ConnectorType.RIVER_FULL,
            tags=["water"],
            quality_score=8.0,
        )
        temp_db.save_connector(connector)

        deleted = temp_db.delete_connector("to_delete")
        assert deleted is True
        assert temp_db.get_connector("to_delete") is None


class TestMinorOperations:
    def test_save_and_get_minor(self, temp_db):
        """Test inserting and retrieving a minor anchor."""
        minor = MinorAnchor(
            id="inn_001",
            category=MinorCategory.INN,
            tags=["rest", "trade"],
            name_fragment="The Weary Traveler",
            narrative_hook="A cozy inn at the crossroads.",
            quality_score=7.5,
        )
        temp_db.save_minor(minor)

        retrieved = temp_db.get_minor("inn_001")
        assert retrieved is not None
        assert retrieved.id == "inn_001"
        assert retrieved.category == MinorCategory.INN
        assert retrieved.name_fragment == "The Weary Traveler"

    def test_get_nonexistent_minor(self, temp_db):
        """Getting a nonexistent minor returns None."""
        result = temp_db.get_minor("nonexistent")
        assert result is None

    def test_list_minors_by_category(self, temp_db):
        """Test listing minors filtered by category."""
        # Insert inns
        for i in range(2):
            minor = MinorAnchor(
                id=f"inn_{i}",
                category=MinorCategory.INN,
                tags=["rest"],
                name_fragment=f"Inn {i}",
                narrative_hook="An inn.",
                quality_score=7.0,
            )
            temp_db.save_minor(minor)

        # Insert a shrine
        shrine = MinorAnchor(
            id="shrine_0",
            category=MinorCategory.SHRINE,
            tags=["sacred"],
            name_fragment="Roadside Shrine",
            narrative_hook="A small shrine.",
            quality_score=7.0,
        )
        temp_db.save_minor(shrine)

        results = temp_db.list_minors(category=MinorCategory.INN)
        assert len(results) == 2

    def test_delete_minor(self, temp_db):
        """Test deleting a minor anchor."""
        minor = MinorAnchor(
            id="to_delete",
            category=MinorCategory.INN,
            tags=["rest"],
            name_fragment="Delete Me",
            narrative_hook="An inn.",
            quality_score=7.0,
        )
        temp_db.save_minor(minor)

        deleted = temp_db.delete_minor("to_delete")
        assert deleted is True
        assert temp_db.get_minor("to_delete") is None


class TestDatabaseStats:
    def test_get_stats_empty_db(self, temp_db):
        """Stats show zero counts for empty database."""
        stats = temp_db.get_stats()
        assert "components" in stats
        assert "connectors" in stats
        assert "minors" in stats
        assert stats["components"] == 0
        assert stats["connectors"] == 0
        assert stats["minors"] == 0

    def test_get_stats_with_data(self, temp_db):
        """Stats reflect actual counts."""
        # Add some components
        for i in range(3):
            comp = Component(
                id=f"comp_{i}",
                category=ComponentCategory.DWARF_HOLD_FORGE,
                tags=["dwarf"],
                species=Species.DWARF,
                terrain=Terrain.UNDERGROUND,
                elevation=500.0,
                moisture=0.2,
                temperature=25.0,
                species_fitness=SpeciesFitness(human=0.3, dwarf=0.9, elf=0.1),
                name_fragment=f"Forge {i}",
                narrative_hook="A forge.",
                quality_score=8.0,
            )
            temp_db.save_component(comp)

        # Add some connectors
        for i in range(2):
            connector = ConnectorCollection(
                id=f"conn_{i}",
                type=ConnectorType.RIVER_FULL,
                tags=["water"],
                quality_score=8.0,
            )
            temp_db.save_connector(connector)

        # Add one minor
        minor = MinorAnchor(
            id="minor_0",
            category=MinorCategory.INN,
            tags=["rest"],
            name_fragment="Inn",
            narrative_hook="An inn.",
            quality_score=7.0,
        )
        temp_db.save_minor(minor)

        stats = temp_db.get_stats()
        assert stats["components"] == 3
        assert stats["connectors"] == 2
        assert stats["minors"] == 1
