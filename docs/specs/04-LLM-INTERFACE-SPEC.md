# 04-LLM-INTERFACE-SPEC
> Natural language command parsing and LLM integration

## Overview

Arc Citadel uses LLM integration to translate natural language player commands into structured game actions. The LLM acts as a **translator only**—it never controls entity behavior directly. This specification defines the parsing pipeline, context building, and graceful degradation.

---

## Core Principle

**The LLM is a translator, not a decision-maker.**

```
┌─────────────────────────────────────────────────────────────────────┐
│  CORRECT: LLM translates player intent                              │
│                                                                      │
│  "Build defenses" → ParsedIntent { action: SetPriority,             │
│                                    target: BuildingType::Wall }     │
│                                                                      │
│  Game systems then execute the intent through normal task flow.     │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│  FORBIDDEN: LLM controls entities                                   │
│                                                                      │
│  LLM: "Marcus should feel afraid and run away"                      │
│  LLM: "The settlers should prioritize safety over food"             │
│                                                                      │
│  Entity behavior emerges from needs/values, not LLM decisions.      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Command Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│  Player Input: "Have the miners focus on iron this week"            │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│  Context Builder                                                    │
│  ├── Entity names and roles                                         │
│  ├── Current positions and states                                   │
│  ├── Available resources and buildings                              │
│  └── Recent events                                                  │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│  LLM Parser                                                         │
│  ├── System prompt with game rules                                  │
│  ├── Context from builder                                           │
│  ├── Player command                                                 │
│  └── JSON schema for output                                         │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│  ParsedIntent                                                       │
│  {                                                                  │
│    action: IntentAction::SetPriority,                               │
│    subjects: SubjectFilter::Role(Role::Miner),                      │
│    target: IntentTarget::Resource(ResourceType::Iron),              │
│    duration: Some(Duration::Weeks(1)),                              │
│    priority: IntentPriority::Normal,                                │
│  }                                                                  │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│  Intent Executor                                                    │
│  ├── Resolve subjects (find matching entities)                      │
│  ├── Validate intent (can these entities do this?)                  │
│  ├── Create tasks or modify priorities                              │
│  └── Queue tasks in entity TaskQueues                               │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Data Structures

### ParsedIntent

```rust
pub struct ParsedIntent {
    pub id: IntentId,
    pub action: IntentAction,
    pub subjects: SubjectFilter,
    pub target: Option<IntentTarget>,
    pub location: Option<LocationSpec>,
    pub duration: Option<Duration>,
    pub priority: IntentPriority,
    pub conditions: Vec<IntentCondition>,
    pub raw_text: String,
    pub confidence: f32,
}

pub enum IntentAction {
    // Movement
    Move,
    Gather,
    Return,

    // Work
    Build,
    Craft,
    Repair,
    Harvest,

    // Combat
    Attack,
    Defend,
    Retreat,
    FormUp,

    // Management
    SetPriority,
    Assign,
    Cancel,
    Wait,

    // Information
    Report,
    Scout,
    Observe,
}

pub enum SubjectFilter {
    Specific(Vec<EntityId>),        // "Marcus and Elena"
    Named(Vec<String>),             // Names to resolve
    Role(Role),                     // "the miners"
    All,                            // "everyone"
    Nearby(Vec2, f32),              // "those near the gate"
    Group(GroupId),                 // "first squad"
}

pub enum IntentTarget {
    Entity(EntityId),
    Named(String),
    Resource(ResourceType),
    Building(BuildingType),
    Location(LocationSpec),
    Direction(Direction),
}

pub enum LocationSpec {
    Named(String),                  // "the eastern gate"
    Relative(Direction, f32),       // "50 units north"
    Position(Vec2),                 // Absolute coordinates
    Near(EntityId),                 // "near Marcus"
    Zone(ZoneId),                   // "in the mining zone"
}

pub enum IntentPriority {
    Critical,   // "immediately", "now"
    High,       // "urgently", "soon"
    Normal,     // Default
    Low,        // "when convenient", "eventually"
    Background, // "if nothing else to do"
}

pub enum IntentCondition {
    When(ConditionTrigger),
    Unless(ConditionTrigger),
    Until(ConditionTrigger),
}

pub enum ConditionTrigger {
    TimeElapsed(Duration),
    ResourceLevel(ResourceType, Comparison, u32),
    ThreatDetected,
    TaskComplete(TaskId),
}
```

---

## LLM Client

### Configuration

```rust
pub struct LlmConfig {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout_ms: u64,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            api_url: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key: env::var("OPENAI_API_KEY").unwrap_or_default(),
            model: "gpt-4".to_string(),
            max_tokens: 500,
            temperature: 0.3,  // Low for consistent parsing
            timeout_ms: 10000,
        }
    }
}
```

### Client Implementation

```rust
pub struct LlmClient {
    config: LlmConfig,
    http_client: reqwest::Client,
}

