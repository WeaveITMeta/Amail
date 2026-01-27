// =============================================================================
// AMAIL-WEBHOOK - Stalwart Webhook Receiver & HTTP API
// =============================================================================
// Table of Contents:
// 1. Webhook handlers
// 2. API routes
// 3. Stalwart integration
// 4. REST API for dashboard
// =============================================================================

pub mod handlers;
pub mod routes;
pub mod stalwart;
pub mod signature;
pub mod websocket;
pub mod api;
pub mod p2p_api;
pub mod config_api;
pub mod storage;

pub use handlers::{webhook_handler, WebhookPayload};
pub use routes::configure_routes;
pub use stalwart::{OutboundEmail, SmtpConfig, SmtpMailer, SmtpStats, StalwartClient, EmailSummary};
pub use websocket::{
    dashboard_ws, AgentState, AgentStatus, DashboardBroadcaster, DashboardEvent,
    EventType, SystemStats, TaskPriority, TaskState, TaskStatus, WsOutMessage,
};
pub use api::{ApiState, configure_routes as configure_api_routes};
pub use p2p_api::{P2PState, configure_p2p_routes};
pub use config_api::{ConfigState, configure_config_routes};
pub use storage::{UnifiedStorage, StoredDomain, StoredMailbox};
