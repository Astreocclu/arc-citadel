# 08-GENETICS-SYSTEM-SPEC
> Genome structure, phenotype derivation, and species-specific value generation

## Overview

The Genetics System creates entity diversity through a pipeline: **Genome → Phenotype → Personality → Values**. This produces entities with unique physical traits and psychological profiles without requiring manual character design. Each species has distinct gene pools and value vocabularies that are **type-incompatible**.

---

## Core Pipeline

```
┌─────────────────────────────────────────────────────────────────────┐
│                         GENETICS PIPELINE                            │
│                                                                      │
│   ┌──────────┐     ┌───────────┐     ┌─────────────┐     ┌────────┐│
│   │  Genome  │────▶│ Phenotype │────▶│ Personality │────▶│ Values ││
│   └──────────┘     └───────────┘     └─────────────┘     └────────┘│
│                                                                      │
│   DNA base pairs   Physical traits   Psychological     Value        │
│   inherited from   derived from      traits derived    priorities   │
│   parents          genes             from physiology   for species  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Genome Structure

### Gene Representation

```rust
/// A gene is a pair of alleles (one from each parent)
#[derive(Clone, Copy)]
pub struct Gene {
    pub allele_a: Allele,  // From parent A
    pub allele_b: Allele,  // From parent B
}

/// An allele is a single genetic variant
#[derive(Clone, Copy)]
pub struct Allele {
    pub value: f32,        // 0.0-1.0 base expression
    pub dominance: f32,    // 0.0-1.0 (recessive to dominant)
    pub stability: f32,    // Mutation resistance
}

impl Gene {
    /// Express the gene based on dominance rules
    pub fn express(&self) -> f32 {
        let weight_a = self.allele_a.dominance;
        let weight_b = self.allele_b.dominance;
        let total = weight_a + weight_b;

        if total == 0.0 {
            // Co-dominant: average
            (self.allele_a.value + self.allele_b.value) / 2.0
        } else {
            // Weighted by dominance
            (self.allele_a.value * weight_a + self.allele_b.value * weight_b) / total
        }
    }

    /// Create offspring gene from two parents
    pub fn inherit(parent_a: &Gene, parent_b: &Gene, rng: &mut impl Rng) -> Gene {
        // Each parent contributes one allele
        let allele_a = if rng.gen_bool(0.5) {
            parent_a.allele_a
        } else {
            parent_a.allele_b
        };

        let allele_b = if rng.gen_bool(0.5) {
            parent_b.allele_a
        } else {
            parent_b.allele_b
        };

        // Mutation chance
        let allele_a = Self::maybe_mutate(allele_a, rng);
        let allele_b = Self::maybe_mutate(allele_b, rng);

        Gene { allele_a, allele_b }
    }

    fn maybe_mutate(mut allele: Allele, rng: &mut impl Rng) -> Allele {
        let mutation_chance = 0.01 * (1.0 - allele.stability);
        if rng.gen::<f32>() < mutation_chance {
            // Small random shift
            let delta = rng.gen_range(-0.1..0.1);
            allele.value = (allele.value + delta).clamp(0.0, 1.0);
        }
        allele
    }
}
```

### Species Genomes

Each species has a different gene pool structure:

```rust
/// Human genome - genes affecting human-specific traits
pub struct HumanGenome {
    // Physical genes
    pub height: Gene,
    pub build: Gene,        // Slight to Heavy
    pub metabolism: Gene,
    pub immune_strength: Gene,

    // Mental genes
    pub neural_density: Gene,
    pub memory_capacity: Gene,
    pub reaction_speed: Gene,

    // Personality genes
    pub introversion: Gene,
    pub conscientiousness: Gene,
    pub openness: Gene,
    pub agreeableness: Gene,
    pub neuroticism: Gene,

    // Appearance genes (for recognition)
    pub skin_tone: Gene,
    pub hair_color: Gene,
    pub eye_color: Gene,
}

/// Dwarf genome - entirely different gene pool
pub struct DwarfGenome {
    // Physical genes
    pub density: Gene,           // Bone/muscle density
    pub stone_affinity: Gene,    // Sensitivity to stone types
    pub heat_tolerance: Gene,    // Forge work adaptation
    pub dark_vision: Gene,

    // Mental genes
    pub pattern_memory: Gene,    // Craft pattern retention
    pub grudge_retention: Gene,  // Memory for slights
    pub tradition_pull: Gene,    // Adherence to tradition

    // Clan genes
    pub beard_growth: Gene,
    pub clan_markers: Gene,
}

