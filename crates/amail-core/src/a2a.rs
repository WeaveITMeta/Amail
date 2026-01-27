// =============================================================================
// A2A Protocol Types (Agent-to-Agent Communication)
// =============================================================================
// Table of Contents:
// 1. A2AMessage - Inter-agent message format
// 2. Delegation - Task delegation request
// 3. TaskResult - Task completion result
// 4. CapabilityRegistry - Agent capability discovery
// 5. A2AEnvelope - Message envelope with routing
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::agent::{AgentCapability, AgentId, AgentRole};
use crate::task::{Task, TaskId, TaskPriority};

/// A2A message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum A2AMessage {
    /// Request to delegate a task
    DelegationRequest(Delegation),
    /// Accept a delegation
    DelegationAccept {
        delegation_id: Uuid,
        agent_id: AgentId,
    },
    /// Reject a delegation
    DelegationReject {
        delegation_id: Uuid,
        agent_id: AgentId,
        reason: String,
    },
    /// Task status update
    TaskStatusUpdate {
        task_id: TaskId,
        status: String,
        progress: Option<f32>,
        message: Option<String>,
    },
    /// Task completion
    TaskComplete(TaskResult),
    /// Task failure
    TaskFailed {
        task_id: TaskId,
        error: String,
        recoverable: bool,
    },
    /// Capability query
    CapabilityQuery {
        query_id: Uuid,
        required_capabilities: Vec<AgentCapability>,
    },
    /// Capability response
    CapabilityResponse {
        query_id: Uuid,
        agent_id: AgentId,
        capabilities: Vec<AgentCapability>,
        available: bool,
    },
    /// Context sharing (for collaborative tasks)
    ContextShare {
        task_id: TaskId,
        context: serde_json::Value,
    },
    /// Heartbeat/ping
    Ping { timestamp: DateTime<Utc> },
    /// Heartbeat response
    Pong {
        timestamp: DateTime<Utc>,
        agent_id: AgentId,
    },
    /// Supervision message (from supervisor to supervised)
    SupervisionCommand(SupervisionCommand),
    /// Supervision report (from supervised to supervisor)
    SupervisionReport(SupervisionReport),
}

/// Task delegation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    /// Unique delegation ID
    pub id: Uuid,
    /// Source agent requesting delegation
    pub from_agent: AgentId,
    /// Target agent (if known) or None for capability-based routing
    pub to_agent: Option<AgentId>,
    /// Required capabilities for this task
    pub required_capabilities: Vec<AgentCapability>,
    /// The task to delegate
    pub task: Task,
    /// Priority override
    pub priority: TaskPriority,
    /// Deadline for acceptance
    pub accept_deadline: Option<DateTime<Utc>>,
    /// Context to pass along
    pub context: HashMap<String, serde_json::Value>,
    /// Delegation chain (for tracking multi-hop delegations)
    pub delegation_chain: Vec<AgentId>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

impl Delegation {
    /// Create a new delegation
    pub fn new(from_agent: AgentId, task: Task) -> Self {
        Self {
            id: Uuid::new_v4(),
            from_agent,
            to_agent: None,
            required_capabilities: Vec::new(),
            task,
            priority: TaskPriority::Normal,
            accept_deadline: None,
            context: HashMap::new(),
            delegation_chain: vec![from_agent],
            created_at: Utc::now(),
        }
    }

    /// Set target agent
    pub fn to(mut self, agent: AgentId) -> Self {
        self.to_agent = Some(agent);
        self
    }

    /// Set required capabilities
    pub fn with_capabilities(mut self, capabilities: Vec<AgentCapability>) -> Self {
        self.required_capabilities = capabilities;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self
    }
}

/// Task result from A2A completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task ID
    pub task_id: TaskId,
    /// Delegation ID (if delegated)
    pub delegation_id: Option<Uuid>,
    /// Completing agent
    pub agent_id: AgentId,
    /// Success flag
    pub success: bool,
    /// Result data
    pub result: serde_json::Value,
    /// Artifacts produced
    pub artifacts: Vec<Artifact>,
    /// Completion timestamp
    pub completed_at: DateTime<Utc>,
    /// Processing duration in milliseconds
    pub duration_ms: u64,
    /// Token usage (if LLM was used)
    pub token_usage: Option<TokenUsage>,
}

/// Artifact produced by task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Artifact ID
    pub id: Uuid,
    /// Artifact type
    pub artifact_type: ArtifactType,
    /// Name/title
    pub name: String,
    /// Content
    pub content: serde_json::Value,
    /// MIME type
    pub mime_type: Option<String>,
}

/// Artifact types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    Email,
    Document,
    Report,
    Data,
    Code,
    Image,
    Audio,
    Video,
    Other(String),
}

/// Token usage tracking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Supervision command from supervisor to agent
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum SupervisionCommand {
    /// Start the agent
    Start,
    /// Stop the agent gracefully
    Stop,
    /// Pause processing
    Pause,
    /// Resume processing
    Resume,
    /// Restart the agent
    Restart,
    /// Update configuration
    UpdateConfig { config: serde_json::Value },
    /// Request status report
    RequestStatus,
}

