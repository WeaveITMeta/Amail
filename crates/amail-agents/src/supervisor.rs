// =============================================================================
// Supervisor Actor - Erlang/OTP-Style Supervision
// =============================================================================
// Table of Contents:
// 1. SupervisorActor struct - Supervisor state
// 2. SupervisionStrategy - One-for-one, one-for-all, rest-for-one
// 3. Actor trait implementation
// 4. Child management and restart logic
// =============================================================================

use actix::prelude::*;
use amail_core::{
    agent::{AgentConfig, AgentId, AgentRole, AgentStatus},
    traits::{SupervisionAction, SupervisionStrategy},
};
use std::collections::HashMap;
use tracing::{error, info, warn};

use crate::actor::AgentActor;
use crate::messages::{AgentStatusResponse, StatusQuery, SupervisionMessage};
use crate::registry::AgentRegistry;

/// Supervisor actor - manages child agent lifecycle
pub struct SupervisorActor {
    /// Supervisor name
    name: String,
    /// Supervision strategy
    strategy: SupervisionStrategy,
    /// Child agents
    children: HashMap<AgentId, ChildInfo>,
    /// Maximum restart intensity (restarts per period)
    max_restarts: u32,
    /// Restart period in seconds
    restart_period_secs: u64,
    /// Recent restart timestamps
    restart_history: Vec<chrono::DateTime<chrono::Utc>>,
    /// Registry address (for registration)
    registry: Option<Addr<AgentRegistry>>,
}

/// Information about a supervised child
struct ChildInfo {
    /// Child configuration
    config: AgentConfig,
    /// Child actor address
    addr: Option<Addr<AgentActor>>,
    /// Restart count
    restart_count: u32,
    /// Last known status
    last_status: AgentStatus,
}

impl SupervisorActor {
    /// Create a new supervisor
    pub fn new(name: impl Into<String>, strategy: SupervisionStrategy) -> Self {
        Self {
            name: name.into(),
            strategy,
            children: HashMap::new(),
            max_restarts: 10,
            restart_period_secs: 60,
            restart_history: Vec::new(),
            registry: None,
        }
    }

    /// Set the registry address
    pub fn with_registry(mut self, registry: Addr<AgentRegistry>) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Add a child agent configuration
    pub fn add_child(&mut self, config: AgentConfig) {
        let id = config.id;
        self.children.insert(
            id,
            ChildInfo {
                config,
                addr: None,
                restart_count: 0,
                last_status: AgentStatus::Starting,
            },
        );
    }

    /// Start a child agent
    fn start_child(&mut self, id: AgentId, ctx: &mut Context<Self>) {
        if let Some(child) = self.children.get_mut(&id) {
            info!("Starting child agent {} ({})", child.config.name, id);

            let agent = AgentActor::new(child.config.clone());
            let addr = agent.start();
            child.addr = Some(addr.clone());
            child.last_status = AgentStatus::Starting;

            // Register with registry if available
            if let Some(registry) = &self.registry {
                registry.do_send(crate::registry::RegisterAgentFull {
                    agent_id: id,
                    role: child.config.role,
                    capabilities: child.config.capabilities.iter().cloned().collect(),
                    addr,
                });
            }
        }
    }

    /// Restart a child agent
    fn restart_child(&mut self, id: AgentId, ctx: &mut Context<Self>) {
        // Check restart intensity first
        if !self.check_restart_intensity() {
            error!(
                "Supervisor {} exceeded restart intensity, stopping",
                self.name
            );
            ctx.stop();
            return;
        }

        if let Some(child) = self.children.get_mut(&id) {
            info!(
                "Restarting child agent {} ({}) - attempt {}",
                child.config.name, id, child.restart_count + 1
            );

            // Stop existing if running
            if let Some(addr) = child.addr.take() {
                addr.do_send(SupervisionMessage::Stop);
            }

            // Increment restart count
            child.restart_count += 1;
            self.restart_history.push(chrono::Utc::now());
        }

        // Start new instance (outside the mutable borrow)
        self.start_child(id, ctx);
    }

    /// Check if restart intensity is within limits
    fn check_restart_intensity(&mut self) -> bool {
        let now = chrono::Utc::now();
        let cutoff = now - chrono::Duration::seconds(self.restart_period_secs as i64);

        // Remove old entries
        self.restart_history.retain(|t| *t > cutoff);

        // Check count
        self.restart_history.len() < self.max_restarts as usize
    }

