// =============================================================================
// Ollama Client - Direct Ollama API Integration
// =============================================================================
// Table of Contents:
// 1. OllamaClient struct
// 2. API request/response types
// 3. Chat and generate methods
// =============================================================================

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};

use crate::client::{ChatMessage, MessageRole};
use crate::tools::ToolCall;

/// Ollama API client
#[derive(Clone)]
pub struct OllamaClient {
    /// HTTP client
    client: Client,
    /// Base URL
    base_url: String,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new(base_url: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    /// Chat completion
    pub async fn chat(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
        temperature: f32,
        max_tokens: u32,
    ) -> Result<OllamaResponse> {
        let ollama_messages: Vec<OllamaMessage> = messages
            .into_iter()
            .map(|m| OllamaMessage {
                role: match m.role {
                    MessageRole::System => "system".to_string(),
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "assistant".to_string(),
                    MessageRole::Tool => "tool".to_string(),
                },
                content: m.content,
            })
            .collect();

        let request = OllamaChatRequest {
            model: model.to_string(),
            messages: ollama_messages,
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(temperature),
                num_predict: Some(max_tokens as i32),
            }),
        };

        let url = format!("{}/api/chat", self.base_url);
        debug!("Sending chat request to {}", url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to Ollama: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Ollama error {}: {}", status, body);
            return Err(anyhow!("Ollama API error {}: {}", status, body));
        }

        let chat_response: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Ollama response: {}", e))?;

        Ok(OllamaResponse {
            content: chat_response.message.content,
            tool_calls: None, // Ollama doesn't natively support tool calls yet
            prompt_tokens: chat_response.prompt_eval_count.unwrap_or(0),
            completion_tokens: chat_response.eval_count.unwrap_or(0),
            total_tokens: chat_response.prompt_eval_count.unwrap_or(0)
                + chat_response.eval_count.unwrap_or(0),
        })
    }

    /// Generate completion (non-chat)
    pub async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let request = OllamaGenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options: None,
        };

        let url = format!("{}/api/generate", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to Ollama: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Ollama API error {}: {}", status, body));
        }

        let gen_response: OllamaGenerateResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Ollama response: {}", e))?;

        Ok(gen_response.response)
    }

    /// Check if a model is available
    pub async fn is_model_available(&self, model: &str) -> bool {
        match self.list_models().await {
            Ok(models) => models.iter().any(|m| m.contains(model)),
            Err(_) => false,
        }
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to list Ollama models: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to list models: {}", response.status()));
        }

        let tags: OllamaTagsResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse tags response: {}", e))?;

        Ok(tags.models.into_iter().map(|m| m.name).collect())
    }

    /// Pull a model
    pub async fn pull_model(&self, model: &str) -> Result<()> {
        let url = format!("{}/api/pull", self.base_url);

        let request = serde_json::json!({
            "name": model,
            "stream": false
        });

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to pull model: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to pull model: {}", response.status()));
        }

        Ok(())
    }
}

/// Internal Ollama response
pub struct OllamaResponse {
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// =============================================================================
// Ollama API Types
// =============================================================================

#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: OllamaResponseMessage,
    #[serde(default)]
    prompt_eval_count: Option<u32>,
    #[serde(default)]
    eval_count: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponseMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
}
