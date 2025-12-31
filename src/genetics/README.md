# Genetics Module

> Genome, phenotype, personality, and values. The biological foundation for emergent behavior.

## Module Structure

```
genetics/
├── mod.rs          # Module exports
├── genome.rs       # Genetic data (stub)
├── phenotype.rs    # Physical trait expression (stub)
├── personality.rs  # Personality traits (stub)
└── values.rs       # Value calculation (stub)
```

## Status: Stub Implementation

This module is planned but not yet implemented. Currently, values are set directly on entities.

## Planned Design

### Genome

```rust
pub struct Genome {
    pub chromosomes: Vec<Chromosome>,
}

pub struct Chromosome {
    pub genes: Vec<Gene>,
}

pub struct Gene {
    pub trait_id: TraitId,
    pub allele_a: Allele,
    pub allele_b: Allele,
}

impl Genome {
    // Inheritance
    pub fn crossover(parent_a: &Genome, parent_b: &Genome) -> Genome {
        // Genetic crossover with recombination
    }

    // Random variation
    pub fn mutate(&mut self, rate: f32) {
        // Small random changes
    }
}
```

### Phenotype

Physical traits expressed from genome:

```rust
pub struct Phenotype {
    pub height: f32,
    pub strength: f32,
    pub endurance: f32,
    pub speed: f32,
    pub perception_range: f32,
}

impl Phenotype {
    pub fn from_genome(genome: &Genome) -> Self {
        // Express genes into physical traits
        // Multiple genes may affect same trait
        // Environment may modify expression
    }
}
```

### Personality

Big Five personality model:

```rust
pub struct Personality {
    pub openness: f32,        // Curiosity, creativity
    pub conscientiousness: f32, // Organization, diligence
    pub extraversion: f32,    // Sociability, energy
    pub agreeableness: f32,   // Cooperation, trust
    pub neuroticism: f32,     // Emotional sensitivity
}

impl Personality {
    pub fn from_genome(genome: &Genome) -> Self {
        // Personality has genetic component
    }

    pub fn develop(&mut self, experiences: &[Experience]) {
        // Personality also shaped by experience
    }
}
```

### Values

Species values emerge from personality and culture:

```rust
impl HumanValues {
    pub fn from_personality(personality: &Personality) -> Self {
        // High openness → high curiosity value
        // High agreeableness → high loyalty value
        // High neuroticism → high safety value
        // etc.
    }

    pub fn influenced_by_culture(&mut self, culture: &Culture) {
        // Cultural context shifts value weights
    }
}
```

## The Emergence Chain

```
Genome
   │
   ▼
Phenotype (physical traits)
   │
   ▼
Personality (behavioral tendencies)
   │
   ▼
Values (what matters to this entity)
   │
   ▼
Perception Filter (what they notice)
   │
   ▼
Thought Generation (how they react)
   │
   ▼
Action Selection (what they do)
```

Each layer builds on the previous, creating unique individuals.

## Integration Points

### With `entity/species/`
- Phenotype affects BodyState capabilities
- Values used in perception and selection

### With `simulation/perception.rs`
- Values filter what entities notice

### With `simulation/action_select.rs`
- Personality weights action choices

## Species Differences

Different species have different:
- **Genome structures** (different genes)
- **Phenotype ranges** (different physical capabilities)
- **Value vocabularies** (different concepts matter)

```rust
// Humans
pub struct HumanGenome { /* ... */ }
pub struct HumanPhenotype { /* ... */ }
pub struct HumanValues { honor, beauty, curiosity, ... }

// Dwarves (future)
pub struct DwarfGenome { /* ... */ }
pub struct DwarfPhenotype { /* stone sense, thermal regulation, ... */ }
pub struct DwarfValues { tradition, craftsmanship, clan_honor, ... }
```

## Future Implementation

1. **Start with simple genome** - few genes affecting major traits
2. **Implement phenotype** expression from genome
3. **Add personality** derived from genome
4. **Connect values** to personality
5. **Add inheritance** for entity reproduction
6. **Add mutation** for generational variation
