"""Tests for schema models."""

import pytest
from worldgen.schemas.base import HexCoord, Terrain, Species


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
