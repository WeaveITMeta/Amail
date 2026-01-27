//! Telegram Bot Integration - Notifications & Human Review
//!
//! Table of Contents:
//! 1. TelegramConfig - Bot configuration
//! 2. TelegramClient - API client for sending/receiving messages
//! 3. ReviewRequest - Pending human review with message threading
//! 4. ReviewResponse - Human decision (approve/reject/edit)
//! 5. NotificationType - Different notification categories
//!
//! Key Feature: Reply-based threading to avoid message mix-up
//! Each email notification creates a thread, replies to that thread
//! are automatically associated with the correct email/review.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Telegram bot configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    /// Bot token from @BotFather
    pub bot_token: String,
    /// Chat ID to send notifications to (user or group)
    pub chat_id: i64,
    /// Confidence threshold (0-100) - below this triggers human review
    /// Default: 75% - anything below requires review
    pub review_threshold: u8,
    /// Auto-approve threshold (0-100) - at or above this auto-sends
    /// Default: 75% - anything at or above auto-approves
    pub auto_approve_threshold: u8,
    /// Enable notifications
    pub enabled: bool,
    /// Notification preferences
    pub notify_on: NotificationPreferences,
}

impl Default for TelegramConfig {
    fn default() -> Self {
        Self {
            bot_token: String::new(),
            chat_id: 0,
            review_threshold: 75,
            auto_approve_threshold: 75,
            enabled: false,
            notify_on: NotificationPreferences::default(),
        }
    }
}

/// What events trigger notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub new_email: bool,
    pub ai_response_sent: bool,
    pub review_needed: bool,
    pub review_timeout: bool,
    pub daily_summary: bool,
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            new_email: false,
            ai_response_sent: true,
            review_needed: true,
            review_timeout: true,
            daily_summary: true,
        }
    }
}

/// Types of notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    /// New email received
    NewEmail {
        from: String,
        subject: String,
        preview: String,
    },
    /// AI drafted a response (high confidence, auto-sent)
    AiResponseSent {
        email_id: String,
        to: String,
        subject: String,
        confidence: u8,
        agent: String,
    },
    /// AI needs human review (low confidence)
    ReviewNeeded {
        review_id: String,
        email_id: String,
        from: String,
        subject: String,
        draft_preview: String,
        confidence: u8,
        agent: String,
    },
    /// Review timed out
    ReviewTimeout {
        review_id: String,
        email_id: String,
        action_taken: String,
    },
    /// Daily summary
    DailySummary {
        emails_received: u32,
        ai_responses: u32,
        reviews_pending: u32,
        reviews_approved: u32,
        reviews_rejected: u32,
    },
}

/// A pending review request - tracks the Telegram message for threading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewRequest {
    pub id: String,
    pub email_id: String,
    pub telegram_message_id: i64,  // For reply threading
    pub chat_id: i64,
    pub from: String,
    pub subject: String,
    pub original_body: String,
    pub draft_response: String,
    pub confidence: u8,
    pub agent_role: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: ReviewStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewStatus {
    Pending,
    Approved,
    Rejected,
    Edited,
    Expired,
}

/// Human's response to a review request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResponse {
    pub review_id: String,
    pub decision: ReviewDecision,
    pub edited_response: Option<String>,
    pub responded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReviewDecision {
    Approve,  // Send the AI draft as-is
    Reject,   // Don't send, discard draft
    Edit,     // Send with human edits
}

/// Telegram API response types
#[derive(Debug, Deserialize)]
pub struct TelegramResponse<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub chat: TelegramChat,
    pub text: Option<String>,
    pub reply_to_message: Option<Box<TelegramMessage>>,
}

#[derive(Debug, Deserialize)]
pub struct TelegramChat {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
    pub callback_query: Option<CallbackQuery>,
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub id: String,
    pub message: Option<TelegramMessage>,
    pub data: Option<String>,
}

