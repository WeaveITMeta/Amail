// =============================================================================
// WebSocket Handler for Dashboard
// =============================================================================
// Table of Contents:
// 1. WebSocket actor
// 2. Message types
// 3. Dashboard broadcast
// 4. Connection management
// =============================================================================

use actix::{Actor, ActorContext, Addr, AsyncContext, Handler, Message, Running, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Heartbeat interval
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// Client timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

// =============================================================================
// Dashboard State (shared across connections)
// =============================================================================

/// Shared dashboard state for broadcasting
#[derive(Clone)]
pub struct DashboardBroadcaster {
    /// Connected clients
    clients: Arc<RwLock<HashMap<String, Addr<DashboardWsSession>>>>,
    /// Current system stats
    stats: Arc<RwLock<SystemStats>>,
    /// Agent states
    agents: Arc<RwLock<HashMap<String, AgentState>>>,
    /// Recent tasks
    tasks: Arc<RwLock<Vec<TaskState>>>,
}

impl DashboardBroadcaster {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(SystemStats::default())),
            agents: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a new client
    pub async fn register(&self, id: String, addr: Addr<DashboardWsSession>) {
        let mut clients = self.clients.write().await;
        clients.insert(id.clone(), addr);
        info!("Dashboard client connected: {} (total: {})", id, clients.len());
    }

    /// Unregister a client
    pub async fn unregister(&self, id: &str) {
        let mut clients = self.clients.write().await;
        clients.remove(id);
        info!("Dashboard client disconnected: {} (total: {})", id, clients.len());
    }

    /// Broadcast message to all clients
    pub async fn broadcast(&self, msg: WsOutMessage) {
        let clients = self.clients.read().await;
        let json = serde_json::to_string(&msg).unwrap_or_default();
        
        for (id, addr) in clients.iter() {
            if addr.try_send(WsTextMessage(json.clone())).is_err() {
                warn!("Failed to send message to client {}", id);
            }
        }
    }

    /// Update agent state and broadcast
    pub async fn update_agent(&self, agent: AgentState) {
        {
            let mut agents = self.agents.write().await;
            agents.insert(agent.id.clone(), agent.clone());
        }
        self.broadcast(WsOutMessage::AgentUpdate(agent)).await;
    }

    /// Remove agent and broadcast
    pub async fn remove_agent(&self, agent_id: &str) {
        {
            let mut agents = self.agents.write().await;
            agents.remove(agent_id);
        }
        self.broadcast(WsOutMessage::AgentRemoved { agent_id: agent_id.to_string() }).await;
    }

    /// Update task and broadcast
    pub async fn update_task(&self, task: TaskState) {
        {
            let mut tasks = self.tasks.write().await;
            if let Some(existing) = tasks.iter_mut().find(|t| t.id == task.id) {
                *existing = task.clone();
            } else {
                tasks.insert(0, task.clone());
                if tasks.len() > 100 {
                    tasks.pop();
                }
            }
        }
        self.broadcast(WsOutMessage::TaskUpdate(task)).await;
    }

    /// Update stats and broadcast
    pub async fn update_stats(&self, stats: SystemStats) {
        {
            let mut current = self.stats.write().await;
            *current = stats.clone();
        }
        self.broadcast(WsOutMessage::StatsUpdate(stats)).await;
    }

    /// Send event to all clients
    pub async fn send_event(&self, event: DashboardEvent) {
        self.broadcast(WsOutMessage::Event(event)).await;
    }

    /// Get full state for new connections
    pub async fn get_full_state(&self) -> WsOutMessage {
        let agents = self.agents.read().await;
        let tasks = self.tasks.read().await;
        let stats = self.stats.read().await;
        
        WsOutMessage::FullState {
            agents: agents.values().cloned().collect(),
            tasks: tasks.clone(),
            stats: stats.clone(),
        }
    }

    /// Get client count
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }
}

impl Default for DashboardBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// WebSocket Session Actor
// =============================================================================

/// WebSocket session for dashboard client
pub struct DashboardWsSession {
    /// Unique session ID
    id: String,
    /// Last heartbeat
    hb: Instant,
    /// Broadcaster reference
    broadcaster: web::Data<DashboardBroadcaster>,
    /// Subscribed channels
    subscriptions: Vec<String>,
}

