// =============================================================================
// Agent Types
// =============================================================================
// Table of Contents:
// 1. AgentId - Unique agent identifier
// 2. AgentRole - Predefined agent roles (30+ types)
// 3. AgentCapability - What an agent can do
// 4. AgentStatus - Current agent state
// 5. AgentConfig - Agent configuration
// =============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

/// Unique agent identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

impl AgentId {
    /// Create a new random agent ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "agent-{}", self.0)
    }
}

/// Predefined agent roles for the virtual organization
/// 30+ specialized agents for comprehensive coverage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRole {
    // -------------------------------------------------------------------------
    // Core Agents (Original 6)
    // -------------------------------------------------------------------------
    /// Triage and routing - first contact for all emails
    Receptionist,
    /// Directory management, scheduling, registry
    Clerk,
    /// Onboarding, email provisioning, high-level decisions
    Executive,
    /// Supervised review of high-stakes outputs
    DraftChecker,
    /// Finance tracking, expense management
    Accountant,
    /// Legal queries, contract review, compliance
    Lawyer,

    // -------------------------------------------------------------------------
    // Extended Agents (25 additional roles)
    // -------------------------------------------------------------------------
    /// Onboarding, employee queries, performance reviews
    HumanResourcesManager,
    /// Tech troubleshooting, access/permissions
    ITSupportTechnician,
    /// Campaign drafting, engagement analysis
    MarketingCoordinator,
    /// Lead processing, quote generation
    SalesRepresentative,
    /// External inquiry/complaint resolution
    CustomerServiceRepresentative,
    /// Task tracking, deadline management, subtask assignment
    ProjectManager,
    /// Data gathering, report summarization
    ResearchAnalyst,
    /// Blog/social/documentation writing
    ContentCreator,
    /// Multilingual email/document handling
    Translator,
    /// Spreadsheet processing, insights generation
    DataAnalyst,
    /// Regulatory adherence (GDPR, industry standards)
    ComplianceOfficer,
    /// Threat monitoring, access log review
    SecurityAnalyst,
    /// Document organization, historical retrieval
    Archivist,
    /// User/agent onboarding, guide creation
    Trainer,
    /// Long-term goal setting, scenario analysis
    StrategicPlanner,
    /// Contract negotiation, vendor discussions
    Negotiator,
    /// Process review, inefficiency flagging
    InternalAuditor,
    /// Purchase management, vendor relations
    ProcurementSpecialist,
    /// Shipment tracking, inventory management
    LogisticsCoordinator,
    /// Agent output/tool validation
    QualityAssuranceTester,
    /// Operational risk identification/mitigation
    RiskManager,
    /// External communications, crisis management
    PublicRelationsOfficer,
    /// Virtual/physical event planning
    EventCoordinator,
    /// New tools/tech exploration (SpatialVortex integration)
    InnovationResearcher,
    /// Physical asset management
    FacilitiesManager,
    /// AI decision oversight, ethical review
    EthicsReviewer,

    // -------------------------------------------------------------------------
    // Special Agents
    // -------------------------------------------------------------------------
    /// Custom agent with user-defined role
    Custom(u32),
}

impl AgentRole {
    /// Get the display name for this role
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Receptionist => "Receptionist",
            Self::Clerk => "Clerk",
            Self::Executive => "Executive",
            Self::DraftChecker => "Draft Checker",
            Self::Accountant => "Accountant",
            Self::Lawyer => "Lawyer",
            Self::HumanResourcesManager => "Human Resources Manager",
            Self::ITSupportTechnician => "IT Support Technician",
            Self::MarketingCoordinator => "Marketing Coordinator",
            Self::SalesRepresentative => "Sales Representative",
            Self::CustomerServiceRepresentative => "Customer Service Representative",
            Self::ProjectManager => "Project Manager",
            Self::ResearchAnalyst => "Research Analyst",
            Self::ContentCreator => "Content Creator",
            Self::Translator => "Translator",
            Self::DataAnalyst => "Data Analyst",
            Self::ComplianceOfficer => "Compliance Officer",
            Self::SecurityAnalyst => "Security Analyst",
            Self::Archivist => "Archivist",
            Self::Trainer => "Trainer",
            Self::StrategicPlanner => "Strategic Planner",
            Self::Negotiator => "Negotiator",
            Self::InternalAuditor => "Internal Auditor",
            Self::ProcurementSpecialist => "Procurement Specialist",
            Self::LogisticsCoordinator => "Logistics Coordinator",
            Self::QualityAssuranceTester => "Quality Assurance Tester",
            Self::RiskManager => "Risk Manager",
            Self::PublicRelationsOfficer => "Public Relations Officer",
            Self::EventCoordinator => "Event Coordinator",
            Self::InnovationResearcher => "Innovation Researcher",
            Self::FacilitiesManager => "Facilities Manager",
            Self::EthicsReviewer => "Ethics Reviewer",
            Self::Custom(_) => "Custom Agent",
        }
    }

    /// Get the default capabilities for this role
    pub fn default_capabilities(&self) -> Vec<AgentCapability> {
        match self {
            Self::Receptionist => vec![
                AgentCapability::EmailTriage,
                AgentCapability::TaskRouting,
                AgentCapability::PriorityAssessment,
            ],
            Self::Clerk => vec![
                AgentCapability::DirectoryManagement,
                AgentCapability::Scheduling,
                AgentCapability::AgentRegistry,
            ],
            Self::Executive => vec![
                AgentCapability::Onboarding,
                AgentCapability::EmailProvisioning,
                AgentCapability::HighLevelDecisions,
            ],
            Self::DraftChecker => vec![
                AgentCapability::ContentReview,
                AgentCapability::QualityAssurance,
                AgentCapability::HumanEscalation,
            ],
            Self::Accountant => vec![
                AgentCapability::FinanceTracking,
                AgentCapability::ExpenseManagement,
                AgentCapability::ReportGeneration,
            ],
            Self::Lawyer => vec![
                AgentCapability::LegalQuery,
                AgentCapability::ContractReview,
                AgentCapability::ComplianceCheck,
            ],
            Self::ComplianceOfficer => vec![
                AgentCapability::ComplianceCheck,
                AgentCapability::RegulatoryAdherence,
                AgentCapability::AuditLogging,
            ],
            Self::SecurityAnalyst => vec![
                AgentCapability::ThreatMonitoring,
                AgentCapability::AccessLogReview,
                AgentCapability::SecurityAssessment,
            ],
            Self::InnovationResearcher => vec![
                AgentCapability::TechExploration,
                AgentCapability::SpatialVortexIntegration,
                AgentCapability::ResearchAnalysis,
            ],
            _ => vec![AgentCapability::GeneralAssistance],
        }
    }

    /// Check if this role requires human oversight for outputs
    pub fn requires_oversight(&self) -> bool {
        matches!(
            self,
            Self::Lawyer
                | Self::Accountant
                | Self::ComplianceOfficer
                | Self::SecurityAnalyst
                | Self::EthicsReviewer
                | Self::Executive
        )
    }
}

