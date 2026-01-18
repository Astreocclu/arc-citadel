# 17-SOCIAL-MEMORY-SPEC
> Entity memory: experiences shape perception, relationships, and decision-making

## Overview

Entities remember their experiences, and those memories shape future behavior. This specification defines how memories are formed, stored, retrieved, and influence entity decisions. Memory is not perfect recall—it's filtered through values, biased by emotion, and fades over time.

---

## Memory Architecture

### Memory Types

```rust
/// A single memory instance
#[derive(Debug, Clone)]
pub struct Memory {
    pub id: MemoryId,
    pub memory_type: MemoryType,
    pub timestamp: SimTime,

    // Content
    pub subject: Option<EntityId>,     // who/what is this about
    pub location: Option<Vec3>,        // where did this happen
    pub action: Option<ActionId>,      // what action occurred
    pub outcome: MemoryOutcome,        // how did it turn out

    // Emotional coloring
    pub emotional_valence: f32,        // -1.0 (negative) to 1.0 (positive)
    pub emotional_intensity: f32,      // 0.0 to 1.0

    // Strength and decay
    pub initial_strength: f32,
    pub current_strength: f32,
    pub recall_count: u32,             // how many times recalled
    pub last_recall: SimTime,

    // Tags for retrieval
    pub tags: Vec<MemoryTag>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryType {
    // Social memories
    Interaction,          // I interacted with someone
    Observation,          // I saw someone do something
    Relationship,         // My feelings about someone changed
    Betrayal,             // Someone broke trust
    Kindness,             // Someone helped me

    // Experience memories
    Achievement,          // I accomplished something
    Failure,              // I failed at something
    Discovery,            // I found/learned something
    Danger,               // I was in danger
    Safety,               // I found safety

    // Trauma/strong memories
    WoundReceived,        // I was wounded
    WoundInflicted,       // I wounded someone
    Death,                // Someone died
    Loss,                 // I lost something important

    // Skill memories
    Practice,             // I practiced a skill
    Lesson,               // I learned from experience
    Mastery,              // I achieved mastery moment
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryOutcome {
    Positive,
    Negative,
    Neutral,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryTag {
    // Entity tags
    Family,
    Friend,
    Enemy,
    Stranger,
    Authority,

    // Emotion tags
    Fear,
    Joy,
    Anger,
    Sadness,
    Trust,
    Disgust,

    // Context tags
    Combat,
    Work,
    Social,
    Survival,
    Exploration,

    // Outcome tags
    Success,
    Failure,
    Trauma,
    Pride,
}
```

### Memory Storage