    /// Handle child failure based on strategy
    fn handle_child_failure(&mut self, failed_id: AgentId, ctx: &mut Context<Self>) {
        match self.strategy {
            SupervisionStrategy::OneForOne => {
                // Only restart the failed child
                self.restart_child(failed_id, ctx);
            }
            SupervisionStrategy::OneForAll => {
                // Restart all children
                let ids: Vec<_> = self.children.keys().copied().collect();
                for id in ids {
                    self.restart_child(id, ctx);
                }
            }
            SupervisionStrategy::RestForOne => {
                // Restart children started after the failed one
                // For simplicity, restart all for now
                // TODO: Track start order
                self.restart_child(failed_id, ctx);
            }
        }
    }
}

impl Actor for SupervisorActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!("Supervisor {} started with {:?} strategy", self.name, self.strategy);

        // Start all children
        let ids: Vec<_> = self.children.keys().copied().collect();
        for id in ids {
            self.start_child(id, ctx);
        }

        // Schedule periodic health checks
        ctx.run_interval(std::time::Duration::from_secs(30), |actor, ctx| {
            actor.health_check_children(ctx);
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Supervisor {} stopped", self.name);

        // Stop all children
        for child in self.children.values() {
            if let Some(addr) = &child.addr {
                addr.do_send(SupervisionMessage::Stop);
            }
        }
    }
}

impl SupervisorActor {
    /// Perform health check on all children
    fn health_check_children(&mut self, ctx: &mut Context<Self>) {
        for (id, child) in &self.children {
            if let Some(addr) = &child.addr {
                let id = *id;
                addr.send(StatusQuery)
                    .into_actor(self)
                    .map(move |result, actor, ctx| {
                        match result {
                            Ok(status) => {
                                if let Some(child) = actor.children.get_mut(&id) {
                                    if let Ok(s) = status {
                                        child.last_status = s.status;
                                        if s.status == AgentStatus::Error {
                                            warn!("Child {} in error state, restarting", id);
                                            actor.handle_child_failure(id, ctx);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to get status from child {}: {}", id, e);
                                actor.handle_child_failure(id, ctx);
                            }
                        }
                    })
                    .spawn(ctx);
            }
        }
    }
}

/// Message to add a child to the supervisor
#[derive(Message)]
#[rtype(result = "Result<AgentId, ()>")]
pub struct AddChild(pub AgentConfig);

impl Handler<AddChild> for SupervisorActor {
    type Result = Result<AgentId, ()>;

    fn handle(&mut self, msg: AddChild, ctx: &mut Self::Context) -> Self::Result {
        let id = msg.0.id;
        self.add_child(msg.0);
        self.start_child(id, ctx);
        Ok(id)
    }
}

/// Message to remove a child from the supervisor
#[derive(Message)]
#[rtype(result = "()")]
pub struct RemoveChild(pub AgentId);

impl Handler<RemoveChild> for SupervisorActor {
    type Result = ();

    fn handle(&mut self, msg: RemoveChild, _ctx: &mut Self::Context) {
        if let Some(child) = self.children.remove(&msg.0) {
            if let Some(addr) = child.addr {
                addr.do_send(SupervisionMessage::Stop);
            }
            info!("Removed child {} from supervisor {}", msg.0, self.name);
        }
    }
}

/// Message to get supervisor status
#[derive(Message)]
#[rtype(result = "Result<SupervisorStatus, ()>")]
pub struct GetSupervisorStatus;

#[derive(Debug, Clone)]
pub struct SupervisorStatus {
    pub name: String,
    pub strategy: SupervisionStrategy,
    pub child_count: usize,
    pub children: Vec<(AgentId, AgentRole, AgentStatus)>,
}

impl Handler<GetSupervisorStatus> for SupervisorActor {
    type Result = Result<SupervisorStatus, ()>;

    fn handle(&mut self, _msg: GetSupervisorStatus, _ctx: &mut Self::Context) -> Self::Result {
        Ok(SupervisorStatus {
            name: self.name.clone(),
            strategy: self.strategy,
            child_count: self.children.len(),
            children: self
                .children
                .iter()
                .map(|(id, child)| (*id, child.config.role, child.last_status))
                .collect(),
        })
    }
}
