"""DeepSeek LLM client wrapper using OpenAI-compatible API."""

import json
from typing import Optional

from openai import OpenAI

from worldgen import config


class DeepSeekClient:
    """Wrapper for DeepSeek API via OpenAI-compatible client."""

    def __init__(
        self,
        api_key: Optional[str] = None,
        base_url: Optional[str] = None,
        model: Optional[str] = None,
    ):
        self.api_key = api_key or config.DEEPSEEK_API_KEY
        self.base_url = base_url or config.DEEPSEEK_BASE_URL
        self.model = model or config.DEEPSEEK_MODEL

        if not self.api_key:
            raise ValueError("DEEPSEEK_API_KEY not set")

        self.client = OpenAI(api_key=self.api_key, base_url=self.base_url)

    def generate(
        self,
        prompt: str,
        system_prompt: str = "You are a world generator for Arc Citadel. Output ONLY valid JSON, no markdown.",
        temperature: float = 0.9,
        max_tokens: int = 2000,
    ) -> str:
        """Generate content from prompt.

        Args:
            prompt: The user prompt to send to the model.
            system_prompt: System instructions for the model.
            temperature: Sampling temperature (0.0-2.0).
            max_tokens: Maximum tokens in the response.

        Returns:
            The generated content as a string.
        """
        response = self.client.chat.completions.create(
            model=self.model,
            messages=[
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": prompt},
            ],
            temperature=temperature,
            max_tokens=max_tokens,
        )
        return response.choices[0].message.content or ""

    def generate_json(
        self,
        prompt: str,
        system_prompt: str = "You are a world generator for Arc Citadel. Output ONLY valid JSON, no markdown.",
        temperature: float = 0.9,
        max_tokens: int = 2000,
    ) -> dict:
        """Generate and parse JSON response.

        Args:
            prompt: The user prompt to send to the model.
            system_prompt: System instructions for the model.
            temperature: Sampling temperature (0.0-2.0).
            max_tokens: Maximum tokens in the response.

        Returns:
            The parsed JSON as a dictionary.

        Raises:
            json.JSONDecodeError: If the response cannot be parsed as JSON.
        """
        content = self.generate(prompt, system_prompt, temperature, max_tokens)
        return self._clean_and_parse_json(content)

    def _clean_and_parse_json(self, content: str) -> dict:
        """Clean up common JSON issues from LLM output.

        Args:
            content: Raw string content from the LLM.

        Returns:
            Parsed JSON as a dictionary.

        Raises:
            json.JSONDecodeError: If the content cannot be parsed as JSON.
        """
        content = content.strip()

        # Remove markdown code blocks
        if content.startswith("```"):
            lines = content.split("\n")
            # Remove first line (```json or ```)
            lines = lines[1:]
            # Remove last line if it's closing ```
            if lines and lines[-1].strip() == "```":
                lines = lines[:-1]
            content = "\n".join(lines)

        # Handle case where json is on the same line as ```
        if content.startswith("json"):
            content = content[4:]

        content = content.strip()
        return json.loads(content)
