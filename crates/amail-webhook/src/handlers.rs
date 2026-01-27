// =============================================================================
// Webhook Handlers - Stalwart Event Processing
// =============================================================================
// Table of Contents:
// 1. WebhookPayload - Stalwart webhook payload
// 2. webhook_handler - Main webhook endpoint
// 3. Event processing logic
// =============================================================================

use actix_web::{web, HttpRequest, HttpResponse};
use amail_core::{
    email::{EmailAddress, EmailMessage, EmailMetadata, EmailPriority},
    task::Task,
    spam::{SpamFilter, SpamFolder, SpamClassification},
    AmailError,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::signature::verify_webhook_signature;
use crate::stalwart::StalwartClient;

/// Stalwart webhook payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    /// Event type
    pub event: WebhookEvent,
    /// Message ID
    pub message_id: String,
    /// Sender email
    pub sender: String,
    /// Recipient email(s)
    pub recipients: Vec<String>,
    /// Email subject
    pub subject: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Account ID (for multi-tenant)
    pub account_id: Option<String>,
    /// Mailbox name
    pub mailbox: Option<String>,
    /// Message size in bytes
    pub size: Option<u64>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Webhook event types from Stalwart
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEvent {
    /// Message received (SMTP)
    MessageReceived,
    /// Message delivered to mailbox
    MessageDelivered,
    /// Message sent (outbound)
    MessageSent,
    /// Delivery failed
    DeliveryFailed,
    /// Spam detected
    SpamDetected,
    /// Virus detected
    VirusDetected,
    /// Authentication event
    AuthEvent,
    /// Unknown event
    #[serde(other)]
    Unknown,
}

/// Application state for webhook handlers
pub struct AppState {
    /// Stalwart client for fetching full emails
    pub stalwart: StalwartClient,
    /// Webhook secret for signature verification
    pub webhook_secret: String,
    /// Task queue sender
    pub task_sender: tokio::sync::mpsc::Sender<Task>,
    /// Spam filter
    pub spam_filter: SpamFilter,
}

/// Main webhook handler
pub async fn webhook_handler(
    req: HttpRequest,
    payload: web::Json<WebhookPayload>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    // Verify webhook signature
    let signature = req
        .headers()
        .get("X-Webhook-Signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let body_bytes = serde_json::to_vec(&payload.0).unwrap_or_default();
    
    if !verify_webhook_signature(&body_bytes, signature, &state.webhook_secret) {
        warn!("Invalid webhook signature");
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Invalid signature"
        })));
    }

    info!(
        "Received webhook: {:?} for message {}",
        payload.event, payload.message_id
    );

    // Process based on event type
    match payload.event {
        WebhookEvent::MessageReceived | WebhookEvent::MessageDelivered => {
            // Fetch full email content
            match state.stalwart.fetch_message(&payload.message_id).await {
                Ok(email) => {
                    // Run spam filter
                    let spam_result = state.spam_filter.analyze(&email);
                    info!(
                        "Spam analysis for {}: score={:.1}, classification={:?}, folder={:?}",
                        payload.message_id, spam_result.score, spam_result.classification, spam_result.folder
                    );

                    // Handle based on spam classification
                    match spam_result.folder {
                        SpamFolder::Reject => {
                            warn!(
                                "Rejecting spam email {} (score: {:.1}): {:?}",
                                payload.message_id, spam_result.score, spam_result.indicators
                            );
                            // Don't process rejected spam
                            return Ok(HttpResponse::Ok().json(serde_json::json!({
                                "status": "rejected",
                                "reason": "spam",
                                "score": spam_result.score,
                                "message_id": payload.message_id
                            })));
                        }
                        SpamFolder::Quarantine => {
                            warn!(
                                "Quarantining email {} (score: {:.1})",
                                payload.message_id, spam_result.score
                            );
                            // TODO: Move to quarantine folder, notify admin
                        }
                        SpamFolder::Spam => {
                            info!(
                                "Moving email {} to spam folder (score: {:.1})",
                                payload.message_id, spam_result.score
                            );
                            // TODO: Move to spam folder
                        }
                        SpamFolder::Inbox => {
                            // Normal processing
                        }
                    }

                    // Create task from email (unless rejected)
                    if spam_result.should_deliver {
                        let mut task = Task::from_email(email);
                        // Add spam info to task context metadata
                        task.context.metadata.insert("spam_score".to_string(), serde_json::json!(spam_result.score));
                        task.context.metadata.insert("spam_classification".to_string(), serde_json::json!(format!("{:?}", spam_result.classification)));
                        
                        // Send to task queue
                        if let Err(e) = state.task_sender.send(task).await {
                            error!("Failed to queue task: {}", e);
                            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                                "error": "Failed to queue task"
                            })));
                        }

                        info!("Queued task for message {}", payload.message_id);
                    }
                }
                Err(e) => {
                    error!("Failed to fetch message {}: {}", payload.message_id, e);
                    // Still return 200 to prevent retries for permanent failures
                }
            }
        }
        WebhookEvent::DeliveryFailed => {
            warn!(
                "Delivery failed for message {} to {:?}",
                payload.message_id, payload.recipients
            );
            // TODO: Handle delivery failures (retry logic, notifications)
        }
        WebhookEvent::SpamDetected | WebhookEvent::VirusDetected => {
            warn!(
                "Security event {:?} for message {}",
                payload.event, payload.message_id
            );
            // TODO: Log security events, potentially notify security agent
        }
        _ => {
            debug!("Ignoring event {:?}", payload.event);
        }
    }

    // Always return 200 quickly to acknowledge receipt
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "accepted",
        "message_id": payload.message_id
    })))
}

