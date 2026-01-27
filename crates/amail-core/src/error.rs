// =============================================================================
// Error Types for Amail
// =============================================================================
// Table of Contents:
// 1. AmailError enum - All error variants
// 2. Result type alias
// =============================================================================

use thiserror::Error;

/// Core error type for Amail operations
#[derive(Error, Debug)]
pub enum AmailError {
    // -------------------------------------------------------------------------
    // Email Errors
    // -------------------------------------------------------------------------
    #[error("Failed to parse email: {0}")]
    EmailParse(String),

    #[error("Invalid email address: {0}")]
    InvalidEmailAddress(String),

    #[error("Email not found: {0}")]
    EmailNotFound(String),

    // -------------------------------------------------------------------------
    // Agent Errors
    // -------------------------------------------------------------------------
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Agent unavailable: {0}")]
    AgentUnavailable(String),

    #[error("Agent capability not found: {0}")]
    CapabilityNotFound(String),

    #[error("Agent supervision error: {0}")]
    SupervisionError(String),

    // -------------------------------------------------------------------------
    // Task Errors
    // -------------------------------------------------------------------------
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),

    #[error("Task timeout: {0}")]
    TaskTimeout(String),

    #[error("Task delegation failed: {0}")]
    DelegationFailed(String),

    // -------------------------------------------------------------------------
    // A2A Protocol Errors
    // -------------------------------------------------------------------------
    #[error("A2A protocol error: {0}")]
    A2AProtocol(String),

    #[error("A2A routing error: {0}")]
    A2ARouting(String),

    // -------------------------------------------------------------------------
    // LLM Errors
    // -------------------------------------------------------------------------
    #[error("LLM inference error: {0}")]
    LlmInference(String),

    #[error("LLM tool call error: {0}")]
    LlmToolCall(String),

    #[error("LLM context overflow: {0}")]
    LlmContextOverflow(String),

    // -------------------------------------------------------------------------
    // MCP Errors
    // -------------------------------------------------------------------------
    #[error("MCP connection error: {0}")]
    McpConnection(String),

    #[error("MCP tool error: {0}")]
    McpTool(String),

    // -------------------------------------------------------------------------
    // Webhook Errors
    // -------------------------------------------------------------------------
    #[error("Webhook signature invalid")]
    WebhookSignatureInvalid,

    #[error("Webhook payload invalid: {0}")]
    WebhookPayloadInvalid(String),

    // -------------------------------------------------------------------------
    // Database Errors
    // -------------------------------------------------------------------------
    #[error("Database error: {0}")]
    Database(String),

    #[error("Queue error: {0}")]
    Queue(String),

    #[error("Network error: {0}")]
    Network(String),

    // -------------------------------------------------------------------------
    // Configuration Errors
    // -------------------------------------------------------------------------
    #[error("Configuration error: {0}")]
    Config(String),

    // -------------------------------------------------------------------------
    // Generic Errors
    // -------------------------------------------------------------------------
    #[error("Internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

/// Result type alias using AmailError
pub type Result<T> = std::result::Result<T, AmailError>;
