"""Axial hex coordinate utilities.

Hex edge numbering (clockwise from East):
    Edge 0: E   (+1,  0)
    Edge 1: NE  (+1, -1)
    Edge 2: NW  ( 0, -1)
    Edge 3: W   (-1,  0)
    Edge 4: SW  (-1, +1)
    Edge 5: SE  ( 0, +1)
"""

from typing import NamedTuple


class HexOffset(NamedTuple):
    """Offset for hex neighbor lookup."""
    dq: int
    dr: int


# Neighbor offsets indexed by edge number (clockwise from E)
HEX_NEIGHBOR_OFFSETS: list[HexOffset] = [
    HexOffset(+1,  0),  # Edge 0: E
    HexOffset(+1, -1),  # Edge 1: NE
    HexOffset( 0, -1),  # Edge 2: NW
    HexOffset(-1,  0),  # Edge 3: W
    HexOffset(-1, +1),  # Edge 4: SW
    HexOffset( 0, +1),  # Edge 5: SE
]

# Direction names for readability
HEX_DIRECTIONS: dict[str, int] = {
    "E": 0,
    "NE": 1,
    "NW": 2,
    "W": 3,
    "SW": 4,
    "SE": 5,
}


def get_neighbor(q: int, r: int, edge: int) -> tuple[int, int]:
    """Get coordinates of neighbor connected by given edge.

    Args:
        q: Axial q coordinate
        r: Axial r coordinate
        edge: Edge index 0-5 (clockwise from E)

    Returns:
        (q, r) of neighbor hex
    """
    offset = HEX_NEIGHBOR_OFFSETS[edge]
    return (q + offset.dq, r + offset.dr)


def get_opposite_edge(edge: int) -> int:
    """Get edge index on opposite side of hex.

    Edge 0 (E) opposite is Edge 3 (W), etc.
    """
    return (edge + 3) % 6


def get_all_neighbors(q: int, r: int) -> list[tuple[int, int, int]]:
    """Get all 6 neighbors with their connecting edge.

    Returns:
        List of (neighbor_q, neighbor_r, edge_from_center)
    """
    return [
        (q + offset.dq, r + offset.dr, edge)
        for edge, offset in enumerate(HEX_NEIGHBOR_OFFSETS)
    ]


def distance(q1: int, r1: int, q2: int, r2: int) -> int:
    """Calculate hex distance between two coordinates.

    Uses axial coordinate distance formula.
    """
    return (abs(q1 - q2) + abs(q1 + r1 - q2 - r2) + abs(r1 - r2)) // 2


def coords_to_key(q: int, r: int) -> str:
    """Convert coordinates to string key for dict lookups."""
    return f"{q},{r}"


def key_to_coords(key: str) -> tuple[int, int]:
    """Convert string key back to coordinates."""
    q, r = key.split(",")
    return (int(q), int(r))
