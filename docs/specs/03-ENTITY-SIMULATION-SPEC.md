# 03-ENTITY-SIMULATION-SPEC
> Entity model, cognition pipeline, and behavior emergence

## Overview

Entities in Arc Citadel are autonomous agents with needs, values, thoughts, and tasks. Their behavior emerges from the interaction of these systems rather than scripted patterns. This specification defines the complete entity model and the perception → thought → action pipeline.

---

## Entity Structure

### Complete Entity Model

```
┌──────────────────────────────────────────────────────────────────┐
│                         ENTITY                                    │
├──────────────────────────────────────────────────────────────────┤
│  IDENTITY                                                        │
│  ├── id: EntityId (u64)                                          │
│  ├── name: String                                                │
│  └── species: Species enum                                       │
├──────────────────────────────────────────────────────────────────┤
│  BODY (Physical State)                                           │
│  ├── position: Vec2                                              │
│  ├── velocity: Vec2                                              │
│  ├── body_state: BodyState                                       │
│  │   ├── health: f32 (0.0-1.0)                                   │
│  │   ├── fatigue: f32 (0.0-1.0)                                  │
│  │   ├── wounds: Vec<Wound>                                      │
│  │   └── body_parts: [BodyPart; 6]                               │
│  └── equipment: Equipment (planned)                              │
├──────────────────────────────────────────────────────────────────┤
│  MIND (Cognitive State)                                          │
│  ├── needs: Needs                                                │
│  │   ├── hunger: f32 (0.0-1.0)                                   │
│  │   ├── thirst: f32 (0.0-1.0)                                   │
│  │   ├── rest: f32 (0.0-1.0)                                     │
│  │   ├── safety: f32 (0.0-1.0)                                   │
│  │   └── social: f32 (0.0-1.0)                                   │
│  ├── values: SpeciesValues (HumanValues, DwarfValues, etc.)      │
│  ├── thoughts: ThoughtBuffer (capacity: 20)                      │
│  └── task_queue: TaskQueue                                       │
├──────────────────────────────────────────────────────────────────┤
│  RELATIONSHIPS (Social Memory)                                   │
│  ├── known_entities: Vec<SocialMemory> (max ~150)                │
│  ├── dispositions: HashMap<EntityId, Disposition>                │
│  └── expectations: HashMap<EntityId, Vec<Expectation>>           │
└──────────────────────────────────────────────────────────────────┘
```

---

## Needs System

### Universal Needs

All entities share the same need types (though species may weight them differently):

```rust
pub struct Needs {
    pub hunger: f32,      // 0.0 = satiated, 1.0 = starving
    pub thirst: f32,      // 0.0 = hydrated, 1.0 = dehydrated
    pub rest: f32,        // 0.0 = rested, 1.0 = exhausted
    pub safety: f32,      // 0.0 = secure, 1.0 = terrified
    pub social: f32,      // 0.0 = fulfilled, 1.0 = isolated
}
```

### Need Decay

Needs increase (worsen) over time:

```rust
impl Needs {
    pub fn decay(&mut self) {
        const DECAY_RATE: f32 = 0.001;

        self.hunger = (self.hunger + DECAY_RATE * 1.0).min(1.0);
        self.thirst = (self.thirst + DECAY_RATE * 1.2).min(1.0);  // Faster than hunger
        self.rest = (self.rest + DECAY_RATE * 0.8).min(1.0);
        self.safety = (self.safety - DECAY_RATE * 0.5).max(0.0);  // Relaxes over time
        self.social = (self.social + DECAY_RATE * 0.3).min(1.0);
    }
}
```

### Safety Asymmetry

**Critical Implementation Detail**: Safety works differently from other needs:

- Other needs: decay increases them (hunger grows)
- Safety: decays toward 0.0 (relaxation) unless threats present
- Threats spike safety instantly
- High safety need triggers fight/flight responses

```rust
impl Needs {
    pub fn spike_safety(&mut self, threat_level: f32) {
        // Immediate jump, not gradual
        self.safety = (self.safety + threat_level).min(1.0);
    }
}
```

### Need Satisfaction

Actions satisfy needs:

