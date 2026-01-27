//! API Server - REST endpoints for dashboard data
//!
//! Table of Contents:
//! 1. App State
//! 2. Domain endpoints
//! 3. Mailbox endpoints
//! 4. Email/Inbox endpoints
//! 5. Analytics endpoints
//! 6. Review endpoints
//! 7. Agent endpoints
//! 8. Config endpoints

use actix_web::{web, HttpResponse, Responder};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use amail_core::repositories::{
    DomainRepository, MailboxRepository, EmailRepository, 
    AnalyticsRepository, ReviewRepository, ConfigRepository,
};
use amail_core::db::{Database, AgentStateRepository};
use amail_core::stalwart::StalwartClient;

// =============================================================================
// 1. App State for API Routes (requires database)
// =============================================================================

#[derive(Clone)]
pub struct ApiState {
    pub db: PgPool,
    pub stalwart: StalwartClient,
}

impl ApiState {
    pub fn new(db: PgPool, stalwart: StalwartClient) -> Self {
        Self { db, stalwart }
    }
    
    pub async fn from_env(database_url: &str, stalwart_url: &str, stalwart_token: Option<String>) -> anyhow::Result<Self> {
        let db = Database::connect(database_url).await?;
        db.migrate().await?;
        
        Ok(Self {
            db: db.pool().clone(),
            stalwart: StalwartClient::new(stalwart_url, stalwart_token),
        })
    }
}

// =============================================================================
// 2. Domain Endpoints
// =============================================================================

#[derive(Debug, Serialize)]
pub struct DomainResponse {
    pub id: String,
    pub name: String,
    pub status: String,
    pub mx_verified: bool,
    pub spf_verified: bool,
    pub dkim_verified: bool,
    pub dmarc_verified: bool,
    pub mailbox_count: i64,
    pub created_at: String,
}

