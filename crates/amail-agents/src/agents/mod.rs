// =============================================================================
// Specialized Agent Implementations
// =============================================================================
// Table of Contents:
// 1. Agent role modules
// 2. Agent configuration factories
// =============================================================================

pub mod receptionist;
pub mod clerk;
pub mod executive;
pub mod draft_checker;
pub mod lawyer;
pub mod accountant;

pub use receptionist::{ReceptionistAgent, EmailClassification, EmailCategory};

pub use clerk::ClerkAgent;
pub use executive::ExecutiveAgent;
pub use draft_checker::DraftCheckerAgent;
pub use lawyer::LawyerAgent;
pub use accountant::AccountantAgent;

use amail_core::agent::{AgentConfig, AgentId, AgentRole};

/// Create default configuration for a role
pub fn create_agent_config(role: AgentRole, name: Option<String>) -> AgentConfig {
    let name = name.unwrap_or_else(|| role.display_name().to_string());
    
    AgentConfig {
        id: AgentId::new(),
        role,
        name,
        capabilities: role.default_capabilities().into_iter().collect(),
        max_concurrent_tasks: match role {
            AgentRole::Receptionist => 20, // High throughput for triage
            AgentRole::DraftChecker => 5,  // Careful review
            AgentRole::Lawyer | AgentRole::Accountant => 3, // Complex tasks
            _ => 10,
        },
        task_timeout_secs: match role {
            AgentRole::Lawyer | AgentRole::ComplianceOfficer => 600, // 10 min for complex
            AgentRole::DraftChecker => 300, // 5 min for review
            _ => 120, // 2 min default
        },
        llm_model: "mistral:latest".to_string(),
        system_prompt: String::new(), // Will use default
        requires_oversight: role.requires_oversight(),
        supervisor: None,
    }
}

/// Create the core agent swarm (6 primary agents)
pub fn create_core_swarm() -> Vec<AgentConfig> {
    vec![
        create_agent_config(AgentRole::Receptionist, None),
        create_agent_config(AgentRole::Clerk, None),
        create_agent_config(AgentRole::Executive, None),
        create_agent_config(AgentRole::DraftChecker, None),
        create_agent_config(AgentRole::Lawyer, None),
        create_agent_config(AgentRole::Accountant, None),
    ]
}

/// Create the full agent swarm (30+ agents)
pub fn create_full_swarm() -> Vec<AgentConfig> {
    let roles = vec![
        // Core 6
        AgentRole::Receptionist,
        AgentRole::Clerk,
        AgentRole::Executive,
        AgentRole::DraftChecker,
        AgentRole::Accountant,
        AgentRole::Lawyer,
        // Extended 25
        AgentRole::HumanResourcesManager,
        AgentRole::ITSupportTechnician,
        AgentRole::MarketingCoordinator,
        AgentRole::SalesRepresentative,
        AgentRole::CustomerServiceRepresentative,
        AgentRole::ProjectManager,
        AgentRole::ResearchAnalyst,
        AgentRole::ContentCreator,
        AgentRole::Translator,
        AgentRole::DataAnalyst,
        AgentRole::ComplianceOfficer,
        AgentRole::SecurityAnalyst,
        AgentRole::Archivist,
        AgentRole::Trainer,
        AgentRole::StrategicPlanner,
        AgentRole::Negotiator,
        AgentRole::InternalAuditor,
        AgentRole::ProcurementSpecialist,
        AgentRole::LogisticsCoordinator,
        AgentRole::QualityAssuranceTester,
        AgentRole::RiskManager,
        AgentRole::PublicRelationsOfficer,
        AgentRole::EventCoordinator,
        AgentRole::InnovationResearcher,
        AgentRole::FacilitiesManager,
        AgentRole::EthicsReviewer,
    ];

    roles.into_iter().map(|r| create_agent_config(r, None)).collect()
}
