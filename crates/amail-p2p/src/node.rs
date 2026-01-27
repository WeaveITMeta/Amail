//! P2P Node - libp2p network node for Amail transfers
//!
//! Table of Contents:
//! 1. P2PConfig - Node configuration
//! 2. P2PEvent - Events from the node
//! 3. P2PNode - Main node implementation

use futures::StreamExt;
use libp2p::{
    identity, kad, mdns, noise, request_response, swarm::SwarmEvent, tcp, yamux, Multiaddr,
    PeerId, Swarm,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::chunk::{ChunkInfo, ChunkManager, FileChunk};
use crate::discovery::{DiscoveryConfig, DiscoveryEvent, PeerInfo, PeerRegistry};
use crate::protocol::{TransferCodec, TransferRequest, TransferResponse, PROTOCOL_NAME};
use crate::transfer::{Transfer, TransferState};

// =============================================================================
// 1. P2P Config
// =============================================================================

/// Configuration for the P2P node
#[derive(Debug, Clone)]
pub struct P2PConfig {
    /// Listen address
    pub listen_addr: String,
    /// Storage directory for chunks
    pub storage_dir: PathBuf,
    /// Discovery configuration
    pub discovery: DiscoveryConfig,
    /// Maximum concurrent downloads
    pub max_concurrent_downloads: usize,
    /// Chunk request timeout
    pub chunk_timeout: Duration,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_addr: "/ip4/0.0.0.0/tcp/0".to_string(),
            storage_dir: PathBuf::from("./p2p_storage"),
            discovery: DiscoveryConfig::default(),
            max_concurrent_downloads: 5,
            chunk_timeout: Duration::from_secs(30),
        }
    }
}

// =============================================================================
// 2. P2P Events
// =============================================================================

/// Events emitted by the P2P node
#[derive(Debug, Clone)]
pub enum P2PEvent {
    /// Node started listening
    Listening { address: String },
    /// Peer connected
    PeerConnected { peer_id: String },
    /// Peer disconnected
    PeerDisconnected { peer_id: String },
    /// Transfer started
    TransferStarted { transfer_id: Uuid },
    /// Transfer progress update
    TransferProgress { transfer_id: Uuid, progress: f64 },
    /// Transfer completed
    TransferCompleted { transfer_id: Uuid, path: PathBuf },
    /// Transfer failed
    TransferFailed { transfer_id: Uuid, error: String },
    /// New transfer announced by peer
    TransferAnnounced { transfer_id: Uuid, info: ChunkInfo },
    /// Discovery event
    Discovery(DiscoveryEvent),
}

// =============================================================================
// 3. P2P Node
// =============================================================================

/// Main P2P node for Amail transfers
pub struct P2PNode {
    /// Node configuration
    config: P2PConfig,
    /// Our peer ID
    peer_id: PeerId,
    /// Chunk manager
    chunk_manager: Arc<RwLock<ChunkManager>>,
    /// Active transfers
    transfers: Arc<RwLock<HashMap<Uuid, Transfer>>>,
    /// Peer registry
    peers: Arc<RwLock<PeerRegistry>>,
    /// Event sender
    event_tx: mpsc::Sender<P2PEvent>,
    /// Event receiver (for external consumers)
    event_rx: Option<mpsc::Receiver<P2PEvent>>,
    /// Files available for sharing (transfer_id -> file_path)
    shared_files: Arc<RwLock<HashMap<Uuid, PathBuf>>>,
}

impl P2PNode {
    /// Create a new P2P node
    pub fn new(config: P2PConfig) -> anyhow::Result<Self> {
        let (event_tx, event_rx) = mpsc::channel(1000);
        
        // Generate identity
        let local_key = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(local_key.public());
        
        info!("P2P Node created with peer ID: {}", peer_id);
        
        // Create storage directory
        std::fs::create_dir_all(&config.storage_dir)?;
        
        let chunk_manager = ChunkManager::new(&config.storage_dir);
        
        Ok(Self {
            config,
            peer_id,
            chunk_manager: Arc::new(RwLock::new(chunk_manager)),
            transfers: Arc::new(RwLock::new(HashMap::new())),
            peers: Arc::new(RwLock::new(PeerRegistry::new())),
            event_tx,
            event_rx: Some(event_rx),
            shared_files: Arc::new(RwLock::new(HashMap::new())),
        })
    }
    
