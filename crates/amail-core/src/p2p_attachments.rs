// =============================================================================
// P2P Attachment Integration
// =============================================================================
// Table of Contents:
// 1. Configuration - Size thresholds, storage paths
// 2. P2P Attachment Types - Metadata for P2P-shared attachments
// 3. Attachment Processor - Analyze and convert large attachments
// 4. Magnet Link Embedding - Embed/extract magnet links in emails
// 5. Download Manager - Track and manage P2P attachment downloads
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

use crate::email::Attachment;

// =============================================================================
// 1. Configuration
// =============================================================================

/// P2P attachment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PAttachmentConfig {
    /// Enable P2P for large attachments
    pub enabled: bool,
    /// Size threshold in bytes (attachments larger than this use P2P)
    /// Default: 10MB (10 * 1024 * 1024)
    pub size_threshold_bytes: u64,
    /// Maximum attachment size to accept (even via P2P)
    /// Default: 2GB
    pub max_attachment_size_bytes: u64,
    /// Directory to store P2P shared files
    pub storage_path: PathBuf,
    /// How long to keep shared files (days)
    pub retention_days: u32,
    /// Auto-download P2P attachments below this size
    pub auto_download_threshold_bytes: u64,
    /// Compress attachments before P2P sharing
    pub compress_before_share: bool,
    /// Encrypt attachments before P2P sharing
    pub encrypt_before_share: bool,
}

impl Default for P2PAttachmentConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            size_threshold_bytes: 10 * 1024 * 1024, // 10 MB
            max_attachment_size_bytes: 2 * 1024 * 1024 * 1024, // 2 GB
            storage_path: PathBuf::from("./data/p2p_attachments"),
            retention_days: 30,
            auto_download_threshold_bytes: 50 * 1024 * 1024, // 50 MB
            compress_before_share: true,
            encrypt_before_share: false,
        }
    }
}

impl P2PAttachmentConfig {
    /// Check if an attachment should use P2P transfer
    pub fn should_use_p2p(&self, size: u64) -> bool {
        self.enabled && size > self.size_threshold_bytes
    }

    /// Check if attachment size is acceptable
    pub fn is_size_acceptable(&self, size: u64) -> bool {
        size <= self.max_attachment_size_bytes
    }

    /// Check if attachment should auto-download
    pub fn should_auto_download(&self, size: u64) -> bool {
        size <= self.auto_download_threshold_bytes
    }
}

// =============================================================================
// 2. P2P Attachment Types
// =============================================================================

/// A P2P-shared attachment (replaces actual content with magnet link)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PAttachment {
    /// Original attachment ID
    pub id: Uuid,
    /// Original filename
    pub filename: String,
    /// MIME type
    pub content_type: String,
    /// Original size in bytes
    pub size: u64,
    /// Magnet link for P2P download
    pub magnet_link: String,
    /// File hash (for verification)
    pub file_hash: String,
    /// Transfer ID in P2P system
    pub transfer_id: Uuid,
    /// When the file was shared
    pub shared_at: DateTime<Utc>,
    /// When the share expires
    pub expires_at: Option<DateTime<Utc>>,
    /// Number of chunks
    pub chunk_count: u32,
    /// Download status (for received attachments)
    pub download_status: P2PDownloadStatus,
    /// Local file path (if downloaded)
    pub local_path: Option<PathBuf>,
    /// Is this an inline attachment
    pub is_inline: bool,
    /// Content-ID for inline attachments
    pub content_id: Option<String>,
}

/// Download status for P2P attachments
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum P2PDownloadStatus {
    #[default]
    NotStarted,
    Queued,
    Downloading,
    Paused,
    Completed,
    Failed,
    Expired,
}

impl P2PDownloadStatus {
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Completed)
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Queued | Self::Downloading)
    }

    pub fn can_retry(&self) -> bool {
        matches!(self, Self::Failed | Self::NotStarted | Self::Paused)
    }
}

/// Attachment that may be regular or P2P
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailAttachment {
    /// Regular inline attachment
    Regular(Attachment),
    /// P2P shared attachment (large file)
    P2P(P2PAttachment),
}