/// Elf genome - different again
pub struct ElfGenome {
    // Physical genes
    pub longevity: Gene,
    pub grace: Gene,
    pub nature_attunement: Gene,
    pub starlight_sensitivity: Gene,

    // Mental genes
    pub pattern_perception: Gene,
    pub temporal_patience: Gene,  // Tolerance for long waits
    pub dream_depth: Gene,

    // Appearance genes
    pub ear_shape: Gene,
    pub eye_luminance: Gene,
}
```

---

## Phenotype Derivation

### Physical Traits

```rust
pub struct Phenotype {
    // Core physical stats
    pub strength: f32,
    pub endurance: f32,
    pub agility: f32,
    pub size: f32,

    // Derived physical
    pub health_max: f32,
    pub fatigue_rate: f32,
    pub recovery_rate: f32,

    // Mental stats
    pub intelligence: f32,
    pub perception: f32,
    pub willpower: f32,

    // Appearance
    pub appearance: Appearance,
}

impl Phenotype {
    /// Derive phenotype from human genome
    pub fn from_human_genome(genome: &HumanGenome) -> Self {
        let height = genome.height.express();
        let build = genome.build.express();

        // Physical stats emerge from gene interaction
        let strength = build * 0.7 + genome.metabolism.express() * 0.3;
        let endurance = genome.metabolism.express() * 0.5 + build * 0.3 + height * 0.2;
        let agility = (1.0 - build) * 0.4 + genome.reaction_speed.express() * 0.6;
        let size = height * 0.6 + build * 0.4;

        // Derived stats
        let health_max = size * 50.0 + endurance * 50.0;
        let fatigue_rate = 1.0 / (endurance * 0.5 + genome.metabolism.express() * 0.5);
        let recovery_rate = genome.metabolism.express() * 0.5 + endurance * 0.3;

        // Mental stats
        let intelligence = genome.neural_density.express() * 0.6
            + genome.memory_capacity.express() * 0.4;
        let perception = genome.reaction_speed.express() * 0.4
            + genome.openness.express() * 0.3
            + genome.neural_density.express() * 0.3;
        let willpower = genome.conscientiousness.express() * 0.4
            + (1.0 - genome.neuroticism.express()) * 0.3
            + genome.introversion.express() * 0.3;

        Phenotype {
            strength,
            endurance,
            agility,
            size,
            health_max,
            fatigue_rate,
            recovery_rate,
            intelligence,
            perception,
            willpower,
            appearance: Appearance::from_genome(genome),
        }
    }
}
```

### Appearance

```rust
pub struct Appearance {
    pub height_cm: u16,
    pub build_desc: BuildDescription,
    pub skin_tone: SkinTone,
    pub hair_color: HairColor,
    pub eye_color: EyeColor,
    pub distinguishing_features: Vec<Feature>,
}

#[derive(Clone, Copy)]
pub enum BuildDescription {
    Slight,
    Lean,
    Average,
    Athletic,
    Stocky,
    Heavy,
}

impl Appearance {
    pub fn from_genome(genome: &HumanGenome) -> Self {
        let height_cm = (150.0 + genome.height.express() * 50.0) as u16;  // 150-200cm range

        let build_desc = match genome.build.express() {
            x if x < 0.15 => BuildDescription::Slight,
            x if x < 0.30 => BuildDescription::Lean,
            x if x < 0.50 => BuildDescription::Average,
            x if x < 0.70 => BuildDescription::Athletic,
            x if x < 0.85 => BuildDescription::Stocky,
            _ => BuildDescription::Heavy,
        };

        Appearance {
            height_cm,
            build_desc,
            skin_tone: SkinTone::from_gene(genome.skin_tone.express()),
            hair_color: HairColor::from_gene(genome.hair_color.express()),
            eye_color: EyeColor::from_gene(genome.eye_color.express()),
            distinguishing_features: Vec::new(),
        }
    }
}
```

---

## Personality Derivation

### Big Five Model

```rust
/// Personality traits derived from phenotype
pub struct Personality {
    pub openness: f32,          // 0.0-1.0
    pub conscientiousness: f32,
    pub extraversion: f32,
    pub agreeableness: f32,
    pub neuroticism: f32,
}

impl Personality {
    pub fn from_genome(genome: &HumanGenome) -> Self {
        Personality {
            openness: genome.openness.express(),
            conscientiousness: genome.conscientiousness.express(),
            extraversion: 1.0 - genome.introversion.express(),
            agreeableness: genome.agreeableness.express(),
            neuroticism: genome.neuroticism.express(),
        }
    }
}
```

---

## Species-Specific Values

### Core Design Principle

**Values are type-incompatible across species.** A human's "honor" and a dwarf's "clan honor" are fundamentally different concepts that cannot be compared.

```rust
// ❌ FORBIDDEN: Universal value comparison
fn compare_honor(human: &HumanValues, dwarf: &DwarfValues) -> Ordering {
    human.honor.cmp(&dwarf.clan_weight)  // TYPE ERROR - doesn't compile
}

