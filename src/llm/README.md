# LLM Module

> Natural language command parsing. The LLM translates player intent into structured game commands.

## Module Structure

```
llm/
├── mod.rs              # Module exports
├── client.rs           # Async HTTP client for LLM API
├── parser.rs           # Parse responses into structured intents
├── context.rs          # Build game context for prompts
├── species_interpret.rs # Species-specific interpretation (stub)
└── prompts.rs          # Prompt templates (stub)
```

## The LLM's Role

```
┌─────────────────────────────────────────────────────────────────────┐
│                     PLAYER COMMAND FLOW                              │
│                                                                      │
│   "Have Marcus build      LLM        ParsedIntent       Game        │
│    a wall near the   ──────────▶   {                  ──────────▶   │
│    eastern gate"        Parser       action: Build,     Systems     │
│                                      target: "wall",                 │
│                                      location: "eastern gate",       │
│                                      subjects: ["Marcus"]            │
│                                    }                                 │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

The LLM:
- Understands natural language player commands
- Extracts structured intent (action, target, location, subjects)
- Identifies ambiguous concepts needing interpretation
- Reports confidence in its parsing

## LlmClient (`client.rs`)

Async HTTP client for Claude API:

```rust
pub struct LlmClient {
    client: Client,      // reqwest HTTP client
    api_key: String,
    api_url: String,
    model: String,
}

impl LlmClient {
    // Create from environment variables
    pub fn from_env() -> Result<Self>;

    // Send a completion request
    pub async fn complete(&self, system: &str, user: &str) -> Result<String>;
}
```

**Environment Variables:**
| Variable | Required | Default |
|----------|----------|---------|
| `LLM_API_KEY` | Yes | - |
| `LLM_API_URL` | No | `https://api.anthropic.com/v1/messages` |
| `LLM_MODEL` | No | `claude-3-haiku-20240307` |

## ParsedIntent (`parser.rs`)

Structured output from command parsing:

```rust
pub struct ParsedIntent {
    pub action: IntentAction,           // What type of action
    pub target: Option<String>,         // What to act on
    pub location: Option<String>,       // Where
    pub subjects: Option<Vec<String>>,  // Who should do it
    pub priority: IntentPriority,       // How urgent
    pub ambiguous_concepts: Vec<String>,// Terms needing interpretation
    pub confidence: f32,                // 0.0-1.0
}

pub enum IntentAction {
    Build,    // Construct structures
    Craft,    // Create items
    Assign,   // Move personnel to roles/locations
    Combat,   // Engage enemies or prepare defenses
    Gather,   // Collect resources
    Move,     // Travel to location
    Query,    // Ask about game state
    Social,   // Interact with characters
    Rest,     // Rest and recover
    Unknown,  // Could not determine action
}
```

## GameContext (`context.rs`)

Assembles world state for LLM prompts:

```rust
pub struct GameContext {
    pub location_name: String,
    pub entity_count: usize,
    pub available_resources: Vec<String>,
    pub recent_events: Vec<String>,
    pub named_entities: Vec<NamedEntity>,
    pub threats: Vec<String>,
}

impl GameContext {
    // Build from world state
    pub fn from_world(world: &World) -> Self;

    // Format for prompt insertion
    pub fn summary(&self) -> String;
}
```

## Usage Pattern

```rust
// In main.rs
let llm_client = LlmClient::from_env().ok();

// Game loop
if let Some(ref client) = llm_client {
    let context = GameContext::from_world(&world);

    match parse_command(client, player_input, &context).await {
        Ok(intent) => {
            println!("Parsed: {:?}", intent.action);

            // Handle ambiguous concepts
            if !intent.ambiguous_concepts.is_empty() {
                println!("Ambiguous: {:?}", intent.ambiguous_concepts);
                // May need species-specific interpretation
            }

            // Convert to game action
            match intent.action {
                IntentAction::Build => {
                    // Create build task for specified subjects
                }
                IntentAction::Assign => {
                    // Assign entity to role/location
                }
                // ... handle other actions
            }
        }
        Err(e) => {
            println!("Could not parse: {}", e);
            // Fall back to simple commands
        }
    }
} else {
    // LLM not available - use simple commands
    println!("Commands: tick, spawn <name>, quit");
}
```

## Graceful Degradation

The game works without LLM:

```rust
let llm_client = LlmClient::from_env().ok();

if llm_client.is_none() {
    tracing::warn!("LLM_API_KEY not set - running without natural language commands");
}

// Simple command fallback
match input {
    "tick" | "t" => run_simulation_tick(&mut world),
    s if s.starts_with("spawn ") => {
        let name = s.strip_prefix("spawn ").unwrap();
        world.spawn_human(name.into());
    }
    "quit" | "q" => break,
    _ => println!("Unknown command"),
}
```

## Best Practices

### Error Handling
```rust
match client.complete(system_prompt, user_prompt).await {
    Ok(response) => {
        // Parse the response
        let intent = parse_response(&response)?;
        Ok(intent)
    }
    Err(e) => {
        tracing::warn!("LLM error: {}", e);
        // Game continues - just can't parse this command
        Err(e)
    }
}
```

### Building Good Context
```rust
let context = GameContext {
    location_name: "Main Camp".into(),
    entity_count: world.entity_count(),
    available_resources: vec!["wood".into(), "stone".into()],
    recent_events: vec!["Raiders spotted to the north".into()],
    named_entities: world.humans.iter_living()
        .take(10)  // Limit context size
        .map(|i| NamedEntity {
            name: world.humans.names[i].clone(),
            species: Species::Human,
            role: "worker".into(),
            status: if world.humans.body_states[i].can_act() {
                "healthy"
            } else {
                "incapacitated"
            }.into(),
        })
        .collect(),
    threats: vec![],
};
```

### Handling Ambiguous Concepts
```rust
if !intent.ambiguous_concepts.is_empty() {
    // "make it beautiful" has ambiguous "beautiful"
    // Different species interpret this differently
    for concept in &intent.ambiguous_concepts {
        match concept.as_str() {
            "beautiful" => {
                // Human: aesthetic, ornate
                // Dwarf: functional, well-crafted
                // Elf: natural, harmonious
            }
            _ => {}
        }
    }
}
```

## Testing

```bash
# LLM tests require API key
LLM_API_KEY=sk-... cargo test --lib llm::
```

### Testing Without LLM
For unit tests, mock the LLM response:

```rust
#[test]
fn test_parse_response() {
    let mock_response = r#"{"action": "BUILD", "target": "wall", "confidence": 0.9}"#;
    let intent = parse_json_response(mock_response).unwrap();
    assert_eq!(intent.action, IntentAction::Build);
}
```

## Future Extensions

### Species-Specific Interpretation (`species_interpret.rs`)
Different species interpret ambiguous commands differently:
- "Make it beautiful" → Human adds ornaments, Dwarf improves craftsmanship
- "Protect us" → Human builds walls, Dwarf digs tunnels

### Prompt Templates (`prompts.rs`)
Reusable prompt templates for consistent parsing:
- Command parsing prompt
- Clarification prompts
- Context summary templates
