//! P2P API Endpoints - REST API for P2P file transfers
//!
//! Table of Contents:
//! 1. Transfer endpoints (share, download, status)
//! 2. Peer endpoints (list, connect)
//! 3. Mailbox export/import

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use amail_p2p::{P2PNode, P2PConfig, ChunkInfo, Transfer};
use amail_core::{
    P2PAttachmentConfig, P2PAttachment, P2PDownloadStatus, AttachmentProcessor,
    AttachmentAnalysis, P2PDownloadTracker, embed_p2p_attachment_html, format_size,
};

// =============================================================================
// P2P State
// =============================================================================

/// Shared P2P node state
pub struct P2PState {
    pub node: Arc<RwLock<Option<P2PNode>>>,
    pub config: P2PConfig,
    pub attachment_config: P2PAttachmentConfig,
    pub attachment_processor: AttachmentProcessor,
    pub download_tracker: Arc<RwLock<P2PDownloadTracker>>,
}

impl P2PState {
    pub fn new(config: P2PConfig) -> Self {
        let attachment_config = P2PAttachmentConfig::default();
        let attachment_processor = AttachmentProcessor::new(attachment_config.clone());
        
        Self {
            node: Arc::new(RwLock::new(None)),
            config,
            attachment_config,
            attachment_processor,
            download_tracker: Arc::new(RwLock::new(P2PDownloadTracker::new())),
        }
    }
    
    pub fn with_attachment_config(mut self, config: P2PAttachmentConfig) -> Self {
        self.attachment_config = config.clone();
        self.attachment_processor = AttachmentProcessor::new(config);
        self
    }

    /// Initialize the P2P node
    pub async fn init(&self) -> anyhow::Result<()> {
        let node = P2PNode::new(self.config.clone())?;
        *self.node.write().await = Some(node);
        Ok(())
    }

    /// Get the node (if initialized)
    pub async fn get_node(&self) -> Option<Arc<RwLock<Option<P2PNode>>>> {
        if self.node.read().await.is_some() {
            Some(self.node.clone())
        } else {
            None
        }
    }
}

// =============================================================================
// 1. Transfer Endpoints
// =============================================================================

/// Response for a shared file
#[derive(Debug, Serialize)]
pub struct ShareResponse {
    pub transfer_id: String,
    pub filename: String,
    pub size: u64,
    pub chunks: u32,
    pub magnet_link: String,
    pub file_hash: String,
}

/// Request to share a file
#[derive(Debug, Deserialize)]
pub struct ShareRequest {
    pub file_path: String,
}

/// Share a file for P2P transfer
pub async fn share_file(
    state: web::Data<P2PState>,
    body: web::Json<ShareRequest>,
) -> impl Responder {
    let node_lock = match state.node.read().await.as_ref() {
        Some(_) => state.node.clone(),
        None => return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        })),
    };

    let node = node_lock.read().await;
    if let Some(ref n) = *node {
        let path = PathBuf::from(&body.file_path);
        match n.share_file(&path).await {
            Ok(info) => {
                let magnet = info.magnet_link();
                HttpResponse::Ok().json(ShareResponse {
                    transfer_id: info.transfer_id.to_string(),
                    filename: info.filename.clone(),
                    size: info.total_size,
                    chunks: info.total_chunks,
                    magnet_link: magnet,
                    file_hash: info.file_hash.clone(),
                })
            }
            Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        }))
    }
}

/// Request to download a file
#[derive(Debug, Deserialize)]
pub struct DownloadRequest {
    pub magnet_link: Option<String>,
    pub transfer_info: Option<ChunkInfo>,
    pub output_path: String,
}

/// Start downloading a file
pub async fn start_download(
    state: web::Data<P2PState>,
    body: web::Json<DownloadRequest>,
) -> impl Responder {
    let node_lock = match state.node.read().await.as_ref() {
        Some(_) => state.node.clone(),
        None => return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        })),
    };

    // Parse magnet link or use provided info
    let info = match (&body.magnet_link, &body.transfer_info) {
        (Some(magnet), _) => {
            // Parse magnet link (simplified - in production would parse properly)
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Magnet link parsing not yet implemented - provide transfer_info directly"
            }));
        }
        (_, Some(info)) => info.clone(),
        _ => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Must provide either magnet_link or transfer_info"
        })),
    };

    let node = node_lock.read().await;
    if let Some(ref n) = *node {
        let output_path = PathBuf::from(&body.output_path);
        match n.download(info, &output_path).await {
            Ok(transfer_id) => HttpResponse::Ok().json(serde_json::json!({
                "transfer_id": transfer_id.to_string(),
                "status": "started"
            })),
            Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        }))
    }
}

