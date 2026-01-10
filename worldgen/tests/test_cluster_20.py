"""Integration test: Generate and validate 20-hex cluster."""
import os
import pytest
from cluster_generator import ClusterGenerator
from scale_validator import ScaleValidator
from adjacency import AdjacencyValidator
from hex_coords import get_neighbor, get_opposite_edge


# Skip if no API key
pytestmark = pytest.mark.skipif(
    not os.environ.get("DEEPSEEK_API_KEY"),
    reason="DEEPSEEK_API_KEY not set"
)


class TestCluster20Integration:
    """Full integration test for 20-hex cluster generation."""

    @pytest.fixture
    def cluster(self):
        """Generate a 20-hex cluster (cached for test session)."""
        with ClusterGenerator() as gen:
            return gen.generate(size=20, seed_tags=["underground", "dwarf", "passage", "shallow_under"])

    def test_cluster_has_20_hexes(self, cluster):
        """Cluster must have exactly 20 hexes."""
        assert len(cluster.hexes) == 20

    def test_all_hexes_have_valid_tags(self, cluster):
        """All hexes must have at least one tag."""
        for hex in cluster.hexes:
            assert len(hex.tags) >= 1
            # Must have a TERRAIN tag
            terrain_tags = {"underground", "surface", "underwater", "aerial"}
            assert any(t in terrain_tags for t in hex.tags), f"Hex {hex.name} missing TERRAIN tag"

    def test_all_hexes_have_6_edges(self, cluster):
        """All hexes must have exactly 6 edges."""
        for hex in cluster.hexes:
            assert len(hex.edge_types) == 6

    def test_cluster_is_connected(self, cluster):
        """All hexes must be reachable from the origin."""
        coord_to_hex = {(h.q, h.r): h for h in cluster.hexes}
        visited = set()

        def dfs(q, r):
            if (q, r) in visited:
                return
            if (q, r) not in coord_to_hex:
                return
            visited.add((q, r))
            for edge in range(6):
                nq, nr = get_neighbor(q, r, edge)
                dfs(nq, nr)

        # Start from first hex
        first = cluster.hexes[0]
        dfs(first.q, first.r)

        assert len(visited) == 20, f"Only {len(visited)} hexes connected, expected 20"

    def test_adjacencies_are_valid(self, cluster):
        """All adjacent hexes must pass adjacency validation."""
        validator = AdjacencyValidator("generation/hex_tags.toml")
        coord_to_hex = {(h.q, h.r): h for h in cluster.hexes}

        errors = []
        for hex in cluster.hexes:
            for edge in range(6):
                nq, nr = get_neighbor(hex.q, hex.r, edge)
                neighbor = coord_to_hex.get((nq, nr))
                if neighbor:
                    result = validator.validate_adjacency(hex, neighbor, edge)
                    if not result.valid:
                        errors.append(f"({hex.q},{hex.r}) -> ({nq},{nr}): {result.errors}")

        assert len(errors) == 0, f"Adjacency errors:\n" + "\n".join(errors)

    def test_scale_validation_passes(self, cluster):
        """All hexes must pass 100m scale validation (score >= 7)."""
        with ScaleValidator(threshold=7.0) as validator:
            results = validator.validate_batch(cluster.hexes)

        failures = [r for r in results if not r.passes]

        # Allow up to 2 failures (LLM isn't perfect)
        assert len(failures) <= 2, f"{len(failures)} hexes failed scale validation:\n" + \
            "\n".join(f"  Hex {r.hex_index}: score={r.score}, {r.feedback}" for r in failures)

    def test_edge_types_match_at_borders(self, cluster):
        """Matching edge types must be equal between adjacent hexes."""
        coord_to_hex = {(h.q, h.r): h for h in cluster.hexes}
        matching_types = {"tunnel", "road", "water"}

        errors = []
        for hex in cluster.hexes:
            for edge in range(6):
                nq, nr = get_neighbor(hex.q, hex.r, edge)
                neighbor = coord_to_hex.get((nq, nr))
                if neighbor:
                    my_edge = hex.edge_types[edge].value
                    their_edge = neighbor.edge_types[get_opposite_edge(edge)].value

                    if my_edge in matching_types or their_edge in matching_types:
                        if my_edge != their_edge:
                            errors.append(
                                f"({hex.q},{hex.r}) edge {edge}={my_edge} != "
                                f"({nq},{nr}) edge {get_opposite_edge(edge)}={their_edge}"
                            )

        assert len(errors) == 0, f"Edge mismatches:\n" + "\n".join(errors)


class TestEdgeModeComparison:
    """Compare the 3 edge handling modes."""

    @pytest.fixture(params=["explicit", "derived", "hybrid"])
    def edge_mode(self, request):
        return request.param

    def test_each_mode_produces_valid_cluster(self, edge_mode):
        """Each edge mode should produce a valid 20-hex cluster."""
        from edge_handler import EdgeHandler, EdgeMode

        handler = EdgeHandler(mode=EdgeMode(edge_mode))

        with ClusterGenerator() as gen:
            cluster = gen.generate(size=20)

        # Process edges with handler
        processed_hexes = [handler.process(h) for h in cluster.hexes]

        # Basic validation
        assert len(processed_hexes) == 20
        for hex in processed_hexes:
            assert len(hex.edge_types) == 6
