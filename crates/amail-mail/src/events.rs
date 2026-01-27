//! Mail event system
//!
//! Handles events from the mail server and routes them to Amail agents.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{info, debug, error};

// =============================================================================
// Mail Events
// =============================================================================

/// Events emitted by the mail server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MailEvent {
    /// New message received
    MessageReceived {
        id: String,
        message_id: Option<String>,
        from: String,
        to: Vec<String>,
        subject: String,
        body_text: Option<String>,
        body_html: Option<String>,
        raw_size: usize,
        received_at: DateTime<Utc>,
    },
    /// Message sent successfully
    MessageSent {
        id: String,
        message_id: String,
        from: String,
        to: Vec<String>,
        subject: String,
        sent_at: DateTime<Utc>,
    },
    /// Message delivery failed
    DeliveryFailed {
        id: String,
        message_id: Option<String>,
        from: String,
        to: Vec<String>,
        error: String,
        failed_at: DateTime<Utc>,
    },
    /// Bounce received
    BounceReceived {
        original_message_id: String,
        bounce_type: String, // hard, soft, complaint
        recipient: String,
        reason: String,
        received_at: DateTime<Utc>,
    },
    /// Domain verified
    DomainVerified {
        domain: String,
        tenant_id: String,
        verified_at: DateTime<Utc>,
    },
    /// Spam detected
    SpamDetected {
        id: String,
        from: String,
        subject: String,
        spam_score: f64,
        detected_at: DateTime<Utc>,
    },
}

// =============================================================================
// Event Handler
// =============================================================================

/// Handles mail events and routes them to appropriate handlers
pub struct MailEventHandler {
    rx: mpsc::Receiver<MailEvent>,
    webhook_url: Option<String>,
    http_client: reqwest::Client,
}

impl MailEventHandler {
    /// Create a new event handler
    pub fn new(rx: mpsc::Receiver<MailEvent>, webhook_url: Option<String>) -> Self {
        Self {
            rx,
            webhook_url,
            http_client: reqwest::Client::new(),
        }
    }

    /// Start processing events
    pub async fn run(mut self) {
        info!("Mail event handler started");
        
        while let Some(event) = self.rx.recv().await {
            debug!("Received mail event: {:?}", event);
            
            // Forward to webhook if configured
            if let Some(url) = &self.webhook_url {
                if let Err(e) = self.forward_to_webhook(url, &event).await {
                    error!("Failed to forward event to webhook: {}", e);
                }
            }

            // Handle event locally
            self.handle_event(&event).await;
        }

        info!("Mail event handler stopped");
    }

    /// Forward event to webhook URL
    async fn forward_to_webhook(&self, url: &str, event: &MailEvent) -> Result<(), reqwest::Error> {
        self.http_client
            .post(url)
            .json(event)
            .send()
            .await?;
        
        debug!("Forwarded event to webhook: {}", url);
        Ok(())
    }

    /// Handle event locally
    async fn handle_event(&self, event: &MailEvent) {
        match event {
            MailEvent::MessageReceived { id, from, to, subject, .. } => {
                info!("New message {}: {} -> {:?}, Subject: {}", id, from, to, subject);
            }
            MailEvent::MessageSent { id, from, to, subject, .. } => {
                info!("Message sent {}: {} -> {:?}, Subject: {}", id, from, to, subject);
            }
            MailEvent::DeliveryFailed { id, error, .. } => {
                error!("Delivery failed {}: {}", id, error);
            }
            MailEvent::BounceReceived { original_message_id, bounce_type, recipient, reason, .. } => {
                info!("Bounce received for {}: {} bounce from {} - {}", 
                      original_message_id, bounce_type, recipient, reason);
            }
            MailEvent::DomainVerified { domain, tenant_id, .. } => {
                info!("Domain {} verified for tenant {}", domain, tenant_id);
            }
            MailEvent::SpamDetected { id, from, spam_score, .. } => {
                info!("Spam detected {}: from {} with score {}", id, from, spam_score);
            }
        }
    }
}

// =============================================================================
// Event Channel
// =============================================================================

/// Create a new event channel
pub fn create_event_channel(buffer_size: usize) -> (mpsc::Sender<MailEvent>, mpsc::Receiver<MailEvent>) {
    mpsc::channel(buffer_size)
}