/// Supervision report from agent to supervisor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisionReport {
    /// Reporting agent
    pub agent_id: AgentId,
    /// Agent role
    pub role: AgentRole,
    /// Current status
    pub status: String,
    /// Active task count
    pub active_tasks: usize,
    /// Completed task count (since last report)
    pub completed_tasks: usize,
    /// Failed task count (since last report)
    pub failed_tasks: usize,
    /// Error messages (if any)
    pub errors: Vec<String>,
    /// Resource usage
    pub resource_usage: ResourceUsage,
    /// Report timestamp
    pub timestamp: DateTime<Utc>,
}

/// Resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    pub memory_mb: f64,
    pub cpu_percent: f64,
    pub queue_depth: usize,
}

/// A2A message envelope with routing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AEnvelope {
    /// Envelope ID
    pub id: Uuid,
    /// Source agent
    pub from: AgentId,
    /// Destination agent(s)
    pub to: Vec<AgentId>,
    /// Message payload
    pub message: A2AMessage,
    /// Correlation ID (for request-response tracking)
    pub correlation_id: Option<Uuid>,
    /// Reply-to agent (for async responses)
    pub reply_to: Option<AgentId>,
    /// Time-to-live in seconds
    pub ttl: Option<u32>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Trace ID (for distributed tracing)
    pub trace_id: Option<String>,
}

impl A2AEnvelope {
    /// Create a new envelope
    pub fn new(from: AgentId, to: AgentId, message: A2AMessage) -> Self {
        Self {
            id: Uuid::new_v4(),
            from,
            to: vec![to],
            message,
            correlation_id: None,
            reply_to: None,
            ttl: None,
            created_at: Utc::now(),
            trace_id: None,
        }
    }

    /// Broadcast to multiple agents
    pub fn broadcast(from: AgentId, to: Vec<AgentId>, message: A2AMessage) -> Self {
        Self {
            id: Uuid::new_v4(),
            from,
            to,
            message,
            correlation_id: None,
            reply_to: None,
            ttl: None,
            created_at: Utc::now(),
            trace_id: None,
        }
    }

    /// Set correlation ID
    pub fn with_correlation(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Set reply-to
    pub fn with_reply_to(mut self, reply_to: AgentId) -> Self {
        self.reply_to = Some(reply_to);
        self
    }
}

/// Agent capability registry for discovery
#[derive(Debug, Clone, Default)]
pub struct CapabilityRegistry {
    /// Map of capability to agents that provide it
    capabilities: HashMap<AgentCapability, HashSet<AgentId>>,
    /// Map of agent to its capabilities
    agents: HashMap<AgentId, HashSet<AgentCapability>>,
    /// Agent roles
    roles: HashMap<AgentId, AgentRole>,
}

impl CapabilityRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an agent with capabilities
    pub fn register(&mut self, agent_id: AgentId, role: AgentRole, capabilities: Vec<AgentCapability>) {
        self.roles.insert(agent_id, role);
        
        let cap_set: HashSet<_> = capabilities.iter().cloned().collect();
        self.agents.insert(agent_id, cap_set.clone());

        for cap in capabilities {
            self.capabilities
                .entry(cap)
                .or_default()
                .insert(agent_id);
        }
    }

    /// Unregister an agent
    pub fn unregister(&mut self, agent_id: &AgentId) {
        if let Some(caps) = self.agents.remove(agent_id) {
            for cap in caps {
                if let Some(agents) = self.capabilities.get_mut(&cap) {
                    agents.remove(agent_id);
                }
            }
        }
        self.roles.remove(agent_id);
    }

    /// Find agents with a specific capability
    pub fn find_by_capability(&self, capability: &AgentCapability) -> Vec<AgentId> {
        self.capabilities
            .get(capability)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Find agents with all required capabilities
    pub fn find_by_capabilities(&self, capabilities: &[AgentCapability]) -> Vec<AgentId> {
        if capabilities.is_empty() {
            return self.agents.keys().copied().collect();
        }

        let mut result: Option<HashSet<AgentId>> = None;

        for cap in capabilities {
            let agents = self
                .capabilities
                .get(cap)
                .cloned()
                .unwrap_or_default();

            result = Some(match result {
                Some(existing) => existing.intersection(&agents).copied().collect(),
                None => agents,
            });
        }

        result.map(|s| s.into_iter().collect()).unwrap_or_default()
    }

    /// Find agents by role
    pub fn find_by_role(&self, role: AgentRole) -> Vec<AgentId> {
        self.roles
            .iter()
            .filter(|(_, r)| **r == role)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get agent capabilities
    pub fn get_capabilities(&self, agent_id: &AgentId) -> Option<&HashSet<AgentCapability>> {
        self.agents.get(agent_id)
    }

    /// Get agent role
    pub fn get_role(&self, agent_id: &AgentId) -> Option<AgentRole> {
        self.roles.get(agent_id).copied()
    }

    /// List all registered agents
    pub fn list_agents(&self) -> Vec<(AgentId, AgentRole)> {
        self.roles.iter().map(|(id, role)| (*id, *role)).collect()
    }
}
