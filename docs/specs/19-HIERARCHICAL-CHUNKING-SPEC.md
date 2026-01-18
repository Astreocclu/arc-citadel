# 19-HIERARCHICAL-CHUNKING-SPEC
> Skill mastery through cognitive chunking: experts don't think faster, they think bigger

## Overview

Skill in Arc Citadel is modeled after cognitive science research on expertise. Experts don't process information faster than novices—they recognize larger patterns and execute longer sequences as single "chunks." This specification defines how skills are acquired, represented, and affect gameplay through hierarchical chunking.

---

## Core Concept

**Mastery is the compression of many steps into few.**

```rust
// ✅ CORRECT: Skill as chunking
fn sword_attack_steps(skill_level: u8) -> Vec<ActionChunk> {
    match skill_level {
        0 => vec![  // Novice: 6 conscious steps
            "raise_arm", "grip_tight", "look_at_target",
            "swing_forward", "follow_through", "recover_stance"
        ],
        5 => vec![  // Competent: 3 chunked sequences
            "prepare_strike", "execute_cut", "recover"
        ],
        10 => vec![ // Master: single automated action
            "strike"
        ],
    }
}

// ❌ FORBIDDEN: Skill as percentage bonus
fn sword_damage(base: f32, skill: u8) -> f32 {
    base * (1.0 + skill as f32 * 0.1)  // NEVER DO THIS
}
```

The master's attack isn't "stronger"—it's faster because fewer cognitive steps means faster execution.

---

## Chunking Theory

### Cognitive Model

```rust
/// A single cognitive chunk - the basic unit of skilled action
#[derive(Debug, Clone)]
pub struct ActionChunk {
    pub id: ChunkId,
    pub name: String,

    // Complexity
    pub sub_steps: Vec<ActionStep>,    // What this chunk compresses
    pub cognitive_load: f32,            // How much attention required

    // Execution
    pub base_duration: f32,             // Seconds to execute
    pub variability: f32,               // How consistent execution is

    // Requirements
    pub prerequisite_chunks: Vec<ChunkId>,
    pub minimum_skill: u8,
}

/// An atomic action step (cannot be further decomposed)
#[derive(Debug, Clone)]
pub struct ActionStep {
    pub id: StepId,
    pub name: String,
    pub duration: f32,                  // Seconds
    pub requires_attention: bool,
    pub physical_component: Option<PhysicalAction>,
}

/// How chunks combine into sequences
#[derive(Debug)]
pub struct ChunkSequence {
    pub chunks: Vec<ActionChunk>,
    pub total_cognitive_load: f32,
    pub total_duration: f32,
    pub transition_overhead: f32,       // Time between chunks
}

impl ChunkSequence {
    /// Calculate total execution time
    pub fn execution_time(&self, skill_level: u8) -> f32 {
        let chunk_time: f32 = self.chunks.iter()
            .map(|c| c.base_duration)
            .sum();

        // Higher skill = smoother transitions
        let transition_factor = 1.0 - (skill_level as f32 / 10.0) * 0.5;
        let transitions = (self.chunks.len() - 1) as f32
            * self.transition_overhead
            * transition_factor;

        chunk_time + transitions
    }

    /// Calculate cognitive load
    pub fn cognitive_load(&self) -> f32 {
        // Parallel chunks share load, serial chunks add
        self.chunks.iter()
            .map(|c| c.cognitive_load)
            .sum()
    }
}
```

### Skill Levels and Chunking

