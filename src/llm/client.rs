//! Async LLM client for command parsing
//!
//! This is a model-agnostic HTTP client for calling LLM APIs.
//! Supports both Anthropic and OpenAI-compatible APIs (DeepSeek, etc).
//! Key principle: LLMs parse COMMANDS only, not entity behavior
//! (entities use rules-based systems for emergent behavior).

use crate::core::error::{ArcError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// API format type
#[derive(Debug, Clone, PartialEq)]
pub enum ApiFormat {
    Anthropic,
    OpenAI,
}

/// Async LLM client for making API calls
pub struct LlmClient {
    client: Client,
    api_key: String,
    api_url: String,
    model: String,
    api_format: ApiFormat,
}

impl LlmClient {
    /// Create a new LLM client with explicit configuration
    pub fn new(api_key: String, api_url: String, model: String) -> Self {
        let api_format = Self::detect_api_format(&api_url);
        Self {
            client: Client::new(),
            api_key,
            api_url,
            model,
            api_format,
        }
    }

    /// Detect API format from URL
    fn detect_api_format(url: &str) -> ApiFormat {
        if url.contains("anthropic.com") {
            ApiFormat::Anthropic
        } else {
            // DeepSeek, OpenAI, and other compatible APIs use OpenAI format
            ApiFormat::OpenAI
        }
    }

    /// Create a client from environment variables
    ///
    /// Required: LLM_API_KEY
    /// Optional: LLM_API_URL (defaults to Anthropic API)
    /// Optional: LLM_MODEL (defaults to claude-3-haiku-20240307)
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("LLM_API_KEY")
            .map_err(|_| ArcError::LlmError("LLM_API_KEY not set".into()))?;
        let api_url = std::env::var("LLM_API_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com/v1/messages".into());
        let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "claude-3-haiku-20240307".into());

        Ok(Self::new(api_key, api_url, model))
    }

    /// Send a completion request to the LLM
    ///
    /// # Arguments
    /// * `system` - System prompt providing context and instructions
    /// * `user` - User message/query to process
    ///
    /// # Returns
    /// The LLM's text response
    pub async fn complete(&self, system: &str, user: &str) -> Result<String> {
        match self.api_format {
            ApiFormat::Anthropic => self.complete_anthropic(system, user).await,
            ApiFormat::OpenAI => self.complete_openai(system, user).await,
        }
    }

    async fn complete_anthropic(&self, system: &str, user: &str) -> Result<String> {
        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: 8192,
            system: system.into(),
            messages: vec![Message {
                role: "user".into(),
                content: user.into(),
            }],
        };

        let response = self
            .client
            .post(&self.api_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ArcError::LlmError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ArcError::LlmError(format!("API error: {}", error_text)));
        }

        let completion: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| ArcError::LlmError(e.to_string()))?;

        completion
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| ArcError::LlmError("Empty response".into()))
    }

    async fn complete_openai(&self, system: &str, user: &str) -> Result<String> {
        // Model-specific max_tokens limits
        let max_tokens = if self.model.contains("reasoner") {
            32768 // DeepSeek R1 supports up to 32K-64K output
        } else {
            8192 // DeepSeek V3/chat max is 8192
        };

        let request = OpenAIRequest {
            model: self.model.clone(),
            max_tokens,
            messages: vec![
                Message {
                    role: "system".into(),
                    content: system.into(),
                },
                Message {
                    role: "user".into(),
                    content: user.into(),
                },
            ],
        };

        let response = self
            .client
            .post(&self.api_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ArcError::LlmError(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(ArcError::LlmError(format!("API error: {}", error_text)));
        }

        let completion: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| ArcError::LlmError(e.to_string()))?;

        completion
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| ArcError::LlmError("Empty response".into()))
    }
}

// Anthropic API format
#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    system: String,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

// OpenAI-compatible API format (DeepSeek, OpenAI, etc.)
#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: String,
}

// Shared
#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = LlmClient::new(
            "test-key".into(),
            "https://api.example.com".into(),
            "test-model".into(),
        );
        assert_eq!(client.api_key, "test-key");
        assert_eq!(client.api_url, "https://api.example.com");
        assert_eq!(client.model, "test-model");
    }

    #[test]
    fn test_from_env_missing_key() {
        // Temporarily clear the env var if set
        let result = LlmClient::from_env();
        // Should fail if LLM_API_KEY is not set
        if std::env::var("LLM_API_KEY").is_err() {
            assert!(result.is_err());
        }
    }
}
