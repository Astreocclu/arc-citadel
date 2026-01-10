//! Parse natural language commands into structured intents
//!
//! This module is key to the "type orders, entities interpret" design pillar.
//! The LLM parses player commands into structured intents, but does NOT
//! control entity behavior - entities interpret commands through their
//! own values and personality.

use crate::core::error::Result;
use crate::llm::client::LlmClient;
use crate::llm::context::GameContext;
use serde::{Deserialize, Serialize};

/// Parsed intent from a natural language command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedIntent {
    /// The type of action requested
    pub action: IntentAction,
    /// What to build/craft/attack/etc (if applicable)
    pub target: Option<String>,
    /// Where the action should take place (if applicable)
    pub location: Option<String>,
    /// Who should perform the action (names or descriptors)
    pub subjects: Option<Vec<String>>,
    /// Priority level for the command
    pub priority: IntentPriority,
    /// Terms that might be interpreted differently by non-human species
    pub ambiguous_concepts: Vec<String>,
    /// Parser's confidence in the interpretation (0.0 - 1.0)
    pub confidence: f32,
}

/// Types of actions the player can command
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntentAction {
    /// Construct structures
    Build,
    /// Create items
    Craft,
    /// Move personnel to roles/locations
    Assign,
    /// Engage enemies or prepare defenses
    Combat,
    /// Collect resources
    Gather,
    /// Travel to location
    Move,
    /// Ask about game state (not an action)
    Query,
    /// Interact with characters
    Social,
    /// Rest and recover
    Rest,
    /// Could not determine intent
    Unknown,
}

/// Priority level for commands
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntentPriority {
    /// Life-threatening urgency
    Critical,
    /// Important, should be done soon
    High,
    /// Standard priority
    Normal,
    /// Can wait, low urgency
    Low,
}

impl Default for IntentPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl Default for ParsedIntent {
    fn default() -> Self {
        Self {
            action: IntentAction::Unknown,
            target: None,
            location: None,
            subjects: None,
            priority: IntentPriority::Normal,
            ambiguous_concepts: Vec::new(),
            confidence: 0.0,
        }
    }
}

/// Parse a natural language command into a structured intent
///
/// # Arguments
/// * `client` - The LLM client to use for parsing
/// * `input` - The player's natural language command
/// * `context` - Current game context for disambiguation
///
/// # Returns
/// A parsed intent that can be interpreted by entities
pub async fn parse_command(
    client: &LlmClient,
    input: &str,
    context: &GameContext,
) -> Result<ParsedIntent> {
    let system_prompt = PARSE_SYSTEM_PROMPT;
    let user_prompt = format!(
        "CONTEXT:\n{}\n\nPLAYER INPUT:\n{}\n\nParse this command into JSON:",
        context.summary(),
        input
    );

    let response = client.complete(system_prompt, &user_prompt).await?;
    let json_str = extract_json(&response)?;

    let intent: ParsedIntent = serde_json::from_str(json_str).map_err(|e| {
        crate::core::error::ArcError::LlmError(format!(
            "Failed to parse intent: {} - Response: {}",
            e, response
        ))
    })?;

    Ok(intent)
}

/// Extract JSON object from LLM response (handles surrounding text)
fn extract_json(response: &str) -> Result<&str> {
    let start = response.find('{').ok_or_else(|| {
        crate::core::error::ArcError::LlmError("No JSON found in response".into())
    })?;
    let end = response.rfind('}').ok_or_else(|| {
        crate::core::error::ArcError::LlmError("No closing brace found in response".into())
    })?;
    Ok(&response[start..=end])
}

/// System prompt for command parsing
const PARSE_SYSTEM_PROMPT: &str = r#"You are parsing player commands for a medieval simulation game.
Convert natural language orders into structured JSON.

AVAILABLE ACTIONS:
- BUILD: Construct structures (walls, buildings, fortifications)
- CRAFT: Create items (weapons, tools, goods)
- ASSIGN: Move personnel to roles/locations
- COMBAT: Engage enemies or prepare defenses
- GATHER: Collect resources (wood, stone, food)
- MOVE: Travel to location
- QUERY: Ask about game state (not an action)
- SOCIAL: Interact with characters (talk, negotiate, befriend)
- REST: Rest and recover