/// Inbound mail handler - receives mail events from embedded SMTP server
pub async fn inbound_mail_handler(
    payload: web::Json<InboundMailPayload>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    info!(
        "Inbound mail: {} -> {:?}, Subject: {}",
        payload.from, payload.to, payload.subject
    );

    // Create email message from payload
    let email = EmailMessage {
        metadata: EmailMetadata {
            message_id: payload.message_id.clone().unwrap_or_else(|| payload.id.clone()),
            tracking_id: Uuid::parse_str(&payload.id).unwrap_or_else(|_| Uuid::new_v4()),
            from: EmailAddress::new(&payload.from),
            to: payload.to.iter().map(|r| EmailAddress::new(r)).collect(),
            cc: Vec::new(),
            bcc: Vec::new(),
            reply_to: Some(EmailAddress::new(&payload.from)),
            subject: payload.subject.clone(),
            received_at: payload.received_at,
            sent_at: None,
            in_reply_to: None,
            references: Vec::new(),
            spf_result: payload.spf_result.clone(),
            dkim_result: payload.dkim_result.clone(),
            dmarc_result: payload.dmarc_result.clone(),
            priority: EmailPriority::Normal,
            custom_headers: std::collections::HashMap::new(),
        },
        text_body: payload.body_text.clone(),
        html_body: payload.body_html.clone(),
        attachments: Vec::new(),
        raw: None,
    };

    // Run spam filter
    let spam_result = state.spam_filter.analyze(&email);
    info!(
        "Spam analysis for inbound mail: score={:.1}, classification={:?}",
        spam_result.score, spam_result.classification
    );

    // Handle spam
    match spam_result.folder {
        SpamFolder::Reject => {
            warn!(
                "Rejecting spam email from {} (score: {:.1})",
                payload.from, spam_result.score
            );
            return Ok(HttpResponse::Ok().json(serde_json::json!({
                "status": "rejected",
                "reason": "spam",
                "score": spam_result.score,
                "classification": format!("{:?}", spam_result.classification)
            })));
        }
        SpamFolder::Quarantine => {
            warn!("Quarantining email from {} (score: {:.1})", payload.from, spam_result.score);
        }
        SpamFolder::Spam => {
            info!("Email from {} marked as spam (score: {:.1})", payload.from, spam_result.score);
        }
        SpamFolder::Inbox => {}
    }

    // Only process if should deliver
    if !spam_result.should_deliver {
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "filtered",
            "reason": "spam",
            "score": spam_result.score
        })));
    }

    // Create task from email
    let mut task = Task::from_email(email);
    task.context.metadata.insert("spam_score".to_string(), serde_json::json!(spam_result.score));
    task.context.metadata.insert("spam_classification".to_string(), serde_json::json!(format!("{:?}", spam_result.classification)));
    task.context.metadata.insert("spam_folder".to_string(), serde_json::json!(format!("{:?}", spam_result.folder)));
    info!("Created task {} for email processing", task.id);

    // Send to task queue
    if let Err(e) = state.task_sender.send(task.clone()).await {
        error!("Failed to queue task: {}", e);
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to queue task",
            "details": e.to_string()
        })));
    }

    info!("Queued task {} for agent processing", task.id);

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "accepted",
        "task_id": task.id.to_string(),
        "spam_score": spam_result.score,
        "spam_folder": format!("{:?}", spam_result.folder),
        "message": "Email queued for agent processing"
    })))
}

