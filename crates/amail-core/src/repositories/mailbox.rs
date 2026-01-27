//! Mailbox Repository - PostgreSQL persistence for mailboxes
//!
//! Table of Contents:
//! 1. MailboxRecord - Database model
//! 2. MailboxRepository - CRUD operations

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{AmailError, Result};

// =============================================================================
// 1. Mailbox Record
// =============================================================================

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MailboxRecord {
    pub id: Uuid,
    pub domain_id: Uuid,
    pub local_part: String,
    pub display_name: Option<String>,
    pub mailbox_type: String,
    pub is_active: bool,
    pub quota_bytes: Option<i64>,
    pub used_bytes: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Mailbox with domain info joined
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MailboxWithDomain {
    pub id: Uuid,
    pub domain_id: Uuid,
    pub local_part: String,
    pub display_name: Option<String>,
    pub mailbox_type: String,
    pub is_active: bool,
    pub quota_bytes: Option<i64>,
    pub used_bytes: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub domain_name: String,
    pub unread_count: i64,
}

impl MailboxWithDomain {
    pub fn email(&self) -> String {
        format!("{}@{}", self.local_part, self.domain_name)
    }
}

// =============================================================================
// 2. Mailbox Repository
// =============================================================================

pub struct MailboxRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> MailboxRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new mailbox
    pub async fn create(
        &self,
        domain_id: Uuid,
        local_part: &str,
        display_name: Option<&str>,
        mailbox_type: &str,
    ) -> Result<MailboxRecord> {
        let record = sqlx::query_as::<_, MailboxRecord>(
            r#"
            INSERT INTO mailboxes (domain_id, local_part, display_name, mailbox_type)
            VALUES ($1, $2, $3, $4)
            RETURNING id, domain_id, local_part, display_name, mailbox_type, is_active,
                      quota_bytes, used_bytes, created_at, updated_at
            "#,
        )
        .bind(domain_id)
        .bind(local_part)
        .bind(display_name)
        .bind(mailbox_type)
        .fetch_one(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(record)
    }

    /// Get mailbox by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<MailboxRecord>> {
        let record = sqlx::query_as::<_, MailboxRecord>(
            r#"
            SELECT id, domain_id, local_part, display_name, mailbox_type, is_active,
                   quota_bytes, used_bytes, created_at, updated_at
            FROM mailboxes WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(record)
    }

    /// Get mailbox by email address
    pub async fn get_by_email(&self, local_part: &str, domain_name: &str) -> Result<Option<MailboxWithDomain>> {
        let record = sqlx::query_as::<_, MailboxWithDomain>(
            r#"
            SELECT m.id, m.domain_id, m.local_part, m.display_name, m.mailbox_type, m.is_active,
                   m.quota_bytes, m.used_bytes, m.created_at, m.updated_at,
                   d.name as domain_name,
                   COALESCE((SELECT COUNT(*) FROM emails e WHERE e.mailbox_id = m.id AND e.is_read = FALSE), 0) as unread_count
            FROM mailboxes m
            JOIN domains d ON m.domain_id = d.id
            WHERE m.local_part = $1 AND d.name = $2
            "#,
        )
        .bind(local_part)
        .bind(domain_name)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(record)
    }

    /// Get all mailboxes for a domain
    pub async fn get_by_domain(&self, domain_id: Uuid) -> Result<Vec<MailboxWithDomain>> {
        let records = sqlx::query_as::<_, MailboxWithDomain>(
            r#"
            SELECT m.id, m.domain_id, m.local_part, m.display_name, m.mailbox_type, m.is_active,
                   m.quota_bytes, m.used_bytes, m.created_at, m.updated_at,
                   d.name as domain_name,
                   COALESCE((SELECT COUNT(*) FROM emails e WHERE e.mailbox_id = m.id AND e.is_read = FALSE), 0) as unread_count
            FROM mailboxes m
            JOIN domains d ON m.domain_id = d.id
            WHERE m.domain_id = $1
            ORDER BY m.local_part
            "#,
        )
        .bind(domain_id)
        .fetch_all(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(records)
    }

    /// Get all mailboxes with domain info
    pub async fn get_all_with_domain(&self) -> Result<Vec<MailboxWithDomain>> {
        let records = sqlx::query_as::<_, MailboxWithDomain>(
            r#"
            SELECT m.id, m.domain_id, m.local_part, m.display_name, m.mailbox_type, m.is_active,
                   m.quota_bytes, m.used_bytes, m.created_at, m.updated_at,
                   d.name as domain_name,
                   COALESCE((SELECT COUNT(*) FROM emails e WHERE e.mailbox_id = m.id AND e.is_read = FALSE), 0) as unread_count
            FROM mailboxes m
            JOIN domains d ON m.domain_id = d.id
            WHERE m.is_active = TRUE
            ORDER BY d.name, m.local_part
            "#,
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(records)
    }

    /// Update mailbox
    pub async fn update(
        &self,
        id: Uuid,
        display_name: Option<&str>,
        is_active: bool,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE mailboxes 
            SET display_name = $1, is_active = $2
            WHERE id = $3
            "#,
        )
        .bind(display_name)
        .bind(is_active)
        .bind(id)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Update used bytes
    pub async fn update_used_bytes(&self, id: Uuid, bytes: i64) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE mailboxes SET used_bytes = used_bytes + $1 WHERE id = $2
            "#,
        )
        .bind(bytes)
        .bind(id)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete mailbox
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM mailboxes WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Count mailboxes
    pub async fn count(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM mailboxes WHERE is_active = TRUE")
            .fetch_one(self.pool)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(row.0)
    }

    /// Count mailboxes for domain
    pub async fn count_by_domain(&self, domain_id: Uuid) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM mailboxes WHERE domain_id = $1 AND is_active = TRUE"
        )
            .bind(domain_id)
            .fetch_one(self.pool)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(row.0)
    }
}
