//! Config Repository - PostgreSQL persistence for server and scheduler settings
//!
//! Table of Contents:
//! 1. Config Records - Database models
//! 2. ConfigRepository - CRUD operations

use chrono::{DateTime, NaiveTime, Utc};
use sqlx::PgPool;

use crate::{AmailError, Result};

// =============================================================================
// 1. Config Records
// =============================================================================

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SchedulerConfigRecord {
    pub id: i32,
    pub auto_start_enabled: bool,
    pub daily_start_time: NaiveTime,
    pub daily_summary_enabled: bool,
    pub process_overnight_emails: bool,
    pub auto_cleanup_enabled: bool,
    pub retention_days: i32,
    pub activate_agents_on_start: bool,
    pub telegram_daily_digest: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ServerConfigRecord {
    pub id: i32,
    pub smtp_port: i32,
    pub http_port: i32,
    pub ollama_url: String,
    pub llm_model: String,
    pub stalwart_url: String,
    pub stalwart_admin_token: Option<String>,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
    pub telegram_enabled: bool,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// 2. Config Repository
// =============================================================================

pub struct ConfigRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ConfigRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    // -------------------------------------------------------------------------
    // Scheduler Config
    // -------------------------------------------------------------------------

    /// Get scheduler config (singleton)
    pub async fn get_scheduler_config(&self) -> Result<SchedulerConfigRecord> {
        let record = sqlx::query_as::<_, SchedulerConfigRecord>(
            r#"
            SELECT id, auto_start_enabled, daily_start_time, daily_summary_enabled,
                   process_overnight_emails, auto_cleanup_enabled, retention_days,
                   activate_agents_on_start, telegram_daily_digest, updated_at
            FROM scheduler_config WHERE id = 1
            "#,
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(record)
    }

    /// Update scheduler config
    pub async fn update_scheduler_config(&self, config: &UpdateSchedulerConfig) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE scheduler_config SET
                auto_start_enabled = $1,
                daily_start_time = $2,
                daily_summary_enabled = $3,
                process_overnight_emails = $4,
                auto_cleanup_enabled = $5,
                retention_days = $6,
                activate_agents_on_start = $7,
                telegram_daily_digest = $8
            WHERE id = 1
            "#,
        )
        .bind(config.auto_start_enabled)
        .bind(config.daily_start_time)
        .bind(config.daily_summary_enabled)
        .bind(config.process_overnight_emails)
        .bind(config.auto_cleanup_enabled)
        .bind(config.retention_days)
        .bind(config.activate_agents_on_start)
        .bind(config.telegram_daily_digest)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Server Config
    // -------------------------------------------------------------------------

    /// Get server config (singleton)
    pub async fn get_server_config(&self) -> Result<ServerConfigRecord> {
        let record = sqlx::query_as::<_, ServerConfigRecord>(
            r#"
            SELECT id, smtp_port, http_port, ollama_url, llm_model, stalwart_url,
                   stalwart_admin_token, telegram_bot_token, telegram_chat_id,
                   telegram_enabled, updated_at
            FROM server_config WHERE id = 1
            "#,
        )
        .fetch_one(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(record)
    }

    /// Update server config
    pub async fn update_server_config(&self, config: &UpdateServerConfig) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE server_config SET
                smtp_port = $1,
                http_port = $2,
                ollama_url = $3,
                llm_model = $4,
                stalwart_url = $5,
                stalwart_admin_token = $6,
                telegram_bot_token = $7,
                telegram_chat_id = $8,
                telegram_enabled = $9
            WHERE id = 1
            "#,
        )
        .bind(config.smtp_port)
        .bind(config.http_port)
        .bind(&config.ollama_url)
        .bind(&config.llm_model)
        .bind(&config.stalwart_url)
        .bind(&config.stalwart_admin_token)
        .bind(&config.telegram_bot_token)
        .bind(&config.telegram_chat_id)
        .bind(config.telegram_enabled)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Update Stalwart credentials
    pub async fn update_stalwart_credentials(&self, url: &str, token: Option<&str>) -> Result<()> {
        sqlx::query(
            "UPDATE server_config SET stalwart_url = $1, stalwart_admin_token = $2 WHERE id = 1"
        )
        .bind(url)
        .bind(token)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Update Telegram credentials
    pub async fn update_telegram_credentials(
        &self,
        bot_token: Option<&str>,
        chat_id: Option<&str>,
        enabled: bool,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE server_config SET 
                telegram_bot_token = $1, telegram_chat_id = $2, telegram_enabled = $3 
            WHERE id = 1
            "#,
        )
        .bind(bot_token)
        .bind(chat_id)
        .bind(enabled)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Update LLM settings
    pub async fn update_llm_settings(&self, ollama_url: &str, model: &str) -> Result<()> {
        sqlx::query(
            "UPDATE server_config SET ollama_url = $1, llm_model = $2 WHERE id = 1"
        )
        .bind(ollama_url)
        .bind(model)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }
}

// =============================================================================
// 3. Update structs
// =============================================================================

pub struct UpdateSchedulerConfig {
    pub auto_start_enabled: bool,
    pub daily_start_time: NaiveTime,
    pub daily_summary_enabled: bool,
    pub process_overnight_emails: bool,
    pub auto_cleanup_enabled: bool,
    pub retention_days: i32,
    pub activate_agents_on_start: bool,
    pub telegram_daily_digest: bool,
}

pub struct UpdateServerConfig {
    pub smtp_port: i32,
    pub http_port: i32,
    pub ollama_url: String,
    pub llm_model: String,
    pub stalwart_url: String,
    pub stalwart_admin_token: Option<String>,
    pub telegram_bot_token: Option<String>,
    pub telegram_chat_id: Option<String>,
    pub telegram_enabled: bool,
}
