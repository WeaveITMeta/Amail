// =============================================================================
// Tool Definitions for Agent Tool Calling
// =============================================================================
// Table of Contents:
// 1. ToolDefinition - Tool schema
// 2. ToolCall - Tool invocation
// 3. ToolResult - Tool execution result
// 4. Built-in tools
// =============================================================================

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Tool definition for LLM function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name (function name)
    pub name: String,
    /// Tool description
    pub description: String,
    /// JSON Schema for parameters
    pub parameters: Value,
    /// Whether tool requires confirmation before execution
    #[serde(default)]
    pub requires_confirmation: bool,
}

impl ToolDefinition {
    /// Create a new tool definition
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
            requires_confirmation: false,
        }
    }

    /// Set parameters schema
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = parameters;
        self
    }

    /// Set requires confirmation
    pub fn with_confirmation(mut self, requires: bool) -> Self {
        self.requires_confirmation = requires;
        self
    }
}

/// Tool call from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique ID for this call
    pub id: String,
    /// Tool name
    pub name: String,
    /// Arguments as JSON
    pub arguments: Value,
}

impl ToolCall {
    /// Create a new tool call
    pub fn new(id: impl Into<String>, name: impl Into<String>, arguments: Value) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            arguments,
        }
    }

    /// Parse arguments as a specific type
    pub fn parse_arguments<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.arguments.clone())
    }
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool call ID this result is for
    pub tool_call_id: String,
    /// Whether execution succeeded
    pub success: bool,
    /// Result content (for LLM consumption)
    pub content: String,
    /// Structured data (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            success: true,
            content: content.into(),
            data: None,
            error: None,
        }
    }

    /// Create a successful result with data
    pub fn success_with_data(
        tool_call_id: impl Into<String>,
        content: impl Into<String>,
        data: Value,
    ) -> Self {
        Self {
            tool_call_id: tool_call_id.into(),
            success: true,
            content: content.into(),
            data: Some(data),
            error: None,
        }
    }

    /// Create a failed result
    pub fn failure(tool_call_id: impl Into<String>, error: impl Into<String>) -> Self {
        let error_str = error.into();
        Self {
            tool_call_id: tool_call_id.into(),
            success: false,
            content: format!("Error: {}", error_str),
            data: None,
            error: Some(error_str),
        }
    }
}

// =============================================================================
// Built-in Tool Definitions
// =============================================================================

/// Create email reply tool
pub fn email_reply_tool() -> ToolDefinition {
    ToolDefinition::new("send_email_reply", "Send an email reply to the sender")
        .with_parameters(serde_json::json!({
            "type": "object",
            "properties": {
                "to": {
                    "type": "string",
                    "description": "Recipient email address"
                },
                "subject": {
                    "type": "string",
                    "description": "Email subject"
                },
                "body": {
                    "type": "string",
                    "description": "Email body content"
                },
                "cc": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "CC recipients"
                }
            },
            "required": ["to", "subject", "body"]
        }))
        .with_confirmation(true)
}

/// Delegate task tool
pub fn delegate_task_tool() -> ToolDefinition {
    ToolDefinition::new("delegate_task", "Delegate a task to another agent")
        .with_parameters(serde_json::json!({
            "type": "object",
            "properties": {
                "target_role": {
                    "type": "string",
                    "description": "Target agent role (e.g., 'lawyer', 'accountant')"
                },
                "task_description": {
                    "type": "string",
                    "description": "Description of the task to delegate"
                },
                "priority": {
                    "type": "string",
                    "enum": ["low", "normal", "high", "urgent"],
                    "description": "Task priority"
                },
                "context": {
                    "type": "object",
                    "description": "Additional context for the task"
                }
            },
            "required": ["target_role", "task_description"]
        }))
}

/// Request human review tool
pub fn request_human_review_tool() -> ToolDefinition {
    ToolDefinition::new("request_human_review", "Escalate to human for review")
        .with_parameters(serde_json::json!({
            "type": "object",
            "properties": {
                "reason": {
                    "type": "string",
                    "description": "Reason for requesting human review"
                },
                "urgency": {
                    "type": "string",
                    "enum": ["low", "medium", "high"],
                    "description": "Urgency level"
                },
                "draft_response": {
                    "type": "string",
                    "description": "Optional draft response for human to review"
                }
            },
            "required": ["reason"]
        }))
}

/// Search knowledge base tool
pub fn search_knowledge_tool() -> ToolDefinition {
    ToolDefinition::new("search_knowledge", "Search the knowledge base for information")
        .with_parameters(serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "category": {
                    "type": "string",
                    "description": "Optional category filter"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum results to return",
                    "default": 5
                }
            },
            "required": ["query"]
        }))
}

/// MCP tool call
pub fn mcp_tool() -> ToolDefinition {
    ToolDefinition::new("mcp_call", "Call an MCP server tool")
        .with_parameters(serde_json::json!({
            "type": "object",
            "properties": {
                "server": {
                    "type": "string",
                    "description": "MCP server name"
                },
                "tool": {
                    "type": "string",
                    "description": "Tool name on the MCP server"
                },
                "arguments": {
                    "type": "object",
                    "description": "Tool arguments"
                }
            },
            "required": ["server", "tool"]
        }))
}

/// Get all built-in tools
pub fn builtin_tools() -> Vec<ToolDefinition> {
    vec![
        email_reply_tool(),
        delegate_task_tool(),
        request_human_review_tool(),
        search_knowledge_tool(),
        mcp_tool(),
    ]
}
