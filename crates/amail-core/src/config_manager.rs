// =============================================================================
// Configuration Manager - JSON-based Email Server Configuration
// =============================================================================
// Table of Contents:
// 1. ConfigManager - Main configuration orchestrator with file watching
// 2. Domain Configuration - Domain settings and DKIM
// 3. Mailbox Configuration - Users, aliases, groups
// 4. DNS Configuration - MX, SPF, DKIM, DMARC records
// 5. SMTP Configuration - Inbound/outbound mail settings
// 6. Agent Configuration - AI agent routing and behavior
// 7. Security Configuration - Spam, rate limiting, content filtering
// =============================================================================

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

// =============================================================================
// 1. ConfigManager
// =============================================================================

/// Main configuration manager with file watching and hot-reload
#[derive(Debug, Clone)]
pub struct ConfigManager {
    config_dir: PathBuf,
    domains: Arc<RwLock<DomainsConfig>>,
    mailboxes: Arc<RwLock<MailboxesConfig>>,
    dns: Arc<RwLock<DnsConfig>>,
    smtp: Arc<RwLock<SmtpConfig>>,
    agents: Arc<RwLock<AgentsConfig>>,
    security: Arc<RwLock<SecurityConfig>>,
}

impl ConfigManager {
    /// Create a new ConfigManager with the given config directory
    pub fn new(config_dir: impl AsRef<Path>) -> Self {
        Self {
            config_dir: config_dir.as_ref().to_path_buf(),
            domains: Arc::new(RwLock::new(DomainsConfig::default())),
            mailboxes: Arc::new(RwLock::new(MailboxesConfig::default())),
            dns: Arc::new(RwLock::new(DnsConfig::default())),
            smtp: Arc::new(RwLock::new(SmtpConfig::default())),
            agents: Arc::new(RwLock::new(AgentsConfig::default())),
            security: Arc::new(RwLock::new(SecurityConfig::default())),
        }
    }

    /// Load all configuration files
    pub async fn load_all(&self) -> Result<(), ConfigError> {
        self.load_domains().await?;
        self.load_mailboxes().await?;
        self.load_dns().await?;
        self.load_smtp().await?;
        self.load_agents().await?;
        self.load_security().await?;
        Ok(())
    }

