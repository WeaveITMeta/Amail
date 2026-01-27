// =============================================================================
// Core Traits
// =============================================================================
// Table of Contents:
// 1. Agent - Base trait for all agents
// 2. Tool - Trait for agent tools
// 3. MessageHandler - Trait for A2A message handling
// 4. TaskProcessor - Trait for task processing
// 5. Supervisor - Trait for agent supervision
// =============================================================================

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::a2a::{A2AEnvelope, A2AMessage, Delegation, TaskResult};
use crate::agent::{AgentCapability, AgentConfig, AgentId, AgentStatus};
use crate::error::Result;
use crate::task::Task;

/// Base trait for all AI agents
#[async_trait]
pub trait Agent: Send + Sync {
    /// Get the agent's unique ID
    fn id(&self) -> AgentId;

    /// Get the agent's configuration
    fn config(&self) -> &AgentConfig;

    /// Get the agent's current status
    fn status(&self) -> AgentStatus;

    /// Get the agent's capabilities
    fn capabilities(&self) -> &[AgentCapability];

    /// Initialize the agent
    async fn initialize(&mut self) -> Result<()>;

    /// Process a task
    async fn process_task(&mut self, task: Task) -> Result<TaskResult>;

    /// Handle an A2A message
    async fn handle_message(&mut self, envelope: A2AEnvelope) -> Result<Option<A2AMessage>>;

    /// Shutdown the agent gracefully
    async fn shutdown(&mut self) -> Result<()>;

    /// Health check
    async fn health_check(&self) -> Result<bool> {
        Ok(self.status() == AgentStatus::Ready || self.status() == AgentStatus::Busy)
    }
}

/// Tool definition for agent tool calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// JSON Schema for parameters
    pub parameters: serde_json::Value,
    /// Whether the tool requires confirmation
    pub requires_confirmation: bool,
}

/// Tool call request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool arguments
    pub arguments: serde_json::Value,
}

/// Tool call result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool call ID
    pub tool_call_id: String,
    /// Success flag
    pub success: bool,
    /// Result content
    pub content: String,
    /// Structured data (optional)
    pub data: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error: Option<String>,
}

/// Trait for agent tools
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool definition
    fn definition(&self) -> ToolDefinition;

    /// Execute the tool
    async fn execute(&self, arguments: serde_json::Value) -> Result<ToolResult>;

    /// Validate arguments before execution
    fn validate_arguments(&self, arguments: &serde_json::Value) -> Result<()> {
        // Default: no validation
        let _ = arguments;
        Ok(())
    }
}

/// Trait for handling A2A messages
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle a delegation request
    async fn handle_delegation(&mut self, delegation: Delegation) -> Result<bool>;

    /// Handle a task result
    async fn handle_task_result(&mut self, result: TaskResult) -> Result<()>;

    /// Handle a capability query
    async fn handle_capability_query(
        &self,
        required: &[AgentCapability],
    ) -> Result<Vec<AgentCapability>>;
}

/// Trait for task processing
#[async_trait]
pub trait TaskProcessor: Send + Sync {
    /// Check if this processor can handle the task
    fn can_handle(&self, task: &Task) -> bool;

    /// Process the task
    async fn process(&mut self, task: Task) -> Result<TaskResult>;

    /// Estimate processing time in seconds
    fn estimate_duration(&self, task: &Task) -> Option<u64> {
        let _ = task;
        None
    }
}

/// Trait for agent supervision (Erlang/OTP-style)
#[async_trait]
pub trait Supervisor: Send + Sync {
    /// Get supervised agent IDs
    fn supervised_agents(&self) -> Vec<AgentId>;

    /// Handle agent failure
    async fn handle_failure(&mut self, agent_id: AgentId, error: &str) -> Result<SupervisionAction>;

    /// Handle agent termination
    async fn handle_termination(&mut self, agent_id: AgentId) -> Result<SupervisionAction>;

    /// Get supervision strategy
    fn strategy(&self) -> SupervisionStrategy;
}

/// Supervision action to take on failure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisionAction {
    /// Restart the failed agent
    Restart,
    /// Stop the failed agent
    Stop,
    /// Escalate to parent supervisor
    Escalate,
    /// Restart all supervised agents
    RestartAll,
    /// Stop all supervised agents
    StopAll,
}

/// Supervision strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SupervisionStrategy {
    /// Restart only the failed agent
    #[default]
    OneForOne,
    /// Restart all agents if one fails
    OneForAll,
    /// Restart agents started after the failed one
    RestForOne,
}

/// Trait for email processing
#[async_trait]
pub trait EmailProcessor: Send + Sync {
    /// Process an incoming email
    async fn process_email(&mut self, email: crate::email::EmailMessage) -> Result<Task>;

    /// Generate a reply
    async fn generate_reply(
        &mut self,
        original: &crate::email::EmailMessage,
        content: &str,
    ) -> Result<crate::email::EmailMessage>;
}
