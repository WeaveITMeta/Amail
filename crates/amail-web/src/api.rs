//! API Client - Fetch real data from backend
//!
//! Table of Contents:
//! 1. API base URL configuration
//! 2. Data types matching backend responses
//! 3. Fetch functions for each endpoint

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

// =============================================================================
// 1. API Configuration
// =============================================================================

/// Get the API base URL (defaults to backend server)
pub fn api_base_url() -> String {
    // Check if we're on the same port as the backend (4000)
    // If not, explicitly use the backend URL
    if let Some(window) = web_sys::window() {
        if let Ok(origin) = window.location().origin() {
            // If running on port 4000, use same origin (production)
            if origin.contains(":4000") {
                return origin;
            }
        }
    }
    // Development: frontend on different port, backend on 4000
    "http://localhost:4000".to_string()
}

// =============================================================================
// 2. Data Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardStats {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainResponse {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub enabled: bool,
    pub primary: bool,
    pub created_at: String,
    // Optional fields for backward compatibility with database API
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub mx_verified: bool,
    #[serde(default)]
    pub spf_verified: bool,
    #[serde(default)]
    pub dkim_verified: bool,
    #[serde(default)]
    pub dmarc_verified: bool,
    #[serde(default)]
    pub mailbox_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxResponse {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    #[serde(alias = "domain_id")]
    pub domain: String,
    pub mailbox_type: String,
    #[serde(default, alias = "enabled")]
    pub is_active: bool,
    #[serde(default)]
    pub unread_count: i64,
    #[serde(default)]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDetailResponse {
    pub id: String,
    pub from_address: String,
    pub from_name: Option<String>,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub subject: String,
    pub text_body: Option<String>,
    pub html_body: Option<String>,
    pub received_at: String,
    pub is_read: bool,
    pub is_starred: bool,
    pub folder: String,
    pub labels: Vec<String>,
    pub attachments: Vec<AttachmentResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentResponse {
    pub id: String,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    /// P2P attachment fields (optional - only present for P2P attachments)
    #[serde(default)]
    pub is_p2p: bool,
    pub magnet_link: Option<String>,
    pub file_hash: Option<String>,
    pub transfer_id: Option<String>,
    pub download_status: Option<String>,
    pub progress_percent: Option<f64>,
}

// =============================================================================
// P2P Attachment Types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PNodeInfo {
    pub initialized: bool,
    pub peer_id: Option<String>,
    pub connected_peers: Option<i64>,
    pub active_downloads: Option<i64>,
    pub active_uploads: Option<i64>,
    pub total_transfers: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PAttachmentConfig {
    pub enabled: bool,
    pub size_threshold_bytes: u64,
    pub size_threshold_display: String,
    pub max_attachment_size_bytes: u64,
    pub max_attachment_size_display: String,
    pub auto_download_threshold_bytes: u64,
    pub auto_download_threshold_display: String,
    pub retention_days: u32,
    pub compress_before_share: bool,
    pub encrypt_before_share: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PTransferStatus {
    pub id: String,
    pub filename: String,
    pub state: String,
    pub is_download: bool,
    pub progress_percent: f64,
    pub transferred_bytes: u64,
    pub total_bytes: u64,
    pub chunks_completed: u32,
    pub total_chunks: u32,
    pub speed: String,
    pub eta: String,
    pub peers: usize,
    pub magnet_link: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PAttachmentAnalysis {
    pub attachment_id: String,
    pub filename: String,
    pub size: u64,
    pub should_use_p2p: bool,
    pub is_acceptable: bool,
    pub is_compressible: bool,
    pub estimated_transfer_time: u64,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PShareResponse {
    pub transfer_id: String,
    pub filename: String,
    pub size: u64,
    pub size_display: String,
    pub magnet_link: String,
    pub file_hash: String,
    pub chunks: u32,
    pub html_embed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PDownloadInfo {
    pub transfer_id: String,
    pub message_id: String,
    pub filename: String,
    pub size: u64,
    pub size_display: String,
    pub status: String,
    pub progress_percent: f64,
    pub bytes_downloaded: Option<u64>,
    pub speed_bps: Option<u64>,
    pub eta_seconds: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub id: String,
    pub role: String,
    pub status: String,
    pub last_heartbeat: String,
    pub tasks_processed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyMetricsResponse {
    pub date: String,
    pub emails_received: i64,
    pub emails_sent: i64,
    pub emails_auto_replied: i64,
    pub tasks_created: i64,
    pub tasks_completed: i64,
    pub tasks_failed: i64,
    pub avg_response_time_ms: Option<i64>,
    pub reviews_requested: i64,
    pub reviews_approved: i64,
    pub reviews_rejected: i64,
    pub llm_calls: i64,
    pub llm_tokens_used: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyMetricsResponse {
    pub hour: String,
    pub emails_received: i32,
    pub emails_sent: i32,
    pub tasks_completed: i32,
    pub avg_response_time_ms: Option<i32>,
}

// =============================================================================
// 3. Fetch Functions
// =============================================================================

/// Fetch dashboard stats
pub async fn fetch_stats() -> Result<DashboardStats, String> {
    let url = format!("{}/api/v1/stats", api_base_url());
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<DashboardStats>()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch all domains
pub async fn fetch_domains() -> Result<Vec<DomainResponse>, String> {
    let url = format!("{}/api/config/domains", api_base_url());
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<DomainResponse>>()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch all mailboxes
pub async fn fetch_mailboxes() -> Result<Vec<MailboxResponse>, String> {
    let url = format!("{}/api/config/mailboxes", api_base_url());
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<MailboxResponse>>()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch mailboxes for a specific domain
pub async fn fetch_mailboxes_by_domain(domain_id: &str) -> Result<Vec<MailboxResponse>, String> {
    let url = format!("{}/api/config/domains/{}/mailboxes", api_base_url(), domain_id);
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<MailboxResponse>>()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch inbox emails
pub async fn fetch_inbox(
    mailbox_id: Option<&str>,
    folder: Option<&str>,
    search: Option<&str>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<InboxEmailResponse>, String> {
    let mut url = format!("{}/api/v1/inbox?", api_base_url());
    
    if let Some(id) = mailbox_id {
        url.push_str(&format!("mailbox_id={}&", id));
    }
    if let Some(f) = folder {
        url.push_str(&format!("folder={}&", f));
    }
    if let Some(s) = search {
        url.push_str(&format!("search={}&", s));
    }
    if let Some(l) = limit {
        url.push_str(&format!("limit={}&", l));
    }
    if let Some(o) = offset {
        url.push_str(&format!("offset={}&", o));
    }
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<InboxEmailResponse>>()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch single email details
pub async fn fetch_email(email_id: &str) -> Result<EmailDetailResponse, String> {
    let url = format!("{}/api/v1/emails/{}", api_base_url(), email_id);
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<EmailDetailResponse>()
        .await
        .map_err(|e| e.to_string())
}

/// Update email (mark read, star, move folder, etc.)
pub async fn update_email(
    email_id: &str,
    is_read: Option<bool>,
    is_starred: Option<bool>,
    folder: Option<&str>,
) -> Result<(), String> {
    let url = format!("{}/api/v1/emails/{}", api_base_url(), email_id);
    
    #[derive(Serialize)]
    struct UpdateRequest {
        #[serde(skip_serializing_if = "Option::is_none")]
        is_read: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_starred: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        folder: Option<String>,
    }
    
    let body = UpdateRequest {
        is_read,
        is_starred,
        folder: folder.map(|s| s.to_string()),
    };
    
    Request::patch(&url)
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Fetch review queue
pub async fn fetch_reviews() -> Result<Vec<ReviewResponse>, String> {
    let url = format!("{}/api/v1/reviews", api_base_url());
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<ReviewResponse>>()
        .await
        .map_err(|e| e.to_string())
}

/// Submit review decision
pub async fn submit_review_decision(
    review_id: &str,
    decision: &str,
    reviewer_notes: Option<&str>,
    edited_response: Option<&str>,
) -> Result<(), String> {
    let url = format!("{}/api/v1/reviews/{}/decision", api_base_url(), review_id);
    
    #[derive(Serialize)]
    struct DecisionRequest {
        decision: String,
        reviewed_by: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        reviewer_notes: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        edited_response: Option<String>,
    }
    
    let body = DecisionRequest {
        decision: decision.to_string(),
        reviewed_by: "dashboard_user".to_string(),
        reviewer_notes: reviewer_notes.map(|s| s.to_string()),
        edited_response: edited_response.map(|s| s.to_string()),
    };
    
    Request::post(&url)
        .json(&body)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Fetch all agents
pub async fn fetch_agents() -> Result<Vec<AgentResponse>, String> {
    let url = format!("{}/api/v1/agents", api_base_url());
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<AgentResponse>>()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch daily metrics
pub async fn fetch_daily_metrics(period: Option<&str>) -> Result<Vec<DailyMetricsResponse>, String> {
    let mut url = format!("{}/api/v1/metrics/daily", api_base_url());
    
    if let Some(p) = period {
        url.push_str(&format!("?period={}", p));
    }
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<DailyMetricsResponse>>()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch hourly metrics
pub async fn fetch_hourly_metrics(period: Option<&str>) -> Result<Vec<HourlyMetricsResponse>, String> {
    let mut url = format!("{}/api/v1/metrics/hourly", api_base_url());
    
    if let Some(p) = period {
        url.push_str(&format!("?period={}", p));
    }
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<HourlyMetricsResponse>>()
        .await
        .map_err(|e| e.to_string())
}

/// Create a new domain
pub async fn create_domain(name: &str) -> Result<DomainResponse, String> {
    let url = format!("{}/api/config/domains", api_base_url());
    
    #[derive(Serialize)]
    struct CreateDomainRequest {
        name: String,
    }
    
    let response = Request::post(&url)
        .json(&CreateDomainRequest { name: name.to_string() })
        .map_err(|e| format!("Failed to build request: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}. Is the backend running on {}?", e, api_base_url()))?;
    
    // Check HTTP status before parsing
    if response.status() >= 400 {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Server error (HTTP {}): {}", response.status(), error_text));
    }
    
    // Check if response is empty
    let text = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
    if text.is_empty() {
        return Err("Server returned empty response. Is the backend running?".to_string());
    }
    
    serde_json::from_str::<DomainResponse>(&text)
        .map_err(|e| format!("Failed to parse response: {}. Response was: {}", e, &text[..text.len().min(200)]))
}

/// Delete a domain
pub async fn delete_domain(domain_id: &str) -> Result<(), String> {
    let url = format!("{}/api/config/domains/{}", api_base_url(), domain_id);
    
    let response = Request::delete(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if response.status() >= 400 {
        return Err(format!("Failed to delete domain: HTTP {}", response.status()));
    }
    
    Ok(())
}

/// Create a new mailbox
pub async fn create_mailbox(
    domain_id: &str,
    local_part: &str,
    display_name: Option<&str>,
) -> Result<(), String> {
    let url = format!("{}/api/config/mailboxes", api_base_url());
    
    #[derive(Serialize)]
    struct CreateMailboxRequest {
        domain_id: String,
        local_part: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        display_name: Option<String>,
    }
    
    Request::post(&url)
        .json(&CreateMailboxRequest {
            domain_id: domain_id.to_string(),
            local_part: local_part.to_string(),
            display_name: display_name.map(|s| s.to_string()),
        })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Delete a mailbox
pub async fn delete_mailbox(mailbox_id: &str) -> Result<(), String> {
    let url = format!("{}/api/config/mailboxes/{}", api_base_url(), mailbox_id);
    
    let response = Request::delete(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if response.status() >= 400 {
        return Err(format!("Failed to delete mailbox: HTTP {}", response.status()));
    }
    
    Ok(())
}

// =============================================================================
// P2P Attachment API Functions
// =============================================================================

/// Get P2P node info
pub async fn fetch_p2p_info() -> Result<P2PNodeInfo, String> {
    let url = format!("{}/api/v1/p2p/info", api_base_url());
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<P2PNodeInfo>()
        .await
        .map_err(|e| e.to_string())
}

/// Get P2P attachment configuration
pub async fn fetch_p2p_attachment_config() -> Result<P2PAttachmentConfig, String> {
    let url = format!("{}/api/v1/p2p/attachments/config", api_base_url());
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<P2PAttachmentConfig>()
        .await
        .map_err(|e| e.to_string())
}

/// Analyze attachment for P2P suitability
pub async fn analyze_attachment(
    filename: &str,
    size: u64,
    content_type: &str,
) -> Result<P2PAttachmentAnalysis, String> {
    let url = format!("{}/api/v1/p2p/attachments/analyze", api_base_url());
    
    #[derive(Serialize)]
    struct AnalyzeRequest {
        filename: String,
        size: u64,
        content_type: String,
    }
    
    #[derive(Deserialize)]
    struct AnalyzeResponse {
        analysis: P2PAttachmentAnalysis,
    }
    
    let resp = Request::post(&url)
        .json(&AnalyzeRequest {
            filename: filename.to_string(),
            size,
            content_type: content_type.to_string(),
        })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<AnalyzeResponse>()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(resp.analysis)
}

/// Start downloading a P2P attachment
pub async fn start_p2p_download(
    magnet_link: &str,
    message_id: &str,
    filename: Option<&str>,
) -> Result<(), String> {
    let url = format!("{}/api/v1/p2p/attachments/download", api_base_url());
    
    #[derive(Serialize)]
    struct DownloadRequest {
        magnet_link: String,
        message_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
    }
    
    Request::post(&url)
        .json(&DownloadRequest {
            magnet_link: magnet_link.to_string(),
            message_id: message_id.to_string(),
            filename: filename.map(|s| s.to_string()),
        })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Get P2P download status
pub async fn fetch_p2p_download_status(transfer_id: &str) -> Result<P2PDownloadInfo, String> {
    let url = format!("{}/api/v1/p2p/attachments/downloads/{}", api_base_url(), transfer_id);
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<P2PDownloadInfo>()
        .await
        .map_err(|e| e.to_string())
}

/// List all P2P attachment downloads
pub async fn fetch_p2p_downloads() -> Result<Vec<P2PDownloadInfo>, String> {
    let url = format!("{}/api/v1/p2p/attachments/downloads", api_base_url());
    
    #[derive(Deserialize)]
    struct DownloadsResponse {
        active_downloads: Vec<P2PDownloadInfo>,
    }
    
    let resp = Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<DownloadsResponse>()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(resp.active_downloads)
}

/// List all P2P transfers
pub async fn fetch_p2p_transfers() -> Result<Vec<P2PTransferStatus>, String> {
    let url = format!("{}/api/v1/p2p/transfers", api_base_url());
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<P2PTransferStatus>>()
        .await
        .map_err(|e| e.to_string())
}

/// Pause a P2P transfer
pub async fn pause_p2p_transfer(transfer_id: &str) -> Result<(), String> {
    let url = format!("{}/api/v1/p2p/transfers/{}/pause", api_base_url(), transfer_id);
    
    Request::post(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Resume a P2P transfer
pub async fn resume_p2p_transfer(transfer_id: &str) -> Result<(), String> {
    let url = format!("{}/api/v1/p2p/transfers/{}/resume", api_base_url(), transfer_id);
    
    Request::post(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Cancel a P2P transfer
pub async fn cancel_p2p_transfer(transfer_id: &str) -> Result<(), String> {
    let url = format!("{}/api/v1/p2p/transfers/{}/cancel", api_base_url(), transfer_id);
    
    Request::post(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(())
}

// =============================================================================
// DNS Configuration API Functions
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecordResponse {
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub value: String,
    pub priority: Option<u16>,
    pub ttl: u32,
}

/// Fetch DNS records for a domain (copy-paste format) - uses v1 API
pub async fn fetch_dns_records(domain_id: &str) -> Result<Vec<DnsRecordResponse>, String> {
    let url = format!("{}/api/v1/domains/{}/dns", api_base_url(), domain_id);
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<Vec<DnsRecordResponse>>()
        .await
        .map_err(|e| e.to_string())
}

/// Fetch DNS records for config wizard - uses config API (Supabase/JSON backend)
pub async fn fetch_dns_records_for_config(domain_id: &str) -> Result<Vec<DnsRecordResponse>, String> {
    let url = format!("{}/api/config/domains/{}/dns", api_base_url(), domain_id);
    
    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if response.status() >= 400 {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Server error (HTTP {}): {}", response.status(), error_text));
    }
    
    let text = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
    if text.is_empty() {
        return Err("Server returned empty response".to_string());
    }
    
    serde_json::from_str::<Vec<DnsRecordResponse>>(&text)
        .map_err(|e| format!("Failed to parse DNS records: {}", e))
}

/// Fetch DNS records as formatted text for copy-paste
pub async fn fetch_dns_records_text(domain_id: &str) -> Result<String, String> {
    let url = format!("{}/api/v1/domains/{}/dns/text", api_base_url(), domain_id);
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())
}

/// Verify DNS records for a domain - uses config API
pub async fn verify_dns_records(domain_id: &str) -> Result<DnsVerificationResponse, String> {
    let url = format!("{}/api/config/domains/{}/dns/verify", api_base_url(), domain_id);
    
    let response = Request::post(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;
    
    if response.status() >= 400 {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Server error (HTTP {}): {}", response.status(), error_text));
    }
    
    response.json::<DnsVerificationResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsVerificationResponse {
    pub mx_verified: bool,
    pub spf_verified: bool,
    pub dkim_verified: bool,
    pub dmarc_verified: bool,
    #[serde(default)]
    pub message: Option<String>,
}

// =============================================================================
// Workspace Rules API
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRulesResponse {
    pub content: String,
    pub last_modified: Option<String>,
}

/// Fetch workspace rules
pub async fn fetch_workspace_rules() -> Result<WorkspaceRulesResponse, String> {
    let url = format!("{}/api/config/workspace-rules", api_base_url());
    
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<WorkspaceRulesResponse>()
        .await
        .map_err(|e| e.to_string())
}

/// Update workspace rules
pub async fn update_workspace_rules(content: &str) -> Result<(), String> {
    let url = format!("{}/api/config/workspace-rules", api_base_url());
    
    #[derive(Serialize)]
    struct UpdateRequest {
        content: String,
    }
    
    let response = Request::put(&url)
        .json(&UpdateRequest { content: content.to_string() })
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if response.status() >= 400 {
        return Err(format!("Failed to update workspace rules: HTTP {}", response.status()));
    }
    
    Ok(())
}
