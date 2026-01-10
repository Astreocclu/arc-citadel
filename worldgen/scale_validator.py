"""100m scale validation for hex descriptions."""

import json
import os
from typing import Optional

import httpx

from schemas import TaggedHex, ScaleValidation


SCALE_VALIDATION_PROMPT = """Analyze this hex description for 100-meter scale consistency.

SCALE RULE: Each hex represents approximately 100m × 100m (about 1 hectare).
- APPROPRIATE (100m scale): A grove of trees, a hilltop with ruins, a market square, a mine entrance area, a small lake
- TOO LARGE (region scale): "vast forest", "mountain range", "sprawling city", anything described as "miles" or "leagues"
- TOO SMALL (room scale): "a closet", "a single room", "a small chamber", anything that fits in a building

REFERENCE: A party of adventurers should be able to thoroughly explore this hex in 10-15 minutes.

HEX DATA:
Name: {name}
Description: {description}
Tags: {tags}

ANALYSIS:
1. Can all described features realistically fit in 100m × 100m?
2. Would exploring this area take roughly 10-15 minutes?
3. Are any features described that are too large OR too small?

OUTPUT JSON:
{{
  "score": <0-10 integer>,
  "feedback": "<brief explanation of score>"
}}

Score guide:
- 9-10: Perfect 100m scale fit
- 7-8: Acceptable, minor scale issues
- 4-6: Questionable scale, needs revision
- 1-3: Wrong scale entirely
"""


class ScaleValidator:
    """Validates hex descriptions for 100m scale consistency."""

    def __init__(
        self,
        threshold: float = 7.0,
        api_key: Optional[str] = None,
        model: str = "deepseek-chat",
    ):
        self.threshold = threshold
        self.api_key = api_key or os.environ.get("DEEPSEEK_API_KEY")
        self.model = model
        self.client = httpx.Client(timeout=60.0)

    def validate(self, hex: TaggedHex) -> ScaleValidation:
        """Validate a hex's description for 100m scale.

        Returns:
            ScaleValidation with score, passes flag, and feedback
        """
        prompt = self._build_prompt(hex)
        response = self._call_llm(prompt)

        score = float(response.get("score", 0))
        feedback = response.get("feedback", "")

        return ScaleValidation(
            hex_index=0,  # Will be set by caller
            score=score,
            passes=score >= self.threshold,
            feedback=feedback,
        )

    def validate_batch(self, hexes: list[TaggedHex]) -> list[ScaleValidation]:
        """Validate multiple hexes."""
        results = []
        for idx, hex in enumerate(hexes):
            result = self.validate(hex)
            result.hex_index = idx
            results.append(result)
        return results

    def _build_prompt(self, hex: TaggedHex) -> str:
        """Build validation prompt for a hex."""
        return SCALE_VALIDATION_PROMPT.format(
            name=hex.name,
            description=hex.description,
            tags=", ".join(hex.tags),
        )

    def _call_llm(self, prompt: str) -> dict:
        """Call LLM API for scale validation."""
        if not self.api_key:
            raise ValueError("DEEPSEEK_API_KEY not set")

        response = self.client.post(
            "https://api.deepseek.com/chat/completions",
            headers={
                "Authorization": f"Bearer {self.api_key}",
                "Content-Type": "application/json",
            },
            json={
                "model": self.model,
                "messages": [
                    {"role": "system", "content": "You are a scale validation assistant. Output JSON only."},
                    {"role": "user", "content": prompt},
                ],
                "temperature": 0.3,  # Low temp for consistent scoring
                "max_tokens": 256,
                "response_format": {"type": "json_object"},
            },
        )
        response.raise_for_status()
        content = response.json()["choices"][0]["message"]["content"]
        return json.loads(content)

    def close(self):
        self.client.close()

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
