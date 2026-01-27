//! # Amail Mail Server Integration
//! 
//! This crate provides embedded mail server functionality for Amail,
//! including SMTP receiving, IMAP access, and mail storage.
//!
//! ## Architecture
//! 
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    AMAIL MAIL                           │
//! ├─────────────────────────────────────────────────────────┤
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
//! │  │ SMTP Server │  │ Mail Store  │  │ Domain Manager  │ │
//! │  │ (Inbound)   │  │ (PostgreSQL)│  │ (Multi-tenant)  │ │
//! │  └─────────────┘  └─────────────┘  └─────────────────┘ │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
//! │  │ SMTP Client │  │ DKIM/SPF    │  │ Webhook Events  │ │
//! │  │ (Outbound)  │  │ Validation  │  │ (to Agents)     │ │
//! │  └─────────────┘  └─────────────┘  └─────────────────┘ │
//! └─────────────────────────────────────────────────────────┘
//! ```

pub mod config;
pub mod smtp;
pub mod storage;
pub mod domain;
pub mod events;
pub mod auth;

pub use config::MailConfig;
pub use smtp::{SmtpServer, SmtpClient};
pub use storage::MailStore;
pub use domain::DomainManager;
pub use events::{MailEvent, MailEventHandler};