```rust
/// Entity's memory system
#[derive(Debug)]
pub struct MemoryBank {
    pub entity_id: EntityId,

    // Storage
    pub memories: Vec<Memory>,
    pub max_memories: usize,           // capacity limit

    // Indices for fast retrieval
    pub by_subject: HashMap<EntityId, Vec<MemoryId>>,
    pub by_type: HashMap<MemoryType, Vec<MemoryId>>,
    pub by_tag: HashMap<MemoryTag, Vec<MemoryId>>,

    // Relationship summaries (cached from memories)
    pub relationships: HashMap<EntityId, RelationshipSummary>,

    // Memory configuration
    pub config: MemoryConfig,
}

#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub base_decay_rate: f32,          // how fast memories fade
    pub emotional_decay_modifier: f32, // emotional memories fade slower
    pub recall_strengthens: f32,       // how much recall strengthens memory
    pub consolidation_threshold: f32,  // strength needed to become long-term
}

impl MemoryBank {
    /// Create a new memory from an experience
    pub fn form_memory(
        &mut self,
        experience: Experience,
        entity_values: &HumanValues,
        current_time: SimTime,
    ) -> MemoryId {
        // Calculate emotional response based on values
        let (valence, intensity) = self.calculate_emotional_response(
            &experience,
            entity_values,
        );

        // Initial strength based on emotional intensity
        let initial_strength = 0.3 + intensity * 0.7;

        // Determine memory type
        let memory_type = self.classify_experience(&experience);

        // Generate tags
        let tags = self.generate_tags(&experience, valence);

        let memory = Memory {
            id: MemoryId::new(),
            memory_type,
            timestamp: current_time,
            subject: experience.subject,
            location: experience.location,
            action: experience.action,
            outcome: if valence > 0.2 {
                MemoryOutcome::Positive
            } else if valence < -0.2 {
                MemoryOutcome::Negative
            } else {
                MemoryOutcome::Neutral
            },
            emotional_valence: valence,
            emotional_intensity: intensity,
            initial_strength,
            current_strength: initial_strength,
            recall_count: 0,
            last_recall: current_time,
            tags,
        };

        let memory_id = memory.id;

        // Update indices
        if let Some(subject) = memory.subject {
            self.by_subject.entry(subject).or_default().push(memory_id);
        }
        self.by_type.entry(memory_type).or_default().push(memory_id);
        for tag in &memory.tags {
            self.by_tag.entry(*tag).or_default().push(memory_id);
        }

        // Update relationship summary if relevant
        if let Some(subject) = experience.subject {
            self.update_relationship_summary(subject, &memory);
        }

        self.memories.push(memory);

        // Enforce capacity limit
        self.consolidate_if_needed();

        memory_id
    }

    /// Calculate emotional response based on entity values
    fn calculate_emotional_response(
        &self,
        experience: &Experience,
        values: &HumanValues,
    ) -> (f32, f32) {
        let mut valence = 0.0;
        let mut intensity = 0.0;

        match &experience.event_type {
            ExperienceType::Helped => {
                // Being helped resonates with safety, loyalty
                valence += 0.5 + values.loyalty * 0.3;
                intensity += 0.3 + values.safety * 0.2;
            }
            ExperienceType::Harmed => {
                // Being harmed is negative, intensity from justice/safety
                valence -= 0.7;
                intensity += 0.5 + values.safety * 0.3 + values.justice * 0.2;
            }
            ExperienceType::Betrayed => {
                // Betrayal hits loyalty and honor hard
                valence -= 0.3 + values.loyalty * 0.5 + values.honor * 0.3;
                intensity += 0.6 + values.loyalty * 0.3;
            }
            ExperienceType::Honored => {
                // Being honored resonates with honor and ambition
                valence += 0.4 + values.honor * 0.4 + values.ambition * 0.2;
                intensity += 0.4 + values.honor * 0.3;
            }
            ExperienceType::Discovered => {
                // Discovery resonates with curiosity
                valence += 0.3 + values.curiosity * 0.5;
                intensity += 0.2 + values.curiosity * 0.4;
            }
            ExperienceType::LossWitnessed => {
                // Witnessing death/loss
                valence -= 0.4;
                intensity += 0.4 + values.love * 0.3;
            }
            ExperienceType::BeautyExperienced => {
                // Beauty affects those who value it
                valence += 0.2 + values.beauty * 0.5;
                intensity += values.beauty * 0.4;
            }
            _ => {
                valence = 0.0;
                intensity = 0.2;
            }
        }

        (valence.clamp(-1.0, 1.0), intensity.clamp(0.0, 1.0))
    }

    /// Memory decay over time
    pub fn process_decay(&mut self, dt_days: f32) {
        for memory in &mut self.memories {
            // Emotional memories decay slower
            let decay_modifier = 1.0 - memory.emotional_intensity
                * self.config.emotional_decay_modifier;

            // Frequently recalled memories decay slower
            let recall_modifier = 1.0 / (1.0 + memory.recall_count as f32 * 0.1);

            let decay = self.config.base_decay_rate
                * decay_modifier
                * recall_modifier
                * dt_days;

            memory.current_strength -= decay;
            memory.current_strength = memory.current_strength.max(0.0);
        }

        // Remove completely faded memories
        self.memories.retain(|m| m.current_strength > 0.01);

        // Rebuild indices after removal
        self.rebuild_indices();
    }

    /// Recall a memory (strengthens it)
    pub fn recall(&mut self, memory_id: MemoryId, current_time: SimTime) -> Option<&Memory> {
        if let Some(memory) = self.memories.iter_mut().find(|m| m.id == memory_id) {
            memory.recall_count += 1;
            memory.last_recall = current_time;

            // Strengthen memory through recall
            memory.current_strength += self.config.recall_strengthens;
            memory.current_strength = memory.current_strength.min(1.0);

            Some(memory)
        } else {
            None
        }
    }

    /// Find relevant memories for a situation
    pub fn retrieve_relevant(
        &self,
        context: &RetrievalContext,
        limit: usize,
    ) -> Vec<&Memory> {
        let mut candidates: Vec<(&Memory, f32)> = Vec::new();

        for memory in &self.memories {
            let relevance = self.calculate_relevance(memory, context);
            if relevance > 0.1 {
                candidates.push((memory, relevance));
            }
        }

        // Sort by relevance × strength
        candidates.sort_by(|a, b| {
            let score_a = a.1 * a.0.current_strength;
            let score_b = b.1 * b.0.current_strength;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        candidates.into_iter()
            .take(limit)
            .map(|(m, _)| m)
            .collect()
    }

    fn calculate_relevance(&self, memory: &Memory, context: &RetrievalContext) -> f32 {
        let mut relevance = 0.0;

        // Subject match
        if let Some(subject) = context.subject {
            if memory.subject == Some(subject) {
                relevance += 0.5;
            }
        }

        // Tag match
        for tag in &context.required_tags {
            if memory.tags.contains(tag) {
                relevance += 0.2;
            }
        }

        // Type match
        if let Some(memory_type) = context.memory_type {
            if memory.memory_type == memory_type {
                relevance += 0.3;
            }
        }

        // Recency bonus
        if let Some(max_age) = context.max_age_days {
            let age_days = (context.current_time - memory.timestamp).as_days();
            if age_days <= max_age {
                relevance += 0.1 * (1.0 - age_days / max_age);
            }
        }

        relevance
    }

    fn consolidate_if_needed(&mut self) {
        if self.memories.len() > self.max_memories {
            // Sort by importance (strength × emotional intensity)
            self.memories.sort_by(|a, b| {
                let importance_a = a.current_strength * (0.5 + a.emotional_intensity * 0.5);
                let importance_b = b.current_strength * (0.5 + b.emotional_intensity * 0.5);
                importance_b.partial_cmp(&importance_a).unwrap_or(std::cmp::Ordering::Equal)
            });

            // Keep only the most important memories
            self.memories.truncate(self.max_memories);
            self.rebuild_indices();
        }
    }

    fn rebuild_indices(&mut self) {
        self.by_subject.clear();
        self.by_type.clear();
        self.by_tag.clear();

        for memory in &self.memories {
            if let Some(subject) = memory.subject {
                self.by_subject.entry(subject).or_default().push(memory.id);
            }
            self.by_type.entry(memory.memory_type).or_default().push(memory.id);
            for tag in &memory.tags {
                self.by_tag.entry(*tag).or_default().push(memory.id);
            }
        }
    }

    // Helper methods...
    fn classify_experience(&self, experience: &Experience) -> MemoryType {
        // Implementation based on experience type
        MemoryType::Interaction
    }

    fn generate_tags(&self, experience: &Experience, valence: f32) -> Vec<MemoryTag> {
        // Implementation...
        vec![]
    }

    fn update_relationship_summary(&mut self, subject: EntityId, memory: &Memory) {
        // Implementation in relationship section below
    }
}

/// Context for memory retrieval
#[derive(Debug)]
pub struct RetrievalContext {
    pub current_time: SimTime,
    pub subject: Option<EntityId>,
    pub memory_type: Option<MemoryType>,
    pub required_tags: Vec<MemoryTag>,
    pub max_age_days: Option<f32>,
}
```