impl LlmClient {
    pub async fn parse_command(&self, input: &str, context: &GameContext) -> ArcResult<ParsedIntent> {
        let system_prompt = build_system_prompt();
        let context_prompt = build_context_prompt(context);
        let user_prompt = format!("Parse this command: {}", input);

        let response = self.call_api(&system_prompt, &context_prompt, &user_prompt).await;

        match response {
            Ok(json) => parse_intent_from_json(&json, input),
            Err(e) => {
                log::warn!("LLM unavailable: {e}, using fallback");
                Ok(FallbackParser::parse(input, context))
            }
        }
    }

    async fn call_api(&self, system: &str, context: &str, user: &str) -> ArcResult<String> {
        let body = json!({
            "model": self.config.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": format!("{}\n\n{}", context, user)}
            ],
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
            "response_format": {"type": "json_object"}
        });

        let response = self.http_client
            .post(&self.config.api_url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&body)
            .timeout(Duration::from_millis(self.config.timeout_ms))
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        Ok(json["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string())
    }
}
```

---

## System Prompt

The system prompt defines how the LLM should interpret commands:

```rust
fn build_system_prompt() -> String {
    r#"You are a command parser for Arc Citadel, a strategy game.

Your job is to translate natural language player commands into structured JSON.

## Rules
1. You are a TRANSLATOR only. Never suggest what entities should feel or decide.
2. Extract: action, subjects, targets, locations, timing, priority.
3. If something is ambiguous, use reasonable defaults.
4. If a command is completely unparseable, set action to "unknown".

## Output Format
Return valid JSON matching this schema:
{
    "action": "string (from: move, gather, build, craft, attack, defend, retreat, set_priority, assign, cancel, wait, report, scout)",
    "subjects": {
        "type": "specific|named|role|all|nearby",
        "value": "depends on type"
    },
    "target": {
        "type": "entity|resource|building|location|direction",
        "value": "depends on type"
    } | null,
    "location": {
        "type": "named|relative|position|near|zone",
        "value": "depends on type"
    } | null,
    "duration": "string like '1 week' or '30 minutes'" | null,
    "priority": "critical|high|normal|low|background",
    "confidence": 0.0-1.0
}

## Examples

Input: "Have the miners focus on iron"
Output: {
    "action": "set_priority",
    "subjects": {"type": "role", "value": "miner"},
    "target": {"type": "resource", "value": "iron"},
    "priority": "normal",
    "confidence": 0.95
}

Input: "Marcus, go to the eastern gate immediately"
Output: {
    "action": "move",
    "subjects": {"type": "named", "value": ["Marcus"]},
    "location": {"type": "named", "value": "eastern gate"},
    "priority": "critical",
    "confidence": 0.9
}

Input: "Everyone retreat to the keep"
Output: {
    "action": "retreat",
    "subjects": {"type": "all"},
    "location": {"type": "named", "value": "keep"},
    "priority": "high",
    "confidence": 0.85
}
"#.to_string()
}
```

---

## Context Building

### GameContext

```rust
pub struct GameContext {
    pub entities: Vec<EntityContext>,
    pub locations: Vec<LocationContext>,
    pub resources: Vec<ResourceContext>,
    pub buildings: Vec<BuildingContext>,
    pub recent_events: Vec<EventContext>,
    pub current_tick: Tick,
}

pub struct EntityContext {
    pub id: EntityId,
    pub name: String,
    pub role: Option<Role>,
    pub position: Vec2,
    pub status: EntityStatus,
}

pub struct LocationContext {
    pub name: String,
    pub position: Vec2,
    pub location_type: LocationType,
}
```

### Context Prompt Builder

```rust
fn build_context_prompt(context: &GameContext) -> String {
    let mut prompt = String::new();

    prompt.push_str("## Current Game State\n\n");

    // Entities
    prompt.push_str("### Entities\n");
    for entity in &context.entities {
        prompt.push_str(&format!(
            "- {} ({}): at ({:.0}, {:.0}), {}\n",
            entity.name,
            entity.role.map(|r| r.to_string()).unwrap_or("none".to_string()),
            entity.position.x,
            entity.position.y,
            entity.status
        ));
    }

    // Locations
    prompt.push_str("\n### Named Locations\n");
    for loc in &context.locations {
        prompt.push_str(&format!(
            "- {}: ({:.0}, {:.0})\n",
            loc.name, loc.position.x, loc.position.y
        ));
    }

    // Resources
    prompt.push_str("\n### Resources\n");
    for resource in &context.resources {
        prompt.push_str(&format!(
            "- {}: {} available\n",
            resource.resource_type, resource.amount
        ));
    }

    prompt
}
```

---

## Intent Execution

### Executing Parsed Intents

```rust
pub fn execute_intent(world: &mut World, intent: &ParsedIntent) -> ArcResult<ExecutionResult> {
    // Step 1: Resolve subjects
    let entity_ids = resolve_subjects(world, &intent.subjects)?;

    if entity_ids.is_empty() {
        return Ok(ExecutionResult::NoMatchingEntities);
    }

    // Step 2: Validate intent
    validate_intent(world, &entity_ids, intent)?;

    // Step 3: Execute based on action type
    match intent.action {
        IntentAction::Move => execute_move(world, &entity_ids, intent),
        IntentAction::Build => execute_build(world, &entity_ids, intent),
        IntentAction::SetPriority => execute_set_priority(world, &entity_ids, intent),
        IntentAction::Attack => execute_attack(world, &entity_ids, intent),
        IntentAction::Retreat => execute_retreat(world, &entity_ids, intent),
        // ... other actions
    }
}

fn resolve_subjects(world: &World, filter: &SubjectFilter) -> ArcResult<Vec<EntityId>> {
    match filter {
        SubjectFilter::Specific(ids) => Ok(ids.clone()),

        SubjectFilter::Named(names) => {
            let mut ids = Vec::new();
            for name in names {
                if let Some(id) = world.find_entity_by_name(name) {
                    ids.push(id);
                }
            }
            Ok(ids)
        }

        SubjectFilter::Role(role) => {
            Ok(world.entities_with_role(*role))
        }

        SubjectFilter::All => {
            Ok(world.all_controllable_entities())
        }

        SubjectFilter::Nearby(center, radius) => {
            Ok(world.spatial_index.query_radius(*center, *radius))
        }

        SubjectFilter::Group(group_id) => {
            Ok(world.group_members(*group_id))
        }
    }
}
```

### Action-Specific Execution

```rust
fn execute_move(world: &mut World, entities: &[EntityId], intent: &ParsedIntent) -> ArcResult<ExecutionResult> {
    let target_pos = resolve_location(world, intent.location.as_ref())?;

    for &entity_id in entities {
        let task = Task {
            id: TaskId::new(),
            action: ActionId::Walk,
            target: Some(TaskTarget::Position(target_pos)),
            progress: 0.0,
            priority: intent.priority.into(),
        };

        world.queue_task(entity_id, task)?;
    }

    Ok(ExecutionResult::TasksQueued(entities.len()))
}

fn execute_set_priority(world: &mut World, entities: &[EntityId], intent: &ParsedIntent) -> ArcResult<ExecutionResult> {
    let target_resource = match &intent.target {
        Some(IntentTarget::Resource(r)) => *r,
        _ => return Err(ArcError::InvalidIntent("SetPriority requires resource target".into())),
    };

    for &entity_id in entities {
        world.set_resource_priority(entity_id, target_resource, intent.priority)?;

        if let Some(duration) = &intent.duration {
            world.schedule_priority_reset(entity_id, target_resource, duration.clone())?;
        }
    }

    Ok(ExecutionResult::PrioritiesSet(entities.len()))
}
```

---

## Fallback Parser

When LLM is unavailable, use pattern matching:

```rust
pub struct FallbackParser;

impl FallbackParser {
    pub fn parse(input: &str, context: &GameContext) -> ParsedIntent {
        let input_lower = input.to_lowercase();

        // Pattern: "<name>, <action> <location>"
        if let Some(intent) = Self::parse_named_command(&input_lower, context) {
            return intent;
        }

        // Pattern: "<role>s <action> <target>"
        if let Some(intent) = Self::parse_role_command(&input_lower, context) {
            return intent;
        }

        // Pattern: "everyone/all <action>"
        if let Some(intent) = Self::parse_all_command(&input_lower, context) {
            return intent;
        }

        // Unknown command
        ParsedIntent {
            id: IntentId::new(),
            action: IntentAction::Wait,
            subjects: SubjectFilter::All,
            target: None,
            location: None,
            duration: None,
            priority: IntentPriority::Normal,
            conditions: Vec::new(),
            raw_text: input.to_string(),
            confidence: 0.1,
        }
    }

