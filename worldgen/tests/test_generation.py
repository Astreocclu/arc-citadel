"""Tests for LLM client and quality loop generation."""

import json
from unittest.mock import MagicMock, patch

import pytest

from worldgen.generation.llm_client import DeepSeekClient
from worldgen.generation.quality_loop import QualityGenerator, ScoringResult
from worldgen.schemas import ComponentCategory


class TestDeepSeekClient:
    """Tests for DeepSeekClient."""

    def test_init_requires_api_key(self):
        """Client should raise if no API key is provided."""
        with patch.dict("os.environ", {"DEEPSEEK_API_KEY": ""}, clear=True):
            # Need to reload config to pick up empty env var
            import worldgen.config as config_module

            original_key = config_module.DEEPSEEK_API_KEY
            config_module.DEEPSEEK_API_KEY = ""
            try:
                with pytest.raises(ValueError, match="DEEPSEEK_API_KEY not set"):
                    DeepSeekClient(api_key="")
            finally:
                config_module.DEEPSEEK_API_KEY = original_key

    def test_init_with_explicit_api_key(self):
        """Client should accept explicit API key."""
        with patch("worldgen.generation.llm_client.OpenAI") as mock_openai:
            client = DeepSeekClient(api_key="test-key")
            assert client.api_key == "test-key"
            mock_openai.assert_called_once_with(
                api_key="test-key",
                base_url="https://api.deepseek.com",
            )

    def test_generate_calls_openai_api(self):
        """Generate should call the OpenAI-compatible API."""
        with patch("worldgen.generation.llm_client.OpenAI") as mock_openai:
            mock_client = MagicMock()
            mock_openai.return_value = mock_client
            mock_response = MagicMock()
            mock_response.choices = [MagicMock(message=MagicMock(content="test output"))]
            mock_client.chat.completions.create.return_value = mock_response

            client = DeepSeekClient(api_key="test-key")
            result = client.generate("test prompt")

            assert result == "test output"
            mock_client.chat.completions.create.assert_called_once()

    def test_generate_json_parses_response(self):
        """Generate JSON should parse valid JSON response."""
        with patch("worldgen.generation.llm_client.OpenAI") as mock_openai:
            mock_client = MagicMock()
            mock_openai.return_value = mock_client
            mock_response = MagicMock()
            mock_response.choices = [
                MagicMock(message=MagicMock(content='{"key": "value"}'))
            ]
            mock_client.chat.completions.create.return_value = mock_response

            client = DeepSeekClient(api_key="test-key")
            result = client.generate_json("test prompt")

            assert result == {"key": "value"}

    def test_clean_and_parse_json_handles_markdown(self):
        """Should strip markdown code blocks from JSON."""
        with patch("worldgen.generation.llm_client.OpenAI") as mock_openai:
            mock_openai.return_value = MagicMock()
            client = DeepSeekClient(api_key="test-key")

            # Test with markdown code blocks
            content = '```json\n{"key": "value"}\n```'
            result = client._clean_and_parse_json(content)
            assert result == {"key": "value"}

            # Test with just ``` without json
            content = '```\n{"key": "value2"}\n```'
            result = client._clean_and_parse_json(content)
            assert result == {"key": "value2"}

    def test_clean_and_parse_json_handles_json_prefix(self):
        """Should handle 'json' prefix in content."""
        with patch("worldgen.generation.llm_client.OpenAI") as mock_openai:
            mock_openai.return_value = MagicMock()
            client = DeepSeekClient(api_key="test-key")

            content = 'json{"key": "value"}'
            result = client._clean_and_parse_json(content)
            assert result == {"key": "value"}


class TestScoringResult:
    """Tests for ScoringResult dataclass."""

    def test_from_dict(self):
        """Should create ScoringResult from dictionary."""
        data = {
            "strategic_score": 8,
            "narrative_score": 7,
            "authenticity_score": 9,
            "sensory_score": 6,
            "overall_score": 7.5,
            "strengths": ["good detail", "creative"],
            "weaknesses": ["too long"],
            "improvement_suggestions": ["be concise"],
        }
        result = ScoringResult.from_dict(data)

        assert result.strategic_score == 8.0
        assert result.narrative_score == 7.0
        assert result.authenticity_score == 9.0
        assert result.sensory_score == 6.0
        assert result.overall_score == 7.5
        assert result.strengths == ["good detail", "creative"]
        assert result.weaknesses == ["too long"]
        assert result.improvement_suggestions == ["be concise"]

    def test_from_dict_with_missing_fields(self):
        """Should handle missing fields with defaults."""
        data = {"overall_score": 5}
        result = ScoringResult.from_dict(data)

        assert result.strategic_score == 0.0
        assert result.overall_score == 5.0
        assert result.strengths == []


