//! Chunk Manager - P2P-style file chunking
//!
//! Table of Contents:
//! 1. ChunkInfo - Metadata for a chunk
//! 2. FileChunk - Actual chunk data
//! 3. ChunkManager - Manages chunking and reassembly

use blake3::Hasher;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use uuid::Uuid;

/// Default chunk size: 1MB (can handle any file size)
pub const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024;

// =============================================================================
// 1. Chunk Info
// =============================================================================

/// Metadata about a file being transferred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    /// Unique transfer ID
    pub transfer_id: Uuid,
    /// Original filename
    pub filename: String,
    /// Total file size in bytes
    pub total_size: u64,
    /// Size of each chunk (except possibly last)
    pub chunk_size: usize,
    /// Total number of chunks
    pub total_chunks: u32,
    /// BLAKE3 hash of entire file
    pub file_hash: String,
    /// BLAKE3 hash of each chunk for verification
    pub chunk_hashes: Vec<String>,
    /// MIME type
    pub mime_type: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ChunkInfo {
    /// Create chunk info from a file
    pub async fn from_file(path: &Path, chunk_size: usize) -> anyhow::Result<Self> {
        let file = File::open(path).await?;
        let metadata = file.metadata().await?;
        let total_size = metadata.len();
        
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        // Calculate total chunks
        let total_chunks = ((total_size as f64) / (chunk_size as f64)).ceil() as u32;
        
        // Calculate file hash and chunk hashes
        let mut reader = BufReader::new(File::open(path).await?);
        let mut file_hasher = Hasher::new();
        let mut chunk_hashes = Vec::with_capacity(total_chunks as usize);
        
        let mut buffer = vec![0u8; chunk_size];
        loop {
            let bytes_read = reader.read(&mut buffer).await?;
            if bytes_read == 0 {
                break;
            }
            
            // Hash this chunk
            let chunk_hash = blake3::hash(&buffer[..bytes_read]);
            chunk_hashes.push(chunk_hash.to_hex().to_string());
            
            // Update file hash
            file_hasher.update(&buffer[..bytes_read]);
        }
        
        let file_hash = file_hasher.finalize().to_hex().to_string();
        
        // Detect MIME type
        let mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();
        
        Ok(Self {
            transfer_id: Uuid::new_v4(),
            filename,
            total_size,
            chunk_size,
            total_chunks,
            file_hash,
            chunk_hashes,
            mime_type,
            created_at: chrono::Utc::now(),
        })
    }
    
    /// Generate a magnet-style link for this transfer
    pub fn magnet_link(&self) -> String {
        format!(
            "amail://{}?xt=urn:blake3:{}&dn={}&xl={}",
            self.transfer_id,
            self.file_hash,
            urlencoding::encode(&self.filename),
            self.total_size
        )
    }
}

// =============================================================================
// 2. File Chunk
// =============================================================================

/// A single chunk of file data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChunk {
    /// Transfer ID this chunk belongs to
    pub transfer_id: Uuid,
    /// Chunk index (0-based)
    pub index: u32,
    /// Chunk data
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    /// BLAKE3 hash of this chunk
    pub hash: String,
}

impl FileChunk {
    /// Verify chunk integrity
    pub fn verify(&self) -> bool {
        let computed_hash = blake3::hash(&self.data).to_hex().to_string();
        computed_hash == self.hash
    }
}

// =============================================================================
// 3. Chunk Manager
// =============================================================================

/// Manages file chunking and reassembly
pub struct ChunkManager {
    /// Chunk size in bytes
    chunk_size: usize,
    /// Storage directory for chunks
    storage_dir: std::path::PathBuf,
    /// Active transfers (transfer_id -> ChunkInfo)
    transfers: HashMap<Uuid, ChunkInfo>,
    /// Received chunks (transfer_id -> set of chunk indices)
    received_chunks: HashMap<Uuid, Vec<bool>>,
}

