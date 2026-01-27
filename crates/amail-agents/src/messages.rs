// =============================================================================
// Actix Message Types
// =============================================================================
// Table of Contents:
// 1. TaskMessage - Task processing message
// 2. A2AMessageWrapper - A2A protocol message wrapper
// 3. SupervisionMessage - Supervision commands
// 4. StatusQuery - Status query message
// =============================================================================

use actix::prelude::*;
use amail_core::{
    a2a::{A2AEnvelope, TaskResult},
    agent::{AgentId, AgentStatus},
    task::Task,
};

/// Message to process a task
#[derive(Message)]
#[rtype(result = "Result<TaskResult, amail_core::AmailError>")]
pub struct ProcessTask(pub Task);

/// Message wrapper for A2A protocol messages
#[derive(Message)]
#[rtype(result = "Result<Option<A2AEnvelope>, amail_core::AmailError>")]
pub struct A2AMessageWrapper(pub A2AEnvelope);

/// Supervision command message
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub enum SupervisionMessage {
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
    /// Health check request
    HealthCheck,
}

/// Status query message
#[derive(Message)]
#[rtype(result = "Result<AgentStatusResponse, ()>")]
pub struct StatusQuery;

/// Status response
#[derive(Debug, Clone)]
pub struct AgentStatusResponse {
    pub agent_id: AgentId,
    pub status: AgentStatus,
    pub active_tasks: usize,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
}

/// Register agent with registry
#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterAgent {
    pub agent_id: AgentId,
    pub addr: Addr<super::actor::AgentActor>,
}

/// Unregister agent from registry
#[derive(Message)]
#[rtype(result = "()")]
pub struct UnregisterAgent(pub AgentId);

/// Find agent by ID
#[derive(Message)]
#[rtype(result = "Option<Addr<super::actor::AgentActor>>")]
pub struct FindAgent(pub AgentId);

/// Broadcast message to all agents
#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastMessage(pub A2AEnvelope);

/// Route message to appropriate agent(s)
#[derive(Message)]
#[rtype(result = "Result<(), amail_core::AmailError>")]
pub struct RouteMessage(pub A2AEnvelope);

/// Route task to agent by role
#[derive(Message)]
#[rtype(result = "Result<(), amail_core::AmailError>")]
pub struct RouteToRole {
    pub role: amail_core::agent::AgentRole,
    pub message: Box<ProcessTask>,
}
