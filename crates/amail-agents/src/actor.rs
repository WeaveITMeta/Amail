// =============================================================================
// Agent Actor - Actix Actor Implementation
// =============================================================================
// Table of Contents:
// 1. AgentActor struct - Core actor state
// 2. Actor trait implementation
// 3. Message handlers (ProcessTask, A2AMessageWrapper, etc.)
// 4. Internal methods
// =============================================================================

use actix::prelude::*;
use amail_core::{
    a2a::{A2AEnvelope, A2AMessage, Delegation, TaskResult, TokenUsage},
    agent::{AgentCapability, AgentConfig, AgentId, AgentStatus},
    task::{Task, TaskId, TaskStatus},
    AmailError,
};
use amail_llm::LlmClient;
use chrono::Utc;
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::messages::{
    A2AMessageWrapper, AgentStatusResponse, ProcessTask, StatusQuery, SupervisionMessage,
};

/// Agent actor - wraps an agent implementation in an Actix actor
pub struct AgentActor {
    /// Agent configuration
    config: AgentConfig,
    /// Current status
    status: AgentStatus,
    /// LLM client for inference
    llm_client: Option<LlmClient>,
    /// Active tasks
    active_tasks: HashMap<TaskId, Task>,
    /// Completed task count
    completed_tasks: u64,
    /// Failed task count
    failed_tasks: u64,
    /// Pending delegations
    pending_delegations: HashMap<Uuid, Delegation>,
    /// System prompt for this agent
    system_prompt: String,
}

impl AgentActor {
    /// Create a new agent actor
    pub fn new(config: AgentConfig) -> Self {
        let system_prompt = if config.system_prompt.is_empty() {
            Self::default_system_prompt(&config)
        } else {
            config.system_prompt.clone()
        };

        Self {
            config,
            status: AgentStatus::Starting,
            llm_client: None,
            active_tasks: HashMap::new(),
            completed_tasks: 0,
            failed_tasks: 0,
            pending_delegations: HashMap::new(),
            system_prompt,
        }
    }

    /// Generate default system prompt based on role
    fn default_system_prompt(config: &AgentConfig) -> String {
        format!(
            r#"You are {}, a {} agent in the Amail AI organization.

Your role: {}
Your capabilities: {:?}

Guidelines:
1. Process tasks efficiently and accurately
2. Delegate tasks outside your expertise to appropriate agents
3. Always maintain professional communication
4. Flag uncertain decisions for human review when appropriate
5. Log all significant actions for audit purposes

When responding, be concise and action-oriented. If you need to use tools, specify them clearly.
If a task requires capabilities you don't have, delegate to an appropriate agent."#,
            config.name,
            config.role.display_name(),
            config.role.display_name(),
            config.capabilities
        )
    }

    /// Initialize the LLM client
    async fn initialize_llm(&mut self) -> Result<(), AmailError> {
        let client = LlmClient::new(&self.config.llm_model)
            .map_err(|e| AmailError::LlmInference(e.to_string()))?;
        self.llm_client = Some(client);
        Ok(())
    }

    /// Process a task using LLM
    async fn process_with_llm(&mut self, task: &Task) -> Result<TaskResult, AmailError> {
        let client = self
            .llm_client
            .as_ref()
            .ok_or_else(|| AmailError::LlmInference("LLM client not initialized".to_string()))?;

        // Build context from task
        let context = self.build_task_context(task);

        // Call LLM
        let response = client
            .chat(&self.system_prompt, &context)
            .await
            .map_err(|e| AmailError::LlmInference(e.to_string()))?;

        // Build result
        Ok(TaskResult {
            task_id: task.id,
            delegation_id: None,
            agent_id: self.config.id,
            success: true,
            result: serde_json::json!({
                "response": response.content,
                "model": self.config.llm_model,
            }),
            artifacts: Vec::new(),
            completed_at: Utc::now(),
            duration_ms: response.duration_ms,
            token_usage: Some(TokenUsage {
                prompt_tokens: response.prompt_tokens,
                completion_tokens: response.completion_tokens,
                total_tokens: response.total_tokens,
            }),
        })
    }

    /// Build context string from task
    fn build_task_context(&self, task: &Task) -> String {
        let mut context = format!("Task: {}\n", task.description);

        if let Some(email) = &task.context.source_email {
            context.push_str(&format!(
                "\nEmail from: {}\nSubject: {}\nBody:\n{}\n",
                email.metadata.from,
                email.metadata.subject,
                email.body_text().unwrap_or("[No body]")
            ));
        }

        // Add conversation history
        for msg in &task.context.conversation_history {
            context.push_str(&format!("\n[{}]: {}", msg.role, msg.content));
        }

        context
    }

