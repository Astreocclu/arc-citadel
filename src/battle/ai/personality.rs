//! AI personality configuration loaded from TOML
//!
//! Personalities define behavior tendencies, decision weights,
//! tactical preferences, and difficulty modifiers.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Behavioral tendencies (0.0 to 1.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    /// Tendency to attack vs defend (0.0 = defensive, 1.0 = aggressive)
    pub aggression: f32,
    /// Tendency to avoid risks (0.0 = reckless, 1.0 = cautious)
    pub caution: f32,
    /// Tendency to act proactively (0.0 = reactive, 1.0 = proactive)
    pub initiative: f32,
    /// Tendency to use deception (feints, fake retreats)
    pub cunning: f32,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            aggression: 0.5,
            caution: 0.5,
            initiative: 0.5,
            cunning: 0.3,
        }
    }
}

/// Decision weights for evaluating options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightConfig {
    /// Weight for attacking weak/vulnerable units
    pub attack_value: f32,
    /// Weight for defending key positions
    pub defense_value: f32,
    /// Weight for flanking opportunities
    pub flanking_value: f32,
    /// Weight for preserving reserves
    pub reserve_value: f32,
    /// Strength ratio threshold to consider retreat
    pub retreat_threshold: f32,
    /// Casualty percentage to trigger withdrawal
    pub casualty_threshold: f32,
}

impl Default for WeightConfig {
    fn default() -> Self {
        Self {
            attack_value: 1.0,
            defense_value: 1.0,
            flanking_value: 1.2,
            reserve_value: 0.8,
            retreat_threshold: 0.3,
            casualty_threshold: 0.5,
        }
    }
}

/// Tactical preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferencesConfig {
    /// Preferred engagement range: "close", "medium", "ranged"
    pub preferred_range: String,
    /// Reserve commitment style: "early", "conservative", "desperate"
    pub reserve_usage: String,
    /// How often to re-evaluate plans (in ticks)
    pub re_evaluation_interval: u64,
}

impl Default for PreferencesConfig {
    fn default() -> Self {
        Self {
            preferred_range: "medium".to_string(),
            reserve_usage: "conservative".to_string(),
            re_evaluation_interval: 10,
        }
    }
}

/// Difficulty modifiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyConfig {
    /// Whether AI ignores fog of war
    pub ignores_fog_of_war: bool,
    /// Reaction delay in ticks (0 = instant, higher = slower)
    pub reaction_delay: u64,
    /// Mistake probability (0.0 = perfect, 1.0 = always mistakes)
    pub mistake_chance: f32,
}

impl Default for DifficultyConfig {
    fn default() -> Self {
        Self {
            ignores_fog_of_war: false,
            reaction_delay: 2,
            mistake_chance: 0.1,
        }
    }
}

/// Complete AI personality configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPersonality {
    /// Name of this personality (set from filename)
    #[serde(default)]
    pub name: String,
    /// Behavioral tendencies
    #[serde(default)]
    pub behavior: BehaviorConfig,
    /// Decision weights
    #[serde(default)]
    pub weights: WeightConfig,
    /// Tactical preferences
    #[serde(default)]
    pub preferences: PreferencesConfig,
    /// Difficulty modifiers
    #[serde(default)]
    pub difficulty: DifficultyConfig,
}

impl Default for AiPersonality {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            behavior: BehaviorConfig::default(),
            weights: WeightConfig::default(),
            preferences: PreferencesConfig::default(),
            difficulty: DifficultyConfig::default(),
        }
    }
}

/// Load personality from TOML file
///
/// Loads from `data/ai_personalities/{name}.toml`
pub fn load_personality(name: &str) -> Result<AiPersonality, String> {
    let path = personality_path(name);

    let contents = fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read personality file {:?}: {}", path, e))?;

    let mut personality: AiPersonality = toml::from_str(&contents)
        .map_err(|e| format!("Failed to parse personality TOML: {}", e))?;

    personality.name = name.to_string();
    Ok(personality)
}

/// Get path to personality file
fn personality_path(name: &str) -> PathBuf {
    PathBuf::from("data/ai_personalities").join(format!("{}.toml", name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default_personality() {
        let personality = load_personality("default").expect("Should load default personality");
        assert!(personality.behavior.aggression >= 0.0);
        assert!(personality.behavior.aggression <= 1.0);
    }

    #[test]
    fn test_personality_weights_bounded() {
        let personality = AiPersonality::default();
        assert!(personality.weights.attack_value >= 0.0);
        assert!(personality.weights.retreat_threshold >= 0.0);
        assert!(personality.weights.retreat_threshold <= 1.0);
    }

    #[test]
    fn test_default_personality_values() {
        let personality = AiPersonality::default();
        assert_eq!(personality.behavior.aggression, 0.5);
        assert!(!personality.difficulty.ignores_fog_of_war);
    }

    #[test]
    fn test_load_aggressive_personality() {
        let personality = load_personality("aggressive").expect("Should load aggressive personality");
        assert!(personality.behavior.aggression > 0.5, "Aggressive should have high aggression");
        assert!(personality.behavior.caution < 0.5, "Aggressive should have low caution");
    }
}
