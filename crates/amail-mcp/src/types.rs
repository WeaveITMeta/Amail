// =============================================================================
// MCP Types
// =============================================================================
// Table of Contents:
// 1. McpTool - Tool definition
// 2. McpResource - Resource definition
// 3. McpToolResult - Tool execution result
// =============================================================================

use serde::{Deserialize, Serialize};

use crate::client::McpContent;

/// MCP Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// Tool name
    pub name: String,
    /// Tool description
    #[serde(default)]
    pub description: Option<String>,
    /// Input schema (JSON Schema)
    #[serde(default, rename = "inputSchema")]
    pub input_schema: Option<serde_json::Value>,
}

/// MCP Resource definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResource {
    /// Resource URI
    pub uri: String,
    /// Resource name
    pub name: String,
    /// Resource description
    #[serde(default)]
    pub description: Option<String>,
    /// MIME type
    #[serde(default, rename = "mimeType")]
    pub mime_type: Option<String>,
}

/// MCP Tool execution result
#[derive(Debug, Clone)]
pub struct McpToolResult {
    /// Whether the call succeeded
    pub success: bool,
    /// Result content
    pub content: Vec<McpContent>,
    /// Whether this is an error response
    pub is_error: bool,
}

impl McpToolResult {
    /// Get text content from result
    pub fn text(&self) -> Option<String> {
        self.content
            .iter()
            .filter_map(|c| match c {
                McpContent::Text { text } => Some(text.clone()),
                _ => None,
            })
            .next()
    }

    /// Get all text content concatenated
    pub fn all_text(&self) -> String {
        self.content
            .iter()
            .filter_map(|c| match c {
                McpContent::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