---

## Relationship System

### Relationship Tracking

```rust
/// Summarized relationship with another entity
#[derive(Debug, Clone)]
pub struct RelationshipSummary {
    pub subject: EntityId,

    // Core metrics (derived from memories)
    pub trust: f32,                    // -1.0 to 1.0
    pub affection: f32,                // -1.0 to 1.0
    pub respect: f32,                  // -1.0 to 1.0
    pub familiarity: f32,              // 0.0 to 1.0

    // Interaction history
    pub total_interactions: u32,
    pub positive_interactions: u32,
    pub negative_interactions: u32,
    pub last_interaction: SimTime,

    // Relationship status
    pub relationship_type: RelationshipType,
    pub debt: f32,                     // positive = they owe me
    pub shared_experiences: u32,

    // Memory references
    pub strongest_positive_memory: Option<MemoryId>,
    pub strongest_negative_memory: Option<MemoryId>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RelationshipType {
    Stranger,
    Acquaintance,
    Colleague,
    Friend,
    CloseFriend,
    Family,
    Rival,
    Enemy,
    Nemesis,
}

impl RelationshipSummary {
    /// Update relationship based on new memory
    pub fn update_from_memory(&mut self, memory: &Memory) {
        self.total_interactions += 1;
        self.last_interaction = memory.timestamp;

        // Update interaction counts
        if memory.emotional_valence > 0.2 {
            self.positive_interactions += 1;
        } else if memory.emotional_valence < -0.2 {
            self.negative_interactions += 1;
        }

        // Update core metrics
        match memory.memory_type {
            MemoryType::Kindness => {
                self.trust += 0.1 * memory.emotional_intensity;
                self.affection += 0.15 * memory.emotional_intensity;
            }
            MemoryType::Betrayal => {
                self.trust -= 0.4 * memory.emotional_intensity;
                self.affection -= 0.2 * memory.emotional_intensity;
            }
            MemoryType::Interaction => {
                self.familiarity += 0.05;
                // Small drift toward valence
                self.affection += memory.emotional_valence * 0.02;
            }
            MemoryType::Achievement => {
                if memory.subject.is_some() {
                    self.respect += 0.1 * memory.emotional_intensity;
                }
            }
            _ => {}
        }

        // Clamp values
        self.trust = self.trust.clamp(-1.0, 1.0);
        self.affection = self.affection.clamp(-1.0, 1.0);
        self.respect = self.respect.clamp(-1.0, 1.0);
        self.familiarity = self.familiarity.clamp(0.0, 1.0);

        // Track strongest memories
        let memory_strength = memory.current_strength * memory.emotional_intensity;
        if memory.emotional_valence > 0.0 {
            if self.strongest_positive_memory.is_none() {
                self.strongest_positive_memory = Some(memory.id);
            }
        } else if memory.emotional_valence < 0.0 {
            if self.strongest_negative_memory.is_none() {
                self.strongest_negative_memory = Some(memory.id);
            }
        }

        // Update relationship type
        self.update_relationship_type();
    }

    fn update_relationship_type(&mut self) {
        let positive_ratio = if self.total_interactions > 0 {
            self.positive_interactions as f32 / self.total_interactions as f32
        } else {
            0.5
        };

        self.relationship_type = match (self.familiarity, self.affection, self.trust) {
            // Negative relationships
            (_, aff, trust) if aff < -0.6 && trust < -0.6 => RelationshipType::Nemesis,
            (_, aff, trust) if aff < -0.3 || trust < -0.3 => RelationshipType::Enemy,
            (fam, aff, _) if fam > 0.3 && aff < -0.1 => RelationshipType::Rival,

            // Positive relationships
            (fam, aff, trust) if fam > 0.8 && aff > 0.6 && trust > 0.6 => {
                RelationshipType::CloseFriend
            }
            (fam, aff, trust) if fam > 0.5 && aff > 0.3 && trust > 0.3 => {
                RelationshipType::Friend
            }
            (fam, _, _) if fam > 0.3 => RelationshipType::Colleague,
            (fam, _, _) if fam > 0.1 => RelationshipType::Acquaintance,

            // Default
            _ => RelationshipType::Stranger,
        };
    }

    /// Calculate willingness to help this entity
    pub fn willingness_to_help(&self) -> f32 {
        let base = (self.affection + self.trust) / 2.0;

        // Debt affects willingness
        let debt_factor = if self.debt > 0.0 {
            -0.1 * self.debt.min(1.0)  // They owe me, less willing
        } else {
            0.1 * (-self.debt).min(1.0) // I owe them, more willing
        };

        (base + debt_factor).clamp(-1.0, 1.0)
    }

    /// Calculate threat assessment
    pub fn perceived_threat(&self) -> f32 {
        if self.trust < 0.0 {
            (-self.trust * 0.5) + (self.negative_interactions as f32 * 0.05).min(0.3)
        } else {
            0.0
        }
    }
}
```

