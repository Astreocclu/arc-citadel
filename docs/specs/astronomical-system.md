# Arc Citadel: Astronomical System Specification

> **DEVELOPMENT PHILOSOPHY: Agent Prompts > Code**
> Design prompts and specs first. Coding LLMs implement.
> This document defines WHAT. Agents determine HOW.

**Version:** 1.0
**Last Updated:** January 2025
**Status:** Implementation Ready

---

## Purpose

This document specifies the astronomical system: sun, moons, time cycles, and their effects on entity behavior, world generation, and settlement properties. The core insight: **astronomy creates emergent calendrical complexity from simple orbital math, feeding cultural differentiation and founding-condition mechanics**.

---

## Design Philosophy

### Predictable Complexity

Orbital mechanics are deterministic. Any entity with sufficient astronomical knowledge can predict events. But the interaction of multiple periods creates apparent complexity—patterns that feel discovered rather than authored.

### Cultural Divergence Engine

Different species attend to different celestial bodies:
- Humans: Solar calendar, agricultural focus
- Elves: Dual-moon tracking, pattern recognition
- Dwarves: Largely indifferent (underground), but track deep-earth tidal effects

### Founding Conditions Matter

When a settlement is founded affects its mechanical properties. A hold established in deep winter during the Dark develops differently than one founded at midsummer under the Double Full.

---

## Orbital Parameters

### The Sun (Solara)

| Parameter | Value | Notes |
|-----------|-------|-------|
| Year length | 360 days | Clean division for seasons |
| Day length | 24 hours | Familiar baseline |
| Axial tilt | 23° | Earth-like seasons |
| Season length | 90 days | 4 equal seasons |

**Seasons:**
- **Spring** (Days 1-90): Thaw, planting, birth season
- **Summer** (Days 91-180): Growth, abundance, campaign season
- **Autumn** (Days 181-270): Harvest, preparation, raiding season
- **Winter** (Days 271-360): Scarcity, survival, siege season

### The Silver Moon (Argent)

| Parameter | Value | Notes |
|-----------|-------|-------|
| Orbital period | 29 days | Prime number, Earth-like feel |
| Orbital inclination | 5.1° | Creates eclipse rarity |
| Eclipse node precession | ~18 years | Shifting eclipse seasons |

**Cultural associations:**
- Tides and water
- Fertility and birth cycles
- Short-term planning
- "Common" magic (if applicable)

### The Blood Moon (Sanguine)

| Parameter | Value | Notes |
|-----------|-------|-------|
| Orbital period | 83 days | Prime number, no common factors with 29 |
| Orbital inclination | 7.3° | Different eclipse pattern |
| Eclipse node precession | ~31 years | Long cycle |

**Cultural associations:**
- War and conflict
- Long-term change
- Omens and portents
- "Deep" magic (if applicable)

### Mathematical Properties

**Perfect Alignment Cycle:**
```
LCM(29, 83) = 2,407 days ≈ 6.69 years

Every 6.69 years: Both moons reach identical phase simultaneously
This is the "Grand Cycle" or "Conjunction Year"
```

**Phase Relationships:**
```
Both moons full simultaneously: Once per 2,407 days
Both moons new simultaneously: Once per 2,407 days (offset)
Opposite phases (one full, one new): Twice per 2,407 days
Near-alignment (within 2 days): ~Every 4-6 months
```

---

## Time-of-Day System

### Solar Phases