    fn parse_named_command(input: &str, context: &GameContext) -> Option<ParsedIntent> {
        // Look for entity names at start
        for entity in &context.entities {
            let name_lower = entity.name.to_lowercase();
            if input.starts_with(&name_lower) {
                let rest = input[name_lower.len()..].trim_start_matches(',').trim();

                // Parse action and target from rest
                let action = Self::extract_action(rest)?;
                let location = Self::extract_location(rest, context);

                return Some(ParsedIntent {
                    id: IntentId::new(),
                    action,
                    subjects: SubjectFilter::Specific(vec![entity.id]),
                    target: None,
                    location,
                    duration: None,
                    priority: Self::extract_priority(rest),
                    conditions: Vec::new(),
                    raw_text: input.to_string(),
                    confidence: 0.6,
                });
            }
        }
        None
    }

    fn extract_action(text: &str) -> Option<IntentAction> {
        if text.contains("move") || text.contains("go to") {
            Some(IntentAction::Move)
        } else if text.contains("build") {
            Some(IntentAction::Build)
        } else if text.contains("attack") {
            Some(IntentAction::Attack)
        } else if text.contains("retreat") || text.contains("fall back") {
            Some(IntentAction::Retreat)
        } else if text.contains("gather") || text.contains("collect") {
            Some(IntentAction::Gather)
        } else {
            None
        }
    }

