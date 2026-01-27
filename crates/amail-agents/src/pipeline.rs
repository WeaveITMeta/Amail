// =============================================================================
// Task Processing Pipeline - Webhook → Queue → Agent → Response
// =============================================================================
// Table of Contents:
// 1. Pipeline struct
// 2. Email ingestion
// 3. Task routing
// 4. Response handling
// =============================================================================

use actix::prelude::*;
use amail_core::{
    agent::{AgentId, AgentRole},
    db::{Database, EmailRepository, TaskRepository},
    email::EmailMessage,
    queue::TaskQueue,
    task::{Task, TaskId, TaskType},
    AmailError, Result,
};
use chrono::Utc;
use amail_llm::LlmClient;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::{
    agents::receptionist::ReceptionistAgent,
    messages::{ProcessTask, RouteToRole},
    reasoning::{ReasoningEngine, ReasoningResult, ReasoningAction},
    registry::AgentRegistry,
};

/// Task processing pipeline
pub struct Pipeline {
    /// Database connection
    db: Option<Database>,
    /// Redis task queue
    queue: Option<TaskQueue>,
    /// Agent registry address
    registry: Addr<AgentRegistry>,
    /// LLM client
    llm: Arc<LlmClient>,
    /// Pipeline configuration
    config: PipelineConfig,
}

/// Pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Enable database persistence
    pub use_database: bool,
    /// Enable Redis queue
    pub use_queue: bool,
    /// Max concurrent tasks per agent
    pub max_concurrent_tasks: usize,
    /// Task timeout in seconds
    pub task_timeout_secs: u64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            use_database: true,
            use_queue: true,
            max_concurrent_tasks: 5,
            task_timeout_secs: 300,
        }
    }
}

impl Pipeline {
    /// Create a new pipeline
    pub fn new(registry: Addr<AgentRegistry>, llm: LlmClient) -> Self {
        Self {
            db: None,
            queue: None,
            registry,
            llm: Arc::new(llm),
            config: PipelineConfig::default(),
        }
    }

    /// Configure database
    pub fn with_database(mut self, db: Database) -> Self {
        self.db = Some(db);
        self
    }

    /// Configure queue
    pub fn with_queue(mut self, queue: TaskQueue) -> Self {
        self.queue = Some(queue);
        self
    }

    /// Configure pipeline
    pub fn with_config(mut self, config: PipelineConfig) -> Self {
        self.config = config;
        self
    }

    /// Ingest an email and create a task
    pub async fn ingest_email(&self, email: EmailMessage) -> Result<TaskId> {
        info!(
            "Ingesting email: {} from {}",
            email.metadata.subject, email.metadata.from.address
        );

        // Persist email to database
        if let Some(ref db) = self.db {
            let repo = EmailRepository::new(db.pool());
            repo.insert(&email).await?;
        }

        // Classify email using Receptionist logic
        let classification = ReceptionistAgent::classify_email(&email);
        debug!(
            "Email classified as {:?} -> {:?} (priority: {:?})",
            classification.category, classification.target_role, classification.priority
        );

        // Create task
        let task = ReceptionistAgent::create_task(email, &classification);
        let task_id = task.id.clone();

        // Persist task
        if let Some(ref db) = self.db {
            let repo = TaskRepository::new(db.pool());
            repo.insert(&task).await?;
        }

        // Queue task
        if let Some(ref queue) = self.queue {
            queue.push(&task).await?;
            info!("Task {} queued with priority {:?}", task_id, task.priority);
        } else {
            // Direct dispatch if no queue
            self.dispatch_task(task, &classification.target_role).await?;
        }

        Ok(task_id)
    }