```rust
/// Skill level determines chunk availability
#[derive(Debug)]
pub struct SkillMastery {
    pub skill_id: SkillId,
    pub level: u8,                      // 0-10
    pub experience: f32,                // Progress to next level

    // Acquired chunks
    pub known_chunks: Vec<ChunkId>,
    pub automating_chunks: Vec<ChunkId>, // Currently being automated

    // Practice history
    pub total_practice_hours: f32,
    pub recent_practice: Vec<PracticeSession>,
}

impl SkillMastery {
    /// Get chunks available at current skill level
    pub fn available_chunks(&self, chunk_library: &ChunkLibrary) -> Vec<&ActionChunk> {
        chunk_library.chunks_for_skill(self.skill_id)
            .filter(|c| c.minimum_skill <= self.level)
            .collect()
    }

    /// Get most efficient chunk sequence for an action
    pub fn optimal_sequence(
        &self,
        action: ActionId,
        chunk_library: &ChunkLibrary,
    ) -> ChunkSequence {
        let available = self.available_chunks(chunk_library);
        let required_steps = chunk_library.steps_for_action(action);

        // Find chunks that cover required steps
        let mut sequence = Vec::new();
        let mut covered_steps: HashSet<StepId> = HashSet::new();

        // Prefer larger chunks (more efficient)
        let mut sorted_chunks: Vec<_> = available.iter()
            .filter(|c| self.known_chunks.contains(&c.id))
            .collect();
        sorted_chunks.sort_by(|a, b|
            b.sub_steps.len().cmp(&a.sub_steps.len())
        );

        for chunk in sorted_chunks {
            let chunk_steps: HashSet<_> = chunk.sub_steps.iter()
                .map(|s| s.id)
                .collect();

            // Check if this chunk covers any uncovered steps
            if chunk_steps.iter().any(|s| !covered_steps.contains(s)) {
                sequence.push((*chunk).clone());
                covered_steps.extend(chunk_steps);
            }

            if required_steps.iter().all(|s| covered_steps.contains(&s.id)) {
                break;
            }
        }

        ChunkSequence {
            total_cognitive_load: sequence.iter().map(|c| c.cognitive_load).sum(),
            total_duration: sequence.iter().map(|c| c.base_duration).sum(),
            transition_overhead: 0.1, // Base transition time
            chunks: sequence,
        }
    }

    /// Process practice for skill improvement
    pub fn practice(
        &mut self,
        action: ActionId,
        outcome: PracticeOutcome,
        duration_hours: f32,
    ) {
        self.total_practice_hours += duration_hours;

        // Experience gain based on outcome
        let base_exp = duration_hours * 10.0;
        let outcome_multiplier = match outcome {
            PracticeOutcome::Success => 1.0,
            PracticeOutcome::Failure => 1.5,  // Learn more from failure
            PracticeOutcome::Challenge => 2.0, // Optimal difficulty
            PracticeOutcome::TooEasy => 0.3,
            PracticeOutcome::TooHard => 0.5,
        };

        self.experience += base_exp * outcome_multiplier;

        // Check for level up
        let required_exp = self.experience_for_next_level();
        if self.experience >= required_exp {
            self.level_up();
        }
    }

    fn experience_for_next_level(&self) -> f32 {
        // Exponential curve: each level requires more practice
        100.0 * (1.5_f32).powi(self.level as i32)
    }

    fn level_up(&mut self) {
        self.level += 1;
        self.experience = 0.0;

        // New chunks become available
        // Existing chunks may begin automation
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PracticeOutcome {
    Success,
    Failure,
    Challenge,  // Difficult but managed
    TooEasy,
    TooHard,
}
```

---

## Action Decomposition

### Sword Combat Example

