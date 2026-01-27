//! Mail storage implementation using PostgreSQL
//!
//! Stores emails, mailboxes, and related metadata.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;
use tracing::{info, debug};

// =============================================================================
// Data Types
// =============================================================================

/// Stored email message
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StoredMessage {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub mailbox_id: Uuid,
    pub message_id: Option<String>,
    pub from_address: String,
    pub to_addresses: Vec<String>,
    pub subject: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub raw_size: i64,
    pub is_read: bool,
    pub is_starred: bool,
    pub is_draft: bool,
    pub is_sent: bool,
    pub is_trash: bool,
    pub is_spam: bool,
    pub labels: Vec<String>,
    pub received_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Mailbox (folder)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Mailbox {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub display_name: String,
    pub mailbox_type: String, // inbox, sent, drafts, trash, spam, custom
    pub parent_id: Option<Uuid>,
    pub unread_count: i64,
    pub total_count: i64,
    pub created_at: DateTime<Utc>,
}

/// Email address associated with a tenant
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailAddress {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub address: String,
    pub display_name: Option<String>,
    pub is_primary: bool,
    pub is_verified: bool,
    pub created_at: DateTime<Utc>,
}

// =============================================================================
// Mail Store
// =============================================================================

/// Mail storage manager
pub struct MailStore {
    pool: PgPool,
}

impl MailStore {
    /// Create a new mail store
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    /// Create from existing pool
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize database schema
    pub async fn init_schema(&self) -> Result<()> {
        sqlx::query(MAIL_SCHEMA)
            .execute(&self.pool)
            .await?;
        info!("Mail storage schema initialized");
        Ok(())
    }

    // =========================================================================
    // Message Operations
    // =========================================================================

    /// Store a new message
    pub async fn store_message(
        &self,
        tenant_id: Uuid,
        mailbox_id: Uuid,
        message_id: Option<&str>,
        from: &str,
        to: &[String],
        subject: &str,
        body_text: Option<&str>,
        body_html: Option<&str>,
        raw_size: i64,
        received_at: DateTime<Utc>,
    ) -> Result<StoredMessage> {
        let id = Uuid::new_v4();
        
        let message = sqlx::query_as::<_, StoredMessage>(
            r#"
            INSERT INTO mail_messages (
                id, tenant_id, mailbox_id, message_id, from_address, to_addresses,
                subject, body_text, body_html, raw_size, received_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(tenant_id)
        .bind(mailbox_id)
        .bind(message_id)
        .bind(from)
        .bind(to)
        .bind(subject)
        .bind(body_text)
        .bind(body_html)
        .bind(raw_size)
        .bind(received_at)
        .fetch_one(&self.pool)
        .await?;

        // Update mailbox counts
        self.update_mailbox_counts(mailbox_id).await?;

        debug!("Stored message {} in mailbox {}", id, mailbox_id);
        Ok(message)
    }

    /// Get message by ID
    pub async fn get_message(&self, id: Uuid) -> Result<Option<StoredMessage>> {
        let message = sqlx::query_as::<_, StoredMessage>(
            "SELECT * FROM mail_messages WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(message)
    }

    /// List messages in a mailbox
    pub async fn list_messages(
        &self,
        tenant_id: Uuid,
        mailbox_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<StoredMessage>> {
        let messages = sqlx::query_as::<_, StoredMessage>(
            r#"
            SELECT * FROM mail_messages 
            WHERE tenant_id = $1 AND mailbox_id = $2 AND is_trash = false
            ORDER BY received_at DESC
            LIMIT $3 OFFSET $4
            "#
        )
        .bind(tenant_id)
        .bind(mailbox_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(messages)
    }

    /// Search messages
    pub async fn search_messages(
        &self,
        tenant_id: Uuid,
        query: &str,
        limit: i64,
    ) -> Result<Vec<StoredMessage>> {
        let messages = sqlx::query_as::<_, StoredMessage>(
            r#"
            SELECT * FROM mail_messages 
            WHERE tenant_id = $1 
              AND (subject ILIKE $2 OR body_text ILIKE $2 OR from_address ILIKE $2)
              AND is_trash = false
            ORDER BY received_at DESC
            LIMIT $3
            "#
        )
        .bind(tenant_id)
        .bind(format!("%{}%", query))
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(messages)
    }

    /// Mark message as read
    pub async fn mark_read(&self, id: Uuid, is_read: bool) -> Result<()> {
        sqlx::query("UPDATE mail_messages SET is_read = $2 WHERE id = $1")
            .bind(id)
            .bind(is_read)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Move message to trash
    pub async fn trash_message(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE mail_messages SET is_trash = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Permanently delete message
    pub async fn delete_message(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM mail_messages WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // =========================================================================
    // Mailbox Operations
    // =========================================================================

    /// Create a mailbox
    pub async fn create_mailbox(
        &self,
        tenant_id: Uuid,
        name: &str,
        display_name: &str,
        mailbox_type: &str,
    ) -> Result<Mailbox> {
        let id = Uuid::new_v4();
        
        let mailbox = sqlx::query_as::<_, Mailbox>(
            r#"
            INSERT INTO mail_mailboxes (id, tenant_id, name, display_name, mailbox_type)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(tenant_id)
        .bind(name)
        .bind(display_name)
        .bind(mailbox_type)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(mailbox)
    }

    /// Get default mailboxes for a tenant (creates if not exist)
    pub async fn get_or_create_default_mailboxes(&self, tenant_id: Uuid) -> Result<Vec<Mailbox>> {
        let defaults = [
            ("inbox", "Inbox", "inbox"),
            ("sent", "Sent", "sent"),
            ("drafts", "Drafts", "drafts"),
            ("trash", "Trash", "trash"),
            ("spam", "Spam", "spam"),
        ];

        let mut mailboxes = Vec::new();
        
        for (name, display, mtype) in defaults {
            let existing = sqlx::query_as::<_, Mailbox>(
                "SELECT * FROM mail_mailboxes WHERE tenant_id = $1 AND name = $2"
            )
            .bind(tenant_id)
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

            let mailbox = match existing {
                Some(m) => m,
                None => self.create_mailbox(tenant_id, name, display, mtype).await?,
            };
            
            mailboxes.push(mailbox);
        }

        Ok(mailboxes)
    }

    /// List mailboxes for a tenant
    pub async fn list_mailboxes(&self, tenant_id: Uuid) -> Result<Vec<Mailbox>> {
        let mailboxes = sqlx::query_as::<_, Mailbox>(
            "SELECT * FROM mail_mailboxes WHERE tenant_id = $1 ORDER BY name"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(mailboxes)
    }

    /// Get inbox mailbox for a tenant
    pub async fn get_inbox(&self, tenant_id: Uuid) -> Result<Option<Mailbox>> {
        let mailbox = sqlx::query_as::<_, Mailbox>(
            "SELECT * FROM mail_mailboxes WHERE tenant_id = $1 AND mailbox_type = 'inbox'"
        )
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(mailbox)
    }

    /// Update mailbox counts
    async fn update_mailbox_counts(&self, mailbox_id: Uuid) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE mail_mailboxes SET
                total_count = (SELECT COUNT(*) FROM mail_messages WHERE mailbox_id = $1 AND is_trash = false),
                unread_count = (SELECT COUNT(*) FROM mail_messages WHERE mailbox_id = $1 AND is_read = false AND is_trash = false)
            WHERE id = $1
            "#
        )
        .bind(mailbox_id)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    // =========================================================================
    // Email Address Operations
    // =========================================================================

    /// Add an email address to a tenant
    pub async fn add_email_address(
        &self,
        tenant_id: Uuid,
        address: &str,
        display_name: Option<&str>,
        is_primary: bool,
    ) -> Result<EmailAddress> {
        let id = Uuid::new_v4();
        
        let email = sqlx::query_as::<_, EmailAddress>(
            r#"
            INSERT INTO mail_addresses (id, tenant_id, address, display_name, is_primary)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(id)
        .bind(tenant_id)
        .bind(address)
        .bind(display_name)
        .bind(is_primary)
        .fetch_one(&self.pool)
        .await?;
        
        Ok(email)
    }

    /// Get email addresses for a tenant
    pub async fn get_email_addresses(&self, tenant_id: Uuid) -> Result<Vec<EmailAddress>> {
        let addresses = sqlx::query_as::<_, EmailAddress>(
            "SELECT * FROM mail_addresses WHERE tenant_id = $1 ORDER BY is_primary DESC"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(addresses)
    }

    /// Find tenant by email address
    pub async fn find_tenant_by_email(&self, address: &str) -> Result<Option<Uuid>> {
        let result = sqlx::query_scalar::<_, Uuid>(
            "SELECT tenant_id FROM mail_addresses WHERE address = $1"
        )
        .bind(address)
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(result)
    }
}

// =============================================================================
// Database Schema
// =============================================================================

const MAIL_SCHEMA: &str = r#"
-- Mail messages table
CREATE TABLE IF NOT EXISTS mail_messages (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    mailbox_id UUID NOT NULL,
    message_id TEXT,
    from_address TEXT NOT NULL,
    to_addresses TEXT[] NOT NULL DEFAULT '{}',
    subject TEXT NOT NULL DEFAULT '',
    body_text TEXT,
    body_html TEXT,
    raw_size BIGINT NOT NULL DEFAULT 0,
    is_read BOOLEAN NOT NULL DEFAULT false,
    is_starred BOOLEAN NOT NULL DEFAULT false,
    is_draft BOOLEAN NOT NULL DEFAULT false,
    is_sent BOOLEAN NOT NULL DEFAULT false,
    is_trash BOOLEAN NOT NULL DEFAULT false,
    is_spam BOOLEAN NOT NULL DEFAULT false,
    labels TEXT[] NOT NULL DEFAULT '{}',
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mail_messages_tenant ON mail_messages(tenant_id);
CREATE INDEX IF NOT EXISTS idx_mail_messages_mailbox ON mail_messages(mailbox_id);
CREATE INDEX IF NOT EXISTS idx_mail_messages_received ON mail_messages(received_at DESC);
CREATE INDEX IF NOT EXISTS idx_mail_messages_from ON mail_messages(from_address);

-- Mailboxes table
CREATE TABLE IF NOT EXISTS mail_mailboxes (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    mailbox_type TEXT NOT NULL DEFAULT 'custom',
    parent_id UUID,
    unread_count BIGINT NOT NULL DEFAULT 0,
    total_count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, name)
);

CREATE INDEX IF NOT EXISTS idx_mail_mailboxes_tenant ON mail_mailboxes(tenant_id);

-- Email addresses table
CREATE TABLE IF NOT EXISTS mail_addresses (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    address TEXT NOT NULL UNIQUE,
    display_name TEXT,
    is_primary BOOLEAN NOT NULL DEFAULT false,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mail_addresses_tenant ON mail_addresses(tenant_id);
CREATE INDEX IF NOT EXISTS idx_mail_addresses_address ON mail_addresses(address);
"#;