### Relationship Influence on Decisions

```rust
/// How relationships modify action selection
pub fn relationship_action_modifier(
    action: &ActionId,
    target: EntityId,
    memory_bank: &MemoryBank,
) -> f32 {
    let relationship = memory_bank.relationships.get(&target);

    match action {
        ActionId::Help | ActionId::Share | ActionId::Teach => {
            // Helping actions boosted by positive relationship
            relationship
                .map(|r| r.willingness_to_help())
                .unwrap_or(0.0)
        }

        ActionId::Attack | ActionId::Steal | ActionId::Deceive => {
            // Aggressive actions require negative relationship or no relationship
            relationship
                .map(|r| -r.affection.max(0.0) - r.trust.max(0.0))
                .unwrap_or(0.1)  // Easier against strangers
        }

        ActionId::Trade | ActionId::Negotiate => {
            // Trade benefits from trust
            relationship
                .map(|r| r.trust * 0.5)
                .unwrap_or(0.0)
        }

        ActionId::Flee | ActionId::Hide => {
            // Fleeing boosted by fear/negative experiences
            relationship
                .map(|r| r.perceived_threat())
                .unwrap_or(0.0)
        }

        _ => 0.0,
    }
}
```

---

## Memory Influence on Perception

### Value-Filtered Perception

