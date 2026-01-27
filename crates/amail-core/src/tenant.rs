//! Multi-Domain Management for Self-Hosted Amail
//!
//! ============================================================================
//! AMAIL PHILOSOPHY - FREEDOM FROM CORPORATE EMAIL
//! ============================================================================
//!
//! Amail is 100% Open Source and FREE forever. No subscriptions. No limits.
//! No corporate overlords draining your wallet month after month.
//!
//! Core Principles:
//! - **Self-Hosted**: You own your server, your data, your privacy
//! - **Decentralized**: Break free from Big Tech email monopolies  
//! - **Customizable AI Staff**: Configure your own AI agents to handle email
//! - **Auto-Startup**: Ready at 8am every morning to tackle your day
//! - **Unlimited**: No artificial limits on domains, mailboxes, or agents
//!
//! This is email the way it should be - owned by you, controlled by you.
//!
//! Table of Contents:
//! 1. ServerConfig - Self-hosted server configuration
//! 2. Domain - Email domain configuration  
//! 3. Mailbox - Individual email addresses with AI agent assignment
//! 4. DomainManager - Domain and mailbox management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;

// =============================================================================
// 1. Server Configuration - Your Self-Hosted Instance
// =============================================================================

/// Configuration for your self-hosted Amail server
/// 
/// This is YOUR server. No limits. No fees. Just freedom.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server instance name (e.g., "My Home Server", "Office Mail")
    pub instance_name: String,
    /// Admin email for this server
    pub admin_email: String,
    /// Server hostname (e.g., "mail.mydomain.com")
    pub hostname: String,
    /// SMTP port (default: 25 or 2525 for non-root)
    pub smtp_port: u16,
    /// Dashboard port (default: 8080)
    pub dashboard_port: u16,
    /// Auto-startup time (default: "08:00")
    pub auto_startup_time: String,
    /// Timezone for scheduling (default: "UTC")
    pub timezone: String,
    /// Ollama/LLM endpoint for AI agents
    pub llm_endpoint: String,
    /// Default LLM model (e.g., "mistral", "llama3")
    pub default_model: String,
    /// Enable Telegram notifications
    pub telegram_enabled: bool,
    /// Telegram bot token (if enabled)
    pub telegram_bot_token: Option<String>,
    /// Telegram chat ID for notifications
    pub telegram_chat_id: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            instance_name: "My Amail Server".to_string(),
            admin_email: "admin@localhost".to_string(),
            hostname: "localhost".to_string(),
            smtp_port: 2525,
            dashboard_port: 8080,
            auto_startup_time: "08:00".to_string(),
            timezone: "UTC".to_string(),
            llm_endpoint: "http://localhost:11434".to_string(),
            default_model: "mistral".to_string(),
            telegram_enabled: false,
            telegram_bot_token: None,
            telegram_chat_id: None,
            created_at: Utc::now(),
        }
    }
}

// =============================================================================
// 2. Domain - Email Domain Configuration
// =============================================================================

/// Email domain configuration
/// 
/// Add as many domains as you want - it's YOUR server!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    /// Unique domain identifier
    pub id: Uuid,
    /// Domain name (e.g., "example.com")
    pub domain_name: String,
    /// Domain status
    pub status: DomainStatus,
    /// DNS verification status
    pub dns_verification: DnsVerification,
    /// Catch-all mailbox (receives unmatched emails)
    pub catch_all_mailbox: Option<String>,
    /// Default agent role for this domain
    pub default_agent_role: Option<String>,
    /// Custom DKIM selector
    pub dkim_selector: String,
    /// Domain-specific settings
    pub settings: DomainSettings,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last verified timestamp
    pub last_verified_at: Option<DateTime<Utc>>,
}

