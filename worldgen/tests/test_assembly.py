"""Tests for world assembly module."""

import random
import tempfile
from pathlib import Path

import pytest

from worldgen.assembly import WorldAssembler
from worldgen.assembly.layout_solver import LayoutSolver
from worldgen.schemas import (
    WorldSeed,
    ClusterPlacement,
    LayoutHint,
    AssembledCluster,
    HexMap,
)
from worldgen.storage import Database


class TestLayoutSolver:
    """Tests for the LayoutSolver class."""

    def test_solver_init(self):
        """Test LayoutSolver can be instantiated."""
        solver = LayoutSolver()
        assert solver.max_attempts == 1000

    def test_solve_empty_clusters(self):
        """Test solving with no clusters returns empty positions."""
        solver = LayoutSolver()
        rng = random.Random(42)
        positions = solver.solve(
            clusters={},
            hints=[],
            world_radius=100,
            rng=rng,
        )
        assert positions == {}

    def test_solve_single_cluster(self):
        """Test solving with a single cluster."""
        solver = LayoutSolver()
        rng = random.Random(42)

        cluster = AssembledCluster(
            template_id="dwarf_hold_major",
            instance_id="test_cluster",
            components={},
            layout={},
            footprint=[(0, 0)],
        )

        positions = solver.solve(
            clusters={"test_cluster": cluster},
            hints=[],
            world_radius=100,
            rng=rng,
        )

        assert "test_cluster" in positions
        q, r = positions["test_cluster"]
        assert abs(q) <= 100
        assert abs(r) <= 100
        assert abs(q + r) <= 100

    def test_solve_multiple_clusters_no_overlap(self):
        """Test that multiple clusters don't overlap."""
        solver = LayoutSolver()
        rng = random.Random(42)

        clusters = {}
        for i in range(5):
            clusters[f"cluster_{i}"] = AssembledCluster(
                template_id="dwarf_hold_major",
                instance_id=f"cluster_{i}",
                components={},
                layout={},
                footprint=[(0, 0), (1, 0), (0, 1)],
            )

        positions = solver.solve(
            clusters=clusters,
            hints=[],
            world_radius=100,
            rng=rng,
        )

        # Check all clusters were placed
        assert len(positions) == 5

        # Check no overlapping footprints
        all_hexes = set()
        for instance_id, cluster in clusters.items():
            pos = positions[instance_id]
            for offset in cluster.footprint:
                hex_pos = (pos[0] + offset[0], pos[1] + offset[1])
                assert hex_pos not in all_hexes, f"Overlapping hex at {hex_pos}"
                all_hexes.add(hex_pos)

    def test_solve_deterministic(self):
        """Test that solving with the same seed produces the same result."""
        solver = LayoutSolver()

        cluster = AssembledCluster(
            template_id="dwarf_hold_major",
            instance_id="test_cluster",
            components={},
            layout={},
            footprint=[(0, 0), (1, 0)],
        )
        clusters = {"test_cluster": cluster}

        rng1 = random.Random(42)
        positions1 = solver.solve(clusters=clusters, hints=[], world_radius=100, rng=rng1)

        rng2 = random.Random(42)
        positions2 = solver.solve(clusters=clusters, hints=[], world_radius=100, rng=rng2)

        assert positions1 == positions2