// ✅ CORRECT: Each species has its own value type
impl HumanValues { /* human-specific methods */ }
impl DwarfValues { /* dwarf-specific methods */ }
```

### Human Values

```rust
/// Human values - the concepts humans care about
pub struct HumanValues {
    pub honor: f32,       // Social standing, reputation, keeping one's word
    pub beauty: f32,      // Aesthetic appreciation, crafted beauty
    pub comfort: f32,     // Physical ease, safety from discomfort
    pub ambition: f32,    // Drive for advancement, achievement
    pub loyalty: f32,     // Group attachment, in-group preference
    pub love: f32,        // Individual attachment, romantic/familial bonds
    pub justice: f32,     // Fairness, punishment of wrongdoing
    pub curiosity: f32,   // Desire to explore, learn, discover
    pub safety: f32,      // Self-preservation, risk aversion
    pub piety: f32,       // Spiritual devotion, religious observance
}

impl HumanValues {
    /// Derive values from personality and phenotype
    pub fn from_personality(personality: &Personality, phenotype: &Phenotype) -> Self {
        HumanValues {
            // Honor: conscientious + low neuroticism
            honor: personality.conscientiousness * 0.6
                + (1.0 - personality.neuroticism) * 0.4,

            // Beauty: openness + perception
            beauty: personality.openness * 0.7 + phenotype.perception * 0.01,

            // Comfort: neuroticism + (inverse) endurance
            comfort: personality.neuroticism * 0.4
                + (1.0 - phenotype.endurance) * 0.3
                + 0.3,

            // Ambition: extraversion + low agreeableness
            ambition: personality.extraversion * 0.5
                + (1.0 - personality.agreeableness) * 0.3
                + personality.conscientiousness * 0.2,

            // Loyalty: agreeableness + (inverse) openness
            loyalty: personality.agreeableness * 0.5
                + (1.0 - personality.openness) * 0.3
                + personality.conscientiousness * 0.2,

            // Love: agreeableness + extraversion
            love: personality.agreeableness * 0.5
                + personality.extraversion * 0.3
                + (1.0 - personality.neuroticism) * 0.2,

            // Justice: conscientiousness + (inverse) agreeableness
            justice: personality.conscientiousness * 0.5
                + (1.0 - personality.agreeableness) * 0.3
                + personality.openness * 0.2,

            // Curiosity: openness + intelligence
            curiosity: personality.openness * 0.7 + phenotype.intelligence * 0.01,

            // Safety: neuroticism + (inverse) extraversion
            safety: personality.neuroticism * 0.5
                + (1.0 - personality.extraversion) * 0.3
                + (1.0 - phenotype.strength) * 0.01,

            // Piety: conscientiousness + (inverse) openness
            piety: personality.conscientiousness * 0.4
                + (1.0 - personality.openness) * 0.4
                + personality.agreeableness * 0.2,
        }
    }

    /// Get the entity's top value priorities
    pub fn top_values(&self, count: usize) -> Vec<(HumanValueType, f32)> {
        let mut values = vec![
            (HumanValueType::Honor, self.honor),
            (HumanValueType::Beauty, self.beauty),
            (HumanValueType::Comfort, self.comfort),
            (HumanValueType::Ambition, self.ambition),
            (HumanValueType::Loyalty, self.loyalty),
            (HumanValueType::Love, self.love),
            (HumanValueType::Justice, self.justice),
            (HumanValueType::Curiosity, self.curiosity),
            (HumanValueType::Safety, self.safety),
            (HumanValueType::Piety, self.piety),
        ];

        values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        values.truncate(count);
        values
    }
}

#[derive(Clone, Copy, Debug)]
pub enum HumanValueType {
    Honor,
    Beauty,
    Comfort,
    Ambition,
    Loyalty,
    Love,
    Justice,
    Curiosity,
    Safety,
    Piety,
}
```

### Dwarf Values

```rust
/// Dwarf values - fundamentally different concepts
pub struct DwarfValues {
    pub craft_truth: f32,     // Honest representation of skill in work
    pub stone_debt: f32,      // Obligation to shape stone properly
    pub clan_weight: f32,     // Burden of clan expectations and honor
    pub oath_chain: f32,      // Binding nature of sworn oaths
    pub deep_memory: f32,     // Reverence for ancestral knowledge
    pub grudge_mark: f32,     // Imperative to remember and repay wrongs
}