class TestQualityGenerator:
    """Tests for QualityGenerator."""

    def test_init_defaults(self):
        """Should initialize with default config values."""
        generator = QualityGenerator()

        assert generator.target_score == 9.0  # DEFAULT_TARGET_SCORE
        assert generator.max_iterations == 10  # MAX_QUALITY_ITERATIONS
        assert generator.candidates_per_round == 3  # CANDIDATES_PER_ROUND
        assert generator.client is None  # Lazy initialization

    def test_init_custom_values(self):
        """Should accept custom configuration."""
        mock_client = MagicMock()
        generator = QualityGenerator(
            target_score=8.0,
            max_iterations=5,
            candidates_per_round=2,
            client=mock_client,
        )

        assert generator.target_score == 8.0
        assert generator.max_iterations == 5
        assert generator.candidates_per_round == 2
        assert generator.client is mock_client

    def test_score_asset(self):
        """Should score an asset using the LLM."""
        mock_client = MagicMock()
        mock_client.generate_json.return_value = {
            "strategic_score": 7,
            "narrative_score": 8,
            "authenticity_score": 6,
            "sensory_score": 7,
            "overall_score": 7,
            "strengths": ["good"],
            "weaknesses": ["bad"],
            "improvement_suggestions": ["fix"],
        }

        generator = QualityGenerator(client=mock_client)
        asset = {"name": "Test Forge", "description": "A forge"}
        result = generator.score_asset(asset, "dwarf forge", "dwarf")

        assert result.overall_score == 7.0
        assert result.strengths == ["good"]
        mock_client.generate_json.assert_called_once()

    def test_generate_with_quality_reaches_target(self):
        """Should return when target score is reached."""
        mock_client = MagicMock()
        # First generation returns high score
        mock_client.generate_json.side_effect = [
            {"name": "Good Forge", "desc": "A great forge"},  # Candidate
            {  # Score
                "strategic_score": 9,
                "narrative_score": 9,
                "authenticity_score": 9,
                "sensory_score": 9,
                "overall_score": 9.0,
                "strengths": ["excellent"],
                "weaknesses": [],
                "improvement_suggestions": [],
            },
        ]

        generator = QualityGenerator(
            target_score=9.0,
            max_iterations=5,
            candidates_per_round=1,
            client=mock_client,
        )

        result, score = generator.generate_with_quality(
            prompt_template="Generate a forge",
            asset_type="dwarf forge",
            species="dwarf",
        )

        assert result is not None
        assert result["name"] == "Good Forge"
        assert score is not None
        assert score.overall_score == 9.0
        # Should have stopped after first iteration
        assert generator.stats.total_generated == 1
        assert generator.stats.total_iterations == 1

    def test_generate_with_quality_iterates_on_low_score(self):
        """Should iterate when score is below target."""
        mock_client = MagicMock()
        # First round: low score, second round: high score
        mock_client.generate_json.side_effect = [
            # Round 1 - low score
            {"name": "Basic Forge"},  # Candidate 1
            {
                "strategic_score": 5,
                "narrative_score": 5,
                "authenticity_score": 5,
                "sensory_score": 5,
                "overall_score": 5.0,
                "strengths": ["ok"],
                "weaknesses": ["generic"],
                "improvement_suggestions": ["add detail"],
            },
            # Round 2 - high score after improvement prompt
            {"name": "Amazing Forge"},  # Candidate 1 (improved)
            {
                "strategic_score": 9,
                "narrative_score": 9,
                "authenticity_score": 9,
                "sensory_score": 9,
                "overall_score": 9.0,
                "strengths": ["excellent"],
                "weaknesses": [],
                "improvement_suggestions": [],
            },
        ]

        generator = QualityGenerator(
            target_score=9.0,
            max_iterations=5,
            candidates_per_round=1,
            client=mock_client,
        )

        result, score = generator.generate_with_quality(
            prompt_template="Generate a forge",
            asset_type="dwarf forge",
            species="dwarf",
        )

        assert result is not None
        assert result["name"] == "Amazing Forge"
        assert score is not None
        assert score.overall_score == 9.0
        assert generator.stats.total_iterations == 2

    def test_generate_with_quality_picks_best_candidate(self):
        """Should pick the best candidate from multiple per round."""
        mock_client = MagicMock()
        mock_client.generate_json.side_effect = [
            # 3 candidates in round 1
            {"name": "Forge A"},
            {"overall_score": 5.0, "strengths": [], "weaknesses": [], "improvement_suggestions": [],
             "strategic_score": 5, "narrative_score": 5, "authenticity_score": 5, "sensory_score": 5},
            {"name": "Forge B"},
            {"overall_score": 9.5, "strengths": ["best"], "weaknesses": [], "improvement_suggestions": [],
             "strategic_score": 9, "narrative_score": 9, "authenticity_score": 9, "sensory_score": 9},
            {"name": "Forge C"},
            {"overall_score": 7.0, "strengths": [], "weaknesses": [], "improvement_suggestions": [],
             "strategic_score": 7, "narrative_score": 7, "authenticity_score": 7, "sensory_score": 7},
        ]

        generator = QualityGenerator(
            target_score=9.0,
            max_iterations=5,
            candidates_per_round=3,
            client=mock_client,
        )

        result, score = generator.generate_with_quality(
            prompt_template="Generate a forge",
            asset_type="dwarf forge",
            species="dwarf",
        )

        assert result is not None
        assert result["name"] == "Forge B"  # The best one
        assert score is not None
        assert score.overall_score == 9.5

    def test_generate_with_quality_returns_best_after_max_iterations(self):
        """Should return best result after exhausting iterations."""
        mock_client = MagicMock()
        # Always return score below target
        mock_client.generate_json.side_effect = [
            {"name": "OK Forge"},
            {
                "strategic_score": 7,
                "narrative_score": 7,
                "authenticity_score": 7,
                "sensory_score": 7,
                "overall_score": 7.0,
                "strengths": ["decent"],
                "weaknesses": ["not great"],
                "improvement_suggestions": ["improve"],
            },
        ] * 10  # Repeat for all iterations

        generator = QualityGenerator(
            target_score=9.0,
            max_iterations=2,
            candidates_per_round=1,
            client=mock_client,
        )

        result, score = generator.generate_with_quality(
            prompt_template="Generate a forge",
            asset_type="dwarf forge",
            species="dwarf",
        )

        assert result is not None
        assert score is not None
        assert score.overall_score == 7.0  # Best we got
        assert generator.stats.total_iterations == 2

    def test_generate_component(self):
        """Should generate a component with quality score added."""
        mock_client = MagicMock()
        mock_client.generate_json.side_effect = [
            {"name_fragment": "Deep Forge", "narrative_hook": "Ancient halls"},
            {
                "strategic_score": 9,
                "narrative_score": 9,
                "authenticity_score": 9,
                "sensory_score": 9,
                "overall_score": 9.0,
                "strengths": ["immersive", "authentic"],
                "weaknesses": [],
                "improvement_suggestions": [],
            },
        ]

        generator = QualityGenerator(
            target_score=9.0,
            candidates_per_round=1,
            client=mock_client,
        )

        result = generator.generate_component(
            category=ComponentCategory.DWARF_HOLD_FORGE,
            prompt_template="Generate a dwarf forge",
            index=1,
        )

        assert result is not None
        assert result["name_fragment"] == "Deep Forge"
        assert result["quality_score"] == 9.0
        assert "Strengths:" in result["generation_notes"]

    def test_improvement_prompt_includes_feedback(self):
        """Improvement prompt should include feedback from scoring."""
        generator = QualityGenerator()
        score_data = ScoringResult(
            strategic_score=5,
            narrative_score=6,
            authenticity_score=4,
            sensory_score=5,
            overall_score=5,
            strengths=["good detail"],
            weaknesses=["too generic"],
            improvement_suggestions=["add specifics"],
        )

        prompt = generator._improvement_prompt(
            original_prompt="Generate a forge",
            score=5.0,
            score_data=score_data,
        )

        assert "5.0/10" in prompt
        assert "good detail" in prompt
        assert "too generic" in prompt
        assert "add specifics" in prompt
        assert "Generate a forge" in prompt


class TestGenerationStats:
    """Tests for GenerationStats."""

    def test_record_generation(self):
        """Should track generation statistics."""
        from worldgen.generation.quality_loop import GenerationStats

        stats = GenerationStats()
        assert stats.total_generated == 0

        stats.record_generation(score=8.0, iterations=3, candidates=6)
        assert stats.total_generated == 1
        assert stats.total_iterations == 3
        assert stats.total_candidates == 6
        assert stats.avg_final_score == 8.0

        stats.record_generation(score=9.0, iterations=2, candidates=4)
        assert stats.total_generated == 2
        assert stats.total_iterations == 5
        assert stats.total_candidates == 10
        assert stats.avg_final_score == 8.5
