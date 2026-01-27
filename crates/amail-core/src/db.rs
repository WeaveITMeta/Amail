// =============================================================================
// Database Layer - PostgreSQL Persistence with SQLx
// =============================================================================
// Table of Contents:
// 1. Database pool and connection
// 2. Task repository
// 3. Email repository
// 4. Agent state repository
// =============================================================================

use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

use crate::{
    agent::{AgentId, AgentRole, AgentStatus},
    email::{EmailMessage, EmailPriority},
    task::{Task, TaskId, TaskStatus, TaskType},
    AmailError, Result,
};

/// Database connection pool
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Run database migrations
    pub async fn migrate(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| AmailError::Database(format!("Migration failed: {}", e)))?;

        Ok(())
    }

    /// Get the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

// =============================================================================
// Task Repository
// =============================================================================

/// Repository for task persistence
pub struct TaskRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> TaskRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Insert a new task
    pub async fn insert(&self, task: &Task) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO tasks (
                id, description, task_type, status, priority,
                assigned_agent, email_id, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(task.id.0)
        .bind(&task.description)
        .bind(task.task_type.as_str())
        .bind(task.status.as_str())
        .bind(task.priority.as_str())
        .bind::<Option<uuid::Uuid>>(None) // assigned_agent - set later
        .bind(task.context.source_email.as_ref().map(|e| &e.metadata.message_id))
        .bind(task.created_at)
        .bind(task.updated_at)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Update task status
    pub async fn update_status(&self, task_id: &TaskId, status: TaskStatus) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE tasks SET status = $1, updated_at = $2 WHERE id = $3
            "#,
        )
        .bind(status.as_str())
        .bind(Utc::now())
        .bind(task_id.0)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Assign task to agent
    pub async fn assign(&self, task_id: &TaskId, agent_id: &AgentId) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE tasks SET assigned_agent = $1, status = 'in_progress', updated_at = $2 WHERE id = $3
            "#,
        )
        .bind(agent_id.0)
        .bind(Utc::now())
        .bind(task_id.0)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get pending tasks
    pub async fn get_pending(&self, limit: i64) -> Result<Vec<TaskRecord>> {
        let rows = sqlx::query_as::<_, TaskRecord>(
            r#"
            SELECT id, description, task_type, status, priority, assigned_agent, email_id, created_at, updated_at
            FROM tasks
            WHERE status = 'pending'
            ORDER BY 
                CASE priority 
                    WHEN 'urgent' THEN 1 
                    WHEN 'high' THEN 2 
                    WHEN 'normal' THEN 3 
                    WHEN 'low' THEN 4 
                END,
                created_at ASC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(rows)
    }

    /// Get task by ID
    pub async fn get_by_id(&self, task_id: &TaskId) -> Result<Option<TaskRecord>> {
        let row = sqlx::query_as::<_, TaskRecord>(
            r#"
            SELECT id, description, task_type, status, priority, assigned_agent, email_id, created_at, updated_at
            FROM tasks WHERE id = $1
            "#,
        )
        .bind(task_id.0)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(row)
    }
}

/// Task database record
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TaskRecord {
    pub id: Uuid,
    pub description: String,
    pub task_type: String,
    pub status: String,
    pub priority: String,
    pub assigned_agent: Option<Uuid>,
    pub email_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// =============================================================================
// Email Repository
// =============================================================================

/// Repository for email persistence
pub struct EmailRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> EmailRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Insert a new email
    pub async fn insert(&self, email: &EmailMessage) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO emails (
                message_id, tracking_id, from_address, from_name,
                to_addresses, cc_addresses, subject, text_body, html_body,
                received_at, priority
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (message_id) DO NOTHING
            "#,
        )
        .bind(&email.metadata.message_id)
        .bind(email.metadata.tracking_id)
        .bind(&email.metadata.from.address)
        .bind(&email.metadata.from.name)
        .bind(
            email
                .metadata
                .to
                .iter()
                .map(|a| a.address.as_str())
                .collect::<Vec<_>>(),
        )
        .bind(
            email
                .metadata
                .cc
                .iter()
                .map(|a| a.address.as_str())
                .collect::<Vec<_>>(),
        )
        .bind(&email.metadata.subject)
        .bind(&email.text_body)
        .bind(&email.html_body)
        .bind(email.metadata.received_at)
        .bind(email.metadata.priority.as_str())
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get email by message ID
    pub async fn get_by_message_id(&self, message_id: &str) -> Result<Option<EmailRecord>> {
        let row = sqlx::query_as::<_, EmailRecord>(
            r#"
            SELECT message_id, tracking_id, from_address, from_name,
                   to_addresses, cc_addresses, subject, text_body, html_body,
                   received_at, priority
            FROM emails WHERE message_id = $1
            "#,
        )
        .bind(message_id)
        .fetch_optional(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(row)
    }
}

/// Email database record
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
}

// =============================================================================
// Agent State Repository
// =============================================================================

