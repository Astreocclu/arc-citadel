"""Connected hex cluster generation."""

import json
import os
import random
from collections import deque
from typing import Optional

import httpx

from schemas import TaggedHex, HexCluster, EdgeType, FoundingContext
from hex_coords import get_neighbor, get_opposite_edge, coords_to_key, HEX_NEIGHBOR_OFFSETS
from adjacency import AdjacencyValidator


HEX_GENERATION_PROMPT = """Generate a fantasy location that fits within a 100-meter hex.

CRITICAL SCALE CONTEXT:
- This location fills a 100-meter hex (100m × 100m, about 1 hectare)
- A party can thoroughly explore this area in 10-15 minutes
- Examples of 100m scale: a forest grove, a hilltop, a mine entrance area, a market square
- NOT 100m scale: a single room (too small), a vast forest (too large)

LOCATION COORDINATES: ({q}, {r})

ADJACENT HEXES (must be compatible):
{adjacency_context}
{founding_context}
TAG OPTIONS:
- TERRAIN (pick 1): underground, surface, underwater, aerial
- CULTURE (0-2): dwarf, elf, human, wild, ancient, corrupted
- FUNCTION (0-2): residential, military, sacred, industrial, commercial, passage
- ELEVATION (pick 1): deep, shallow_under, surface, elevated, peak

TERRAIN PREFERENCE: Default to surface terrain unless adjacent hexes require underground/underwater/aerial transition.

EDGE TYPES (exactly 6, clockwise from East):
- tunnel: underground passage
- road: surface road/path
- water: water connection
- wilderness: open terrain
- entrance: transition point
- blocked: impassable

{constraint_hints}

NAMING RULES:
- Use SPECIFIC names referencing a person, event, or unique feature
- Pattern: [Person]'s [Place/Event] or The [Adjective] [Landmark]
- A local would give directions using this name
- Draw character names from diverse backgrounds and cultures

OUTPUT JSON:
{{
  "q": {q},
  "r": {r},
  "name": "<specific, memorable 2-4 word name>",
  "description": "<2-3 sentences describing what exists in this 100m space>",
  "tags": ["<terrain>", "<culture>", "<function>", "<elevation>"],
  "edge_types": ["<edge0>", "<edge1>", "<edge2>", "<edge3>", "<edge4>", "<edge5>"]
}}
"""


