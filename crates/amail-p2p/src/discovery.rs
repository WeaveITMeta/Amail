//! Peer Discovery - mDNS and DHT for finding peers
//!
//! Table of Contents:
//! 1. Discovery Config
//! 2. Peer Info
//! 3. Discovery Events

use libp2p::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use uuid::Uuid;

// =============================================================================
// 1. Discovery Config
// =============================================================================

/// Configuration for peer discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Enable mDNS for local network discovery
    pub mdns_enabled: bool,
    /// Enable Kademlia DHT for internet-wide discovery
    pub dht_enabled: bool,
    /// Bootstrap peers for DHT
    pub bootstrap_peers: Vec<String>,
    /// How often to announce ourselves (seconds)
    pub announce_interval_secs: u64,
    /// Maximum peers to connect to
    pub max_peers: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            mdns_enabled: true,
            dht_enabled: true,
            bootstrap_peers: vec![],
            announce_interval_secs: 300,
            max_peers: 50,
        }
    }
}

// =============================================================================
// 2. Peer Info
// =============================================================================

/// Information about a discovered peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// libp2p peer ID
    pub peer_id: String,
    /// Known addresses
    pub addresses: Vec<String>,
    /// Transfers this peer has
    pub available_transfers: Vec<Uuid>,
    /// Last seen timestamp
    pub last_seen: chrono::DateTime<chrono::Utc>,
    /// Connection latency (ms)
    pub latency_ms: Option<u64>,
    /// Is currently connected
    pub connected: bool,
}

impl PeerInfo {
    pub fn new(peer_id: String) -> Self {
        Self {
            peer_id,
            addresses: Vec::new(),
            available_transfers: Vec::new(),
            last_seen: chrono::Utc::now(),
            latency_ms: None,
            connected: false,
        }
    }
    
    pub fn with_address(mut self, addr: String) -> Self {
        self.addresses.push(addr);
        self
    }
    
    pub fn update_seen(&mut self) {
        self.last_seen = chrono::Utc::now();
    }
}

// =============================================================================
// 3. Discovery Events
// =============================================================================

/// Events from the discovery system
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    /// New peer discovered
    PeerDiscovered { peer_id: String, addresses: Vec<String> },
    /// Peer went offline
    PeerLost { peer_id: String },
    /// Peer announced a transfer
    TransferAnnounced { peer_id: String, transfer_id: Uuid },
    /// DHT bootstrap complete
    DhtReady,
    /// mDNS discovery started
    MdnsReady,
}

/// Peer registry for tracking discovered peers
#[derive(Debug, Default)]
pub struct PeerRegistry {
    peers: HashMap<String, PeerInfo>,
    /// Map of transfer_id -> peer_ids that have it
    transfer_providers: HashMap<Uuid, Vec<String>>,
}

impl PeerRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add or update a peer
    pub fn upsert_peer(&mut self, peer_id: String, addresses: Vec<String>) {
        let peer = self.peers.entry(peer_id.clone()).or_insert_with(|| PeerInfo::new(peer_id));
        peer.addresses = addresses;
        peer.update_seen();
    }
    
    /// Mark peer as connected
    pub fn set_connected(&mut self, peer_id: &str, connected: bool) {
        if let Some(peer) = self.peers.get_mut(peer_id) {
            peer.connected = connected;
            peer.update_seen();
        }
    }
    
    /// Remove a peer
    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peers.remove(peer_id);
        // Remove from transfer providers
        for providers in self.transfer_providers.values_mut() {
            providers.retain(|p| p != peer_id);
        }
    }
    
    /// Register that a peer has a transfer
    pub fn register_transfer_provider(&mut self, peer_id: String, transfer_id: Uuid) {
        // Update peer's available transfers
        if let Some(peer) = self.peers.get_mut(&peer_id) {
            if !peer.available_transfers.contains(&transfer_id) {
                peer.available_transfers.push(transfer_id);
            }
        }
        
        // Update transfer providers map
        let providers = self.transfer_providers.entry(transfer_id).or_default();
        if !providers.contains(&peer_id) {
            providers.push(peer_id);
        }
    }
    
    /// Get peers that have a specific transfer
    pub fn get_transfer_providers(&self, transfer_id: &Uuid) -> Vec<&PeerInfo> {
        self.transfer_providers.get(transfer_id)
            .map(|peer_ids| {
                peer_ids.iter()
                    .filter_map(|id| self.peers.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get all connected peers
    pub fn connected_peers(&self) -> Vec<&PeerInfo> {
        self.peers.values().filter(|p| p.connected).collect()
    }
    
    /// Get peer by ID
    pub fn get_peer(&self, peer_id: &str) -> Option<&PeerInfo> {
        self.peers.get(peer_id)
    }
    
    /// Get all peers
    pub fn all_peers(&self) -> Vec<&PeerInfo> {
        self.peers.values().collect()
    }
    
    /// Count connected peers
    pub fn connected_count(&self) -> usize {
        self.peers.values().filter(|p| p.connected).count()
    }
}