/// Agent capabilities - what an agent can do
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentCapability {
    // -------------------------------------------------------------------------
    // Email & Communication
    // -------------------------------------------------------------------------
    EmailTriage,
    EmailCompose,
    EmailReply,
    TaskRouting,
    PriorityAssessment,

    // -------------------------------------------------------------------------
    // Organization & Management
    // -------------------------------------------------------------------------
    DirectoryManagement,
    Scheduling,
    AgentRegistry,
    Onboarding,
    EmailProvisioning,
    HighLevelDecisions,

    // -------------------------------------------------------------------------
    // Review & Quality
    // -------------------------------------------------------------------------
    ContentReview,
    QualityAssurance,
    HumanEscalation,

    // -------------------------------------------------------------------------
    // Finance & Legal
    // -------------------------------------------------------------------------
    FinanceTracking,
    ExpenseManagement,
    ReportGeneration,
    LegalQuery,
    ContractReview,
    ComplianceCheck,
    RegulatoryAdherence,
    AuditLogging,

    // -------------------------------------------------------------------------
    // Security & Risk
    // -------------------------------------------------------------------------
    ThreatMonitoring,
    AccessLogReview,
    SecurityAssessment,
    RiskAssessment,

    // -------------------------------------------------------------------------
    // Research & Innovation
    // -------------------------------------------------------------------------
    TechExploration,
    SpatialVortexIntegration,
    ResearchAnalysis,
    DataAnalysis,

    // -------------------------------------------------------------------------
    // General
    // -------------------------------------------------------------------------
    GeneralAssistance,
    Translation,
    ContentCreation,

    // -------------------------------------------------------------------------
    // Custom
    // -------------------------------------------------------------------------
    Custom(String),
}

impl AgentCapability {
    /// Get the capability identifier for A2A protocol
    pub fn capability_id(&self) -> String {
        match self {
            Self::Custom(id) => id.clone(),
            other => format!("{:?}", other).to_lowercase(),
        }
    }
}

/// Current agent status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    /// Agent is starting up
    Starting,
    /// Agent is ready to receive tasks
    #[default]
    Ready,
    /// Agent is processing a task
    Busy,
    /// Agent is paused
    Paused,
    /// Agent is shutting down
    ShuttingDown,
    /// Agent has stopped
    Stopped,
    /// Agent encountered an error
    Error,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent ID
    pub id: AgentId,
    /// Agent role
    pub role: AgentRole,
    /// Agent name (human-readable)
    pub name: String,
    /// Agent capabilities
    pub capabilities: HashSet<AgentCapability>,
    /// Maximum concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Task timeout in seconds
    pub task_timeout_secs: u64,
    /// LLM model to use
    pub llm_model: String,
    /// System prompt for the agent
    pub system_prompt: String,
    /// Whether to require human oversight
    pub requires_oversight: bool,
    /// Parent supervisor agent (for supervision tree)
    pub supervisor: Option<AgentId>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        let role = AgentRole::Receptionist;
        Self {
            id: AgentId::new(),
            role,
            name: role.display_name().to_string(),
            capabilities: role.default_capabilities().into_iter().collect(),
            max_concurrent_tasks: 5,
            task_timeout_secs: 300,
            llm_model: "mistral:latest".to_string(),
            system_prompt: String::new(),
            requires_oversight: role.requires_oversight(),
            supervisor: None,
        }
    }
}