```rust
pub fn satisfy_need(&mut self, need_type: NeedType, amount: f32) {
    match need_type {
        NeedType::Hunger => self.hunger = (self.hunger - amount).max(0.0),
        NeedType::Thirst => self.thirst = (self.thirst - amount).max(0.0),
        NeedType::Rest => self.rest = (self.rest - amount).max(0.0),
        NeedType::Safety => self.safety = (self.safety - amount).max(0.0),
        NeedType::Social => self.social = (self.social - amount).max(0.0),
    }
}
```

---

## Values System

### Species-Specific Values

Values are **type-incompatible** across species. This is a core design constraint.

> **MVP Priority**: Human → Elf → Dwarf. All gameplay tuning targets these three species first. Monster species (Orc, Kobold, etc.) have basic rules but are post-MVP optimization targets.

#### Human Values

```rust
pub struct HumanValues {
    pub honor: f32,       // Social standing, keeping one's word
    pub beauty: f32,      // Aesthetic appreciation
    pub comfort: f32,     // Physical ease and safety
    pub ambition: f32,    // Drive for advancement
    pub loyalty: f32,     // Group attachment
    pub love: f32,        // Individual attachment
    pub justice: f32,     // Fairness and equity
    pub curiosity: f32,   // Desire to explore and learn
    pub safety: f32,      // Self-preservation
    pub piety: f32,       // Spiritual devotion
}
```

#### Dwarf Values (Future)

```rust
pub struct DwarfValues {
    pub craft_truth: f32,    // Honest work, true craftsmanship
    pub stone_debt: f32,     // Obligation to the mountain
    pub clan_weight: f32,    // Family honor and duty
    pub oath_chain: f32,     // Binding promises
    pub deep_memory: f32,    // Ancestral knowledge
    pub grudge_mark: f32,    // Wrongs that must be righted
}
```

#### Elf Values (Future)

```rust
pub struct ElfValues {
    pub pattern_beauty: f32,   // Appreciation of natural patterns
    pub slow_growth: f32,      // Patience, long-term thinking
    pub star_longing: f32,     // Connection to celestial
    pub cycle_wisdom: f32,     // Understanding of cycles
    pub tree_bond: f32,        // Connection to living things
    pub fate_thread: f32,      // Acceptance of destiny
}
```

### Value Derivation

Values derive from the genome → phenotype → personality → values pipeline:

```
Genome (DNA-like encoding)
   │
   ▼ expression
Phenotype (physical traits: strength, intelligence, etc.)
   │
   ▼ development
Personality (temperament traits)
   │
   ▼ cultural formation
Values (species-specific motivations)
```

**Specification**: [08-GENETICS-SYSTEM-SPEC](08-GENETICS-SYSTEM-SPEC.md)

---

## Thought System

### ThoughtBuffer

Entities maintain a buffer of recent thoughts:

```rust
pub struct ThoughtBuffer {
    thoughts: Vec<Thought>,
    capacity: usize,  // Default: 20
}

pub struct Thought {
    pub id: ThoughtId,
    pub content: ThoughtContent,
    pub intensity: f32,           // 0.0-1.0, decays over time
    pub source: ThoughtSource,
    pub created_tick: Tick,
}

pub enum ThoughtSource {
    Perception(PerceptionId),     // Saw something
    Need(NeedType),               // Felt a need
    Memory(MemoryId),             // Recalled something
    Value(ValueType),             // Value-driven thought
}

pub enum ThoughtContent {
    // Perception-based
    NoticeEntity { entity: EntityId, disposition: Disposition },
    NoticeResource { resource_type: ResourceType, position: Vec2 },
    NoticeThreat { source: EntityId, threat_level: f32 },

    // Need-based
    FeelHungry { intensity: f32 },
    FeelTired { intensity: f32 },
    FeelUnsafe { source: Option<EntityId> },
    FeelLonely,

    // Value-based
    ValueConflict { action: ActionId, conflicting_value: ValueType },
    ValueAlignment { action: ActionId, supporting_value: ValueType },
}
```

### Thought Decay

Thoughts fade over time:

```rust
impl ThoughtBuffer {
    pub fn decay(&mut self) {
        const DECAY_RATE: f32 = 0.05;

        for thought in &mut self.thoughts {
            thought.intensity -= DECAY_RATE;
        }

        // Remove faded thoughts
        self.thoughts.retain(|t| t.intensity > 0.0);
    }

    pub fn add(&mut self, thought: Thought) {
        if self.thoughts.len() >= self.capacity {
            // Evict weakest thought
            if let Some(min_idx) = self.thoughts
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.intensity.partial_cmp(&b.intensity).unwrap())
                .map(|(idx, _)| idx)
            {
                self.thoughts.remove(min_idx);
            }
        }
        self.thoughts.push(thought);
    }
}
```

### Memory Conversion

Strong thoughts become long-term memories:

```rust
impl ThoughtBuffer {
    pub fn check_memory_conversion(&mut self, social_memory: &mut SocialMemory) {
        for thought in &self.thoughts {
            // Thoughts about entities can become memories
            if let ThoughtContent::NoticeEntity { entity, disposition } = &thought.content {
                if thought.intensity > 0.8 {
                    social_memory.update_memory(*entity, disposition);
                }
            }
        }
    }
}
```

---

## Task System

### TaskQueue

Entities have a queue of tasks to execute:

```rust
pub struct TaskQueue {
    tasks: VecDeque<Task>,
    max_size: usize,
}

pub struct Task {
    pub id: TaskId,
    pub action: ActionId,
    pub target: Option<TaskTarget>,
    pub progress: f32,           // 0.0-1.0
    pub priority: TaskPriority,
}

pub enum TaskTarget {
    Entity(EntityId),
    Position(Vec2),
    Resource(ResourceType),
    Building(BuildingId),
}

pub enum TaskPriority {
    Critical,    // Life-threatening (flee, defend)
    High,        // Urgent needs (eat when starving)
    Normal,      // Standard tasks
    Low,         // Optional tasks
    Background,  // When nothing else to do
}
```

### Task Execution

```rust
pub fn execute_task(world: &mut World, entity_idx: usize) {
    let task_queue = &mut world.humans.task_queues[entity_idx];

    if let Some(task) = task_queue.front_mut() {
        // Execute action
        let result = execute_action(world, entity_idx, &task.action, &task.target);

        match result {
            ActionResult::InProgress(progress_delta) => {
                task.progress += progress_delta;
                if task.progress >= 1.0 {
                    task_queue.pop_front();
                }
            }
            ActionResult::Complete => {
                // Apply effects
                apply_task_effects(world, entity_idx, &task);
                task_queue.pop_front();
            }
            ActionResult::Failed(reason) => {
                // Task failed, remove it
                task_queue.pop_front();
            }
            ActionResult::Interrupted => {
                // Keep task, will retry next tick
            }
        }
    }
}
```

---

## Cognition Pipeline

### Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                       COGNITION PIPELINE                             │
│                                                                      │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐     │
│   │PERCEPTION│───▶│ THOUGHT  │───▶│ ACTION   │───▶│  TASK    │     │
│   │          │    │GENERATION│    │SELECTION │    │EXECUTION │     │
│   └──────────┘    └──────────┘    └──────────┘    └──────────┘     │
│        │               │               │               │            │
│        ▼               ▼               ▼               ▼            │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐     │
│   │ Spatial  │    │  Entity  │    │  Entity  │    │   ECS    │     │
│   │  Index   │    │ Thoughts │    │   Needs  │    │  World   │     │
│   └──────────┘    └──────────┘    └──────────┘    └──────────┘     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Phase 1: Perception

What entities notice in their environment:

```rust
pub fn run_perception(world: &World, spatial: &SparseHashGrid) -> Vec<(EntityId, Vec<Perception>)> {
    let mut results = Vec::new();

    for idx in world.humans.iter_alive() {
        let entity_id = world.humans.ids[idx];
        let position = world.humans.positions[idx];
        let perception_range = calculate_perception_range(world, idx);

        // Find nearby entities
        let nearby = spatial.query_radius(position, perception_range);
        let perceptions = nearby
            .iter()
            .filter(|&&id| id != entity_id)
            .filter_map(|&id| create_perception(world, entity_id, id))
            .collect();

        results.push((entity_id, perceptions));
    }

    results
}

pub struct Perception {
    pub source: EntityId,
    pub target: EntityId,
    pub perception_type: PerceptionType,
    pub distance: f32,
    pub disposition: Disposition,
}

pub enum PerceptionType {
    SeeEntity,
    HearSound,
    SmellScent,
    SensePresence,  // For special abilities
}
```

