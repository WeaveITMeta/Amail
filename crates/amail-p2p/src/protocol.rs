//! P2P Protocol - Request/Response messages for transfers
//!
//! Table of Contents:
//! 1. Transfer Request/Response types
//! 2. Protocol codec for libp2p

use libp2p::request_response;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::chunk::{ChunkInfo, FileChunk};

// =============================================================================
// 1. Transfer Request/Response
// =============================================================================

/// Request types for P2P transfer protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferRequest {
    /// Request transfer metadata
    GetInfo { transfer_id: Uuid },
    /// Request a specific chunk
    GetChunk { transfer_id: Uuid, index: u32 },
    /// Request list of available transfers
    ListTransfers,
    /// Announce a new transfer is available
    AnnounceTransfer { info: ChunkInfo },
    /// Request missing chunks for a transfer
    GetMissingChunks { transfer_id: Uuid },
    /// Ping for connectivity check
    Ping,
}

/// Response types for P2P transfer protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferResponse {
    /// Transfer metadata
    Info { info: ChunkInfo },
    /// Chunk data
    Chunk { chunk: FileChunk },
    /// List of available transfer IDs
    TransferList { transfers: Vec<Uuid> },
    /// Acknowledgment of announcement
    AnnounceAck { transfer_id: Uuid },
    /// List of missing chunk indices
    MissingChunks { transfer_id: Uuid, indices: Vec<u32> },
    /// Pong response
    Pong,
    /// Error response
    Error { message: String },
    /// Transfer not found
    NotFound { transfer_id: Uuid },
}

// =============================================================================
// 2. Protocol Codec
// =============================================================================

/// Protocol name for libp2p
pub const PROTOCOL_NAME: &str = "/amail/transfer/1.0.0";

/// CBOR codec for request/response
#[derive(Debug, Clone, Default)]
pub struct TransferCodec;

impl request_response::Codec for TransferCodec {
    type Protocol = &'static str;
    type Request = TransferRequest;
    type Response = TransferResponse;

    fn read_request<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _protocol: &'life1 Self::Protocol,
        io: &'life2 mut T,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::io::Result<Self::Request>> + Send + 'async_trait>>
    where
        T: futures::AsyncRead + Unpin + Send + 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let mut buf = Vec::new();
            futures::AsyncReadExt::read_to_end(io, &mut buf).await?;
            serde_cbor::from_slice(&buf)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })
    }

    fn read_response<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _protocol: &'life1 Self::Protocol,
        io: &'life2 mut T,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::io::Result<Self::Response>> + Send + 'async_trait>>
    where
        T: futures::AsyncRead + Unpin + Send + 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let mut buf = Vec::new();
            futures::AsyncReadExt::read_to_end(io, &mut buf).await?;
            serde_cbor::from_slice(&buf)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
        })
    }

    fn write_request<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _protocol: &'life1 Self::Protocol,
        io: &'life2 mut T,
        req: Self::Request,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::io::Result<()>> + Send + 'async_trait>>
    where
        T: futures::AsyncWrite + Unpin + Send + 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let buf = serde_cbor::to_vec(&req)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            futures::AsyncWriteExt::write_all(io, &buf).await?;
            Ok(())
        })
    }

    fn write_response<'life0, 'life1, 'life2, 'async_trait, T>(
        &'life0 mut self,
        _protocol: &'life1 Self::Protocol,
        io: &'life2 mut T,
        res: Self::Response,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::io::Result<()>> + Send + 'async_trait>>
    where
        T: futures::AsyncWrite + Unpin + Send + 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            let buf = serde_cbor::to_vec(&res)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            futures::AsyncWriteExt::write_all(io, &buf).await?;
            Ok(())
        })
    }
}
