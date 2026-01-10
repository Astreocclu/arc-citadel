"""Tests for edge handling modes."""
import pytest
from schemas import TaggedHex, EdgeType
from edge_handler import EdgeHandler, EdgeMode


@pytest.fixture
def explicit_handler():
    return EdgeHandler(mode=EdgeMode.EXPLICIT)


@pytest.fixture
def derived_handler():
    return EdgeHandler(mode=EdgeMode.DERIVED)


@pytest.fixture
def hybrid_handler():
    return EdgeHandler(mode=EdgeMode.HYBRID)


class TestExplicitMode:
    def test_explicit_passes_through_unchanged(self, explicit_handler):
        """Explicit mode returns edges as-is from LLM."""
        hex = TaggedHex(
            q=0, r=0,
            name="Test",
            description="Test",
            tags=["underground", "passage"],
            edge_types=["tunnel", "road", "blocked", "tunnel", "blocked", "blocked"],
        )
        result = explicit_handler.process(hex)
        assert result.edge_types == hex.edge_types


class TestDerivedMode:
    def test_underground_gets_tunnels(self, derived_handler):
        """Underground hex derives tunnel edges."""
        hex = TaggedHex(
            q=0, r=0,
            name="Cavern",
            description="Underground cavern",
            tags=["underground", "wild"],
            edge_types=["blocked"] * 6,  # Will be overwritten
        )
        result = derived_handler.process(hex)
        # Should have at least some tunnel edges for underground
        assert EdgeType.TUNNEL in result.edge_types

    def test_surface_wild_gets_wilderness(self, derived_handler):
        """Surface wild hex derives wilderness edges."""
        hex = TaggedHex(
            q=0, r=0,
            name="Forest",
            description="Forest clearing",
            tags=["surface", "wild"],
            edge_types=["blocked"] * 6,
        )
        result = derived_handler.process(hex)
        assert EdgeType.WILDERNESS in result.edge_types

    def test_passage_gets_multiple_open_edges(self, derived_handler):
        """Passage tag should have multiple non-blocked edges."""
        hex = TaggedHex(
            q=0, r=0,
            name="Tunnel Junction",
            description="Junction point",
            tags=["underground", "passage"],
            edge_types=["blocked"] * 6,
        )
        result = derived_handler.process(hex)
        non_blocked = [e for e in result.edge_types if e != EdgeType.BLOCKED]
        assert len(non_blocked) >= 2


class TestHybridMode:
    def test_heals_mismatched_edges(self, hybrid_handler):
        """Hybrid mode should fix obviously wrong edges."""
        hex = TaggedHex(
            q=0, r=0,
            name="Underground Road",
            description="Underground passage",
            tags=["underground", "passage"],
            edge_types=["road", "road", "road", "road", "road", "road"],  # Wrong for underground
        )
        result = hybrid_handler.process(hex)
        # Should convert roads to tunnels for underground
        assert EdgeType.TUNNEL in result.edge_types