### Phase 2: Thought Generation

Convert perceptions into thoughts, filtered by values:

```rust
pub fn generate_thoughts(world: &mut World, entity_id: EntityId, perceptions: &[Perception]) {
    let idx = world.humans.index_of(entity_id).unwrap();
    let values = &world.humans.values[idx];
    let needs = &world.humans.needs[idx];

    for perception in perceptions {
        // Value filtering: What matters to this entity?
        let relevance = calculate_relevance(values, perception);

        if relevance > 0.1 {
            let thought = Thought {
                id: ThoughtId::new(),
                content: perception_to_thought_content(perception),
                intensity: relevance * perception_intensity(perception),
                source: ThoughtSource::Perception(perception.into()),
                created_tick: world.current_tick,
            };

            world.humans.thoughts[idx].add(thought);
        }
    }

    // Also generate need-based thoughts
    generate_need_thoughts(world, idx, needs);
}

fn calculate_relevance(values: &HumanValues, perception: &Perception) -> f32 {
    match perception.perception_type {
        PerceptionType::SeeEntity => {
            match perception.disposition {
                Disposition::Hostile => values.safety * 0.5 + values.honor * 0.3,
                Disposition::Friendly => values.loyalty * 0.4 + values.love * 0.3,
                Disposition::Neutral => values.curiosity * 0.5,
            }
        }
        // ... other perception types
    }
}
```

### Phase 3: Action Selection

Choose action based on thoughts, needs, and values:

```rust
pub fn select_action(world: &World, idx: usize) -> Task {
    let needs = &world.humans.needs[idx];
    let values = &world.humans.values[idx];
    let thoughts = &world.humans.thoughts[idx];

    // Gather available actions
    let available_actions = get_available_actions(world, idx);

    // Score each action
    let scored: Vec<(ActionId, f32)> = available_actions
        .iter()
        .map(|&action| {
            let score = score_action(action, needs, values, thoughts);
            (action, score)
        })
        .collect();

    // Select highest scoring action
    let best_action = scored
        .iter()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(action, _)| *action)
        .unwrap_or(ActionId::Idle);

    Task {
        id: TaskId::new(),
        action: best_action,
        target: get_action_target(world, idx, best_action),
        progress: 0.0,
        priority: TaskPriority::Normal,
    }
}

fn score_action(action: ActionId, needs: &Needs, values: &HumanValues, thoughts: &ThoughtBuffer) -> f32 {
    let mut score = 0.0;

    // Need satisfaction
    for (need_type, satisfaction) in action.satisfies_needs() {
        let need_urgency = needs.get(need_type);
        score += satisfaction * need_urgency * SATISFACTION_MULTIPLIER;
    }

    // Value alignment
    for (value_type, alignment) in action.value_alignment() {
        let value_strength = values.get(value_type);
        score += alignment * value_strength;
    }

    // Thought reinforcement
    for thought in thoughts.iter() {
        if thought.supports_action(action) {
            score += thought.intensity * 0.5;
        }
        if thought.opposes_action(action) {
            score -= thought.intensity * 0.5;
        }
    }

    score
}
```

### The Satisfaction Multiplier

**Critical Implementation Detail**: Actions satisfy needs with a multiplier:

```rust
const SATISFACTION_MULTIPLIER: f32 = 1.0;

// This multiplier is the primary balance lever for behavior emergence
// Higher = needs dominate decisions
// Lower = values have more influence

// The value should be tuned so that:
// - Starving entities prioritize food
// - But comfortable entities follow their values
```

---

## Social Memory

### Structure

```rust
pub struct SocialMemory {
    pub memories: Vec<EntityMemory>,
    pub max_size: usize,  // ~150 (Dunbar limit)
}

pub struct EntityMemory {
    pub entity_id: EntityId,
    pub disposition: Disposition,
    pub last_interaction: Tick,
    pub expectations: Vec<Expectation>,
    pub violations: Vec<Violation>,
}

pub enum Disposition {
    Hostile,
    Unfriendly,
    Neutral,
    Friendly,
    Trusted,
}

pub struct Expectation {
    pub expectation_type: ExpectationType,
    pub strength: f32,
}

pub enum ExpectationType {
    WillNotAttack,
    WillShareFood,
    WillDefend,
    WillHonorAgreement,
}

pub struct Violation {
    pub expectation: ExpectationType,
    pub tick: Tick,
    pub severity: f32,
}
```