/// Repository for agent state persistence
pub struct AgentStateRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> AgentStateRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Upsert agent state
    pub async fn upsert(&self, agent_id: &AgentId, role: &AgentRole, status: &AgentStatus) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO agent_states (id, role, status, last_heartbeat, tasks_processed)
            VALUES ($1, $2, $3, $4, 0)
            ON CONFLICT (id) DO UPDATE SET
                status = $3,
                last_heartbeat = $4
            "#,
        )
        .bind(agent_id.0)
        .bind(role.as_str())
        .bind(status.as_str())
        .bind(Utc::now())
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Increment tasks processed
    pub async fn increment_tasks(&self, agent_id: &AgentId) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE agent_states SET tasks_processed = tasks_processed + 1, last_heartbeat = $1 WHERE id = $2
            "#,
        )
        .bind(Utc::now())
        .bind(agent_id.0)
        .execute(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(())
    }

    /// Get all agent states
    pub async fn get_all(&self) -> Result<Vec<AgentStateRecord>> {
        let rows = sqlx::query_as::<_, AgentStateRecord>(
            r#"
            SELECT id, role, status, last_heartbeat, tasks_processed
            FROM agent_states
            ORDER BY role
            "#,
        )
        .fetch_all(self.pool)
        .await
        .map_err(|e| AmailError::Database(e.to_string()))?;

        Ok(rows)
    }
}

/// Agent state database record
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AgentStateRecord {
    pub id: Uuid,
    pub role: String,
    pub status: String,
    pub last_heartbeat: DateTime<Utc>,
    pub tasks_processed: i64,
}

// =============================================================================
// Helper trait implementations
// =============================================================================

impl TaskType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskType::ProcessEmail => "process_email",
            TaskType::ComposeReply => "compose_reply",
            TaskType::Delegate => "delegate",
            TaskType::ToolCall => "tool_call",
            TaskType::HumanReview => "human_review",
            TaskType::ComplianceCheck => "compliance_check",
            TaskType::LegalReview => "legal_review",
            TaskType::FinancialProcessing => "financial_processing",
            TaskType::Research => "research",
            TaskType::ContentGeneration => "content_generation",
            TaskType::Translation => "translation",
            TaskType::Custom(_) => "custom",
        }
    }
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Queued => "queued",
            TaskStatus::Assigned { .. } => "assigned",
            TaskStatus::InProgress { .. } => "in_progress",
            TaskStatus::WaitingForTool { .. } => "waiting_for_tool",
            TaskStatus::WaitingForHuman { .. } => "waiting_for_human",
            TaskStatus::Delegated { .. } => "delegated",
            TaskStatus::Completed { .. } => "completed",
            TaskStatus::Failed { .. } => "failed",
            TaskStatus::Cancelled { .. } => "cancelled",
        }
    }
}


impl AgentRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentRole::Receptionist => "receptionist",
            AgentRole::Clerk => "clerk",
            AgentRole::Executive => "executive",
            AgentRole::DraftChecker => "draft_checker",
            AgentRole::Lawyer => "lawyer",
            AgentRole::Accountant => "accountant",
            AgentRole::HumanResourcesManager => "hr_manager",
            AgentRole::ITSupportTechnician => "it_support",
            AgentRole::MarketingCoordinator => "marketing",
            AgentRole::SalesRepresentative => "sales",
            AgentRole::CustomerServiceRepresentative => "customer_service",
            AgentRole::ProjectManager => "project_manager",
            AgentRole::ResearchAnalyst => "research_analyst",
            AgentRole::ContentCreator => "content_creator",
            AgentRole::Translator => "translator",
            AgentRole::DataAnalyst => "data_analyst",
            AgentRole::ComplianceOfficer => "compliance",
            AgentRole::SecurityAnalyst => "security",
            AgentRole::Archivist => "archivist",
            AgentRole::Trainer => "trainer",
            AgentRole::StrategicPlanner => "strategic_planner",
            AgentRole::Negotiator => "negotiator",
            AgentRole::InternalAuditor => "internal_auditor",
            AgentRole::ProcurementSpecialist => "procurement",
            AgentRole::LogisticsCoordinator => "logistics",
            AgentRole::QualityAssuranceTester => "qa_tester",
            AgentRole::RiskManager => "risk_manager",
            AgentRole::PublicRelationsOfficer => "pr_officer",
            AgentRole::EventCoordinator => "event_coordinator",
            AgentRole::InnovationResearcher => "innovation",
            AgentRole::FacilitiesManager => "facilities",
            AgentRole::EthicsReviewer => "ethics_reviewer",
            AgentRole::Custom(_) => "custom",
        }
    }
}

impl AgentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentStatus::Starting => "starting",
            AgentStatus::Ready => "ready",
            AgentStatus::Busy => "busy",
            AgentStatus::Paused => "paused",
            AgentStatus::ShuttingDown => "shutting_down",
            AgentStatus::Stopped => "stopped",
            AgentStatus::Error => "error",
        }
    }
}

impl EmailPriority {
    pub fn as_str(&self) -> &'static str {
        match self {
            EmailPriority::Low => "low",
            EmailPriority::Normal => "normal",
            EmailPriority::High => "high",
            EmailPriority::Urgent => "urgent",
        }
    }
}
