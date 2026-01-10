"""LLM-based anchor selection for connectors."""

import json
import os
from typing import Optional

import httpx

from worldgen.schemas.connector import ConnectorCollection
from worldgen.schemas.minor import MinorAnchor, MinorCategory


ANCHOR_SELECTION_PROMPT = """You are placing points of interest along a fantasy travel route.

CONNECTOR CONTEXT:
- Type: {connector_type}
- Length: {length} hexes (each hex is 100m, so {length_km:.1f}km total)
- Terrain: {terrain_summary}
- Available slots: {slot_count} positions

SLOT DETAILS:
{slot_details}

ANCHOR TYPES AVAILABLE:
- Rest: inn, tavern, waystation, camp, caravanserai
- Crossings: bridge_wood, bridge_stone, ford_improved, ferry, tunnel
- Sacred: shrine, temple_small, standing_stone, sacred_spring
- Military: watchtower, toll_gate, border_post, signal_tower
- Economic: market_small, mill, mine_entrance, lumber_camp
- Mysterious: hermit_hut, witch_cottage, abandoned_camp, unmarked_graves

RULES:
1. Select 0 to {max_anchors} anchors total
2. Match anchor type to slot context (crossings need bridges/fords)
3. Trade routes need rest stops; military roads need watchtowers
4. Space anchors reasonably (not every slot needs an anchor)
5. Add narrative variety - not all inns, mix it up

OUTPUT JSON:
{{
  "anchors": [
    {{
      "slot_id": "<slot_id from above>",
      "category": "<MinorCategory value>",
      "name_fragment": "<evocative 2-4 word name>",
      "narrative_hook": "<1 sentence hook for this location>"
    }}
  ]
}}
"""


class AnchorPlacer:
    """LLM-based selection of minor anchors for connectors."""

    def __init__(
        self,
        api_key: Optional[str] = None,
        model: str = "deepseek-chat",
        max_anchors_per_connector: int = 5,
    ):
        self.api_key = api_key or os.environ.get("DEEPSEEK_API_KEY")
        self.model = model
        self.max_anchors = max_anchors_per_connector
        self.client = httpx.Client(timeout=60.0)

    def place_anchors(
        self, connector: ConnectorCollection
    ) -> list[MinorAnchor]:
        """Select and generate anchors for a connector via LLM.

        Args:
            connector: ConnectorCollection with slots defined

        Returns:
            List of MinorAnchor objects to place
        """
        if not connector.minor_slots:
            return []

        # Build context for LLM
        context = self._build_context(connector)
        prompt = ANCHOR_SELECTION_PROMPT.format(**context)

        # Call LLM
        try:
            response = self._call_llm(prompt)
            anchors = self._parse_response(response, connector)
            return anchors
        except Exception as e:
            print(f"LLM anchor selection failed: {e}")
            return self._fallback_selection(connector)

    def _build_context(self, connector: ConnectorCollection) -> dict:
        """Build prompt context from connector."""
        # Summarize terrain
        terrain_counts: dict[str, int] = {}
        for chex in connector.hexes:
            t = chex.terrain.value
            terrain_counts[t] = terrain_counts.get(t, 0) + 1
        terrain_summary = ", ".join(f"{t}: {c}" for t, c in terrain_counts.items())

        # Format slot details
        slot_details = []
        for slot in connector.minor_slots:
            slot_details.append(
                f"- {slot.slot_id}: index {slot.hex_index}, "
                f"context: {slot.narrative_context}, "
                f"compatible: {slot.compatible_categories}"
            )

        return {
            "connector_type": connector.type.value,
            "length": len(connector.hexes),
            "length_km": len(connector.hexes) * 0.1,
            "terrain_summary": terrain_summary,
            "slot_count": len(connector.minor_slots),
            "slot_details": "\n".join(slot_details),
            "max_anchors": min(self.max_anchors, len(connector.minor_slots)),
        }

    def _call_llm(self, prompt: str) -> dict:
        """Call DeepSeek API."""
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
                    {"role": "system", "content": "You are a fantasy worldbuilder. Output valid JSON only."},
                    {"role": "user", "content": prompt},
                ],
                "temperature": 0.9,
                "max_tokens": 1024,
                "response_format": {"type": "json_object"},
            },
        )
        response.raise_for_status()
        content = response.json()["choices"][0]["message"]["content"]
        return json.loads(content)

    def _parse_response(
        self, response: dict, connector: ConnectorCollection
    ) -> list[MinorAnchor]:
        """Parse LLM response into MinorAnchor objects."""
        anchors = []
        slot_map = {slot.slot_id: slot for slot in connector.minor_slots}

        for item in response.get("anchors", []):
            slot_id = item.get("slot_id")
            if slot_id not in slot_map:
                continue

            slot = slot_map[slot_id]

            try:
                category = MinorCategory(item["category"])
            except ValueError:
                continue

            # Validate category is compatible with slot
            if slot.compatible_categories and category.value not in slot.compatible_categories:
                continue

            anchor = MinorAnchor(
                id=f"anchor_{connector.id}_{slot_id}",
                category=category,
                slot_contexts=[slot.narrative_context],
                compatible_terrain=[connector.hexes[slot.hex_index].terrain],
                name_fragment=item.get("name_fragment", f"The {category.value}"),
                narrative_hook=item.get("narrative_hook", "A place of interest along the route."),
            )

            # Set service flags based on category
            if category in (MinorCategory.INN, MinorCategory.TAVERN, MinorCategory.WAYSTATION):
                anchor.provides_rest = True
            if category in (MinorCategory.MARKET_SMALL, MinorCategory.CARAVANSERAI):
                anchor.provides_trade = True
            if category in (MinorCategory.SHRINE, MinorCategory.HERMIT_HUT):
                anchor.provides_information = True
            if category in (MinorCategory.TOLL_GATE, MinorCategory.BORDER_POST):
                anchor.blocks_passage = True

            anchors.append(anchor)

        return anchors

    def _fallback_selection(self, connector: ConnectorCollection) -> list[MinorAnchor]:
        """Rule-based fallback when LLM fails."""
        anchors = []

        # Place anchors at required slots
        for slot in connector.minor_slots:
            if not slot.required:
                continue

            # Pick first compatible category
            if slot.compatible_categories:
                try:
                    category = MinorCategory(slot.compatible_categories[0])
                except ValueError:
                    continue

                anchor = MinorAnchor(
                    id=f"anchor_{connector.id}_{slot.slot_id}",
                    category=category,
                    slot_contexts=[slot.narrative_context],
                    compatible_terrain=[connector.hexes[slot.hex_index].terrain],
                    name_fragment=f"The {category.value.replace('_', ' ').title()}",
                    narrative_hook=slot.narrative_context,
                )
                anchors.append(anchor)

        return anchors

    def close(self):
        self.client.close()

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
