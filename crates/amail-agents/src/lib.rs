// =============================================================================
// AMAIL-AGENTS - Actix Actor-Based AI Agent Swarm
// =============================================================================
// Table of Contents:
// 1. Actor Module - Base actor implementations
// 2. Agents Module - Specialized agent implementations
// 3. Registry Module - Agent registry and discovery
// 4. Supervisor Module - Supervision tree management
// 5. Messages Module - Actix message types
// =============================================================================

pub mod actor;
pub mod agents;
pub mod registry;
pub mod supervisor;
pub mod messages;
pub mod prompts;
pub mod reasoning;
pub mod pipeline;
pub mod workspace_rules;

pub use actor::AgentActor;
pub use registry::AgentRegistry;
pub use supervisor::SupervisorActor;
