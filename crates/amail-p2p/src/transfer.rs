//! Transfer Manager - Tracks active P2P transfers
//!
//! Table of Contents:
//! 1. TransferState - State of a transfer
//! 2. TransferProgress - Progress tracking
//! 3. Transfer - Active transfer instance

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::chunk::ChunkInfo;

// =============================================================================
// 1. Transfer State
// =============================================================================

/// State of a P2P transfer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferState {
    /// Waiting to start
    Pending,
    /// Discovering peers
    Discovering,
    /// Actively downloading/uploading
    Active,
    /// Paused by user
    Paused,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed,
    /// Cancelled by user
    Cancelled,
}

impl TransferState {
    pub fn is_active(&self) -> bool {
        matches!(self, TransferState::Active | TransferState::Discovering)
    }
    
    pub fn is_finished(&self) -> bool {
        matches!(self, TransferState::Completed | TransferState::Failed | TransferState::Cancelled)
    }
}

// =============================================================================
// 2. Transfer Progress
// =============================================================================

/// Progress information for a transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferProgress {
    /// Total bytes to transfer
    pub total_bytes: u64,
    /// Bytes transferred so far
    pub transferred_bytes: u64,
    /// Number of chunks completed
    pub chunks_completed: u32,
    /// Total number of chunks
    pub total_chunks: u32,
    /// Current transfer speed (bytes/sec)
    pub speed_bps: u64,
    /// Estimated time remaining (seconds)
    pub eta_seconds: Option<u64>,
    /// Number of active peers
    pub active_peers: usize,
}

impl TransferProgress {
    pub fn new(total_bytes: u64, total_chunks: u32) -> Self {
        Self {
            total_bytes,
            transferred_bytes: 0,
            chunks_completed: 0,
            total_chunks,
            speed_bps: 0,
            eta_seconds: None,
            active_peers: 0,
        }
    }
    
    /// Get progress as percentage (0-100)
    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            return 100.0;
        }
        (self.transferred_bytes as f64 / self.total_bytes as f64) * 100.0
    }
    
    /// Format speed for display
    pub fn speed_display(&self) -> String {
        let speed = self.speed_bps as f64;
        if speed >= 1_000_000_000.0 {
            format!("{:.2} GB/s", speed / 1_000_000_000.0)
        } else if speed >= 1_000_000.0 {
            format!("{:.2} MB/s", speed / 1_000_000.0)
        } else if speed >= 1_000.0 {
            format!("{:.2} KB/s", speed / 1_000.0)
        } else {
            format!("{:.0} B/s", speed)
        }
    }
    
    /// Format ETA for display
    pub fn eta_display(&self) -> String {
        match self.eta_seconds {
            Some(secs) if secs >= 3600 => {
                let hours = secs / 3600;
                let mins = (secs % 3600) / 60;
                format!("{}h {}m", hours, mins)
            }
            Some(secs) if secs >= 60 => {
                let mins = secs / 60;
                let secs = secs % 60;
                format!("{}m {}s", mins, secs)
            }
            Some(secs) => format!("{}s", secs),
            None => "calculating...".to_string(),
        }
    }
}

// =============================================================================
// 3. Transfer
// =============================================================================

/// An active P2P transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    /// Transfer ID
    pub id: Uuid,
    /// Transfer metadata
    pub info: ChunkInfo,
    /// Current state
    pub state: TransferState,
    /// Progress tracking
    pub progress: TransferProgress,
    /// Direction (true = download, false = upload)
    pub is_download: bool,
    /// Peers we're connected to for this transfer
    pub peers: HashSet<String>,
    /// Chunks we have
    pub have_chunks: Vec<bool>,
    /// When transfer started
    pub started_at: DateTime<Utc>,
    /// When transfer completed (if finished)
    pub completed_at: Option<DateTime<Utc>>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Local file path
    pub local_path: Option<String>,
}

impl Transfer {
    /// Create a new download transfer
    pub fn new_download(info: ChunkInfo) -> Self {
        let total_chunks = info.total_chunks;
        let total_bytes = info.total_size;
        
        Self {
            id: info.transfer_id,
            info,
            state: TransferState::Pending,
            progress: TransferProgress::new(total_bytes, total_chunks),
            is_download: true,
            peers: HashSet::new(),
            have_chunks: vec![false; total_chunks as usize],
            started_at: Utc::now(),
            completed_at: None,
            error: None,
            local_path: None,
        }
    }
    
    /// Create a new upload transfer (seeding)
    pub fn new_upload(info: ChunkInfo, local_path: String) -> Self {
        let total_chunks = info.total_chunks;
        let total_bytes = info.total_size;
        
        Self {
            id: info.transfer_id,
            info,
            state: TransferState::Active,
            progress: TransferProgress {
                total_bytes,
                transferred_bytes: total_bytes, // Already have all data
                chunks_completed: total_chunks,
                total_chunks,
                speed_bps: 0,
                eta_seconds: Some(0),
                active_peers: 0,
            },
            is_download: false,
            peers: HashSet::new(),
            have_chunks: vec![true; total_chunks as usize], // Have all chunks
            started_at: Utc::now(),
            completed_at: None,
            error: None,
            local_path: Some(local_path),
        }
    }
    
    /// Mark a chunk as received
    pub fn mark_chunk_received(&mut self, index: u32, chunk_size: usize) {
        if let Some(have) = self.have_chunks.get_mut(index as usize) {
            if !*have {
                *have = true;
                self.progress.chunks_completed += 1;
                self.progress.transferred_bytes += chunk_size as u64;
            }
        }
        
        // Check if complete
        if self.progress.chunks_completed == self.progress.total_chunks {
            self.state = TransferState::Completed;
            self.completed_at = Some(Utc::now());
        }
    }
    
    /// Get list of chunks we need
    pub fn needed_chunks(&self) -> Vec<u32> {
        self.have_chunks.iter()
            .enumerate()
            .filter(|(_, &have)| !have)
            .map(|(i, _)| i as u32)
            .collect()
    }
    
    /// Add a peer
    pub fn add_peer(&mut self, peer_id: String) {
        self.peers.insert(peer_id);
        self.progress.active_peers = self.peers.len();
    }
    
    /// Remove a peer
    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peers.remove(peer_id);
        self.progress.active_peers = self.peers.len();
    }
    
    /// Update transfer speed
    pub fn update_speed(&mut self, bytes_per_second: u64) {
        self.progress.speed_bps = bytes_per_second;
        
        // Calculate ETA
        if bytes_per_second > 0 {
            let remaining = self.progress.total_bytes - self.progress.transferred_bytes;
            self.progress.eta_seconds = Some(remaining / bytes_per_second);
        }
    }
    
    /// Pause the transfer
    pub fn pause(&mut self) {
        if self.state == TransferState::Active {
            self.state = TransferState::Paused;
        }
    }
    
    /// Resume the transfer
    pub fn resume(&mut self) {
        if self.state == TransferState::Paused {
            self.state = TransferState::Active;
        }
    }
    
    /// Cancel the transfer
    pub fn cancel(&mut self) {
        self.state = TransferState::Cancelled;
        self.completed_at = Some(Utc::now());
    }
    
    /// Mark as failed
    pub fn fail(&mut self, error: String) {
        self.state = TransferState::Failed;
        self.error = Some(error);
        self.completed_at = Some(Utc::now());
    }
    
    /// Get magnet link for sharing
    pub fn magnet_link(&self) -> String {
        self.info.magnet_link()
    }
}
