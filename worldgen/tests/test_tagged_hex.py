"""Tests for tag-based hex schemas."""
import pytest
from pydantic import ValidationError
from schemas import EdgeType, TaggedHex, HexCluster


class TestEdgeType:
    def test_valid_edge_types(self):
        assert EdgeType.TUNNEL == "tunnel"
        assert EdgeType.BLOCKED == "blocked"

    def test_all_edge_types_exist(self):
        expected = {"tunnel", "road", "water", "wilderness", "entrance", "blocked"}
        actual = {e.value for e in EdgeType}
        assert expected == actual


class TestTaggedHex:
    def test_valid_hex(self):
        hex = TaggedHex(
            q=0, r=0,
            name="Test Cavern",
            description="A small underground chamber",
            tags=["underground", "dwarf", "industrial"],
            edge_types=["tunnel", "tunnel", "blocked", "blocked", "blocked", "blocked"],
        )
        assert hex.q == 0
        assert len(hex.edge_types) == 6

    def test_requires_6_edges(self):
        with pytest.raises(ValidationError):
            TaggedHex(
                q=0, r=0,
                name="Bad Hex",
                description="Missing edges",
                tags=["surface"],
                edge_types=["road", "road"],  # Only 2 edges
            )

    def test_requires_at_least_one_tag(self):
        with pytest.raises(ValidationError):
            TaggedHex(
                q=0, r=0,
                name="No Tags",
                description="Empty tags",
                tags=[],
                edge_types=["blocked"] * 6,
            )

    def test_invalid_edge_type_rejected(self):
        with pytest.raises(ValidationError):
            TaggedHex(
                q=0, r=0,
                name="Bad Edge",
                description="Invalid edge type",
                tags=["surface"],
                edge_types=["tunnel", "road", "INVALID", "blocked", "blocked", "blocked"],
            )


class TestHexCluster:
    def test_cluster_requires_20_hexes(self):
        hexes = [
            TaggedHex(
                q=i, r=0,
                name=f"Hex {i}",
                description="Test hex",
                tags=["surface", "wild"],
                edge_types=["wilderness"] * 6,
            )
            for i in range(20)
        ]
        cluster = HexCluster(hexes=hexes)
        assert len(cluster.hexes) == 20

    def test_cluster_rejects_fewer_than_20(self):
        hexes = [
            TaggedHex(
                q=i, r=0,
                name=f"Hex {i}",
                description="Test hex",
                tags=["surface"],
                edge_types=["wilderness"] * 6,
            )
            for i in range(10)  # Only 10
        ]
        with pytest.raises(ValidationError):
            HexCluster(hexes=hexes)
