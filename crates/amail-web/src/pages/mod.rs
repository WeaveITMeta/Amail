//! Admin UI Pages
//!
//! Table of Contents:
//! 1. Dashboard - Main overview
//! 2. Domains - Domain management
//! 3. DNS Wizard - DNS verification
//! 4. Agents - Agent monitoring
//! 5. Settings - System settings
//! 6. Inbox - Gmail-like email interface
//! 7. Tenants - Multi-tenant management
//! 8. Reviews - Human review queue for AI responses
//! 9. Analytics - KPI tracking and summaries
//! 10. Docs - Documentation viewer with floating TOC
//! 11. Mailboxes - Mailbox management for enterprise email
//! 12. Workspace Rules - General rules for all AI agents

pub mod dashboard;
pub mod domains;
pub mod dns_wizard;
pub mod agents;
pub mod settings;
pub mod inbox;
pub mod tenants;
pub mod reviews;
pub mod analytics;
pub mod docs;
pub mod mailboxes;
pub mod workspace_rules;

pub use dashboard::DashboardPage;
pub use domains::DomainsPage;
pub use dns_wizard::DnsWizardPage;
pub use agents::AgentsPage;
pub use settings::SettingsPage;
pub use inbox::InboxPage;
pub use tenants::TenantsPage;
pub use reviews::ReviewsPage;
pub use analytics::AnalyticsPage;
pub use docs::DocsPage;
pub use mailboxes::MailboxesPage;
pub use workspace_rules::WorkspaceRulesPage;
