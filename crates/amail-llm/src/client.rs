// =============================================================================
// LLM Client - Main Interface for LLM Inference
// =============================================================================
// Table of Contents:
// 1. LlmClient struct
// 2. ChatMessage and ChatResponse types
// 3. Chat completion methods
// 4. Tool calling integration
// =============================================================================

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::ollama::OllamaClient;
use crate::tools::{ToolCall, ToolDefinition};

/// LLM client for inference
#[derive(Clone)]
pub struct LlmClient {
    /// Model name
    pub model: String,
    /// Base URL for Ollama
    pub base_url: String,
    /// Ollama client
    ollama: OllamaClient,
    /// Available tools
    tools: Vec<ToolDefinition>,
    /// Temperature (0.0 - 1.0)
    temperature: f32,
    /// Max tokens
    max_tokens: u32,
}

impl LlmClient {
    /// Create a new LLM client
    pub fn new(model: &str) -> Result<Self> {
        let base_url = "http://localhost:11434".to_string();
        let ollama = OllamaClient::new(&base_url)?;

        Ok(Self {
            model: model.to_string(),
            base_url,
            ollama,
            tools: Vec::new(),
            temperature: 0.7,
            max_tokens: 4096,
        })
    }

    /// Create with custom Ollama URL
    pub fn with_url(model: &str, url: &str) -> Result<Self> {
        let ollama = OllamaClient::new(url)?;

        Ok(Self {
            model: model.to_string(),
            base_url: url.to_string(),
            ollama,
            tools: Vec::new(),
            temperature: 0.7,
            max_tokens: 4096,
        })
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature.clamp(0.0, 1.0);
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// Register tools for function calling
    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = tools;
        self
    }

    /// Simple chat completion
    pub async fn chat(&self, system_prompt: &str, user_message: &str) -> Result<ChatResponse> {
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: system_prompt.to_string(),
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: user_message.to_string(),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        self.chat_completion(messages).await
    }

    /// Chat completion with message history
    pub async fn chat_completion(&self, messages: Vec<ChatMessage>) -> Result<ChatResponse> {
        let start = std::time::Instant::now();

        let response = self
            .ollama
            .chat(&self.model, messages, self.temperature, self.max_tokens)
            .await?;

        let duration_ms = start.elapsed().as_millis() as u64;

        debug!(
            "LLM response in {}ms: {} tokens",
            duration_ms, response.total_tokens
        );

        Ok(ChatResponse {
            content: response.content,
            tool_calls: response.tool_calls,
            prompt_tokens: response.prompt_tokens,
            completion_tokens: response.completion_tokens,
            total_tokens: response.total_tokens,
            duration_ms,
            model: self.model.clone(),
        })
    }

    /// Chat with tool calling support
    pub async fn chat_with_tools(
        &self,
        system_prompt: &str,
        user_message: &str,
        tools: &[ToolDefinition],
    ) -> Result<ChatResponse> {
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: self.build_tool_system_prompt(system_prompt, tools),
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: user_message.to_string(),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        self.chat_completion(messages).await
    }

    /// Build system prompt with tool definitions
    fn build_tool_system_prompt(&self, base_prompt: &str, tools: &[ToolDefinition]) -> String {
        if tools.is_empty() {
            return base_prompt.to_string();
        }

        let tool_descriptions: Vec<String> = tools
            .iter()
            .map(|t| {
                format!(
                    "- **{}**: {}\n  Parameters: {}",
                    t.name,
                    t.description,
                    serde_json::to_string_pretty(&t.parameters).unwrap_or_default()
                )
            })
            .collect();

        format!(
            r#"{}

## Available Tools

You have access to the following tools. To use a tool, respond with a JSON object in this format:
```json
{{"tool": "tool_name", "arguments": {{...}}}}
```

Tools:
{}

If you don't need to use a tool, respond normally."#,
            base_prompt,
            tool_descriptions.join("\n\n")
        )
    }

    /// Check if model is available
    pub async fn is_available(&self) -> bool {
        self.ollama.is_model_available(&self.model).await
    }

    /// List available models
    pub async fn list_models(&self) -> Result<Vec<String>> {
        self.ollama.list_models().await
    }
}

/// Message role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Chat response
#[derive(Debug, Clone)]
pub struct ChatResponse {
    /// Response content
    pub content: String,
    /// Tool calls (if any)
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Prompt tokens used
    pub prompt_tokens: u32,
    /// Completion tokens used
    pub completion_tokens: u32,
    /// Total tokens used
    pub total_tokens: u32,
    /// Response time in milliseconds
    pub duration_ms: u64,
    /// Model used
    pub model: String,
}

impl ChatResponse {
    /// Check if response contains tool calls
    pub fn has_tool_calls(&self) -> bool {
        self.tool_calls.as_ref().map(|t| !t.is_empty()).unwrap_or(false)
    }

    /// Get tool calls
    pub fn get_tool_calls(&self) -> &[ToolCall] {
        self.tool_calls.as_deref().unwrap_or(&[])
    }
}