class ClusterGenerator:
    """Generates connected clusters of tagged hexes."""

    def __init__(
        self,
        api_key: Optional[str] = None,
        model: str = "deepseek-chat",
        tags_path: str = "generation/hex_tags.toml",
    ):
        self.api_key = api_key or os.environ.get("DEEPSEEK_API_KEY")
        self.model = model
        self.client = httpx.Client(timeout=120.0)
        self.validator = AdjacencyValidator(tags_path)
        # Founding context state (set during generate())
        self._current_founding_context: Optional[FoundingContext] = None
        self._current_founding_cluster_id: Optional[int] = None

    def generate(
        self,
        size: int = 20,
        seed_tags: Optional[list[str]] = None,
        founding_context: Optional[FoundingContext] = None,
        founding_cluster_id: Optional[int] = None,
    ) -> HexCluster:
        """Generate a connected cluster of hexes.

        Args:
            size: Number of hexes to generate (default 20)
            seed_tags: Optional tags for the seed hex
            founding_context: Optional founding conditions for settlement generation
            founding_cluster_id: Optional ID to link hexes sharing founding conditions

        Returns:
            HexCluster with connected hexes
        """
        # Store founding context for use during generation
        self._current_founding_context = founding_context
        self._current_founding_cluster_id = founding_cluster_id

        # Initialize with seed hex at origin
        seed_hex = self._generate_seed_hex(seed_tags)
        hexes: dict[str, TaggedHex] = {coords_to_key(0, 0): seed_hex}

        # BFS expansion frontier
        frontier: deque[tuple[int, int]] = deque([(0, 0)])

        while len(hexes) < size and frontier:
            q, r = frontier.popleft()
            current_hex = hexes[coords_to_key(q, r)]

            # Find available expansion edges
            available_edges = []
            for edge in range(6):
                nq, nr = get_neighbor(q, r, edge)
                if coords_to_key(nq, nr) not in hexes:
                    # Check if this edge allows expansion
                    if current_hex.edge_types[edge] != EdgeType.BLOCKED:
                        available_edges.append((edge, nq, nr))

            if not available_edges:
                continue

            # Add 1-2 new hexes from this position
            num_to_add = min(random.randint(1, 2), size - len(hexes), len(available_edges))
            edges_to_expand = random.sample(available_edges, num_to_add)

            for edge, nq, nr in edges_to_expand:
                if len(hexes) >= size:
                    break

                # Build context for new hex
                context = self._build_adjacency_context(hexes, nq, nr)
                diversity_hints = self._build_diversity_hints(hexes)

                # Generate hex with founding-aware selection
                try:
                    best_hex = None
                    best_score = -1.0

                    # Generate 2 candidates (if founding context exists) and pick best
                    num_candidates = 2 if self._current_founding_context else 1

                    for _ in range(num_candidates):
                        candidate = self._generate_single_hex(nq, nr, context, diversity_hints=diversity_hints)

                        # Validate adjacency
                        adj_validation = self.validator.validate_adjacency(
                            current_hex, candidate, edge
                        )

                        # Validate founding constraints
                        founding_validation = self.validator.validate_founding_constraints(
                            candidate, self._current_founding_context
                        )

                        all_errors = adj_validation.errors + founding_validation.errors

                        if not all_errors:
                            # Valid candidate - calculate score
                            score = self._calculate_founding_score(candidate)
                            if score > best_score:
                                best_score = score
                                best_hex = candidate

                    if best_hex is not None:
                        hexes[coords_to_key(nq, nr)] = best_hex
                        frontier.append((nq, nr))
                    else:
                        # All candidates failed validation - retry with strict hints
                        print(f"Validation failed at ({nq},{nr}), retrying with hints")
                        new_hex = self._generate_single_hex(
                            nq, nr, context,
                            constraint_hints=f"MUST FIX: avoid tags {self._current_founding_context.bias_against if self._current_founding_context else []}"
                        )

                        # Check founding again
                        founding_v2 = self.validator.validate_founding_constraints(
                            new_hex, self._current_founding_context
                        )

                        # If still fails, apply tag injection
                        if founding_v2.errors:
                            new_hex = self._apply_tag_injection(new_hex)

                        hexes[coords_to_key(nq, nr)] = new_hex
                        frontier.append((nq, nr))

                except Exception as e:
                    print(f"Generation failed at ({nq},{nr}): {e}")

        # Build adjacency list
        adjacencies = self._compute_adjacencies(hexes)

        return HexCluster(
            hexes=list(hexes.values()),
            adjacencies=adjacencies,
        )

    def _generate_seed_hex(self, tags: Optional[list[str]] = None) -> TaggedHex:
        """Generate the starting hex at origin."""
        default_tags = tags or ["surface", "wild", "passage", "surface"]

        return TaggedHex(
            q=0,
            r=0,
            name="Origin Point",
            description="A natural crossroads where several paths converge. Worn stones mark the intersection.",
            tags=default_tags,
            edge_types=[EdgeType.WILDERNESS] * 6,
            founding_context=self._current_founding_context,
            founding_cluster_id=self._current_founding_cluster_id,
        )

    def _build_adjacency_context(
        self,
        existing_hexes: dict[str, TaggedHex],
        new_q: int,
        new_r: int,
    ) -> dict:
        """Build context about adjacent hexes for LLM."""
        neighbors = []

        for edge in range(6):
            # Check if there's a hex in the opposite direction
            nq, nr = get_neighbor(new_q, new_r, edge)
            key = coords_to_key(nq, nr)

            if key in existing_hexes:
                neighbor = existing_hexes[key]
                opposite = get_opposite_edge(edge)
                neighbors.append({
                    "direction": edge,
                    "direction_name": ["E", "NE", "NW", "W", "SW", "SE"][edge],
                    "tags": neighbor.tags,
                    "their_edge_type": neighbor.edge_types[opposite].value,
                    "hint": f"Edge {edge} must match their edge {opposite} ({neighbor.edge_types[opposite].value})",
                })

        return {"neighbors": neighbors}

    def _build_founding_context_str(self) -> str:
        """Build founding context string for LLM prompt."""
        ctx = self._current_founding_context
        if ctx is None:
            return ""

        lines = ["\nSETTLEMENT FOUNDING CONTEXT:"]
        lines.append(f"- Founded in: {ctx.season}")
        if ctx.astronomical_event:
            lines.append(f"- Celestial event: {ctx.astronomical_event}")
        if ctx.flavor:
            lines.append(f"- Character: {ctx.flavor}")
        if ctx.bias_tags:
            lines.append(f"- PREFER these features: {', '.join(ctx.bias_tags)}")
        if ctx.bias_against:
            lines.append(f"- AVOID these features: {', '.join(ctx.bias_against)}")
        if ctx.siege_mentality:
            lines.append("- Settlement has siege mentality: emphasize defensive, enclosed structures")
        if ctx.martial_culture > 0.2:
            lines.append("- Martial culture: include military/training features")

        lines.append("")
        return "\n".join(lines)

    def _calculate_founding_score(self, hex: TaggedHex) -> float:
        """Calculate how well a hex matches founding conditions.

        Returns a score from 0.0 to 1.0:
        - 1.0 = perfect match (has preferred tags, no forbidden tags)
        - 0.0 = terrible match (has forbidden tags, no preferred tags)

        This is used for soft ranking during generation, not hard rejection.
        """
        ctx = self._current_founding_context
        if ctx is None:
            return 1.0  # No context = all hexes equally valid

        tag_set = set(hex.tags)
        score = 0.5  # Base score

        # Penalty for forbidden tags (-0.3 each, max -0.5)
        forbidden = set(ctx.bias_against)
        violations = tag_set & forbidden
        score -= min(len(violations) * 0.3, 0.5)

        # Bonus for preferred tags (+0.2 each, max +0.5)
        preferred = set(ctx.bias_tags)
        matches = tag_set & preferred
        score += min(len(matches) * 0.2, 0.5)

        # Bonus for siege_mentality + military tag
        if ctx.siege_mentality and "military" in tag_set:
            score += 0.1

        # Bonus for martial_culture + military/industrial
        if ctx.martial_culture > 0.2 and ("military" in tag_set or "industrial" in tag_set):
            score += 0.1

        return max(0.0, min(1.0, score))

    def _apply_tag_injection(self, hex: TaggedHex) -> TaggedHex:
        """Apply tag injection fallback to enforce founding constraints.

        1. Remove any tags from bias_against
        2. Add first bias_tag if no preferred tags present

        Returns a new TaggedHex with corrected tags.
        """
        ctx = self._current_founding_context
        if ctx is None:
            return hex

        tags = list(hex.tags)
        forbidden = set(ctx.bias_against)
        preferred = set(ctx.bias_tags)

        # Remove forbidden tags
        tags = [t for t in tags if t not in forbidden]

        # Add first preferred tag if none present
        if preferred and not (set(tags) & preferred):
            # Pick first bias_tag that's valid for this hex type
            for preferred_tag in ctx.bias_tags:
                # Skip underground if we're holding off on that
                if preferred_tag in ("underground", "deep", "shallow_under"):
                    continue
                tags.append(preferred_tag)
                break

        return TaggedHex(
            q=hex.q,
            r=hex.r,
            name=hex.name,
            description=hex.description,
            tags=tags,
            edge_types=hex.edge_types,
            scale_score=hex.scale_score,
            founding_context=hex.founding_context,
            founding_cluster_id=hex.founding_cluster_id,
        )

    def _build_diversity_hints(self, hexes: dict[str, "TaggedHex"]) -> str:
        """Build hints to encourage tag variety based on current distribution.

        If any function/culture tag appears in >25% of hexes, suggest alternatives.
        """
        if len(hexes) < 4:
            return ""

        from collections import Counter

        # Count function and culture tags
        function_tags = ["residential", "military", "sacred", "industrial", "commercial", "passage"]
        culture_tags = ["dwarf", "elf", "human", "wild", "ancient", "corrupted"]

        func_counts: Counter = Counter()
        culture_counts: Counter = Counter()

        for h in hexes.values():
            for tag in h.tags:
                if tag in function_tags:
                    func_counts[tag] += 1
                elif tag in culture_tags:
                    culture_counts[tag] += 1

        hints = []
        threshold = len(hexes) * 0.25  # 25% threshold

        # Check for overused function tags
        for tag, count in func_counts.items():
            if count >= threshold:
                alternatives = [t for t in function_tags if t != tag and func_counts[t] < threshold]
                if alternatives:
                    hints.append(f"Already have {count} {tag} hexes. Try: {', '.join(alternatives[:3])}")

        # Check for overused culture tags
        for tag, count in culture_counts.items():
            if count >= threshold:
                alternatives = [t for t in culture_tags if t != tag and culture_counts[t] < threshold]
                if alternatives:
                    hints.append(f"Already have {count} {tag} hexes. Try: {', '.join(alternatives[:3])}")

        if hints:
            return "DIVERSITY NOTE: " + "; ".join(hints)
        return ""

    def _generate_single_hex(
        self,
        q: int,
        r: int,
        adjacency_context: dict,
        constraint_hints: str = "",
        diversity_hints: str = "",
    ) -> TaggedHex:
        """Generate a single hex with LLM."""
        # Format adjacency context for prompt
        if adjacency_context["neighbors"]:
            context_str = json.dumps(adjacency_context["neighbors"], indent=2)
        else:
            context_str = "None - this is an edge hex"

        # Build founding context for settlement hexes
        founding_context_str = self._build_founding_context_str()

        # Combine all hints
        all_hints = "\n".join(h for h in [constraint_hints, diversity_hints] if h)

        prompt = HEX_GENERATION_PROMPT.format(
            q=q,
            r=r,
            adjacency_context=context_str,
            founding_context=founding_context_str,
            constraint_hints=all_hints,
        )

        response = self._call_llm(prompt)

        # Parse and validate
        return TaggedHex(
            q=response["q"],
            r=response["r"],
            name=response["name"],
            description=response["description"],
            tags=response["tags"],
            edge_types=[EdgeType(e) for e in response["edge_types"]],
            founding_context=self._current_founding_context,
            founding_cluster_id=self._current_founding_cluster_id,
        )

    def _call_llm(self, prompt: str) -> dict:
        """Call LLM API."""
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
                    {"role": "system", "content": "You are a fantasy world generator. Output valid JSON only."},
                    {"role": "user", "content": prompt},
                ],
                "temperature": 0.8,
                "max_tokens": 1024,
                "response_format": {"type": "json_object"},
            },
        )
        response.raise_for_status()
        content = response.json()["choices"][0]["message"]["content"]
        return json.loads(content)

    def _compute_adjacencies(
        self,
        hexes: dict[str, TaggedHex],
    ) -> list[tuple[int, int, int]]:
        """Compute adjacency list from hex positions."""
        hex_list = list(hexes.values())
        coord_to_idx = {(h.q, h.r): i for i, h in enumerate(hex_list)}

        adjacencies = []
        for idx, hex in enumerate(hex_list):
            for edge in range(6):
                nq, nr = get_neighbor(hex.q, hex.r, edge)
                neighbor_idx = coord_to_idx.get((nq, nr))
                if neighbor_idx is not None and neighbor_idx > idx:
                    adjacencies.append((idx, neighbor_idx, edge))

        return adjacencies

    def close(self):
        self.client.close()

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