```rust
/// Memories bias what entities notice
pub fn memory_perception_filter(
    perception: &Perception,
    memory_bank: &MemoryBank,
) -> f32 {
    // Base salience from perception
    let mut salience = perception.base_salience;

    // Check if we have memories about this subject
    if let Some(subject) = perception.subject {
        if let Some(relationship) = memory_bank.relationships.get(&subject) {
            // Known entities are more salient
            salience += relationship.familiarity * 0.3;

            // Threats are very salient
            salience += relationship.perceived_threat() * 0.5;

            // Strong emotions increase salience
            let emotional_significance =
                relationship.affection.abs() + relationship.trust.abs();
            salience += emotional_significance * 0.2;
        }

        // Check for traumatic memories
        let trauma_memories = memory_bank.retrieve_relevant(
            &RetrievalContext {
                current_time: perception.timestamp,
                subject: Some(subject),
                memory_type: Some(MemoryType::WoundReceived),
                required_tags: vec![MemoryTag::Trauma],
                max_age_days: None,
            },
            3,
        );

        if !trauma_memories.is_empty() {
            // Traumatic memories make subject hyper-salient
            salience += 0.5;
        }
    }

    salience.clamp(0.0, 1.0)
}
```

### Expectation from Memory

```rust
/// Predict behavior based on past interactions
pub fn predict_behavior(
    subject: EntityId,
    memory_bank: &MemoryBank,
    context: &SituationContext,
) -> BehaviorPrediction {
    let relationship = memory_bank.relationships.get(&subject);

    // Retrieve relevant memories
    let memories = memory_bank.retrieve_relevant(
        &RetrievalContext {
            current_time: context.current_time,
            subject: Some(subject),
            memory_type: None,
            required_tags: context_to_tags(context),
            max_age_days: Some(365.0),
        },
        10,
    );

    if memories.is_empty() {
        return BehaviorPrediction::Unknown;
    }

    // Analyze patterns
    let mut hostile_count = 0;
    let mut friendly_count = 0;
    let mut neutral_count = 0;

    for memory in memories {
        match memory.outcome {
            MemoryOutcome::Positive => friendly_count += 1,
            MemoryOutcome::Negative => hostile_count += 1,
            _ => neutral_count += 1,
        }
    }

    let total = (hostile_count + friendly_count + neutral_count) as f32;

    if hostile_count as f32 / total > 0.6 {
        BehaviorPrediction::LikelyHostile {
            confidence: hostile_count as f32 / total,
        }
    } else if friendly_count as f32 / total > 0.6 {
        BehaviorPrediction::LikelyFriendly {
            confidence: friendly_count as f32 / total,
        }
    } else {
        BehaviorPrediction::Unpredictable
    }
}

#[derive(Debug)]
pub enum BehaviorPrediction {
    Unknown,
    LikelyHostile { confidence: f32 },
    LikelyFriendly { confidence: f32 },
    Unpredictable,
}

fn context_to_tags(context: &SituationContext) -> Vec<MemoryTag> {
    let mut tags = Vec::new();

    if context.in_combat {
        tags.push(MemoryTag::Combat);
    }
    if context.is_social {
        tags.push(MemoryTag::Social);
    }
    if context.is_work {
        tags.push(MemoryTag::Work);
    }

    tags
}
```

---

## Group Memory

### Collective Memory

