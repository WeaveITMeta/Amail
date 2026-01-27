//! Database Repositories - Production-grade PostgreSQL data access
//!
//! Table of Contents:
//! 1. Domain Repository
//! 2. Mailbox Repository
//! 3. Email Repository (enhanced)
//! 4. Analytics Repository
//! 5. Review Repository
//! 6. Config Repository

pub mod domain;
pub mod mailbox;
pub mod email;
pub mod analytics;
pub mod review;
pub mod config;

pub use domain::DomainRepository;
pub use mailbox::MailboxRepository;
pub use email::EmailRepository;
pub use analytics::AnalyticsRepository;
pub use review::ReviewRepository;
pub use config::ConfigRepository;