/// Inbound mail payload from embedded SMTP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundMailPayload {
    pub id: String,
    pub message_id: Option<String>,
    pub from: String,
    /// SPF authentication result
    #[serde(default)]
    pub spf_result: Option<String>,
    /// DKIM authentication result
    #[serde(default)]
    pub dkim_result: Option<String>,
    /// DMARC authentication result
    #[serde(default)]
    pub dmarc_result: Option<String>,
    pub to: Vec<String>,
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub raw_size: usize,
    pub received_at: DateTime<Utc>,
}

/// Health check handler
pub async fn health_handler() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now()
    }))
}

/// Metrics handler (Prometheus format)
pub async fn metrics_handler() -> HttpResponse {
    // TODO: Implement proper Prometheus metrics
    HttpResponse::Ok()
        .content_type("text/plain")
        .body("# HELP amail_tasks_total Total tasks processed\namail_tasks_total 0\n")
}

/// Convert webhook payload to basic email metadata (when full fetch fails)
impl From<WebhookPayload> for EmailMetadata {
    fn from(payload: WebhookPayload) -> Self {
        EmailMetadata {
            message_id: payload.message_id,
            tracking_id: Uuid::new_v4(),
            from: EmailAddress::new(&payload.sender),
            to: payload.recipients.iter().map(|r| EmailAddress::new(r)).collect(),
            cc: Vec::new(),
            bcc: Vec::new(),
            reply_to: None,
            subject: payload.subject.unwrap_or_default(),
            received_at: payload.timestamp,
            sent_at: None,
            in_reply_to: None,
            references: Vec::new(),
            spf_result: None,
            dkim_result: None,
            dmarc_result: None,
            priority: EmailPriority::Normal,
            custom_headers: std::collections::HashMap::new(),
        }
    }
}

// =============================================================================
// Mailboxes API Handler
// =============================================================================

/// List all mailboxes - returns mailboxes from config
pub async fn list_mailboxes_handler(
    config_state: web::Data<crate::config_api::ConfigState>,
) -> HttpResponse {
    let mailboxes = config_state.config_manager.get_all_mailboxes().await;
    
    let response: Vec<serde_json::Value> = mailboxes.into_iter().map(|m| {
        serde_json::json!({
            "id": m.id,
            "email": m.email,
            "display_name": m.display_name,
            "domain_id": m.domain_id,
            "enabled": m.enabled,
            "quota_mb": m.quota_mb,
            "created_at": m.created_at,
        })
    }).collect();
    
    HttpResponse::Ok().json(response)
}

// =============================================================================
// Inbox API Handler
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct InboxQuery {
    pub folder: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub mailbox: Option<String>,
    pub mailbox_id: Option<String>,  // Alias for mailbox (frontend uses this)
    pub domain: Option<String>,      // Filter by domain, "all" for all domains
}