    /// Dispatch a task to an agent
    async fn dispatch_task(&self, task: Task, target_role: &AgentRole) -> Result<()> {
        info!(
            "Dispatching task {} to {:?}",
            task.id, target_role
        );

        // Try to route to target role, fall back to Receptionist if not found
        let route_msg = RouteToRole {
            role: target_role.clone(),
            message: Box::new(ProcessTask(task.clone())),
        };

        match self.registry.send(route_msg).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(_)) | Err(_) => {
                // Fallback to Receptionist for unhandled roles
                warn!(
                    "No agent for role {:?}, falling back to Receptionist",
                    target_role
                );
                let fallback_msg = RouteToRole {
                    role: AgentRole::Receptionist,
                    message: Box::new(ProcessTask(task.clone())),
                };
                self.registry.send(fallback_msg).await.map_err(|e| {
                    AmailError::Internal(format!("Failed to route task: {}", e))
                })??;
                Ok(())
            }
        }
    }

    /// Process tasks from the queue (worker loop)
    pub async fn process_queue_loop(&self) -> Result<()> {
        let queue = self.queue.as_ref().ok_or_else(|| {
            AmailError::Queue("Queue not configured".to_string())
        })?;

        info!("Starting queue processing loop");

        loop {
            // Pop next task
            match queue.pop().await {
                Ok(Some(task)) => {
                    info!("Processing task {} from queue", task.id);

                    // Determine target role from task type
                    let target_role = self.determine_target_role(&task);

                    // Process the task
                    match self.process_task(&task, &target_role).await {
                        Ok(result) => {
                            info!(
                                "Task {} completed: {:?}",
                                task.id, result.action
                            );

                            // Mark complete in queue
                            queue.complete(&task.id).await?;

                            // Update database
                            if let Some(ref db) = self.db {
                                let repo = TaskRepository::new(db.pool());
                                repo.update_status(&task.id, amail_core::task::TaskStatus::Completed {
                                agent_id: AgentId::new(),
                                completed_at: Utc::now(),
                            }).await?;
                            }

                            // Handle result action
                            self.handle_result(&task, result).await?;
                        }
                        Err(e) => {
                            error!("Task {} failed: {}", task.id, e);

                            // Move to DLQ after max retries
                            queue.move_to_dlq(&task.id, &e.to_string()).await?;

                            // Update database
                            if let Some(ref db) = self.db {
                                let repo = TaskRepository::new(db.pool());
                                repo.update_status(&task.id, amail_core::task::TaskStatus::Failed {
                                agent_id: None,
                                error: e.to_string(),
                                failed_at: Utc::now(),
                            }).await?;
                            }
                        }
                    }
                }
                Ok(None) => {
                    // No tasks, wait before polling again
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
                Err(e) => {
                    error!("Queue error: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                }
            }
        }
    }

    /// Process a single task
    async fn process_task(&self, task: &Task, role: &AgentRole) -> Result<ReasoningResult> {
        // Create agent config for reasoning
        let config = amail_core::agent::AgentConfig {
            id: AgentId::new(),
            role: role.clone(),
            name: format!("{:?}Agent", role),
            capabilities: role.default_capabilities().into_iter().collect(),
            max_concurrent_tasks: self.config.max_concurrent_tasks,
            task_timeout_secs: self.config.task_timeout_secs,
            llm_model: "mistral:latest".to_string(),
            system_prompt: String::new(),
            requires_oversight: role.requires_oversight(),
            supervisor: None,
        };

        // Create reasoning engine
        let llm = LlmClient::with_url(&self.llm.model, &self.llm.base_url)
            .expect("Failed to create LLM client");
        let engine = ReasoningEngine::new(config, llm);

        // Process task
        engine.process_task(task).await
    }

    /// Determine target role from task
    fn determine_target_role(&self, task: &Task) -> AgentRole {
        use amail_core::task::TaskType;

        match task.task_type {
            TaskType::ProcessEmail => AgentRole::Receptionist,
            TaskType::ComposeReply => AgentRole::Clerk,
            TaskType::HumanReview => AgentRole::DraftChecker,
            TaskType::ToolCall => AgentRole::Clerk,
            TaskType::LegalReview => AgentRole::Lawyer,
            TaskType::FinancialProcessing => AgentRole::Accountant,
            TaskType::ComplianceCheck => AgentRole::ComplianceOfficer,
            TaskType::Research => AgentRole::ResearchAnalyst,
            TaskType::ContentGeneration => AgentRole::ContentCreator,
            TaskType::Translation => AgentRole::Translator,
            TaskType::Delegate => AgentRole::Executive,
            TaskType::Custom(_) => AgentRole::Clerk,
        }
    }

    /// Handle the result of task processing
    async fn handle_result(&self, task: &Task, result: ReasoningResult) -> Result<()> {
        match result.action {
            ReasoningAction::Respond(text) => {
                info!("Task {} generated response: {} chars", task.id, text.len());
                // TODO: Store response, potentially send email
            }
            ReasoningAction::Delegate(delegation) => {
                info!(
                    "Task {} delegated to {}",
                    task.id, delegation.target_role
                );

                // Parse target role
                let target_role = self.parse_role(&delegation.target_role);

                // Create delegation task
                let mut new_task = task.clone();
                new_task.id = amail_core::task::TaskId::new();
                new_task.description = delegation.task_description;
                new_task.task_type = amail_core::task::TaskType::Delegate;

                // Queue or dispatch
                if let Some(ref queue) = self.queue {
                    queue.push(&new_task).await?;
                } else {
                    self.dispatch_task(new_task, &target_role).await?;
                }
            }
            ReasoningAction::Escalate(reason) => {
                warn!("Task {} escalated: {}", task.id, reason);
                // TODO: Create human review entry in database
            }
            ReasoningAction::ToolExecuted(tool_call) => {
                debug!(
                    "Task {} executed tool: {}",
                    task.id, tool_call.name
                );
                // Tool already executed in reasoning engine
            }
        }

        Ok(())
    }

    /// Parse role string to AgentRole
    fn parse_role(&self, role_str: &str) -> AgentRole {
        match role_str.to_lowercase().as_str() {
            "receptionist" => AgentRole::Receptionist,
            "clerk" => AgentRole::Clerk,
            "executive" => AgentRole::Executive,
            "draft_checker" | "draftchecker" => AgentRole::DraftChecker,
            "lawyer" | "legal" => AgentRole::Lawyer,
            "accountant" | "finance" => AgentRole::Accountant,
            "hr" | "hr_manager" | "human_resources" => AgentRole::HumanResourcesManager,
            "it" | "it_support" => AgentRole::ITSupportTechnician,
            "customer_service" | "support" => AgentRole::CustomerServiceRepresentative,
            "sales" => AgentRole::SalesRepresentative,
            "marketing" => AgentRole::MarketingCoordinator,
            "project_manager" => AgentRole::ProjectManager,
            "research" | "research_analyst" => AgentRole::ResearchAnalyst,
            "compliance" => AgentRole::ComplianceOfficer,
            "security" => AgentRole::SecurityAnalyst,
            "data_analyst" => AgentRole::DataAnalyst,
            _ => AgentRole::Clerk, // Default fallback
        }
    }

    /// Get queue statistics
    pub async fn queue_stats(&self) -> Result<amail_core::queue::QueueStats> {
        let queue = self.queue.as_ref().ok_or_else(|| {
            AmailError::Queue("Queue not configured".to_string())
        })?;

        queue.stats().await
    }
}
