//! Mail server configuration

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Mail server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailConfig {
    /// SMTP server bind address
    pub smtp_bind: SocketAddr,
    /// SMTP submission port (587)
    pub smtp_submission_bind: Option<SocketAddr>,
    /// SMTPS port (465)
    pub smtps_bind: Option<SocketAddr>,
    /// Server hostname (used in SMTP HELO)
    pub hostname: String,
    /// Primary mail domain
    pub domain: String,
    /// Organization name
    pub org_name: String,
    /// TLS certificate path (PEM)
    pub tls_cert: Option<String>,
    /// TLS key path (PEM)
    pub tls_key: Option<String>,
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Maximum recipients per message
    pub max_recipients: usize,
    /// Database URL for mail storage
    pub database_url: String,
    /// Enable DKIM signing
    pub dkim_enabled: bool,
    /// DKIM selector
    pub dkim_selector: String,
    /// DKIM private key path
    pub dkim_key: Option<String>,
    /// Enable SPF checking
    pub spf_enabled: bool,
    /// Enable DMARC checking
    pub dmarc_enabled: bool,
    /// Webhook URL for mail events (to Amail agents)
    pub webhook_url: Option<String>,
}

impl Default for MailConfig {
    fn default() -> Self {
        Self {
            smtp_bind: "0.0.0.0:25".parse().unwrap(),
            smtp_submission_bind: Some("0.0.0.0:587".parse().unwrap()),
            smtps_bind: Some("0.0.0.0:465".parse().unwrap()),
            hostname: "localhost".to_string(),
            domain: "localhost".to_string(),
            org_name: "Amail".to_string(),
            tls_cert: None,
            tls_key: None,
            max_message_size: 50 * 1024 * 1024, // 50MB
            max_recipients: 100,
            database_url: "postgres://amail:password@localhost:5432/amail".to_string(),
            dkim_enabled: false,
            dkim_selector: "default".to_string(),
            dkim_key: None,
            spf_enabled: true,
            dmarc_enabled: true,
            webhook_url: Some("http://localhost:4000/api/v1/mail/inbound".to_string()),
        }
    }
}

impl MailConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(val) = std::env::var("MAIL_SMTP_BIND") {
            if let Ok(addr) = val.parse() {
                config.smtp_bind = addr;
            }
        }
        
        if let Ok(val) = std::env::var("MAIL_HOSTNAME") {
            config.hostname = val;
        }
        
        if let Ok(val) = std::env::var("MAIL_DOMAIN") {
            config.domain = val;
        }
        
        if let Ok(val) = std::env::var("MAIL_ORG_NAME") {
            config.org_name = val;
        }
        
        if let Ok(val) = std::env::var("MAIL_TLS_CERT") {
            config.tls_cert = Some(val);
        }
        
        if let Ok(val) = std::env::var("MAIL_TLS_KEY") {
            config.tls_key = Some(val);
        }
        
        if let Ok(val) = std::env::var("DATABASE_URL") {
            config.database_url = val;
        }
        
        if let Ok(val) = std::env::var("MAIL_WEBHOOK_URL") {
            config.webhook_url = Some(val);
        }
        
        if let Ok(val) = std::env::var("MAIL_DKIM_ENABLED") {
            config.dkim_enabled = val.to_lowercase() == "true";
        }
        
        if let Ok(val) = std::env::var("MAIL_DKIM_SELECTOR") {
            config.dkim_selector = val;
        }
        
        if let Ok(val) = std::env::var("MAIL_DKIM_KEY") {
            config.dkim_key = Some(val);
        }
        
        config
    }
}
