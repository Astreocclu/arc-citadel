"""Tests for schema models."""

import pytest
from worldgen.schemas.base import HexCoord, Terrain, Species, SpeciesFitness
from worldgen.schemas.component import Component, ComponentCategory, ConnectionPoint
from worldgen.schemas.template import (
    SlotCount,
    ClusterSlot,
    InternalConnection,
    ExternalPort,
    ClusterTemplate,
    AssembledCluster,
)
from worldgen.schemas.connector import (
    ConnectorType,
    EntryPoint,
    MinorAnchorSlot,
    ElasticSegment,
    ConnectorHex,
    ConnectorCollection,
)
from worldgen.schemas.minor import MinorCategory, MinorAnchor
from worldgen.schemas.seed import ClusterPlacement, ConnectorAssignment, LayoutHint, WorldSeed
from worldgen.schemas.world import WorldHex, HexMap, AssembledWorld


class TestHexCoord:
    def test_distance_to_same_hex(self):
        a = HexCoord(q=0, r=0)
        assert a.distance_to(a) == 0

    def test_distance_to_adjacent(self):
        a = HexCoord(q=0, r=0)
        b = HexCoord(q=1, r=0)
        assert a.distance_to(b) == 1

    def test_distance_to_diagonal(self):
        a = HexCoord(q=0, r=0)
        b = HexCoord(q=2, r=-1)
        assert a.distance_to(b) == 2

    def test_distance_symmetric(self):
        a = HexCoord(q=3, r=-2)
        b = HexCoord(q=-1, r=4)
        assert a.distance_to(b) == b.distance_to(a)

    def test_hash_equality(self):
        a = HexCoord(q=5, r=3)
        b = HexCoord(q=5, r=3)
        assert hash(a) == hash(b)
        assert a == b


class TestEnums:
    def test_terrain_values(self):
        assert Terrain.MOUNTAINS.value == "mountains"
        assert Terrain.FOREST.value == "forest"

    def test_species_values(self):
        assert Species.DWARF.value == "dwarf"
        assert Species.ELF.value == "elf"
        assert Species.HUMAN.value == "human"


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


class TestClusterTemplate:
    def test_create_minimal_template(self):
        slot = ClusterSlot(
            slot_id="entrance",
            component_category=ComponentCategory.DWARF_HOLD_ENTRANCE,
            required=True,
            count=SlotCount(min=1, max=1),
        )
        template = ClusterTemplate(
            id="test_template",
            name="Test Template",
            species=Species.DWARF,
            slots=[slot],
            min_footprint=5,
            max_footprint=10,
            description="A test template",
        )
        assert template.id == "test_template"
        assert len(template.slots) == 1
        assert template.slots[0].slot_id == "entrance"

    def test_template_with_connections(self):
        slot1 = ClusterSlot(
            slot_id="entrance",
            component_category=ComponentCategory.DWARF_HOLD_ENTRANCE,
            required=True,
            count=SlotCount(min=1, max=1),
        )
        slot2 = ClusterSlot(
            slot_id="forge",
            component_category=ComponentCategory.DWARF_HOLD_FORGE,
            required=True,
            count=SlotCount(min=1, max=2),
        )
        conn = InternalConnection(
            from_slot="entrance",
            to_slot="forge",
            connector_category="dwarf_hold_tunnel",
        )
        port = ExternalPort(
            port_id="main_gate",
            attached_slot="entrance",
            direction_preference=["S", "SE"],
        )
        template = ClusterTemplate(
            id="test_template",
            name="Test Template",
            species=Species.DWARF,
            slots=[slot1, slot2],
            internal_connections=[conn],
            external_ports=[port],
            min_footprint=10,
            max_footprint=20,
            description="A test template with connections",
        )
        assert len(template.internal_connections) == 1
        assert len(template.external_ports) == 1


class TestAssembledCluster:
    def test_create_assembled_cluster(self):
        cluster = AssembledCluster(
            template_id="dwarf_hold_major",
            instance_id="dwarf_0",
            components={"entrance": ["comp_001"]},
            layout={"comp_001": (0, 0)},
            footprint=[(0, 0), (1, 0), (0, 1)],
        )
        assert cluster.template_id == "dwarf_hold_major"
        assert cluster.instance_id == "dwarf_0"

    def test_get_component_at(self):
        cluster = AssembledCluster(
            template_id="test",
            instance_id="test_0",
            components={"slot1": ["comp_a", "comp_b"]},
            layout={"comp_a": (0, 0), "comp_b": (1, 0)},
            footprint=[(0, 0), (1, 0)],
        )
        assert cluster.get_component_at((0, 0)) == "comp_a"
        assert cluster.get_component_at((1, 0)) == "comp_b"
        assert cluster.get_component_at((2, 0)) is None


