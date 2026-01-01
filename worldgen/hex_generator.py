"""DeepSeek integration for hex generation."""

import json
import os
import time
from pathlib import Path
from typing import Optional

import httpx
from pydantic import ValidationError

from schemas import HexRegion, GenerationSeed


DEEPSEEK_API_URL = "https://api.deepseek.com/chat/completions"
DEFAULT_MODEL = "deepseek-chat"
MAX_RETRIES = 3
RETRY_DELAY = 5.0


class HexGenerator:
    """Generates hex regions using DeepSeek API."""

    def __init__(
        self,
        api_key: Optional[str] = None,
        model: str = DEFAULT_MODEL,
        timeout: float = 120.0,
    ):
        self.api_key = api_key or os.environ.get("DEEPSEEK_API_KEY")
        if not self.api_key:
            raise ValueError("DEEPSEEK_API_KEY not set")
        self.model = model
        self.timeout = timeout
        self.client = httpx.Client(timeout=timeout)

    def load_prompt(self, prompt_path: Path, **kwargs) -> str:
        """Load and format a prompt template."""
        template = prompt_path.read_text()
        return template.format(**kwargs)

    def _call_deepseek(self, prompt: str, system_prompt: str = "") -> str:
        """Make API call to DeepSeek with retries."""
        messages = []
        if system_prompt:
            messages.append({"role": "system", "content": system_prompt})
        messages.append({"role": "user", "content": prompt})

        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Content-Type": "application/json",
        }

        payload = {
            "model": self.model,
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": 8192,
            "response_format": {"type": "json_object"},
        }

        last_error = None
        for attempt in range(MAX_RETRIES):
            try:
                response = self.client.post(
                    DEEPSEEK_API_URL,
                    headers=headers,
                    json=payload,
                )
                response.raise_for_status()
                data = response.json()
                return data["choices"][0]["message"]["content"]
            except httpx.HTTPStatusError as e:
                last_error = e
                if e.response.status_code == 429:
                    wait_time = RETRY_DELAY * (2 ** attempt)
                    print(f"Rate limited, waiting {wait_time}s...")
                    time.sleep(wait_time)
                elif e.response.status_code >= 500:
                    wait_time = RETRY_DELAY * (attempt + 1)
                    print(f"Server error {e.response.status_code}, waiting {wait_time}s...")
                    time.sleep(wait_time)
                else:
                    raise
            except httpx.TimeoutException as e:
                last_error = e
                wait_time = RETRY_DELAY * (attempt + 1)
                print(f"Timeout, waiting {wait_time}s...")
                time.sleep(wait_time)

        raise RuntimeError(f"Failed after {MAX_RETRIES} attempts: {last_error}")

    def generate_region(
        self,
        prompt_path: Path,
        region_name: str,
        region_theme: str,
        hex_count: int = 19,
        **extra_context,
    ) -> Optional[HexRegion]:
        """Generate a single hex region."""
        system_prompt = """You are a world-building AI that generates hex map data.
Output valid JSON matching the schema exactly. Be creative with descriptions but strict with data types."""

        prompt = self.load_prompt(
            prompt_path,
            region_name=region_name,
            region_theme=region_theme,
            hex_count=hex_count,
            **extra_context,
        )

        try:
            raw_response = self._call_deepseek(prompt, system_prompt)
            data = json.loads(raw_response)
            region = HexRegion.model_validate(data)
            return region
        except json.JSONDecodeError as e:
            print(f"JSON parse error: {e}")
            return None
        except ValidationError as e:
            print(f"Validation error: {e}")
            return None

    def generate_seed(
        self,
        prompt_path: Path,
        seed_id: str,
        regions_config: list[dict],
    ) -> Optional[GenerationSeed]:
        """Generate a complete seed with multiple regions."""
        regions = []
        for config in regions_config:
            region = self.generate_region(prompt_path, **config)
            if region:
                regions.append(region)
            else:
                print(f"Failed to generate region: {config.get('region_name', 'unknown')}")

        if not regions:
            return None

        prompt_text = prompt_path.read_text()
        return GenerationSeed(
            seed_id=seed_id,
            regions=regions,
            generation_prompt=prompt_text[:500],
        )

    def close(self):
        """Close the HTTP client."""
        self.client.close()

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