impl DwarfValues {
    pub fn from_genome(genome: &DwarfGenome) -> Self {
        DwarfValues {
            craft_truth: genome.pattern_memory.express() * 0.6
                + genome.stone_affinity.express() * 0.4,

            stone_debt: genome.stone_affinity.express() * 0.7
                + genome.tradition_pull.express() * 0.3,

            clan_weight: genome.tradition_pull.express() * 0.5
                + genome.grudge_retention.express() * 0.3
                + genome.clan_markers.express() * 0.2,

            oath_chain: genome.tradition_pull.express() * 0.6
                + genome.pattern_memory.express() * 0.4,

            deep_memory: genome.pattern_memory.express() * 0.5
                + genome.grudge_retention.express() * 0.3
                + genome.tradition_pull.express() * 0.2,

            grudge_mark: genome.grudge_retention.express() * 0.7
                + genome.clan_markers.express() * 0.3,
        }
    }
}
```

### Elf Values

```rust
/// Elf values - yet another distinct vocabulary
pub struct ElfValues {
    pub pattern_beauty: f32,   // Appreciation of natural and magical patterns
    pub slow_growth: f32,      // Patience for gradual development
    pub star_longing: f32,     // Connection to celestial cycles
    pub cycle_wisdom: f32,     // Understanding of natural rhythms
    pub tree_bond: f32,        // Connection to living wood
    pub fate_thread: f32,      // Acceptance of destiny's weaving
}

impl ElfValues {
    pub fn from_genome(genome: &ElfGenome) -> Self {
        ElfValues {
            pattern_beauty: genome.pattern_perception.express() * 0.6
                + genome.nature_attunement.express() * 0.4,

            slow_growth: genome.temporal_patience.express() * 0.7
                + genome.longevity.express() * 0.3,

            star_longing: genome.starlight_sensitivity.express() * 0.6
                + genome.dream_depth.express() * 0.4,

            cycle_wisdom: genome.temporal_patience.express() * 0.4
                + genome.pattern_perception.express() * 0.3
                + genome.nature_attunement.express() * 0.3,

            tree_bond: genome.nature_attunement.express() * 0.7
                + genome.longevity.express() * 0.3,

            fate_thread: genome.dream_depth.express() * 0.5
                + genome.temporal_patience.express() * 0.3
                + genome.pattern_perception.express() * 0.2,
        }
    }
}
```

---

## Value Filtering

Values filter perception—entities notice things that matter to them:

```rust
impl HumanValues {
    /// How interesting is this perception to this entity?
    pub fn perception_relevance(&self, perception: &Perception) -> f32 {
        match perception {
            Perception::SeePerson { relationship, .. } => {
                self.love * 0.3 + self.loyalty * 0.2 +
                if relationship.is_some() { 0.3 } else { 0.0 }
            }
            Perception::SeeWealth { value, .. } => {
                self.ambition * 0.4 + self.comfort * 0.3
            }
            Perception::SeeDanger { threat_level, .. } => {
                self.safety * 0.5 + (1.0 - self.ambition) * 0.2
            }
            Perception::SeeBeauty { quality, .. } => {
                self.beauty * 0.6 + self.curiosity * 0.2
            }
            Perception::SeeInjustice { severity, .. } => {
                self.justice * 0.5 + self.honor * 0.3
            }
            Perception::SeeReligiousSite { .. } => {
                self.piety * 0.6 + self.curiosity * 0.2
            }
            // ... other perceptions
            _ => 0.1,  // Base interest
        }
    }
}
```

---

## Spawning System

### Random Generation

```rust
pub fn spawn_random_human(rng: &mut impl Rng) -> (HumanGenome, Phenotype, Personality, HumanValues) {
    let genome = HumanGenome::random(rng);
    let phenotype = Phenotype::from_human_genome(&genome);
    let personality = Personality::from_genome(&genome);
    let values = HumanValues::from_personality(&personality, &phenotype);

    (genome, phenotype, personality, values)
}

impl HumanGenome {
    pub fn random(rng: &mut impl Rng) -> Self {
        HumanGenome {
            height: Gene::random(rng),
            build: Gene::random(rng),
            metabolism: Gene::random(rng),
            immune_strength: Gene::random(rng),
            neural_density: Gene::random(rng),
            memory_capacity: Gene::random(rng),
            reaction_speed: Gene::random(rng),
            introversion: Gene::random(rng),
            conscientiousness: Gene::random(rng),
            openness: Gene::random(rng),
            agreeableness: Gene::random(rng),
            neuroticism: Gene::random(rng),
            skin_tone: Gene::random(rng),
            hair_color: Gene::random(rng),
            eye_color: Gene::random(rng),
        }
    }
}