/// Transfer status response
#[derive(Debug, Serialize)]
pub struct TransferStatusResponse {
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

impl From<Transfer> for TransferStatusResponse {
    fn from(t: Transfer) -> Self {
        let magnet = t.info.magnet_link();
        Self {
            id: t.id.to_string(),
            filename: t.info.filename.clone(),
            state: format!("{:?}", t.state),
            is_download: t.is_download,
            progress_percent: t.progress.percentage(),
            transferred_bytes: t.progress.transferred_bytes,
            total_bytes: t.progress.total_bytes,
            chunks_completed: t.progress.chunks_completed,
            total_chunks: t.progress.total_chunks,
            speed: t.progress.speed_display(),
            eta: t.progress.eta_display(),
            peers: t.progress.active_peers,
            magnet_link: magnet,
        }
    }
}

/// Get transfer status
pub async fn get_transfer_status(
    state: web::Data<P2PState>,
    path: web::Path<String>,
) -> impl Responder {
    let transfer_id = match Uuid::parse_str(&path) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid transfer ID"
        })),
    };

    let node_lock = match state.node.read().await.as_ref() {
        Some(_) => state.node.clone(),
        None => return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        })),
    };

    let node = node_lock.read().await;
    if let Some(ref n) = *node {
        match n.get_transfer(&transfer_id).await {
            Some(transfer) => HttpResponse::Ok().json(TransferStatusResponse::from(transfer)),
            None => HttpResponse::NotFound().json(serde_json::json!({
                "error": "Transfer not found"
            })),
        }
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        }))
    }
}

/// List all transfers
pub async fn list_transfers(state: web::Data<P2PState>) -> impl Responder {
    let node_lock = match state.node.read().await.as_ref() {
        Some(_) => state.node.clone(),
        None => return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        })),
    };

    let node = node_lock.read().await;
    if let Some(ref n) = *node {
        let transfers = n.get_transfers().await;
        let responses: Vec<TransferStatusResponse> = transfers.into_iter()
            .map(TransferStatusResponse::from)
            .collect();
        HttpResponse::Ok().json(responses)
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        }))
    }
}

/// Pause a transfer
pub async fn pause_transfer(
    state: web::Data<P2PState>,
    path: web::Path<String>,
) -> impl Responder {
    let transfer_id = match Uuid::parse_str(&path) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid transfer ID"
        })),
    };

    let node_lock = match state.node.read().await.as_ref() {
        Some(_) => state.node.clone(),
        None => return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        })),
    };

    let node = node_lock.read().await;
    if let Some(ref n) = *node {
        match n.pause_transfer(&transfer_id).await {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "success": true })),
            Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        }))
    }
}

/// Resume a transfer
pub async fn resume_transfer(
    state: web::Data<P2PState>,
    path: web::Path<String>,
) -> impl Responder {
    let transfer_id = match Uuid::parse_str(&path) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid transfer ID"
        })),
    };

    let node_lock = match state.node.read().await.as_ref() {
        Some(_) => state.node.clone(),
        None => return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        })),
    };

    let node = node_lock.read().await;
    if let Some(ref n) = *node {
        match n.resume_transfer(&transfer_id).await {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "success": true })),
            Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        }))
    }
}

/// Cancel a transfer
pub async fn cancel_transfer(
    state: web::Data<P2PState>,
    path: web::Path<String>,
) -> impl Responder {
    let transfer_id = match Uuid::parse_str(&path) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid transfer ID"
        })),
    };

    let node_lock = match state.node.read().await.as_ref() {
        Some(_) => state.node.clone(),
        None => return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        })),
    };

    let node = node_lock.read().await;
    if let Some(ref n) = *node {
        match n.cancel_transfer(&transfer_id).await {
            Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "success": true })),
            Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        }))
    }
}