    /// Load domains configuration
    pub async fn load_domains(&self) -> Result<(), ConfigError> {
        let path = self.config_dir.join("domains.json");
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await
                .map_err(|e| ConfigError::IoError(e.to_string()))?;
            let config: DomainsConfig = serde_json::from_str(&content)
                .map_err(|e| ConfigError::ParseError(format!("domains.json: {}", e)))?;
            *self.domains.write().await = config;
        }
        Ok(())
    }

    /// Load mailboxes configuration
    pub async fn load_mailboxes(&self) -> Result<(), ConfigError> {
        let path = self.config_dir.join("mailboxes.json");
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await
                .map_err(|e| ConfigError::IoError(e.to_string()))?;
            let config: MailboxesConfig = serde_json::from_str(&content)
                .map_err(|e| ConfigError::ParseError(format!("mailboxes.json: {}", e)))?;
            *self.mailboxes.write().await = config;
        }
        Ok(())
    }

    /// Load DNS configuration
    pub async fn load_dns(&self) -> Result<(), ConfigError> {
        let path = self.config_dir.join("dns.json");
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await
                .map_err(|e| ConfigError::IoError(e.to_string()))?;
            let config: DnsConfig = serde_json::from_str(&content)
                .map_err(|e| ConfigError::ParseError(format!("dns.json: {}", e)))?;
            *self.dns.write().await = config;
        }
        Ok(())
    }

    /// Load SMTP configuration
    pub async fn load_smtp(&self) -> Result<(), ConfigError> {
        let path = self.config_dir.join("smtp.json");
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await
                .map_err(|e| ConfigError::IoError(e.to_string()))?;
            let config: SmtpConfig = serde_json::from_str(&content)
                .map_err(|e| ConfigError::ParseError(format!("smtp.json: {}", e)))?;
            *self.smtp.write().await = config;
        }
        Ok(())
    }

    /// Load agents configuration
    pub async fn load_agents(&self) -> Result<(), ConfigError> {
        let path = self.config_dir.join("agents.json");
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await
                .map_err(|e| ConfigError::IoError(e.to_string()))?;
            let config: AgentsConfig = serde_json::from_str(&content)
                .map_err(|e| ConfigError::ParseError(format!("agents.json: {}", e)))?;
            *self.agents.write().await = config;
        }
        Ok(())
    }

    /// Load security configuration
    pub async fn load_security(&self) -> Result<(), ConfigError> {
        let path = self.config_dir.join("security.json");
        if path.exists() {
            let content = tokio::fs::read_to_string(&path).await
                .map_err(|e| ConfigError::IoError(e.to_string()))?;
            let config: SecurityConfig = serde_json::from_str(&content)
                .map_err(|e| ConfigError::ParseError(format!("security.json: {}", e)))?;
            *self.security.write().await = config;
        }
        Ok(())
    }

    /// Save domains configuration
    pub async fn save_domains(&self) -> Result<(), ConfigError> {
        let path = self.config_dir.join("domains.json");
        let mut config = self.domains.write().await;
        config.updated_at = Some(Utc::now());
        let content = serde_json::to_string_pretty(&*config)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        tokio::fs::write(&path, content).await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Save mailboxes configuration
    pub async fn save_mailboxes(&self) -> Result<(), ConfigError> {
        let path = self.config_dir.join("mailboxes.json");
        let mut config = self.mailboxes.write().await;
        config.updated_at = Some(Utc::now());
        let content = serde_json::to_string_pretty(&*config)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        tokio::fs::write(&path, content).await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        Ok(())
    }

    /// Save DNS configuration
    pub async fn save_dns(&self) -> Result<(), ConfigError> {
        let path = self.config_dir.join("dns.json");
        let mut config = self.dns.write().await;
        config.updated_at = Some(Utc::now());
        let content = serde_json::to_string_pretty(&*config)
            .map_err(|e| ConfigError::SerializeError(e.to_string()))?;
        tokio::fs::write(&path, content).await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        Ok(())
    }

    // =========================================================================
    // Domain Operations
    // =========================================================================

    /// Get all domains
    pub async fn get_domains(&self) -> Vec<DomainConfig> {
        self.domains.read().await.domains.clone()
    }

    /// Get domain by ID (case-insensitive)
    pub async fn get_domain(&self, id: &str) -> Option<DomainConfig> {
        let id_lower = id.to_lowercase();
        self.domains.read().await.domains.iter()
            .find(|d| d.id.to_lowercase() == id_lower)
            .cloned()
    }

    /// Add a new domain
    pub async fn add_domain(&self, mut domain: DomainConfig) -> Result<DomainConfig, ConfigError> {
        let mut config = self.domains.write().await;
        
        // Check for duplicate (case-insensitive)
        let name_lower = domain.name.to_lowercase();
        if config.domains.iter().any(|d| d.name.to_lowercase() == name_lower) {
            return Err(ConfigError::DuplicateEntry(format!("Domain {} already exists", domain.name)));
        }

        // Generate ID if not set (lowercase for case-insensitive lookups)
        if domain.id.is_empty() {
            domain.id = domain.name.to_lowercase().replace('.', "-");
        }
        domain.created_at = Utc::now();

        config.domains.push(domain.clone());
        drop(config);
        self.save_domains().await?;
        Ok(domain)
    }

    /// Update a domain
    pub async fn update_domain(&self, id: &str, updates: DomainConfig) -> Result<DomainConfig, ConfigError> {
        let mut config = self.domains.write().await;
        
        if let Some(domain) = config.domains.iter_mut().find(|d| d.id == id) {
            domain.name = updates.name;
            domain.display_name = updates.display_name;
            domain.enabled = updates.enabled;
            domain.settings = updates.settings;
            domain.dkim = updates.dkim;
            domain.branding = updates.branding;
            let updated = domain.clone();
            drop(config);
            self.save_domains().await?;
            Ok(updated)
        } else {
            Err(ConfigError::NotFound(format!("Domain {} not found", id)))
        }
    }

    /// Update DNS verification status for a domain
    pub async fn update_dns_verification(
        &self,
        id: &str,
        mx_verified: bool,
        spf_verified: bool,
        dkim_verified: bool,
        dmarc_verified: bool,
    ) -> Result<DomainConfig, ConfigError> {
        let mut config = self.domains.write().await;
        
        if let Some(domain) = config.domains.iter_mut().find(|d| d.id.to_lowercase() == id.to_lowercase()) {
            domain.mx_verified = mx_verified;
            domain.spf_verified = spf_verified;
            domain.dkim_verified = dkim_verified;
            domain.dmarc_verified = dmarc_verified;
            domain.last_verified = Some(Utc::now());
            let updated = domain.clone();
            drop(config);
            self.save_domains().await?;
            Ok(updated)
        } else {
            Err(ConfigError::NotFound(format!("Domain {} not found", id)))
        }
    }

    /// Delete a domain
    pub async fn delete_domain(&self, id: &str) -> Result<(), ConfigError> {
        let mut config = self.domains.write().await;
        let initial_len = config.domains.len();
        config.domains.retain(|d| d.id != id);
        
        if config.domains.len() == initial_len {
            return Err(ConfigError::NotFound(format!("Domain {} not found", id)));
        }
        
        drop(config);
        self.save_domains().await?;
        
        // Also remove associated mailboxes
        let mut mailboxes = self.mailboxes.write().await;
        mailboxes.mailboxes.retain(|m| m.domain_id != id);
        mailboxes.aliases.retain(|a| a.domain_id != id);
        mailboxes.groups.retain(|g| g.domain_id != id);
        drop(mailboxes);
        self.save_mailboxes().await?;
        
        Ok(())
    }

    // =========================================================================
    // Mailbox Operations
    // =========================================================================

    /// Get all mailboxes for a domain (case-insensitive)
    pub async fn get_mailboxes(&self, domain_id: &str) -> Vec<MailboxConfig> {
        let domain_id_lower = domain_id.to_lowercase();
        self.mailboxes.read().await.mailboxes.iter()
            .filter(|m| m.domain_id.to_lowercase() == domain_id_lower)
            .cloned()
            .collect()
    }

    /// Get all mailboxes across all domains
    pub async fn get_all_mailboxes(&self) -> Vec<MailboxConfig> {
        self.mailboxes.read().await.mailboxes.clone()
    }

    /// Get mailbox by ID
    pub async fn get_mailbox(&self, id: &str) -> Option<MailboxConfig> {
        self.mailboxes.read().await.mailboxes.iter().find(|m| m.id == id).cloned()
    }

    /// Add a new mailbox
    pub async fn add_mailbox(&self, mut mailbox: MailboxConfig) -> Result<MailboxConfig, ConfigError> {
        // Verify domain exists
        if self.get_domain(&mailbox.domain_id).await.is_none() {
            return Err(ConfigError::NotFound(format!("Domain {} not found", mailbox.domain_id)));
        }

        let mut config = self.mailboxes.write().await;
        
        // Check for duplicate email
        if config.mailboxes.iter().any(|m| m.email == mailbox.email) {
            return Err(ConfigError::DuplicateEntry(format!("Mailbox {} already exists", mailbox.email)));
        }

        // Generate ID if not set
        if mailbox.id.is_empty() {
            mailbox.id = mailbox.email.replace('@', "-").replace('.', "-");
        }
        mailbox.created_at = Utc::now();

        config.mailboxes.push(mailbox.clone());
        drop(config);
        self.save_mailboxes().await?;
        Ok(mailbox)
    }

    /// Update a mailbox
    pub async fn update_mailbox(&self, id: &str, updates: MailboxConfig) -> Result<MailboxConfig, ConfigError> {
        let mut config = self.mailboxes.write().await;
        
        if let Some(mailbox) = config.mailboxes.iter_mut().find(|m| m.id == id) {
            mailbox.display_name = updates.display_name;
            mailbox.enabled = updates.enabled;
            mailbox.quota_mb = updates.quota_mb;
            mailbox.settings = updates.settings;
            mailbox.aliases = updates.aliases;
            mailbox.ai_agent = updates.ai_agent;
            let updated = mailbox.clone();
            drop(config);
            self.save_mailboxes().await?;
            Ok(updated)
        } else {
            Err(ConfigError::NotFound(format!("Mailbox {} not found", id)))
        }
    }

    /// Delete a mailbox
    pub async fn delete_mailbox(&self, id: &str) -> Result<(), ConfigError> {
        let mut config = self.mailboxes.write().await;
        let initial_len = config.mailboxes.len();
        config.mailboxes.retain(|m| m.id != id);
        
        if config.mailboxes.len() == initial_len {
            return Err(ConfigError::NotFound(format!("Mailbox {} not found", id)));
        }
        
        drop(config);
        self.save_mailboxes().await
    }

    // =========================================================================
    // DNS Record Generation
    // =========================================================================

    /// Generate DNS records for a domain (copy-paste format)
    /// If no DNS config exists, generates sensible defaults based on the domain
    pub async fn generate_dns_records(&self, domain_id: &str) -> Result<Vec<DnsRecord>, ConfigError> {
        let dns_config = self.dns.read().await;
        let server_ip = dns_config.server.ipv4.clone().unwrap_or_else(|| "YOUR_SERVER_IP".to_string());
        
        // Try to find existing DNS config for this domain
        let existing_config = dns_config.domains.iter().find(|d| d.domain_id == domain_id).cloned();
        drop(dns_config); // Release the lock early
        
        if let Some(domain_dns) = existing_config {
            // Use existing config
            let mut records = Vec::new();

            // MX Record
            if domain_dns.records.mx.enabled {
                records.push(DnsRecord {
                    record_type: "MX".to_string(),
                    name: "@".to_string(),
                    value: domain_dns.records.mx.host.clone(),
                    priority: Some(domain_dns.records.mx.priority),
                    ttl: domain_dns.records.mx.ttl,
                });
            }

            // A Record for mail subdomain
            records.push(DnsRecord {
                record_type: "A".to_string(),
                name: "mail".to_string(),
                value: server_ip.clone(),
                priority: None,
                ttl: 3600,
            });

            // SPF Record
            if domain_dns.records.spf.enabled {
                let spf_value = domain_dns.records.spf.policy.replace("{server_ip}", &server_ip);
                records.push(DnsRecord {
                    record_type: "TXT".to_string(),
                    name: "@".to_string(),
                    value: spf_value,
                    priority: None,
                    ttl: domain_dns.records.spf.ttl,
                });
            }

            // DKIM Record
            if domain_dns.records.dkim.enabled {
                if let Some(ref public_key) = domain_dns.records.dkim.public_key {
                    records.push(DnsRecord {
                        record_type: "TXT".to_string(),
                        name: format!("{}._domainkey", domain_dns.records.dkim.selector),
                        value: format!("v=DKIM1; k=rsa; p={}", public_key),
                        priority: None,
                        ttl: domain_dns.records.dkim.ttl,
                    });
                }
            }

            // DMARC Record
            if domain_dns.records.dmarc.enabled {
                let dmarc_value = format!(
                    "v=DMARC1; p={}; rua=mailto:{}; pct={}",
                    domain_dns.records.dmarc.policy,
                    domain_dns.records.dmarc.rua,
                    domain_dns.records.dmarc.pct
                );
                records.push(DnsRecord {
                    record_type: "TXT".to_string(),
                    name: "_dmarc".to_string(),
                    value: dmarc_value,
                    priority: None,
                    ttl: domain_dns.records.dmarc.ttl,
                });
            }

            return Ok(records);
        }
        
        // No existing config - generate defaults based on domain info
        let domain = self.get_domain(domain_id).await
            .ok_or_else(|| ConfigError::NotFound(format!("Domain {} not found", domain_id)))?;
        
        let domain_name = &domain.name;
        let mut records = Vec::new();
        
        // MX Record (Required) - point to mail.domain.com
        records.push(DnsRecord {
            record_type: "MX".to_string(),
            name: "@".to_string(),
            value: format!("mail.{}", domain_name),
            priority: Some(10),
            ttl: 3600,
        });
        
        // A Record for mail subdomain
        records.push(DnsRecord {
            record_type: "A".to_string(),
            name: "mail".to_string(),
            value: server_ip.clone(),
            priority: None,
            ttl: 3600,
        });
        
        // SPF Record (Required) - authorize this server
        records.push(DnsRecord {
            record_type: "TXT".to_string(),
            name: "@".to_string(),
            value: format!("v=spf1 ip4:{} ~all", server_ip),
            priority: None,
            ttl: 3600,
        });
        
        // DKIM Record (Optional) - placeholder, needs key generation
        records.push(DnsRecord {
            record_type: "TXT".to_string(),
            name: "default._domainkey".to_string(),
            value: "v=DKIM1; k=rsa; p=YOUR_DKIM_PUBLIC_KEY".to_string(),
            priority: None,
            ttl: 3600,
        });
        
        // DMARC Record (Optional) - basic policy
        records.push(DnsRecord {
            record_type: "TXT".to_string(),
            name: "_dmarc".to_string(),
            value: format!("v=DMARC1; p=none; rua=mailto:postmaster@{}", domain_name),
            priority: None,
            ttl: 3600,
        });

        Ok(records)
    }

    /// Format DNS records as copy-paste text
    pub async fn format_dns_records(&self, domain_id: &str) -> Result<String, ConfigError> {
        let records = self.generate_dns_records(domain_id).await?;
        let domain = self.get_domain(domain_id).await
            .ok_or_else(|| ConfigError::NotFound(format!("Domain {} not found", domain_id)))?;

        let mut output = format!("# DNS Records for {}\n", domain.name);
        output.push_str("# Copy these records to your DNS provider\n\n");

        for record in records {
            let priority = record.priority.map(|p| format!(" {}", p)).unwrap_or_default();
            output.push_str(&format!(
                "Type: {}\nName: {}\nValue: {}{}\nTTL: {}\n\n",
                record.record_type,
                record.name,
                record.value,
                priority,
                record.ttl
            ));
        }

        Ok(output)
    }

    // =========================================================================
    // Accessors
    // =========================================================================

    pub async fn get_smtp_config(&self) -> SmtpConfig {
        self.smtp.read().await.clone()
    }

    pub async fn get_agents_config(&self) -> AgentsConfig {
        self.agents.read().await.clone()
    }

    pub async fn get_security_config(&self) -> SecurityConfig {
        self.security.read().await.clone()
    }

    /// Get the config directory path
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    // =========================================================================
    // File Watching
    // =========================================================================

    /// Start watching config files for changes and auto-reload
    /// Returns a handle that can be used to stop watching
    pub fn start_watching(self: Arc<Self>) -> Result<ConfigWatcher, ConfigError> {
        use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
        use std::sync::mpsc::channel;

        let (tx, rx) = channel();
        let config_dir = self.config_dir.clone();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default(),
        ).map_err(|e| ConfigError::IoError(format!("Failed to create watcher: {}", e)))?;

        watcher.watch(&config_dir, RecursiveMode::NonRecursive)
            .map_err(|e| ConfigError::IoError(format!("Failed to watch directory: {}", e)))?;

        let manager = self.clone();
        let handle = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            
            for event in rx {
                if let notify::EventKind::Modify(_) = event.kind {
                    for path in event.paths {
                        if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                            let manager = manager.clone();
                            rt.block_on(async {
                                let result = match filename {
                                    "domains.json" => manager.load_domains().await,
                                    "mailboxes.json" => manager.load_mailboxes().await,
                                    "dns.json" => manager.load_dns().await,
                                    "smtp.json" => manager.load_smtp().await,
                                    "agents.json" => manager.load_agents().await,
                                    "security.json" => manager.load_security().await,
                                    _ => Ok(()),
                                };
                                if let Err(e) = result {
                                    tracing::error!("Failed to reload {}: {}", filename, e);
                                } else {
                                    tracing::info!("Reloaded config: {}", filename);
                                }
                            });
                        }
                    }
                }
            }
        });

        Ok(ConfigWatcher {
            _watcher: watcher,
            _handle: handle,
        })
    }
}