```rust
/// Complete decomposition of sword combat
pub fn sword_combat_chunks() -> Vec<ActionChunk> {
    vec![
        // Level 0-2: Atomic steps (novice thinks about each)
        ActionChunk {
            id: ChunkId::new("sword_grip"),
            name: "Grip Sword".into(),
            sub_steps: vec![
                ActionStep { id: StepId::new("hand_position"), .. },
                ActionStep { id: StepId::new("finger_pressure"), .. },
            ],
            cognitive_load: 0.2,
            base_duration: 0.3,
            variability: 0.4,
            minimum_skill: 0,
            ..
        },

        ActionChunk {
            id: ChunkId::new("stance"),
            name: "Fighting Stance".into(),
            sub_steps: vec![
                ActionStep { id: StepId::new("foot_position"), .. },
                ActionStep { id: StepId::new("weight_distribution"), .. },
                ActionStep { id: StepId::new("arm_position"), .. },
            ],
            cognitive_load: 0.3,
            base_duration: 0.5,
            variability: 0.3,
            minimum_skill: 0,
            ..
        },

        // Level 3-5: Combined chunks (journeyman)
        ActionChunk {
            id: ChunkId::new("ready_stance"),
            name: "Ready Stance".into(),
            sub_steps: vec![
                // Contains grip + stance
                ActionStep { id: StepId::new("hand_position"), .. },
                ActionStep { id: StepId::new("finger_pressure"), .. },
                ActionStep { id: StepId::new("foot_position"), .. },
                ActionStep { id: StepId::new("weight_distribution"), .. },
                ActionStep { id: StepId::new("arm_position"), .. },
            ],
            cognitive_load: 0.3, // Less than sum of parts
            base_duration: 0.4, // Faster than sequential
            variability: 0.2,
            minimum_skill: 3,
            prerequisite_chunks: vec![
                ChunkId::new("sword_grip"),
                ChunkId::new("stance"),
            ],
            ..
        },

        ActionChunk {
            id: ChunkId::new("basic_cut"),
            name: "Basic Cut".into(),
            sub_steps: vec![
                ActionStep { id: StepId::new("raise_weapon"), .. },
                ActionStep { id: StepId::new("target_selection"), .. },
                ActionStep { id: StepId::new("swing_arc"), .. },
                ActionStep { id: StepId::new("impact_timing"), .. },
            ],
            cognitive_load: 0.4,
            base_duration: 0.6,
            variability: 0.3,
            minimum_skill: 2,
            ..
        },

        // Level 6-8: Advanced chunks (expert)
        ActionChunk {
            id: ChunkId::new("attack_sequence"),
            name: "Attack Sequence".into(),
            sub_steps: vec![
                // Contains ready + cut + recovery
            ],
            cognitive_load: 0.3,
            base_duration: 0.8,
            variability: 0.15,
            minimum_skill: 6,
            prerequisite_chunks: vec![
                ChunkId::new("ready_stance"),
                ChunkId::new("basic_cut"),
            ],
            ..
        },

        ActionChunk {
            id: ChunkId::new("parry_riposte"),
            name: "Parry-Riposte".into(),
            sub_steps: vec![
                // Defense immediately into counter
            ],
            cognitive_load: 0.4,
            base_duration: 0.5,
            variability: 0.2,
            minimum_skill: 7,
            ..
        },

        // Level 9-10: Master chunks
        ActionChunk {
            id: ChunkId::new("combat_flow"),
            name: "Combat Flow".into(),
            sub_steps: vec![
                // Entire combat sequence as single thought
            ],
            cognitive_load: 0.2, // Masters make it look easy
            base_duration: 0.3, // Per action in flow
            variability: 0.1,   // Very consistent
            minimum_skill: 9,
            ..
        },
    ]
}
```

### Time Comparison Table

| Skill Level | Action | Chunks | Time | Notes |
|-------------|--------|--------|------|-------|
| 0 (Novice) | Basic Attack | 8 | 4.0s | Each step conscious |
| 3 (Journeyman) | Basic Attack | 4 | 1.8s | Some automation |
| 6 (Expert) | Basic Attack | 2 | 0.9s | Fluid sequences |
| 10 (Master) | Basic Attack | 1 | 0.3s | Single thought |

---

## Learning and Automation

### Chunk Acquisition