impl Domain {
    /// Create a new domain - no limits, add as many as you need!
    pub fn new(domain_name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            domain_name: domain_name.to_lowercase(),
            status: DomainStatus::Pending,
            dns_verification: DnsVerification::default(),
            catch_all_mailbox: None,
            default_agent_role: Some("Receptionist".to_string()),
            dkim_selector: "amail".to_string(),
            settings: DomainSettings::default(),
            created_at: Utc::now(),
            last_verified_at: None,
        }
    }
    
    /// Generate TXT verification record value
    pub fn txt_verification_value(&self) -> String {
        format!("amail-verify={}", self.id.to_string().replace("-", "")[..16].to_string())
    }
    
    /// Generate expected MX record (points to your server)
    pub fn mx_record(&self, hostname: &str) -> String {
        hostname.to_string()
    }
    
    /// Generate SPF record value
    pub fn spf_record(&self, hostname: &str) -> String {
        format!("v=spf1 a mx ip4:{} ~all", hostname)
    }
    
    /// Generate DKIM record name
    pub fn dkim_record_name(&self) -> String {
        format!("{}._domainkey.{}", self.dkim_selector, self.domain_name)
    }
    
    /// Generate DMARC record value
    pub fn dmarc_record(&self) -> String {
        format!("v=DMARC1; p=quarantine; rua=mailto:dmarc@{}", self.domain_name)
    }
    
    /// Check if domain is fully verified
    pub fn is_fully_verified(&self) -> bool {
        self.dns_verification.txt_verified &&
        self.dns_verification.mx_verified &&
        self.dns_verification.spf_verified
    }
    
    /// Update verification status
    pub fn update_verification(&mut self, verification: DnsVerification) {
        self.dns_verification = verification;
        if self.is_fully_verified() {
            self.status = DomainStatus::Active;
            self.last_verified_at = Some(Utc::now());
        }
    }
}

/// Domain status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DomainStatus {
    /// Pending DNS verification
    Pending,
    /// Active and receiving email
    Active,
    /// DNS verification failed
    Failed,
    /// Disabled by admin
    Disabled,
}

/// DNS verification status for a domain
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DnsVerification {
    /// TXT record verified
    pub txt_verified: bool,
    /// MX record verified
    pub mx_verified: bool,
    /// SPF record verified
    pub spf_verified: bool,
    /// DKIM record verified
    pub dkim_verified: bool,
    /// DMARC record verified
    pub dmarc_verified: bool,
    /// Last check timestamp
    pub last_check: Option<DateTime<Utc>>,
    /// Error message if verification failed
    pub error_message: Option<String>,
}

impl DnsVerification {
    /// Calculate verification progress (0-100)
    pub fn progress(&self) -> u8 {
        let mut count = 0;
        if self.txt_verified { count += 1; }
        if self.mx_verified { count += 1; }
        if self.spf_verified { count += 1; }
        if self.dkim_verified { count += 1; }
        if self.dmarc_verified { count += 1; }
        (count * 100 / 5) as u8
    }
}

/// Domain-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSettings {
    /// Enable AI auto-reply
    pub ai_auto_reply: bool,
    /// Require human approval for AI responses
    pub require_approval: bool,
    /// Minimum confidence threshold for auto-send (0.0-1.0)
    pub auto_send_threshold: f32,
    /// Enable spam filtering
    pub spam_filtering: bool,
    /// Forward unhandled emails to
    pub forward_to: Option<String>,
    /// Custom greeting for AI responses
    pub ai_greeting: Option<String>,
    /// Custom signature for AI responses
    pub ai_signature: Option<String>,
    /// Business hours (for auto-reply timing)
    pub business_hours: Option<BusinessHours>,
}

impl Default for DomainSettings {
    fn default() -> Self {
        Self {
            ai_auto_reply: true,
            require_approval: true,
            auto_send_threshold: 0.9,
            spam_filtering: true,
            forward_to: None,
            ai_greeting: None,
            ai_signature: None,
            business_hours: None,
        }
    }
}

/// Business hours configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessHours {
    /// Timezone (e.g., "America/New_York")
    pub timezone: String,
    /// Start hour (0-23)
    pub start_hour: u8,
    /// End hour (0-23)
    pub end_hour: u8,
    /// Working days (0=Sunday, 6=Saturday)
    pub working_days: Vec<u8>,
}

// =============================================================================
// 3. Mailbox - Individual Email Address
// =============================================================================

