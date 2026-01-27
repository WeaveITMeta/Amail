// =============================================================================
// Email Types
// =============================================================================
// Table of Contents:
// 1. EmailAddress - Validated email address
// 2. EmailMetadata - Envelope information
// 3. Attachment - Email attachment with content
// 4. EmailMessage - Complete email representation
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Validated email address
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EmailAddress {
    /// Display name (optional)
    pub name: Option<String>,
    /// Email address
    pub address: String,
}

impl EmailAddress {
    /// Create a new email address
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            name: None,
            address: address.into(),
        }
    }

    /// Create with display name
    pub fn with_name(name: impl Into<String>, address: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            address: address.into(),
        }
    }

    /// Get the domain part of the email
    pub fn domain(&self) -> Option<&str> {
        self.address.split('@').nth(1)
    }

    /// Get the local part of the email
    pub fn local_part(&self) -> Option<&str> {
        self.address.split('@').next()
    }
}

impl std::fmt::Display for EmailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{} <{}>", name, self.address),
            None => write!(f, "{}", self.address),
        }
    }
}

/// Email envelope metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMetadata {
    /// Unique message ID
    pub message_id: String,
    /// Internal tracking ID
    pub tracking_id: Uuid,
    /// Sender address
    pub from: EmailAddress,
    /// Recipient addresses
    pub to: Vec<EmailAddress>,
    /// CC addresses
    pub cc: Vec<EmailAddress>,
    /// BCC addresses (if available)
    pub bcc: Vec<EmailAddress>,
    /// Reply-to address
    pub reply_to: Option<EmailAddress>,
    /// Email subject
    pub subject: String,
    /// Received timestamp
    pub received_at: DateTime<Utc>,
    /// Original sent timestamp
    pub sent_at: Option<DateTime<Utc>>,
    /// In-Reply-To header
    pub in_reply_to: Option<String>,
    /// References header (thread)
    pub references: Vec<String>,
    /// SPF result
    pub spf_result: Option<String>,
    /// DKIM result
    pub dkim_result: Option<String>,
    /// DMARC result
    pub dmarc_result: Option<String>,
    /// Priority/importance
    pub priority: EmailPriority,
    /// Custom headers
    pub custom_headers: std::collections::HashMap<String, String>,
}

/// Email priority level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum EmailPriority {
    Low,
    #[default]
    Normal,
    High,
    Urgent,
}

/// Email attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Attachment ID
    pub id: Uuid,
    /// Original filename
    pub filename: String,
    /// MIME type
    pub content_type: String,
    /// Size in bytes
    pub size: usize,
    /// Content (base64 encoded for serialization)
    pub content: Vec<u8>,
    /// Content-ID for inline attachments
    pub content_id: Option<String>,
    /// Is inline attachment
    pub is_inline: bool,
}

impl Attachment {
    /// Create a new attachment
    pub fn new(filename: impl Into<String>, content_type: impl Into<String>, content: Vec<u8>) -> Self {
        let content_len = content.len();
        Self {
            id: Uuid::new_v4(),
            filename: filename.into(),
            content_type: content_type.into(),
            size: content_len,
            content,
            content_id: None,
            is_inline: false,
        }
    }

    /// Check if attachment is an image
    pub fn is_image(&self) -> bool {
        self.content_type.starts_with("image/")
    }

    /// Check if attachment is a document
    pub fn is_document(&self) -> bool {
        matches!(
            self.content_type.as_str(),
            "application/pdf"
                | "application/msword"
                | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                | "application/vnd.ms-excel"
                | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                | "text/plain"
                | "text/csv"
        )
    }
}

/// Complete email message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    /// Email metadata
    pub metadata: EmailMetadata,
    /// Plain text body
    pub text_body: Option<String>,
    /// HTML body
    pub html_body: Option<String>,
    /// Attachments
    pub attachments: Vec<Attachment>,
    /// Raw email bytes (optional, for archival)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<Vec<u8>>,
}

impl EmailMessage {
    /// Get the best available body text
    pub fn body_text(&self) -> Option<&str> {
        self.text_body.as_deref().or(self.html_body.as_deref())
    }

    /// Check if email has attachments
    pub fn has_attachments(&self) -> bool {
        !self.attachments.is_empty()
    }

    /// Get non-inline attachments
    pub fn file_attachments(&self) -> impl Iterator<Item = &Attachment> {
        self.attachments.iter().filter(|a| !a.is_inline)
    }

    /// Check if this is a reply
    pub fn is_reply(&self) -> bool {
        self.metadata.in_reply_to.is_some() || !self.metadata.references.is_empty()
    }

    /// Get thread ID (first reference or message_id)
    pub fn thread_id(&self) -> &str {
        self.metadata
            .references
            .first()
            .map(|s| s.as_str())
            .unwrap_or(&self.metadata.message_id)
    }
}

impl Default for EmailMetadata {
    fn default() -> Self {
        Self {
            message_id: String::new(),
            tracking_id: Uuid::new_v4(),
            from: EmailAddress::new("unknown@unknown.com"),
            to: Vec::new(),
            cc: Vec::new(),
            bcc: Vec::new(),
            reply_to: None,
            subject: String::new(),
            received_at: Utc::now(),
            sent_at: None,
            in_reply_to: None,
            references: Vec::new(),
            spf_result: None,
            dkim_result: None,
            dmarc_result: None,
            priority: EmailPriority::default(),
            custom_headers: std::collections::HashMap::new(),
        }
    }
}
