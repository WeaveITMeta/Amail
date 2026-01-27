// =============================================================================
// Agent Registry - Actix Actor for Agent Discovery
// =============================================================================
// Table of Contents:
// 1. AgentRegistry struct - Registry state
// 2. Actor trait implementation
// 3. Message handlers (Register, Unregister, Find, Route)
// =============================================================================

use actix::prelude::*;
use amail_core::{
    a2a::{A2AEnvelope, CapabilityRegistry},
    agent::{AgentCapability, AgentId, AgentRole},
    AmailError,
};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::actor::AgentActor;
use crate::messages::{
    BroadcastMessage, FindAgent, ProcessTask, RegisterAgent, RouteMessage, RouteToRole, UnregisterAgent,
};

/// Agent registry actor - manages agent discovery and routing
pub struct AgentRegistry {
    /// Map of agent ID to actor address
    agents: HashMap<AgentId, Addr<AgentActor>>,
    /// Capability registry for discovery
    capabilities: CapabilityRegistry,
    /// Role to agent mapping (for quick role-based lookup)
    roles: HashMap<AgentRole, Vec<AgentId>>,
}

impl AgentRegistry {
    /// Create a new agent registry
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            capabilities: CapabilityRegistry::new(),
            roles: HashMap::new(),
        }
    }

    /// Get agent count
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Find agents by capability
    pub fn find_by_capability(&self, capability: &AgentCapability) -> Vec<AgentId> {
        self.capabilities.find_by_capability(capability)
    }

    /// Find agents by role
    pub fn find_by_role(&self, role: AgentRole) -> Vec<AgentId> {
        self.roles.get(&role).cloned().unwrap_or_default()
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for AgentRegistry {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        info!("Agent registry started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!("Agent registry stopped");
    }
}

/// Handle agent registration
impl Handler<RegisterAgent> for AgentRegistry {
    type Result = ();

    fn handle(&mut self, msg: RegisterAgent, _ctx: &mut Self::Context) {
        info!("Registering agent {}", msg.agent_id);
        self.agents.insert(msg.agent_id, msg.addr);
    }
}

/// Extended registration with capabilities
#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterAgentFull {
    pub agent_id: AgentId,
    pub role: AgentRole,
    pub capabilities: Vec<AgentCapability>,
    pub addr: Addr<AgentActor>,
}

impl Handler<RegisterAgentFull> for AgentRegistry {
    type Result = ();

    fn handle(&mut self, msg: RegisterAgentFull, _ctx: &mut Self::Context) {
        info!(
            "Registering agent {} with role {:?}",
            msg.agent_id, msg.role
        );

        // Store address
        self.agents.insert(msg.agent_id, msg.addr);

        // Register capabilities
        self.capabilities
            .register(msg.agent_id, msg.role, msg.capabilities);

        // Update role mapping
        self.roles
            .entry(msg.role)
            .or_default()
            .push(msg.agent_id);
    }
}

/// Handle agent unregistration
impl Handler<UnregisterAgent> for AgentRegistry {
    type Result = ();

    fn handle(&mut self, msg: UnregisterAgent, _ctx: &mut Self::Context) {
        info!("Unregistering agent {}", msg.0);
        self.agents.remove(&msg.0);
        self.capabilities.unregister(&msg.0);

        // Remove from role mapping
        for agents in self.roles.values_mut() {
            agents.retain(|id| *id != msg.0);
        }
    }
}

/// Handle agent lookup
impl Handler<FindAgent> for AgentRegistry {
    type Result = Option<Addr<AgentActor>>;

    fn handle(&mut self, msg: FindAgent, _ctx: &mut Self::Context) -> Self::Result {
        self.agents.get(&msg.0).cloned()
    }
}

/// Find agent by role
#[derive(Message)]
#[rtype(result = "Option<Addr<AgentActor>>")]
pub struct FindAgentByRole(pub AgentRole);

impl Handler<FindAgentByRole> for AgentRegistry {
    type Result = Option<Addr<AgentActor>>;

    fn handle(&mut self, msg: FindAgentByRole, _ctx: &mut Self::Context) -> Self::Result {
        // Get first available agent with this role
        self.roles
            .get(&msg.0)
            .and_then(|ids| ids.first())
            .and_then(|id| self.agents.get(id))
            .cloned()
    }
}

/// Find agent by capability
#[derive(Message)]
#[rtype(result = "Option<Addr<AgentActor>>")]
pub struct FindAgentByCapability(pub AgentCapability);

impl Handler<FindAgentByCapability> for AgentRegistry {
    type Result = Option<Addr<AgentActor>>;

    fn handle(&mut self, msg: FindAgentByCapability, _ctx: &mut Self::Context) -> Self::Result {
        let agents = self.capabilities.find_by_capability(&msg.0);
        agents
            .first()
            .and_then(|id| self.agents.get(id))
            .cloned()
    }
}

/// Handle broadcast messages
impl Handler<BroadcastMessage> for AgentRegistry {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, _ctx: &mut Self::Context) {
        debug!("Broadcasting message to {} agents", self.agents.len());
        for (_id, addr) in &self.agents {
            addr.do_send(crate::messages::A2AMessageWrapper(msg.0.clone()));
        }
    }
}

/// Handle message routing
impl Handler<RouteMessage> for AgentRegistry {
    type Result = Result<(), AmailError>;

    fn handle(&mut self, msg: RouteMessage, _ctx: &mut Self::Context) -> Self::Result {
        let envelope = msg.0;

        for target_id in &envelope.to {
            if let Some(addr) = self.agents.get(target_id) {
                addr.do_send(crate::messages::A2AMessageWrapper(envelope.clone()));
            } else {
                warn!("Target agent {} not found in registry", target_id);
            }
        }

        Ok(())
    }
}

/// List all agents
#[derive(Message)]
#[rtype(result = "Vec<(AgentId, AgentRole)>")]
pub struct ListAgents;

impl Handler<ListAgents> for AgentRegistry {
    type Result = Vec<(AgentId, AgentRole)>;

    fn handle(&mut self, _msg: ListAgents, _ctx: &mut Self::Context) -> Self::Result {
        self.capabilities.list_agents()
    }
}

/// Handle routing task to agent by role
impl Handler<RouteToRole> for AgentRegistry {
    type Result = Result<(), AmailError>;

    fn handle(&mut self, msg: RouteToRole, _ctx: &mut Self::Context) -> Self::Result {
        // Find first available agent with this role
        let agent_addr = self.roles
            .get(&msg.role)
            .and_then(|ids| ids.first())
            .and_then(|id| self.agents.get(id))
            .cloned();

        if let Some(addr) = agent_addr {
            addr.do_send(*msg.message);
            Ok(())
        } else {
            Err(AmailError::AgentNotFound(format!(
                "No agent found for role {:?}",
                msg.role
            )))
        }
    }
}