/// Individual mailbox within a domain
/// 
/// Create unlimited mailboxes - each can have its own AI agent!
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mailbox {
    /// Unique mailbox identifier
    pub id: Uuid,
    /// Owning domain ID
    pub domain_id: Uuid,
    /// Local part of email (before @)
    pub local_part: String,
    /// Full email address
    pub email_address: String,
    /// Display name
    pub display_name: Option<String>,
    /// Assigned agent role (e.g., "Receptionist", "CustomerService", "Sales")
    pub agent_role: Option<String>,
    /// Mailbox type
    pub mailbox_type: MailboxType,
    /// Is this mailbox active?
    pub is_active: bool,
    /// Forward emails to
    pub forward_to: Option<String>,
    /// Auto-reply message (when AI is disabled)
    pub auto_reply: Option<String>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

impl Mailbox {
    /// Create a new mailbox with an AI agent
    pub fn new(domain_id: Uuid, local_part: String, domain_name: &str) -> Self {
        let email_address = format!("{}@{}", local_part.to_lowercase(), domain_name);
        Self {
            id: Uuid::new_v4(),
            domain_id,
            local_part: local_part.to_lowercase(),
            email_address,
            display_name: None,
            agent_role: Some("Receptionist".to_string()),
            mailbox_type: MailboxType::Standard,
            is_active: true,
            forward_to: None,
            auto_reply: None,
            created_at: Utc::now(),
        }
    }
    
    /// Create a mailbox with a specific agent role
    pub fn with_agent(domain_id: Uuid, local_part: String, domain_name: &str, agent_role: &str) -> Self {
        let mut mailbox = Self::new(domain_id, local_part, domain_name);
        mailbox.agent_role = Some(agent_role.to_string());
        mailbox
    }
}

/// Type of mailbox
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MailboxType {
    /// Standard mailbox with AI agent
    Standard,
    /// Shared/team mailbox
    Shared,
    /// Alias (forwards to another mailbox)
    Alias,
    /// System mailbox (postmaster, abuse, etc.)
    System,
}

// =============================================================================
// 4. Domain Manager - CRUD Operations
// =============================================================================

/// In-memory storage for domains and mailboxes
#[derive(Debug, Default)]
pub struct DomainStore {
    /// Server configuration
    pub config: Option<ServerConfig>,
    /// All domains
    domains: HashMap<Uuid, Domain>,
    /// All mailboxes
    mailboxes: HashMap<Uuid, Mailbox>,
    /// Index: domain_name -> domain_id
    domain_index: HashMap<String, Uuid>,
    /// Index: email_address -> mailbox_id
    mailbox_index: HashMap<String, Uuid>,
    /// Index: domain_id -> mailbox_ids
    domain_mailboxes: HashMap<Uuid, Vec<Uuid>>,
}

/// Domain manager for self-hosted Amail
/// 
/// No limits. No quotas. Just manage your domains freely.
#[derive(Clone)]
pub struct DomainManager {
    store: Arc<RwLock<DomainStore>>,
}

impl DomainManager {
    /// Create a new domain manager
    pub fn new() -> Self {
        Self {
            store: Arc::new(RwLock::new(DomainStore::default())),
        }
    }
    
    /// Initialize with server config
    pub async fn init(&self, config: ServerConfig) {
        let mut store = self.store.write().await;
        store.config = Some(config);
    }
    
    /// Get server config
    pub async fn get_config(&self) -> Option<ServerConfig> {
        let store = self.store.read().await;
        store.config.clone()
    }
    
    /// Update server config
    pub async fn update_config(&self, config: ServerConfig) {
        let mut store = self.store.write().await;
        store.config = Some(config);
    }
    
    // -------------------------------------------------------------------------
    // Domain Operations - Unlimited!
    // -------------------------------------------------------------------------
    
    /// Add a new domain - no limits!
    pub async fn add_domain(&self, domain_name: String) -> Result<Domain, DomainError> {
        let mut store = self.store.write().await;
        
        // Check domain not already registered
        let domain_lower = domain_name.to_lowercase();
        if store.domain_index.contains_key(&domain_lower) {
            return Err(DomainError::AlreadyExists(domain_name));
        }
        
        // Create domain
        let domain = Domain::new(domain_name);
        store.domains.insert(domain.id, domain.clone());
        store.domain_index.insert(domain_lower, domain.id);
        store.domain_mailboxes.insert(domain.id, Vec::new());
        
        Ok(domain)
    }
    
    /// Get domain by ID
    pub async fn get_domain(&self, domain_id: Uuid) -> Option<Domain> {
        let store = self.store.read().await;
        store.domains.get(&domain_id).cloned()
    }
    
    /// Get domain by name
    pub async fn get_domain_by_name(&self, domain_name: &str) -> Option<Domain> {
        let store = self.store.read().await;
        let domain_id = store.domain_index.get(&domain_name.to_lowercase())?;
        store.domains.get(domain_id).cloned()
    }
    
    /// List all domains
    pub async fn list_domains(&self) -> Vec<Domain> {
        let store = self.store.read().await;
        store.domains.values().cloned().collect()
    }
    
    /// Update domain
    pub async fn update_domain(&self, domain: Domain) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        if !store.domains.contains_key(&domain.id) {
            return Err(DomainError::NotFound);
        }
        store.domains.insert(domain.id, domain);
        Ok(())
    }
    
    /// Delete domain
    pub async fn delete_domain(&self, domain_id: Uuid) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        
        let domain = store.domains.remove(&domain_id)
            .ok_or(DomainError::NotFound)?;
        
        // Remove from index
        store.domain_index.remove(&domain.domain_name);
        
        // Remove all mailboxes for this domain
        if let Some(mailbox_ids) = store.domain_mailboxes.remove(&domain_id) {
            for mailbox_id in mailbox_ids {
                if let Some(mailbox) = store.mailboxes.remove(&mailbox_id) {
                    store.mailbox_index.remove(&mailbox.email_address);
                }
            }
        }
        
        Ok(())
    }
    
    /// Update domain verification status
    pub async fn update_domain_verification(
        &self,
        domain_id: Uuid,
        verification: DnsVerification,
    ) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        let domain = store.domains.get_mut(&domain_id)
            .ok_or(DomainError::NotFound)?;
        domain.update_verification(verification);
        Ok(())
    }
    
    // -------------------------------------------------------------------------
    // Mailbox Operations - Unlimited!
    // -------------------------------------------------------------------------
    
    /// Add mailbox to domain - no limits!
    pub async fn add_mailbox(
        &self,
        domain_id: Uuid,
        local_part: String,
        agent_role: Option<&str>,
    ) -> Result<Mailbox, DomainError> {
        let mut store = self.store.write().await;
        
        // Get domain
        let domain = store.domains.get(&domain_id)
            .ok_or(DomainError::NotFound)?
            .clone();
        
        // Create mailbox
        let mailbox = match agent_role {
            Some(role) => Mailbox::with_agent(domain_id, local_part, &domain.domain_name, role),
            None => Mailbox::new(domain_id, local_part, &domain.domain_name),
        };
        
        // Check email not already registered
        if store.mailbox_index.contains_key(&mailbox.email_address) {
            return Err(DomainError::MailboxExists(mailbox.email_address));
        }
        
        store.mailboxes.insert(mailbox.id, mailbox.clone());
        store.mailbox_index.insert(mailbox.email_address.clone(), mailbox.id);
        store.domain_mailboxes.entry(domain_id).or_default().push(mailbox.id);
        
        Ok(mailbox)
    }
    
    /// Get mailbox by email address
    pub async fn get_mailbox_by_email(&self, email: &str) -> Option<Mailbox> {
        let store = self.store.read().await;
        let mailbox_id = store.mailbox_index.get(&email.to_lowercase())?;
        store.mailboxes.get(mailbox_id).cloned()
    }
    
    /// List mailboxes for domain
    pub async fn list_domain_mailboxes(&self, domain_id: Uuid) -> Vec<Mailbox> {
        let store = self.store.read().await;
        store.domain_mailboxes.get(&domain_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| store.mailboxes.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Update mailbox
    pub async fn update_mailbox(&self, mailbox: Mailbox) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        if !store.mailboxes.contains_key(&mailbox.id) {
            return Err(DomainError::NotFound);
        }
        store.mailboxes.insert(mailbox.id, mailbox);
        Ok(())
    }
    
    /// Delete mailbox
    pub async fn delete_mailbox(&self, mailbox_id: Uuid) -> Result<(), DomainError> {
        let mut store = self.store.write().await;
        
        let mailbox = store.mailboxes.remove(&mailbox_id)
            .ok_or(DomainError::NotFound)?;
        
        store.mailbox_index.remove(&mailbox.email_address);
        
        if let Some(mailbox_ids) = store.domain_mailboxes.get_mut(&mailbox.domain_id) {
            mailbox_ids.retain(|id| *id != mailbox_id);
        }
        
        Ok(())
    }
    
    // -------------------------------------------------------------------------
    // Routing Helper
    // -------------------------------------------------------------------------
    
    /// Get routing info for an email address
    pub async fn get_routing_info(&self, email: &str) -> Option<RoutingInfo> {
        let store = self.store.read().await;
        
        // Try exact mailbox match
        if let Some(mailbox_id) = store.mailbox_index.get(&email.to_lowercase()) {
            if let Some(mailbox) = store.mailboxes.get(mailbox_id) {
                if let Some(domain) = store.domains.get(&mailbox.domain_id) {
                    return Some(RoutingInfo {
                        domain_id: domain.id,
                        mailbox_id: Some(mailbox.id),
                        agent_role: mailbox.agent_role.clone()
                            .or_else(|| domain.default_agent_role.clone()),
                        settings: domain.settings.clone(),
                    });
                }
            }
        }
        
        // Try catch-all for domain
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() == 2 {
            let domain_name = parts[1].to_lowercase();
            if let Some(domain_id) = store.domain_index.get(&domain_name) {
                if let Some(domain) = store.domains.get(domain_id) {
                    if domain.catch_all_mailbox.is_some() {
                        return Some(RoutingInfo {
                            domain_id: domain.id,
                            mailbox_id: None,
                            agent_role: domain.default_agent_role.clone(),
                            settings: domain.settings.clone(),
                        });
                    }
                }
            }
        }
        
        None
    }
    
    /// Get statistics for the server
    pub async fn get_stats(&self) -> ServerStats {
        let store = self.store.read().await;
        ServerStats {
            total_domains: store.domains.len(),
            active_domains: store.domains.values().filter(|d| d.status == DomainStatus::Active).count(),
            total_mailboxes: store.mailboxes.len(),
            active_mailboxes: store.mailboxes.values().filter(|m| m.is_active).count(),
        }
    }
}

