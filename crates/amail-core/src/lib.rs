// =============================================================================
// AMAIL-CORE - Core Types, Traits, and Utilities
// =============================================================================
// Table of Contents:
// 1. Error Types
// 2. Email Types (EmailMessage, EmailMetadata, Attachment)
// 3. Agent Types (AgentId, AgentRole, AgentCapability)
// 4. Task Types (Task, TaskStatus, TaskPriority)
// 5. A2A Protocol Types (A2AMessage, Delegation, TaskResult)
// 6. Traits (Agent, Tool, MessageHandler)
// =============================================================================

pub mod error;
pub mod email;
pub mod agent;
pub mod task;
pub mod a2a;
pub mod traits;
pub mod db;
pub mod queue;
pub mod tenant;
pub mod telegram;
pub mod analytics;
pub mod scheduler;
pub mod repositories;
pub mod stalwart;
pub mod spam;
pub mod compliance;
pub mod p2p_attachments;
pub mod config_manager;

pub use error::{AmailError, Result};
pub use email::{EmailMessage, EmailMetadata, Attachment, EmailAddress};
pub use agent::{AgentId, AgentRole, AgentCapability, AgentStatus};
pub use task::{Task, TaskId, TaskStatus, TaskPriority, TaskContext};
pub use a2a::{A2AMessage, Delegation, TaskResult, CapabilityRegistry};
pub use traits::{Agent, Tool, MessageHandler};
pub use tenant::{
    // Core types - Self-hosted, unlimited, FREE!
    ServerConfig, Domain, DomainStatus, DnsVerification, DomainSettings,
    Mailbox, MailboxType, DomainManager, DomainError, RoutingInfo,
    BusinessHours, ServerStats,
    // Backwards compatibility (deprecated)
    Tenant, TenantType, TenantStatus, TenantPlan, TenantBranding, TenantManager, TenantError,
};
pub use telegram::{
    TelegramConfig, TelegramClient, TelegramError,
    NotificationType, NotificationPreferences,
    ReviewRequest, ReviewResponse, ReviewStatus, ReviewDecision,
};
pub use analytics::{
    KpiTracker, MetricsPeriod, MetricsSnapshot,
    EmailMetrics, AgentMetrics, ReviewMetrics,
    SummaryReport, TrendItem, Trend,
};
pub use scheduler::{
    SchedulerConfig, AmailScheduler,
    generate_task_scheduler_xml, generate_systemd_unit, generate_systemd_timer,
    generate_launchd_plist, generate_powershell_install_script,
};
pub use spam::{
    SpamFilter, SpamResult, SpamClassification, SpamFolder, SpamIndicator,
    CanSpamCompliance, CanSpamCheckResult,
    AutoReplyDetector, AutoReplyResult,
};
pub use compliance::{
    ComplianceConfig, ComplianceError, ComplianceAuditLog, ComplianceEventType,
    CsamCompliance, NcmecReport, CsamIncidentType, ReportStatus,
    DmcaCompliance, DmcaTakedownNotice, DmcaCounterNotice, DmcaStatus,
    CoppaCompliance, GdprCompliance, DataSubjectRequest, DataBreachNotification,
    RetentionPolicy, LegalHold, Jurisdiction,
};
pub use p2p_attachments::{
    P2PAttachmentConfig, P2PAttachment, P2PDownloadStatus, EmailAttachment,
    AttachmentProcessor, AttachmentAnalysis, AttachmentRecommendation,
    P2PDownloadTracker, P2PDownloadInfo,
    embed_p2p_attachment_html, embed_p2p_attachment_text, extract_p2p_attachments,
    extract_magnet_links, format_size, P2P_ATTACHMENT_CSS,
};
pub use config_manager::{
    ConfigManager, ConfigError,
    DomainsConfig, DomainConfig, DomainSettings as ConfigDomainSettings, DkimConfig, DomainBranding,
    MailboxesConfig, MailboxConfig, MailboxSettings, MailboxAgentConfig, AliasConfig, GroupConfig,
    DnsConfig, DnsServerConfig, DomainDnsConfig, DnsRecord,
    SmtpConfig, InboundSmtpConfig, OutboundSmtpConfig, TlsConfig,
    AgentsConfig, GlobalAgentConfig, LlmConfig, ConfidenceThresholds,
    SecurityConfig, SpamConfig as ConfigSpamConfig, RateLimitingConfig, AutoReplyPreventionConfig,
};