```rust
/// How chunks are learned and automated
#[derive(Debug)]
pub struct ChunkLearning {
    pub chunk_id: ChunkId,
    pub acquisition_state: AcquisitionState,

    // Learning progress
    pub practice_count: u32,
    pub success_rate: f32,
    pub automation_progress: f32,       // 0.0 - 1.0

    // Quality
    pub execution_variability: f32,     // Decreases with practice
    pub cognitive_load: f32,            // Decreases with automation
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AcquisitionState {
    Unknown,          // Haven't encountered this chunk
    Observed,         // Seen others do it
    Attempting,       // Trying to learn
    Conscious,        // Can do with full attention
    Automating,       // Building automaticity
    Automated,        // Requires minimal attention
}

impl ChunkLearning {
    /// Practice this chunk
    pub fn practice(&mut self, outcome: ChunkOutcome) {
        self.practice_count += 1;

        // Update success rate (rolling average)
        let success = if outcome.succeeded { 1.0 } else { 0.0 };
        self.success_rate = self.success_rate * 0.9 + success * 0.1;

        // Variability decreases with successful practice
        if outcome.succeeded {
            self.execution_variability *= 0.99;
        }

        // Automation progress
        if self.acquisition_state == AcquisitionState::Automating {
            self.automation_progress += 0.01 * self.success_rate;

            if self.automation_progress >= 1.0 {
                self.acquisition_state = AcquisitionState::Automated;
                self.cognitive_load *= 0.3; // Dramatic reduction
            }
        }

        // State transitions
        self.update_state();
    }

    fn update_state(&mut self) {
        match self.acquisition_state {
            AcquisitionState::Attempting => {
                if self.success_rate > 0.5 && self.practice_count > 10 {
                    self.acquisition_state = AcquisitionState::Conscious;
                }
            }
            AcquisitionState::Conscious => {
                if self.success_rate > 0.8 && self.practice_count > 50 {
                    self.acquisition_state = AcquisitionState::Automating;
                    self.automation_progress = 0.0;
                }
            }
            _ => {}
        }
    }

    /// Current execution time for this chunk
    pub fn execution_time(&self) -> f32 {
        let base = match self.acquisition_state {
            AcquisitionState::Attempting => 2.0,
            AcquisitionState::Conscious => 1.0,
            AcquisitionState::Automating => 0.6,
            AcquisitionState::Automated => 0.3,
            _ => 3.0, // Unknown/observed
        };

        // Add variability
        base * (1.0 + self.execution_variability * rand::random::<f32>())
    }
}

#[derive(Debug)]
pub struct ChunkOutcome {
    pub succeeded: bool,
    pub execution_time: f32,
    pub quality: f32,
}
```

### Deliberate Practice

```rust
/// Optimal practice conditions for skill acquisition
#[derive(Debug)]
pub struct PracticeContext {
    pub difficulty: Difficulty,
    pub feedback_quality: f32,          // 0.0 - 1.0
    pub focus_level: f32,               // 0.0 - 1.0
    pub fatigue: f32,                   // 0.0 - 1.0
}

#[derive(Debug, Clone, Copy)]
pub enum Difficulty {
    TooEasy,          // Boredom, no learning
    Easy,             // Reinforcement
    Optimal,          // Flow state, max learning
    Challenging,      // Growth with effort
    TooHard,          // Frustration, limited learning
}

/// Calculate learning rate from practice conditions
pub fn learning_rate(context: &PracticeContext) -> f32 {
    let difficulty_factor = match context.difficulty {
        Difficulty::TooEasy => 0.2,
        Difficulty::Easy => 0.6,
        Difficulty::Optimal => 1.0,
        Difficulty::Challenging => 0.8,
        Difficulty::TooHard => 0.3,
    };

    let feedback_factor = 0.5 + context.feedback_quality * 0.5;
    let focus_factor = context.focus_level;
    let fatigue_factor = 1.0 - context.fatigue * 0.5;

    difficulty_factor * feedback_factor * focus_factor * fatigue_factor
}

/// Determine practice difficulty based on current skill
pub fn assess_difficulty(
    skill: &SkillMastery,
    action: ActionId,
    chunk_library: &ChunkLibrary,
) -> Difficulty {
    let sequence = skill.optimal_sequence(action, chunk_library);
    let cognitive_load = sequence.cognitive_load();

    // Compare to skill capacity
    let capacity = 1.0 + skill.level as f32 * 0.2;
    let load_ratio = cognitive_load / capacity;

    match load_ratio {
        r if r < 0.3 => Difficulty::TooEasy,
        r if r < 0.5 => Difficulty::Easy,
        r if r < 0.8 => Difficulty::Optimal,
        r if r < 1.2 => Difficulty::Challenging,
        _ => Difficulty::TooHard,
    }
}
```

---

## Skills in Combat

### Combat Timing