impl Default for DomainManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Routing information for an incoming email
#[derive(Debug, Clone)]
pub struct RoutingInfo {
    pub domain_id: Uuid,
    pub mailbox_id: Option<Uuid>,
    pub agent_role: Option<String>,
    pub settings: DomainSettings,
}

/// Server statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStats {
    pub total_domains: usize,
    pub active_domains: usize,
    pub total_mailboxes: usize,
    pub active_mailboxes: usize,
}

/// Domain operation errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum DomainError {
    #[error("Domain not found")]
    NotFound,
    #[error("Domain '{0}' already exists")]
    AlreadyExists(String),
    #[error("Mailbox '{0}' already exists")]
    MailboxExists(String),
    #[error("Invalid domain name: {0}")]
    InvalidDomain(String),
}

// =============================================================================
// Backwards Compatibility - Tenant types (deprecated, kept for migration)
// =============================================================================

/// DEPRECATED: Use Domain directly. Tenant concept removed in favor of self-hosted model.
pub type Tenant = Domain;
pub type TenantManager = DomainManager;
pub type TenantError = DomainError;

/// DEPRECATED: No more tenant types - it's YOUR server!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantType { Personal }

/// DEPRECATED: No more status restrictions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantStatus { Active }

/// DEPRECATED: No more pricing plans - Amail is FREE!
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TenantPlan { Free }

