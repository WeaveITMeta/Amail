//! Review Repository - PostgreSQL persistence for human review queue
//!
//! Table of Contents:
//! 1. Review Records - Database models
//! 2. ReviewRepository - CRUD operations

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{AmailError, Result};

// =============================================================================
// 1. Review Records
// =============================================================================

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ReviewRecord {
    pub id: Uuid,
    pub task_id: Uuid,
    pub agent_id: Option<Uuid>,
    pub email_id: Option<String>,
    pub reason: String,
    pub urgency: String,
    pub draft_response: Option<String>,
    pub status: String,
    pub reviewer_notes: Option<String>,
    pub reviewed_by: Option<String>,
    pub confidence_score: Option<f64>,
    pub telegram_message_id: Option<i64>,
    pub decision: Option<String>,
    pub edited_response: Option<String>,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

/// Review with email and agent info for queue view
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ReviewQueueItem {
    pub id: Uuid,
    pub task_id: Uuid,
    pub email_id: Option<String>,
    pub reason: String,
    pub urgency: String,
    pub status: String,
    pub confidence_score: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub from_address: Option<String>,
    pub from_name: Option<String>,
    pub subject: Option<String>,
    pub email_preview: Option<String>,
    pub draft_response: Option<String>,
    pub agent_role: Option<String>,
    pub agent_name: Option<String>,
}

/// Draft response record
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DraftResponseRecord {
    pub id: Uuid,
    pub email_id: String,
    pub agent_id: Uuid,
    pub agent_role: String,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub confidence_score: f64,
    pub reasoning: Option<String>,
    pub status: String,
    pub review_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
}

// =============================================================================
// 2. Review Repository
// =============================================================================

pub struct ReviewRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ReviewRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    // -------------------------------------------------------------------------
    // Human Reviews
    // -------------------------------------------------------------------------

    /// Create a new review request
    pub async fn create_review(&self, review: &NewReview) -> Result<Uuid> {
        let row: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO human_reviews (task_id, agent_id, email_id, reason, urgency, draft_response, confidence_score)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
        .bind(review.task_id)
        .bind(review.agent_id)
        .bind(&review.email_id)
        .bind(&review.reason)
        .bind(&review.urgency)
        .bind(&review.draft_response)
        .bind(review.confidence_score)
        .fetch_one(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(row.0)
    }

    /// Get review by ID
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<ReviewRecord>> {
        let record = sqlx::query_as::<_, ReviewRecord>(
            r#"
            SELECT id, task_id, agent_id, email_id, reason, urgency, draft_response, status,
                   reviewer_notes, reviewed_by, confidence_score, telegram_message_id,
                   decision, edited_response, created_at, reviewed_at
            FROM human_reviews WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(record)
    }

    /// Get pending reviews (queue)
    pub async fn get_pending_queue(&self, limit: i64) -> Result<Vec<ReviewQueueItem>> {
        let records = sqlx::query_as::<_, ReviewQueueItem>(
            r#"
            SELECT 
                hr.id, hr.task_id, hr.email_id, hr.reason, hr.urgency, hr.status,
                hr.confidence_score, hr.created_at,
                e.from_address, e.from_name, e.subject,
                LEFT(COALESCE(e.text_body, ''), 200) as email_preview,
                dr.body_text as draft_response,
                dr.agent_role,
                a.role as agent_name
            FROM human_reviews hr
            LEFT JOIN emails e ON hr.email_id = e.message_id
            LEFT JOIN draft_responses dr ON dr.review_id = hr.id
            LEFT JOIN agent_states a ON hr.agent_id = a.id
            WHERE hr.status = 'pending'
            ORDER BY 
                CASE hr.urgency 
                    WHEN 'critical' THEN 1 
                    WHEN 'high' THEN 2 
                    WHEN 'medium' THEN 3 
                    WHEN 'low' THEN 4 
                END,
                hr.created_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(records)
    }

    /// Update review with decision
    pub async fn update_decision(
        &self,
        id: Uuid,
        decision: &str,
        reviewer_notes: Option<&str>,
        reviewed_by: &str,
        edited_response: Option<&str>,
    ) -> Result<()> {
        let status = match decision {
            "approved" | "approve" => "approved",
            "rejected" | "reject" => "rejected",
            "edited" | "edit" => "edited",
            _ => "reviewed",
        };

        sqlx::query(
            r#"
            UPDATE human_reviews 
            SET status = $1, decision = $2, reviewer_notes = $3, reviewed_by = $4,
                edited_response = $5, reviewed_at = NOW()
            WHERE id = $6
            "#,
        )
        .bind(status)
        .bind(decision)
        .bind(reviewer_notes)
        .bind(reviewed_by)
        .bind(edited_response)
        .bind(id)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Update Telegram message ID
    pub async fn update_telegram_message_id(&self, id: Uuid, message_id: i64) -> Result<()> {
        sqlx::query(
            "UPDATE human_reviews SET telegram_message_id = $1 WHERE id = $2"
        )
        .bind(message_id)
        .bind(id)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Count pending reviews
    pub async fn count_pending(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM human_reviews WHERE status = 'pending'"
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(row.0)
    }

    /// Count reviews by urgency
    pub async fn count_by_urgency(&self, urgency: &str) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM human_reviews WHERE status = 'pending' AND urgency = $1"
        )
        .bind(urgency)
        .fetch_one(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(row.0)
    }

    // -------------------------------------------------------------------------
    // Draft Responses
    // -------------------------------------------------------------------------

    /// Create a draft response
    pub async fn create_draft(&self, draft: &NewDraftResponse) -> Result<Uuid> {
        let row: (Uuid,) = sqlx::query_as(
            r#"
            INSERT INTO draft_responses (email_id, agent_id, agent_role, subject, body_text, body_html, confidence_score, reasoning, review_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
            "#,
        )
        .bind(&draft.email_id)
        .bind(draft.agent_id)
        .bind(&draft.agent_role)
        .bind(&draft.subject)
        .bind(&draft.body_text)
        .bind(&draft.body_html)
        .bind(draft.confidence_score)
        .bind(&draft.reasoning)
        .bind(draft.review_id)
        .fetch_one(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(row.0)
    }

    /// Get draft by ID
    pub async fn get_draft_by_id(&self, id: Uuid) -> Result<Option<DraftResponseRecord>> {
        let record = sqlx::query_as::<_, DraftResponseRecord>(
            r#"
            SELECT id, email_id, agent_id, agent_role, subject, body_text, body_html,
                   confidence_score, reasoning, status, review_id, created_at, approved_at, sent_at
            FROM draft_responses WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(record)
    }

    /// Get pending draft for email
    pub async fn get_pending_draft_for_email(&self, email_id: &str) -> Result<Option<DraftResponseRecord>> {
        let record = sqlx::query_as::<_, DraftResponseRecord>(
            r#"
            SELECT id, email_id, agent_id, agent_role, subject, body_text, body_html,
                   confidence_score, reasoning, status, review_id, created_at, approved_at, sent_at
            FROM draft_responses WHERE email_id = $1 AND status = 'pending'
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(email_id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(record)
    }

    /// Update draft status
    pub async fn update_draft_status(&self, id: Uuid, status: &str) -> Result<()> {
        let approved_at: Option<DateTime<Utc>> = if status == "approved" { Some(Utc::now()) } else { None };
        let sent_at: Option<DateTime<Utc>> = if status == "sent" { Some(Utc::now()) } else { None };

        sqlx::query(
            r#"
            UPDATE draft_responses 
            SET status = $1, approved_at = COALESCE($2, approved_at), sent_at = COALESCE($3, sent_at)
            WHERE id = $4
            "#,
        )
        .bind(status)
        .bind(approved_at)
        .bind(sent_at)
        .bind(id)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Count pending drafts
    pub async fn count_pending_drafts(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM draft_responses WHERE status = 'pending'"
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(row.0)
    }
}

// =============================================================================
// 3. New structs for inserts
// =============================================================================

pub struct NewReview {
    pub task_id: Uuid,
    pub agent_id: Option<Uuid>,
    pub email_id: Option<String>,
    pub reason: String,
    pub urgency: String,
    pub draft_response: Option<String>,
    pub confidence_score: Option<f64>,
}

pub struct NewDraftResponse {
    pub email_id: String,
    pub agent_id: Uuid,
    pub agent_role: String,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub confidence_score: f64,
    pub reasoning: Option<String>,
    pub review_id: Option<Uuid>,
}