impl Gene {
    pub fn random(rng: &mut impl Rng) -> Self {
        Gene {
            allele_a: Allele {
                value: rng.gen(),
                dominance: rng.gen(),
                stability: rng.gen_range(0.8..1.0),
            },
            allele_b: Allele {
                value: rng.gen(),
                dominance: rng.gen(),
                stability: rng.gen_range(0.8..1.0),
            },
        }
    }
}
```

### Breeding

```rust
pub fn breed_humans(
    parent_a: &HumanGenome,
    parent_b: &HumanGenome,
    rng: &mut impl Rng,
) -> (HumanGenome, Phenotype, Personality, HumanValues) {
    let child_genome = HumanGenome {
        height: Gene::inherit(&parent_a.height, &parent_b.height, rng),
        build: Gene::inherit(&parent_a.build, &parent_b.build, rng),
        metabolism: Gene::inherit(&parent_a.metabolism, &parent_b.metabolism, rng),
        // ... inherit all genes
        immune_strength: Gene::inherit(&parent_a.immune_strength, &parent_b.immune_strength, rng),
        neural_density: Gene::inherit(&parent_a.neural_density, &parent_b.neural_density, rng),
        memory_capacity: Gene::inherit(&parent_a.memory_capacity, &parent_b.memory_capacity, rng),
        reaction_speed: Gene::inherit(&parent_a.reaction_speed, &parent_b.reaction_speed, rng),
        introversion: Gene::inherit(&parent_a.introversion, &parent_b.introversion, rng),
        conscientiousness: Gene::inherit(&parent_a.conscientiousness, &parent_b.conscientiousness, rng),
        openness: Gene::inherit(&parent_a.openness, &parent_b.openness, rng),
        agreeableness: Gene::inherit(&parent_a.agreeableness, &parent_b.agreeableness, rng),
        neuroticism: Gene::inherit(&parent_a.neuroticism, &parent_b.neuroticism, rng),
        skin_tone: Gene::inherit(&parent_a.skin_tone, &parent_b.skin_tone, rng),
        hair_color: Gene::inherit(&parent_a.hair_color, &parent_b.hair_color, rng),
        eye_color: Gene::inherit(&parent_a.eye_color, &parent_b.eye_color, rng),
    };

    let phenotype = Phenotype::from_human_genome(&child_genome);
    let personality = Personality::from_genome(&child_genome);
    let values = HumanValues::from_personality(&personality, &phenotype);

    (child_genome, phenotype, personality, values)
}
```

---

## Cross-Species Interaction

Since values are type-incompatible, cross-species interaction requires translation:

```rust
/// Trait for species-agnostic value queries
pub trait ValueHolder {
    /// Get the entity's most urgent current priority
    fn current_priority(&self) -> ValuePriority;

    /// How does this entity react to violence?
    fn violence_response(&self) -> ViolenceResponse;

    /// What triggers this entity's anger?
    fn anger_triggers(&self) -> Vec<AngerTrigger>;
}

/// Common priority categories for cross-species interaction
pub enum ValuePriority {
    SelfPreservation,
    GroupPreservation,
    ResourceAcquisition,
    SocialStanding,
    TaskCompletion,
    Exploration,
}

impl ValueHolder for HumanValues {
    fn current_priority(&self) -> ValuePriority {
        // Map human values to common priority
        let top = self.top_values(1)[0].0;
        match top {
            HumanValueType::Safety => ValuePriority::SelfPreservation,
            HumanValueType::Loyalty | HumanValueType::Love => ValuePriority::GroupPreservation,
            HumanValueType::Ambition | HumanValueType::Comfort => ValuePriority::ResourceAcquisition,
            HumanValueType::Honor => ValuePriority::SocialStanding,
            HumanValueType::Curiosity => ValuePriority::Exploration,
            _ => ValuePriority::TaskCompletion,
        }
    }

    // ... other implementations
}
```

---

## Related Specifications

| Document | Relationship |
|----------|--------------|
| [01-GAME-DESIGN-DOCUMENT](01-GAME-DESIGN-DOCUMENT.md) | Species-Specific Cognition pillar |
| [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md) | Value filtering in perception |
| [17-SOCIAL-MEMORY-SPEC](17-SOCIAL-MEMORY-SPEC.md) | Value-based relationship formation |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification created |
