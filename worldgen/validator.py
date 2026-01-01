"""Validation logic for generated hex data."""

from dataclasses import dataclass, field
from typing import Callable

from schemas import HexRegion, HexTile, TerrainType, GenerationSeed


@dataclass
class ValidationResult:
    """Result of validation check."""
    valid: bool
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)

    def add_error(self, msg: str):
        self.errors.append(msg)
        self.valid = False

    def add_warning(self, msg: str):
        self.warnings.append(msg)

    def merge(self, other: "ValidationResult"):
        self.errors.extend(other.errors)
        self.warnings.extend(other.warnings)
        if not other.valid:
            self.valid = False


class HexValidator:
    """Validates hex regions against game invariants."""

    def __init__(self):
        self.hex_checks: list[Callable[[HexTile], ValidationResult]] = [
            self._check_terrain_elevation,
            self._check_terrain_moisture,
            self._check_traversability,
            self._check_resource_limits,
        ]
        self.region_checks: list[Callable[[HexRegion], ValidationResult]] = [
            self._check_unique_coordinates,
            self._check_connectivity,
            self._check_terrain_distribution,
        ]

    def validate_hex(self, hex_tile: HexTile) -> ValidationResult:
        """Validate a single hex tile."""
        result = ValidationResult(valid=True)
        for check in self.hex_checks:
            result.merge(check(hex_tile))
        return result

    def validate_region(self, region: HexRegion) -> ValidationResult:
        """Validate an entire region."""
        result = ValidationResult(valid=True)

        for hex_tile in region.hexes:
            hex_result = self.validate_hex(hex_tile)
            if not hex_result.valid:
                for err in hex_result.errors:
                    result.add_error(f"Hex ({hex_tile.q},{hex_tile.r}): {err}")
            result.warnings.extend(hex_result.warnings)

        for check in self.region_checks:
            result.merge(check(region))

        return result

    def validate_seed(self, seed: GenerationSeed) -> ValidationResult:
        """Validate a complete generation seed."""
        result = ValidationResult(valid=True)

        for region in seed.regions:
            region_result = self.validate_region(region)
            if not region_result.valid:
                for err in region_result.errors:
                    result.add_error(f"Region '{region.name}': {err}")
            result.warnings.extend(region_result.warnings)

        return result

    def _check_terrain_elevation(self, h: HexTile) -> ValidationResult:
        """Mountains should be high elevation, water should be low."""
        result = ValidationResult(valid=True)

        if h.terrain == TerrainType.MOUNTAINS and h.elevation < 0.7:
            result.add_error(f"Mountains must have elevation >= 0.7, got {h.elevation}")

        if h.terrain == TerrainType.WATER and h.elevation > 0.3:
            result.add_error(f"Water must have elevation <= 0.3, got {h.elevation}")

        if h.terrain == TerrainType.HILLS and not (0.4 <= h.elevation <= 0.8):
            result.add_warning(f"Hills typically have elevation 0.4-0.8, got {h.elevation}")

        return result

    def _check_terrain_moisture(self, h: HexTile) -> ValidationResult:
        """Desert should be dry, swamp should be wet."""
        result = ValidationResult(valid=True)

        if h.terrain == TerrainType.DESERT and h.moisture > 0.2:
            result.add_error(f"Desert must have moisture <= 0.2, got {h.moisture}")

        if h.terrain == TerrainType.SWAMP and h.moisture < 0.7:
            result.add_error(f"Swamp must have moisture >= 0.7, got {h.moisture}")

        if h.terrain == TerrainType.WATER and h.moisture < 1.0:
            result.add_warning(f"Water typically has moisture 1.0, got {h.moisture}")

        return result

    def _check_traversability(self, h: HexTile) -> ValidationResult:
        """Mountains with very high elevation should be impassable."""
        result = ValidationResult(valid=True)

        if h.terrain == TerrainType.MOUNTAINS and h.elevation > 0.95 and h.traversable:
            result.add_warning("Very high mountains (>0.95) are usually impassable")

        if h.terrain == TerrainType.WATER and h.traversable:
            result.add_warning("Water hexes are typically not traversable by land units")

        return result

    def _check_resource_limits(self, h: HexTile) -> ValidationResult:
        """Check resource distribution makes sense."""
        result = ValidationResult(valid=True)

        if len(h.resources) > 3:
            result.add_error(f"Max 3 resources per hex, got {len(h.resources)}")

        from schemas import ResourceType
        if h.terrain == TerrainType.WATER:
            land_resources = [r for r in h.resources if r.type not in [ResourceType.FISH]]
            if land_resources:
                result.add_warning(f"Water hex has land resources: {[r.type.value for r in land_resources]}")

        return result

    def _check_unique_coordinates(self, region: HexRegion) -> ValidationResult:
        """All hex coordinates in a region must be unique."""
        result = ValidationResult(valid=True)

        coords = [(h.q, h.r) for h in region.hexes]
        seen = set()
        for coord in coords:
            if coord in seen:
                result.add_error(f"Duplicate coordinates: {coord}")
            seen.add(coord)

        return result

    def _check_connectivity(self, region: HexRegion) -> ValidationResult:
        """Check that all hexes are connected (optional, warning only)."""
        result = ValidationResult(valid=True)

        if len(region.hexes) < 2:
            return result

        coords = {(h.q, h.r) for h in region.hexes}

        def neighbors(q: int, r: int) -> list[tuple[int, int]]:
            return [
                (q + 1, r), (q - 1, r),
                (q, r + 1), (q, r - 1),
                (q + 1, r - 1), (q - 1, r + 1),
            ]

        start = next(iter(coords))
        visited = {start}
        frontier = [start]

        while frontier:
            current = frontier.pop()
            for neighbor in neighbors(*current):
                if neighbor in coords and neighbor not in visited:
                    visited.add(neighbor)
                    frontier.append(neighbor)

        if len(visited) != len(coords):
            disconnected = coords - visited
            result.add_warning(f"Region has {len(disconnected)} disconnected hexes")

        return result

    def _check_terrain_distribution(self, region: HexRegion) -> ValidationResult:
        """Check terrain variety is reasonable."""
        result = ValidationResult(valid=True)

        terrain_counts: dict[TerrainType, int] = {}
        for h in region.hexes:
            terrain_counts[h.terrain] = terrain_counts.get(h.terrain, 0) + 1

        total = len(region.hexes)
        for terrain, count in terrain_counts.items():
            ratio = count / total
            if ratio > 0.8 and total > 5:
                result.add_warning(f"Region is {ratio*100:.0f}% {terrain.value} - limited variety")

        return result


def quick_validate(region: HexRegion) -> bool:
    """Fast validation check - returns True if valid."""
    validator = HexValidator()
    result = validator.validate_region(region)
    return result.valid