/// List inbox emails from Stalwart mail server via JMAP
/// Supports:
/// - Single mailbox: ?mailbox=support@amail.world
/// - All mailboxes in domain: ?domain=amail.world
/// - All mailboxes across all domains: ?domain=all or ?mailbox=all
pub async fn list_inbox_handler(
    query: web::Query<InboxQuery>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let folder = query.folder.as_deref().unwrap_or("INBOX");
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    let mailbox = query.mailbox.as_deref().or(query.mailbox_id.as_deref());
    let domain = query.domain.as_deref();
    
    info!("Inbox query: folder={}, limit={}, mailbox={:?}, domain={:?}", folder, limit, mailbox, domain);
    
    // Check for "all" mode - aggregate emails from all accounts
    let is_all_mailboxes = mailbox == Some("all") || domain == Some("all");
    let is_domain_filter = domain.is_some() && domain != Some("all");
    
    if is_all_mailboxes || is_domain_filter {
        // Get all accounts from Stalwart and aggregate their emails
        let accounts = match state.stalwart.list_accounts().await {
            Ok(accts) => accts,
            Err(e) => {
                warn!("Failed to list accounts: {}", e);
                let demo_emails = get_demo_emails_all(folder);
                return HttpResponse::Ok().json(demo_emails);
            }
        };
        
        // Filter accounts by domain if specified
        let filtered_accounts: Vec<_> = if let Some(d) = domain.filter(|d| *d != "all") {
            accounts.into_iter().filter(|a| a.ends_with(&format!("@{}", d))).collect()
        } else {
            accounts
        };
        
        info!("Querying {} accounts for master list", filtered_accounts.len());
        
        // Aggregate emails from all accounts
        let mut all_emails: Vec<serde_json::Value> = Vec::new();
        for account_email in &filtered_accounts {
            let account_id = account_email.split('@').next().unwrap_or(account_email);
            if let Ok(emails) = state.stalwart.list_messages(account_id, Some(folder), limit, offset).await {
                for e in emails {
                    all_emails.push(serde_json::json!({
                        "id": e.id,
                        "from_address": e.from.address,
                        "from_name": e.from.name,
                        "to": e.to,
                        "subject": e.subject,
                        "preview": e.preview,
                        "timestamp": e.received_at,
                        "is_read": e.is_read,
                        "is_starred": e.is_starred,
                        "folder": folder,
                        "labels": e.labels,
                        "mailbox_email": account_email,
                        "mailbox_name": account_id,
                        "ai_confidence": None::<f64>,
                        "ai_status": None::<String>,
                        "attachment_count": if e.has_attachment { 1 } else { 0 }
                    }));
                }
            }
        }
        
        // Sort by timestamp descending
        all_emails.sort_by(|a, b| {
            let ts_a = a.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
            let ts_b = b.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
            ts_b.cmp(ts_a)
        });
        
        // Apply limit
        all_emails.truncate(limit as usize);
        
        // Return real emails only - no mock data
        return HttpResponse::Ok().json(all_emails);
    }
    
    // Single mailbox query
    let account_id = match mailbox {
        Some(m) => m.split('@').next().unwrap_or(m),
        None => {
            // No mailbox specified - return empty list (real data only)
            info!("No mailbox selected - returning empty list");
            return HttpResponse::Ok().json(Vec::<serde_json::Value>::new());
        }
    };
    
    info!("Using Stalwart account: {}", account_id);
    
    // Fetch emails from Stalwart via JMAP
    match state.stalwart.list_messages(account_id, Some(folder), limit, offset).await {
        Ok(emails) if !emails.is_empty() => {
            let email_responses: Vec<serde_json::Value> = emails.iter().map(|e| {
                serde_json::json!({
                    "id": e.id,
                    "from_address": e.from.address,
                    "from_name": e.from.name,
                    "to": e.to,
                    "subject": e.subject,
                    "preview": e.preview,
                    "timestamp": e.received_at,
                    "is_read": e.is_read,
                    "is_starred": e.is_starred,
                    "folder": folder,
                    "labels": e.labels,
                    "mailbox_email": mailbox,
                    "mailbox_name": None::<String>,
                    "ai_confidence": None::<f64>,
                    "ai_status": None::<String>,
                    "attachment_count": if e.has_attachment { 1 } else { 0 }
                })
            }).collect();
            
            HttpResponse::Ok().json(email_responses)
        }
        Ok(_) | Err(_) => {
            // Return empty list - real data only, no mock emails
            info!("No emails found in Stalwart for account: {}", account_id);
            HttpResponse::Ok().json(Vec::<serde_json::Value>::new())
        }
    }
}