impl EmailAttachment {
    pub fn id(&self) -> Uuid {
        match self {
            Self::Regular(a) => a.id,
            Self::P2P(a) => a.id,
        }
    }

    pub fn filename(&self) -> &str {
        match self {
            Self::Regular(a) => &a.filename,
            Self::P2P(a) => &a.filename,
        }
    }

    pub fn size(&self) -> u64 {
        match self {
            Self::Regular(a) => a.size as u64,
            Self::P2P(a) => a.size,
        }
    }

    pub fn content_type(&self) -> &str {
        match self {
            Self::Regular(a) => &a.content_type,
            Self::P2P(a) => &a.content_type,
        }
    }

    pub fn is_p2p(&self) -> bool {
        matches!(self, Self::P2P(_))
    }

    pub fn is_inline(&self) -> bool {
        match self {
            Self::Regular(a) => a.is_inline,
            Self::P2P(a) => a.is_inline,
        }
    }

    /// Get magnet link if P2P attachment
    pub fn magnet_link(&self) -> Option<&str> {
        match self {
            Self::P2P(a) => Some(&a.magnet_link),
            _ => None,
        }
    }

    /// Check if download is complete (always true for regular)
    pub fn is_available(&self) -> bool {
        match self {
            Self::Regular(_) => true,
            Self::P2P(a) => a.download_status.is_complete(),
        }
    }
}

// =============================================================================
// 3. Attachment Processor
// =============================================================================

/// Result of processing an attachment for P2P
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PProcessResult {
    /// Original attachment ID
    pub attachment_id: Uuid,
    /// Whether P2P was used
    pub used_p2p: bool,
    /// The resulting attachment (regular or P2P)
    pub attachment: EmailAttachment,
    /// Processing message
    pub message: String,
}

/// Attachment processor for P2P integration
#[derive(Debug, Clone)]
pub struct AttachmentProcessor {
    pub config: P2PAttachmentConfig,
}

impl AttachmentProcessor {
    pub fn new(config: P2PAttachmentConfig) -> Self {
        Self { config }
    }

    /// Analyze an attachment and determine if it should use P2P
    pub fn analyze(&self, attachment: &Attachment) -> AttachmentAnalysis {
        let size = attachment.size as u64;
        
        AttachmentAnalysis {
            attachment_id: attachment.id,
            filename: attachment.filename.clone(),
            size,
            should_use_p2p: self.config.should_use_p2p(size),
            is_acceptable: self.config.is_size_acceptable(size),
            is_compressible: self.is_compressible(&attachment.content_type),
            estimated_transfer_time: self.estimate_transfer_time(size),
            recommendation: if !self.config.is_size_acceptable(size) {
                AttachmentRecommendation::Reject
            } else if self.config.should_use_p2p(size) {
                AttachmentRecommendation::UseP2P
            } else {
                AttachmentRecommendation::Inline
            },
        }
    }

    /// Check if content type is compressible
    fn is_compressible(&self, content_type: &str) -> bool {
        // Already compressed formats
        let compressed = [
            "application/zip",
            "application/gzip",
            "application/x-rar",
            "application/x-7z-compressed",
            "image/jpeg",
            "image/png",
            "image/gif",
            "image/webp",
            "video/mp4",
            "video/webm",
            "audio/mp3",
            "audio/mpeg",
            "audio/ogg",
        ];
        
        !compressed.iter().any(|&ct| content_type.starts_with(ct))
    }

    /// Estimate transfer time in seconds (rough estimate)
    fn estimate_transfer_time(&self, size: u64) -> u64 {
        // Assume 1 MB/s average P2P speed
        size / (1024 * 1024) + 1
    }

