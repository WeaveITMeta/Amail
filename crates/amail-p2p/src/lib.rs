//! Amail P2P - Peer-to-Peer Mail Data Transfer
//!
//! Table of Contents:
//! 1. P2P Node - libp2p network node
//! 2. Chunked Transfer - BitTorrent-style file chunking
//! 3. Peer Discovery - mDNS and DHT
//! 4. Transfer Protocol - Request/Response for chunks
//! 5. Mail Export/Import - Full mailbox transfer

pub mod node;
pub mod chunk;
pub mod transfer;
pub mod protocol;
pub mod discovery;

pub use node::{P2PNode, P2PConfig, P2PEvent};
pub use chunk::{ChunkManager, FileChunk, ChunkInfo};
pub use transfer::{Transfer, TransferState, TransferProgress};
pub use protocol::{TransferRequest, TransferResponse};