```rust
/// How skill affects combat timing
pub fn combat_action_time(
    entity: &Entity,
    action: CombatAction,
    chunk_library: &ChunkLibrary,
) -> f32 {
    let skill = entity.skill_for_action(&action);
    let sequence = skill.optimal_sequence(action.to_action_id(), chunk_library);

    let base_time = sequence.execution_time(skill.level);

    // Physical factors
    let fatigue_factor = 1.0 + entity.fatigue.exertion * 0.5;
    let wound_factor = 1.0 + entity.wounds.total_severity() * 0.3;
    let encumbrance_factor = 1.0 + entity.encumbrance_ratio() * 0.2;

    base_time * fatigue_factor * wound_factor * encumbrance_factor
}

/// Calculate reaction time based on chunking
pub fn reaction_time(
    entity: &Entity,
    stimulus: Stimulus,
    chunk_library: &ChunkLibrary,
) -> f32 {
    // Base perception time
    let perception_time = 0.15; // ~150ms human baseline

    // Recognition time depends on pattern matching
    let skill = entity.relevant_skill_for_stimulus(&stimulus);
    let recognition_time = match skill.level {
        0..=2 => 0.5,   // Novice: slow recognition
        3..=5 => 0.3,   // Journeyman: familiar patterns
        6..=8 => 0.15,  // Expert: quick recognition
        9..=10 => 0.05, // Master: near-instant
        _ => 0.5,
    };

    // Decision time depends on chunk availability
    let decision_time = if skill.known_chunks.len() > 5 {
        0.1 // Many options = quick selection
    } else {
        0.3 // Few options = deliberation
    };

    perception_time + recognition_time + decision_time
}
```

### Skill Affects Quality

```rust
/// Skill affects execution quality, not damage directly
pub fn execution_quality(
    skill: &SkillMastery,
    chunk: &ActionChunk,
) -> ExecutionQuality {
    // Check if chunk is automated
    let chunk_learning = skill.chunk_status(chunk.id);

    let precision = match chunk_learning.acquisition_state {
        AcquisitionState::Automated => 0.9 + rand::random::<f32>() * 0.1,
        AcquisitionState::Automating => 0.7 + rand::random::<f32>() * 0.2,
        AcquisitionState::Conscious => 0.5 + rand::random::<f32>() * 0.3,
        _ => 0.3 + rand::random::<f32>() * 0.4,
    };

    let consistency = 1.0 - chunk_learning.execution_variability;

    // Physical efficiency
    let efficiency = match chunk_learning.acquisition_state {
        AcquisitionState::Automated => 0.9,
        AcquisitionState::Automating => 0.7,
        AcquisitionState::Conscious => 0.5,
        _ => 0.3,
    };

    ExecutionQuality {
        precision,
        consistency,
        efficiency,
    }
}

#[derive(Debug)]
pub struct ExecutionQuality {
    pub precision: f32,      // How accurate the action is
    pub consistency: f32,    // How repeatable
    pub efficiency: f32,     // How little wasted motion
}

/// Apply execution quality to combat result
pub fn apply_quality_to_attack(
    base_impact: &ImpactForce,
    quality: &ExecutionQuality,
) -> ImpactForce {
    ImpactForce {
        // Precision affects hit location accuracy
        // (handled separately in targeting)

        // Efficiency affects force delivery
        force_newtons: base_impact.force_newtons * quality.efficiency,

        // Consistency affects follow-through
        contact_time: base_impact.contact_time * (0.8 + quality.consistency * 0.4),

        ..base_impact.clone()
    }
}
```

---

## Non-Combat Skills

### Crafting Skills