| Phase | Time Range | Light Level | Entity Effects |
|-------|------------|-------------|----------------|
| DEEP_NIGHT | 00:00-04:00 | 0.0-0.1 | Nocturnal peak, human vision penalty |
| PRE_DAWN | 04:00-06:00 | 0.1-0.3 | Transition, ambush timing |
| DAWN | 06:00-08:00 | 0.3-0.7 | Wake cycle, morale bonus |
| MORNING | 08:00-11:00 | 0.7-0.9 | Peak productivity |
| MIDDAY | 11:00-14:00 | 1.0 | Heat fatigue (summer), full visibility |
| AFTERNOON | 14:00-17:00 | 0.9-0.7 | Standard activity |
| DUSK | 17:00-19:00 | 0.7-0.3 | Transition, return-to-shelter instinct |
| EVENING | 19:00-22:00 | 0.3-0.1 | Social time, reduced activity |
| NIGHT | 22:00-00:00 | 0.1-0.0 | Sleep cycle, nocturnal emergence |

**Seasonal Variation:**
- Summer: Dawn earlier (05:00), Dusk later (21:00)
- Winter: Dawn later (07:30), Dusk earlier (16:30)
- Affects available work hours, march distances, battle timing

### Lunar Light Contribution

```
Moon light contribution = base_albedo × phase_factor × elevation_factor

Full moon adds +0.15 to night light level (per moon)
Double Full: Night becomes "bright night" (0.3 base)
Double New: "True darkness" (0.0, even elves struggle)
```

---

## Celestial Events

### Common Events (Entity Behavior Triggers)

| Event | Frequency | Detection | Effects |
|-------|-----------|-----------|---------|
| Full Argent | Every 29 days | Automatic | +Tidal fishing, +Fertility checks, Wolf activity |
| New Argent | Every 29 days | Automatic | Stealth bonus, Darkness penalty |
| Full Sanguine | Every 83 days | Automatic | +Aggression modifier, Battle omens |
| New Sanguine | Every 83 days | Automatic | Deep darkness, Underground creature activity |

### Uncommon Events (Cultural Significance)

| Event | Frequency | Detection | Effects |
|-------|-----------|-----------|---------|
| Near Double Full | Every 4-6 months | Astronomy skill | Festival timing, Morale boost |
| Near Double New (The Dimming) | Every 4-6 months | Astronomy skill | Increased supernatural activity |
| Silver Eclipse | 2-3 per year | Location-specific | Omen interpretation, Brief darkness |
| Blood Eclipse | 1-2 per year | Location-specific | Major omen, Extended dimming |

### Rare Events (History-Shaping)

| Event | Frequency | Detection | Effects |
|-------|-----------|-----------|---------|
| Perfect Double Full (The Radiance) | Every 6.69 years | Astronomy skill (predicted), Obvious (when occurring) | Major fertility event, Founding bonus, Festival |
| Perfect Double New (The Dark) | Every 6.69 years | Same | Supernatural peak, Founding penalty OR theocratic bonus |
| Double Eclipse | Every 15-25 years | Advanced astronomy | Cataclysmic omen, History-generation trigger |
| Grand Conjunction + Solstice | Every ~50 years | Master astronomy | Legendary event, Unique founding myths |

---

## Founding Conditions System

### Season at Founding

When a settlement is founded, the season affects initial mechanical properties.

| Founding Season | Mechanical Effects |
|-----------------|-------------------|
| **Deep Winter** (Days 300-360) | `stockpile_efficiency: +15%`, `initial_population: -20%`, `defensive_architecture_weight: +0.3`, `siege_mentality: true` |
| **Early Spring** (Days 1-45) | `growth_rate: +10%`, `agricultural_priority: +0.2`, `optimism_baseline: +0.1` |
| **Late Spring** (Days 46-90) | `fertility_rate: +15%`, `expansion_tendency: +0.2` |
| **Summer** (Days 91-180) | `initial_population: +15%`, `trade_infrastructure: +0.2`, `defensive_instinct: -0.1` |
| **Autumn** (Days 181-270) | `harvest_storage: +20%`, `preparation_trait: true`, `balanced modifiers` |
| **Early Winter** (Days 271-299) | `resource_efficiency: +10%`, `caution_baseline: +0.15` |

### Astronomical Event at Founding

If a notable astronomical event occurs within 3 days of founding:

| Founding Event | Mechanical Effects |
|----------------|-------------------|
| **Double Full** | `morale_baseline: +0.1`, `expansion_tendency: +0.25`, `blessed_tag: true`, `fertility_bonus: +10%` |
| **The Dark** | `underground_preference: +0.3`, `stealth_culture: +0.2`, `superstition_weight: +0.2`, `secrecy_trait: true` |
| **Silver Eclipse** | `theocratic_tendency: +0.15`, `astronomy_priority: +0.3`, `silver_association: true` |
| **Blood Eclipse** | `martial_culture: +0.2`, `omen_sensitivity: +0.25`, `blood_association: true` |
| **Double Eclipse** | `unique_founding_myth: true`, `supernatural_affinity: +0.3`, `isolation_tendency: +0.2` |

### Combined Effects

Effects stack. A settlement founded in Deep Winter during The Dark:

```
SETTLEMENT: Thornhold
Founded: Day 342, Year 1 (Deep Winter)
Astronomical: The Dark (Double New Moon)

COMBINED MODIFIERS:
├── stockpile_efficiency: +15% (winter)
├── initial_population: -20% (winter)
├── defensive_architecture_weight: +0.3 (winter)
├── siege_mentality: true (winter)
├── underground_preference: +0.3 (dark)
├── stealth_culture: +0.2 (dark)
├── superstition_weight: +0.2 (dark)
└── secrecy_trait: true (dark)

NARRATIVE IMPLICATION:
"Founded in the harshest season under lightless skies,
Thornhold's people learned to hoard, to hide, to endure.
Their halls run deep and their secrets deeper."
```

---

## Astronomy Skill System

### Skill Tiers

| Tier | Capability |
|------|------------|
| **None** (default) | Notices current phase, recognizes full/new moons |
| **Basic** | Predicts moon phases 1 month ahead, knows eclipse seasons |
| **Intermediate** | Predicts phases 1 year ahead, calculates Double events |
| **Advanced** | Predicts eclipses with location, knows Conjunction years |
| **Master** | Full orbital model, predicts all events indefinitely |

### Prediction Mechanics

```
prediction_accuracy = base_accuracy × (1 - time_distance_factor)

Base accuracy by tier:
├── Basic: 0.8
├── Intermediate: 0.95
├── Advanced: 0.99
└── Master: 1.0

Time distance factor:
├── Days ahead / (tier_limit × 30)
└── Clamped to [0, 0.5]

Example: Intermediate astronomer predicting 8 months ahead
accuracy = 0.95 × (1 - (240 / (2 × 30 × 10))) = 0.95 × 0.6 = 0.57
(Unreliable prediction—beyond comfortable range)
```

### Cultural Astronomy

| Species | Default Astronomy | Notes |
|---------|-------------------|-------|
| Human | Basic (farmers), None (others) | Solar/agricultural focus |
| Elf | Intermediate baseline | Both moons tracked, pattern-seekers |
| Dwarf | None (surface), Basic (tidal/seismic) | Different celestial model entirely |

---

## Entity Behavior Modifiers

### Time-of-Day Effects

```rust
struct TimeOfDayModifiers {
    // Vision
    human_perception_mult: f32,      // 1.0 day, 0.3 night, 0.1 deep night
    elf_perception_mult: f32,        // 1.0 day, 0.7 night, 0.4 deep night
    dwarf_perception_mult: f32,      // 0.9 day (surface), 1.0 underground always

    // Activity
    diurnal_activity_mult: f32,      // Humans, most creatures
    nocturnal_activity_mult: f32,    // Wolves, certain monsters
    crepuscular_activity_mult: f32,  // Dawn/dusk hunters

    // Combat
    ambush_success_modifier: f32,    // Higher at night
    fatigue_recovery_modifier: f32,  // Higher during species sleep cycle
}
```

### Lunar Effects