    /// Get our peer ID
    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }
    
    /// Take the event receiver (can only be called once)
    pub fn take_event_receiver(&mut self) -> Option<mpsc::Receiver<P2PEvent>> {
        self.event_rx.take()
    }
    
    /// Share a file for P2P transfer
    pub async fn share_file(&self, path: &Path) -> anyhow::Result<ChunkInfo> {
        let mut chunk_manager = self.chunk_manager.write().await;
        let info = chunk_manager.chunk_file(path).await?;
        
        // Register the file for sharing
        self.shared_files.write().await.insert(info.transfer_id, path.to_path_buf());
        
        // Create upload transfer
        let transfer = Transfer::new_upload(info.clone(), path.to_string_lossy().to_string());
        self.transfers.write().await.insert(info.transfer_id, transfer);
        
        info!("Sharing file: {} ({} bytes, {} chunks)", 
            info.filename, info.total_size, info.total_chunks);
        info!("Magnet link: {}", info.magnet_link());
        
        let _ = self.event_tx.send(P2PEvent::TransferStarted { 
            transfer_id: info.transfer_id 
        }).await;
        
        Ok(info)
    }
    
    /// Start downloading a file from peers
    pub async fn download(&self, info: ChunkInfo, output_path: &Path) -> anyhow::Result<Uuid> {
        let transfer_id = info.transfer_id;
        
        // Register the transfer
        self.chunk_manager.write().await.register_transfer(info.clone());
        
        // Create download transfer
        let mut transfer = Transfer::new_download(info);
        transfer.local_path = Some(output_path.to_string_lossy().to_string());
        transfer.state = TransferState::Discovering;
        
        self.transfers.write().await.insert(transfer_id, transfer);
        
        let _ = self.event_tx.send(P2PEvent::TransferStarted { transfer_id }).await;
        
        Ok(transfer_id)
    }
    
    /// Get transfer status
    pub async fn get_transfer(&self, transfer_id: &Uuid) -> Option<Transfer> {
        self.transfers.read().await.get(transfer_id).cloned()
    }
    
    /// Get all active transfers
    pub async fn get_transfers(&self) -> Vec<Transfer> {
        self.transfers.read().await.values().cloned().collect()
    }
    
    /// Pause a transfer
    pub async fn pause_transfer(&self, transfer_id: &Uuid) -> anyhow::Result<()> {
        if let Some(transfer) = self.transfers.write().await.get_mut(transfer_id) {
            transfer.pause();
            Ok(())
        } else {
            anyhow::bail!("Transfer not found: {}", transfer_id)
        }
    }
    
    /// Resume a transfer
    pub async fn resume_transfer(&self, transfer_id: &Uuid) -> anyhow::Result<()> {
        if let Some(transfer) = self.transfers.write().await.get_mut(transfer_id) {
            transfer.resume();
            Ok(())
        } else {
            anyhow::bail!("Transfer not found: {}", transfer_id)
        }
    }
    
    /// Cancel a transfer
    pub async fn cancel_transfer(&self, transfer_id: &Uuid) -> anyhow::Result<()> {
        if let Some(transfer) = self.transfers.write().await.get_mut(transfer_id) {
            transfer.cancel();
            self.chunk_manager.write().await.cleanup_transfer(transfer_id).await?;
            Ok(())
        } else {
            anyhow::bail!("Transfer not found: {}", transfer_id)
        }
    }
    
    /// Get connected peers
    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.read().await.all_peers().into_iter().cloned().collect()
    }
    
    /// Get connected peer count
    pub async fn connected_peer_count(&self) -> usize {
        self.peers.read().await.connected_count()
    }
    
    /// Handle incoming chunk request
    async fn handle_chunk_request(&self, transfer_id: Uuid, index: u32) -> TransferResponse {
        // Check if we have this file
        let shared_files = self.shared_files.read().await;
        if let Some(path) = shared_files.get(&transfer_id) {
            let chunk_manager = self.chunk_manager.read().await;
            match chunk_manager.read_chunk(path, index).await {
                Ok(chunk) => TransferResponse::Chunk { chunk },
                Err(e) => TransferResponse::Error { message: e.to_string() },
            }
        } else {
            TransferResponse::NotFound { transfer_id }
        }
    }
    
    /// Handle incoming info request
    async fn handle_info_request(&self, transfer_id: Uuid) -> TransferResponse {
        let chunk_manager = self.chunk_manager.read().await;
        if let Some(info) = chunk_manager.get_transfer(&transfer_id) {
            TransferResponse::Info { info: info.clone() }
        } else {
            TransferResponse::NotFound { transfer_id }
        }
    }
    
    /// Process a received chunk
    async fn process_received_chunk(&self, chunk: FileChunk) -> anyhow::Result<()> {
        let transfer_id = chunk.transfer_id;
        let index = chunk.index;
        let chunk_size = chunk.data.len();
        
        // Store the chunk
        let complete = self.chunk_manager.write().await.store_chunk(chunk).await?;
        
        // Update transfer progress
        if let Some(transfer) = self.transfers.write().await.get_mut(&transfer_id) {
            transfer.mark_chunk_received(index, chunk_size);
            
            let progress = transfer.progress.percentage();
            let _ = self.event_tx.send(P2PEvent::TransferProgress { 
                transfer_id, 
                progress 
            }).await;
            
            // If complete, reassemble
            if complete {
                if let Some(ref output_path) = transfer.local_path {
                    let output = PathBuf::from(output_path);
                    self.chunk_manager.read().await.reassemble(&transfer_id, &output).await?;
                    
                    let _ = self.event_tx.send(P2PEvent::TransferCompleted { 
                        transfer_id, 
                        path: output 
                    }).await;
                }
            }
        }
        
        Ok(())
    }
}

/// Export mailbox data for P2P transfer
pub async fn export_mailbox(
    node: &P2PNode,
    mailbox_data: &[u8],
    filename: &str,
) -> anyhow::Result<ChunkInfo> {
    // Write to temp file
    let temp_path = node.config.storage_dir.join(format!("{}.amail", filename));
    tokio::fs::write(&temp_path, mailbox_data).await?;
    
    // Share via P2P
    let info = node.share_file(&temp_path).await?;
    
    Ok(info)
}

/// Import mailbox data from P2P transfer
pub async fn import_mailbox(
    node: &P2PNode,
    info: ChunkInfo,
) -> anyhow::Result<Vec<u8>> {
    let output_path = node.config.storage_dir.join(format!("{}.amail", info.transfer_id));
    
    // Start download
    node.download(info, &output_path).await?;
    
    // Wait for completion (in real implementation, this would be async)
    // For now, return empty - the actual data comes via events
    Ok(Vec::new())
}
