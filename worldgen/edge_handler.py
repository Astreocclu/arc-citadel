"""Edge handling modes for hex composition."""

from enum import Enum
from copy import deepcopy

from schemas import TaggedHex, EdgeType


class EdgeMode(str, Enum):
    """Edge handling strategy."""
    EXPLICIT = "explicit"  # Use LLM output as-is
    DERIVED = "derived"    # Infer edges from tags
    HYBRID = "hybrid"      # LLM suggests, system heals conflicts


class EdgeHandler:
    """Handles hex edge processing based on selected mode."""

    # Derivation rules: tag -> default edge type
    TAG_EDGE_DEFAULTS = {
        "underground": EdgeType.TUNNEL,
        "surface": EdgeType.WILDERNESS,
        "underwater": EdgeType.WATER,
        "aerial": EdgeType.BLOCKED,
        "passage": None,  # Special handling
        "dead_end": EdgeType.BLOCKED,
        "entrance": EdgeType.ENTRANCE,
    }

    # Tags that should NOT have certain edge types
    TAG_EDGE_CONFLICTS = {
        "underground": {EdgeType.ROAD, EdgeType.WILDERNESS},
        "underwater": {EdgeType.ROAD, EdgeType.TUNNEL},
        "aerial": {EdgeType.TUNNEL, EdgeType.WATER},
    }

    def __init__(self, mode: EdgeMode = EdgeMode.EXPLICIT):
        self.mode = mode

    def process(self, hex: TaggedHex) -> TaggedHex:
        """Process hex edges according to mode.

        Args:
            hex: Input hex (may be modified in DERIVED/HYBRID modes)

        Returns:
            TaggedHex with processed edges
        """
        if self.mode == EdgeMode.EXPLICIT:
            return hex
        elif self.mode == EdgeMode.DERIVED:
            return self._derive_edges(hex)
        elif self.mode == EdgeMode.HYBRID:
            return self._heal_edges(hex)
        else:
            raise ValueError(f"Unknown mode: {self.mode}")

    def _derive_edges(self, hex: TaggedHex) -> TaggedHex:
        """Infer edges entirely from tags."""
        result = deepcopy(hex)
        tag_set = set(hex.tags)

        # Determine base edge type from terrain
        base_edge = EdgeType.WILDERNESS
        if "underground" in tag_set:
            base_edge = EdgeType.TUNNEL
        elif "underwater" in tag_set:
            base_edge = EdgeType.WATER
        elif "aerial" in tag_set:
            base_edge = EdgeType.BLOCKED

        # Start with base edges
        new_edges = [base_edge] * 6

        # Apply function modifiers
        if "passage" in tag_set:
            # Passage: at least 2 open edges on opposite sides
            pass  # Keep base edges (allows passage)
        elif "dead_end" in tag_set:
            # Dead end: only 1 non-blocked edge
            new_edges = [EdgeType.BLOCKED] * 6
            new_edges[0] = base_edge

        if "entrance" in tag_set:
            # Entrance: one edge is entrance type
            new_edges[0] = EdgeType.ENTRANCE

        result.edge_types = new_edges
        return result

    def _heal_edges(self, hex: TaggedHex) -> TaggedHex:
        """Heal edges that conflict with tags."""
        result = deepcopy(hex)
        tag_set = set(hex.tags)

        # Find conflicting edge types for this hex's tags
        forbidden_edges: set[EdgeType] = set()
        for tag in tag_set:
            if tag in self.TAG_EDGE_CONFLICTS:
                forbidden_edges.update(self.TAG_EDGE_CONFLICTS[tag])

        if not forbidden_edges:
            return result

        # Determine replacement edge type
        replacement = EdgeType.BLOCKED
        if "underground" in tag_set:
            replacement = EdgeType.TUNNEL
        elif "underwater" in tag_set:
            replacement = EdgeType.WATER
        elif "surface" in tag_set:
            replacement = EdgeType.WILDERNESS

        # Replace forbidden edges
        new_edges = []
        for edge in result.edge_types:
            if edge in forbidden_edges:
                new_edges.append(replacement)
            else:
                new_edges.append(edge)

        result.edge_types = new_edges
        return result
