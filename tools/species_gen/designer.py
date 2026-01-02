"""Species designer - uses LLM to architect compelling species gameplay."""

import json
import sys
from pathlib import Path

# Add project root to path for imports
project_root = Path(__file__).parent.parent.parent
sys.path.insert(0, str(project_root))

import toml
from worldgen.generation.llm_client import DeepSeekClient


# Available actions in the game (from src/actions/catalog.rs)
AVAILABLE_ACTIONS = [
    "MoveTo", "Follow", "Flee", "Rest", "Eat", "SeekSafety",
    "Build", "Craft", "Gather", "Repair",
    "TalkTo", "Help", "Trade",
    "Attack", "Defend", "Charge", "HoldPosition",
    "IdleWander", "IdleObserve"
]

# Available polity types (from src/aggregate/polity.rs)
AVAILABLE_POLITY_TYPES = [
    "Kingdom", "Tribe", "CityState",  # Human
    "Clan", "Hold",                    # Dwarf
    "Grove", "Court",                  # Elf
    "Warband", "Horde"                 # Orc/generic
]

# Terrain types
TERRAIN_TYPES = ["mountain", "hills", "forest", "plains", "marsh", "coast", "desert", "river"]

DESIGNER_SYSTEM_PROMPT = """You are a game designer creating species for Arc Citadel, an emergent behavior simulation.

CRITICAL DESIGN PRINCIPLES:

1. VALUES DRIVE BEHAVIOR
   - Values are internal desires/drives (0.0-1.0), NOT stats or capabilities
   - Good values: greed, hunger, rage, loyalty, cowardice, cunning, territoriality
   - Bad values: regeneration (that's a stat), intelligence (too vague), power (not actionable)
   - Each value should connect to at least one action_rule or idle_behavior

2. ACTION RULES CREATE EMERGENT GAMEPLAY
   - When value > threshold, entity takes action
   - Use HIGH thresholds (0.7-0.9) for dramatic actions, LOWER (0.4-0.6) for common behaviors
   - Available actions: MoveTo, Follow, Flee, Rest, Eat, SeekSafety, Build, Craft, Gather, Repair, TalkTo, Help, Trade, Attack, Defend, Charge, HoldPosition, IdleWander, IdleObserve

3. TERRAIN CREATES CONFLICT
   - Species should strongly prefer 2-3 terrain types (0.7-0.9)
   - Have 2-3 terrains they dislike (0.1-0.3)
   - Overlapping preferences with other species = territorial conflict

4. POLITY STATE TRACKS FACTION-LEVEL DRAMA
   - Use meaningful state: grudge_list, raid_targets, hoard_value, war_exhaustion
   - NOT memes (bridge_count) or capabilities (regeneration_level)

5. ANTAGONIST DESIGN
   - What makes them THREATENING? (aggression triggers, territorial expansion)
   - What makes them INTERESTING? (internal conflicts, value tensions)
   - What creates STORIES? (grudges, alliances, escalation)

OUTPUT FORMAT: Valid TOML only, no markdown, no explanation."""

DESIGNER_PROMPT_TEMPLATE = """Design a species for Arc Citadel:

CONCEPT: {concept}

ROLE: {role}

Create a complete species TOML with:

1. [metadata] - name (PascalCase), module_name (snake_case)

2. [entity_values] - 4-6 values that drive behavior
   Format: value_name = {{ type = "f32", default = X.X, description = "..." }}

3. [polity_state] - 2-4 faction-level state fields
   Format: field_name = {{ type = "TYPE", default = "DEFAULT_EXPR" }}
   Types: f32, u32, Vec<u32>, Vec<String>

4. [terrain_fitness] - all 8 terrains with fitness 0.1-0.9
   Terrains: mountain, hills, forest, plains, marsh, coast, desert, river

5. [growth] rate = 1.01-1.10 (1.02 = slow, 1.05 = moderate, 1.08 = fast)

6. [expansion] threshold = 0.2-0.5 (lower = more aggressive expansion)

7. [naming] prefixes and suffixes (10 each, thematic)

8. [polity_types] small, medium, large (must use: {polity_types})

9. [[action_rules]] - 2-4 rules linking values to actions
   Required fields: trigger_value, threshold, action, priority (High/Normal/Low), requires_target (bool), description
   Available actions: {actions}

10. [[idle_behaviors]] - 2-3 idle behaviors
    Required fields: value, threshold, action, requires_target, description

DESIGN GOALS FOR THIS SPECIES:
- Primary threat/appeal: {threat}
- Key behavioral tension: {tension}
- Relationship to player: {relationship}

Output ONLY the TOML, starting with [metadata]."""