pub async fn list_domains(state: web::Data<ApiState>) -> impl Responder {
    let repo = DomainRepository::new(&state.db);
    let mailbox_repo = MailboxRepository::new(&state.db);
    
    match repo.get_all().await {
        Ok(domains) => {
            let mut responses = Vec::new();
            for domain in domains {
                let mailbox_count = mailbox_repo.count_by_domain(domain.id).await.unwrap_or(0);
                responses.push(DomainResponse {
                    id: domain.id.to_string(),
                    name: domain.name,
                    status: domain.status,
                    mx_verified: domain.mx_verified,
                    spf_verified: domain.spf_verified,
                    dkim_verified: domain.dkim_verified,
                    dmarc_verified: domain.dmarc_verified,
                    mailbox_count,
                    created_at: domain.created_at.to_rfc3339(),
                });
            }
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateDomainRequest {
    pub name: String,
}

pub async fn create_domain(
    state: web::Data<ApiState>,
    body: web::Json<CreateDomainRequest>,
) -> impl Responder {
    let repo = DomainRepository::new(&state.db);
    
    match repo.create(&body.name).await {
        Ok(domain) => HttpResponse::Created().json(DomainResponse {
            id: domain.id.to_string(),
            name: domain.name,
            status: domain.status,
            mx_verified: domain.mx_verified,
            spf_verified: domain.spf_verified,
            dkim_verified: domain.dkim_verified,
            dmarc_verified: domain.dmarc_verified,
            mailbox_count: 0,
            created_at: domain.created_at.to_rfc3339(),
        }),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn get_domain(
    state: web::Data<ApiState>,
    path: web::Path<String>,
) -> impl Responder {
    let domain_id = match Uuid::parse_str(&path) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid domain ID"
        })),
    };
    
    let repo = DomainRepository::new(&state.db);
    let mailbox_repo = MailboxRepository::new(&state.db);
    
    match repo.get_by_id(domain_id).await {
        Ok(Some(domain)) => {
            let mailbox_count = mailbox_repo.count_by_domain(domain.id).await.unwrap_or(0);
            HttpResponse::Ok().json(DomainResponse {
                id: domain.id.to_string(),
                name: domain.name,
                status: domain.status,
                mx_verified: domain.mx_verified,
                spf_verified: domain.spf_verified,
                dkim_verified: domain.dkim_verified,
                dmarc_verified: domain.dmarc_verified,
                mailbox_count,
                created_at: domain.created_at.to_rfc3339(),
            })
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Domain not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn delete_domain(
    state: web::Data<ApiState>,
    path: web::Path<String>,
) -> impl Responder {
    let domain_id = match Uuid::parse_str(&path) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid domain ID"
        })),
    };
    
    let repo = DomainRepository::new(&state.db);
    
    match repo.delete(domain_id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// =============================================================================
// 3. Mailbox Endpoints
// =============================================================================

#[derive(Debug, Serialize)]
pub struct MailboxResponse {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub domain: String,
    pub mailbox_type: String,
    pub is_active: bool,
    pub unread_count: i64,
}

pub async fn list_mailboxes(state: web::Data<ApiState>) -> impl Responder {
    let repo = MailboxRepository::new(&state.db);
    
    match repo.get_all_with_domain().await {
        Ok(mailboxes) => {
            let responses: Vec<MailboxResponse> = mailboxes.into_iter().map(|m| MailboxResponse {
                id: m.id.to_string(),
                email: m.email(),
                display_name: m.display_name,
                domain: m.domain_name,
                mailbox_type: m.mailbox_type,
                is_active: m.is_active,
                unread_count: m.unread_count,
            }).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateMailboxRequest {
    pub domain_id: String,
    pub local_part: String,
    pub display_name: Option<String>,
    pub mailbox_type: Option<String>,
}

pub async fn create_mailbox(
    state: web::Data<ApiState>,
    body: web::Json<CreateMailboxRequest>,
) -> impl Responder {
    let domain_id = match Uuid::parse_str(&body.domain_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid domain ID"
        })),
    };
    
    let repo = MailboxRepository::new(&state.db);
    let mailbox_type = body.mailbox_type.as_deref().unwrap_or("standard");
    
    match repo.create(domain_id, &body.local_part, body.display_name.as_deref(), mailbox_type).await {
        Ok(mailbox) => HttpResponse::Created().json(serde_json::json!({
            "id": mailbox.id.to_string(),
            "local_part": mailbox.local_part,
            "display_name": mailbox.display_name,
            "mailbox_type": mailbox.mailbox_type,
        })),
        Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn list_mailboxes_by_domain(
    state: web::Data<ApiState>,
    path: web::Path<String>,
) -> impl Responder {
    let domain_id = match Uuid::parse_str(&path) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid domain ID"
        })),
    };
    
    let repo = MailboxRepository::new(&state.db);
    
    match repo.get_by_domain(domain_id).await {
        Ok(mailboxes) => {
            let responses: Vec<MailboxResponse> = mailboxes.into_iter().map(|m| MailboxResponse {
                id: m.id.to_string(),
                email: m.email(),
                display_name: m.display_name,
                domain: m.domain_name,
                mailbox_type: m.mailbox_type,
                is_active: m.is_active,
                unread_count: m.unread_count,
            }).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// =============================================================================
// 4. Email/Inbox Endpoints
// =============================================================================

#[derive(Debug, Serialize)]
pub struct InboxEmailResponse {
    pub id: String,
    pub from_address: String,
    pub from_name: Option<String>,
    pub to: Vec<String>,
    pub subject: String,
    pub preview: Option<String>,
    pub timestamp: String,
    pub is_read: bool,
    pub is_starred: bool,
    pub folder: String,
    pub labels: Vec<String>,
    pub mailbox_email: Option<String>,
    pub mailbox_name: Option<String>,
    pub ai_confidence: Option<f64>,
    pub ai_status: Option<String>,
    pub attachment_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct InboxQuery {
    pub mailbox_id: Option<String>,
    pub folder: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn get_inbox(
    state: web::Data<ApiState>,
    query: web::Query<InboxQuery>,
) -> impl Responder {
    let repo = EmailRepository::new(&state.db);
    
    let mailbox_id = query.mailbox_id.as_ref()
        .and_then(|id| Uuid::parse_str(id).ok());
    let folder = query.folder.as_deref().unwrap_or("inbox");
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    
    match repo.get_inbox(mailbox_id, folder, query.search.as_deref(), limit, offset).await {
        Ok(emails) => {
            let responses: Vec<InboxEmailResponse> = emails.into_iter().map(|e| InboxEmailResponse {
                id: e.message_id,
                from_address: e.from_address,
                from_name: e.from_name,
                to: e.to_addresses,
                subject: e.subject,
                preview: e.preview,
                timestamp: e.received_at.to_rfc3339(),
                is_read: e.is_read,
                is_starred: e.is_starred,
                folder: e.folder,
                labels: e.labels,
                mailbox_email: e.mailbox_email,
                mailbox_name: e.mailbox_name,
                ai_confidence: e.ai_confidence,
                ai_status: e.ai_status,
                attachment_count: e.attachment_count.unwrap_or(0),
            }).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn get_email(
    state: web::Data<ApiState>,
    path: web::Path<String>,
) -> impl Responder {
    let repo = EmailRepository::new(&state.db);
    
    match repo.get_by_id(&path).await {
        Ok(Some(email)) => {
            let attachments = repo.get_attachments(&email.message_id).await.unwrap_or_default();
            HttpResponse::Ok().json(serde_json::json!({
                "id": email.message_id,
                "from_address": email.from_address,
                "from_name": email.from_name,
                "to": email.to_addresses,
                "cc": email.cc_addresses,
                "subject": email.subject,
                "text_body": email.text_body,
                "html_body": email.html_body,
                "received_at": email.received_at.to_rfc3339(),
                "is_read": email.is_read,
                "is_starred": email.is_starred,
                "folder": email.folder,
                "labels": email.labels,
                "attachments": attachments.into_iter().map(|a| serde_json::json!({
                    "id": a.id.to_string(),
                    "filename": a.filename,
                    "content_type": a.content_type,
                    "size": a.size_bytes,
                })).collect::<Vec<_>>(),
            }))
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Email not found"
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateEmailRequest {
    pub is_read: Option<bool>,
    pub is_starred: Option<bool>,
    pub folder: Option<String>,
    pub add_label: Option<String>,
    pub remove_label: Option<String>,
}

pub async fn update_email(
    state: web::Data<ApiState>,
    path: web::Path<String>,
    body: web::Json<UpdateEmailRequest>,
) -> impl Responder {
    let repo = EmailRepository::new(&state.db);
    let message_id = path.into_inner();
    
    if let Some(is_read) = body.is_read {
        if let Err(e) = repo.mark_read(&message_id, is_read).await {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }));
        }
    }
    
    if let Some(is_starred) = body.is_starred {
        if let Err(e) = repo.mark_starred(&message_id, is_starred).await {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }));
        }
    }
    
    if let Some(ref folder) = body.folder {
        if let Err(e) = repo.move_to_folder(&message_id, folder).await {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }));
        }
    }
    
    if let Some(ref label) = body.add_label {
        if let Err(e) = repo.add_label(&message_id, label).await {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }));
        }
    }
    
    if let Some(ref label) = body.remove_label {
        if let Err(e) = repo.remove_label(&message_id, label).await {
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": e.to_string()
            }));
        }
    }
    
    HttpResponse::Ok().json(serde_json::json!({ "success": true }))
}

