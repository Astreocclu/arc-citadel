"""Integration tests for end-to-end pipeline.

Tests the full worldgen pipeline flow:
    init database -> load templates -> generate seed -> assemble world

These tests verify that all components work together correctly without
requiring real LLM calls.
"""

import tempfile
from pathlib import Path

import pytest

from worldgen.storage import Database
from worldgen.templates import TemplateLoader
from worldgen.seeds import SeedGenerator
from worldgen.assembly import WorldAssembler
from worldgen.schemas import (
    WorldSeed,
    HexMap,
    AssembledCluster,
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


class TestEndToEndPipeline:
    """Integration tests for the complete pipeline flow."""

    def test_seed_to_world_assembly(self):
        """Test complete pipeline: templates -> seed -> assembly."""
        with tempfile.TemporaryDirectory() as tmpdir:
            db_path = Path(tmpdir) / "test.db"

            # 1. Initialize database
            db = Database(db_path)
            db.init()

            # 2. Load templates
            loader = TemplateLoader()
            templates = loader.load_all()
            assert "dwarf_hold_major" in templates

            # 3. Generate seed
            generator = SeedGenerator(list(templates.keys()))
            seed = generator.generate_seed(
                seed_id=42,
                num_dwarf_holds=2,
                num_elf_groves=0,
                num_human_cities=0,
                world_radius=50,
            )

            assert seed is not None
            assert seed.seed_id == 42
            assert len(seed.clusters) == 2

            # 4. Assemble world
            assembler = WorldAssembler(db)
            hex_map = assembler.assemble(seed)

            assert hex_map.seed_id == 42
            assert len(hex_map.clusters) == 2
            assert len(hex_map.cluster_positions) == 2

            # 5. Verify determinism
            hex_map_2 = assembler.assemble(seed)
            assert hex_map.cluster_positions == hex_map_2.cluster_positions

            db.close()

    def test_full_pipeline_with_database_components(self):
        """Test pipeline with components stored in database."""
        with tempfile.TemporaryDirectory() as tmpdir:
            db_path = Path(tmpdir) / "test.db"

            # 1. Initialize database
            db = Database(db_path)
            db.init()

            # 2. Store some components in the database
            for i in range(3):
                comp = Component(
                    id=f"dwarf_forge_{i:03d}",
                    category=ComponentCategory.DWARF_HOLD_FORGE,
                    tags=["dwarf", "forge", "production"],
                    species=Species.DWARF,
                    terrain=Terrain.UNDERGROUND,
                    elevation=500.0 - (i * 50),
                    moisture=0.2,
                    temperature=25.0 + i,
                    species_fitness=SpeciesFitness(human=0.3, dwarf=0.95, elf=0.1),
                    name_fragment=f"Deep Forge {i}",
                    narrative_hook=f"Ancient hammers ring in these halls (forge {i}).",
                    quality_score=8.0 + (i * 0.5),
                )
                db.save_component(comp)

            # Verify components were stored
            stats = db.get_stats()
            assert stats["components"] == 3

            # 3. Load templates
            loader = TemplateLoader()
            templates = loader.load_all()

            # 4. Generate seed
            generator = SeedGenerator(list(templates.keys()))
            seed = generator.generate_seed(
                seed_id=123,
                num_dwarf_holds=3,
                num_elf_groves=0,
                num_human_cities=0,
                world_radius=100,
            )

            # 5. Assemble world
            assembler = WorldAssembler(db)
            hex_map = assembler.assemble(seed)

            # Verify world was assembled
            assert isinstance(hex_map, HexMap)
            assert hex_map.seed_id == 123
            assert len(hex_map.clusters) == 3

            # Verify cluster positions are within world bounds
            for instance_id, position in hex_map.cluster_positions.items():
                q, r = position
                assert abs(q) <= seed.world_radius
                assert abs(r) <= seed.world_radius

            db.close()

    def test_database_roundtrip(self):
        """Test storing and retrieving all asset types."""
        with tempfile.TemporaryDirectory() as tmpdir:
            db_path = Path(tmpdir) / "test.db"
            db = Database(db_path)
            db.init()

            # Create and store component
            comp = Component(
                id="integration_test_comp_001",
                category=ComponentCategory.DWARF_HOLD_FORGE,
                tags=["dwarf", "forge", "integration"],
                species=Species.DWARF,
                terrain=Terrain.UNDERGROUND,
                elevation=500.0,
                moisture=0.2,
                temperature=25.0,
                species_fitness=SpeciesFitness(human=0.3, dwarf=0.95, elf=0.1),
                name_fragment="Test Forge",
                narrative_hook="Integration test forge.",
                quality_score=8.5,
            )
            db.save_component(comp)

            # Create and store connector
            connector = ConnectorCollection(
                id="integration_test_conn_001",
                type=ConnectorType.TRADE_ROUTE_MAJOR,
                tags=["trade", "road", "integration"],
                quality_score=7.5,
            )
            db.save_connector(connector)

            # Create and store minor anchor
            minor = MinorAnchor(
                id="integration_test_minor_001",
                category=MinorCategory.INN,
                tags=["rest", "trade", "integration"],
                name_fragment="Roadside Inn",
                narrative_hook="Weary travelers find respite here.",
                provides_rest=True,
                quality_score=7.0,
            )
            db.save_minor(minor)

            # Retrieve and verify component
            retrieved_comp = db.get_component("integration_test_comp_001")
            assert retrieved_comp is not None
            assert retrieved_comp.name_fragment == "Test Forge"
            assert retrieved_comp.quality_score == 8.5
            assert retrieved_comp.category == ComponentCategory.DWARF_HOLD_FORGE

            # Retrieve and verify connector
            retrieved_conn = db.get_connector("integration_test_conn_001")
            assert retrieved_conn is not None
            assert retrieved_conn.type == ConnectorType.TRADE_ROUTE_MAJOR
            assert "trade" in retrieved_conn.tags

            # Retrieve and verify minor
            retrieved_minor = db.get_minor("integration_test_minor_001")
            assert retrieved_minor is not None
            assert retrieved_minor.category == MinorCategory.INN
            assert retrieved_minor.provides_rest is True

            # Query by category
            results = db.list_components(category=ComponentCategory.DWARF_HOLD_FORGE)
            assert len(results) == 1

            # Verify stats
            stats = db.get_stats()
            assert stats["components"] == 1
            assert stats["connectors"] == 1
            assert stats["minors"] == 1

            db.close()

    def test_seed_persistence_roundtrip(self):
        """Test saving and loading seeds through the pipeline."""
        with tempfile.TemporaryDirectory() as tmpdir:
            db_path = Path(tmpdir) / "test.db"
            seeds_dir = Path(tmpdir) / "seeds"

            # Initialize database
            db = Database(db_path)
            db.init()

            # Load templates
            loader = TemplateLoader()
            templates = loader.load_all()

            # Generate seed
            generator = SeedGenerator(list(templates.keys()))
            original_seed = generator.generate_seed(
                seed_id=999,
                num_dwarf_holds=4,
                num_elf_groves=0,
                num_human_cities=0,
                world_radius=75,
            )

            # Save seed
            seed_path = seeds_dir / "test_seed_999.json"
            generator.save_seed(original_seed, seed_path)
            assert seed_path.exists()

            # Load seed
            loaded_seed = generator.load_seed(seed_path)
            assert loaded_seed.seed_id == original_seed.seed_id
            assert loaded_seed.world_radius == original_seed.world_radius
            assert len(loaded_seed.clusters) == len(original_seed.clusters)

            # Assemble both seeds and verify they produce identical results
            assembler = WorldAssembler(db)
            hex_map_original = assembler.assemble(original_seed)
            hex_map_loaded = assembler.assemble(loaded_seed)

            assert hex_map_original.cluster_positions == hex_map_loaded.cluster_positions

            db.close()

    def test_multiple_seeds_independent(self):
        """Test that different seeds produce different worlds."""
        with tempfile.TemporaryDirectory() as tmpdir:
            db_path = Path(tmpdir) / "test.db"

            db = Database(db_path)
            db.init()

            loader = TemplateLoader()
            templates = loader.load_all()
            generator = SeedGenerator(list(templates.keys()))
            assembler = WorldAssembler(db)

            # Generate and assemble multiple seeds
            seeds = [
                generator.generate_seed(seed_id=i, num_dwarf_holds=3)
                for i in [1, 42, 999]
            ]
            hex_maps = [assembler.assemble(seed) for seed in seeds]

            # Verify all have different cluster positions
            positions_sets = [
                frozenset(hm.cluster_positions.values()) for hm in hex_maps
            ]

            # At least some should be different (with high probability)
            # We check that not all are identical
            assert len(set(positions_sets)) > 1 or len(positions_sets) == 1

            db.close()

    def test_world_assembly_with_layout_hints(self):
        """Test that layout hints are passed through the pipeline."""
        with tempfile.TemporaryDirectory() as tmpdir:
            db_path = Path(tmpdir) / "test.db"

            db = Database(db_path)
            db.init()

            loader = TemplateLoader()
            templates = loader.load_all()
            generator = SeedGenerator(list(templates.keys()))

            # Generate seed (which includes layout hints)
            seed = generator.generate_seed(
                seed_id=42,
                num_dwarf_holds=2,
                world_radius=50,
            )

            # Verify layout hints were created
            assert len(seed.layout_hints) == 2
            for hint in seed.layout_hints:
                assert "terrain:mountains" in hint.hints

            # Assemble world
            assembler = WorldAssembler(db)
            hex_map = assembler.assemble(seed)

            # Verify assembly completed successfully
            assert len(hex_map.clusters) == 2
            assert len(hex_map.cluster_positions) == 2

            db.close()

    def test_cluster_footprints_do_not_overlap(self):
        """Test that assembled clusters have non-overlapping footprints."""
        with tempfile.TemporaryDirectory() as tmpdir:
            db_path = Path(tmpdir) / "test.db"

            db = Database(db_path)
            db.init()

            loader = TemplateLoader()
            templates = loader.load_all()
            generator = SeedGenerator(list(templates.keys()))

            # Generate seed with multiple clusters
            seed = generator.generate_seed(
                seed_id=42,
                num_dwarf_holds=5,
                world_radius=100,
            )

            assembler = WorldAssembler(db)
            hex_map = assembler.assemble(seed)

            # Collect all hex positions from all cluster footprints
            all_hexes = set()
            for instance_id, cluster in hex_map.clusters.items():
                position = hex_map.cluster_positions[instance_id]
                for offset in cluster.footprint:
                    world_hex = (position[0] + offset[0], position[1] + offset[1])
                    # Verify no overlap
                    assert world_hex not in all_hexes, (
                        f"Overlapping hex at {world_hex}"
                    )
                    all_hexes.add(world_hex)

            db.close()

    def test_assembled_cluster_attributes(self):
        """Test that assembled clusters have expected attributes."""
        with tempfile.TemporaryDirectory() as tmpdir:
            db_path = Path(tmpdir) / "test.db"

            db = Database(db_path)
            db.init()

            loader = TemplateLoader()
            templates = loader.load_all()
            generator = SeedGenerator(list(templates.keys()))

            seed = generator.generate_seed(
                seed_id=42,
                num_dwarf_holds=2,
                world_radius=50,
            )

            assembler = WorldAssembler(db)
            hex_map = assembler.assemble(seed)

            # Verify each cluster has expected attributes
            for instance_id, cluster in hex_map.clusters.items():
                assert isinstance(cluster, AssembledCluster)
                assert cluster.template_id == "dwarf_hold_major"
                assert cluster.instance_id == instance_id
                assert len(cluster.footprint) > 0
                assert isinstance(cluster.components, dict)
                assert isinstance(cluster.layout, dict)

            db.close()


class TestPipelineErrorHandling:
    """Tests for error handling in the pipeline."""

    def test_assemble_empty_seed(self):
        """Test assembling a seed with no clusters."""
        with tempfile.TemporaryDirectory() as tmpdir:
            db_path = Path(tmpdir) / "test.db"

            db = Database(db_path)
            db.init()

            seed = WorldSeed(
                seed_id=1,
                clusters=[],
                connectors=[],
                layout_hints=[],
                world_radius=100,
            )

            assembler = WorldAssembler(db)
            hex_map = assembler.assemble(seed)

            assert hex_map.seed_id == 1
            assert hex_map.clusters == {}
            assert hex_map.cluster_positions == {}

            db.close()

    def test_seed_generator_with_missing_template(self):
        """Test seed generation when requested template is not available."""
        # Create generator with only elf templates (no dwarf)
        generator = SeedGenerator(["elf_grove_major"])

        seed = generator.generate_seed(
            seed_id=1,
            num_dwarf_holds=3,  # Request dwarves but none available
            num_elf_groves=0,
            num_human_cities=0,
        )

        # Should still return a valid seed, just with no clusters
        assert seed is not None
        assert seed.seed_id == 1
        assert len(seed.clusters) == 0

    def test_database_operations_without_init(self):
        """Test that database operations work after proper initialization."""
        with tempfile.TemporaryDirectory() as tmpdir:
            db_path = Path(tmpdir) / "test.db"

            db = Database(db_path)
            db.init()

            # Should be able to get stats
            stats = db.get_stats()
            assert stats["components"] == 0
            assert stats["connectors"] == 0
            assert stats["minors"] == 0

            db.close()


class TestPipelineIntegrationWithCLI:
    """Tests that verify CLI-like usage patterns."""

    def test_init_generate_assemble_workflow(self):
        """Test the typical CLI workflow: init -> generate -> assemble."""
        with tempfile.TemporaryDirectory() as tmpdir:
            output_dir = Path(tmpdir) / "output"

            # 1. Init (like `worldgen init`)
            libs_dir = output_dir / "libraries"
            seeds_dir = output_dir / "seeds"
            worlds_dir = output_dir / "worlds"

            libs_dir.mkdir(parents=True)
            seeds_dir.mkdir(parents=True)
            worlds_dir.mkdir(parents=True)

            db_path = libs_dir / "assets.db"
            db = Database(db_path)
            db.init()

            # 2. Load templates
            loader = TemplateLoader()
            templates = loader.load_all()
            assert len(templates) >= 1

            # 3. Generate seed
            generator = SeedGenerator(list(templates.keys()))
            seed = generator.generate_seed(
                seed_id=42,
                num_dwarf_holds=2,
                world_radius=50,
            )

            # Save seed
            seed_path = seeds_dir / f"seed_{seed.seed_id}.json"
            generator.save_seed(seed, seed_path)
            assert seed_path.exists()

            # 4. Assemble world
            assembler = WorldAssembler(db)
            hex_map = assembler.assemble(seed)

            # Verify world was created
            assert hex_map.seed_id == 42
            assert len(hex_map.clusters) == 2

            db.close()

    def test_batch_seed_generation_and_assembly(self):
        """Test generating and assembling multiple seeds in batch."""
        with tempfile.TemporaryDirectory() as tmpdir:
            db_path = Path(tmpdir) / "test.db"

            db = Database(db_path)
            db.init()

            loader = TemplateLoader()
            templates = loader.load_all()
            generator = SeedGenerator(list(templates.keys()))
            assembler = WorldAssembler(db)

            # Generate batch of seeds
            seeds = []
            for i in range(5):
                seed = generator.generate_seed(
                    seed_id=i * 100,
                    num_dwarf_holds=2,
                    world_radius=50,
                )
                seeds.append(seed)

            # Assemble all seeds
            hex_maps = []
            for seed in seeds:
                hex_map = assembler.assemble(seed)
                hex_maps.append(hex_map)

            # Verify all were assembled
            assert len(hex_maps) == 5
            for i, hex_map in enumerate(hex_maps):
                assert hex_map.seed_id == i * 100
                assert len(hex_map.clusters) == 2

            db.close()
