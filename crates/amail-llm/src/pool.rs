// =============================================================================
// LLM Pool - Load-Balanced Multi-Instance Ollama Client
// =============================================================================
// Table of Contents:
// 1. LlmPool struct - Round-robin load balancer
// 2. Instance health checking
// 3. Automatic failover
// =============================================================================

use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::client::{ChatMessage, ChatResponse, LlmClient};
use crate::tools::ToolDefinition;

/// Health status of an Ollama instance
#[derive(Debug, Clone)]
pub struct InstanceHealth {
    pub url: String,
    pub available: bool,
    pub last_check: std::time::Instant,
    pub consecutive_failures: u32,
}

/// Load-balanced LLM pool for multiple Ollama instances
#[derive(Clone)]
pub struct LlmPool {
    /// All configured instances
    instances: Arc<Vec<LlmClient>>,
    /// Health status of each instance
    health: Arc<RwLock<Vec<InstanceHealth>>>,
    /// Round-robin counter
    counter: Arc<AtomicUsize>,
    /// Model name
    model: String,
}

impl LlmPool {
    /// Create a new LLM pool from a list of Ollama URLs
    pub fn new(urls: Vec<String>, model: &str) -> Result<Self> {
        if urls.is_empty() {
            return Err(anyhow!("At least one Ollama URL is required"));
        }

        let mut instances = Vec::new();
        let mut health = Vec::new();

        for url in &urls {
            match LlmClient::with_url(model, url) {
                Ok(client) => {
                    instances.push(client);
                    health.push(InstanceHealth {
                        url: url.clone(),
                        available: true, // Assume available until proven otherwise
                        last_check: std::time::Instant::now(),
                        consecutive_failures: 0,
                    });
                    info!("LLM pool: Added instance at {}", url);
                }
                Err(e) => {
                    warn!("LLM pool: Failed to create client for {}: {}", url, e);
                }
            }
        }

        if instances.is_empty() {
            return Err(anyhow!("No valid Ollama instances could be created"));
        }

        info!("LLM pool initialized with {} instances", instances.len());

        Ok(Self {
            instances: Arc::new(instances),
            health: Arc::new(RwLock::new(health)),
            counter: Arc::new(AtomicUsize::new(0)),
            model: model.to_string(),
        })
    }

    /// Create from a single URL (backwards compatible)
    pub fn single(url: &str, model: &str) -> Result<Self> {
        Self::new(vec![url.to_string()], model)
    }

    /// Get the next available instance using round-robin
    async fn get_next_instance(&self) -> Option<&LlmClient> {
        let health = self.health.read().await;
        let instance_count = self.instances.len();
        
        // Try each instance starting from the current counter
        for _ in 0..instance_count {
            let idx = self.counter.fetch_add(1, Ordering::Relaxed) % instance_count;
            
            if health[idx].available || health[idx].consecutive_failures < 3 {
                return Some(&self.instances[idx]);
            }
        }

        // If all instances have failed, try the first one anyway
        Some(&self.instances[0])
    }

    /// Mark an instance as failed
    async fn mark_failed(&self, idx: usize) {
        let mut health = self.health.write().await;
        if idx < health.len() {
            health[idx].consecutive_failures += 1;
            if health[idx].consecutive_failures >= 3 {
                health[idx].available = false;
                warn!(
                    "LLM pool: Instance {} marked unavailable after {} failures",
                    health[idx].url, health[idx].consecutive_failures
                );
            }
        }
    }

    /// Mark an instance as successful
    async fn mark_success(&self, idx: usize) {
        let mut health = self.health.write().await;
        if idx < health.len() {
            health[idx].available = true;
            health[idx].consecutive_failures = 0;
            health[idx].last_check = std::time::Instant::now();
        }
    }

    /// Simple chat completion with automatic failover
    pub async fn chat(&self, system_prompt: &str, user_message: &str) -> Result<ChatResponse> {
        let instance_count = self.instances.len();
        let start_idx = self.counter.load(Ordering::Relaxed) % instance_count;

        // Try each instance
        for attempt in 0..instance_count {
            let idx = (start_idx + attempt) % instance_count;
            let instance = &self.instances[idx];

            debug!("LLM pool: Trying instance {} ({})", idx, instance.base_url);

            match instance.chat(system_prompt, user_message).await {
                Ok(response) => {
                    self.mark_success(idx).await;
                    return Ok(response);
                }
                Err(e) => {
                    warn!("LLM pool: Instance {} failed: {}", idx, e);
                    self.mark_failed(idx).await;
                }
            }
        }

        Err(anyhow!("All LLM instances failed"))
    }

    /// Chat completion with message history
    pub async fn chat_completion(&self, messages: Vec<ChatMessage>) -> Result<ChatResponse> {
        let instance_count = self.instances.len();
        let start_idx = self.counter.load(Ordering::Relaxed) % instance_count;

        for attempt in 0..instance_count {
            let idx = (start_idx + attempt) % instance_count;
            let instance = &self.instances[idx];

            match instance.chat_completion(messages.clone()).await {
                Ok(response) => {
                    self.mark_success(idx).await;
                    return Ok(response);
                }
                Err(e) => {
                    warn!("LLM pool: Instance {} failed: {}", idx, e);
                    self.mark_failed(idx).await;
                }
            }
        }

        Err(anyhow!("All LLM instances failed"))
    }

    /// Chat with tool calling support
    pub async fn chat_with_tools(
        &self,
        system_prompt: &str,
        user_message: &str,
        tools: &[ToolDefinition],
    ) -> Result<ChatResponse> {
        let instance_count = self.instances.len();
        let start_idx = self.counter.load(Ordering::Relaxed) % instance_count;

        for attempt in 0..instance_count {
            let idx = (start_idx + attempt) % instance_count;
            let instance = &self.instances[idx];

            match instance.chat_with_tools(system_prompt, user_message, tools).await {
                Ok(response) => {
                    self.mark_success(idx).await;
                    return Ok(response);
                }
                Err(e) => {
                    warn!("LLM pool: Instance {} failed: {}", idx, e);
                    self.mark_failed(idx).await;
                }
            }
        }

        Err(anyhow!("All LLM instances failed"))
    }

    /// Check health of all instances
    pub async fn health_check(&self) -> Vec<InstanceHealth> {
        let mut health = self.health.write().await;
        
        for (idx, instance) in self.instances.iter().enumerate() {
            let available = instance.is_available().await;
            health[idx].available = available;
            health[idx].last_check = std::time::Instant::now();
            if available {
                health[idx].consecutive_failures = 0;
            }
        }

        health.clone()
    }

    /// Get number of instances
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    /// Get number of available instances
    pub async fn available_count(&self) -> usize {
        let health = self.health.read().await;
        health.iter().filter(|h| h.available).count()
    }

    /// Get model name
    pub fn model(&self) -> &str {
        &self.model
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pool_creation() {
        let pool = LlmPool::new(
            vec![
                "http://localhost:11434".to_string(),
                "http://localhost:11435".to_string(),
            ],
            "mistral:latest",
        );
        assert!(pool.is_ok());
        assert_eq!(pool.unwrap().instance_count(), 2);
    }
}
