"""Tests for adjacency validation."""
import pytest
from schemas import TaggedHex, EdgeType, FoundingContext
from adjacency import AdjacencyValidator, ValidationResult


@pytest.fixture
def validator():
    return AdjacencyValidator("generation/hex_tags.toml")


@pytest.fixture
def underground_hex():
    return TaggedHex(
        q=0, r=0,
        name="Dwarf Mine",
        description="Underground mining chamber",
        tags=["underground", "dwarf", "industrial"],
        edge_types=["tunnel", "tunnel", "blocked", "blocked", "blocked", "blocked"],
    )


@pytest.fixture
def surface_hex():
    return TaggedHex(
        q=1, r=0,
        name="Forest Clearing",
        description="Surface clearing",
        tags=["surface", "wild"],
        edge_types=["wilderness", "wilderness", "wilderness", "wilderness", "wilderness", "wilderness"],
    )


@pytest.fixture
def aerial_hex():
    return TaggedHex(
        q=0, r=-1,
        name="Cloud Platform",
        description="Floating platform",
        tags=["aerial", "ancient"],
        edge_types=["blocked", "blocked", "blocked", "blocked", "blocked", "blocked"],
    )


class TestHardConstraints:
    def test_underground_cannot_adjoin_aerial(self, validator, underground_hex, aerial_hex):
        """Underground hex cannot be adjacent to aerial hex."""
        result = validator.validate_adjacency(underground_hex, aerial_hex, edge=1)
        assert not result.valid
        assert "TERRAIN" in result.errors[0]

    def test_edge_types_must_match(self, validator, underground_hex):
        """Tunnel edge must connect to tunnel edge."""
        mismatched = TaggedHex(
            q=1, r=0,
            name="Bad Neighbor",
            description="Has road where tunnel expected",
            tags=["underground", "dwarf"],
            edge_types=["road", "tunnel", "blocked", "blocked", "blocked", "blocked"],
        )
        result = validator.validate_adjacency(underground_hex, mismatched, edge=0)
        assert not result.valid
        assert "edge" in result.errors[0].lower()


class TestSoftConstraints:
    def test_dwarf_elf_adjacency_warns(self, validator):
        """Dwarf adjacent to elf generates warning, not error."""
        dwarf_hex = TaggedHex(
            q=0, r=0,
            name="Dwarf Hold",
            description="Dwarven area",
            tags=["underground", "dwarf", "residential"],
            edge_types=["tunnel"] * 6,
        )
        elf_hex = TaggedHex(
            q=1, r=0,
            name="Elf Outpost",
            description="Elven area",
            tags=["underground", "elf", "residential"],
            edge_types=["tunnel"] * 6,
        )
        result = validator.validate_adjacency(dwarf_hex, elf_hex, edge=0)
        assert result.valid  # Soft constraint doesn't block
        assert len(result.warnings) > 0
        assert "culture" in result.warnings[0].lower()


class TestValidAdjacencies:
    def test_matching_tunnels_valid(self, validator):
        """Two underground hexes with matching tunnel edges are valid."""
        hex_a = TaggedHex(
            q=0, r=0,
            name="Tunnel A",
            description="Tunnel section",
            tags=["underground", "passage"],
            edge_types=["tunnel", "blocked", "blocked", "blocked", "blocked", "blocked"],
        )
        hex_b = TaggedHex(
            q=1, r=0,
            name="Tunnel B",
            description="Tunnel section",
            tags=["underground", "passage"],
            edge_types=["blocked", "blocked", "blocked", "tunnel", "blocked", "blocked"],
        )
        result = validator.validate_adjacency(hex_a, hex_b, edge=0)
        assert result.valid
        assert len(result.errors) == 0


class TestFoundingConstraints:
    """Tests for founding condition validation."""

    def test_bias_against_creates_error(self, validator, surface_hex):
        """Hex with forbidden tag should fail validation."""
        founding = FoundingContext(
            season="winter",
            bias_against=["wild"],  # surface_hex has "wild" tag
        )

        result = validator.validate_founding_constraints(surface_hex, founding)

        assert not result.valid
        assert len(result.errors) == 1
        assert "FOUNDING" in result.errors[0]
        assert "wild" in result.errors[0]

    def test_missing_preferred_tags_creates_warning(self, validator, surface_hex):
        """Hex missing preferred tags should create warning, not error."""
        founding = FoundingContext(
            season="winter",
            bias_tags=["military", "defensive"],  # surface_hex has neither
        )

        result = validator.validate_founding_constraints(surface_hex, founding)

        assert result.valid  # Missing preferred is soft constraint
        assert len(result.warnings) == 1
        assert "FOUNDING suggestion" in result.warnings[0]

    def test_no_founding_context_always_valid(self, validator, surface_hex):
        """Without founding context, all hexes are valid."""
        result = validator.validate_founding_constraints(surface_hex, None)

        assert result.valid
        assert len(result.errors) == 0
        assert len(result.warnings) == 0

    def test_hex_with_preferred_tags_no_warning(self, validator):
        """Hex with preferred tags should have no warnings."""
        military_hex = TaggedHex(
            q=0, r=0,
            name="Fort",
            description="Military outpost",
            tags=["surface", "human", "military"],
            edge_types=["road"] * 6,
        )
        founding = FoundingContext(
            season="winter",
            bias_tags=["military", "defensive"],
        )

        result = validator.validate_founding_constraints(military_hex, founding)

        assert result.valid
        assert len(result.warnings) == 0

    def test_multiple_forbidden_tags(self, validator):
        """Multiple forbidden tags should all be reported."""
        hex_with_many = TaggedHex(
            q=0, r=0,
            name="Mixed",
            description="Has multiple forbidden tags",
            tags=["surface", "commercial", "exposed"],
            edge_types=["road"] * 6,
        )
        founding = FoundingContext(
            season="winter",
            bias_against=["commercial", "exposed"],
        )

        result = validator.validate_founding_constraints(hex_with_many, founding)

        assert not result.valid
        assert "commercial" in result.errors[0] or "exposed" in result.errors[0]
