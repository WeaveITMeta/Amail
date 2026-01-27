//! Stalwart Mail Server Client - Real email fetching and sending
//!
//! Table of Contents:
//! 1. StalwartClient - HTTP client for Stalwart REST API
//! 2. Email fetching (JMAP-like API)
//! 3. Email sending
//! 4. Mailbox management
//! 5. Domain management
//! 6. ConfigApplier - Apply JSON configs to Stalwart

use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use crate::{AmailError, Result};

// =============================================================================
// 1. Stalwart Client
// =============================================================================

/// Client for interacting with Stalwart Mail Server REST API
#[derive(Clone)]
pub struct StalwartClient {
    client: Client,
    base_url: String,
    admin_token: Option<String>,
}

impl StalwartClient {
    /// Create a new Stalwart client
    pub fn new(base_url: &str, admin_token: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            admin_token,
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let base_url = std::env::var("STALWART_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        let admin_token = std::env::var("STALWART_ADMIN_TOKEN").ok();
        
        Ok(Self::new(&base_url, admin_token))
    }

    /// Test connection to Stalwart
    pub async fn health_check(&self) -> Result<bool> {
        let response = self.client
            .get(format!("{}/healthz", self.base_url))
            .send()
            .await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        Ok(response.status().is_success())
    }

    /// Build request with auth header
    fn auth_header(&self) -> Option<String> {
        self.admin_token.as_ref().map(|t| format!("Bearer {}", t))
    }

    // -------------------------------------------------------------------------
    // 2. Email Fetching
    // -------------------------------------------------------------------------

    /// Fetch emails from a mailbox
    pub async fn fetch_emails(
        &self,
        account: &str,
        folder: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<StalwartEmail>> {
        let url = format!(
            "{}/api/v1/accounts/{}/mailboxes/{}/messages?limit={}&offset={}",
            self.base_url, account, folder, limit, offset
        );

        let mut request = self.client.get(&url);
        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AmailError::Network(format!(
                "Failed to fetch emails: {}",
                response.status()
            )));
        }

        let emails: Vec<StalwartEmail> = response.json().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        Ok(emails)
    }

    /// Fetch a single email by ID
    pub async fn fetch_email(&self, account: &str, message_id: &str) -> Result<Option<StalwartEmail>> {
        let url = format!(
            "{}/api/v1/accounts/{}/messages/{}",
            self.base_url, account, message_id
        );

        let mut request = self.client.get(&url);
        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(AmailError::Network(format!(
                "Failed to fetch email: {}",
                response.status()
            )));
        }

