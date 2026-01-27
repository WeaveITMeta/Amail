//! Analytics Repository - PostgreSQL persistence for KPI tracking
//!
//! Table of Contents:
//! 1. Metrics Records - Database models
//! 2. AnalyticsRepository - CRUD operations
//! 3. Dashboard stats queries

use chrono::{DateTime, NaiveDate, Timelike, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{AmailError, Result};

// =============================================================================
// 1. Metrics Records
// =============================================================================

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DailyMetricsRecord {
    pub id: Uuid,
    pub date: NaiveDate,
    pub emails_received: i64,
    pub emails_sent: i64,
    pub emails_auto_replied: i64,
    pub emails_forwarded: i64,
    pub tasks_created: i64,
    pub tasks_completed: i64,
    pub tasks_failed: i64,
    pub avg_response_time_ms: Option<i64>,
    pub avg_processing_time_ms: Option<i64>,
    pub reviews_requested: i64,
    pub reviews_approved: i64,
    pub reviews_rejected: i64,
    pub llm_calls: i64,
    pub llm_tokens_used: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HourlyMetricsRecord {
    pub id: Uuid,
    pub hour: DateTime<Utc>,
    pub emails_received: i32,
    pub emails_sent: i32,
    pub tasks_completed: i32,
    pub avg_response_time_ms: Option<i32>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AgentMetricsRecord {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub date: NaiveDate,
    pub tasks_processed: i64,
    pub tasks_successful: i64,
    pub tasks_failed: i64,
    pub avg_processing_time_ms: Option<i64>,
    pub llm_calls: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct DashboardStats {
    pub active_agents: Option<i64>,
    pub total_agents: Option<i64>,
    pub pending_tasks: Option<i64>,
    pub active_tasks: Option<i64>,
    pub pending_reviews: Option<i64>,
    pub emails_today: Option<i64>,
    pub unread_emails: Option<i64>,
    pub tasks_week: Option<i64>,
    pub avg_response_time_ms: Option<f64>,
}

// =============================================================================
// 2. Analytics Repository
// =============================================================================

pub struct AnalyticsRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> AnalyticsRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    // -------------------------------------------------------------------------
    // Daily Metrics
    // -------------------------------------------------------------------------

    /// Get or create today's metrics record
    pub async fn get_or_create_today(&self) -> Result<DailyMetricsRecord> {
        let today = Utc::now().date_naive();
        
        // Try to get existing
        if let Some(record) = self.get_daily_metrics(today).await? {
            return Ok(record);
        }

        // Create new
        let record = sqlx::query_as::<_, DailyMetricsRecord>(
            r#"
            INSERT INTO daily_metrics (date)
            VALUES ($1)
            ON CONFLICT (date) DO UPDATE SET date = $1
            RETURNING id, date, emails_received, emails_sent, emails_auto_replied, emails_forwarded,
                      tasks_created, tasks_completed, tasks_failed, avg_response_time_ms,
                      avg_processing_time_ms, reviews_requested, reviews_approved, reviews_rejected,
                      llm_calls, llm_tokens_used, created_at
            "#,
        )
        .bind(today)
        .fetch_one(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(record)
    }

    /// Get daily metrics for a specific date
    pub async fn get_daily_metrics(&self, date: NaiveDate) -> Result<Option<DailyMetricsRecord>> {
        let record = sqlx::query_as::<_, DailyMetricsRecord>(
            r#"
            SELECT id, date, emails_received, emails_sent, emails_auto_replied, emails_forwarded,
                   tasks_created, tasks_completed, tasks_failed, avg_response_time_ms,
                   avg_processing_time_ms, reviews_requested, reviews_approved, reviews_rejected,
                   llm_calls, llm_tokens_used, created_at
            FROM daily_metrics WHERE date = $1
            "#,
        )
        .bind(date)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(record)
    }

    /// Get daily metrics for date range
    pub async fn get_daily_metrics_range(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<DailyMetricsRecord>> {
        let records = sqlx::query_as::<_, DailyMetricsRecord>(
            r#"
            SELECT id, date, emails_received, emails_sent, emails_auto_replied, emails_forwarded,
                   tasks_created, tasks_completed, tasks_failed, avg_response_time_ms,
                   avg_processing_time_ms, reviews_requested, reviews_approved, reviews_rejected,
                   llm_calls, llm_tokens_used, created_at
            FROM daily_metrics 
            WHERE date >= $1 AND date <= $2
            ORDER BY date ASC
            "#,
        )
        .bind(start_date)
        .bind(end_date)
        .fetch_all(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(records)
    }

    /// Increment email received counter
    pub async fn increment_emails_received(&self) -> Result<()> {
        let today = Utc::now().date_naive();
        sqlx::query(
            r#"
            INSERT INTO daily_metrics (date, emails_received)
            VALUES ($1, 1)
            ON CONFLICT (date) DO UPDATE SET emails_received = daily_metrics.emails_received + 1
            "#,
        )
        .bind(today)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Increment email sent counter
    pub async fn increment_emails_sent(&self, auto_replied: bool) -> Result<()> {
        let today = Utc::now().date_naive();
        if auto_replied {
            sqlx::query(
                r#"
                INSERT INTO daily_metrics (date, emails_sent, emails_auto_replied)
                VALUES ($1, 1, 1)
                ON CONFLICT (date) DO UPDATE SET 
                    emails_sent = daily_metrics.emails_sent + 1,
                    emails_auto_replied = daily_metrics.emails_auto_replied + 1
                "#,
            )
            .bind(today)
            .execute(self.pool)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO daily_metrics (date, emails_sent)
                VALUES ($1, 1)
                ON CONFLICT (date) DO UPDATE SET emails_sent = daily_metrics.emails_sent + 1
                "#,
            )
            .bind(today)
            .execute(self.pool)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;
        }

        Ok(())
    }

    /// Increment task counters
    pub async fn increment_task_created(&self) -> Result<()> {
        let today = Utc::now().date_naive();
        sqlx::query(
            r#"
            INSERT INTO daily_metrics (date, tasks_created)
            VALUES ($1, 1)
            ON CONFLICT (date) DO UPDATE SET tasks_created = daily_metrics.tasks_created + 1
            "#,
        )
        .bind(today)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Increment task completed counter
    pub async fn increment_task_completed(&self, response_time_ms: Option<i64>) -> Result<()> {
        let today = Utc::now().date_naive();
        sqlx::query(
            r#"
            INSERT INTO daily_metrics (date, tasks_completed, avg_response_time_ms)
            VALUES ($1, 1, $2)
            ON CONFLICT (date) DO UPDATE SET 
                tasks_completed = daily_metrics.tasks_completed + 1,
                avg_response_time_ms = COALESCE(
                    (daily_metrics.avg_response_time_ms * daily_metrics.tasks_completed + $2) / (daily_metrics.tasks_completed + 1),
                    $2
                )
            "#,
        )
        .bind(today)
        .bind(response_time_ms)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Increment task failed counter
    pub async fn increment_task_failed(&self) -> Result<()> {
        let today = Utc::now().date_naive();
        sqlx::query(
            r#"
            INSERT INTO daily_metrics (date, tasks_failed)
            VALUES ($1, 1)
            ON CONFLICT (date) DO UPDATE SET tasks_failed = daily_metrics.tasks_failed + 1
            "#,
        )
        .bind(today)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Increment review counters
    pub async fn increment_review_requested(&self) -> Result<()> {
        let today = Utc::now().date_naive();
        sqlx::query(
            r#"
            INSERT INTO daily_metrics (date, reviews_requested)
            VALUES ($1, 1)
            ON CONFLICT (date) DO UPDATE SET reviews_requested = daily_metrics.reviews_requested + 1
            "#,
        )
        .bind(today)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Increment review approved/rejected
    pub async fn increment_review_decision(&self, approved: bool) -> Result<()> {
        let today = Utc::now().date_naive();
        if approved {
            sqlx::query(
                r#"
                INSERT INTO daily_metrics (date, reviews_approved)
                VALUES ($1, 1)
                ON CONFLICT (date) DO UPDATE SET reviews_approved = daily_metrics.reviews_approved + 1
                "#,
            )
            .bind(today)
            .execute(self.pool)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO daily_metrics (date, reviews_rejected)
                VALUES ($1, 1)
                ON CONFLICT (date) DO UPDATE SET reviews_rejected = daily_metrics.reviews_rejected + 1
                "#,
            )
            .bind(today)
            .execute(self.pool)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;
        }

        Ok(())
    }

    /// Increment LLM usage
    pub async fn increment_llm_usage(&self, tokens: i64) -> Result<()> {
        let today = Utc::now().date_naive();
        sqlx::query(
            r#"
            INSERT INTO daily_metrics (date, llm_calls, llm_tokens_used)
            VALUES ($1, 1, $2)
            ON CONFLICT (date) DO UPDATE SET 
                llm_calls = daily_metrics.llm_calls + 1,
                llm_tokens_used = daily_metrics.llm_tokens_used + $2
            "#,
        )
        .bind(today)
        .bind(tokens)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Hourly Metrics
    // -------------------------------------------------------------------------

    /// Record hourly metrics
    pub async fn record_hourly_metrics(
        &self,
        emails_received: i32,
        emails_sent: i32,
        tasks_completed: i32,
        avg_response_time_ms: Option<i32>,
    ) -> Result<()> {
        let hour = Utc::now().date_naive().and_hms_opt(Utc::now().time().hour() as u32, 0, 0)
            .unwrap()
            .and_utc();

        sqlx::query(
            r#"
            INSERT INTO hourly_metrics (hour, emails_received, emails_sent, tasks_completed, avg_response_time_ms)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (hour) DO UPDATE SET 
                emails_received = hourly_metrics.emails_received + $2,
                emails_sent = hourly_metrics.emails_sent + $3,
                tasks_completed = hourly_metrics.tasks_completed + $4
            "#,
        )
        .bind(hour)
        .bind(emails_received)
        .bind(emails_sent)
        .bind(tasks_completed)
        .bind(avg_response_time_ms)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get hourly metrics for last N hours
    pub async fn get_hourly_metrics(&self, hours: i32) -> Result<Vec<HourlyMetricsRecord>> {
        let records = sqlx::query_as::<_, HourlyMetricsRecord>(
            r#"
            SELECT id, hour, emails_received, emails_sent, tasks_completed, avg_response_time_ms
            FROM hourly_metrics 
            WHERE hour >= NOW() - INTERVAL '1 hour' * $1
            ORDER BY hour ASC
            "#,
        )
        .bind(hours)
        .fetch_all(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(records)
    }

    // -------------------------------------------------------------------------
    // Agent Metrics
    // -------------------------------------------------------------------------

    /// Record agent task completion
    pub async fn record_agent_task(
        &self,
        agent_id: Uuid,
        successful: bool,
        processing_time_ms: i64,
        llm_calls: i64,
    ) -> Result<()> {
        let today = Utc::now().date_naive();
        
        if successful {
            sqlx::query(
                r#"
                INSERT INTO agent_metrics (agent_id, date, tasks_processed, tasks_successful, avg_processing_time_ms, llm_calls)
                VALUES ($1, $2, 1, 1, $3, $4)
                ON CONFLICT (agent_id, date) DO UPDATE SET 
                    tasks_processed = agent_metrics.tasks_processed + 1,
                    tasks_successful = agent_metrics.tasks_successful + 1,
                    avg_processing_time_ms = (agent_metrics.avg_processing_time_ms * agent_metrics.tasks_processed + $3) / (agent_metrics.tasks_processed + 1),
                    llm_calls = agent_metrics.llm_calls + $4
                "#,
            )
            .bind(agent_id)
            .bind(today)
            .bind(processing_time_ms)
            .bind(llm_calls)
            .execute(self.pool)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO agent_metrics (agent_id, date, tasks_processed, tasks_failed, llm_calls)
                VALUES ($1, $2, 1, 1, $3)
                ON CONFLICT (agent_id, date) DO UPDATE SET 
                    tasks_processed = agent_metrics.tasks_processed + 1,
                    tasks_failed = agent_metrics.tasks_failed + 1,
                    llm_calls = agent_metrics.llm_calls + $3
                "#,
            )
            .bind(agent_id)
            .bind(today)
            .bind(llm_calls)
            .execute(self.pool)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;
        }

        Ok(())
    }

    /// Get agent metrics for date range
    pub async fn get_agent_metrics(
        &self,
        agent_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<AgentMetricsRecord>> {
        let records = sqlx::query_as::<_, AgentMetricsRecord>(
            r#"
            SELECT id, agent_id, date, tasks_processed, tasks_successful, tasks_failed,
                   avg_processing_time_ms, llm_calls
            FROM agent_metrics 
            WHERE agent_id = $1 AND date >= $2 AND date <= $3
            ORDER BY date ASC
            "#,
        )
        .bind(agent_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(records)
    }

    // -------------------------------------------------------------------------
    // Dashboard Stats
    // -------------------------------------------------------------------------

    /// Get dashboard stats from view
    pub async fn get_dashboard_stats(&self) -> Result<DashboardStats> {
        let stats = sqlx::query_as::<_, DashboardStats>(
            "SELECT * FROM dashboard_stats_view"
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(stats)
    }

    /// Delete old metrics (for cleanup job)
    pub async fn delete_old_metrics(&self, days: i32) -> Result<i64> {
        let cutoff = Utc::now().date_naive() - chrono::Duration::days(days as i64);
        
        let result = sqlx::query("DELETE FROM daily_metrics WHERE date < $1")
            .bind(cutoff)
            .execute(self.pool)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;

        let hourly_result = sqlx::query(
            "DELETE FROM hourly_metrics WHERE hour < NOW() - INTERVAL '1 day' * $1"
        )
        .bind(days)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(result.rows_affected() as i64 + hourly_result.rows_affected() as i64)
    }
}
