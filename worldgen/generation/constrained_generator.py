"""Constrained component generation.

Architecture principle: Emergence through simulation, not generation.
- Structural fields (terrain, elevation, etc.): Deterministic from constraints
- Creative fields (name, narrative, sensory): LLM-generated at high temp
- Validation: Structural = free/instant, Narrative = LLM-scored
"""

import json
import random
from dataclasses import dataclass
from typing import Optional

from worldgen import config
from worldgen.schemas import (
    Component,
    ComponentCategory,
    Species,
    Terrain,
    Resource,
    Feature,
    SpeciesFitness,
    Abundance,
)
from .constraints import (
    StructuralConstraints,
    get_constraints,
    validate_against_constraints,
)
from .llm_client import DeepSeekClient


@dataclass
class GenerationResult:
    """Result of constrained generation."""

    component: Optional[Component]
    narrative_score: float
    iterations: int
    rejected_count: int  # Structural validation failures (free)
    error: Optional[str] = None


CREATIVE_PROMPT_TEMPLATE = """Generate CREATIVE content for a {species} {category_name}.

{constraint_section}

YOU MUST USE THE EXACT STRUCTURAL VALUES ABOVE.
Your job is to generate ONLY the creative elements that make this location memorable.

Generate JSON with these fields:
{{
  "name_fragment": "<evocative 2-5 word name>",
  "narrative_hook": "<1-2 sentences that seed emergent gameplay - what makes entities INTERACT with this place?>",
  "sensory_details": ["<sound>", "<smell>", "<texture>", "<visual>"],
  "tags": ["<theme1>", "<theme2>", "<theme3>"],
  "resources": [
    {{"type": "<from allowed list>", "abundance": "<trace|scarce|low|medium|high|rich|legendary>"}},
    ...
  ],
  "features": [
    {{"type": "<from allowed list>", "details": "<specific detail that enables interaction>"}},
    ...
  ],
  "terrain": "{terrain}",
  "elevation": <number in range>,
  "moisture": <number in range>,
  "temperature": <number in range>,
  "species_fitness": {{"human": 0.0-1.0, "dwarf": 0.0-1.0, "elf": 0.0-1.0}}
}}

CRITICAL: The narrative_hook should suggest HOW entities will interact with this place.
Bad: "An ancient forge" (static description)
Good: "The anvil hums differently when lies are spoken nearby" (enables gameplay)

Species values for {species}:
- Dwarf: CRAFT-TRUTH, STONE-DEBT, OATH-CHAIN, ANCESTOR-WEIGHT
- Elf: PATTERN-BEAUTY, GROWTH-CYCLE, SPIRIT-HARMONY, TIME-PATIENCE
- Human: HONOR, AMBITION, LOYALTY, JUSTICE, CURIOSITY
"""

NARRATIVE_SCORING_PROMPT = """Rate this component's NARRATIVE QUALITY only (structure is pre-validated).

COMPONENT:
{component_json}

Rate 1-10 on:
1. INTERACTION POTENTIAL: Does the narrative_hook enable entity behavior? (not just describe)
2. SENSORY IMMERSION: Can you feel being there?
3. SPECIES AUTHENTICITY: Does it reflect {species} values, not generic fantasy?
4. EMERGENT SEEDS: Does it suggest stories that EMERGE from simulation?

A 7 is competent. A 9 requires genuine creativity that surprises you.

Respond with ONLY JSON:
{{
  "interaction_score": <1-10>,
  "sensory_score": <1-10>,
  "authenticity_score": <1-10>,
  "emergence_score": <1-10>,
  "overall_score": <1-10>,
  "strengths": ["...", "..."],
  "weaknesses": ["...", "..."],
  "improvement_suggestions": ["...", "..."]
}}"""