        let email: StalwartEmail = response.json().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        Ok(Some(email))
    }

    /// Fetch unread emails
    pub async fn fetch_unread(&self, account: &str, limit: u32) -> Result<Vec<StalwartEmail>> {
        let url = format!(
            "{}/api/v1/accounts/{}/messages?filter=unseen&limit={}",
            self.base_url, account, limit
        );

        let mut request = self.client.get(&url);
        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AmailError::Network(format!(
                "Failed to fetch unread emails: {}",
                response.status()
            )));
        }

        let emails: Vec<StalwartEmail> = response.json().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        Ok(emails)
    }

    // -------------------------------------------------------------------------
    // 3. Email Sending
    // -------------------------------------------------------------------------

    /// Send an email
    pub async fn send_email(&self, email: &SendEmail) -> Result<String> {
        let url = format!("{}/api/v1/send", self.base_url);

        let mut request = self.client.post(&url).json(email);
        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AmailError::Network(format!(
                "Failed to send email: {}",
                error_text
            )));
        }

        let result: SendEmailResponse = response.json().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        Ok(result.message_id)
    }

    /// Queue email for sending (with retry)
    pub async fn queue_email(&self, email: &SendEmail) -> Result<String> {
        let url = format!("{}/api/v1/queue", self.base_url);

        let mut request = self.client.post(&url).json(email);
        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AmailError::Network(format!(
                "Failed to queue email: {}",
                error_text
            )));
        }

        let result: SendEmailResponse = response.json().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        Ok(result.message_id)
    }

    // -------------------------------------------------------------------------
    // 4. Mailbox Management
    // -------------------------------------------------------------------------

    /// List mailboxes for an account
    pub async fn list_mailboxes(&self, account: &str) -> Result<Vec<StalwartMailbox>> {
        let url = format!("{}/api/v1/accounts/{}/mailboxes", self.base_url, account);

        let mut request = self.client.get(&url);
        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AmailError::Network(format!(
                "Failed to list mailboxes: {}",
                response.status()
            )));
        }

        let mailboxes: Vec<StalwartMailbox> = response.json().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        Ok(mailboxes)
    }

    /// Create a mailbox
    pub async fn create_mailbox(&self, account: &str, name: &str) -> Result<()> {
        let url = format!("{}/api/v1/accounts/{}/mailboxes", self.base_url, account);

        let body = serde_json::json!({ "name": name });

        let mut request = self.client.post(&url).json(&body);
        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AmailError::Network(format!(
                "Failed to create mailbox: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Move email to folder
    pub async fn move_email(&self, account: &str, message_id: &str, folder: &str) -> Result<()> {
        let url = format!(
            "{}/api/v1/accounts/{}/messages/{}/move",
            self.base_url, account, message_id
        );

        let body = serde_json::json!({ "mailbox": folder });

        let mut request = self.client.post(&url).json(&body);
        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AmailError::Network(format!(
                "Failed to move email: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Mark email as read/unread
    pub async fn mark_read(&self, account: &str, message_id: &str, read: bool) -> Result<()> {
        let url = format!(
            "{}/api/v1/accounts/{}/messages/{}/flags",
            self.base_url, account, message_id
        );

        let body = if read {
            serde_json::json!({ "add": ["\\Seen"] })
        } else {
            serde_json::json!({ "remove": ["\\Seen"] })
        };

        let mut request = self.client.patch(&url).json(&body);
        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AmailError::Network(format!(
                "Failed to update flags: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Delete email
    pub async fn delete_email(&self, account: &str, message_id: &str) -> Result<()> {
        let url = format!(
            "{}/api/v1/accounts/{}/messages/{}",
            self.base_url, account, message_id
        );

        let mut request = self.client.delete(&url);
        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AmailError::Network(format!(
                "Failed to delete email: {}",
                response.status()
            )));
        }

        Ok(())
    }

    // -------------------------------------------------------------------------
    // 5. Domain Management
    // -------------------------------------------------------------------------

    /// List domains
    pub async fn list_domains(&self) -> Result<Vec<StalwartDomain>> {
        let url = format!("{}/api/v1/domains", self.base_url);

        let mut request = self.client.get(&url);
        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AmailError::Network(format!(
                "Failed to list domains: {}",
                response.status()
            )));
        }

        let domains: Vec<StalwartDomain> = response.json().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        Ok(domains)
    }

    /// Create domain
    pub async fn create_domain(&self, name: &str) -> Result<()> {
        let url = format!("{}/api/v1/domains", self.base_url);

        let body = serde_json::json!({ "name": name });

        let mut request = self.client.post(&url).json(&body);
        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(AmailError::Network(format!(
                "Failed to create domain: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Create account (mailbox)
    pub async fn create_account(&self, account: &CreateAccount) -> Result<()> {
        let url = format!("{}/api/v1/accounts", self.base_url);

        let mut request = self.client.post(&url).json(account);
        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AmailError::Network(format!(
                "Failed to create account: {}",
                error_text
            )));
        }

        Ok(())
    }

    /// Get account info
    pub async fn get_account(&self, account: &str) -> Result<Option<StalwartAccount>> {
        let url = format!("{}/api/v1/accounts/{}", self.base_url, account);

        let mut request = self.client.get(&url);
        if let Some(auth) = self.auth_header() {
            request = request.header("Authorization", auth);
        }

        let response = request.send().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(AmailError::Network(format!(
                "Failed to get account: {}",
                response.status()
            )));
        }

        let account: StalwartAccount = response.json().await
            .map_err(|e| AmailError::Network(e.to_string()))?;

        Ok(Some(account))
    }
}

// =============================================================================
// Data Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StalwartEmail {
    pub id: String,
    pub message_id: Option<String>,
    pub from: Vec<EmailAddress>,
    pub to: Vec<EmailAddress>,
    pub cc: Vec<EmailAddress>,
    pub subject: String,
    pub date: DateTime<Utc>,
    pub size: u64,
    pub flags: Vec<String>,
    pub preview: Option<String>,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub attachments: Vec<StalwartAttachment>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
    pub thread_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    pub name: Option<String>,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StalwartAttachment {
    pub id: String,
    pub name: String,
    pub content_type: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StalwartMailbox {
    pub id: String,
    pub name: String,
    pub total_messages: u64,
    pub unread_messages: u64,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StalwartDomain {
    pub name: String,
    pub accounts: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StalwartAccount {
    pub name: String,
    pub email: String,
    pub quota: u64,
    pub used: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmail {
    pub from: String,
    pub to: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
    pub subject: String,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailResponse {
    pub message_id: String,
    pub queued: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAccount {
    pub name: String,
    pub email: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota: Option<u64>,
}

impl StalwartEmail {
    /// Check if email is read
    pub fn is_read(&self) -> bool {
        self.flags.iter().any(|f| f == "\\Seen")
    }

    /// Check if email is starred/flagged
    pub fn is_starred(&self) -> bool {
        self.flags.iter().any(|f| f == "\\Flagged")
    }

    /// Get from address as string
    pub fn from_address(&self) -> String {
        self.from.first()
            .map(|a| a.email.clone())
            .unwrap_or_default()
    }

    /// Get from name
    pub fn from_name(&self) -> Option<String> {
        self.from.first().and_then(|a| a.name.clone())
    }
}

// =============================================================================
// 6. ConfigApplier - Apply JSON configs to Stalwart
// =============================================================================

use crate::config_manager::{ConfigManager, DomainConfig, MailboxConfig};
use std::sync::Arc;

/// Applies JSON configuration to Stalwart mail server
pub struct ConfigApplier {
    stalwart: StalwartClient,
    config: Arc<ConfigManager>,
}

impl ConfigApplier {
    /// Create a new ConfigApplier
    pub fn new(stalwart: StalwartClient, config: Arc<ConfigManager>) -> Self {
        Self { stalwart, config }
    }

    /// Sync all configuration to Stalwart
    pub async fn sync_all(&self) -> Result<SyncResult> {
        let mut result = SyncResult::default();

        // Sync domains
        let domain_result = self.sync_domains().await?;
        result.domains_created += domain_result.created;
        result.domains_updated += domain_result.updated;

        // Sync mailboxes
        let mailbox_result = self.sync_mailboxes().await?;
        result.mailboxes_created += mailbox_result.created;
        result.mailboxes_updated += mailbox_result.updated;

        Ok(result)
    }

    /// Sync domains from config to Stalwart
    pub async fn sync_domains(&self) -> Result<SyncStats> {
        let mut stats = SyncStats::default();
        let domains = self.config.get_domains().await;
        let existing = self.stalwart.list_domains().await.unwrap_or_default();
        let existing_names: Vec<&str> = existing.iter().map(|d| d.name.as_str()).collect();

        for domain in domains {
            if !domain.enabled {
                continue;
            }

            if !existing_names.contains(&domain.name.as_str()) {
                // Create new domain
                match self.stalwart.create_domain(&domain.name).await {
                    Ok(_) => {
                        tracing::info!("Created domain: {}", domain.name);
                        stats.created += 1;
                    }
                    Err(e) => {
                        tracing::error!("Failed to create domain {}: {}", domain.name, e);
                        stats.errors.push(format!("Domain {}: {}", domain.name, e));
                    }
                }
            } else {
                stats.updated += 1;
            }
        }

        Ok(stats)
    }

    /// Sync mailboxes from config to Stalwart
    pub async fn sync_mailboxes(&self) -> Result<SyncStats> {
        let mut stats = SyncStats::default();
        let mailboxes = self.config.get_all_mailboxes().await;

        for mailbox in mailboxes {
            if !mailbox.enabled {
                continue;
            }

            // Check if account exists
            let account_name = mailbox.email.split('@').next().unwrap_or(&mailbox.email);
            match self.stalwart.get_account(account_name).await {
                Ok(Some(_)) => {
                    // Account exists, could update here
                    stats.updated += 1;
                }
                Ok(None) => {
                    // Create new account
                    let create_account = CreateAccount {
                        name: mailbox.display_name.clone(),
                        email: mailbox.email.clone(),
                        password: self.generate_temp_password(),
                        quota: Some(mailbox.quota_mb * 1024 * 1024), // Convert MB to bytes
                    };

                    match self.stalwart.create_account(&create_account).await {
                        Ok(_) => {
                            tracing::info!("Created mailbox: {}", mailbox.email);
                            stats.created += 1;
                        }
                        Err(e) => {
                            tracing::error!("Failed to create mailbox {}: {}", mailbox.email, e);
                            stats.errors.push(format!("Mailbox {}: {}", mailbox.email, e));
                        }
                    }
                }
                Err(e) => {
                    stats.errors.push(format!("Check mailbox {}: {}", mailbox.email, e));
                }
            }
        }

        Ok(stats)
    }

    /// Apply a single domain config
    pub async fn apply_domain(&self, domain: &DomainConfig) -> Result<()> {
        if !domain.enabled {
            return Ok(());
        }

        // Check if domain exists
        let existing = self.stalwart.list_domains().await?;
        let exists = existing.iter().any(|d| d.name == domain.name);

        if !exists {
            self.stalwart.create_domain(&domain.name).await?;
            tracing::info!("Applied domain: {}", domain.name);
        }

        Ok(())
    }

    /// Apply a single mailbox config
    pub async fn apply_mailbox(&self, mailbox: &MailboxConfig) -> Result<()> {
        if !mailbox.enabled {
            return Ok(());
        }

        let account_name = mailbox.email.split('@').next().unwrap_or(&mailbox.email);
        
        if self.stalwart.get_account(account_name).await?.is_none() {
            let create_account = CreateAccount {
                name: mailbox.display_name.clone(),
                email: mailbox.email.clone(),
                password: self.generate_temp_password(),
                quota: Some(mailbox.quota_mb * 1024 * 1024),
            };
            self.stalwart.create_account(&create_account).await?;
            tracing::info!("Applied mailbox: {}", mailbox.email);
        }

        Ok(())
    }

    /// Generate a temporary password for new accounts
    fn generate_temp_password(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("temp_{:x}", timestamp)
    }
}

/// Result of a sync operation
#[derive(Debug, Default)]
pub struct SyncResult {
    pub domains_created: u32,
    pub domains_updated: u32,
    pub mailboxes_created: u32,
    pub mailboxes_updated: u32,
    pub errors: Vec<String>,
}

/// Stats for a single sync type
#[derive(Debug, Default)]
pub struct SyncStats {
    pub created: u32,
    pub updated: u32,
    pub deleted: u32,
    pub errors: Vec<String>,
}
