// =============================================================================
// AMAIL-LLM - LLM Integration (Ollama, Mistral, Tool Calling)
// =============================================================================
// Table of Contents:
// 1. LlmClient - Main client for LLM inference
// 2. ChatResponse - Response from LLM
// 3. Tool calling support
// 4. Ollama integration
// =============================================================================

pub mod client;
pub mod tools;
pub mod ollama;
pub mod pool;

pub use client::{LlmClient, ChatResponse, ChatMessage, MessageRole};
pub use tools::{ToolDefinition, ToolCall, ToolResult};
pub use pool::{LlmPool, InstanceHealth};
