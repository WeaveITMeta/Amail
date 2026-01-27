// =============================================================================
// Stalwart Mail Server Client
// =============================================================================
// Table of Contents:
// 1. StalwartClient - REST API client for message fetching
// 2. SmtpMailer - SMTP client for sending emails
// 3. Message fetching
// 4. Email sending via SMTP
// 5. Email composition utilities
// =============================================================================

use anyhow::{anyhow, Result};
use amail_core::email::{Attachment, EmailAddress, EmailMessage, EmailMetadata, EmailPriority};
use base64::Engine;
use chrono::Utc;
use lettre::{
    message::{header::ContentType, Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Stalwart Mail Server REST API client
pub struct StalwartClient {
    /// HTTP client
    client: Client,
    /// Base URL for Stalwart API
    base_url: String,
    /// API key for authentication
    api_key: String,
    /// SMTP mailer for sending emails
    smtp_mailer: Option<Arc<SmtpMailer>>,
}

/// SMTP configuration for sending emails
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    /// SMTP server host
    pub host: String,
    /// SMTP server port (typically 587 for STARTTLS, 465 for SSL)
    pub port: u16,
    /// Username for SMTP authentication
    pub username: String,
    /// Password for SMTP authentication
    pub password: String,
    /// Use STARTTLS (port 587) vs implicit TLS (port 465)
    pub starttls: bool,
    /// Default sender address
    pub default_from: String,
    /// Default sender name
    pub default_from_name: Option<String>,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 587,
            username: String::new(),
            password: String::new(),
            starttls: true,
            default_from: "agent@localhost".to_string(),
            default_from_name: Some("Amail Agent".to_string()),
        }
    }
}

/// SMTP mailer for sending emails via Stalwart
pub struct SmtpMailer {
    /// SMTP transport
    transport: AsyncSmtpTransport<Tokio1Executor>,
    /// Configuration
    config: SmtpConfig,
    /// Send statistics
    stats: RwLock<SmtpStats>,
}

/// SMTP sending statistics
#[derive(Debug, Default, Clone)]
pub struct SmtpStats {
    pub emails_sent: u64,
    pub emails_failed: u64,
    pub last_send_at: Option<chrono::DateTime<Utc>>,
    pub last_error: Option<String>,
}