```rust
/// Shared memories within a group
#[derive(Debug)]
pub struct GroupMemory {
    pub group_id: GroupId,

    // Collective memories
    pub shared_events: Vec<GroupEvent>,
    pub group_reputation: HashMap<EntityId, f32>,
    pub group_knowledge: HashMap<TopicId, f32>,

    // Oral tradition
    pub stories: Vec<Story>,
    pub heroes: Vec<EntityId>,
    pub villains: Vec<EntityId>,
}

#[derive(Debug, Clone)]
pub struct GroupEvent {
    pub id: GroupEventId,
    pub event_type: GroupEventType,
    pub timestamp: SimTime,
    pub participants: Vec<EntityId>,
    pub impact: f32,                   // how significant to group
    pub memory_strength: f32,          // how well remembered
}

#[derive(Debug, Clone, Copy)]
pub enum GroupEventType {
    // Positive events
    Victory,
    Founding,
    Alliance,
    Discovery,
    Celebration,

    // Negative events
    Defeat,
    Betrayal,
    Disaster,
    Atrocity,
    Schism,

    // Neutral events
    Leadership,
    Migration,
    Trade,
}

#[derive(Debug, Clone)]
pub struct Story {
    pub id: StoryId,
    pub title: String,
    pub events: Vec<GroupEventId>,
    pub moral: Option<StoryMoral>,
    pub popularity: f32,               // how often told
}

#[derive(Debug, Clone, Copy)]
pub enum StoryMoral {
    Courage,
    Loyalty,
    Caution,
    Justice,
    Mercy,
    Sacrifice,
}

impl GroupMemory {
    /// Add a new group event
    pub fn record_event(
        &mut self,
        event_type: GroupEventType,
        participants: Vec<EntityId>,
        impact: f32,
        timestamp: SimTime,
    ) {
        let event = GroupEvent {
            id: GroupEventId::new(),
            event_type,
            timestamp,
            participants: participants.clone(),
            impact,
            memory_strength: impact,
        };

        // Update heroes/villains based on event
        match event_type {
            GroupEventType::Victory | GroupEventType::Discovery => {
                for participant in &participants {
                    if impact > 0.5 && !self.heroes.contains(participant) {
                        self.heroes.push(*participant);
                    }
                }
            }
            GroupEventType::Betrayal => {
                for participant in &participants {
                    if impact > 0.5 && !self.villains.contains(participant) {
                        self.villains.push(*participant);
                    }
                }
            }
            _ => {}
        }

        self.shared_events.push(event);
    }

    /// Decay group memories over time
    pub fn process_decay(&mut self, dt_days: f32) {
        let decay_rate = 0.001; // Groups forget slowly

        for event in &mut self.shared_events {
            event.memory_strength -= decay_rate * dt_days;
        }

        // Remove forgotten events
        self.shared_events.retain(|e| e.memory_strength > 0.1);

        // Stories keep events alive
        for story in &self.stories {
            for event_id in &story.events {
                if let Some(event) = self.shared_events.iter_mut()
                    .find(|e| e.id == *event_id)
                {
                    // Stories preserve memory
                    event.memory_strength = event.memory_strength.max(0.3);
                }
            }
        }
    }

    /// Check if entity is known to group
    pub fn knows_entity(&self, entity: EntityId) -> bool {
        self.group_reputation.contains_key(&entity)
            || self.heroes.contains(&entity)
            || self.villains.contains(&entity)
    }

    /// Get group's opinion of an entity
    pub fn opinion_of(&self, entity: EntityId) -> f32 {
        if self.villains.contains(&entity) {
            return -0.8;
        }
        if self.heroes.contains(&entity) {
            return 0.8;
        }
        *self.group_reputation.get(&entity).unwrap_or(&0.0)
    }
}
```

---

## Memory and Learning

### Skill Acquisition from Memory