    /// Convert a regular attachment to P2P attachment metadata
    /// Note: Actual file sharing happens via P2P node
    pub fn create_p2p_metadata(
        &self,
        attachment: &Attachment,
        magnet_link: String,
        file_hash: String,
        transfer_id: Uuid,
        chunk_count: u32,
    ) -> P2PAttachment {
        let expires_at = if self.config.retention_days > 0 {
            Some(Utc::now() + chrono::Duration::days(self.config.retention_days as i64))
        } else {
            None
        };

        P2PAttachment {
            id: attachment.id,
            filename: attachment.filename.clone(),
            content_type: attachment.content_type.clone(),
            size: attachment.size as u64,
            magnet_link,
            file_hash,
            transfer_id,
            shared_at: Utc::now(),
            expires_at,
            chunk_count,
            download_status: P2PDownloadStatus::NotStarted,
            local_path: None,
            is_inline: attachment.is_inline,
            content_id: attachment.content_id.clone(),
        }
    }
}

/// Analysis result for an attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentAnalysis {
    pub attachment_id: Uuid,
    pub filename: String,
    pub size: u64,
    pub should_use_p2p: bool,
    pub is_acceptable: bool,
    pub is_compressible: bool,
    pub estimated_transfer_time: u64,
    pub recommendation: AttachmentRecommendation,
}

/// Recommendation for handling an attachment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttachmentRecommendation {
    /// Include inline in email (small file)
    Inline,
    /// Use P2P transfer (large file)
    UseP2P,
    /// Reject (too large)
    Reject,
}

// =============================================================================
// 4. Magnet Link Embedding
// =============================================================================

/// Marker for P2P attachments in email body
const P2P_ATTACHMENT_MARKER: &str = "<!-- P2P_ATTACHMENT:";
const P2P_ATTACHMENT_END: &str = " -->";

/// Embed P2P attachment info in email HTML body
pub fn embed_p2p_attachment_html(attachment: &P2PAttachment) -> String {
    let size_display = format_size(attachment.size);
    
    format!(
        r#"<div class="p2p-attachment" data-magnet="{magnet}" data-hash="{hash}" data-transfer-id="{transfer_id}">
    <div class="p2p-attachment-icon">📎</div>
    <div class="p2p-attachment-info">
        <div class="p2p-attachment-name">{filename}</div>
        <div class="p2p-attachment-size">{size} • P2P Transfer</div>
    </div>
    <a href="{magnet}" class="p2p-download-btn" title="Download via P2P">
        ⬇️ Download
    </a>
</div>
{marker}{json}{end}"#,
        magnet = attachment.magnet_link,
        hash = attachment.file_hash,
        transfer_id = attachment.transfer_id,
        filename = html_escape(&attachment.filename),
        size = size_display,
        marker = P2P_ATTACHMENT_MARKER,
        json = serde_json::to_string(attachment).unwrap_or_default(),
        end = P2P_ATTACHMENT_END,
    )
}

/// Embed P2P attachment info in plain text email body
pub fn embed_p2p_attachment_text(attachment: &P2PAttachment) -> String {
    let size_display = format_size(attachment.size);
    
    format!(
        r#"
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📎 P2P Attachment: {filename}
   Size: {size}
   Download: {magnet}
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
"#,
        filename = attachment.filename,
        size = size_display,
        magnet = attachment.magnet_link,
    )
}

/// Extract P2P attachment metadata from email HTML body
pub fn extract_p2p_attachments(html_body: &str) -> Vec<P2PAttachment> {
    let mut attachments = Vec::new();
    
    let mut search_pos = 0;
    while let Some(start) = html_body[search_pos..].find(P2P_ATTACHMENT_MARKER) {
        let abs_start = search_pos + start + P2P_ATTACHMENT_MARKER.len();
        if let Some(end) = html_body[abs_start..].find(P2P_ATTACHMENT_END) {
            let json = &html_body[abs_start..abs_start + end];
            if let Ok(attachment) = serde_json::from_str::<P2PAttachment>(json) {
                attachments.push(attachment);
            }
            search_pos = abs_start + end;
        } else {
            break;
        }
    }
    
    attachments
}

