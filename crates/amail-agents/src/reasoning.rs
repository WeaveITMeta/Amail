// =============================================================================
// Agent Reasoning Engine - LLM-Powered Task Processing
// =============================================================================
// Table of Contents:
// 1. ReasoningEngine struct
// 2. Task processing logic
// 3. Tool execution
// 4. Response parsing
// =============================================================================

use amail_core::{
    agent::AgentConfig,
    task::Task,
    AmailError,
};
use amail_llm::{ChatMessage, ChatResponse, LlmClient, MessageRole, ToolCall, ToolResult};
use amail_mcp::McpClient;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::prompts::get_system_prompt;

/// Reasoning engine for agent task processing
pub struct ReasoningEngine {
    /// LLM client
    llm: LlmClient,
    /// Agent configuration
    config: AgentConfig,
    /// System prompt for this agent
    system_prompt: String,
    /// Max reasoning iterations (prevent infinite loops)
    max_iterations: u32,
    /// MCP client for external tool access
    mcp: Option<Arc<McpClient>>,
}

impl ReasoningEngine {
    /// Create a new reasoning engine for an agent
    pub fn new(config: AgentConfig, llm: LlmClient) -> Self {
        let system_prompt = get_system_prompt(&config.role);

        Self {
            llm,
            config,
            system_prompt,
            max_iterations: 5,
            mcp: None,
        }
    }

    /// Create with MCP client for extended tool capabilities
    pub fn with_mcp(config: AgentConfig, llm: LlmClient, mcp: Arc<McpClient>) -> Self {
        let system_prompt = get_system_prompt(&config.role);

        Self {
            llm,
            config,
            system_prompt,
            max_iterations: 5,
            mcp: Some(mcp),
        }
    }

    /// Set MCP client
    pub fn set_mcp(&mut self, mcp: Arc<McpClient>) {
        self.mcp = Some(mcp);
    }

    /// Process a task and return the result
    pub async fn process_task(&self, task: &Task) -> Result<ReasoningResult, AmailError> {
        info!(
            "Agent {:?} processing task: {}",
            self.config.role, task.id
        );

        // Build the user message from task context
        let user_message = self.build_task_message(task);

        // Start reasoning loop
        let mut messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: self.system_prompt.clone(),
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: user_message,
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let mut iterations = 0;
        let mut final_result: Option<ReasoningResult> = None;

        while iterations < self.max_iterations {
            iterations += 1;
            debug!("Reasoning iteration {} for task {}", iterations, task.id);

            // Get LLM response
            let response = self
                .llm
                .chat_completion(messages.clone())
                .await
                .map_err(|e| AmailError::LlmInference(e.to_string()))?;

            // Parse the response
            let parsed = self.parse_response(&response)?;

            match parsed {
                ParsedResponse::ToolCall(tool_call) => {
                    // Execute the tool
                    let tool_result = self.execute_tool(&tool_call).await?;

                    // Add assistant message with tool call
                    messages.push(ChatMessage {
                        role: MessageRole::Assistant,
                        content: response.content.clone(),
                        tool_calls: Some(vec![tool_call.clone()]),
                        tool_call_id: None,
                    });

                    // Add tool result
                    messages.push(ChatMessage {
                        role: MessageRole::Tool,
                        content: tool_result.content.clone(),
                        tool_calls: None,
                        tool_call_id: Some(tool_call.id.clone()),
                    });

                    // Check if this was a terminal action
                    if self.is_terminal_action(&tool_call) {
                        final_result = Some(ReasoningResult {
                            action: ReasoningAction::ToolExecuted(tool_call),
                            response_text: tool_result.content,
                            iterations,
                            token_usage: TokenUsage {
                                prompt: response.prompt_tokens,
                                completion: response.completion_tokens,
                                total: response.total_tokens,
                            },
                        });
                        break;
                    }
                }
                ParsedResponse::Delegation(delegation) => {
                    final_result = Some(ReasoningResult {
                        action: ReasoningAction::Delegate(delegation),
                        response_text: response.content,
                        iterations,
                        token_usage: TokenUsage {
                            prompt: response.prompt_tokens,
                            completion: response.completion_tokens,
                            total: response.total_tokens,
                        },
                    });
                    break;
                }
                ParsedResponse::Escalation(reason) => {
                    final_result = Some(ReasoningResult {
                        action: ReasoningAction::Escalate(reason),
                        response_text: response.content,
                        iterations,
                        token_usage: TokenUsage {
                            prompt: response.prompt_tokens,
                            completion: response.completion_tokens,
                            total: response.total_tokens,
                        },
                    });
                    break;
                }
                ParsedResponse::TextResponse(text) => {
                    final_result = Some(ReasoningResult {
                        action: ReasoningAction::Respond(text.clone()),
                        response_text: text,
                        iterations,
                        token_usage: TokenUsage {
                            prompt: response.prompt_tokens,
                            completion: response.completion_tokens,
                            total: response.total_tokens,
                        },
                    });
                    break;
                }
            }
        }

        final_result.ok_or_else(|| {
            AmailError::Internal(format!(
                "Reasoning failed after {} iterations",
                self.max_iterations
            ))
        })
    }

