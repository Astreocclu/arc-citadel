"""Constraint propagation filler generation."""

import random
import tomllib
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

from worldgen.schemas.base import HexCoord, Terrain, SpeciesFitness
from worldgen.schemas.world import WorldHex, HexMap
from worldgen.hex_coords import get_all_neighbors, coords_to_key, key_to_coords


@dataclass
class WaveCell:
    """Wave function cell for constraint propagation."""
    possible_tags: set[str] = field(default_factory=set)
    collapsed: bool = False
    chosen_tag: Optional[str] = None


class FillerGenerator:
    """Constraint propagation for filler hex generation."""

    def __init__(
        self,
        tags_path: str | Path | None = None,
        max_iterations: int = 10000,
    ):
        if tags_path is None:
            # Use path relative to this module
            tags_path = Path(__file__).parent / "generation" / "hex_tags.toml"
        with open(tags_path, "rb") as f:
            self.config = tomllib.load(f)

        self.max_iterations = max_iterations
        self.transitions = self.config.get("transitions", {})
        self.weights = self.config.get("transitions", {}).get("weights", {})

        # All possible terrain tags
        self.all_terrain_tags = set(self.config["categories"]["TERRAIN"]["values"])

    def generate_fillers(
        self,
        hex_map: HexMap,
        world_radius: int,
    ) -> HexMap:
        """Fill empty hexes using constraint propagation.

        Args:
            hex_map: HexMap with clusters and connectors already placed
            world_radius: Maximum world radius for hex coordinates

        Returns:
            HexMap with filler hexes added
        """
        # 1. Identify fixed hexes (clusters + connectors)
        fixed_positions = set(hex_map.hexes.keys())

        # 2. Identify all empty positions within world radius
        empty_positions = self._find_empty_positions(fixed_positions, world_radius)

        if not empty_positions:
            return hex_map

        # 3. Initialize wave function for each empty position
        wave = self._initialize_wave(empty_positions, fixed_positions, hex_map)

        # 4. Build frontier from positions adjacent to fixed hexes
        frontier = self._build_initial_frontier(empty_positions, fixed_positions)

        # 5. Constraint propagation loop
        iteration = 0
        while frontier and iteration < self.max_iterations:
            # Get position with minimum entropy (fewest possibilities)
            pos_key = self._select_min_entropy(frontier, wave)
            if pos_key is None:
                break

            frontier.remove(pos_key)
            cell = wave[pos_key]

            if cell.collapsed:
                continue

            # Collapse wave function
            chosen_terrain = self._collapse(cell, pos_key, wave, hex_map)

            if chosen_terrain is None:
                # Contradiction - use fallback
                chosen_terrain = "surface"

            cell.chosen_tag = chosen_terrain
            cell.collapsed = True

            # Create filler hex
            q, r = key_to_coords(pos_key)
            filler_hex = self._create_filler_hex(q, r, chosen_terrain)
            hex_map.hexes[pos_key] = filler_hex

            # Add uncollapsed neighbors to frontier
            for nq, nr, _ in get_all_neighbors(q, r):
                nkey = coords_to_key(nq, nr)
                if nkey in wave and not wave[nkey].collapsed and nkey not in frontier:
                    # Propagate constraints to neighbor
                    self._propagate_constraints(nkey, wave, hex_map)
                    frontier.add(nkey)

            iteration += 1

        # 6. Fill any remaining empty hexes with fallback
        for pos_key, cell in wave.items():
            if not cell.collapsed:
                q, r = key_to_coords(pos_key)
                filler_hex = self._create_filler_hex(q, r, "surface")
                hex_map.hexes[pos_key] = filler_hex

        return hex_map

    def _find_empty_positions(
        self,
        fixed: set[str],
        world_radius: int,
    ) -> set[str]:
        """Find all empty hex positions within world radius."""
        empty = set()

        for q in range(-world_radius, world_radius + 1):
            for r in range(-world_radius, world_radius + 1):
                # Check hex is within world bounds (using axial distance)
                s = -q - r
                if max(abs(q), abs(r), abs(s)) <= world_radius:
                    key = coords_to_key(q, r)
                    if key not in fixed:
                        empty.add(key)

        return empty

    def _initialize_wave(
        self,
        empty: set[str],
        fixed: set[str],
        hex_map: HexMap,
    ) -> dict[str, WaveCell]:
        """Initialize wave function with all possibilities."""
        wave = {}

        for pos_key in empty:
            q, r = key_to_coords(pos_key)

            # Start with all terrain tags possible
            possible = set(self.all_terrain_tags)

            # Constrain by adjacent fixed hexes
            for nq, nr, _ in get_all_neighbors(q, r):
                nkey = coords_to_key(nq, nr)
                if nkey in fixed and nkey in hex_map.hexes:
                    neighbor_hex = hex_map.hexes[nkey]
                    neighbor_terrain = self._extract_terrain_tag(neighbor_hex)

                    if neighbor_terrain and neighbor_terrain in self.transitions:
                        compatible = set(self.transitions[neighbor_terrain])
                        possible &= compatible

            # Ensure at least one possibility
            if not possible:
                possible = {"surface"}

            wave[pos_key] = WaveCell(possible_tags=possible)

        return wave

    def _build_initial_frontier(
        self,
        empty: set[str],
        fixed: set[str],
    ) -> set[str]:
        """Build initial frontier of empty hexes adjacent to fixed hexes."""
        frontier = set()

        for pos_key in empty:
            q, r = key_to_coords(pos_key)

            for nq, nr, _ in get_all_neighbors(q, r):
                nkey = coords_to_key(nq, nr)
                if nkey in fixed:
                    frontier.add(pos_key)
                    break

        return frontier

    def _select_min_entropy(
        self,
        frontier: set[str],
        wave: dict[str, WaveCell],
    ) -> Optional[str]:
        """Select position with minimum entropy (fewest possibilities)."""
        min_entropy = float('inf')
        min_pos = None

        for pos_key in frontier:
            cell = wave.get(pos_key)
            if cell and not cell.collapsed:
                entropy = len(cell.possible_tags)
                # Add small random factor to break ties
                entropy += random.random() * 0.1
                if entropy < min_entropy:
                    min_entropy = entropy
                    min_pos = pos_key

        return min_pos

    def _collapse(
        self,
        cell: WaveCell,
        pos_key: str,
        wave: dict[str, WaveCell],
        hex_map: HexMap,
    ) -> Optional[str]:
        """Collapse wave function to single terrain."""
        if not cell.possible_tags:
            return None

        # Weight by transition weights if available
        weighted_choices = []
        for tag in cell.possible_tags:
            weight = 1.0

            # Check neighbor-based weights
            q, r = key_to_coords(pos_key)
            for nq, nr, _ in get_all_neighbors(q, r):
                nkey = coords_to_key(nq, nr)

                # Check collapsed wave cells
                if nkey in wave and wave[nkey].collapsed:
                    neighbor_tag = wave[nkey].chosen_tag
                    weight_key = f"{neighbor_tag}.{tag}"
                    if weight_key in self.weights:
                        weight *= self.weights[weight_key]

                # Check fixed hexes
                if nkey in hex_map.hexes:
                    neighbor_hex = hex_map.hexes[nkey]
                    neighbor_tag = self._extract_terrain_tag(neighbor_hex)
                    if neighbor_tag:
                        weight_key = f"{neighbor_tag}.{tag}"
                        if weight_key in self.weights:
                            weight *= self.weights[weight_key]

            weighted_choices.append((tag, weight))

        # Weighted random selection
        total_weight = sum(w for _, w in weighted_choices)
        if total_weight <= 0:
            return random.choice(list(cell.possible_tags))

        rand = random.random() * total_weight
        cumulative = 0.0
        for tag, weight in weighted_choices:
            cumulative += weight
            if rand <= cumulative:
                return tag

        return weighted_choices[-1][0]

    def _propagate_constraints(
        self,
        pos_key: str,
        wave: dict[str, WaveCell],
        hex_map: HexMap,
    ):
        """Propagate constraints to a cell from its neighbors."""
        cell = wave.get(pos_key)
        if not cell or cell.collapsed:
            return

        q, r = key_to_coords(pos_key)

        for nq, nr, _ in get_all_neighbors(q, r):
            nkey = coords_to_key(nq, nr)

            # Constraint from collapsed wave cells
            if nkey in wave and wave[nkey].collapsed:
                neighbor_tag = wave[nkey].chosen_tag
                if neighbor_tag and neighbor_tag in self.transitions:
                    compatible = set(self.transitions[neighbor_tag])
                    cell.possible_tags &= compatible

            # Constraint from fixed hexes
            if nkey in hex_map.hexes:
                neighbor_hex = hex_map.hexes[nkey]
                neighbor_tag = self._extract_terrain_tag(neighbor_hex)
                if neighbor_tag and neighbor_tag in self.transitions:
                    compatible = set(self.transitions[neighbor_tag])
                    cell.possible_tags &= compatible

        # Ensure at least one possibility
        if not cell.possible_tags:
            cell.possible_tags = {"surface"}

    def _extract_terrain_tag(self, hex: WorldHex) -> Optional[str]:
        """Extract terrain tag from WorldHex."""
        # WorldHex uses Terrain enum, map to tag
        terrain_to_tag = {
            "underground": "underground",
            "cavern": "underground",
            "deep_water": "underwater",
            "shallow_water": "surface",
            "plains": "surface",
            "hills": "surface",
            "forest": "surface",
            "dense_forest": "surface",
            "mountains": "surface",
            "high_mountains": "elevated",
            "marsh": "surface",
            "desert": "surface",
            "tundra": "surface",
            "volcanic": "surface",
            "glacier": "surface",
        }
        return terrain_to_tag.get(hex.terrain.value, "surface")

    def _create_filler_hex(self, q: int, r: int, terrain_tag: str) -> WorldHex:
        """Create a filler WorldHex."""
        # Map tag to Terrain enum
        tag_to_terrain = {
            "surface": Terrain.PLAINS,
            "underground": Terrain.UNDERGROUND,
            "underwater": Terrain.DEEP_WATER,
            "aerial": Terrain.HIGH_MOUNTAINS,
            "elevated": Terrain.HILLS,
            "peak": Terrain.HIGH_MOUNTAINS,
        }
        terrain = tag_to_terrain.get(terrain_tag, Terrain.PLAINS)

        # Vary elevation based on terrain
        if terrain_tag == "elevated":
            elevation = 400.0 + random.random() * 200.0
        elif terrain_tag == "peak":
            elevation = 800.0 + random.random() * 400.0
        elif terrain_tag == "underground":
            elevation = -100.0 - random.random() * 200.0
        else:
            elevation = 100.0 + random.random() * 100.0

        return WorldHex(
            coord=HexCoord(q=q, r=r),
            terrain=terrain,
            elevation=elevation,
            moisture=0.4 + random.random() * 0.2,
            temperature=15.0,
            species_fitness=SpeciesFitness(human=0.5, dwarf=0.5, elf=0.5),
        )