class ConstrainedGenerator:
    """Generate components with structural constraints and creative freedom.

    The key insight: LLMs are good at creativity, bad at consistency.
    So we constrain structure (deterministic) and unleash creativity (high temp).
    """

    def __init__(
        self,
        client: Optional[DeepSeekClient] = None,
        target_score: float = config.DEFAULT_TARGET_SCORE,
        max_iterations: int = config.MAX_QUALITY_ITERATIONS,
    ):
        self.client = client
        self.target_score = target_score
        self.max_iterations = max_iterations

    def _get_client(self) -> DeepSeekClient:
        if self.client is None:
            self.client = DeepSeekClient()
        return self.client

    def generate(
        self,
        category: ComponentCategory,
        index: int,
    ) -> GenerationResult:
        """Generate a component with constrained structure and creative narrative.

        Args:
            category: The component category to generate.
            index: Index for ID generation.

        Returns:
            GenerationResult with component (if successful) and metrics.
        """
        constraints = get_constraints(category)
        if constraints is None:
            return GenerationResult(
                component=None,
                narrative_score=0.0,
                iterations=0,
                rejected_count=0,
                error=f"No constraints defined for {category.value}",
            )

        client = self._get_client()
        best_component: Optional[Component] = None
        best_score = 0.0
        rejected_count = 0

        # Build the creative prompt with embedded constraints
        category_name = category.value.replace("_", " ")
        prompt = CREATIVE_PROMPT_TEMPLATE.format(
            species=constraints.species.value,
            category_name=category_name,
            constraint_section=constraints.to_prompt_section(),
            terrain=constraints.terrain.value,
        )

        for iteration in range(self.max_iterations):
            try:
                # Generate with high temperature for creativity
                raw = client.generate_json(
                    prompt=prompt,
                    temperature=config.CREATIVE_TEMPERATURE,
                    max_tokens=1500,
                )

                # FREE: Structural validation (deterministic, instant)
                errors = validate_against_constraints(raw, constraints)
                if errors:
                    rejected_count += 1
                    # Fix structural errors and retry
                    raw = self._fix_structure(raw, constraints)
                    errors = validate_against_constraints(raw, constraints)
                    if errors:
                        continue  # Still invalid, skip

                # Build Component from validated data
                component = self._build_component(raw, category, constraints, index)

                # PAID: Narrative scoring (LLM-based)
                score = self._score_narrative(component, constraints.species)

                if score > best_score:
                    best_score = score
                    best_component = component

                if score >= self.target_score:
                    return GenerationResult(
                        component=best_component,
                        narrative_score=best_score,
                        iterations=iteration + 1,
                        rejected_count=rejected_count,
                    )

                # Build improvement prompt for next iteration
                prompt = self._improvement_prompt(prompt, raw, score)

            except Exception as e:
                continue  # Log and retry

        return GenerationResult(
            component=best_component,
            narrative_score=best_score,
            iterations=self.max_iterations,
            rejected_count=rejected_count,
        )

    def _fix_structure(
        self, raw: dict, constraints: StructuralConstraints
    ) -> dict:
        """Fix structural fields to match constraints (deterministic)."""
        raw["terrain"] = constraints.terrain.value
        raw["elevation"] = max(
            constraints.elevation_min,
            min(constraints.elevation_max, raw.get("elevation", 0)),
        )
        raw["moisture"] = max(
            constraints.moisture_min,
            min(constraints.moisture_max, raw.get("moisture", 0.5)),
        )
        raw["temperature"] = max(
            constraints.temperature_min,
            min(constraints.temperature_max, raw.get("temperature", 15)),
        )
        return raw

    def _build_component(
        self,
        raw: dict,
        category: ComponentCategory,
        constraints: StructuralConstraints,
        index: int,
    ) -> Component:
        """Build a validated Component from raw generation data."""
        # Parse resources
        resources = []
        for r in raw.get("resources", [])[:3]:
            try:
                resources.append(Resource(
                    type=r["type"],
                    abundance=r.get("abundance", "medium"),
                ))
            except Exception:
                pass

        # Parse features
        features = []
        for f in raw.get("features", [])[:2]:
            try:
                features.append(Feature(
                    type=f["type"],
                    details=f.get("details"),
                    species_origin=constraints.species,
                ))
            except Exception:
                pass

        # Parse species fitness
        sf_raw = raw.get("species_fitness", {})
        species_fitness = SpeciesFitness(
            human=float(sf_raw.get("human", 0.3)),
            dwarf=float(sf_raw.get("dwarf", 0.3)),
            elf=float(sf_raw.get("elf", 0.3)),
        )

        return Component(
            id=f"comp_{category.value}_{index:05d}",
            category=category,
            tags=raw.get("tags", [])[:5],
            species=constraints.species,
            terrain=Terrain(raw["terrain"]),
            elevation=float(raw["elevation"]),
            moisture=float(raw["moisture"]),
            temperature=float(raw["temperature"]),
            resources=resources,
            features=features,
            species_fitness=species_fitness,
            name_fragment=raw.get("name_fragment", "Unnamed"),
            narrative_hook=raw.get("narrative_hook", ""),
            sensory_details=raw.get("sensory_details", [])[:4],
            quality_score=0.0,  # Will be set after scoring
        )

    def _score_narrative(self, component: Component, species: Species) -> float:
        """Score only the narrative quality (structure already validated)."""
        client = self._get_client()

        prompt = NARRATIVE_SCORING_PROMPT.format(
            component_json=json.dumps({
                "name_fragment": component.name_fragment,
                "narrative_hook": component.narrative_hook,
                "sensory_details": component.sensory_details,
                "tags": component.tags,
                "features": [{"type": f.type.value, "details": f.details} for f in component.features],
            }, indent=2),
            species=species.value,
        )

        try:
            result = client.generate_json(
                prompt=prompt,
                system_prompt="You are a harsh game design critic. Most things are 5-7. A 9 is genuinely surprising.",
                temperature=config.SCORING_TEMPERATURE,
                max_tokens=500,
            )
            return float(result.get("overall_score", 0))
        except Exception:
            return 5.0  # Default to mediocre if scoring fails

    def _improvement_prompt(self, original: str, previous: dict, score: float) -> str:
        """Build prompt for next iteration based on feedback."""
        return f"""Previous attempt scored {score}/10. The structure was correct but narrative needs work.

KEEP the good parts:
- name_fragment: "{previous.get('name_fragment', '')}"

IMPROVE:
- narrative_hook: Make it suggest HOW entities interact, not just describe
- sensory_details: More specific, less generic
- features: Add details that enable gameplay

{original}"""