/// Telegram client for sending notifications and receiving replies
#[derive(Clone)]
pub struct TelegramClient {
    config: TelegramConfig,
    http_client: reqwest::Client,
    /// Maps telegram_message_id -> review_id for reply threading
    pending_reviews: Arc<RwLock<HashMap<i64, ReviewRequest>>>,
    /// Last processed update ID for polling
    last_update_id: Arc<RwLock<i64>>,
}

impl TelegramClient {
    pub fn new(config: TelegramConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
            pending_reviews: Arc::new(RwLock::new(HashMap::new())),
            last_update_id: Arc::new(RwLock::new(0)),
        }
    }

    /// Base URL for Telegram Bot API
    fn api_url(&self, method: &str) -> String {
        format!(
            "https://api.telegram.org/bot{}/{}",
            self.config.bot_token, method
        )
    }

    /// Send a notification message
    pub async fn send_notification(
        &self,
        notification: NotificationType,
    ) -> Result<Option<i64>, TelegramError> {
        if !self.config.enabled {
            return Ok(None);
        }

        let (text, parse_mode, reply_markup) = self.format_notification(&notification);

        let mut params = serde_json::json!({
            "chat_id": self.config.chat_id,
            "text": text,
            "parse_mode": parse_mode,
        });

        if let Some(markup) = reply_markup {
            params["reply_markup"] = markup;
        }

        let response: TelegramResponse<TelegramMessage> = self
            .http_client
            .post(self.api_url("sendMessage"))
            .json(&params)
            .send()
            .await
            .map_err(|e| TelegramError::NetworkError(e.to_string()))?
            .json()
            .await
            .map_err(|e| TelegramError::ParseError(e.to_string()))?;

        if response.ok {
            let message_id = response.result.map(|m| m.message_id);
            
            // If this is a review request, store the mapping for reply threading
            if let NotificationType::ReviewNeeded { review_id, email_id, from, subject, draft_preview, confidence, agent } = notification {
                if let Some(msg_id) = message_id {
                    let review = ReviewRequest {
                        id: review_id,
                        email_id,
                        telegram_message_id: msg_id,
                        chat_id: self.config.chat_id,
                        from,
                        subject,
                        original_body: String::new(), // Would be populated from email
                        draft_response: draft_preview,
                        confidence,
                        agent_role: agent,
                        created_at: Utc::now(),
                        expires_at: Utc::now() + chrono::Duration::hours(24),
                        status: ReviewStatus::Pending,
                    };
                    self.pending_reviews.write().await.insert(msg_id, review);
                }
            }
            
            Ok(message_id)
        } else {
            Err(TelegramError::ApiError(
                response.description.unwrap_or_else(|| "Unknown error".to_string()),
            ))
        }
    }

    /// Format notification for Telegram (returns text, parse_mode, optional reply_markup)
    fn format_notification(
        &self,
        notification: &NotificationType,
    ) -> (String, &'static str, Option<serde_json::Value>) {
        match notification {
            NotificationType::NewEmail { from, subject, preview } => {
                let text = format!(
                    "📬 *New Email*\n\n\
                    *From:* {}\n\
                    *Subject:* {}\n\n\
                    _{}_",
                    escape_markdown(from),
                    escape_markdown(subject),
                    escape_markdown(&truncate(preview, 200))
                );
                (text, "MarkdownV2", None)
            }
            NotificationType::AiResponseSent { to, subject, confidence, agent, .. } => {
                let text = format!(
                    "✅ *AI Response Sent*\n\n\
                    *To:* {}\n\
                    *Subject:* {}\n\
                    *Agent:* {}\n\
                    *Confidence:* {}%",
                    escape_markdown(to),
                    escape_markdown(subject),
                    escape_markdown(agent),
                    confidence
                );
                (text, "MarkdownV2", None)
            }
            NotificationType::ReviewNeeded { review_id, from, subject, draft_preview, confidence, agent, .. } => {
                let text = format!(
                    "⚠️ *Human Review Needed*\n\n\
                    *From:* {}\n\
                    *Subject:* {}\n\
                    *Agent:* {} \\({}% confidence\\)\n\n\
                    *Draft Response:*\n\
                    _{}_\n\n\
                    Reply to this message to edit, or use buttons below\\.",
                    escape_markdown(from),
                    escape_markdown(subject),
                    escape_markdown(agent),
                    confidence,
                    escape_markdown(&truncate(draft_preview, 500))
                );
                
                let markup = serde_json::json!({
                    "inline_keyboard": [[
                        {"text": "✅ Approve", "callback_data": format!("approve:{}", review_id)},
                        {"text": "❌ Reject", "callback_data": format!("reject:{}", review_id)}
                    ]]
                });
                
                (text, "MarkdownV2", Some(markup))
            }
            NotificationType::ReviewTimeout { email_id, action_taken, .. } => {
                let text = format!(
                    "⏰ *Review Timeout*\n\n\
                    Email {} review expired\\.\n\
                    Action: {}",
                    escape_markdown(email_id),
                    escape_markdown(action_taken)
                );
                (text, "MarkdownV2", None)
            }
            NotificationType::DailySummary { emails_received, ai_responses, reviews_pending, reviews_approved, reviews_rejected } => {
                let text = format!(
                    "📊 *Daily Summary*\n\n\
                    📬 Emails received: {}\n\
                    🤖 AI responses: {}\n\
                    ⏳ Reviews pending: {}\n\
                    ✅ Reviews approved: {}\n\
                    ❌ Reviews rejected: {}",
                    emails_received, ai_responses, reviews_pending, reviews_approved, reviews_rejected
                );
                (text, "MarkdownV2", None)
            }
        }
    }

    /// Poll for updates (replies to review messages)
    pub async fn poll_updates(&self) -> Result<Vec<ReviewResponse>, TelegramError> {
        if !self.config.enabled {
            return Ok(vec![]);
        }

        let last_id = *self.last_update_id.read().await;
        
        let params = serde_json::json!({
            "offset": last_id + 1,
            "timeout": 30,
            "allowed_updates": ["message", "callback_query"]
        });

        let response: TelegramResponse<Vec<TelegramUpdate>> = self
            .http_client
            .post(self.api_url("getUpdates"))
            .json(&params)
            .send()
            .await
            .map_err(|e| TelegramError::NetworkError(e.to_string()))?
            .json()
            .await
            .map_err(|e| TelegramError::ParseError(e.to_string()))?;

        if !response.ok {
            return Err(TelegramError::ApiError(
                response.description.unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        let updates = response.result.unwrap_or_default();
        let mut responses = Vec::new();

        for update in updates {
            // Update last processed ID
            *self.last_update_id.write().await = update.update_id;

            // Handle callback queries (button presses)
            if let Some(callback) = update.callback_query {
                if let Some(data) = callback.data {
                    if let Some(response) = self.handle_callback(&data).await {
                        responses.push(response);
                    }
                    // Answer callback to remove loading state
                    let _ = self.answer_callback(&callback.id).await;
                }
            }

            // Handle message replies (edited responses)
            if let Some(message) = update.message {
                if let Some(ref reply_to) = message.reply_to_message {
                    if let Some(response) = self.handle_reply(reply_to, &message).await {
                        responses.push(response);
                    }
                }
            }
        }

        Ok(responses)
    }

    /// Handle callback query (approve/reject button)
    async fn handle_callback(&self, data: &str) -> Option<ReviewResponse> {
        let parts: Vec<&str> = data.split(':').collect();
        if parts.len() != 2 {
            return None;
        }

        let (action, review_id) = (parts[0], parts[1]);
        
        let decision = match action {
            "approve" => ReviewDecision::Approve,
            "reject" => ReviewDecision::Reject,
            _ => return None,
        };

        // Find and update the review
        let mut reviews = self.pending_reviews.write().await;
        for review in reviews.values_mut() {
            if review.id == review_id {
                review.status = match decision {
                    ReviewDecision::Approve => ReviewStatus::Approved,
                    ReviewDecision::Reject => ReviewStatus::Rejected,
                    ReviewDecision::Edit => ReviewStatus::Edited,
                };
                
                return Some(ReviewResponse {
                    review_id: review_id.to_string(),
                    decision,
                    edited_response: None,
                    responded_at: Utc::now(),
                });
            }
        }

        None
    }

    /// Handle reply message (edited response text)
    async fn handle_reply(
        &self,
        reply_to: &TelegramMessage,
        message: &TelegramMessage,
    ) -> Option<ReviewResponse> {
        let reviews = self.pending_reviews.read().await;
        
        // Find review by the message being replied to
        if let Some(review) = reviews.get(&reply_to.message_id) {
            if review.status == ReviewStatus::Pending {
                let edited_text = message.text.clone()?;
                
                return Some(ReviewResponse {
                    review_id: review.id.clone(),
                    decision: ReviewDecision::Edit,
                    edited_response: Some(edited_text),
                    responded_at: Utc::now(),
                });
            }
        }

        None
    }

    /// Answer callback query to remove loading indicator
    async fn answer_callback(&self, callback_id: &str) -> Result<(), TelegramError> {
        let params = serde_json::json!({
            "callback_query_id": callback_id,
            "text": "Response recorded!"
        });

        self.http_client
            .post(self.api_url("answerCallbackQuery"))
            .json(&params)
            .send()
            .await
            .map_err(|e| TelegramError::NetworkError(e.to_string()))?;

        Ok(())
    }

    /// Update message after review decision
    pub async fn update_review_message(
        &self,
        message_id: i64,
        decision: &ReviewDecision,
    ) -> Result<(), TelegramError> {
        let status_text = match decision {
            ReviewDecision::Approve => "✅ *APPROVED* \\- Response sent",
            ReviewDecision::Reject => "❌ *REJECTED* \\- Response discarded",
            ReviewDecision::Edit => "✏️ *EDITED* \\- Modified response sent",
        };

        let params = serde_json::json!({
            "chat_id": self.config.chat_id,
            "message_id": message_id,
            "text": status_text,
            "parse_mode": "MarkdownV2"
        });

        self.http_client
            .post(self.api_url("editMessageText"))
            .json(&params)
            .send()
            .await
            .map_err(|e| TelegramError::NetworkError(e.to_string()))?;

        Ok(())
    }

    /// Get pending reviews
    pub async fn get_pending_reviews(&self) -> Vec<ReviewRequest> {
        self.pending_reviews
            .read()
            .await
            .values()
            .filter(|r| r.status == ReviewStatus::Pending)
            .cloned()
            .collect()
    }

    /// Check if review threshold requires human review
    pub fn needs_review(&self, confidence: u8) -> bool {
        confidence < self.config.review_threshold
    }

    /// Check if confidence is high enough for auto-approve
    pub fn can_auto_approve(&self, confidence: u8) -> bool {
        confidence >= self.config.auto_approve_threshold
    }
}

/// Telegram-specific errors
#[derive(Debug, thiserror::Error)]
pub enum TelegramError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Not configured")]
    NotConfigured,
}

/// Escape special characters for MarkdownV2
fn escape_markdown(text: &str) -> String {
    let special_chars = ['_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!'];
    let mut result = String::with_capacity(text.len() * 2);
    for c in text.chars() {
        if special_chars.contains(&c) {
            result.push('\\');
        }
        result.push(c);
    }
    result
}

/// Truncate text with ellipsis
fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_markdown() {
        assert_eq!(escape_markdown("Hello_World"), "Hello\\_World");
        assert_eq!(escape_markdown("*bold*"), "\\*bold\\*");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a long text", 10), "this is...");
    }

    #[test]
    fn test_confidence_thresholds() {
        let config = TelegramConfig {
            review_threshold: 70,
            auto_approve_threshold: 90,
            ..Default::default()
        };
        let client = TelegramClient::new(config);

        assert!(client.needs_review(50));
        assert!(client.needs_review(69));
        assert!(!client.needs_review(70));
        assert!(!client.needs_review(90));

        assert!(!client.can_auto_approve(89));
        assert!(client.can_auto_approve(90));
        assert!(client.can_auto_approve(100));
    }
}
