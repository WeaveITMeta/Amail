//! Email Repository - PostgreSQL persistence for emails
//!
//! Table of Contents:
//! 1. EmailRecord - Database model
//! 2. EmailRepository - CRUD operations
//! 3. Inbox queries with filtering

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{AmailError, Result};

// =============================================================================
// 1. Email Record
// =============================================================================

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EmailRecord {
    pub message_id: String,
    pub tracking_id: Uuid,
    pub from_address: String,
    pub from_name: Option<String>,
    pub to_addresses: Vec<String>,
    pub cc_addresses: Vec<String>,
    pub subject: String,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub received_at: DateTime<Utc>,
    pub priority: String,
    pub mailbox_id: Option<Uuid>,
    pub folder: String,
    pub is_read: bool,
    pub is_starred: bool,
    pub is_archived: bool,
    pub is_spam: bool,
    pub is_trash: bool,
    pub labels: Vec<String>,
    pub thread_id: Option<String>,
}

/// Email with AI response info for inbox view
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InboxEmail {
    pub message_id: String,
    pub tracking_id: Uuid,
    pub from_address: String,
    pub from_name: Option<String>,
    pub to_addresses: Vec<String>,
    pub subject: String,
    pub preview: Option<String>,
    pub received_at: DateTime<Utc>,
    pub is_read: bool,
    pub is_starred: bool,
    pub folder: String,
    pub labels: Vec<String>,
    pub mailbox_email: Option<String>,
    pub mailbox_name: Option<String>,
    pub domain_name: Option<String>,
    pub ai_confidence: Option<f64>,
    pub ai_status: Option<String>,
    pub attachment_count: Option<i64>,
}

/// Email attachment record
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AttachmentRecord {
    pub id: Uuid,
    pub email_id: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_path: Option<String>,
}

// =============================================================================
// 2. Email Repository
// =============================================================================