    /// Build the user message from task context
    fn build_task_message(&self, task: &Task) -> String {
        let mut message = String::new();

        message.push_str("## Task\n\n");
        message.push_str(&format!("**Task ID**: {}\n", task.id));
        message.push_str(&format!("**Type**: {:?}\n", task.task_type));
        message.push_str(&format!("**Priority**: {:?}\n", task.priority));
        message.push_str(&format!("**Description**: {}\n\n", task.description));

        // Add email context if present
        if let Some(ref email) = task.context.source_email {
            message.push_str("## Email\n\n");
            message.push_str(&format!("**From**: {}\n", email.metadata.from.address));
            message.push_str(&format!(
                "**To**: {}\n",
                email
                    .metadata
                    .to
                    .iter()
                    .map(|a| a.address.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            message.push_str(&format!("**Subject**: {}\n", email.metadata.subject));
            message.push_str(&format!(
                "**Received**: {}\n\n",
                email.metadata.received_at
            ));

            if let Some(ref body) = email.text_body {
                message.push_str("**Body**:\n```\n");
                // Truncate very long bodies
                if body.len() > 4000 {
                    message.push_str(&body[..4000]);
                    message.push_str("\n... [truncated]\n");
                } else {
                    message.push_str(body);
                }
                message.push_str("\n```\n\n");
            }

            if !email.attachments.is_empty() {
                message.push_str("**Attachments**:\n");
                for att in &email.attachments {
                    message.push_str(&format!(
                        "- {} ({}, {} bytes)\n",
                        att.filename, att.content_type, att.size
                    ));
                }
                message.push_str("\n");
            }
        }

        // Add conversation history
        if !task.context.conversation_history.is_empty() {
            message.push_str("## Conversation History\n\n");
            for msg in &task.context.conversation_history {
                message.push_str(&format!("**{}**: {}\n\n", msg.role, msg.content));
            }
        }

        // Add metadata if present
        if !task.context.metadata.is_empty() {
            message.push_str("## Additional Context\n\n");
            for (key, value) in &task.context.metadata {
                message.push_str(&format!("**{}**: {}\n\n", key, value));
            }
        }

        message.push_str("## Instructions\n\n");
        message.push_str("Process this task according to your role. ");
        message.push_str("Use tools when appropriate. ");
        message.push_str("If you cannot handle this task, delegate or escalate.\n");

        message
    }

    /// Parse LLM response into structured action
    fn parse_response(&self, response: &ChatResponse) -> Result<ParsedResponse, AmailError> {
        let content = response.content.trim();

        // Check for tool call JSON
        if let Some(tool_call) = self.extract_tool_call(content) {
            // Check if it's a delegation
            if tool_call.name == "delegate_task" {
                if let Ok(delegation) = serde_json::from_value::<DelegationRequest>(tool_call.arguments.clone()) {
                    return Ok(ParsedResponse::Delegation(delegation));
                }
            }

            // Check if it's an escalation
            if tool_call.name == "request_human_review" {
                let reason = tool_call
                    .arguments
                    .get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Escalated by agent")
                    .to_string();
                return Ok(ParsedResponse::Escalation(reason));
            }

            return Ok(ParsedResponse::ToolCall(tool_call));
        }

        // Plain text response
        Ok(ParsedResponse::TextResponse(content.to_string()))
    }

    /// Extract tool call from response content
    fn extract_tool_call(&self, content: &str) -> Option<ToolCall> {
        // Look for JSON block
        let json_start = content.find('{');
        let json_end = content.rfind('}');

        if let (Some(start), Some(end)) = (json_start, json_end) {
            if end > start {
                let json_str = &content[start..=end];
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(tool_name) = parsed.get("tool").and_then(|v| v.as_str()) {
                        let arguments = parsed
                            .get("arguments")
                            .cloned()
                            .unwrap_or(serde_json::json!({}));

                        return Some(ToolCall {
                            id: uuid::Uuid::new_v4().to_string(),
                            name: tool_name.to_string(),
                            arguments,
                        });
                    }
                }
            }
        }

        None
    }

    /// Execute a tool call
    async fn execute_tool(&self, tool_call: &ToolCall) -> Result<ToolResult, AmailError> {
        info!("Executing tool: {}", tool_call.name);

        match tool_call.name.as_str() {
            "send_email_reply" => {
                // TODO: Actually send email via Stalwart
                let to = tool_call.arguments.get("to").and_then(|v| v.as_str()).unwrap_or("");
                let subject = tool_call.arguments.get("subject").and_then(|v| v.as_str()).unwrap_or("");
                
                Ok(ToolResult::success(
                    &tool_call.id,
                    format!("Email queued for delivery to {} with subject: {}", to, subject),
                ))
            }
            "delegate_task" => {
                let target = tool_call.arguments.get("target_role").and_then(|v| v.as_str()).unwrap_or("clerk");
                
                Ok(ToolResult::success(
                    &tool_call.id,
                    format!("Task delegated to {} agent", target),
                ))
            }
            "request_human_review" => {
                let reason = tool_call.arguments.get("reason").and_then(|v| v.as_str()).unwrap_or("No reason provided");
                
                Ok(ToolResult::success(
                    &tool_call.id,
                    format!("Escalated to human review: {}", reason),
                ))
            }
            "search_knowledge" => {
                let query = tool_call.arguments.get("query").and_then(|v| v.as_str()).unwrap_or("");
                
                // TODO: Implement actual knowledge base search
                Ok(ToolResult::success(
                    &tool_call.id,
                    format!("Knowledge search for '{}': No results found (knowledge base not configured)", query),
                ))
            }
            "mcp_call" => {
                let server = tool_call.arguments.get("server").and_then(|v| v.as_str()).unwrap_or("");
                let tool = tool_call.arguments.get("tool").and_then(|v| v.as_str()).unwrap_or("");
                let args = tool_call.arguments.get("arguments").cloned().unwrap_or(serde_json::json!({}));
                
                if let Some(mcp) = &self.mcp {
                    match mcp.call_tool(server, tool, args).await {
                        Ok(result) => {
                            let text = result.all_text();
                            if result.is_error {
                                Ok(ToolResult::failure(&tool_call.id, text))
                            } else {
                                Ok(ToolResult::success(&tool_call.id, text))
                            }
                        }
                        Err(e) => Ok(ToolResult::failure(&tool_call.id, format!("MCP error: {}", e))),
                    }
                } else {
                    Ok(ToolResult::failure(&tool_call.id, "MCP client not configured"))
                }
            }
            _ => {
                // Check if it's an MCP tool (mcp_server_toolname format)
                if tool_call.name.starts_with("mcp_") {
                    if let Some(mcp) = &self.mcp {
                        match mcp.call_tool_by_name(&tool_call.name, tool_call.arguments.clone()).await {
                            Ok(result) => {
                                let text = result.all_text();
                                if result.is_error {
                                    return Ok(ToolResult::failure(&tool_call.id, text));
                                } else {
                                    return Ok(ToolResult::success(&tool_call.id, text));
                                }
                            }
                            Err(e) => return Ok(ToolResult::failure(&tool_call.id, format!("MCP error: {}", e))),
                        }
                    } else {
                        return Ok(ToolResult::failure(&tool_call.id, "MCP client not configured"));
                    }
                }
                
                warn!("Unknown tool: {}", tool_call.name);
                Ok(ToolResult::failure(
                    &tool_call.id,
                    format!("Unknown tool: {}", tool_call.name),
                ))
            }
        }
    }

    /// Check if a tool call is a terminal action
    fn is_terminal_action(&self, tool_call: &ToolCall) -> bool {
        matches!(
            tool_call.name.as_str(),
            "send_email_reply" | "delegate_task" | "request_human_review"
        )
    }
}

// =============================================================================
// Types
// =============================================================================

/// Result of agent reasoning
#[derive(Debug, Clone)]
pub struct ReasoningResult {
    /// Action taken
    pub action: ReasoningAction,
    /// Full response text
    pub response_text: String,
    /// Number of reasoning iterations
    pub iterations: u32,
    /// Token usage
    pub token_usage: TokenUsage,
}

/// Action determined by reasoning
#[derive(Debug, Clone)]
pub enum ReasoningAction {
    /// Send a response (email or message)
    Respond(String),
    /// Delegate to another agent
    Delegate(DelegationRequest),
    /// Escalate to human review
    Escalate(String),
    /// Tool was executed
    ToolExecuted(ToolCall),
}

/// Token usage statistics
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub prompt: u32,
    pub completion: u32,
    pub total: u32,
}

/// Parsed response from LLM
enum ParsedResponse {
    ToolCall(ToolCall),
    Delegation(DelegationRequest),
    Escalation(String),
    TextResponse(String),
}

/// Delegation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationRequest {
    pub target_role: String,
    pub task_description: String,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub context: Option<serde_json::Value>,
}