impl DashboardWsSession {
    pub fn new(broadcaster: web::Data<DashboardBroadcaster>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            hb: Instant::now(),
            broadcaster,
            subscriptions: Vec::new(),
        }
    }

    /// Start heartbeat
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                warn!("WebSocket client {} heartbeat failed, disconnecting", act.id);
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for DashboardWsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Start heartbeat
        self.hb(ctx);
        
        // Register with broadcaster
        let id = self.id.clone();
        let addr = ctx.address();
        let broadcaster = self.broadcaster.clone();
        
        actix::spawn(async move {
            broadcaster.register(id, addr).await;
        });
        
        // Send full state
        let broadcaster = self.broadcaster.clone();
        let addr = ctx.address();
        actix::spawn(async move {
            let full_state = broadcaster.get_full_state().await;
            let json = serde_json::to_string(&full_state).unwrap_or_default();
            let _ = addr.try_send(WsTextMessage(json));
        });
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        // Unregister from broadcaster
        let id = self.id.clone();
        let broadcaster = self.broadcaster.clone();
        
        actix::spawn(async move {
            broadcaster.unregister(&id).await;
        });
        
        Running::Stop
    }
}

/// Handle incoming WebSocket messages
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for DashboardWsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                // Parse incoming message
                if let Ok(msg) = serde_json::from_str::<WsInMessage>(&text) {
                    match msg {
                        WsInMessage::Subscribe { channels } => {
                            self.subscriptions = channels;
                            debug!("Client {} subscribed to: {:?}", self.id, self.subscriptions);
                        }
                        WsInMessage::Ping => {
                            let pong = serde_json::to_string(&WsOutMessage::Pong).unwrap_or_default();
                            ctx.text(pong);
                        }
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                debug!("Binary message received (ignored)");
            }
            Ok(ws::Message::Close(reason)) => {
                info!("WebSocket close requested: {:?}", reason);
                ctx.close(reason);
                ctx.stop();
            }
            Err(e) => {
                error!("WebSocket error: {:?}", e);
                ctx.stop();
            }
            _ => {}
        }
    }
}

/// Message for sending text to client
#[derive(Message)]
#[rtype(result = "()")]
pub struct WsTextMessage(pub String);

impl Handler<WsTextMessage> for DashboardWsSession {
    type Result = ();

    fn handle(&mut self, msg: WsTextMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

// =============================================================================
// Message Types
// =============================================================================

/// Incoming WebSocket messages
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum WsInMessage {
    Subscribe { channels: Vec<String> },
    Ping,
}

/// Outgoing WebSocket messages
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "lowercase")]
pub enum WsOutMessage {
    AgentUpdate(AgentState),
    AgentRemoved { agent_id: String },
    TaskUpdate(TaskState),
    StatsUpdate(SystemStats),
    Event(DashboardEvent),
    FullState {
        agents: Vec<AgentState>,
        tasks: Vec<TaskState>,
        stats: SystemStats,
    },
    Pong,
}

// =============================================================================
// State Types (matching dashboard frontend)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: String,
    pub name: String,
    pub role: String,
    pub status: AgentStatus,
    pub active_tasks: u32,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub last_activity: Option<String>,
    pub current_task: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Idle,
    Processing,
    Error,
    Starting,
    Stopping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    pub id: String,
    pub task_type: String,
    pub description: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub assigned_agent: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Critical,
    High,
    Normal,
    Low,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemStats {
    pub total_agents: u32,
    pub active_agents: u32,
    pub total_tasks_processed: u64,
    pub tasks_per_minute: f64,
    pub avg_task_duration_ms: u64,
    pub emails_received: u64,
    pub emails_sent: u64,
    pub llm_calls: u64,
    pub mcp_calls: u64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardEvent {
    pub id: String,
    pub event_type: EventType,
    pub message: String,
    pub timestamp: String,
    pub agent_id: Option<String>,
    pub task_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EventType {
    Info,
    Success,
    Warning,
    Error,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    AgentStarted,
    AgentStopped,
    EmailReceived,
    EmailSent,
}

// =============================================================================
// HTTP Handler
// =============================================================================

/// WebSocket upgrade handler
pub async fn dashboard_ws(
    req: HttpRequest,
    stream: web::Payload,
    broadcaster: web::Data<DashboardBroadcaster>,
) -> Result<HttpResponse, Error> {
    let session = DashboardWsSession::new(broadcaster);
    ws::start(session, &req, stream)
}