/// DEPRECATED: Branding is now part of ServerConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBranding {
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_add_domain() {
        let manager = DomainManager::new();
        let domain = manager.add_domain("example.com".to_string()).await.unwrap();
        
        assert_eq!(domain.domain_name, "example.com");
        assert_eq!(domain.status, DomainStatus::Pending);
    }
    
    #[tokio::test]
    async fn test_add_mailbox() {
        let manager = DomainManager::new();
        let domain = manager.add_domain("example.com".to_string()).await.unwrap();
        
        let mailbox = manager.add_mailbox(
            domain.id, 
            "support".to_string(),
            Some("CustomerServiceRepresentative"),
        ).await.unwrap();
        
        assert_eq!(mailbox.email_address, "support@example.com");
        assert_eq!(mailbox.agent_role, Some("CustomerServiceRepresentative".to_string()));
    }
    
    #[tokio::test]
    async fn test_unlimited_domains() {
        let manager = DomainManager::new();
        
        // Add many domains - no limits!
        for i in 0..100 {
            let result = manager.add_domain(format!("domain{}.com", i)).await;
            assert!(result.is_ok(), "Should be able to add unlimited domains");
        }
        
        let domains = manager.list_domains().await;
        assert_eq!(domains.len(), 100);
    }
    
    #[tokio::test]
    async fn test_routing() {
        let manager = DomainManager::new();
        let domain = manager.add_domain("example.com".to_string()).await.unwrap();
        
        manager.add_mailbox(
            domain.id,
            "sales".to_string(),
            Some("SalesRepresentative"),
        ).await.unwrap();
        
        let routing = manager.get_routing_info("sales@example.com").await;
        assert!(routing.is_some());
        assert_eq!(routing.unwrap().agent_role, Some("SalesRepresentative".to_string()));
    }
}