/// Handle for the config file watcher
/// Dropping this will stop watching
pub struct ConfigWatcher {
    _watcher: notify::RecommendedWatcher,
    _handle: std::thread::JoinHandle<()>,
}

// =============================================================================
// 2. Domain Configuration
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomainsConfig {
    pub version: String,
    pub updated_at: Option<DateTime<Utc>>,
    pub domains: Vec<DomainConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub enabled: bool,
    #[serde(default)]
    pub primary: bool,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub settings: DomainSettings,
    #[serde(default)]
    pub dkim: DkimConfig,
    #[serde(default)]
    pub branding: DomainBranding,
    #[serde(default)]
    pub mx_verified: bool,
    #[serde(default)]
    pub spf_verified: bool,
    #[serde(default)]
    pub dkim_verified: bool,
    #[serde(default)]
    pub dmarc_verified: bool,
    #[serde(default)]
    pub last_verified: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomainSettings {
    pub catch_all: Option<String>,
    #[serde(default = "default_quota")]
    pub default_quota_mb: u64,
    pub max_mailboxes: Option<u32>,
    #[serde(default = "default_max_aliases")]
    pub max_aliases_per_mailbox: u32,
}

fn default_quota() -> u64 { 5120 }
fn default_max_aliases() -> u32 { 10 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DkimConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_selector")]
    pub selector: String,
    #[serde(default = "default_key_type")]
    pub key_type: String,
    #[serde(default = "default_key_bits")]
    pub key_bits: u32,
    pub public_key: Option<String>,
    pub private_key_path: Option<String>,
    pub generated_at: Option<DateTime<Utc>>,
}

fn default_true() -> bool { true }
fn default_selector() -> String { "default".to_string() }
fn default_key_type() -> String { "rsa".to_string() }
fn default_key_bits() -> u32 { 2048 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomainBranding {
    pub logo_url: Option<String>,
    #[serde(default = "default_color")]
    pub primary_color: String,
    pub support_email: Option<String>,
}

fn default_color() -> String { "#3B82F6".to_string() }

// =============================================================================
// 3. Mailbox Configuration
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MailboxesConfig {
    pub version: String,
    pub updated_at: Option<DateTime<Utc>>,
    pub mailboxes: Vec<MailboxConfig>,
    #[serde(default)]
    pub aliases: Vec<AliasConfig>,
    #[serde(default)]
    pub groups: Vec<GroupConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxConfig {
    pub id: String,
    pub domain_id: String,
    pub email: String,
    pub display_name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "default_mailbox_type")]
    pub r#type: String,
    pub password_hash: Option<String>,
    #[serde(default = "default_quota")]
    pub quota_mb: u64,
    #[serde(default)]
    pub settings: MailboxSettings,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub ai_agent: MailboxAgentConfig,
}

fn default_mailbox_type() -> String { "user".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MailboxSettings {
    #[serde(default)]
    pub auto_reply_enabled: bool,
    pub auto_reply_subject: Option<String>,
    pub auto_reply_body: Option<String>,
    pub auto_reply_start: Option<DateTime<Utc>>,
    pub auto_reply_end: Option<DateTime<Utc>>,
    pub forward_to: Option<String>,
    #[serde(default = "default_true")]
    pub forward_keep_copy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MailboxAgentConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_agent_role")]
    pub role: String,
    #[serde(default)]
    pub auto_respond: bool,
    #[serde(default = "default_true")]
    pub require_review: bool,
}

fn default_agent_role() -> String { "receptionist".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasConfig {
    pub id: String,
    pub domain_id: String,
    pub email: String,
    pub forward_to: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupConfig {
    pub id: String,
    pub domain_id: String,
    pub email: String,
    pub display_name: String,
    pub members: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// 4. DNS Configuration
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DnsConfig {
    pub version: String,
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub server: DnsServerConfig,
    #[serde(default)]
    pub domains: Vec<DomainDnsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DnsServerConfig {
    pub hostname: String,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainDnsConfig {
    pub domain_id: String,
    pub domain: String,
    pub records: DnsRecordsConfig,
    #[serde(default)]
    pub verification: DnsVerification,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DnsRecordsConfig {
    pub mx: MxRecordConfig,
    pub spf: SpfRecordConfig,
    pub dkim: DkimRecordConfig,
    pub dmarc: DmarcRecordConfig,
    #[serde(default)]
    pub autodiscover: AutodiscoverConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MxRecordConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_mx_priority")]
    pub priority: u16,
    pub host: String,
    #[serde(default = "default_ttl")]
    pub ttl: u32,
}

fn default_mx_priority() -> u16 { 10 }
fn default_ttl() -> u32 { 3600 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpfRecordConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_spf_policy")]
    pub policy: String,
    #[serde(default = "default_ttl")]
    pub ttl: u32,
}

fn default_spf_policy() -> String { "v=spf1 mx a -all".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DkimRecordConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_selector")]
    pub selector: String,
    pub public_key: Option<String>,
    #[serde(default = "default_ttl")]
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DmarcRecordConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_dmarc_policy")]
    pub policy: String,
    #[serde(default)]
    pub rua: String,
    pub ruf: Option<String>,
    #[serde(default = "default_pct")]
    pub pct: u8,
    #[serde(default = "default_ttl")]
    pub ttl: u32,
}

fn default_dmarc_policy() -> String { "quarantine".to_string() }
fn default_pct() -> u8 { 100 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutodiscoverConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub cname: Option<String>,
    #[serde(default = "default_ttl")]
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DnsVerification {
    #[serde(default)]
    pub mx_verified: bool,
    #[serde(default)]
    pub spf_verified: bool,
    #[serde(default)]
    pub dkim_verified: bool,
    #[serde(default)]
    pub dmarc_verified: bool,
    pub last_checked: Option<DateTime<Utc>>,
}

/// DNS record for output/display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub record_type: String,
    pub name: String,
    pub value: String,
    pub priority: Option<u16>,
    pub ttl: u32,
}

// =============================================================================
// 5. SMTP Configuration
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmtpConfig {
    pub version: String,
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub inbound: InboundSmtpConfig,
    #[serde(default)]
    pub outbound: OutboundSmtpConfig,
    #[serde(default)]
    pub authentication: SmtpAuthConfig,
    #[serde(default)]
    pub tls: TlsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InboundSmtpConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    #[serde(default)]
    pub tls_required: bool,
    #[serde(default = "default_max_message_size")]
    pub max_message_size_mb: u32,
    #[serde(default = "default_max_recipients")]
    pub max_recipients: u32,
    #[serde(default = "default_greeting")]
    pub greeting: String,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,
}

fn default_smtp_port() -> u16 { 25 }
fn default_max_message_size() -> u32 { 50 }
fn default_max_recipients() -> u32 { 100 }
fn default_greeting() -> String { "Amail ESMTP Server".to_string() }
fn default_timeout() -> u32 { 300 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutboundSmtpConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_submission_port")]
    pub port: u16,
    #[serde(default = "default_true")]
    pub tls_required: bool,
    #[serde(default)]
    pub relay: SmtpRelayConfig,
    #[serde(default)]
    pub rate_limit: SmtpRateLimitConfig,
    #[serde(default)]
    pub retry: SmtpRetryConfig,
}

fn default_submission_port() -> u16 { 587 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmtpRelayConfig {
    #[serde(default)]
    pub enabled: bool,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(default = "default_auth_method")]
    pub auth_method: String,
}

fn default_auth_method() -> String { "plain".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmtpRateLimitConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_rate_per_hour")]
    pub max_per_hour: u32,
    #[serde(default = "default_rate_per_day")]
    pub max_per_day: u32,
}

fn default_rate_per_hour() -> u32 { 500 }
fn default_rate_per_day() -> u32 { 5000 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmtpRetryConfig {
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,
    #[serde(default = "default_initial_delay")]
    pub initial_delay_seconds: u32,
    #[serde(default = "default_max_delay")]
    pub max_delay_seconds: u32,
}

fn default_max_attempts() -> u32 { 5 }
fn default_initial_delay() -> u32 { 300 }
fn default_max_delay() -> u32 { 86400 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmtpAuthConfig {
    #[serde(default = "default_true")]
    pub require_auth: bool,
    #[serde(default = "default_auth_methods")]
    pub allowed_methods: Vec<String>,
    #[serde(default)]
    pub allow_plain_without_tls: bool,
}

fn default_auth_methods() -> Vec<String> { vec!["plain".to_string(), "login".to_string()] }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TlsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub certificate_path: Option<String>,
    pub private_key_path: Option<String>,
    #[serde(default = "default_true")]
    pub auto_renew: bool,
    #[serde(default = "default_tls_provider")]
    pub provider: String,
}

fn default_tls_provider() -> String { "letsencrypt".to_string() }

// =============================================================================
// 6. Agent Configuration
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentsConfig {
    pub version: String,
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub global: GlobalAgentConfig,
    #[serde(default)]
    pub domain_overrides: Vec<DomainAgentOverride>,
    #[serde(default)]
    pub mailbox_overrides: Vec<MailboxAgentOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalAgentConfig {
    #[serde(default)]
    pub llm: LlmConfig,
    #[serde(default)]
    pub confidence_thresholds: ConfidenceThresholds,
    #[serde(default)]
    pub review: ReviewConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmConfig {
    #[serde(default = "default_llm_provider")]
    pub provider: String,
    #[serde(default = "default_ollama_url")]
    pub url: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,
}

fn default_llm_provider() -> String { "ollama".to_string() }
fn default_ollama_url() -> String { "http://localhost:11434".to_string() }
fn default_model() -> String { "mistral:latest".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceThresholds {
    #[serde(default = "default_auto_respond_threshold")]
    pub auto_respond: f32,
    #[serde(default = "default_require_review_threshold")]
    pub require_review: f32,
    #[serde(default = "default_escalate_threshold")]
    pub escalate_human: f32,
}

impl Default for ConfidenceThresholds {
    fn default() -> Self {
        Self {
            auto_respond: 0.85,
            require_review: 0.60,
            escalate_human: 0.40,
        }
    }
}

fn default_auto_respond_threshold() -> f32 { 0.85 }
fn default_require_review_threshold() -> f32 { 0.60 }
fn default_escalate_threshold() -> f32 { 0.40 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReviewConfig {
    #[serde(default)]
    pub telegram_enabled: bool,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
    #[serde(default)]
    pub email_review_enabled: bool,
    pub review_email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainAgentOverride {
    pub domain_id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub auto_respond: bool,
    #[serde(default = "default_true")]
    pub require_review: bool,
    #[serde(default)]
    pub working_hours: Option<WorkingHoursConfig>,
    #[serde(default)]
    pub agent_routing: Option<AgentRoutingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkingHoursConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_timezone")]
    pub timezone: String,
    #[serde(default = "default_start_time")]
    pub start: String,
    #[serde(default = "default_end_time")]
    pub end: String,
    #[serde(default = "default_work_days")]
    pub days: Vec<String>,
}

fn default_timezone() -> String { "America/Los_Angeles".to_string() }
fn default_start_time() -> String { "08:00".to_string() }
fn default_end_time() -> String { "18:00".to_string() }
fn default_work_days() -> Vec<String> { 
    vec!["mon".to_string(), "tue".to_string(), "wed".to_string(), "thu".to_string(), "fri".to_string()] 
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentRoutingConfig {
    #[serde(default = "default_agent_role")]
    pub default_agent: String,
    #[serde(default)]
    pub rules: Vec<RoutingRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub condition: String,
    pub value: String,
    pub route_to: String,
    #[serde(default = "default_priority")]
    pub priority: u32,
}

fn default_priority() -> u32 { 10 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxAgentOverride {
    pub mailbox_id: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub auto_respond: bool,
    #[serde(default = "default_true")]
    pub require_review: bool,
    #[serde(default = "default_agent_role")]
    pub agent_role: String,
}

// =============================================================================
// 7. Security Configuration
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecurityConfig {
    pub version: String,
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub spam: SpamConfig,
    #[serde(default)]
    pub rate_limiting: RateLimitingConfig,
    #[serde(default)]
    pub auto_reply_prevention: AutoReplyPreventionConfig,
    #[serde(default)]
    pub content_filtering: ContentFilteringConfig,
    #[serde(default)]
    pub audit: AuditConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpamConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_spam_threshold")]
    pub spam_threshold: f32,
    #[serde(default = "default_quarantine_threshold")]
    pub quarantine_threshold: f32,
    #[serde(default = "default_reject_threshold")]
    pub reject_threshold: f32,
    #[serde(default = "default_true")]
    pub check_spf: bool,
    #[serde(default = "default_true")]
    pub check_dkim: bool,
    #[serde(default = "default_true")]
    pub check_dmarc: bool,
    #[serde(default)]
    pub blocked_domains: Vec<String>,
    #[serde(default)]
    pub blocked_addresses: Vec<String>,
    #[serde(default)]
    pub allowed_domains: Vec<String>,
    #[serde(default)]
    pub allowed_addresses: Vec<String>,
}

impl Default for SpamConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            spam_threshold: 50.0,
            quarantine_threshold: 80.0,
            reject_threshold: 95.0,
            check_spf: true,
            check_dkim: true,
            check_dmarc: true,
            blocked_domains: Vec::new(),
            blocked_addresses: Vec::new(),
            allowed_domains: Vec::new(),
            allowed_addresses: Vec::new(),
        }
    }
}

fn default_spam_threshold() -> f32 { 50.0 }
fn default_quarantine_threshold() -> f32 { 80.0 }
fn default_reject_threshold() -> f32 { 95.0 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RateLimitingConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub login_attempts: LoginRateLimitConfig,
    #[serde(default)]
    pub api_requests: ApiRateLimitConfig,
    #[serde(default)]
    pub email_sending: EmailRateLimitConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRateLimitConfig {
    #[serde(default = "default_login_attempts")]
    pub max_attempts: u32,
    #[serde(default = "default_lockout_minutes")]
    pub lockout_minutes: u32,
}

impl Default for LoginRateLimitConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            lockout_minutes: 15,
        }
    }
}

fn default_login_attempts() -> u32 { 5 }
fn default_lockout_minutes() -> u32 { 15 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRateLimitConfig {
    #[serde(default = "default_api_per_minute")]
    pub max_per_minute: u32,
    #[serde(default = "default_api_per_hour")]
    pub max_per_hour: u32,
}

impl Default for ApiRateLimitConfig {
    fn default() -> Self {
        Self {
            max_per_minute: 60,
            max_per_hour: 1000,
        }
    }
}

fn default_api_per_minute() -> u32 { 60 }
fn default_api_per_hour() -> u32 { 1000 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRateLimitConfig {
    #[serde(default = "default_email_per_minute")]
    pub max_per_minute: u32,
    #[serde(default = "default_email_per_hour")]
    pub max_per_hour: u32,
}

impl Default for EmailRateLimitConfig {
    fn default() -> Self {
        Self {
            max_per_minute: 10,
            max_per_hour: 100,
        }
    }
}

fn default_email_per_minute() -> u32 { 10 }
fn default_email_per_hour() -> u32 { 100 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoReplyPreventionConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub block_noreply: bool,
    #[serde(default = "default_true")]
    pub block_mailer_daemon: bool,
    #[serde(default = "default_true")]
    pub block_automated_headers: bool,
    #[serde(default = "default_true")]
    pub block_mailing_lists: bool,
    #[serde(default = "default_blocked_sender_patterns")]
    pub blocked_sender_patterns: Vec<String>,
    #[serde(default = "default_blocked_auto_domains")]
    pub blocked_domains: Vec<String>,
}

impl Default for AutoReplyPreventionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            block_noreply: true,
            block_mailer_daemon: true,
            block_automated_headers: true,
            block_mailing_lists: true,
            blocked_sender_patterns: default_blocked_sender_patterns(),
            blocked_domains: default_blocked_auto_domains(),
        }
    }
}

fn default_blocked_sender_patterns() -> Vec<String> {
    vec![
        "noreply@".to_string(),
        "no-reply@".to_string(),
        "donotreply@".to_string(),
        "mailer-daemon@".to_string(),
        "postmaster@".to_string(),
        "bounce@".to_string(),
        "notifications@".to_string(),
    ]
}

fn default_blocked_auto_domains() -> Vec<String> {
    vec![
        "sendgrid.net".to_string(),
        "mailchimp.com".to_string(),
        "amazonses.com".to_string(),
        "mailgun.org".to_string(),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentFilteringConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_max_attachment_size")]
    pub max_attachment_size_mb: u32,
    #[serde(default = "default_blocked_extensions")]
    pub blocked_extensions: Vec<String>,
    #[serde(default = "default_true")]
    pub scan_attachments: bool,
    #[serde(default = "default_true")]
    pub strip_dangerous_html: bool,
}

impl Default for ContentFilteringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attachment_size_mb: 25,
            blocked_extensions: default_blocked_extensions(),
            scan_attachments: true,
            strip_dangerous_html: true,
        }
    }
}

fn default_max_attachment_size() -> u32 { 25 }
fn default_blocked_extensions() -> Vec<String> {
    vec![
        ".exe".to_string(),
        ".bat".to_string(),
        ".cmd".to_string(),
        ".scr".to_string(),
        ".pif".to_string(),
        ".js".to_string(),
        ".vbs".to_string(),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub log_logins: bool,
    #[serde(default = "default_true")]
    pub log_email_access: bool,
    #[serde(default = "default_true")]
    pub log_config_changes: bool,
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_logins: true,
            log_email_access: true,
            log_config_changes: true,
            retention_days: 90,
        }
    }
}

fn default_retention_days() -> u32 { 90 }

// =============================================================================
// Error Types
// =============================================================================

#[derive(Debug, Clone)]
pub enum ConfigError {
    IoError(String),
    ParseError(String),
    SerializeError(String),
    ValidationError(String),
    NotFound(String),
    DuplicateEntry(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError(msg) => write!(f, "IO error: {}", msg),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::SerializeError(msg) => write!(f, "Serialize error: {}", msg),
            Self::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::DuplicateEntry(msg) => write!(f, "Duplicate entry: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_manager_new() {
        let manager = ConfigManager::new("./data/config");
        assert!(manager.get_domains().await.is_empty());
    }

    #[test]
    fn test_domain_config_serialization() {
        let domain = DomainConfig {
            id: "test-com".to_string(),
            name: "test.com".to_string(),
            display_name: "Test Organization".to_string(),
            enabled: true,
            primary: true,
            created_at: Utc::now(),
            settings: DomainSettings::default(),
            dkim: DkimConfig::default(),
            branding: DomainBranding::default(),
        };

        let json = serde_json::to_string_pretty(&domain).unwrap();
        let parsed: DomainConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "test.com");
    }
}