```rust
/// Crafting skill chunking
pub fn crafting_skill_chunks() -> Vec<ActionChunk> {
    vec![
        // Blacksmithing
        ActionChunk {
            id: ChunkId::new("heat_assessment"),
            name: "Assess Metal Heat".into(),
            sub_steps: vec![
                ActionStep { id: StepId::new("observe_color"), .. },
                ActionStep { id: StepId::new("estimate_temperature"), .. },
            ],
            cognitive_load: 0.3,
            base_duration: 2.0,  // Novice stares for 2 seconds
            minimum_skill: 0,
            ..
        },

        ActionChunk {
            id: ChunkId::new("hammer_strike"),
            name: "Hammer Strike".into(),
            sub_steps: vec![
                ActionStep { id: StepId::new("position_metal"), .. },
                ActionStep { id: StepId::new("aim_strike"), .. },
                ActionStep { id: StepId::new("swing_hammer"), .. },
                ActionStep { id: StepId::new("assess_result"), .. },
            ],
            cognitive_load: 0.4,
            base_duration: 1.5,
            minimum_skill: 1,
            ..
        },

        // Expert chunk: combines heat assessment with striking
        ActionChunk {
            id: ChunkId::new("forge_rhythm"),
            name: "Forge Rhythm".into(),
            sub_steps: vec![
                // Heat + strike + assess as single flow
            ],
            cognitive_load: 0.3,
            base_duration: 0.8,  // Much faster per strike
            minimum_skill: 6,
            prerequisite_chunks: vec![
                ChunkId::new("heat_assessment"),
                ChunkId::new("hammer_strike"),
            ],
            ..
        },

        // Master chunk
        ActionChunk {
            id: ChunkId::new("shaping_flow"),
            name: "Shaping Flow".into(),
            sub_steps: vec![
                // Entire shaping operation as single extended action
            ],
            cognitive_load: 0.2,
            base_duration: 0.5,  // Per unit of progress
            minimum_skill: 9,
            ..
        },
    ]
}
```

### Social Skills

```rust
/// Social interaction chunking
pub fn social_skill_chunks() -> Vec<ActionChunk> {
    vec![
        // Reading people
        ActionChunk {
            id: ChunkId::new("read_expression"),
            name: "Read Expression".into(),
            sub_steps: vec![
                ActionStep { id: StepId::new("observe_face"), .. },
                ActionStep { id: StepId::new("interpret_emotion"), .. },
            ],
            cognitive_load: 0.4,
            base_duration: 1.0,
            minimum_skill: 0,
            ..
        },

        ActionChunk {
            id: ChunkId::new("read_body_language"),
            name: "Read Body Language".into(),
            sub_steps: vec![
                ActionStep { id: StepId::new("observe_posture"), .. },
                ActionStep { id: StepId::new("observe_gestures"), .. },
                ActionStep { id: StepId::new("interpret_intent"), .. },
            ],
            cognitive_load: 0.5,
            base_duration: 2.0,
            minimum_skill: 2,
            ..
        },

        // Expert chunk
        ActionChunk {
            id: ChunkId::new("rapid_assessment"),
            name: "Rapid Assessment".into(),
            sub_steps: vec![
                // Expression + body language + context
            ],
            cognitive_load: 0.3,
            base_duration: 0.3,  // Instant read
            minimum_skill: 7,
            ..
        },

        // Persuasion
        ActionChunk {
            id: ChunkId::new("build_rapport"),
            name: "Build Rapport".into(),
            sub_steps: vec![
                ActionStep { id: StepId::new("mirror_posture"), .. },
                ActionStep { id: StepId::new("match_energy"), .. },
                ActionStep { id: StepId::new("find_common_ground"), .. },
            ],
            cognitive_load: 0.6,
            base_duration: 30.0,  // Takes time
            minimum_skill: 3,
            ..
        },

        // Master social skill
        ActionChunk {
            id: ChunkId::new("social_flow"),
            name: "Social Flow".into(),
            sub_steps: vec![
                // Read, respond, guide conversation as one
            ],
            cognitive_load: 0.2,
            base_duration: 0.1,  // Per interaction beat
            minimum_skill: 9,
            ..
        },
    ]
}
```

---

## Teaching and Knowledge Transfer

### Teaching System

