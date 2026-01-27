//! Domain Repository - PostgreSQL persistence for domains
//!
//! Table of Contents:
//! 1. DomainRecord - Database model
//! 2. DomainRepository - CRUD operations

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{AmailError, Result};

// =============================================================================
// 1. Domain Record
// =============================================================================

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DomainRecord {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub mx_verified: bool,
    pub spf_verified: bool,
    pub dkim_verified: bool,
    pub dmarc_verified: bool,
    pub verification_token: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

impl DomainRecord {
    pub fn is_fully_verified(&self) -> bool {
        self.mx_verified && self.spf_verified && self.dkim_verified && self.dmarc_verified
    }
}

// =============================================================================
// 2. Domain Repository
// =============================================================================

pub struct DomainRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> DomainRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new domain
    pub async fn create(&self, name: &str) -> Result<DomainRecord> {
        let verification_token = format!("amail-verify-{}", Uuid::new_v4());
        
        let record = sqlx::query_as::<_, DomainRecord>(
            r#"
            INSERT INTO domains (name, verification_token)
            VALUES ($1, $2)
            RETURNING id, name, status, mx_verified, spf_verified, dkim_verified, dmarc_verified,
                      verification_token, created_at, updated_at, verified_at
            "#,
        )
        .bind(name)
        .bind(&verification_token)
        .fetch_one(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(record)
    }

    /// Get domain by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<DomainRecord>> {
        let record = sqlx::query_as::<_, DomainRecord>(
            r#"
            SELECT id, name, status, mx_verified, spf_verified, dkim_verified, dmarc_verified,
                   verification_token, created_at, updated_at, verified_at
            FROM domains WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(record)
    }

    /// Get domain by name
    pub async fn get_by_name(&self, name: &str) -> Result<Option<DomainRecord>> {
        let record = sqlx::query_as::<_, DomainRecord>(
            r#"
            SELECT id, name, status, mx_verified, spf_verified, dkim_verified, dmarc_verified,
                   verification_token, created_at, updated_at, verified_at
            FROM domains WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(record)
    }

    /// Get all domains
    pub async fn get_all(&self) -> Result<Vec<DomainRecord>> {
        let records = sqlx::query_as::<_, DomainRecord>(
            r#"
            SELECT id, name, status, mx_verified, spf_verified, dkim_verified, dmarc_verified,
                   verification_token, created_at, updated_at, verified_at
            FROM domains
            ORDER BY name
            "#,
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(records)
    }

    /// Update domain verification status
    pub async fn update_verification(
        &self,
        id: Uuid,
        mx: bool,
        spf: bool,
        dkim: bool,
        dmarc: bool,
    ) -> Result<()> {
        let all_verified = mx && spf && dkim && dmarc;
        let status = if all_verified { "verified" } else { "pending" };
        let verified_at: Option<DateTime<Utc>> = if all_verified { Some(Utc::now()) } else { None };

        sqlx::query(
            r#"
            UPDATE domains 
            SET mx_verified = $1, spf_verified = $2, dkim_verified = $3, dmarc_verified = $4,
                status = $5, verified_at = COALESCE($6, verified_at)
            WHERE id = $7
            "#,
        )
        .bind(mx)
        .bind(spf)
        .bind(dkim)
        .bind(dmarc)
        .bind(status)
        .bind(verified_at)
        .bind(id)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Delete domain
    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM domains WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Count domains
    pub async fn count(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM domains")
            .fetch_one(self.pool)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(row.0)
    }

    /// Count verified domains
    pub async fn count_verified(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM domains WHERE status = 'verified'"
        )
            .fetch_one(self.pool)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(row.0)
    }
}
