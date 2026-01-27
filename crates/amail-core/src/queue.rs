// =============================================================================
// Redis Task Queue - Distributed Task Processing
// =============================================================================
// Table of Contents:
// 1. TaskQueue struct
// 2. Push/pop operations
// 3. Priority queue support
// 4. Dead letter queue
// =============================================================================

use chrono::Utc;
use redis::{aio::MultiplexedConnection, AsyncCommands, Client};
use uuid::Uuid;

use crate::{
    task::{Task, TaskId, TaskPriority},
    AmailError, Result,
};

/// Redis-backed task queue
#[derive(Clone)]
pub struct TaskQueue {
    client: Client,
    prefix: String,
}

impl TaskQueue {
    /// Create a new task queue
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| AmailError::Queue(format!("Failed to connect to Redis: {}", e)))?;

        Ok(Self {
            client,
            prefix: "amail".to_string(),
        })
    }

    /// Create with custom prefix
    pub fn with_prefix(redis_url: &str, prefix: &str) -> Result<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| AmailError::Queue(format!("Failed to connect to Redis: {}", e)))?;

        Ok(Self {
            client,
            prefix: prefix.to_string(),
        })
    }

    /// Get async connection
    async fn get_conn(&self) -> Result<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AmailError::Queue(format!("Redis connection failed: {}", e)))
    }

    /// Queue key for priority
    fn queue_key(&self, priority: &TaskPriority) -> String {
        format!("{}:tasks:{}", self.prefix, priority.as_str())
    }

    /// Processing queue key
    fn processing_key(&self) -> String {
        format!("{}:processing", self.prefix)
    }

    /// Dead letter queue key
    fn dlq_key(&self) -> String {
        format!("{}:dlq", self.prefix)
    }

    /// Task data key
    fn task_key(&self, task_id: &TaskId) -> String {
        format!("{}:task:{}", self.prefix, task_id.0)
    }

    /// Push a task to the queue
    pub async fn push(&self, task: &Task) -> Result<()> {
        let mut conn = self.get_conn().await?;

        // Serialize task
        let task_json = serde_json::to_string(task)
            .map_err(|e| AmailError::Queue(format!("Failed to serialize task: {}", e)))?;

        // Store task data
        let task_key = self.task_key(&task.id);
        conn.set_ex::<_, _, ()>(&task_key, &task_json, 86400 * 7) // 7 day TTL
            .await
            .map_err(|e| AmailError::Queue(e.to_string()))?;

        // Add to priority queue (using sorted set with timestamp as score)
        let queue_key = self.queue_key(&task.priority);
        let score = Utc::now().timestamp_millis() as f64;

        conn.zadd::<_, _, _, ()>(&queue_key, task.id.0.to_string(), score)
            .await
            .map_err(|e| AmailError::Queue(e.to_string()))?;

        Ok(())
    }

    /// Pop the next task from the queue (priority order: urgent > high > normal > low)
    pub async fn pop(&self) -> Result<Option<Task>> {
        let mut conn = self.get_conn().await?;

        // Try each priority queue in order
        for priority in &[
            TaskPriority::Urgent,
            TaskPriority::High,
            TaskPriority::Normal,
            TaskPriority::Low,
        ] {
            let queue_key = self.queue_key(priority);

            // Atomically pop from sorted set and add to processing set
            let result: Option<(String, f64)> = conn
                .zpopmin(&queue_key, 1)
                .await
                .map_err(|e| AmailError::Queue(e.to_string()))?;

            if let Some((task_id_str, _score)) = result.and_then(|v: (String, f64)| Some(v)) {
                // Parse task ID
                let task_id = Uuid::parse_str(&task_id_str)
                    .map_err(|e| AmailError::Queue(format!("Invalid task ID: {}", e)))?;

                // Get task data
                let task_key = self.task_key(&TaskId(task_id));
                let task_json: Option<String> = conn
                    .get(&task_key)
                    .await
                    .map_err(|e| AmailError::Queue(e.to_string()))?;

                if let Some(json) = task_json {
                    // Add to processing set with timeout
                    let processing_key = self.processing_key();
                    let timeout = Utc::now().timestamp_millis() as f64 + 300_000.0; // 5 min timeout
                    conn.zadd::<_, _, _, ()>(&processing_key, &task_id_str, timeout)
                        .await
                        .map_err(|e| AmailError::Queue(e.to_string()))?;

                    // Deserialize and return
                    let task: Task = serde_json::from_str(&json)
                        .map_err(|e| AmailError::Queue(format!("Failed to deserialize task: {}", e)))?;

                    return Ok(Some(task));
                }
            }
        }

        Ok(None)
    }

    /// Mark a task as completed (remove from processing)
    pub async fn complete(&self, task_id: &TaskId) -> Result<()> {
        let mut conn = self.get_conn().await?;

        // Remove from processing set
        let processing_key = self.processing_key();
        conn.zrem::<_, _, ()>(&processing_key, task_id.0.to_string())
            .await
            .map_err(|e| AmailError::Queue(e.to_string()))?;

        // Delete task data
        let task_key = self.task_key(task_id);
        conn.del::<_, ()>(&task_key)
            .await
            .map_err(|e| AmailError::Queue(e.to_string()))?;

        Ok(())
    }

    /// Move a task to the dead letter queue
    pub async fn move_to_dlq(&self, task_id: &TaskId, error: &str) -> Result<()> {
        let mut conn = self.get_conn().await?;

        // Remove from processing
        let processing_key = self.processing_key();
        conn.zrem::<_, _, ()>(&processing_key, task_id.0.to_string())
            .await
            .map_err(|e| AmailError::Queue(e.to_string()))?;

        // Add to DLQ with error info
        let dlq_key = self.dlq_key();
        let dlq_entry = serde_json::json!({
            "task_id": task_id.0.to_string(),
            "error": error,
            "failed_at": Utc::now().to_rfc3339()
        });

        conn.lpush::<_, _, ()>(&dlq_key, dlq_entry.to_string())
            .await
            .map_err(|e| AmailError::Queue(e.to_string()))?;

        Ok(())
    }

    /// Requeue timed-out tasks from processing back to their queues
    pub async fn requeue_timeouts(&self) -> Result<u64> {
        let mut conn = self.get_conn().await?;
        let processing_key = self.processing_key();
        let now = Utc::now().timestamp_millis() as f64;

        // Get timed-out tasks
        let timed_out: Vec<String> = conn
            .zrangebyscore(&processing_key, 0f64, now)
            .await
            .map_err(|e| AmailError::Queue(e.to_string()))?;

        let mut requeued = 0;

        for task_id_str in timed_out {
            // Get task data to determine priority
            let task_id = match Uuid::parse_str(&task_id_str) {
                Ok(id) => TaskId(id),
                Err(_) => continue,
            };

            let task_key = self.task_key(&task_id);
            let task_json: Option<String> = conn
                .get(&task_key)
                .await
                .map_err(|e| AmailError::Queue(e.to_string()))?;

            if let Some(json) = task_json {
                if let Ok(task) = serde_json::from_str::<Task>(&json) {
                    // Remove from processing
                    conn.zrem::<_, _, ()>(&processing_key, &task_id_str)
                        .await
                        .ok();

                    // Re-add to priority queue
                    let queue_key = self.queue_key(&task.priority);
                    let score = Utc::now().timestamp_millis() as f64;
                    conn.zadd::<_, _, _, ()>(&queue_key, &task_id_str, score)
                        .await
                        .ok();

                    requeued += 1;
                }
            }
        }

        Ok(requeued)
    }

    /// Get queue statistics
    pub async fn stats(&self) -> Result<QueueStats> {
        let mut conn = self.get_conn().await?;

        let urgent: u64 = conn
            .zcard(&self.queue_key(&TaskPriority::Urgent))
            .await
            .unwrap_or(0);
        let high: u64 = conn
            .zcard(&self.queue_key(&TaskPriority::High))
            .await
            .unwrap_or(0);
        let normal: u64 = conn
            .zcard(&self.queue_key(&TaskPriority::Normal))
            .await
            .unwrap_or(0);
        let low: u64 = conn
            .zcard(&self.queue_key(&TaskPriority::Low))
            .await
            .unwrap_or(0);
        let processing: u64 = conn.zcard(&self.processing_key()).await.unwrap_or(0);
        let dlq: u64 = conn.llen(&self.dlq_key()).await.unwrap_or(0);

        Ok(QueueStats {
            pending_urgent: urgent,
            pending_high: high,
            pending_normal: normal,
            pending_low: low,
            processing,
            dead_letter: dlq,
            total_pending: urgent + high + normal + low,
        })
    }
}

/// Queue statistics
#[derive(Debug, Clone, Default)]
pub struct QueueStats {
    pub pending_urgent: u64,
    pub pending_high: u64,
    pub pending_normal: u64,
    pub pending_low: u64,
    pub processing: u64,
    pub dead_letter: u64,
    pub total_pending: u64,
}