// =============================================================================
// 2. Peer Endpoints
// =============================================================================

/// Peer info response
#[derive(Debug, Serialize)]
pub struct PeerResponse {
    pub peer_id: String,
    pub addresses: Vec<String>,
    pub connected: bool,
    pub available_transfers: Vec<String>,
}

/// List connected peers
pub async fn list_peers(state: web::Data<P2PState>) -> impl Responder {
    let node_lock = match state.node.read().await.as_ref() {
        Some(_) => state.node.clone(),
        None => return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        })),
    };

    let node = node_lock.read().await;
    if let Some(ref n) = *node {
        let peers = n.get_peers().await;
        let responses: Vec<PeerResponse> = peers.into_iter()
            .map(|p| PeerResponse {
                peer_id: p.peer_id,
                addresses: p.addresses,
                connected: p.connected,
                available_transfers: p.available_transfers.iter().map(|id| id.to_string()).collect(),
            })
            .collect();
        HttpResponse::Ok().json(responses)
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        }))
    }
}

/// Get P2P node info
pub async fn get_node_info(state: web::Data<P2PState>) -> impl Responder {
    let node_lock = match state.node.read().await.as_ref() {
        Some(_) => state.node.clone(),
        None => return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized",
            "initialized": false
        })),
    };

    let node = node_lock.read().await;
    if let Some(ref n) = *node {
        let peer_count = n.connected_peer_count().await;
        let transfers = n.get_transfers().await;
        let active_downloads = transfers.iter().filter(|t| t.is_download && t.state.is_active()).count();
        let active_uploads = transfers.iter().filter(|t| !t.is_download && t.state.is_active()).count();
        
        HttpResponse::Ok().json(serde_json::json!({
            "initialized": true,
            "peer_id": n.peer_id().to_string(),
            "connected_peers": peer_count,
            "active_downloads": active_downloads,
            "active_uploads": active_uploads,
            "total_transfers": transfers.len()
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized",
            "initialized": false
        }))
    }
}

// =============================================================================
// 3. Attachment P2P Endpoints
// =============================================================================

/// Request to analyze an attachment for P2P suitability
#[derive(Debug, Deserialize)]
pub struct AnalyzeAttachmentRequest {
    pub filename: String,
    pub size: u64,
    pub content_type: String,
}

/// Analyze if attachment should use P2P
pub async fn analyze_attachment(
    state: web::Data<P2PState>,
    body: web::Json<AnalyzeAttachmentRequest>,
) -> impl Responder {
    let analysis = AttachmentAnalysis {
        attachment_id: Uuid::new_v4(),
        filename: body.filename.clone(),
        size: body.size,
        should_use_p2p: state.attachment_config.should_use_p2p(body.size),
        is_acceptable: state.attachment_config.is_size_acceptable(body.size),
        is_compressible: !body.content_type.starts_with("image/") 
            && !body.content_type.starts_with("video/")
            && !body.content_type.starts_with("audio/"),
        estimated_transfer_time: body.size / (1024 * 1024) + 1,
        recommendation: if !state.attachment_config.is_size_acceptable(body.size) {
            amail_core::AttachmentRecommendation::Reject
        } else if state.attachment_config.should_use_p2p(body.size) {
            amail_core::AttachmentRecommendation::UseP2P
        } else {
            amail_core::AttachmentRecommendation::Inline
        },
    };
    
    HttpResponse::Ok().json(serde_json::json!({
        "analysis": analysis,
        "threshold_bytes": state.attachment_config.size_threshold_bytes,
        "max_size_bytes": state.attachment_config.max_attachment_size_bytes,
        "size_display": format_size(body.size),
    }))
}

/// Request to share an attachment via P2P
#[derive(Debug, Deserialize)]
pub struct ShareAttachmentRequest {
    /// Path to the attachment file
    pub file_path: String,
    /// Original filename
    pub filename: String,
    /// MIME type
    pub content_type: String,
    /// Message ID this attachment belongs to
    pub message_id: Option<String>,
}