```rust
/// Teaching as knowledge transfer
#[derive(Debug)]
pub struct TeachingSession {
    pub teacher: EntityId,
    pub student: EntityId,
    pub skill: SkillId,
    pub chunk: ChunkId,

    pub teacher_skill: u8,
    pub student_skill: u8,
    pub teaching_quality: f32,
}

impl TeachingSession {
    /// Calculate learning multiplier from teaching
    pub fn learning_multiplier(&self) -> f32 {
        // Skill gap matters
        let skill_gap = self.teacher_skill as i32 - self.student_skill as i32;
        let gap_factor = if skill_gap < 2 {
            0.5  // Too close in skill
        } else if skill_gap > 5 {
            0.7  // Too far apart (hard to relate)
        } else {
            1.0  // Optimal gap
        };

        // Teaching quality
        let quality_factor = 0.5 + self.teaching_quality * 0.5;

        gap_factor * quality_factor
    }

    /// Transfer a chunk from teacher to student
    pub fn transfer_chunk(
        &self,
        student_learning: &mut ChunkLearning,
    ) {
        // Student can observe the chunk
        if student_learning.acquisition_state == AcquisitionState::Unknown {
            student_learning.acquisition_state = AcquisitionState::Observed;
        }

        // Teaching accelerates progression
        let multiplier = self.learning_multiplier();

        match student_learning.acquisition_state {
            AcquisitionState::Observed => {
                // Can start attempting with guidance
                student_learning.acquisition_state = AcquisitionState::Attempting;
            }
            AcquisitionState::Attempting => {
                student_learning.success_rate += 0.1 * multiplier;
            }
            AcquisitionState::Conscious => {
                student_learning.automation_progress += 0.05 * multiplier;
            }
            _ => {}
        }
    }
}

/// Teaching skill affects transfer efficiency
pub fn teacher_quality(
    teacher: &Entity,
    subject_skill: SkillId,
) -> f32 {
    let subject_mastery = teacher.skills.get(&subject_skill)
        .map(|s| s.level)
        .unwrap_or(0);

    let teaching_skill = teacher.skills.get(&SkillId::Teaching)
        .map(|s| s.level)
        .unwrap_or(0);

    // Need both subject expertise and teaching ability
    let subject_factor = subject_mastery as f32 / 10.0;
    let teaching_factor = 0.3 + (teaching_skill as f32 / 10.0) * 0.7;

    subject_factor * teaching_factor
}
```

---

## Integration with Other Systems

### Memory and Skill

```rust
/// Skill practice creates memories
pub fn create_practice_memory(
    skill: SkillId,
    chunk: ChunkId,
    outcome: PracticeOutcome,
    current_time: SimTime,
) -> Memory {
    Memory {
        id: MemoryId::new(),
        memory_type: MemoryType::Practice,
        timestamp: current_time,
        subject: None,
        location: None,
        action: Some(ActionId::Practice(skill)),
        outcome: match outcome {
            PracticeOutcome::Success => MemoryOutcome::Positive,
            PracticeOutcome::Failure => MemoryOutcome::Negative,
            PracticeOutcome::Challenge => MemoryOutcome::Mixed,
            _ => MemoryOutcome::Neutral,
        },
        emotional_valence: match outcome {
            PracticeOutcome::Success => 0.3,
            PracticeOutcome::Failure => -0.2,
            PracticeOutcome::Challenge => 0.4,
            _ => 0.0,
        },
        emotional_intensity: match outcome {
            PracticeOutcome::Challenge => 0.6,
            _ => 0.3,
        },
        initial_strength: 0.4,
        current_strength: 0.4,
        recall_count: 0,
        last_recall: current_time,
        tags: vec![MemoryTag::Success],
    }
}
```

### Fatigue and Skill

```rust
/// Fatigue affects chunk execution
pub fn fatigue_skill_modifier(
    fatigue: f32,
    chunk: &ActionChunk,
) -> ChunkModifier {
    // Automated chunks resist fatigue better
    let automation_resistance = 0.3; // Placeholder for chunk automation state

    let time_penalty = fatigue * (1.0 - automation_resistance) * 0.5;
    let variability_increase = fatigue * (1.0 - automation_resistance) * 0.3;

    ChunkModifier {
        time_multiplier: 1.0 + time_penalty,
        variability_increase,
        cognitive_load_increase: fatigue * 0.2,
    }
}

#[derive(Debug)]
pub struct ChunkModifier {
    pub time_multiplier: f32,
    pub variability_increase: f32,
    pub cognitive_load_increase: f32,
}
```

---

## Related Specifications

| Document | Relationship |
|----------|--------------|
| [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md) | Entity skill storage |
| [06-BALANCE-COMPOSITION-SPEC](06-BALANCE-COMPOSITION-SPEC.md) | Skill as chunking concept |
| [14-PHYSICS-COMBAT-SPEC](14-PHYSICS-COMBAT-SPEC.md) | Combat timing |
| [17-SOCIAL-MEMORY-SPEC](17-SOCIAL-MEMORY-SPEC.md) | Practice memories |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification created |