    fn extract_priority(text: &str) -> IntentPriority {
        if text.contains("immediately") || text.contains("now") || text.contains("urgent") {
            IntentPriority::Critical
        } else if text.contains("soon") || text.contains("quickly") {
            IntentPriority::High
        } else if text.contains("eventually") || text.contains("when convenient") {
            IntentPriority::Low
        } else {
            IntentPriority::Normal
        }
    }
}
```

---

## Error Handling

### Parsing Errors

```rust
pub enum ParseError {
    LlmUnavailable,
    InvalidJson(String),
    MissingRequiredField(String),
    UnknownAction(String),
    AmbiguousSubject(String),
}

impl LlmClient {
    fn handle_parse_error(&self, error: ParseError, input: &str, context: &GameContext) -> ParsedIntent {
        log::warn!("Parse error: {:?}, falling back", error);

        match error {
            ParseError::LlmUnavailable => FallbackParser::parse(input, context),
            ParseError::AmbiguousSubject(subject) => {
                // Return intent that asks for clarification
                ParsedIntent {
                    action: IntentAction::Wait,
                    confidence: 0.0,
                    // ... flag for UI to request clarification
                }
            }
            _ => FallbackParser::parse(input, context),
        }
    }
}
```

### Graceful Degradation

The game must function without LLM:

1. **Fallback parser handles common patterns**
2. **Direct task assignment always works** (if player knows entity IDs)
3. **UI can show suggested commands**
4. **Error messages guide player to valid syntax**

---

## Performance Considerations

### Caching

```rust
pub struct IntentCache {
    cache: HashMap<String, (ParsedIntent, Instant)>,
    ttl: Duration,
}

impl IntentCache {
    pub fn get(&self, input: &str, context_hash: u64) -> Option<&ParsedIntent> {
        let key = format!("{}:{}", input, context_hash);
        self.cache.get(&key)
            .filter(|(_, time)| time.elapsed() < self.ttl)
            .map(|(intent, _)| intent)
    }

    pub fn insert(&mut self, input: &str, context_hash: u64, intent: ParsedIntent) {
        let key = format!("{}:{}", input, context_hash);
        self.cache.insert(key, (intent, Instant::now()));
    }
}
```

### Batching

```rust
impl LlmClient {
    pub async fn parse_batch(&self, inputs: &[&str], context: &GameContext) -> Vec<ParsedIntent> {
        // Batch multiple commands into single LLM call when possible
        // This reduces latency for multi-command inputs
    }
}
```

---

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| LlmClient | Complete | API calls working |
| ParsedIntent | Complete | All fields defined |
| Context Builder | Complete | Entity/location context |
| System Prompt | Complete | JSON schema output |
| Intent Executor | Partial | Move, Build working; others pending |
| Fallback Parser | Partial | Basic patterns only |
| Caching | Not started | |

---

## Related Specifications

| Document | Relationship |
|----------|--------------|
| [01-GAME-DESIGN-DOCUMENT](01-GAME-DESIGN-DOCUMENT.md) | Design pillar |
| [03-ENTITY-SIMULATION-SPEC](03-ENTITY-SIMULATION-SPEC.md) | Task execution |
| [11-ACTION-CATALOG](11-ACTION-CATALOG.md) | Available actions |
| [13-MODULE-SCHEMA-SPEC](13-MODULE-SCHEMA-SPEC.md) | Content generation |

---

## Changelog

| Date | Change |
|------|--------|
| 2026-01-14 | Initial specification created |