class SpeciesDesigner:
    """Uses LLM to design compelling species gameplay."""

    def __init__(self):
        self.client = DeepSeekClient()

    def design(
        self,
        concept: str,
        role: str = "antagonist",
        threat: str = "territorial aggression",
        tension: str = "individual vs group needs",
        relationship: str = "enemy that can occasionally be negotiated with"
    ) -> str:
        """Design a species from a concept description.

        Args:
            concept: Natural language description (e.g., "sneaky goblin raiders")
            role: "antagonist", "neutral", "ally"
            threat: What makes them dangerous/interesting
            tension: Internal conflict that creates emergent behavior
            relationship: How player typically interacts with them

        Returns:
            TOML string ready to save to file
        """
        prompt = DESIGNER_PROMPT_TEMPLATE.format(
            concept=concept,
            role=role,
            threat=threat,
            tension=tension,
            relationship=relationship,
            actions=", ".join(AVAILABLE_ACTIONS),
            polity_types=", ".join(AVAILABLE_POLITY_TYPES)
        )

        response = self.client.generate(
            prompt=prompt,
            system_prompt=DESIGNER_SYSTEM_PROMPT,
            temperature=0.8,
            max_tokens=2000
        )

        # Clean up response
        content = response.strip()
        if content.startswith("```"):
            lines = content.split("\n")
            lines = lines[1:]
            if lines and lines[-1].strip() == "```":
                lines = lines[:-1]
            content = "\n".join(lines)
        if content.startswith("toml"):
            content = content[4:].strip()

        return content

    def design_and_validate(
        self,
        concept: str,
        **kwargs
    ) -> tuple[str, list[str]]:
        """Design and validate a species.

        Returns:
            (toml_content, list_of_validation_errors)
        """
        content = self.design(concept, **kwargs)
        errors = self.validate(content)
        return content, errors

    def validate(self, toml_content: str) -> list[str]:
        """Validate generated TOML against schema requirements."""
        errors = []

        try:
            spec = toml.loads(toml_content)
        except toml.TomlDecodeError as e:
            return [f"Invalid TOML: {e}"]

        # Check required sections
        required = ["metadata", "entity_values", "polity_state", "terrain_fitness",
                   "growth", "expansion", "naming", "polity_types"]
        for section in required:
            if section not in spec:
                errors.append(f"Missing required section: [{section}]")

        # Validate metadata
        if "metadata" in spec:
            if "name" not in spec["metadata"]:
                errors.append("metadata.name is required")
            if "module_name" not in spec["metadata"]:
                errors.append("metadata.module_name is required")

        # Validate terrain_fitness has all terrains
        if "terrain_fitness" in spec:
            for terrain in TERRAIN_TYPES:
                if terrain not in spec["terrain_fitness"]:
                    errors.append(f"Missing terrain fitness: {terrain}")

        # Validate action_rules use valid actions
        for rule in spec.get("action_rules", []):
            if rule.get("action") not in AVAILABLE_ACTIONS:
                errors.append(f"Invalid action in rule: {rule.get('action')}")

        # Validate idle_behaviors use valid actions
        for behavior in spec.get("idle_behaviors", []):
            if behavior.get("action") not in AVAILABLE_ACTIONS:
                errors.append(f"Invalid action in idle behavior: {behavior.get('action')}")

        # Validate polity_types use valid types
        if "polity_types" in spec:
            for size, ptype in spec["polity_types"].items():
                if ptype not in AVAILABLE_POLITY_TYPES:
                    errors.append(f"Invalid polity type: {ptype}")

        return errors


def main():
    """Interactive species designer."""
    import argparse

    parser = argparse.ArgumentParser(description="Design a species using LLM")
    parser.add_argument("concept", help="Species concept (e.g., 'sneaky goblin raiders')")
    parser.add_argument("--role", default="antagonist", choices=["antagonist", "neutral", "ally"])
    parser.add_argument("--threat", default="territorial aggression", help="What makes them dangerous")
    parser.add_argument("--tension", default="individual vs group needs", help="Internal conflict")
    parser.add_argument("--output", "-o", help="Output file path")
    parser.add_argument("--validate-only", action="store_true", help="Only validate, don't save")

    args = parser.parse_args()

    designer = SpeciesDesigner()

    print(f"Designing species: {args.concept}")
    print(f"Role: {args.role}, Threat: {args.threat}")
    print("-" * 50)

    content, errors = designer.design_and_validate(
        args.concept,
        role=args.role,
        threat=args.threat,
        tension=args.tension
    )

    if errors:
        print("VALIDATION ERRORS:")
        for err in errors:
            print(f"  - {err}")
        print()

    print("GENERATED TOML:")
    print(content)

    if args.output and not errors:
        Path(args.output).write_text(content)
        print(f"\nSaved to: {args.output}")
    elif args.output and errors:
        print(f"\nNot saving due to validation errors")


if __name__ == "__main__":
    main()
