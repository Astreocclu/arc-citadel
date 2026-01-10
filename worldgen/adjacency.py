"""Adjacency validation for tag-based hex composition."""

import tomllib
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

from schemas import TaggedHex, EdgeType, FoundingContext
from hex_coords import get_opposite_edge


@dataclass
class ValidationResult:
    """Result of adjacency validation."""
    valid: bool
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)


class AdjacencyValidator:
    """Validates hex adjacencies using tag constraints."""

    # Edge types that must match between adjacent hexes
    MATCHING_EDGES = {EdgeType.TUNNEL, EdgeType.ROAD, EdgeType.WATER}

    def __init__(self, tags_path: str | Path):
        """Load constraints from hex_tags.toml."""
        with open(tags_path, "rb") as f:
            self.config = tomllib.load(f)

        self.hard_constraints = self.config.get("constraints", {}).get("hard", {})
        self.soft_constraints = self.config.get("constraints", {}).get("soft", {})

        # Build lookup sets for faster validation
        self.incompatible_terrain = {
            frozenset(pair) for pair in self.hard_constraints.get("incompatible_terrain", [])
        }
        self.culture_tensions = {
            frozenset(pair) for pair in self.soft_constraints.get("culture_tensions", [])
        }
        self.function_clashes = {
            frozenset(pair) for pair in self.soft_constraints.get("function_clashes", [])
        }

    def validate_adjacency(
        self,
        hex_a: TaggedHex,
        hex_b: TaggedHex,
        edge: int,
    ) -> ValidationResult:
        """Validate that two hexes can be adjacent.

        Args:
            hex_a: First hex
            hex_b: Second hex (adjacent to hex_a)
            edge: Edge index from hex_a to hex_b (0-5)

        Returns:
            ValidationResult with valid flag, errors, and warnings
        """
        errors: list[str] = []
        warnings: list[str] = []

        # 1. HARD: Check terrain compatibility
        terrain_a = self._get_terrain_tag(hex_a.tags)
        terrain_b = self._get_terrain_tag(hex_b.tags)

        if terrain_a and terrain_b:
            pair = frozenset([terrain_a, terrain_b])
            if pair in self.incompatible_terrain:
                errors.append(f"TERRAIN incompatible: {terrain_a} cannot adjoin {terrain_b}")

        # 2. HARD: Check edge type matching
        edge_from_a = hex_a.edge_types[edge]
        opposite_edge = get_opposite_edge(edge)
        edge_from_b = hex_b.edge_types[opposite_edge]

        if edge_from_a in self.MATCHING_EDGES or edge_from_b in self.MATCHING_EDGES:
            if edge_from_a != edge_from_b:
                errors.append(
                    f"Edge mismatch: hex_a edge {edge}={edge_from_a.value}, "
                    f"hex_b edge {opposite_edge}={edge_from_b.value}"
                )

        # 3. SOFT: Check culture tensions
        cultures_a = self._get_culture_tags(hex_a.tags)
        cultures_b = self._get_culture_tags(hex_b.tags)

        for ca in cultures_a:
            for cb in cultures_b:
                pair = frozenset([ca, cb])
                if pair in self.culture_tensions:
                    warnings.append(f"Culture tension: {ca} adjacent to {cb}")

        # 4. SOFT: Check function clashes
        functions_a = self._get_function_tags(hex_a.tags)
        functions_b = self._get_function_tags(hex_b.tags)

        for fa in functions_a:
            for fb in functions_b:
                pair = frozenset([fa, fb])
                if pair in self.function_clashes:
                    warnings.append(f"Function clash: {fa} adjacent to {fb}")

        return ValidationResult(
            valid=len(errors) == 0,
            errors=errors,
            warnings=warnings,
        )

    def _get_terrain_tag(self, tags: list[str]) -> str | None:
        """Extract TERRAIN tag from list."""
        terrain_values = set(self.config["categories"]["TERRAIN"]["values"])
        for tag in tags:
            if tag in terrain_values:
                return tag
        return None

    def _get_culture_tags(self, tags: list[str]) -> list[str]:
        """Extract CULTURE tags from list."""
        culture_values = set(self.config["categories"]["CULTURE"]["values"])
        return [tag for tag in tags if tag in culture_values]

    def _get_function_tags(self, tags: list[str]) -> list[str]:
        """Extract FUNCTION tags from list."""
        function_values = set(self.config["categories"]["FUNCTION"]["values"])
        return [tag for tag in tags if tag in function_values]

    def validate_hex_internal(self, hex: TaggedHex) -> ValidationResult:
        """Validate a hex's internal tag consistency.

        Checks required pairings (e.g., 'deep' requires 'underground').
        """
        errors: list[str] = []
        warnings: list[str] = []

        tag_set = set(hex.tags)

        for requirement in self.hard_constraints.get("requires", []):
            tag = requirement["tag"]
            if tag not in tag_set:
                continue

            if "requires" in requirement:
                required = requirement["requires"]
                if required not in tag_set:
                    errors.append(f"Tag '{tag}' requires '{required}'")

            if "requires_one_of" in requirement:
                options = requirement["requires_one_of"]
                if not any(opt in tag_set for opt in options):
                    errors.append(f"Tag '{tag}' requires one of {options}")

        return ValidationResult(
            valid=len(errors) == 0,
            errors=errors,
            warnings=warnings,
        )

    def validate_founding_constraints(
        self,
        hex: TaggedHex,
        founding_context: Optional[FoundingContext],
    ) -> ValidationResult:
        """Validate hex against founding conditions.

        HARD constraints:
        - Hex must NOT contain any tags from bias_against

        SOFT constraints (warnings):
        - Hex SHOULD contain at least one tag from bias_tags

        Args:
            hex: The hex to validate
            founding_context: Founding conditions (if any)

        Returns:
            ValidationResult with errors for bias_against violations,
            warnings for missing bias_tags
        """
        if founding_context is None:
            return ValidationResult(valid=True)

        errors: list[str] = []
        warnings: list[str] = []
        tag_set = set(hex.tags)

        # HARD: Check for forbidden tags (bias_against)
        forbidden = set(founding_context.bias_against)
        violations = tag_set & forbidden
        if violations:
            errors.append(
                f"FOUNDING violation: hex contains forbidden tags {list(violations)}. "
                f"Settlement founding conditions forbid: {founding_context.bias_against}"
            )

        # SOFT: Check for preferred tags (bias_tags)
        preferred = set(founding_context.bias_tags)
        if preferred and not (tag_set & preferred):
            warnings.append(
                f"FOUNDING suggestion: hex has none of preferred tags {founding_context.bias_tags}"
            )

        return ValidationResult(
            valid=len(errors) == 0,
            errors=errors,
            warnings=warnings,
        )