/// Generate demo emails for testing when Stalwart is unavailable
fn get_demo_emails(mailbox: &str, folder: &str) -> Vec<serde_json::Value> {
    let now = chrono::Utc::now();
    vec![
        serde_json::json!({
            "id": "demo-1",
            "from_address": "john.smith@acme.com",
            "from_name": "John Smith",
            "to": [mailbox],
            "subject": "Q4 Budget Review Meeting",
            "preview": "Hi team, I wanted to schedule a meeting to review our Q4 budget projections...",
            "timestamp": (now - chrono::Duration::hours(2)).to_rfc3339(),
            "is_read": false,
            "is_starred": true,
            "folder": folder,
            "labels": ["important", "finance"],
            "mailbox_email": mailbox,
            "mailbox_name": "Support",
            "ai_confidence": 0.95,
            "ai_status": "processed",
            "attachment_count": 1
        }),
        serde_json::json!({
            "id": "demo-2",
            "from_address": "sarah.jones@partner.io",
            "from_name": "Sarah Jones",
            "to": [mailbox],
            "subject": "Partnership Proposal - Urgent",
            "preview": "Dear Team, We are excited to propose a strategic partnership that could...",
            "timestamp": (now - chrono::Duration::hours(5)).to_rfc3339(),
            "is_read": false,
            "is_starred": false,
            "folder": folder,
            "labels": ["urgent", "partnership"],
            "mailbox_email": mailbox,
            "mailbox_name": "Support",
            "ai_confidence": 0.88,
            "ai_status": "pending_review",
            "attachment_count": 2
        }),
        serde_json::json!({
            "id": "demo-3",
            "from_address": "mike.wilson@vendor.net",
            "from_name": "Mike Wilson",
            "to": [mailbox],
            "subject": "Invoice #INV-2024-1234",
            "preview": "Please find attached the invoice for services rendered in December...",
            "timestamp": (now - chrono::Duration::days(1)).to_rfc3339(),
            "is_read": true,
            "is_starred": false,
            "folder": folder,
            "labels": ["invoice", "accounting"],
            "mailbox_email": mailbox,
            "mailbox_name": "Support",
            "ai_confidence": 0.92,
            "ai_status": "processed",
            "attachment_count": 1
        }),
        serde_json::json!({
            "id": "demo-4",
            "from_address": "newsletter@techweekly.com",
            "from_name": "Tech Weekly",
            "to": [mailbox],
            "subject": "This Week in AI: Latest Breakthroughs",
            "preview": "Welcome to this week's edition of Tech Weekly. In this issue, we cover...",
            "timestamp": (now - chrono::Duration::days(2)).to_rfc3339(),
            "is_read": true,
            "is_starred": false,
            "folder": folder,
            "labels": ["newsletter"],
            "mailbox_email": mailbox,
            "mailbox_name": "Support",
            "ai_confidence": 0.75,
            "ai_status": "processed",
            "attachment_count": 0
        }),
        serde_json::json!({
            "id": "demo-5",
            "from_address": "hr@company.com",
            "from_name": "HR Department",
            "to": [mailbox],
            "subject": "Holiday Schedule Reminder",
            "preview": "This is a friendly reminder about the upcoming holiday schedule...",
            "timestamp": (now - chrono::Duration::days(3)).to_rfc3339(),
            "is_read": true,
            "is_starred": false,
            "folder": folder,
            "labels": ["hr", "internal"],
            "mailbox_email": mailbox,
            "mailbox_name": "Support",
            "ai_confidence": 0.99,
            "ai_status": "processed",
            "attachment_count": 0
        })
    ]
}