impl SmtpMailer {
    /// Create a new SMTP mailer
    pub fn new(config: SmtpConfig) -> Result<Self> {
        let creds = Credentials::new(config.username.clone(), config.password.clone());

        let transport = if config.starttls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.host)?
                .port(config.port)
                .credentials(creds)
                .build()
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)?
                .port(config.port)
                .credentials(creds)
                .build()
        };

        Ok(Self {
            transport,
            config,
            stats: RwLock::new(SmtpStats::default()),
        })
    }

    /// Send an email
    pub async fn send(&self, email: &OutboundEmail) -> Result<String> {
        let message = self.build_message(email)?;
        
        match self.transport.send(message).await {
            Ok(response) => {
                let mut stats = self.stats.write().await;
                stats.emails_sent += 1;
                stats.last_send_at = Some(Utc::now());
                
                let message_id = response.message().next().map(|m| m.to_string()).unwrap_or_else(|| Uuid::new_v4().to_string());
                info!("Email sent successfully: {}", message_id);
                Ok(message_id)
            }
            Err(e) => {
                let mut stats = self.stats.write().await;
                stats.emails_failed += 1;
                stats.last_error = Some(e.to_string());
                
                error!("Failed to send email: {}", e);
                Err(anyhow!("SMTP send failed: {}", e))
            }
        }
    }

    /// Build a lettre Message from OutboundEmail
    fn build_message(&self, email: &OutboundEmail) -> Result<Message> {
        // Parse sender
        let from_addr = email.from.as_ref().unwrap_or(&self.config.default_from);
        let from_name = email.from_name.as_ref().or(self.config.default_from_name.as_ref());
        let from_mailbox: Mailbox = if let Some(name) = from_name {
            format!("{} <{}>", name, from_addr).parse()
        } else {
            from_addr.parse()
        }.map_err(|e| anyhow!("Invalid from address: {}", e))?;

        // Build message
        let mut builder = Message::builder()
            .from(from_mailbox)
            .subject(&email.subject);

        // Add recipients
        for to in &email.to {
            let to_mailbox: Mailbox = to.parse()
                .map_err(|e| anyhow!("Invalid to address '{}': {}", to, e))?;
            builder = builder.to(to_mailbox);
        }

        // Add CC recipients
        for cc in &email.cc {
            let cc_mailbox: Mailbox = cc.parse()
                .map_err(|e| anyhow!("Invalid cc address '{}': {}", cc, e))?;
            builder = builder.cc(cc_mailbox);
        }

        // Add BCC recipients
        for bcc in &email.bcc {
            let bcc_mailbox: Mailbox = bcc.parse()
                .map_err(|e| anyhow!("Invalid bcc address '{}': {}", bcc, e))?;
            builder = builder.bcc(bcc_mailbox);
        }

        // Add Reply-To if specified
        if let Some(reply_to) = &email.reply_to {
            let reply_mailbox: Mailbox = reply_to.parse()
                .map_err(|e| anyhow!("Invalid reply-to address: {}", e))?;
            builder = builder.reply_to(reply_mailbox);
        }

        // Add In-Reply-To header for threading
        if let Some(in_reply_to) = &email.in_reply_to {
            builder = builder.in_reply_to(in_reply_to.clone());
        }

        // Add References header for threading
        if !email.references.is_empty() {
            builder = builder.references(email.references.join(" "));
        }

        // Build body (multipart if both text and HTML)
        let message = match (&email.text_body, &email.html_body) {
            (Some(text), Some(html)) => {
                builder.multipart(
                    MultiPart::alternative()
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_PLAIN)
                                .body(text.clone())
                        )
                        .singlepart(
                            SinglePart::builder()
                                .header(ContentType::TEXT_HTML)
                                .body(html.clone())
                        )
                )?
            }
            (Some(text), None) => {
                builder.body(text.clone())?
            }
            (None, Some(html)) => {
                builder
                    .header(ContentType::TEXT_HTML)
                    .body(html.clone())?
            }
            (None, None) => {
                builder.body(String::new())?
            }
        };

        Ok(message)
    }

    /// Get sending statistics
    pub async fn stats(&self) -> SmtpStats {
        self.stats.read().await.clone()
    }

    /// Test SMTP connection
    pub async fn test_connection(&self) -> Result<bool> {
        match self.transport.test_connection().await {
            Ok(true) => {
                info!("SMTP connection test successful");
                Ok(true)
            }
            Ok(false) => {
                error!("SMTP connection test failed");
                Ok(false)
            }
            Err(e) => {
                error!("SMTP connection test error: {}", e);
                Err(anyhow!("SMTP connection test failed: {}", e))
            }
        }
    }
}

/// Default admin password for Stalwart Docker container
/// Read from STALWART_ADMIN_PASS env var, fallback to "admin"
fn get_stalwart_secret() -> String {
    std::env::var("STALWART_ADMIN_PASS").unwrap_or_else(|_| "admin".to_string())
}

