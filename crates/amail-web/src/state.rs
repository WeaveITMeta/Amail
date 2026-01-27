// =============================================================================
// Dashboard State Management
// =============================================================================
// Table of Contents:
// 1. DashboardState - Central state store
// 2. Agent state types
// 3. Task state types
// 4. Statistics types
// =============================================================================

use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Connection status for WebSocket
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
}

impl ConnectionStatus {
    pub fn label(&self) -> &'static str {
        match self {
            ConnectionStatus::Connected => "Online",
            ConnectionStatus::Connecting => "Connecting",
            ConnectionStatus::Disconnected => "Offline",
        }
    }
    
    pub fn css_class(&self) -> &'static str {
        match self {
            ConnectionStatus::Connected => "connected",
            ConnectionStatus::Connecting => "connecting",
            ConnectionStatus::Disconnected => "disconnected",
        }
    }
}

/// Central dashboard state
#[derive(Clone)]
pub struct DashboardState {
    /// Connected agents
    pub agents: RwSignal<HashMap<String, AgentState>>,
    /// Active tasks
    pub tasks: RwSignal<Vec<TaskState>>,
    /// System statistics
    pub stats: RwSignal<SystemStats>,
    /// Recent events
    pub events: RwSignal<Vec<DashboardEvent>>,
    /// WebSocket connection status
    pub ws_connected: RwSignal<bool>,
    /// Connection status (connected, connecting, disconnected)
    pub connection_status: RwSignal<ConnectionStatus>,
}

impl DashboardState {
    pub fn new() -> Self {
        Self {
            agents: create_rw_signal(HashMap::new()),
            tasks: create_rw_signal(Vec::new()),
            stats: create_rw_signal(SystemStats::default()),
            events: create_rw_signal(Vec::new()),
            ws_connected: create_rw_signal(false),
            connection_status: create_rw_signal(ConnectionStatus::Connecting),
        }
    }

    /// Update agent state
    pub fn update_agent(&self, agent: AgentState) {
        self.agents.update(|agents| {
            agents.insert(agent.id.clone(), agent);
        });
    }

    /// Remove agent
    pub fn remove_agent(&self, agent_id: &str) {
        self.agents.update(|agents| {
            agents.remove(agent_id);
        });
    }

    /// Add or update task
    pub fn update_task(&self, task: TaskState) {
        self.tasks.update(|tasks| {
            if let Some(existing) = tasks.iter_mut().find(|t| t.id == task.id) {
                *existing = task;
            } else {
                tasks.insert(0, task);
                // Keep only last 100 tasks
                if tasks.len() > 100 {
                    tasks.pop();
                }
            }
        });
    }

    /// Update statistics
    pub fn update_stats(&self, stats: SystemStats) {
        self.stats.set(stats);
    }

    /// Add event
    pub fn add_event(&self, event: DashboardEvent) {
        self.events.update(|events| {
            events.insert(0, event);
            // Keep only last 50 events
            if events.len() > 50 {
                events.pop();
            }
        });
    }

    /// Set connection status
    pub fn set_connected(&self, connected: bool) {
        self.ws_connected.set(connected);
        if connected {
            self.connection_status.set(ConnectionStatus::Connected);
        } else {
            self.connection_status.set(ConnectionStatus::Disconnected);
        }
    }
    
    /// Set connecting status (yellow, trying to connect)
    pub fn set_connecting(&self) {
        self.connection_status.set(ConnectionStatus::Connecting);
    }

    /// Handle incoming WebSocket message
    pub fn handle_message(&self, msg: WsMessage) {
        match msg {
            WsMessage::AgentUpdate(agent) => self.update_agent(agent),
            WsMessage::AgentRemoved { agent_id } => self.remove_agent(&agent_id),
            WsMessage::TaskUpdate(task) => self.update_task(task),
            WsMessage::StatsUpdate(stats) => self.update_stats(stats),
            WsMessage::Event(event) => self.add_event(event),
            WsMessage::FullState { agents, tasks, stats } => {
                self.agents.set(agents.into_iter().map(|a| (a.id.clone(), a)).collect());
                self.tasks.set(tasks);
                self.stats.set(stats);
            }
        }
    }
}

impl Default for DashboardState {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Agent State
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentState {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: AgentStatus,
    pub active_tasks: u32,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub last_activity: Option<String>,
    pub current_task: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Idle,
    Processing,
    Error,
    Starting,
    Stopping,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl AgentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentStatus::Idle => "idle",
            AgentStatus::Processing => "processing",
            AgentStatus::Error => "error",
            AgentStatus::Starting => "starting",
            AgentStatus::Stopping => "stopping",
        }
    }

    pub fn display(&self) -> &'static str {
        match self {
            AgentStatus::Idle => "Idle",
            AgentStatus::Processing => "Processing",
            AgentStatus::Error => "Error",
            AgentStatus::Starting => "Starting",
            AgentStatus::Stopping => "Stopping",
        }
    }
}

// =============================================================================
// Task State
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TaskState {
    pub id: String,
    pub task_type: String,
    pub description: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub assigned_agent: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Critical,
    High,
    Normal,
    Low,
}

impl TaskPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskPriority::Critical => "critical",
            TaskPriority::High => "high",
            TaskPriority::Normal => "normal",
            TaskPriority::Low => "low",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl TaskStatus {
    pub fn display(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "Pending",
            TaskStatus::Processing => "Processing",
            TaskStatus::Completed => "Completed",
            TaskStatus::Failed => "Failed",
        }
    }
}

// =============================================================================
// System Statistics
// =============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SystemStats {
    pub total_agents: u32,
    pub active_agents: u32,
    pub total_tasks_processed: u64,
    pub tasks_per_minute: f64,
    pub avg_task_duration_ms: u64,
    pub emails_received: u64,
    pub emails_sent: u64,
    pub llm_calls: u64,
    pub mcp_calls: u64,
    pub uptime_seconds: u64,
}

// =============================================================================
// Dashboard Events
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardEvent {
    pub id: String,
    pub event_type: EventType,
    pub message: String,
    pub timestamp: String,
    pub agent_id: Option<String>,
    pub task_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Info,
    Success,
    Warning,
    Error,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    AgentStarted,
    AgentStopped,
    EmailReceived,
    EmailSent,
}

// =============================================================================
// WebSocket Messages
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
pub enum WsMessage {
    #[serde(rename = "agentupdate")]
    AgentUpdate(AgentState),
    #[serde(rename = "agentremoved")]
    AgentRemoved { agent_id: String },
    #[serde(rename = "taskupdate")]
    TaskUpdate(TaskState),
    #[serde(rename = "statsupdate")]
    StatsUpdate(SystemStats),
    #[serde(rename = "event")]
    Event(DashboardEvent),
    #[serde(rename = "fullstate")]
    FullState {
        agents: Vec<AgentState>,
        tasks: Vec<TaskState>,
        stats: SystemStats,
    },
}