IMPORTANT: Identify "ambiguous_concepts" - terms that different species might interpret differently:
- "beautiful" - humans value visual beauty, dwarves value craftsmanship
- "valuable" - differs by culture and individual values
- "safe" - varies by risk tolerance
- "worthy" - depends on individual values

OUTPUT FORMAT (JSON only, no explanation):
{
  "action": "ACTION_TYPE",
  "target": "what to build/craft/attack/etc or null",
  "location": "where (if applicable) or null",
  "subjects": ["who should do this"] or null if unspecified,
  "priority": "CRITICAL|HIGH|NORMAL|LOW",
  "ambiguous_concepts": ["terms that might be interpreted differently by non-humans"],
  "confidence": 0.0-1.0
}

Examples:
"build a wall" -> {"action": "BUILD", "target": "wall", "location": null, "subjects": null, "priority": "NORMAL", "ambiguous_concepts": [], "confidence": 0.9}
"have Marcus guard the east" -> {"action": "ASSIGN", "target": "guard duty", "location": "east", "subjects": ["Marcus"], "priority": "NORMAL", "ambiguous_concepts": [], "confidence": 0.85}
"make it beautiful" -> {"action": "CRAFT", "target": null, "location": null, "subjects": null, "priority": "NORMAL", "ambiguous_concepts": ["beautiful"], "confidence": 0.6}
"everyone rest now" -> {"action": "REST", "target": null, "location": null, "subjects": null, "priority": "HIGH", "ambiguous_concepts": [], "confidence": 0.95}
"send the brave ones to scout" -> {"action": "ASSIGN", "target": "scouting", "location": null, "subjects": null, "priority": "NORMAL", "ambiguous_concepts": ["brave"], "confidence": 0.7}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_simple() {
        let response = r#"{"action": "BUILD", "target": "wall"}"#;
        let json = extract_json(response).unwrap();
        assert_eq!(json, response);
    }

    #[test]
    fn test_extract_json_with_surrounding_text() {
        let response = r#"Here is the parsed command:
{"action": "BUILD", "target": "wall", "location": null, "subjects": null, "priority": "NORMAL", "ambiguous_concepts": [], "confidence": 0.9}
Let me know if you need anything else."#;
        let json = extract_json(response).unwrap();
        assert!(json.starts_with('{'));
        assert!(json.ends_with('}'));
        assert!(json.contains("BUILD"));
    }

    #[test]
    fn test_extract_json_no_json() {
        let response = "I don't understand that command";
        let result = extract_json(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_parsed_intent_default() {
        let intent = ParsedIntent::default();
        assert_eq!(intent.action, IntentAction::Unknown);
        assert_eq!(intent.priority, IntentPriority::Normal);
        assert_eq!(intent.confidence, 0.0);
    }

    #[test]
    fn test_intent_action_serialization() {
        let action = IntentAction::Build;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"BUILD\"");
    }

    #[test]
    fn test_intent_action_deserialization() {
        let json = "\"COMBAT\"";
        let action: IntentAction = serde_json::from_str(json).unwrap();
        assert_eq!(action, IntentAction::Combat);
    }

    #[test]
    fn test_full_intent_deserialization() {
        let json = r#"{
            "action": "ASSIGN",
            "target": "guard duty",
            "location": "east gate",
            "subjects": ["Marcus", "Elena"],
            "priority": "HIGH",
            "ambiguous_concepts": ["east"],
            "confidence": 0.85
        }"#;
        let intent: ParsedIntent = serde_json::from_str(json).unwrap();
        assert_eq!(intent.action, IntentAction::Assign);
        assert_eq!(intent.target, Some("guard duty".to_string()));
        assert_eq!(intent.location, Some("east gate".to_string()));
        assert_eq!(
            intent.subjects,
            Some(vec!["Marcus".to_string(), "Elena".to_string()])
        );
        assert_eq!(intent.priority, IntentPriority::High);
        assert_eq!(intent.ambiguous_concepts, vec!["east".to_string()]);
        assert!((intent.confidence - 0.85).abs() < 0.001);
    }
}