/// Extract magnet links from email body
pub fn extract_magnet_links(body: &str) -> Vec<String> {
    let mut links = Vec::new();
    
    // Simple magnet link extraction
    for part in body.split_whitespace() {
        if part.starts_with("magnet:?") {
            // Clean up the link (remove trailing punctuation, quotes, etc.)
            let clean = part
                .trim_end_matches(|c: char| c == '"' || c == '\'' || c == '>' || c == ')' || c == ']');
            links.push(clean.to_string());
        }
    }
    
    // Also check for href attributes
    let mut pos = 0;
    while let Some(start) = body[pos..].find("href=\"magnet:?") {
        let abs_start = pos + start + 6; // Skip 'href="'
        if let Some(end) = body[abs_start..].find('"') {
            links.push(body[abs_start..abs_start + end].to_string());
        }
        pos = abs_start;
    }
    
    links
}

/// Simple HTML escape
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Format file size for display
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

// =============================================================================
// 5. Download Manager
// =============================================================================

/// Tracks P2P attachment downloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PDownloadTracker {
    /// Active downloads by transfer ID
    pub downloads: HashMap<Uuid, P2PDownloadInfo>,
    /// Completed downloads (for history)
    pub completed: Vec<P2PDownloadInfo>,
    /// Failed downloads
    pub failed: Vec<P2PDownloadInfo>,
}

impl Default for P2PDownloadTracker {
    fn default() -> Self {
        Self {
            downloads: HashMap::new(),
            completed: Vec::new(),
            failed: Vec::new(),
        }
    }
}

/// Information about a P2P download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PDownloadInfo {
    /// Transfer ID
    pub transfer_id: Uuid,
    /// Email message ID this attachment belongs to
    pub message_id: String,
    /// Attachment info
    pub attachment: P2PAttachment,
    /// Download started at
    pub started_at: Option<DateTime<Utc>>,
    /// Download completed at
    pub completed_at: Option<DateTime<Utc>>,
    /// Progress (0-100)
    pub progress_percent: f64,
    /// Bytes downloaded
    pub bytes_downloaded: u64,
    /// Download speed (bytes/sec)
    pub speed_bps: u64,
    /// Estimated time remaining (seconds)
    pub eta_seconds: Option<u64>,
    /// Error message if failed
    pub error: Option<String>,
}

impl P2PDownloadTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start tracking a download
    pub fn start_download(&mut self, message_id: &str, attachment: P2PAttachment) -> Uuid {
        let transfer_id = attachment.transfer_id;
        
        let info = P2PDownloadInfo {
            transfer_id,
            message_id: message_id.to_string(),
            attachment,
            started_at: Some(Utc::now()),
            completed_at: None,
            progress_percent: 0.0,
            bytes_downloaded: 0,
            speed_bps: 0,
            eta_seconds: None,
            error: None,
        };
        
        self.downloads.insert(transfer_id, info);
        transfer_id
    }

    /// Update download progress
    pub fn update_progress(
        &mut self,
        transfer_id: &Uuid,
        progress: f64,
        bytes: u64,
        speed: u64,
        eta: Option<u64>,
    ) {
        if let Some(info) = self.downloads.get_mut(transfer_id) {
            info.progress_percent = progress;
            info.bytes_downloaded = bytes;
            info.speed_bps = speed;
            info.eta_seconds = eta;
        }
    }

    /// Mark download as complete
    pub fn complete_download(&mut self, transfer_id: &Uuid, local_path: PathBuf) {
        if let Some(mut info) = self.downloads.remove(transfer_id) {
            info.completed_at = Some(Utc::now());
            info.progress_percent = 100.0;
            info.bytes_downloaded = info.attachment.size;
            info.attachment.download_status = P2PDownloadStatus::Completed;
            info.attachment.local_path = Some(local_path);
            self.completed.push(info);
        }
    }

    /// Mark download as failed
    pub fn fail_download(&mut self, transfer_id: &Uuid, error: &str) {
        if let Some(mut info) = self.downloads.remove(transfer_id) {
            info.error = Some(error.to_string());
            info.attachment.download_status = P2PDownloadStatus::Failed;
            self.failed.push(info);
        }
    }

    /// Get active download count
    pub fn active_count(&self) -> usize {
        self.downloads.len()
    }

    /// Get download by transfer ID
    pub fn get_download(&self, transfer_id: &Uuid) -> Option<&P2PDownloadInfo> {
        self.downloads.get(transfer_id)
    }

    /// Get all active downloads for a message
    pub fn get_downloads_for_message(&self, message_id: &str) -> Vec<&P2PDownloadInfo> {
        self.downloads
            .values()
            .filter(|d| d.message_id == message_id)
            .collect()
    }
}