    /// Handle delegation request
    fn handle_delegation(&mut self, delegation: Delegation) -> bool {
        // Check if we have the required capabilities
        let has_capabilities = delegation
            .required_capabilities
            .iter()
            .all(|cap| self.config.capabilities.contains(cap));

        if has_capabilities && self.active_tasks.len() < self.config.max_concurrent_tasks {
            // Accept delegation
            self.pending_delegations.insert(delegation.id, delegation);
            true
        } else {
            false
        }
    }
}

impl Actor for AgentActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        info!(
            "Agent {} ({}) starting",
            self.config.name,
            self.config.id
        );

        // Initialize LLM client asynchronously
        let config_model = self.config.llm_model.clone();
        ctx.spawn(
            async move {
                LlmClient::new(&config_model)
            }
            .into_actor(self)
            .map(|result, actor, _ctx| {
                match result {
                    Ok(client) => {
                        actor.llm_client = Some(client);
                        actor.status = AgentStatus::Ready;
                        info!("Agent {} ready", actor.config.name);
                    }
                    Err(e) => {
                        error!("Failed to initialize LLM for agent {}: {}", actor.config.name, e);
                        actor.status = AgentStatus::Error;
                    }
                }
            }),
        );
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        info!(
            "Agent {} ({}) stopped",
            self.config.name,
            self.config.id
        );
        self.status = AgentStatus::Stopped;
    }
}

/// Handle task processing
impl Handler<ProcessTask> for AgentActor {
    type Result = ResponseActFuture<Self, Result<TaskResult, AmailError>>;

    fn handle(&mut self, msg: ProcessTask, _ctx: &mut Self::Context) -> Self::Result {
        let task = msg.0;
        let task_id = task.id;

        info!(
            "Agent {} processing task {}",
            self.config.name, task_id
        );

        // Check capacity
        if self.active_tasks.len() >= self.config.max_concurrent_tasks {
            return Box::pin(async move {
                Err(AmailError::AgentUnavailable(
                    "Agent at capacity".to_string(),
                ))
            }.into_actor(self));
        }

        // Track active task
        self.active_tasks.insert(task_id, task.clone());
        self.status = AgentStatus::Busy;

        // Process task with LLM
        let llm_client = self.llm_client.clone();
        let system_prompt = self.system_prompt.clone();
        let agent_name = self.config.name.clone();
        let agent_id = self.config.id;
        let llm_model = self.config.llm_model.clone();
        
        Box::pin(
            async move { task }
                .into_actor(self)
                .then(move |task, actor, _ctx| {
                    let task_clone = task.clone();
                    let llm = llm_client.clone();
                    let prompt = system_prompt.clone();
                    let name = agent_name.clone();
                    let model = llm_model.clone();
                    
                    async move {
                        // Build context from task
                        let mut context = format!("Task: {}\n", task_clone.description);
                        if let Some(email) = &task_clone.context.source_email {
                            context.push_str(&format!(
                                "\nEmail from: {}\nSubject: {}\nBody:\n{}\n",
                                email.metadata.from,
                                email.metadata.subject,
                                email.text_body.as_deref().unwrap_or("[No body]")
                            ));
                        }
                        
                        // Call LLM if available
                        if let Some(client) = llm {
                            info!("Agent {} calling LLM for task {}", name, task_clone.id);
                            match client.chat(&prompt, &context).await {
                                Ok(response) => {
                                    info!("Agent {} got LLM response: {} chars", name, response.content.len());
                                    Ok::<_, AmailError>((task_clone, Some(response)))
                                }
                                Err(e) => {
                                    warn!("Agent {} LLM call failed: {}", name, e);
                                    Ok((task_clone, None))
                                }
                            }
                        } else {
                            warn!("Agent {} has no LLM client", name);
                            Ok((task_clone, None))
                        }
                    }
                    .into_actor(actor)
                })
                .map(|result, actor, _ctx| {
                    match result {
                        Ok((task, llm_response)) => {
                            actor.active_tasks.remove(&task.id);
                            actor.completed_tasks += 1;
                            
                            if actor.active_tasks.is_empty() {
                                actor.status = AgentStatus::Ready;
                            }

                            // Build result with LLM response if available
                            let (response_content, duration, tokens) = if let Some(resp) = llm_response {
                                (
                                    serde_json::json!({
                                        "status": "completed",
                                        "description": task.description,
                                        "llm_response": resp.content,
                                        "model": resp.model,
                                    }),
                                    resp.duration_ms,
                                    Some(TokenUsage {
                                        prompt_tokens: resp.prompt_tokens,
                                        completion_tokens: resp.completion_tokens,
                                        total_tokens: resp.total_tokens,
                                    }),
                                )
                            } else {
                                (
                                    serde_json::json!({
                                        "status": "completed_no_llm",
                                        "description": task.description,
                                    }),
                                    0,
                                    None,
                                )
                            };

                            Ok(TaskResult {
                                task_id: task.id,
                                delegation_id: None,
                                agent_id: actor.config.id,
                                success: true,
                                result: response_content,
                                artifacts: Vec::new(),
                                completed_at: Utc::now(),
                                duration_ms: duration,
                                token_usage: tokens,
                            })
                        }
                        Err(e) => {
                            actor.failed_tasks += 1;
                            if actor.active_tasks.is_empty() {
                                actor.status = AgentStatus::Ready;
                            }
                            Err(e)
                        }
                    }
                }),
        )
    }
}