impl ChunkManager {
    /// Create a new chunk manager
    pub fn new(storage_dir: impl AsRef<Path>) -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            storage_dir: storage_dir.as_ref().to_path_buf(),
            transfers: HashMap::new(),
            received_chunks: HashMap::new(),
        }
    }
    
    /// Set custom chunk size
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }
    
    /// Register a new transfer
    pub fn register_transfer(&mut self, info: ChunkInfo) {
        let transfer_id = info.transfer_id;
        let total_chunks = info.total_chunks as usize;
        self.transfers.insert(transfer_id, info);
        self.received_chunks.insert(transfer_id, vec![false; total_chunks]);
    }
    
    /// Get transfer info
    pub fn get_transfer(&self, transfer_id: &Uuid) -> Option<&ChunkInfo> {
        self.transfers.get(transfer_id)
    }
    
    /// Chunk a file for transfer
    pub async fn chunk_file(&mut self, path: &Path) -> anyhow::Result<ChunkInfo> {
        let info = ChunkInfo::from_file(path, self.chunk_size).await?;
        self.register_transfer(info.clone());
        Ok(info)
    }
    
    /// Read a specific chunk from a file
    pub async fn read_chunk(&self, path: &Path, index: u32) -> anyhow::Result<FileChunk> {
        let info = ChunkInfo::from_file(path, self.chunk_size).await?;
        
        let mut file = File::open(path).await?;
        let offset = (index as u64) * (self.chunk_size as u64);
        
        // Seek to chunk position
        use tokio::io::AsyncSeekExt;
        file.seek(std::io::SeekFrom::Start(offset)).await?;
        
        // Read chunk data
        let mut buffer = vec![0u8; self.chunk_size];
        let bytes_read = file.read(&mut buffer).await?;
        buffer.truncate(bytes_read);
        
        let hash = blake3::hash(&buffer).to_hex().to_string();
        
        Ok(FileChunk {
            transfer_id: info.transfer_id,
            index,
            data: buffer,
            hash,
        })
    }
    
    /// Store a received chunk
    pub async fn store_chunk(&mut self, chunk: FileChunk) -> anyhow::Result<bool> {
        // Verify chunk integrity
        if !chunk.verify() {
            anyhow::bail!("Chunk {} failed integrity check", chunk.index);
        }
        
        // Get transfer info
        let info = self.transfers.get(&chunk.transfer_id)
            .ok_or_else(|| anyhow::anyhow!("Unknown transfer: {}", chunk.transfer_id))?;
        
        // Verify chunk hash matches expected
        if let Some(expected_hash) = info.chunk_hashes.get(chunk.index as usize) {
            if &chunk.hash != expected_hash {
                anyhow::bail!("Chunk {} hash mismatch", chunk.index);
            }
        }
        
        // Create chunk file
        let chunk_dir = self.storage_dir.join(chunk.transfer_id.to_string());
        tokio::fs::create_dir_all(&chunk_dir).await?;
        
        let chunk_path = chunk_dir.join(format!("chunk_{:08}.bin", chunk.index));
        let mut file = File::create(&chunk_path).await?;
        file.write_all(&chunk.data).await?;
        
        // Mark chunk as received
        if let Some(received) = self.received_chunks.get_mut(&chunk.transfer_id) {
            if let Some(flag) = received.get_mut(chunk.index as usize) {
                *flag = true;
            }
        }
        
        // Check if transfer is complete
        Ok(self.is_transfer_complete(&chunk.transfer_id))
    }
    
    /// Check if all chunks have been received
    pub fn is_transfer_complete(&self, transfer_id: &Uuid) -> bool {
        self.received_chunks.get(transfer_id)
            .map(|chunks| chunks.iter().all(|&received| received))
            .unwrap_or(false)
    }
    
    /// Get missing chunk indices
    pub fn get_missing_chunks(&self, transfer_id: &Uuid) -> Vec<u32> {
        self.received_chunks.get(transfer_id)
            .map(|chunks| {
                chunks.iter()
                    .enumerate()
                    .filter(|(_, &received)| !received)
                    .map(|(i, _)| i as u32)
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get transfer progress (0.0 - 1.0)
    pub fn get_progress(&self, transfer_id: &Uuid) -> f64 {
        self.received_chunks.get(transfer_id)
            .map(|chunks| {
                let received = chunks.iter().filter(|&&r| r).count();
                received as f64 / chunks.len() as f64
            })
            .unwrap_or(0.0)
    }
    
    /// Reassemble chunks into final file
    pub async fn reassemble(&self, transfer_id: &Uuid, output_path: &Path) -> anyhow::Result<()> {
        let info = self.transfers.get(transfer_id)
            .ok_or_else(|| anyhow::anyhow!("Unknown transfer: {}", transfer_id))?;
        
        if !self.is_transfer_complete(transfer_id) {
            anyhow::bail!("Transfer not complete");
        }
        
        let chunk_dir = self.storage_dir.join(transfer_id.to_string());
        let mut output = BufWriter::new(File::create(output_path).await?);
        let mut file_hasher = Hasher::new();
        
        for i in 0..info.total_chunks {
            let chunk_path = chunk_dir.join(format!("chunk_{:08}.bin", i));
            let chunk_data = tokio::fs::read(&chunk_path).await?;
            
            file_hasher.update(&chunk_data);
            output.write_all(&chunk_data).await?;
        }
        
        output.flush().await?;
        
        // Verify final file hash
        let final_hash = file_hasher.finalize().to_hex().to_string();
        if final_hash != info.file_hash {
            tokio::fs::remove_file(output_path).await?;
            anyhow::bail!("Final file hash mismatch");
        }
        
        // Clean up chunk files
        tokio::fs::remove_dir_all(&chunk_dir).await?;
        
        Ok(())
    }
    
    /// Clean up a transfer
    pub async fn cleanup_transfer(&mut self, transfer_id: &Uuid) -> anyhow::Result<()> {
        let chunk_dir = self.storage_dir.join(transfer_id.to_string());
        if chunk_dir.exists() {
            tokio::fs::remove_dir_all(&chunk_dir).await?;
        }
        self.transfers.remove(transfer_id);
        self.received_chunks.remove(transfer_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[tokio::test]
    async fn test_chunk_info() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        tokio::fs::write(&file_path, "Hello, World!").await.unwrap();
        
        let info = ChunkInfo::from_file(&file_path, 1024).await.unwrap();
        assert_eq!(info.filename, "test.txt");
        assert_eq!(info.total_size, 13);
        assert_eq!(info.total_chunks, 1);
    }
}