// =============================================================================
// CSS for P2P Attachment Display
// =============================================================================

/// CSS styles for P2P attachment display in emails
pub const P2P_ATTACHMENT_CSS: &str = r#"
.p2p-attachment {
    display: flex;
    align-items: center;
    padding: 12px 16px;
    margin: 8px 0;
    background: linear-gradient(135deg, #f8f9fa 0%, #e9ecef 100%);
    border: 1px solid #dee2e6;
    border-radius: 8px;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.p2p-attachment-icon {
    font-size: 24px;
    margin-right: 12px;
}

.p2p-attachment-info {
    flex: 1;
}

.p2p-attachment-name {
    font-weight: 600;
    color: #212529;
    margin-bottom: 2px;
}

.p2p-attachment-size {
    font-size: 12px;
    color: #6c757d;
}

.p2p-download-btn {
    display: inline-flex;
    align-items: center;
    padding: 8px 16px;
    background: #0d6efd;
    color: white;
    text-decoration: none;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    transition: background 0.2s;
}

.p2p-download-btn:hover {
    background: #0b5ed7;
}

.p2p-attachment.downloading {
    background: linear-gradient(135deg, #e7f1ff 0%, #cfe2ff 100%);
    border-color: #9ec5fe;
}

.p2p-attachment.completed {
    background: linear-gradient(135deg, #d1e7dd 0%, #badbcc 100%);
    border-color: #a3cfbb;
}

.p2p-attachment.failed {
    background: linear-gradient(135deg, #f8d7da 0%, #f5c2c7 100%);
    border-color: #f1aeb5;
}

.p2p-progress-bar {
    height: 4px;
    background: #e9ecef;
    border-radius: 2px;
    margin-top: 8px;
    overflow: hidden;
}

.p2p-progress-fill {
    height: 100%;
    background: #0d6efd;
    transition: width 0.3s ease;
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_thresholds() {
        let config = P2PAttachmentConfig::default();
        
        // Below threshold - should not use P2P
        assert!(!config.should_use_p2p(5 * 1024 * 1024)); // 5 MB
        
        // Above threshold - should use P2P
        assert!(config.should_use_p2p(15 * 1024 * 1024)); // 15 MB
        
        // Size acceptable
        assert!(config.is_size_acceptable(1024 * 1024 * 1024)); // 1 GB
        
        // Size too large
        assert!(!config.is_size_acceptable(3 * 1024 * 1024 * 1024)); // 3 GB
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 bytes");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_extract_magnet_links() {
        let body = r#"
            Check out this file: magnet:?xt=urn:btih:abc123&dn=test.zip
            And this one: <a href="magnet:?xt=urn:btih:def456&dn=file.pdf">Download</a>
        "#;
        
        let links = extract_magnet_links(body);
        assert_eq!(links.len(), 2);
        assert!(links[0].starts_with("magnet:?"));
    }

    #[test]
    fn test_attachment_analysis() {
        let config = P2PAttachmentConfig::default();
        let processor = AttachmentProcessor::new(config);
        
        let small_attachment = Attachment::new("small.txt", "text/plain", vec![0; 1024]);
        let analysis = processor.analyze(&small_attachment);
        assert_eq!(analysis.recommendation, AttachmentRecommendation::Inline);
        
        let large_attachment = Attachment::new("large.zip", "application/zip", vec![0; 20 * 1024 * 1024]);
        let analysis = processor.analyze(&large_attachment);
        assert_eq!(analysis.recommendation, AttachmentRecommendation::UseP2P);
    }
}