/// Handle A2A messages
impl Handler<A2AMessageWrapper> for AgentActor {
    type Result = Result<Option<A2AEnvelope>, AmailError>;

    fn handle(&mut self, msg: A2AMessageWrapper, _ctx: &mut Self::Context) -> Self::Result {
        let envelope = msg.0;
        debug!(
            "Agent {} received A2A message from {}",
            self.config.name, envelope.from
        );

        match envelope.message {
            A2AMessage::DelegationRequest(delegation) => {
                let accepted = self.handle_delegation(delegation.clone());
                let response_msg = if accepted {
                    A2AMessage::DelegationAccept {
                        delegation_id: delegation.id,
                        agent_id: self.config.id,
                    }
                } else {
                    A2AMessage::DelegationReject {
                        delegation_id: delegation.id,
                        agent_id: self.config.id,
                        reason: "Insufficient capacity or capabilities".to_string(),
                    }
                };

                Ok(Some(A2AEnvelope::new(
                    self.config.id,
                    envelope.from,
                    response_msg,
                )))
            }
            A2AMessage::Ping { timestamp } => {
                Ok(Some(A2AEnvelope::new(
                    self.config.id,
                    envelope.from,
                    A2AMessage::Pong {
                        timestamp,
                        agent_id: self.config.id,
                    },
                )))
            }
            A2AMessage::SupervisionCommand(cmd) => {
                debug!("Received supervision command: {:?}", cmd);
                // Handle supervision commands
                Ok(None)
            }
            _ => {
                warn!("Unhandled A2A message type");
                Ok(None)
            }
        }
    }
}

/// Handle supervision messages
impl Handler<SupervisionMessage> for AgentActor {
    type Result = ();

    fn handle(&mut self, msg: SupervisionMessage, ctx: &mut Self::Context) {
        match msg {
            SupervisionMessage::Start => {
                self.status = AgentStatus::Ready;
                info!("Agent {} started", self.config.name);
            }
            SupervisionMessage::Stop => {
                self.status = AgentStatus::ShuttingDown;
                info!("Agent {} stopping", self.config.name);
                ctx.stop();
            }
            SupervisionMessage::Pause => {
                self.status = AgentStatus::Paused;
                info!("Agent {} paused", self.config.name);
            }
            SupervisionMessage::Resume => {
                self.status = AgentStatus::Ready;
                info!("Agent {} resumed", self.config.name);
            }
            SupervisionMessage::Restart => {
                info!("Agent {} restarting", self.config.name);
                self.status = AgentStatus::Starting;
                // Re-initialize
            }
            SupervisionMessage::HealthCheck => {
                debug!("Agent {} health check: {:?}", self.config.name, self.status);
            }
        }
    }
}

/// Handle status queries
impl Handler<StatusQuery> for AgentActor {
    type Result = Result<AgentStatusResponse, ()>;

    fn handle(&mut self, _msg: StatusQuery, _ctx: &mut Self::Context) -> Self::Result {
        Ok(AgentStatusResponse {
            agent_id: self.config.id,
            status: self.status,
            active_tasks: self.active_tasks.len(),
            completed_tasks: self.completed_tasks,
            failed_tasks: self.failed_tasks,
        })
    }
}