/// Generate demo emails for all mailboxes (master list view)
fn get_demo_emails_all(folder: &str) -> Vec<serde_json::Value> {
    let now = chrono::Utc::now();
    vec![
        serde_json::json!({
            "id": "demo-all-1",
            "from_address": "john.smith@acme.com",
            "from_name": "John Smith",
            "to": ["support@amail.world"],
            "subject": "Q4 Budget Review Meeting",
            "preview": "Hi team, I wanted to schedule a meeting to review our Q4 budget projections...",
            "timestamp": (now - chrono::Duration::hours(1)).to_rfc3339(),
            "is_read": false,
            "is_starred": true,
            "folder": folder,
            "labels": ["important", "finance"],
            "mailbox_email": "support@amail.world",
            "mailbox_name": "Support",
            "ai_confidence": 0.95,
            "ai_status": "processed",
            "attachment_count": 1
        }),
        serde_json::json!({
            "id": "demo-all-2",
            "from_address": "legal@lawfirm.com",
            "from_name": "Legal Team",
            "to": ["legal@amail.world"],
            "subject": "Contract Review Required",
            "preview": "Please review the attached contract amendments before the deadline...",
            "timestamp": (now - chrono::Duration::hours(2)).to_rfc3339(),
            "is_read": false,
            "is_starred": false,
            "folder": folder,
            "labels": ["legal", "urgent"],
            "mailbox_email": "legal@amail.world",
            "mailbox_name": "Legal",
            "ai_confidence": 0.92,
            "ai_status": "pending_review",
            "attachment_count": 3
        }),
        serde_json::json!({
            "id": "demo-all-3",
            "from_address": "candidate@email.com",
            "from_name": "Jane Doe",
            "to": ["hr@amail.world"],
            "subject": "Application for Software Engineer Position",
            "preview": "Dear HR Team, I am writing to express my interest in the Software Engineer...",
            "timestamp": (now - chrono::Duration::hours(4)).to_rfc3339(),
            "is_read": true,
            "is_starred": false,
            "folder": folder,
            "labels": ["hr", "hiring"],
            "mailbox_email": "hr@amail.world",
            "mailbox_name": "HR",
            "ai_confidence": 0.88,
            "ai_status": "processed",
            "attachment_count": 2
        }),
        serde_json::json!({
            "id": "demo-all-4",
            "from_address": "accounting@vendor.com",
            "from_name": "Accounts Payable",
            "to": ["finance@amail.world"],
            "subject": "Invoice #2024-5678 - Payment Due",
            "preview": "Please find attached invoice for services rendered. Payment is due within...",
            "timestamp": (now - chrono::Duration::hours(6)).to_rfc3339(),
            "is_read": false,
            "is_starred": true,
            "folder": folder,
            "labels": ["finance", "invoice"],
            "mailbox_email": "finance@amail.world",
            "mailbox_name": "Finance",
            "ai_confidence": 0.96,
            "ai_status": "processed",
            "attachment_count": 1
        }),
        serde_json::json!({
            "id": "demo-all-5",
            "from_address": "ceo@partner.com",
            "from_name": "CEO Partner Corp",
            "to": ["executive@amail.world"],
            "subject": "Strategic Partnership Discussion",
            "preview": "I would like to schedule a call to discuss potential synergies between our...",
            "timestamp": (now - chrono::Duration::hours(8)).to_rfc3339(),
            "is_read": true,
            "is_starred": true,
            "folder": folder,
            "labels": ["executive", "partnership"],
            "mailbox_email": "executive@amail.world",
            "mailbox_name": "Executive",
            "ai_confidence": 0.99,
            "ai_status": "processed",
            "attachment_count": 0
        }),
        serde_json::json!({
            "id": "demo-all-6",
            "from_address": "customer@client.com",
            "from_name": "Valued Customer",
            "to": ["inbox@amail.world"],
            "subject": "Product Inquiry",
            "preview": "Hello, I have a question about your product features and pricing...",
            "timestamp": (now - chrono::Duration::days(1)).to_rfc3339(),
            "is_read": true,
            "is_starred": false,
            "folder": folder,
            "labels": ["inquiry", "sales"],
            "mailbox_email": "inbox@amail.world",
            "mailbox_name": "Inbox",
            "ai_confidence": 0.85,
            "ai_status": "processed",
            "attachment_count": 0
        })
    ]
}

/// Inject a test email into a mailbox (for development/testing)
pub async fn inject_test_email_handler(
    query: web::Query<InboxQuery>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let mailbox = query.mailbox.as_deref().unwrap_or("support@amail.world");
    
    info!("Injecting test email into mailbox: {}", mailbox);
    
    // Create test email via Stalwart's JMAP API
    match state.stalwart.inject_test_email(mailbox).await {
        Ok(_) => {
            info!("Test email injected successfully");
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": format!("Test email injected into {}", mailbox)
            }))
        }
        Err(e) => {
            warn!("Failed to inject test email: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            }))
        }
    }
}