impl StalwartClient {
    /// Create a new Stalwart client (REST API only)
    /// Uses default internal secret if api_key is empty
    pub fn new(base_url: &str, api_key: &str) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        // Use default secret if none provided
        let effective_key = if api_key.is_empty() {
            get_stalwart_secret()
        } else {
            api_key.to_string()
        };

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: effective_key,
            smtp_mailer: None,
        })
    }

    /// Create with SMTP mailer for sending emails
    /// Uses default internal secret if api_key is empty
    pub fn with_smtp(base_url: &str, api_key: &str, smtp_config: SmtpConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let smtp_mailer = SmtpMailer::new(smtp_config)?;

        // Use default secret if none provided
        let effective_key = if api_key.is_empty() {
            get_stalwart_secret()
        } else {
            api_key.to_string()
        };

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: effective_key,
            smtp_mailer: Some(Arc::new(smtp_mailer)),
        })
    }

    /// Get SMTP mailer reference
    pub fn smtp(&self) -> Option<&Arc<SmtpMailer>> {
        self.smtp_mailer.as_ref()
    }

    /// Initialize Stalwart server with domain configuration
    /// This should be called during Amail startup to ensure Stalwart is properly configured
    /// Creates the domain and a default support mailbox
    pub async fn initialize(&self, _hostname: &str, domain: &str) -> Result<()> {
        info!("Initializing Stalwart server for domain: {}", domain);
        
        let auth = base64::engine::general_purpose::STANDARD.encode(
            format!("admin:{}", self.api_key)
        );

        // Step 1: Create domain
        if let Err(e) = self.create_domain(domain).await {
            warn!("Domain creation note: {}", e);
        }

        // Step 2: Create default support mailbox with full permissions
        let support_email = format!("support@{}", domain);
        let account_data = serde_json::json!({
            "type": "individual",
            "name": "support",
            "emails": [support_email],
            "description": "Support Mailbox",
            "secrets": ["support123"],
            "enabledPermissions": [
                "authenticate",
                "jmap-email-get",
                "jmap-email-set", 
                "jmap-email-query",
                "jmap-mailbox-get",
                "jmap-mailbox-set",
                "jmap-mailbox-query",
                "jmap-blob-upload",
                "jmap-blob-download"
            ]
        });

        let url = format!("{}/api/principal", self.base_url);
        let resp = self.client
            .post(&url)
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/json")
            .json(&account_data)
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                info!("Created support mailbox: {}", support_email);
            }
            Ok(r) => {
                let body = r.text().await.unwrap_or_default();
                if body.contains("alreadyExists") {
                    debug!("Support mailbox already exists");
                } else {
                    debug!("Support mailbox creation response: {}", body);
                }
            }
            Err(e) => {
                warn!("Failed to create support mailbox: {}", e);
            }
        }

        // Step 3: Reload settings to apply changes
        let reload_url = format!("{}/api/reload", self.base_url);
        let _ = self.client
            .get(&reload_url)
            .header("Authorization", format!("Basic {}", auth))
            .send()
            .await;

        info!("Stalwart initialization complete for {}", domain);
        Ok(())
    }

    /// Get the internal account ID for a username from Stalwart's principal API
    async fn get_account_id(&self, username: &str) -> Result<String> {
        let url = format!("{}/api/principal/{}", self.base_url, username);
        
        let auth = base64::engine::general_purpose::STANDARD.encode(
            format!("admin:{}", self.api_key)
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Basic {}", auth))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Account '{}' not found", username));
        }

        let result: serde_json::Value = response.json().await?;
        
        // The internal ID is in the "data.id" field
        let id = result
            .get("data")
            .and_then(|d| d.get("id"))
            .and_then(|id| id.as_u64())
            .ok_or_else(|| anyhow!("No ID found for account '{}'", username))?;

        Ok(id.to_string())
    }

    /// List all email accounts from Stalwart
    /// Returns a list of email addresses for all individual accounts
    pub async fn list_accounts(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/principal?type=individual", self.base_url);
        
        let auth = base64::engine::general_purpose::STANDARD.encode(
            format!("admin:{}", self.api_key)
        );

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Basic {}", auth))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to list accounts"));
        }

        let result: serde_json::Value = response.json().await?;
        
        // Extract email addresses from accounts
        let accounts = result
            .get("data")
            .and_then(|d| d.get("items"))
            .and_then(|items| items.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        item.get("emails")
                            .and_then(|e| e.as_array())
                            .and_then(|emails| emails.first())
                            .and_then(|email| email.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(accounts)
    }

    /// Create a domain in Stalwart
    /// This must be done before creating accounts for that domain
    pub async fn create_domain(&self, domain: &str) -> Result<()> {
        let url = format!("{}/api/principal", self.base_url);
        
        let domain_data = serde_json::json!({
            "type": "domain",
            "name": domain.to_lowercase()
        });

        debug!("Creating Stalwart domain: {}", domain);

        let auth = base64::engine::general_purpose::STANDARD.encode(
            format!("admin:{}", self.api_key)
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/json")
            .json(&domain_data)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                info!("Created Stalwart domain: {}", domain);
                Ok(())
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                // 409 Conflict means domain already exists - that's OK
                if status.as_u16() == 409 || body.contains("alreadyExists") {
                    debug!("Stalwart domain already exists: {}", domain);
                    Ok(())
                } else {
                    warn!("Failed to create Stalwart domain {}: {} - {}", domain, status, body);
                    Err(anyhow!("Failed to create domain: {} - {}", status, body))
                }
            }
            Err(e) => {
                warn!("Stalwart API error creating domain: {}", e);
                Err(anyhow!("Stalwart API error: {}", e))
            }
        }
    }

    /// Create a mailbox account in Stalwart
    /// This creates the actual email account that can receive mail
    /// Automatically creates the domain if it doesn't exist
    pub async fn create_account(&self, email: &str, name: &str, password: &str) -> Result<()> {
        // First ensure the domain exists
        if let Some(domain) = email.split('@').nth(1) {
            let _ = self.create_domain(domain).await;
        }

        let url = format!("{}/api/principal", self.base_url);
        
        // Extract username from email (part before @)
        let username = email.split('@').next().unwrap_or(email);
        
        let account_data = serde_json::json!({
            "type": "individual",
            "name": username,
            "emails": [email],
            "secrets": [password],
            "description": name
        });

        debug!("Creating Stalwart account: {} ({})", email, name);

        // Stalwart uses Basic auth for admin API
        let auth = base64::engine::general_purpose::STANDARD.encode(
            format!("admin:{}", self.api_key)
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/json")
            .json(&account_data)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                info!("Created Stalwart account: {}", email);
                Ok(())
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                // 409 Conflict means account already exists - that's OK
                if status.as_u16() == 409 {
                    debug!("Stalwart account already exists: {}", email);
                    Ok(())
                } else {
                    warn!("Failed to create Stalwart account {}: {} - {}", email, status, body);
                    Err(anyhow!("Failed to create account: {} - {}", status, body))
                }
            }
            Err(e) => {
                warn!("Stalwart API error creating account: {}", e);
                Err(anyhow!("Stalwart API error: {}", e))
            }
        }
    }

    /// Delete a mailbox account from Stalwart
    pub async fn delete_account(&self, email: &str) -> Result<()> {
        let username = email.split('@').next().unwrap_or(email);
        let url = format!("{}/api/principal/{}", self.base_url, username);

        let auth = base64::engine::general_purpose::STANDARD.encode(
            format!("admin:{}", self.api_key)
        );

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Basic {}", auth))
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                info!("Deleted Stalwart account: {}", email);
                Ok(())
            }
            Ok(resp) if resp.status().as_u16() == 404 => {
                debug!("Stalwart account not found: {}", email);
                Ok(())
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                warn!("Failed to delete Stalwart account {}: {} - {}", email, status, body);
                Err(anyhow!("Failed to delete account: {} - {}", status, body))
            }
            Err(e) => {
                warn!("Stalwart API error deleting account: {}", e);
                Err(anyhow!("Stalwart API error: {}", e))
            }
        }
    }

    /// Fetch a message by ID
    pub async fn fetch_message(&self, message_id: &str) -> Result<EmailMessage> {
        let url = format!("{}/api/v1/messages/{}", self.base_url, message_id);

        debug!("Fetching message from {}", url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(|e| anyhow!("Failed to fetch message: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Stalwart API error {}: {}", status, body);
            return Err(anyhow!("Stalwart API error {}: {}", status, body));
        }

        let raw_bytes = response.bytes().await?;
        
        // Parse email using mail-parser
        self.parse_email(&raw_bytes, message_id)
    }

    /// Parse raw email bytes into EmailMessage
    fn parse_email(&self, raw: &[u8], message_id: &str) -> Result<EmailMessage> {
        use mail_parser::{Message, MimeHeaders, MessageParser};

        let parsed = MessageParser::default()
            .parse(raw)
            .ok_or_else(|| anyhow!("Failed to parse email"))?;

        // Extract sender
        let from = parsed
            .from()
            .and_then(|addrs| addrs.first())
            .map(|addr| EmailAddress {
                name: addr.name().map(|s| s.to_string()),
                address: addr.address().unwrap_or("unknown@unknown.com").to_string(),
            })
            .unwrap_or_else(|| EmailAddress::new("unknown@unknown.com"));

        // Extract recipients
        let to: Vec<EmailAddress> = parsed
            .to()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| EmailAddress {
                        name: addr.name().map(|s| s.to_string()),
                        address: addr.address().unwrap_or("").to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let cc: Vec<EmailAddress> = parsed
            .cc()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| EmailAddress {
                        name: addr.name().map(|s| s.to_string()),
                        address: addr.address().unwrap_or("").to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Extract subject
        let subject = parsed.subject().unwrap_or("(No Subject)").to_string();

        // Extract body
        let text_body = parsed.body_text(0).map(|s| s.to_string());
        let html_body = parsed.body_html(0).map(|s| s.to_string());

        // Extract attachments
        let attachments: Vec<Attachment> = parsed
            .attachments()
            .map(|att| Attachment {
                id: Uuid::new_v4(),
                filename: att.attachment_name().unwrap_or("unnamed").to_string(),
                content_type: att.content_type().map(|ct| ct.ctype().to_string()).unwrap_or_else(|| "application/octet-stream".to_string()),
                size: att.contents().len(),
                content: att.contents().to_vec(),
                content_id: att.content_id().map(|s| s.to_string()),
                is_inline: false, // mail-parser doesn't expose is_inline directly
            })
            .collect();

        // Build metadata
        let metadata = EmailMetadata {
            message_id: parsed.message_id().unwrap_or(message_id).to_string(),
            tracking_id: Uuid::new_v4(),
            from,
            to,
            cc,
            bcc: Vec::new(),
            reply_to: parsed.reply_to().and_then(|addrs| addrs.first()).map(|addr| {
                EmailAddress {
                    name: addr.name().map(|s| s.to_string()),
                    address: addr.address().unwrap_or("").to_string(),
                }
            }),
            subject,
            received_at: Utc::now(),
            sent_at: parsed.date().map(|d| d.to_rfc3339().parse().ok()).flatten(),
            in_reply_to: parsed.in_reply_to().as_text().map(|s| s.to_string()),
            references: parsed
                .references()
                .as_text_list()
                .map(|refs| refs.iter().map(|s| s.to_string()).collect())
                .unwrap_or_default(),
            spf_result: None,
            dkim_result: None,
            dmarc_result: None,
            priority: EmailPriority::Normal,
            custom_headers: std::collections::HashMap::new(),
        };

        Ok(EmailMessage {
            metadata,
            text_body,
            html_body,
            attachments,
            raw: Some(raw.to_vec()),
        })
    }

    /// Send an email via Stalwart SMTP submission
    pub async fn send_email(&self, email: &EmailMessage) -> Result<String> {
        let url = format!("{}/api/v1/send", self.base_url);

        let request = SendEmailRequest {
            from: email.metadata.from.address.clone(),
            to: email.metadata.to.iter().map(|a| a.address.clone()).collect(),
            cc: email.metadata.cc.iter().map(|a| a.address.clone()).collect(),
            subject: email.metadata.subject.clone(),
            text_body: email.text_body.clone(),
            html_body: email.html_body.clone(),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send email: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to send email {}: {}", status, body));
        }

        let result: SendEmailResponse = response.json().await?;
        Ok(result.message_id)
    }

    /// Check if Stalwart is healthy
    pub async fn health_check(&self) -> bool {
        let url = format!("{}/api/v1/health", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// List messages from a mailbox using JMAP
    /// Returns a list of email summaries for the inbox view
    pub async fn list_messages(
        &self,
        account_id: &str,
        _folder: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<EmailSummary>> {
        let url = format!("{}/jmap", self.base_url);
        let auth = base64::engine::general_purpose::STANDARD.encode(
            format!("admin:{}", self.api_key)
        );
        
        // Get the numeric account ID from Stalwart
        let numeric_account_id = match self.get_account_id(account_id).await {
            Ok(id) => id,
            Err(e) => {
                warn!("Failed to get account ID for '{}': {}", account_id, e);
                return Ok(vec![]);
            }
        };
        
        debug!("Resolved account '{}' to numeric ID '{}'", account_id, numeric_account_id);
        
        // Query all emails for this account (no mailbox filter - get everything)
        let jmap_request = serde_json::json!({
            "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
            "methodCalls": [
                ["Email/query", {
                    "accountId": numeric_account_id,
                    "filter": {},
                    "sort": [{"property": "receivedAt", "isAscending": false}],
                    "position": offset,
                    "limit": limit
                }, "0"],
                ["Email/get", {
                    "accountId": numeric_account_id,
                    "#ids": {
                        "resultOf": "0",
                        "name": "Email/query",
                        "path": "/ids"
                    },
                    "properties": ["id", "from", "to", "subject", "receivedAt", "preview", "keywords", "hasAttachment", "mailboxIds"]
                }, "1"]
            ]
        });

        debug!("JMAP request to {}: {:?}", url, jmap_request);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/json")
            .json(&jmap_request)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                self.parse_jmap_email_list(&body)
            }
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                warn!("JMAP request failed {}: {}", status, body);
                // Return empty list on error - Stalwart may not be configured
                Ok(vec![])
            }
            Err(e) => {
                warn!("JMAP request error: {}", e);
                Ok(vec![])
            }
        }
    }

    /// Parse JMAP Email/get response into EmailSummary list
    fn parse_jmap_email_list(&self, response: &serde_json::Value) -> Result<Vec<EmailSummary>> {
        let mut emails = Vec::new();
        
        if let Some(method_responses) = response.get("methodResponses").and_then(|v| v.as_array()) {
            for method_response in method_responses {
                if let Some(arr) = method_response.as_array() {
                    if arr.len() >= 2 && arr[0].as_str() == Some("Email/get") {
                        if let Some(list) = arr[1].get("list").and_then(|v| v.as_array()) {
                            for email in list {
                                if let Some(summary) = self.parse_email_summary(email) {
                                    emails.push(summary);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(emails)
    }

    /// Parse a single email from JMAP response
    fn parse_email_summary(&self, email: &serde_json::Value) -> Option<EmailSummary> {
        let id = email.get("id")?.as_str()?.to_string();
        
        let from = email.get("from")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .map(|f| EmailAddress {
                name: f.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
                address: f.get("email").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
            })
            .unwrap_or_else(|| EmailAddress {
                name: None,
                address: "unknown@unknown.com".to_string(),
            });

        let to: Vec<String> = email.get("to")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|t| t.get("email").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .collect())
            .unwrap_or_default();

        let subject = email.get("subject")
            .and_then(|v| v.as_str())
            .unwrap_or("(No Subject)")
            .to_string();

        let preview = email.get("preview")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let received_at = email.get("receivedAt")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let keywords = email.get("keywords")
            .and_then(|v| v.as_object())
            .map(|obj| obj.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();

        let is_read = keywords.contains(&"$seen".to_string());
        let is_starred = keywords.contains(&"$flagged".to_string());

        let has_attachment = email.get("hasAttachment")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        Some(EmailSummary {
            id,
            from,
            to,
            subject,
            preview,
            received_at,
            is_read,
            is_starred,
            has_attachment,
            labels: keywords,
        })
    }

    /// Inject a test email into a mailbox (for development/testing)
    /// Uses JMAP Email/set to create an email directly (no blob storage needed)
    pub async fn inject_test_email(&self, mailbox_email: &str) -> Result<()> {
        let url = format!("{}/jmap", self.base_url);
        let username = mailbox_email.split('@').next().unwrap_or("support");
        
        // Get the internal account ID from Stalwart's principal API
        let account_id = self.get_account_id(username).await?;
        info!("Resolved account '{}' to internal ID: {}", username, account_id);
        
        let now = chrono::Utc::now();
        let message_id = format!("<test-{}@amail.local>", uuid::Uuid::new_v4());

        let auth = base64::engine::general_purpose::STANDARD.encode(
            format!("admin:{}", self.api_key)
        );

        // First get the INBOX mailbox ID
        let mailbox_query = serde_json::json!({
            "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
            "methodCalls": [
                ["Mailbox/query", {
                    "accountId": account_id,
                    "filter": { "name": "INBOX" }
                }, "0"],
                ["Mailbox/get", {
                    "accountId": account_id,
                    "#ids": {
                        "resultOf": "0",
                        "name": "Mailbox/query",
                        "path": "/ids"
                    }
                }, "1"]
            ]
        });

        let mailbox_response = self.client
            .post(&url)
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/json")
            .json(&mailbox_query)
            .send()
            .await?;

        let mailbox_result: serde_json::Value = mailbox_response.json().await?;
        
        // Extract INBOX mailbox ID
        let inbox_id = mailbox_result
            .get("methodResponses")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.iter().find(|r| {
                r.as_array().map(|a| a.get(0).and_then(|v| v.as_str()) == Some("Mailbox/get")).unwrap_or(false)
            }))
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.get(1))
            .and_then(|v| v.get("list"))
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
            .unwrap_or("INBOX");

        info!("Found INBOX mailbox ID: {}", inbox_id);

        // Create email using Email/set
        let email_request = serde_json::json!({
            "using": ["urn:ietf:params:jmap:core", "urn:ietf:params:jmap:mail"],
            "methodCalls": [
                ["Email/set", {
                    "accountId": account_id,
                    "create": {
                        "test1": {
                            "mailboxIds": { inbox_id: true },
                            "from": [{ "name": "Test Sender", "email": "test@example.com" }],
                            "to": [{ "email": mailbox_email }],
                            "subject": format!("Test Email - {}", now.format("%Y-%m-%d %H:%M:%S")),
                            "receivedAt": now.to_rfc3339(),
                            "messageId": [message_id],
                            "bodyStructure": {
                                "type": "text/plain",
                                "partId": "1"
                            },
                            "bodyValues": {
                                "1": {
                                    "value": format!(
                                        "This is a test email injected for development purposes.\n\nTimestamp: {}\nMailbox: {}",
                                        now.to_rfc3339(),
                                        mailbox_email
                                    )
                                }
                            }
                        }
                    }
                }, "0"]
            ]
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/json")
            .json(&email_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("Failed to create email: {} - {}", status, body));
        }

        let result: serde_json::Value = response.json().await?;
        debug!("Email/set result: {:?}", result);

        // Check for errors in response
        if let Some(responses) = result.get("methodResponses").and_then(|v| v.as_array()) {
            for response in responses {
                if let Some(arr) = response.as_array() {
                    if arr.get(0).and_then(|v| v.as_str()) == Some("error") {
                        let error_desc = arr.get(1)
                            .and_then(|v| v.get("description"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown error");
                        return Err(anyhow!("JMAP error: {}", error_desc));
                    }
                    // Check for Email/set errors
                    if arr.get(0).and_then(|v| v.as_str()) == Some("Email/set") {
                        if let Some(not_created) = arr.get(1).and_then(|v| v.get("notCreated")) {
                            if let Some(obj) = not_created.as_object() {
                                if !obj.is_empty() {
                                    let error_msg = serde_json::to_string(not_created).unwrap_or_default();
                                    return Err(anyhow!("Failed to create email: {}", error_msg));
                                }
                            }
                        }
                    }
                }
            }
        }

        info!("Test email injected into {}", mailbox_email);
        Ok(())
    }
}

/// Email summary for inbox list view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailSummary {
    pub id: String,
    pub from: EmailAddress,
    pub to: Vec<String>,
    pub subject: String,
    pub preview: Option<String>,
    pub received_at: String,
    pub is_read: bool,
    pub is_starred: bool,
    pub has_attachment: bool,
    pub labels: Vec<String>,
}

/// Outbound email for SMTP sending
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundEmail {
    /// Sender address (uses default if None)
    pub from: Option<String>,
    /// Sender display name
    pub from_name: Option<String>,
    /// Recipient addresses
    pub to: Vec<String>,
    /// CC addresses
    pub cc: Vec<String>,
    /// BCC addresses
    pub bcc: Vec<String>,
    /// Reply-To address
    pub reply_to: Option<String>,
    /// Email subject
    pub subject: String,
    /// Plain text body
    pub text_body: Option<String>,
    /// HTML body
    pub html_body: Option<String>,
    /// In-Reply-To header for threading
    pub in_reply_to: Option<String>,
    /// References header for threading
    pub references: Vec<String>,
}

impl OutboundEmail {
    /// Create a new outbound email
    pub fn new(to: Vec<String>, subject: impl Into<String>) -> Self {
        Self {
            from: None,
            from_name: None,
            to,
            cc: Vec::new(),
            bcc: Vec::new(),
            reply_to: None,
            subject: subject.into(),
            text_body: None,
            html_body: None,
            in_reply_to: None,
            references: Vec::new(),
        }
    }

    /// Create a reply to an existing email
    pub fn reply_to_email(original: &EmailMessage, body: String) -> Self {
        let mut references = original.metadata.references.clone();
        references.push(original.metadata.message_id.clone());

        Self {
            from: None,
            from_name: None,
            to: vec![original.metadata.from.address.clone()],
            cc: Vec::new(),
            bcc: Vec::new(),
            reply_to: None,
            subject: format!("Re: {}", original.metadata.subject.trim_start_matches("Re: ")),
            text_body: Some(body),
            html_body: None,
            in_reply_to: Some(original.metadata.message_id.clone()),
            references,
        }
    }

    /// Set sender
    pub fn with_from(mut self, from: impl Into<String>, name: Option<String>) -> Self {
        self.from = Some(from.into());
        self.from_name = name;
        self
    }

    /// Set text body
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text_body = Some(text.into());
        self
    }

    /// Set HTML body
    pub fn with_html(mut self, html: impl Into<String>) -> Self {
        self.html_body = Some(html.into());
        self
    }

    /// Add CC recipient
    pub fn with_cc(mut self, cc: impl Into<String>) -> Self {
        self.cc.push(cc.into());
        self
    }

    /// Add BCC recipient
    pub fn with_bcc(mut self, bcc: impl Into<String>) -> Self {
        self.bcc.push(bcc.into());
        self
    }
}

impl StalwartClient {
    /// Send email via SMTP (preferred method)
    pub async fn send_smtp(&self, email: &OutboundEmail) -> Result<String> {
        if let Some(smtp) = &self.smtp_mailer {
            smtp.send(email).await
        } else {
            Err(anyhow!("SMTP mailer not configured. Use StalwartClient::with_smtp() to enable SMTP sending."))
        }
    }

    /// Send a reply to an email
    pub async fn send_reply(&self, original: &EmailMessage, body: String) -> Result<String> {
        let reply = OutboundEmail::reply_to_email(original, body);
        self.send_smtp(&reply).await
    }

    /// Test SMTP connection
    pub async fn test_smtp(&self) -> Result<bool> {
        if let Some(smtp) = &self.smtp_mailer {
            smtp.test_connection().await
        } else {
            Err(anyhow!("SMTP mailer not configured"))
        }
    }

    /// Get SMTP statistics
    pub async fn smtp_stats(&self) -> Option<SmtpStats> {
        if let Some(smtp) = &self.smtp_mailer {
            Some(smtp.stats().await)
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize)]
struct SendEmailRequest {
    from: String,
    to: Vec<String>,
    cc: Vec<String>,
    subject: String,
    text_body: Option<String>,
    html_body: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SendEmailResponse {
    message_id: String,
}