pub struct EmailRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> EmailRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Insert a new email
    pub async fn insert(&self, email: &NewEmail) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO emails (
                message_id, tracking_id, from_address, from_name,
                to_addresses, cc_addresses, subject, text_body, html_body,
                received_at, priority, mailbox_id, folder, thread_id, in_reply_to, references_ids
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (message_id) DO NOTHING
            "#,
        )
        .bind(&email.message_id)
        .bind(email.tracking_id)
        .bind(&email.from_address)
        .bind(&email.from_name)
        .bind(&email.to_addresses)
        .bind(&email.cc_addresses)
        .bind(&email.subject)
        .bind(&email.text_body)
        .bind(&email.html_body)
        .bind(email.received_at)
        .bind(&email.priority)
        .bind(email.mailbox_id)
        .bind(&email.folder)
        .bind(&email.thread_id)
        .bind(&email.in_reply_to)
        .bind(&email.references_ids)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get email by message ID
    pub async fn get_by_id(&self, message_id: &str) -> Result<Option<EmailRecord>> {
        let record = sqlx::query_as::<_, EmailRecord>(
            r#"
            SELECT message_id, tracking_id, from_address, from_name,
                   to_addresses, cc_addresses, subject, text_body, html_body,
                   received_at, priority, mailbox_id, folder, is_read, is_starred,
                   is_archived, is_spam, is_trash, labels, thread_id
            FROM emails WHERE message_id = $1
            "#,
        )
        .bind(message_id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(record)
    }

    /// Get inbox emails with filtering
    pub async fn get_inbox(
        &self,
        mailbox_id: Option<Uuid>,
        folder: &str,
        search: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<InboxEmail>> {
        let search_pattern = search.map(|s| format!("%{}%", s.to_lowercase()));
        
        let records = sqlx::query_as::<_, InboxEmail>(
            r#"
            SELECT 
                e.message_id,
                e.tracking_id,
                e.from_address,
                e.from_name,
                e.to_addresses,
                e.subject,
                LEFT(COALESCE(e.text_body, ''), 200) as preview,
                e.received_at,
                e.is_read,
                e.is_starred,
                e.folder,
                e.labels,
                m.local_part || '@' || d.name as mailbox_email,
                m.display_name as mailbox_name,
                d.name as domain_name,
                dr.confidence_score as ai_confidence,
                dr.status as ai_status,
                (SELECT COUNT(*) FROM email_attachments ea WHERE ea.email_id = e.message_id) as attachment_count
            FROM emails e
            LEFT JOIN mailboxes m ON e.mailbox_id = m.id
            LEFT JOIN domains d ON m.domain_id = d.id
            LEFT JOIN draft_responses dr ON dr.email_id = e.message_id AND dr.status = 'pending'
            WHERE e.folder = $1
              AND ($2::uuid IS NULL OR e.mailbox_id = $2)
              AND e.is_trash = FALSE
              AND e.is_spam = FALSE
              AND ($3::text IS NULL OR (
                  LOWER(e.subject) LIKE $3 OR 
                  LOWER(e.from_address) LIKE $3 OR 
                  LOWER(COALESCE(e.text_body, '')) LIKE $3
              ))
            ORDER BY e.received_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(folder)
        .bind(mailbox_id)
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(records)
    }

    /// Get starred emails
    pub async fn get_starred(&self, limit: i64) -> Result<Vec<InboxEmail>> {
        let records = sqlx::query_as::<_, InboxEmail>(
            r#"
            SELECT 
                e.message_id, e.tracking_id, e.from_address, e.from_name, e.to_addresses,
                e.subject, LEFT(COALESCE(e.text_body, ''), 200) as preview, e.received_at,
                e.is_read, e.is_starred, e.folder, e.labels,
                m.local_part || '@' || d.name as mailbox_email,
                m.display_name as mailbox_name, d.name as domain_name,
                dr.confidence_score as ai_confidence, dr.status as ai_status,
                (SELECT COUNT(*) FROM email_attachments ea WHERE ea.email_id = e.message_id) as attachment_count
            FROM emails e
            LEFT JOIN mailboxes m ON e.mailbox_id = m.id
            LEFT JOIN domains d ON m.domain_id = d.id
            LEFT JOIN draft_responses dr ON dr.email_id = e.message_id AND dr.status = 'pending'
            WHERE e.is_starred = TRUE AND e.is_trash = FALSE
            ORDER BY e.received_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(records)
    }

    /// Mark email as read
    pub async fn mark_read(&self, message_id: &str, is_read: bool) -> Result<()> {
        sqlx::query("UPDATE emails SET is_read = $1 WHERE message_id = $2")
            .bind(is_read)
            .bind(message_id)
            .execute(self.pool)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Mark email as starred
    pub async fn mark_starred(&self, message_id: &str, is_starred: bool) -> Result<()> {
        sqlx::query("UPDATE emails SET is_starred = $1 WHERE message_id = $2")
            .bind(is_starred)
            .bind(message_id)
            .execute(self.pool)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Move email to folder
    pub async fn move_to_folder(&self, message_id: &str, folder: &str) -> Result<()> {
        let (is_trash, is_spam, is_archived) = match folder {
            "trash" => (true, false, false),
            "spam" => (false, true, false),
            "archive" => (false, false, true),
            _ => (false, false, false),
        };

        sqlx::query(
            r#"
            UPDATE emails 
            SET folder = $1, is_trash = $2, is_spam = $3, is_archived = $4 
            WHERE message_id = $5
            "#,
        )
        .bind(folder)
        .bind(is_trash)
        .bind(is_spam)
        .bind(is_archived)
        .bind(message_id)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Add label to email
    pub async fn add_label(&self, message_id: &str, label: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE emails 
            SET labels = array_append(labels, $1) 
            WHERE message_id = $2 AND NOT ($1 = ANY(labels))
            "#,
        )
        .bind(label)
        .bind(message_id)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Remove label from email
    pub async fn remove_label(&self, message_id: &str, label: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE emails 
            SET labels = array_remove(labels, $1) 
            WHERE message_id = $2
            "#,
        )
        .bind(label)
        .bind(message_id)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete email permanently
    pub async fn delete(&self, message_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM emails WHERE message_id = $1")
            .bind(message_id)
            .execute(self.pool)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Count unread emails
    pub async fn count_unread(&self, mailbox_id: Option<Uuid>) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM emails 
            WHERE is_read = FALSE AND is_trash = FALSE AND is_spam = FALSE
              AND ($1::uuid IS NULL OR mailbox_id = $1)
            "#,
        )
        .bind(mailbox_id)
        .fetch_one(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(row.0)
    }

    /// Count emails by folder
    pub async fn count_by_folder(&self, folder: &str) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM emails WHERE folder = $1"
        )
        .bind(folder)
        .fetch_one(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(row.0)
    }

    /// Count emails received today
    pub async fn count_today(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM emails WHERE received_at >= CURRENT_DATE"
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(row.0)
    }

    /// Get attachments for email
    pub async fn get_attachments(&self, email_id: &str) -> Result<Vec<AttachmentRecord>> {
        let records = sqlx::query_as::<_, AttachmentRecord>(
            r#"
            SELECT id, email_id, filename, content_type, size_bytes, storage_path
            FROM email_attachments WHERE email_id = $1
            "#,
        )
        .bind(email_id)
        .fetch_all(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(records)
    }

    /// Insert attachment
    pub async fn insert_attachment(&self, attachment: &NewAttachment) -> Result<Uuid> {
        let row: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO email_attachments (email_id, filename, content_type, size_bytes, storage_path, checksum)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(&attachment.email_id)
        .bind(&attachment.filename)
        .bind(&attachment.content_type)
        .bind(attachment.size_bytes)
        .bind(&attachment.storage_path)
        .bind(&attachment.checksum)
        .fetch_one(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(row.0)
    }

    /// Delete old emails (for cleanup job)
    pub async fn delete_older_than(&self, days: i32) -> Result<i64> {
        let result = sqlx::query(
            r#"
            DELETE FROM emails 
            WHERE received_at < NOW() - INTERVAL '1 day' * $1
              AND is_starred = FALSE
            "#,
        )
        .bind(days)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(result.rows_affected() as i64)
    }
}

// =============================================================================
// 3. New Email / Attachment structs for inserts
// =============================================================================

pub struct NewEmail {
    pub message_id: String,
    pub tracking_id: Uuid,
    pub from_address: String,
    pub from_name: Option<String>,
    pub to_addresses: Vec<String>,
    pub cc_addresses: Vec<String>,
    pub subject: String,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub received_at: DateTime<Utc>,
    pub priority: String,
    pub mailbox_id: Option<Uuid>,
    pub folder: String,
    pub thread_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub references_ids: Vec<String>,
}

pub struct NewAttachment {
    pub email_id: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_path: Option<String>,
    pub checksum: Option<String>,
}