pub async fn delete_email(
    state: web::Data<ApiState>,
    path: web::Path<String>,
) -> impl Responder {
    let repo = EmailRepository::new(&state.db);
    
    match repo.delete(&path).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// =============================================================================
// 5. Analytics Endpoints
// =============================================================================

#[derive(Debug, Serialize)]
pub struct DashboardStatsResponse {
    pub active_agents: i64,
    pub total_agents: i64,
    pub pending_tasks: i64,
    pub active_tasks: i64,
    pub pending_reviews: i64,
    pub emails_today: i64,
    pub unread_emails: i64,
    pub tasks_week: i64,
    pub avg_response_time_ms: f64,
}

pub async fn get_dashboard_stats(state: web::Data<ApiState>) -> impl Responder {
    let repo = AnalyticsRepository::new(&state.db);
    
    match repo.get_dashboard_stats().await {
        Ok(stats) => HttpResponse::Ok().json(DashboardStatsResponse {
            active_agents: stats.active_agents.unwrap_or(0),
            total_agents: stats.total_agents.unwrap_or(0),
            pending_tasks: stats.pending_tasks.unwrap_or(0),
            active_tasks: stats.active_tasks.unwrap_or(0),
            pending_reviews: stats.pending_reviews.unwrap_or(0),
            emails_today: stats.emails_today.unwrap_or(0),
            unread_emails: stats.unread_emails.unwrap_or(0),
            tasks_week: stats.tasks_week.unwrap_or(0),
            avg_response_time_ms: stats.avg_response_time_ms.unwrap_or(0.0),
        }),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub period: Option<String>,
}

pub async fn get_daily_metrics(
    state: web::Data<ApiState>,
    query: web::Query<MetricsQuery>,
) -> impl Responder {
    let repo = AnalyticsRepository::new(&state.db);
    
    let end_date = query.end_date.as_ref()
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| Utc::now().date_naive());
    
    let days = match query.period.as_deref() {
        Some("week") => 7,
        Some("month") => 30,
        Some("quarter") => 90,
        _ => 7,
    };
    
    let start_date = query.start_date.as_ref()
        .and_then(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .unwrap_or_else(|| end_date - chrono::Duration::days(days));
    
    match repo.get_daily_metrics_range(start_date, end_date).await {
        Ok(metrics) => {
            let responses: Vec<serde_json::Value> = metrics.into_iter().map(|m| serde_json::json!({
                "date": m.date.to_string(),
                "emails_received": m.emails_received,
                "emails_sent": m.emails_sent,
                "emails_auto_replied": m.emails_auto_replied,
                "tasks_created": m.tasks_created,
                "tasks_completed": m.tasks_completed,
                "tasks_failed": m.tasks_failed,
                "avg_response_time_ms": m.avg_response_time_ms,
                "reviews_requested": m.reviews_requested,
                "reviews_approved": m.reviews_approved,
                "reviews_rejected": m.reviews_rejected,
                "llm_calls": m.llm_calls,
                "llm_tokens_used": m.llm_tokens_used,
            })).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn get_hourly_metrics(
    state: web::Data<ApiState>,
    query: web::Query<MetricsQuery>,
) -> impl Responder {
    let repo = AnalyticsRepository::new(&state.db);
    
    let hours = match query.period.as_deref() {
        Some("day") => 24,
        Some("week") => 168,
        _ => 24,
    };
    
    match repo.get_hourly_metrics(hours).await {
        Ok(metrics) => {
            let responses: Vec<serde_json::Value> = metrics.into_iter().map(|m| serde_json::json!({
                "hour": m.hour.to_rfc3339(),
                "emails_received": m.emails_received,
                "emails_sent": m.emails_sent,
                "tasks_completed": m.tasks_completed,
                "avg_response_time_ms": m.avg_response_time_ms,
            })).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// =============================================================================
// 6. Review Endpoints
// =============================================================================

#[derive(Debug, Serialize)]
pub struct ReviewResponse {
    pub id: String,
    pub task_id: String,
    pub email_id: Option<String>,
    pub reason: String,
    pub urgency: String,
    pub status: String,
    pub confidence_score: Option<f64>,
    pub created_at: String,
    pub from_address: Option<String>,
    pub from_name: Option<String>,
    pub subject: Option<String>,
    pub email_preview: Option<String>,
    pub draft_response: Option<String>,
    pub agent_role: Option<String>,
}

pub async fn get_review_queue(state: web::Data<ApiState>) -> impl Responder {
    let repo = ReviewRepository::new(&state.db);
    
    match repo.get_pending_queue(100).await {
        Ok(reviews) => {
            let responses: Vec<ReviewResponse> = reviews.into_iter().map(|r| ReviewResponse {
                id: r.id.to_string(),
                task_id: r.task_id.to_string(),
                email_id: r.email_id,
                reason: r.reason,
                urgency: r.urgency,
                status: r.status,
                confidence_score: r.confidence_score,
                created_at: r.created_at.to_rfc3339(),
                from_address: r.from_address,
                from_name: r.from_name,
                subject: r.subject,
                email_preview: r.email_preview,
                draft_response: r.draft_response,
                agent_role: r.agent_role,
            }).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct ReviewDecisionRequest {
    pub decision: String,
    pub reviewer_notes: Option<String>,
    pub reviewed_by: String,
    pub edited_response: Option<String>,
}

pub async fn submit_review_decision(
    state: web::Data<ApiState>,
    path: web::Path<String>,
    body: web::Json<ReviewDecisionRequest>,
) -> impl Responder {
    let review_id = match Uuid::parse_str(&path) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid review ID"
        })),
    };
    
    let repo = ReviewRepository::new(&state.db);
    let analytics = AnalyticsRepository::new(&state.db);
    
    match repo.update_decision(
        review_id,
        &body.decision,
        body.reviewer_notes.as_deref(),
        &body.reviewed_by,
        body.edited_response.as_deref(),
    ).await {
        Ok(_) => {
            // Track analytics
            let approved = body.decision == "approved" || body.decision == "approve";
            let _ = analytics.increment_review_decision(approved).await;
            
            HttpResponse::Ok().json(serde_json::json!({ "success": true }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// =============================================================================
// 7. Agent Endpoints
// =============================================================================

#[derive(Debug, Serialize)]
pub struct AgentResponse {
    pub id: String,
    pub role: String,
    pub status: String,
    pub last_heartbeat: String,
    pub tasks_processed: i64,
}

pub async fn list_agents(state: web::Data<ApiState>) -> impl Responder {
    let repo = AgentStateRepository::new(&state.db);
    
    match repo.get_all().await {
        Ok(agents) => {
            let responses: Vec<AgentResponse> = agents.into_iter().map(|a| AgentResponse {
                id: a.id.to_string(),
                role: a.role,
                status: a.status,
                last_heartbeat: a.last_heartbeat.to_rfc3339(),
                tasks_processed: a.tasks_processed,
            }).collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// =============================================================================
// 8. Config Endpoints
// =============================================================================

pub async fn get_scheduler_config(state: web::Data<ApiState>) -> impl Responder {
    let repo = ConfigRepository::new(&state.db);
    
    match repo.get_scheduler_config().await {
        Ok(config) => HttpResponse::Ok().json(serde_json::json!({
            "auto_start_enabled": config.auto_start_enabled,
            "daily_start_time": config.daily_start_time.format("%H:%M").to_string(),
            "daily_summary_enabled": config.daily_summary_enabled,
            "process_overnight_emails": config.process_overnight_emails,
            "auto_cleanup_enabled": config.auto_cleanup_enabled,
            "retention_days": config.retention_days,
            "activate_agents_on_start": config.activate_agents_on_start,
            "telegram_daily_digest": config.telegram_daily_digest,
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

pub async fn get_server_config(state: web::Data<ApiState>) -> impl Responder {
    let repo = ConfigRepository::new(&state.db);
    
    match repo.get_server_config().await {
        Ok(config) => HttpResponse::Ok().json(serde_json::json!({
            "smtp_port": config.smtp_port,
            "http_port": config.http_port,
            "ollama_url": config.ollama_url,
            "llm_model": config.llm_model,
            "stalwart_url": config.stalwart_url,
            "telegram_enabled": config.telegram_enabled,
            "telegram_chat_id": config.telegram_chat_id,
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        })),
    }
}

// =============================================================================
// Route Configuration
// =============================================================================

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            // Domains
            .route("/domains", web::get().to(list_domains))
            .route("/domains", web::post().to(create_domain))
            .route("/domains/{id}", web::get().to(get_domain))
            .route("/domains/{id}", web::delete().to(delete_domain))
            .route("/domains/{id}/mailboxes", web::get().to(list_mailboxes_by_domain))
            
            // Mailboxes
            .route("/mailboxes", web::get().to(list_mailboxes))
            .route("/mailboxes", web::post().to(create_mailbox))
            
            // Emails/Inbox
            .route("/inbox", web::get().to(get_inbox))
            .route("/emails/{id}", web::get().to(get_email))
            .route("/emails/{id}", web::patch().to(update_email))
            .route("/emails/{id}", web::delete().to(delete_email))
            
            // Analytics
            .route("/stats", web::get().to(get_dashboard_stats))
            .route("/metrics/daily", web::get().to(get_daily_metrics))
            .route("/metrics/hourly", web::get().to(get_hourly_metrics))
            
            // Reviews
            .route("/reviews", web::get().to(get_review_queue))
            .route("/reviews/{id}/decision", web::post().to(submit_review_decision))
            
            // Agents
            .route("/agents", web::get().to(list_agents))
            
            // Config
            .route("/config/scheduler", web::get().to(get_scheduler_config))
            .route("/config/server", web::get().to(get_server_config))
    );
}