### Dunbar Limit

Entities can only maintain meaningful relationships with ~150 others:

```rust
impl SocialMemory {
    pub fn add_memory(&mut self, entity_id: EntityId, disposition: Disposition) {
        if self.memories.len() >= self.max_size {
            // Evict least recent interaction
            if let Some(oldest_idx) = self.memories
                .iter()
                .enumerate()
                .min_by_key(|(_, m)| m.last_interaction)
                .map(|(idx, _)| idx)
            {
                self.memories.remove(oldest_idx);
            }
        }

        self.memories.push(EntityMemory {
            entity_id,
            disposition,
            last_interaction: Tick::now(),
            expectations: Vec::new(),
            violations: Vec::new(),
        });
    }
}
```

### Expectation Violations

When expectations are violated, relationships change:

```rust
impl SocialMemory {
    pub fn record_violation(&mut self, entity_id: EntityId, expectation: ExpectationType, severity: f32) {
        if let Some(memory) = self.memories.iter_mut().find(|m| m.entity_id == entity_id) {
            memory.violations.push(Violation {
                expectation,
                tick: Tick::now(),
                severity,
            });

            // Disposition degrades based on violation severity
            memory.disposition = memory.disposition.degrade(severity);
        }
    }
}
```

**Specification**: [17-SOCIAL-MEMORY-SPEC](17-SOCIAL-MEMORY-SPEC.md)

---

## Emergence Examples

### Example 1: Cowardly Behavior

A "coward" isn't scripted—cowardice emerges:

```
Values: safety = 0.9, honor = 0.2
Needs: safety = 0.7 (threatened)

Perception: hostile entity nearby
  → Thought: NoticeThreat { intensity: 0.8 }

Action Scoring:
  - Flee: satisfies safety (0.9 * 0.7 = 0.63), no value conflict
  - Fight: low safety satisfaction, conflicts with high safety value
  - Hide: moderate safety satisfaction, aligns with comfort

Result: Flee scores highest → entity flees

The entity flees because their values and needs combine that way,
not because of `if coward then flee` code.
```

### Example 2: Heroic Behavior

Heroism also emerges:

```
Values: honor = 0.9, loyalty = 0.8, safety = 0.3
Needs: safety = 0.5 (moderate threat)

Perception: ally in danger
  → Thought: AllyThreatened { intensity: 0.7 }

Action Scoring:
  - Defend: aligns with loyalty (0.8) and honor (0.9)
  - Flee: conflicts with honor, doesn't satisfy loyalty
  - Observe: low relevance

Result: Defend scores highest → entity fights

The entity defends because honor and loyalty outweigh their moderate
safety concern, not because they're tagged "hero".
```

---

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Needs | Complete | 5 needs, decay working |
| ThoughtBuffer | Complete | Decay, eviction working |
| TaskQueue | Complete | Priority ordering working |
| Perception | Complete | Distance-based, disposition-aware |
| Action Selection | Complete | Needs/values/thoughts integration |
| Social Memory | Partial | Structure exists, violations need work |
| Values | Partial | HumanValues only, other species pending |

---

## Related Specifications

| Document | Relationship |
|----------|--------------|
| [01-GAME-DESIGN-DOCUMENT](01-GAME-DESIGN-DOCUMENT.md) | Design pillars |
| [08-GENETICS-SYSTEM-SPEC](08-GENETICS-SYSTEM-SPEC.md) | Value derivation |
| [11-ACTION-CATALOG](11-ACTION-CATALOG.md) | Available actions |
| [17-SOCIAL-MEMORY-SPEC](17-SOCIAL-MEMORY-SPEC.md) | Relationship details |
| [19-HIERARCHICAL-CHUNKING-SPEC](19-HIERARCHICAL-CHUNKING-SPEC.md) | Skill system |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification created |