class TestWorldAssembler:
    """Tests for the WorldAssembler class."""

    @pytest.fixture
    def temp_db(self):
        """Create a temporary database for testing."""
        with tempfile.TemporaryDirectory() as tmpdir:
            db_path = Path(tmpdir) / "test.db"
            db = Database(db_path)
            db.init()
            yield db
            db.close()

    def test_assembler_init(self, temp_db):
        """Test WorldAssembler can be instantiated."""
        assembler = WorldAssembler(temp_db)
        assert assembler.database is temp_db
        assert isinstance(assembler.layout_solver, LayoutSolver)

    def test_assemble_empty_seed(self, temp_db):
        """Test assembling a seed with no clusters."""
        assembler = WorldAssembler(temp_db)
        seed = WorldSeed(
            seed_id=42,
            clusters=[],
            connectors=[],
            layout_hints=[],
            world_radius=100,
        )

        hex_map = assembler.assemble(seed)

        assert isinstance(hex_map, HexMap)
        assert hex_map.seed_id == 42
        assert hex_map.world_radius == 100
        assert hex_map.clusters == {}
        assert hex_map.cluster_positions == {}

    def test_assemble_single_cluster(self, temp_db):
        """Test assembling a seed with a single cluster."""
        assembler = WorldAssembler(temp_db)
        seed = WorldSeed(
            seed_id=42,
            clusters=[
                ClusterPlacement(
                    template_id="dwarf_hold_major",
                    instance_id="dwarf_0",
                )
            ],
            connectors=[],
            layout_hints=[],
            world_radius=100,
        )

        hex_map = assembler.assemble(seed)

        assert len(hex_map.clusters) == 1
        assert "dwarf_0" in hex_map.clusters
        assert len(hex_map.cluster_positions) == 1
        assert "dwarf_0" in hex_map.cluster_positions

    def test_assemble_multiple_clusters(self, temp_db):
        """Test assembling a seed with multiple clusters."""
        assembler = WorldAssembler(temp_db)
        seed = WorldSeed(
            seed_id=42,
            clusters=[
                ClusterPlacement(
                    template_id="dwarf_hold_major",
                    instance_id="dwarf_0",
                ),
                ClusterPlacement(
                    template_id="dwarf_hold_major",
                    instance_id="dwarf_1",
                ),
                ClusterPlacement(
                    template_id="dwarf_hold_major",
                    instance_id="dwarf_2",
                ),
            ],
            connectors=[],
            layout_hints=[],
            world_radius=100,
        )

        hex_map = assembler.assemble(seed)

        assert len(hex_map.clusters) == 3
        assert len(hex_map.cluster_positions) == 3

    def test_assemble_deterministic(self, temp_db):
        """Test that assembling the same seed produces the same result."""
        assembler = WorldAssembler(temp_db)
        seed = WorldSeed(
            seed_id=42,
            clusters=[
                ClusterPlacement(
                    template_id="dwarf_hold_major",
                    instance_id="dwarf_0",
                ),
                ClusterPlacement(
                    template_id="dwarf_hold_major",
                    instance_id="dwarf_1",
                ),
            ],
            connectors=[],
            layout_hints=[],
            world_radius=100,
        )

        hex_map1 = assembler.assemble(seed)
        hex_map2 = assembler.assemble(seed)

        assert hex_map1.cluster_positions == hex_map2.cluster_positions

    def test_assemble_with_layout_hints(self, temp_db):
        """Test assembling a seed with layout hints."""
        assembler = WorldAssembler(temp_db)
        seed = WorldSeed(
            seed_id=42,
            clusters=[
                ClusterPlacement(
                    template_id="dwarf_hold_major",
                    instance_id="dwarf_0",
                    region_hint="N",
                ),
            ],
            connectors=[],
            layout_hints=[
                LayoutHint(
                    cluster_id="dwarf_0",
                    hints=["terrain:mountains", "region:north"],
                ),
            ],
            world_radius=100,
        )

        hex_map = assembler.assemble(seed)

        # Hints are currently not used in the stub, but should still work
        assert len(hex_map.clusters) == 1
        assert "dwarf_0" in hex_map.cluster_positions

    def test_assembled_cluster_has_footprint(self, temp_db):
        """Test that assembled clusters have a footprint."""
        assembler = WorldAssembler(temp_db)
        seed = WorldSeed(
            seed_id=42,
            clusters=[
                ClusterPlacement(
                    template_id="dwarf_hold_major",
                    instance_id="dwarf_0",
                ),
            ],
            connectors=[],
            layout_hints=[],
            world_radius=100,
        )

        hex_map = assembler.assemble(seed)

        cluster = hex_map.clusters["dwarf_0"]
        assert len(cluster.footprint) > 0
        assert cluster.template_id == "dwarf_hold_major"
        assert cluster.instance_id == "dwarf_0"