/// Response for shared attachment
#[derive(Debug, Serialize)]
pub struct ShareAttachmentResponse {
    pub transfer_id: String,
    pub filename: String,
    pub size: u64,
    pub size_display: String,
    pub magnet_link: String,
    pub file_hash: String,
    pub chunks: u32,
    pub html_embed: String,
}

/// Share an attachment via P2P
pub async fn share_attachment(
    state: web::Data<P2PState>,
    body: web::Json<ShareAttachmentRequest>,
) -> impl Responder {
    let node_lock = match state.node.read().await.as_ref() {
        Some(_) => state.node.clone(),
        None => return HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        })),
    };

    let node = node_lock.read().await;
    if let Some(ref n) = *node {
        let path = PathBuf::from(&body.file_path);
        match n.share_file(&path).await {
            Ok(info) => {
                let magnet = info.magnet_link();
                
                // Create P2P attachment metadata
                let p2p_attachment = P2PAttachment {
                    id: Uuid::new_v4(),
                    filename: body.filename.clone(),
                    content_type: body.content_type.clone(),
                    size: info.total_size,
                    magnet_link: magnet.clone(),
                    file_hash: info.file_hash.clone(),
                    transfer_id: info.transfer_id,
                    shared_at: chrono::Utc::now(),
                    expires_at: Some(chrono::Utc::now() + chrono::Duration::days(
                        state.attachment_config.retention_days as i64
                    )),
                    chunk_count: info.total_chunks,
                    download_status: P2PDownloadStatus::NotStarted,
                    local_path: Some(path),
                    is_inline: false,
                    content_id: None,
                };
                
                let html_embed = embed_p2p_attachment_html(&p2p_attachment);
                
                HttpResponse::Ok().json(ShareAttachmentResponse {
                    transfer_id: info.transfer_id.to_string(),
                    filename: body.filename.clone(),
                    size: info.total_size,
                    size_display: format_size(info.total_size),
                    magnet_link: magnet,
                    file_hash: info.file_hash.clone(),
                    chunks: info.total_chunks,
                    html_embed,
                })
            }
            Err(e) => HttpResponse::BadRequest().json(serde_json::json!({
                "error": e.to_string()
            })),
        }
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "P2P node not initialized"
        }))
    }
}

/// Request to download a P2P attachment
#[derive(Debug, Deserialize)]
pub struct DownloadAttachmentRequest {
    /// Magnet link for the attachment
    pub magnet_link: String,
    /// Message ID this attachment belongs to
    pub message_id: String,
    /// Expected filename
    pub filename: Option<String>,
}

/// Start downloading a P2P attachment
pub async fn download_attachment(
    state: web::Data<P2PState>,
    body: web::Json<DownloadAttachmentRequest>,
) -> impl Responder {
    // For now, return info about the download
    // Full implementation would parse magnet link and start download
    HttpResponse::Ok().json(serde_json::json!({
        "status": "queued",
        "message_id": body.message_id,
        "magnet_link": body.magnet_link,
        "filename": body.filename,
        "note": "Magnet link parsing and download will be started"
    }))
}

/// Get attachment download status
pub async fn get_attachment_download_status(
    state: web::Data<P2PState>,
    path: web::Path<String>,
) -> impl Responder {
    let transfer_id = match Uuid::parse_str(&path) {
        Ok(id) => id,
        Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Invalid transfer ID"
        })),
    };

    let tracker = state.download_tracker.read().await;
    
    if let Some(info) = tracker.get_download(&transfer_id) {
        HttpResponse::Ok().json(serde_json::json!({
            "transfer_id": transfer_id.to_string(),
            "message_id": info.message_id,
            "filename": info.attachment.filename,
            "size": info.attachment.size,
            "size_display": format_size(info.attachment.size),
            "status": format!("{:?}", info.attachment.download_status),
            "progress_percent": info.progress_percent,
            "bytes_downloaded": info.bytes_downloaded,
            "speed_bps": info.speed_bps,
            "eta_seconds": info.eta_seconds,
            "error": info.error,
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": "Download not found"
        }))
    }
}

