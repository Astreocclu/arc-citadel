"""Tests for hex coordinate utilities."""
import pytest
from hex_coords import (
    HEX_DIRECTIONS,
    get_neighbor,
    get_opposite_edge,
    get_all_neighbors,
    distance,
)


class TestHexDirections:
    def test_six_directions(self):
        assert len(HEX_DIRECTIONS) == 6

    def test_direction_names(self):
        expected = {"E", "NE", "NW", "W", "SW", "SE"}
        assert set(HEX_DIRECTIONS.keys()) == expected


class TestGetNeighbor:
    def test_east_neighbor(self):
        q, r = get_neighbor(0, 0, 0)  # Edge 0 = East
        assert (q, r) == (1, 0)

    def test_west_neighbor(self):
        q, r = get_neighbor(0, 0, 3)  # Edge 3 = West
        assert (q, r) == (-1, 0)

    def test_all_neighbors_from_origin(self):
        neighbors = [get_neighbor(0, 0, edge) for edge in range(6)]
        expected = [(1, 0), (1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1)]
        assert neighbors == expected


class TestGetOppositeEdge:
    def test_opposite_of_east_is_west(self):
        assert get_opposite_edge(0) == 3  # E -> W

    def test_opposite_of_northeast_is_southwest(self):
        assert get_opposite_edge(1) == 4  # NE -> SW

    def test_all_opposites(self):
        for edge in range(6):
            opposite = get_opposite_edge(edge)
            assert get_opposite_edge(opposite) == edge  # Double opposite = original


class TestDistance:
    def test_same_hex_distance_zero(self):
        assert distance(0, 0, 0, 0) == 0

    def test_adjacent_hex_distance_one(self):
        assert distance(0, 0, 1, 0) == 1
        assert distance(0, 0, 0, 1) == 1

    def test_diagonal_distance(self):
        assert distance(0, 0, 2, -1) == 2
