"""Configuration for worldgen pipeline."""

import os
from pathlib import Path

# Paths
WORLDGEN_ROOT = Path(__file__).parent
OUTPUT_DIR = WORLDGEN_ROOT / "output"
LIBRARIES_DIR = OUTPUT_DIR / "libraries"
SEEDS_DIR = OUTPUT_DIR / "seeds"
WORLDS_DIR = OUTPUT_DIR / "worlds"
TEMPLATES_DIR = WORLDGEN_ROOT / "templates"
PROMPTS_DIR = WORLDGEN_ROOT / "generation" / "prompts"

# Database
DATABASE_PATH = LIBRARIES_DIR / "assets.db"

# DeepSeek API
DEEPSEEK_API_KEY = os.environ.get("DEEPSEEK_API_KEY", "")
DEEPSEEK_BASE_URL = "https://api.deepseek.com"
DEEPSEEK_MODEL = "deepseek-chat"

# Generation
DEFAULT_TARGET_SCORE = 9.0
MAX_QUALITY_ITERATIONS = 10
CANDIDATES_PER_ROUND = 3

# MVP: Reduced counts for validation
MVP_COMPONENTS_PER_CATEGORY = 100