```rust
struct LunarModifiers {
    // Argent (Silver Moon)
    tidal_strength: f32,             // Affects fishing, coastal hexes
    fertility_modifier: f32,         // Birth rate calculations
    lycanthrope_activity: f32,       // If applicable

    // Sanguine (Blood Moon)
    aggression_modifier: f32,        // Combat initiation threshold
    omen_weight: f32,                // Entity superstition response
    deep_creature_activity: f32,     // Underground emergence

    // Combined
    night_light_level: f32,          // Sum of both contributions
    supernatural_activity: f32,      // If applicable
}
```

---

## World Generation Integration

### Seed Parameters

```rust
struct AstronomicalSeed {
    // Starting conditions
    start_day_of_year: u16,          // 1-360
    start_time_of_day: f32,          // 0.0-24.0
    argent_phase: f32,               // 0.0-1.0 (0=new, 0.5=full)
    sanguine_phase: f32,             // 0.0-1.0

    // Orbital parameters (usually constant, but moddable)
    year_length: u16,                // Default 360
    argent_period: u8,               // Default 29
    sanguine_period: u8,             // Default 83

    // Eclipse parameters
    argent_node_position: f32,       // Current eclipse season
    sanguine_node_position: f32,
}
```

### History Generation Hooks

When generating settlement history, the LLM receives:

```
FOUNDING_CONDITIONS:
├── date: "Day 142, Year -847"
├── season: "Summer"
├── time_of_day: "Dawn"
├── argent_phase: "Waxing Gibbous"
├── sanguine_phase: "Full"
├── astronomical_event: "Blood Moon Full"
├── mechanical_modifiers: { ... }
└── prompt_context: "Founded at dawn under a full Blood Moon
    in high summer—a martial beginning under war-omens,
    yet blessed with abundance and long days."
```

---

## Implementation Notes

### Tick Integration

Astronomical state updates once per game-day (not per tick):

```
Daily astronomical update:
1. Increment day counter
2. Update sun position (season, time progression)
3. Update moon phases (simple modular arithmetic)
4. Check for event triggers
5. Cache light levels for the day
6. Notify relevant systems (perception, behavior)
```

### Performance Considerations

- Orbital calculations are O(1)—just modular arithmetic
- Event checking is O(1)—precomputed event calendar
- Light level is cached daily, not computed per-entity
- Eclipse visibility is spatial query (which hexes see it)

### Data Storage

```rust
struct AstronomicalState {
    current_day: u32,                // Days since epoch
    current_time: f32,               // Hours (0.0-24.0)

    // Cached daily values
    season: Season,
    solar_phase: SolarPhase,
    argent_phase: MoonPhase,
    sanguine_phase: MoonPhase,
    base_light_level: f32,
    active_events: Vec<CelestialEvent>,

    // Precomputed calendar (sparse—only notable events)
    event_calendar: HashMap<u32, Vec<CelestialEvent>>,
}
```

---

## Relationship to Other Specs

| Spec | Integration Point |
|------|-------------------|
| 03 - Entity Simulation | Time-of-day perception modifiers, behavior triggers |
| 08 - Genetics System | Species-specific astronomical awareness |
| 10 - Performance Architecture | Daily update batching, light level caching |
| 15 - World Generation | Founding conditions, history generation prompts |
| 17 - Social Memory | Festival/event memory tied to astronomical calendar |

---

## Open Questions

1. **Tidal mechanics** — Do we simulate actual tides affecting coastal hexes, or just abstract "tidal fishing bonus"?
2. **Supernatural integration** — If magic exists, how tightly coupled to lunar cycles?
3. **Calendar systems** — Do species have named months? Religious holidays tied to astronomy?
4. **Climate interaction** — Does moon position affect weather, or is that purely solar/seasonal?

---

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | Jan 2025 | Initial specification |

---

*This spec defines the astronomical system. For world generation, see 15-WORLD-GENERATION-SPEC-V2. For entity behavior, see 03-ENTITY-SIMULATION-SPEC.*