```rust
/// Memory-based skill improvement
pub fn process_practice_memory(
    memory: &Memory,
    skill_level: &mut u8,
    learning_rate: f32,
) -> SkillProgress {
    if memory.memory_type != MemoryType::Practice {
        return SkillProgress::NotApplicable;
    }

    // Learning is more effective from failures
    let outcome_modifier = match memory.outcome {
        MemoryOutcome::Negative => 1.5, // We learn more from failure
        MemoryOutcome::Positive => 1.0,
        MemoryOutcome::Mixed => 1.2,
        MemoryOutcome::Neutral => 0.8,
    };

    // Emotional intensity affects learning
    let emotional_modifier = 1.0 + memory.emotional_intensity * 0.5;

    let progress = learning_rate * outcome_modifier * emotional_modifier;

    // Diminishing returns at higher skill levels
    let level_modifier = 1.0 / (*skill_level as f32 + 1.0);
    let actual_progress = progress * level_modifier;

    if actual_progress > 0.1 {
        SkillProgress::Improved { amount: actual_progress }
    } else {
        SkillProgress::Incremental { amount: actual_progress }
    }
}

#[derive(Debug)]
pub enum SkillProgress {
    NotApplicable,
    Improved { amount: f32 },
    Incremental { amount: f32 },
}
```

### Lesson Memories

```rust
/// Create a lesson memory from experience
pub fn create_lesson_memory(
    experience: &Experience,
    outcome: MemoryOutcome,
    current_time: SimTime,
) -> Memory {
    let lesson_type = match (&experience.event_type, outcome) {
        (_, MemoryOutcome::Negative) => LessonType::AvoidThis,
        (_, MemoryOutcome::Positive) => LessonType::RepeatThis,
        _ => LessonType::Observation,
    };

    Memory {
        id: MemoryId::new(),
        memory_type: MemoryType::Lesson,
        timestamp: current_time,
        subject: experience.subject,
        location: experience.location,
        action: experience.action,
        outcome,
        emotional_valence: if outcome == MemoryOutcome::Positive { 0.3 } else { -0.3 },
        emotional_intensity: 0.5,
        initial_strength: 0.6,
        current_strength: 0.6,
        recall_count: 0,
        last_recall: current_time,
        tags: vec![
            match lesson_type {
                LessonType::AvoidThis => MemoryTag::Fear,
                LessonType::RepeatThis => MemoryTag::Success,
                LessonType::Observation => MemoryTag::Social,
            }
        ],
    }
}

#[derive(Debug, Clone, Copy)]
enum LessonType {
    AvoidThis,
    RepeatThis,
    Observation,
}
```

---

## Integration with Thought System

### Memory-Triggered Thoughts

```rust
/// Memories can spontaneously trigger thoughts
pub fn check_memory_triggers(
    memory_bank: &MemoryBank,
    current_context: &PerceptionContext,
    thought_buffer: &mut ThoughtBuffer,
) {
    // Check for context-triggered memories
    for perception in &current_context.perceptions {
        if let Some(subject) = perception.subject {
            // Strong memories about this subject can trigger thoughts
            let memories = memory_bank.retrieve_relevant(
                &RetrievalContext {
                    current_time: current_context.current_time,
                    subject: Some(subject),
                    memory_type: None,
                    required_tags: vec![],
                    max_age_days: None,
                },
                3,
            );

            for memory in memories {
                if memory.current_strength > 0.5 && memory.emotional_intensity > 0.6 {
                    // Strong emotional memory triggers thought
                    let thought = create_memory_thought(memory, perception);
                    thought_buffer.add(thought);
                }
            }
        }
    }
}

fn create_memory_thought(memory: &Memory, trigger: &Perception) -> Thought {
    let thought_type = if memory.emotional_valence > 0.0 {
        ThoughtType::Memory { positive: true }
    } else {
        ThoughtType::Memory { positive: false }
    };

    Thought {
        id: ThoughtId::new(),
        thought_type,
        source: ThoughtSource::Memory(memory.id),
        intensity: memory.emotional_intensity * memory.current_strength,
        valence: memory.emotional_valence,
        subject: memory.subject,
        // ... other fields
    }
}
```

---

## Related Specifications

| Document | Relationship |
|----------|--------------|
| [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md) | Entity perception and cognition |
| [06-BALANCE-COMPOSITION-SPEC](06-BALANCE-COMPOSITION-SPEC.md) | Values composition |
| [18-SOCIAL-PRESSURE-MORALE-SPEC](18-SOCIAL-PRESSURE-MORALE-SPEC.md) | Group dynamics |
| [19-HIERARCHICAL-CHUNKING-SPEC](19-HIERARCHICAL-CHUNKING-SPEC.md) | Skill memory |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification created |
