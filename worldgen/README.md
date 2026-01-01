# Worldgen - DeepSeek Hex Generation Pipeline

Generates hex map data for Arc Citadel using DeepSeek API.

## Setup

```bash
pip install -r requirements.txt
export DEEPSEEK_API_KEY="your-key-here"
```

## Quick Start

```bash
# Generate 100 valid seeds overnight
python batch_runner.py -n 100

# Generate 5 seeds for testing
python batch_runner.py -n 5 --delay 1.0
```

## Files

| File | Purpose |
|------|---------|
| `schemas.py` | Pydantic models for hex data |
| `hex_generator.py` | DeepSeek API integration |
| `validator.py` | Invariant checking |
| `batch_runner.py` | Overnight batch execution |
| `prompts/hex_prompt.txt` | Generation prompt template |

## Output

Valid seeds saved to `output/valid/`, invalid to `output/invalid/`.

Each seed contains:
- Multiple regions (2-4)
- 12-25 hexes per region
- Validation results

## Validation Rules

- Mountains: elevation >= 0.7
- Water: elevation <= 0.3
- Desert: moisture <= 0.2
- Swamp: moisture >= 0.7
- Max 3 resources per hex
- Unique coordinates per region
- Connected hex clusters (warning only)

## Customization

Edit `prompts/hex_prompt.txt` to change generation guidelines.
Edit `REGION_THEMES` in `batch_runner.py` to add region types.
