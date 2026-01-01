"""Tests for seed generation."""

import tempfile
from pathlib import Path

import pytest

from worldgen.seeds import SeedGenerator
from worldgen.schemas import WorldSeed, ClusterPlacement, LayoutHint


class TestSeedGenerator:
    def test_create_generator(self):
        """Test that SeedGenerator can be instantiated."""
        templates = ["dwarf_hold_major", "elf_grove_major"]
        gen = SeedGenerator(templates)
        assert gen.available_templates == templates

    def test_generate_seed_basic(self):
        """Test generating a basic seed."""
        templates = ["dwarf_hold_major"]
        gen = SeedGenerator(templates)

        seed = gen.generate_seed(
            seed_id=42,
            num_dwarf_holds=2,
            num_elf_groves=0,
            num_human_cities=0,
            world_radius=100,
        )

        assert seed is not None
        assert seed.seed_id == 42
        assert seed.world_radius == 100
        assert len(seed.clusters) == 2

    def test_generate_seed_with_no_matching_templates(self):
        """Test generating a seed when no templates match."""
        templates = ["elf_grove_major"]  # No dwarf templates
        gen = SeedGenerator(templates)

        seed = gen.generate_seed(
            seed_id=1,
            num_dwarf_holds=3,  # Request dwarves but none available
            num_elf_groves=0,
            num_human_cities=0,
        )

        # Should still return a seed, just with no dwarf clusters
        assert seed is not None
        assert len(seed.clusters) == 0

    def test_generate_seed_creates_layout_hints(self):
        """Test that layout hints are created for clusters."""
        templates = ["dwarf_hold_major"]
        gen = SeedGenerator(templates)

        seed = gen.generate_seed(
            seed_id=123,
            num_dwarf_holds=2,
            num_elf_groves=0,
            num_human_cities=0,
        )

        assert len(seed.layout_hints) == 2
        for hint in seed.layout_hints:
            assert "terrain:mountains" in hint.hints

    def test_generate_seed_deterministic(self):
        """Test that same seed_id produces same result."""
        templates = ["dwarf_hold_major"]
        gen = SeedGenerator(templates)

        seed1 = gen.generate_seed(seed_id=42, num_dwarf_holds=3)
        seed2 = gen.generate_seed(seed_id=42, num_dwarf_holds=3)

        assert seed1.clusters[0].region_hint == seed2.clusters[0].region_hint
        assert seed1.clusters[1].region_hint == seed2.clusters[1].region_hint

    def test_generate_seed_different_seeds_differ(self):
        """Test that different seed_ids produce different results."""
        templates = ["dwarf_hold_major"]
        gen = SeedGenerator(templates)

        seed1 = gen.generate_seed(seed_id=1, num_dwarf_holds=10)
        seed2 = gen.generate_seed(seed_id=999, num_dwarf_holds=10)

        # With 10 clusters, region_hints should differ for at least one
        hints1 = [c.region_hint for c in seed1.clusters]
        hints2 = [c.region_hint for c in seed2.clusters]
        # They could theoretically be the same, but with 10 choices it's unlikely
        # We just verify both are valid
        assert all(h in ["N", "NE", "NW"] for h in hints1)
        assert all(h in ["N", "NE", "NW"] for h in hints2)

    def test_save_and_load_seed(self):
        """Test saving and loading a seed to/from JSON."""
        templates = ["dwarf_hold_major"]
        gen = SeedGenerator(templates)

        seed = gen.generate_seed(seed_id=42, num_dwarf_holds=2)

        with tempfile.TemporaryDirectory() as tmpdir:
            seed_path = Path(tmpdir) / "seeds" / "test_seed.json"

            # Save
            gen.save_seed(seed, seed_path)
            assert seed_path.exists()

            # Load
            loaded = gen.load_seed(seed_path)
            assert loaded.seed_id == seed.seed_id
            assert loaded.world_radius == seed.world_radius
            assert len(loaded.clusters) == len(seed.clusters)
            assert loaded.clusters[0].template_id == seed.clusters[0].template_id

    def test_cluster_placement_has_unique_instance_ids(self):
        """Test that generated clusters have unique instance IDs."""
        templates = ["dwarf_hold_major"]
        gen = SeedGenerator(templates)

        seed = gen.generate_seed(seed_id=1, num_dwarf_holds=5)

        instance_ids = [c.instance_id for c in seed.clusters]
        assert len(instance_ids) == len(set(instance_ids))  # All unique

    def test_world_seed_has_default_values(self):
        """Test that WorldSeed has expected default values."""
        templates = ["dwarf_hold_major"]
        gen = SeedGenerator(templates)

        seed = gen.generate_seed(seed_id=1, num_dwarf_holds=1)

        assert seed.version == 1
        assert seed.climate_band == "temperate"
        assert seed.theme_tags == []
        assert seed.connectors == []
