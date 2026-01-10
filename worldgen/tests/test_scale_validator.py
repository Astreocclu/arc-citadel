"""Tests for 100m scale validation."""
import pytest
from unittest.mock import Mock, patch
from schemas import TaggedHex, ScaleValidation
from scale_validator import ScaleValidator


@pytest.fixture
def validator():
    return ScaleValidator(threshold=7.0)


class TestScaleValidation:
    def test_good_scale_description_passes(self, validator):
        """Description fitting 100m scale should pass."""
        hex = TaggedHex(
            q=0, r=0,
            name="Forest Grove",
            description="A small clearing surrounded by ancient oaks. A moss-covered boulder sits at the center, with a spring bubbling nearby.",
            tags=["surface", "wild"],
            edge_types=["wilderness"] * 6,
        )

        with patch.object(validator, '_call_llm') as mock_llm:
            mock_llm.return_value = {"score": 9, "feedback": "Good 100m scale"}
            result = validator.validate(hex)

        assert result.passes
        assert result.score >= 7.0

    def test_too_large_description_fails(self, validator):
        """Description suggesting region-scale should fail."""
        hex = TaggedHex(
            q=0, r=0,
            name="The Great Forest",
            description="A vast forest stretching for miles, home to countless creatures and ancient secrets lost to time.",
            tags=["surface", "wild"],
            edge_types=["wilderness"] * 6,
        )

        with patch.object(validator, '_call_llm') as mock_llm:
            mock_llm.return_value = {"score": 3, "feedback": "Too large - describes miles"}
            result = validator.validate(hex)

        assert not result.passes
        assert result.score < 7.0

    def test_too_small_description_fails(self, validator):
        """Description suggesting room-scale should fail."""
        hex = TaggedHex(
            q=0, r=0,
            name="Storage Closet",
            description="A small closet with a few shelves holding dusty bottles.",
            tags=["underground", "dwarf", "residential"],
            edge_types=["tunnel", "blocked", "blocked", "blocked", "blocked", "blocked"],
        )

        with patch.object(validator, '_call_llm') as mock_llm:
            mock_llm.return_value = {"score": 2, "feedback": "Too small - a closet is not 100m"}
            result = validator.validate(hex)

        assert not result.passes


class TestScalePrompt:
    def test_prompt_includes_scale_context(self, validator):
        """Validation prompt should mention 100m scale."""
        hex = TaggedHex(
            q=0, r=0,
            name="Test",
            description="Test description",
            tags=["surface"],
            edge_types=["wilderness"] * 6,
        )

        prompt = validator._build_prompt(hex)
        assert "100m" in prompt or "100 meter" in prompt.lower()
        assert "10-15 minutes" in prompt
