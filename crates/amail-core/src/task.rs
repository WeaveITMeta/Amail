// =============================================================================
// Task Types
// =============================================================================
// Table of Contents:
// 1. TaskId - Unique task identifier
// 2. TaskStatus - Task execution state
// 3. TaskPriority - Task urgency level
// 4. TaskContext - Execution context
// 5. Task - Complete task representation
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::agent::AgentId;
use crate::email::EmailMessage;

/// Unique task identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

impl TaskId {
    /// Create a new random task ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "task-{}", self.0)
    }
}

/// Task execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// Task is queued, waiting to be picked up
    Queued,
    /// Task is assigned to an agent
    Assigned { agent_id: AgentId },
    /// Task is being processed
    InProgress {
        agent_id: AgentId,
        started_at: DateTime<Utc>,
    },
    /// Task is waiting for tool call results
    WaitingForTool {
        agent_id: AgentId,
        tool_name: String,
    },
    /// Task is waiting for human review
    WaitingForHuman {
        agent_id: AgentId,
        reason: String,
    },
    /// Task is delegated to another agent
    Delegated {
        from_agent: AgentId,
        to_agent: AgentId,
    },
    /// Task completed successfully
    Completed {
        agent_id: AgentId,
        completed_at: DateTime<Utc>,
    },
    /// Task failed
    Failed {
        agent_id: Option<AgentId>,
        error: String,
        failed_at: DateTime<Utc>,
    },
    /// Task was cancelled
    Cancelled {
        reason: String,
        cancelled_at: DateTime<Utc>,
    },
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Queued
    }
}

/// Task priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    /// Low priority - can wait
    Low = 0,
    /// Normal priority - standard processing
    #[default]
    Normal = 1,
    /// High priority - process soon
    High = 2,
    /// Urgent - process immediately
    Urgent = 3,
    /// Critical - drop everything
    Critical = 4,
}

impl TaskPriority {
    /// Get string representation for queue keys
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::Low => "low",
            TaskPriority::Normal => "normal",
            TaskPriority::High => "high",
            TaskPriority::Urgent => "urgent",
            TaskPriority::Critical => "critical",
        }
    }
}

/// Task execution context
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskContext {
    /// Original email that triggered this task
    pub source_email: Option<EmailMessage>,
    /// Parent task ID (for subtasks)
    pub parent_task_id: Option<TaskId>,
    /// Child task IDs (subtasks spawned)
    pub child_task_ids: Vec<TaskId>,
    /// Conversation history for LLM context
    pub conversation_history: Vec<ConversationMessage>,
    /// Tool call history
    pub tool_calls: Vec<ToolCallRecord>,
    /// Arbitrary metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// User ID (for multi-tenant)
    pub user_id: Option<String>,
    /// Tenant/organization ID
    pub tenant_id: Option<String>,
}

/// A message in the conversation history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    /// Role: system, user, assistant, tool
    pub role: String,
    /// Message content
    pub content: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Tool call ID (if role is tool)
    pub tool_call_id: Option<String>,
}

/// Record of a tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    /// Tool call ID
    pub id: String,
    /// Tool name
    pub name: String,
    /// Tool arguments (JSON)
    pub arguments: serde_json::Value,
    /// Tool result (JSON)
    pub result: Option<serde_json::Value>,
    /// Error if failed
    pub error: Option<String>,
    /// Call timestamp
    pub called_at: DateTime<Utc>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

/// Complete task representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task ID
    pub id: TaskId,
    /// Task type/action
    pub task_type: TaskType,
    /// Human-readable description
    pub description: String,
    /// Current status
    pub status: TaskStatus,
    /// Priority level
    pub priority: TaskPriority,
    /// Execution context
    pub context: TaskContext,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Deadline (optional)
    pub deadline: Option<DateTime<Utc>>,
    /// Retry count
    pub retry_count: u32,
    /// Maximum retries allowed
    pub max_retries: u32,
    /// Result data (when completed)
    pub result: Option<TaskResult>,
}

/// Task type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    /// Process incoming email
    ProcessEmail,
    /// Compose and send email reply
    ComposeReply,
    /// Delegate to another agent
    Delegate,
    /// Execute tool call
    ToolCall,
    /// Request human review
    HumanReview,
    /// Compliance check
    ComplianceCheck,
    /// Legal review
    LegalReview,
    /// Financial processing
    FinancialProcessing,
    /// Research task
    Research,
    /// Content generation
    ContentGeneration,
    /// Translation
    Translation,
    /// Custom task type
    Custom(String),
}

/// Task result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Success flag
    pub success: bool,
    /// Result data
    pub data: serde_json::Value,
    /// Output artifacts (e.g., generated emails, documents)
    pub artifacts: Vec<TaskArtifact>,
    /// Summary for logging
    pub summary: String,
}

/// Task artifact (output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskArtifact {
    /// Artifact type
    pub artifact_type: String,
    /// Artifact name
    pub name: String,
    /// Artifact content
    pub content: serde_json::Value,
}

impl Task {
    /// Create a new task
    pub fn new(task_type: TaskType, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: TaskId::new(),
            task_type,
            description: description.into(),
            status: TaskStatus::Queued,
            priority: TaskPriority::Normal,
            context: TaskContext::default(),
            created_at: now,
            updated_at: now,
            deadline: None,
            retry_count: 0,
            max_retries: 3,
            result: None,
        }
    }

    /// Create a task from an email
    pub fn from_email(email: EmailMessage) -> Self {
        let description = format!(
            "Process email from {} - {}",
            email.metadata.from.address, email.metadata.subject
        );
        let mut task = Self::new(TaskType::ProcessEmail, description);
        task.context.source_email = Some(email);
        task
    }

    /// Set priority
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set deadline
    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Check if task is terminal (completed, failed, or cancelled)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            TaskStatus::Completed { .. } | TaskStatus::Failed { .. } | TaskStatus::Cancelled { .. }
        )
    }

    /// Check if task can be retried
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }
}
