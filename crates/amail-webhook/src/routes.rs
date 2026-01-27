// =============================================================================
// HTTP Routes Configuration
// =============================================================================
// Table of Contents:
// 1. Route configuration
// 2. API versioning
// =============================================================================

use actix_web::web;

use crate::handlers::{health_handler, metrics_handler, webhook_handler, inbound_mail_handler, list_mailboxes_handler, list_inbox_handler, inject_test_email_handler};
use crate::websocket::dashboard_ws;

/// Configure all routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            // Health and metrics
            .route("/health", web::get().to(health_handler))
            .route("/metrics", web::get().to(metrics_handler))
            // Webhook endpoint
            .route("/webhook", web::post().to(webhook_handler))
            .route("/webhook/stalwart", web::post().to(webhook_handler))
            // Inbound mail from embedded SMTP server
            .route("/mail/inbound", web::post().to(inbound_mail_handler))
            // Mailboxes API
            .route("/mailboxes", web::get().to(list_mailboxes_handler))
            // Inbox API
            .route("/inbox", web::get().to(list_inbox_handler))
            // Test email injection (development only)
            .route("/inbox/test", web::post().to(inject_test_email_handler))
    )
    // WebSocket endpoint for dashboard
    .route("/ws/dashboard", web::get().to(dashboard_ws));
}