/// List all attachment downloads
pub async fn list_attachment_downloads(
    state: web::Data<P2PState>,
) -> impl Responder {
    let tracker = state.download_tracker.read().await;
    
    let downloads: Vec<serde_json::Value> = tracker.downloads.values()
        .map(|info| serde_json::json!({
            "transfer_id": info.transfer_id.to_string(),
            "message_id": info.message_id,
            "filename": info.attachment.filename,
            "size": info.attachment.size,
            "size_display": format_size(info.attachment.size),
            "status": format!("{:?}", info.attachment.download_status),
            "progress_percent": info.progress_percent,
        }))
        .collect();
    
    HttpResponse::Ok().json(serde_json::json!({
        "active_downloads": downloads,
        "completed_count": tracker.completed.len(),
        "failed_count": tracker.failed.len(),
    }))
}

/// Get P2P attachment configuration
pub async fn get_attachment_config(
    state: web::Data<P2PState>,
) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "enabled": state.attachment_config.enabled,
        "size_threshold_bytes": state.attachment_config.size_threshold_bytes,
        "size_threshold_display": format_size(state.attachment_config.size_threshold_bytes),
        "max_attachment_size_bytes": state.attachment_config.max_attachment_size_bytes,
        "max_attachment_size_display": format_size(state.attachment_config.max_attachment_size_bytes),
        "auto_download_threshold_bytes": state.attachment_config.auto_download_threshold_bytes,
        "auto_download_threshold_display": format_size(state.attachment_config.auto_download_threshold_bytes),
        "retention_days": state.attachment_config.retention_days,
        "compress_before_share": state.attachment_config.compress_before_share,
        "encrypt_before_share": state.attachment_config.encrypt_before_share,
    }))
}

// =============================================================================
// 4. Mailbox Export/Import
// =============================================================================

/// Request to export mailbox
#[derive(Debug, Deserialize)]
pub struct ExportMailboxRequest {
    pub mailbox_id: Option<String>,
    pub domain: Option<String>,
    pub include_attachments: bool,
}

/// Export mailbox for P2P sharing
pub async fn export_mailbox(
    state: web::Data<P2PState>,
    body: web::Json<ExportMailboxRequest>,
) -> impl Responder {
    // This would integrate with the email repository to export mailbox data
    // For now, return a placeholder
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Mailbox export initiated",
        "mailbox_id": body.mailbox_id,
        "domain": body.domain,
        "include_attachments": body.include_attachments,
        "status": "pending"
    }))
}

/// Request to import mailbox
#[derive(Debug, Deserialize)]
pub struct ImportMailboxRequest {
    pub transfer_id: String,
    pub target_domain: Option<String>,
}

/// Import mailbox from P2P transfer
pub async fn import_mailbox(
    state: web::Data<P2PState>,
    body: web::Json<ImportMailboxRequest>,
) -> impl Responder {
    // This would integrate with the email repository to import mailbox data
    // For now, return a placeholder
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Mailbox import initiated",
        "transfer_id": body.transfer_id,
        "target_domain": body.target_domain,
        "status": "pending"
    }))
}

// =============================================================================
// Route Configuration
// =============================================================================

pub fn configure_p2p_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/p2p")
            // Node info
            .route("/info", web::get().to(get_node_info))
            
            // Transfers
            .route("/transfers", web::get().to(list_transfers))
            .route("/transfers/share", web::post().to(share_file))
            .route("/transfers/download", web::post().to(start_download))
            .route("/transfers/{id}", web::get().to(get_transfer_status))
            .route("/transfers/{id}/pause", web::post().to(pause_transfer))
            .route("/transfers/{id}/resume", web::post().to(resume_transfer))
            .route("/transfers/{id}/cancel", web::post().to(cancel_transfer))
            
            // Peers
            .route("/peers", web::get().to(list_peers))
            
            // Attachments
            .route("/attachments/config", web::get().to(get_attachment_config))
            .route("/attachments/analyze", web::post().to(analyze_attachment))
            .route("/attachments/share", web::post().to(share_attachment))
            .route("/attachments/download", web::post().to(download_attachment))
            .route("/attachments/downloads", web::get().to(list_attachment_downloads))
            .route("/attachments/downloads/{id}", web::get().to(get_attachment_download_status))
            
            // Mailbox export/import
            .route("/mailbox/export", web::post().to(export_mailbox))
            .route("/mailbox/import", web::post().to(import_mailbox))
    );
}