class TestConnectorCollection:
    def test_create_minimal_connector(self):
        connector = ConnectorCollection(
            id="river_001",
            type=ConnectorType.RIVER_MIDDLE,
            tags=["river", "navigable"],
        )
        assert connector.id == "river_001"
        assert connector.type == ConnectorType.RIVER_MIDDLE

    def test_connector_with_hexes(self):
        hex1 = ConnectorHex(
            index=0,
            terrain=Terrain.SHALLOW_WATER,
            elevation=100.0,
            moisture=1.0,
            species_fitness=SpeciesFitness(human=0.5, dwarf=0.2, elf=0.6),
        )
        hex2 = ConnectorHex(
            index=1,
            terrain=Terrain.SHALLOW_WATER,
            elevation=95.0,
            moisture=1.0,
            species_fitness=SpeciesFitness(human=0.5, dwarf=0.2, elf=0.6),
            connects_to=[0],
        )
        connector = ConnectorCollection(
            id="river_002",
            type=ConnectorType.RIVER_UPPER,
            hexes=[hex1, hex2],
            base_length=2,
        )
        assert len(connector.hexes) == 2
        assert connector.hexes[1].connects_to == [0]

    def test_connector_with_entry_points(self):
        entry = EntryPoint(
            hex_index=0,
            direction="N",
            elevation=100.0,
            terrain=Terrain.SHALLOW_WATER,
            is_terminus=True,
        )
        connector = ConnectorCollection(
            id="river_003",
            type=ConnectorType.RIVER_HEADWATERS,
            entry_points=[entry],
        )
        assert len(connector.entry_points) == 1
        assert connector.entry_points[0].is_terminus is True


class TestMinorAnchor:
    def test_create_minimal_minor(self):
        minor = MinorAnchor(
            id="inn_001",
            category=MinorCategory.INN,
            name_fragment="Traveler's Rest",
            narrative_hook="A welcoming light shines from the windows.",
            provides_rest=True,
        )
        assert minor.id == "inn_001"
        assert minor.category == MinorCategory.INN
        assert minor.provides_rest is True

    def test_minor_with_terrain(self):
        minor = MinorAnchor(
            id="bridge_001",
            category=MinorCategory.BRIDGE_STONE,
            name_fragment="Old Stone Bridge",
            narrative_hook="Worn stone shows centuries of travelers.",
            compatible_terrain=[Terrain.SHALLOW_WATER, Terrain.MARSH],
        )
        assert len(minor.compatible_terrain) == 2
        assert Terrain.SHALLOW_WATER in minor.compatible_terrain


class TestWorldSeed:
    def test_create_minimal_seed(self):
        seed = WorldSeed(seed_id=42)
        assert seed.seed_id == 42
        assert seed.version == 1
        assert seed.world_radius == 150

    def test_seed_with_clusters(self):
        cluster1 = ClusterPlacement(
            template_id="dwarf_hold_major",
            instance_id="dwarf_0",
            region_hint="N",
        )
        cluster2 = ClusterPlacement(
            template_id="dwarf_hold_major",
            instance_id="dwarf_1",
            region_hint="NE",
        )
        hint = LayoutHint(
            cluster_id="dwarf_0",
            hints=["terrain:mountains", "region:north"],
        )
        seed = WorldSeed(
            seed_id=123,
            name="Test World",
            clusters=[cluster1, cluster2],
            layout_hints=[hint],
            world_radius=100,
        )
        assert len(seed.clusters) == 2
        assert len(seed.layout_hints) == 1
        assert seed.name == "Test World"

    def test_seed_with_connectors(self):
        connector = ConnectorAssignment(
            collection_id="river_full_001",
            instance_id="river_0",
            start_cluster="dwarf_0",
            start_port="main_gate",
            end_cluster="elf_0",
            end_port="river_dock",
        )
        seed = WorldSeed(
            seed_id=456,
            connectors=[connector],
        )
        assert len(seed.connectors) == 1
        assert seed.connectors[0].start_cluster == "dwarf_0"


class TestWorldHex:
    def test_create_world_hex(self):
        coord = HexCoord(q=5, r=-3)
        hex = WorldHex(
            coord=coord,
            terrain=Terrain.MOUNTAINS,
            elevation=2000.0,
            moisture=0.3,
            temperature=-5.0,
            species_fitness=SpeciesFitness(human=0.2, dwarf=0.9, elf=0.1),
        )
        assert hex.coord.q == 5
        assert hex.coord.r == -3
        assert hex.terrain == Terrain.MOUNTAINS

    def test_world_hex_with_cluster(self):
        coord = HexCoord(q=0, r=0)
        hex = WorldHex(
            coord=coord,
            terrain=Terrain.UNDERGROUND,
            elevation=500.0,
            moisture=0.2,
            temperature=15.0,
            species_fitness=SpeciesFitness(human=0.3, dwarf=0.95, elf=0.1),
            cluster_id="dwarf_0",
            component_id="forge_001",
        )
        assert hex.cluster_id == "dwarf_0"
        assert hex.component_id == "forge_001"


class TestHexMap:
    def test_create_hex_map(self):
        hex_map = HexMap(
            seed_id=42,
            world_radius=50,
        )
        assert hex_map.seed_id == 42
        assert hex_map.world_radius == 50
        assert len(hex_map.hexes) == 0

    def test_hex_map_with_clusters(self):
        cluster = AssembledCluster(
            template_id="test",
            instance_id="test_0",
            components={},
            layout={},
            footprint=[(0, 0)],
        )
        hex_map = HexMap(
            seed_id=42,
            world_radius=50,
            clusters={"test_0": cluster},
            cluster_positions={"test_0": (10, 10)},
        )
        assert "test_0" in hex_map.clusters
        assert hex_map.cluster_positions["test_0"] == (10, 10)


class TestAssembledWorld:
    def test_create_assembled_world(self):
        hex_map = HexMap(seed_id=42, world_radius=50)
        world = AssembledWorld(
            seed_id=42,
            name="Test World",
            hex_map=hex_map,
            generation_time_ms=1500,
            total_hexes=100,
            total_clusters=5,
        )
        assert world.seed_id == 42
        assert world.name == "Test World"
        assert world.generation_time_ms == 1500
        assert world.total_hexes == 100
        assert world.total_clusters == 5
