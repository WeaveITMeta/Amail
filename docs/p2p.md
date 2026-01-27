# P2P File Transfer

Amail includes peer-to-peer file transfers, eliminating attachment size limits and enabling direct transfers between Amail servers.

---

## Why P2P?

| Traditional Email | Amail P2P |
|-------------------|-----------|
| 25MB attachment limit | **No limit** |
| Files stored on mail server | **Direct peer transfer** |
| Single point of failure | **Distributed** |
| Slow for large files | **Parallel chunk downloads** |

---

## How It Works

### Architecture

```
┌─────────────┐                    ┌─────────────┐
│  Amail A    │                    │  Amail B    │
│             │                    │             │
│ ┌─────────┐ │    P2P Protocol    │ ┌─────────┐ │
│ │  P2P    │◄├────────────────────┤►│  P2P    │ │
│ │  Node   │ │                    │ │  Node   │ │
│ └─────────┘ │                    │ └─────────┘ │
│             │                    │             │
│ ┌─────────┐ │                    │ ┌─────────┐ │
│ │ Chunks  │ │                    │ │ Chunks  │ │
│ │ Storage │ │                    │ │ Storage │ │
│ └─────────┘ │                    │ └─────────┘ │
└─────────────┘                    └─────────────┘
```

### Transfer Process

1. **Chunking**: File split into 1MB chunks
2. **Hashing**: Each chunk gets BLAKE3 hash
3. **Announcement**: Transfer announced to DHT
4. **Discovery**: Peers find each other via mDNS/DHT
5. **Transfer**: Chunks downloaded in parallel
6. **Verification**: Hashes verified on receipt
7. **Assembly**: Chunks reassembled into file

---

## Magnet Links

Amail uses magnet-style links for sharing:

```
amail://transfer-uuid?xt=urn:blake3:filehash&dn=filename.zip&xl=filesize
```

Components:
- `transfer-uuid`: Unique transfer identifier
- `xt`: File hash (BLAKE3)
- `dn`: Display name (filename)
- `xl`: File size in bytes

---

## API Endpoints

### Share a File

```bash
POST /api/v1/p2p/transfers/share
Content-Type: application/json

{
  "file_path": "/path/to/large-file.zip"
}
```

Response:
```json
{
  "transfer_id": "550e8400-e29b-41d4-a716-446655440000",
  "filename": "large-file.zip",
  "size": 1073741824,
  "chunks": 1024,
  "magnet_link": "amail://550e8400...?xt=urn:blake3:abc123&dn=large-file.zip&xl=1073741824",
  "file_hash": "abc123..."
}
```

### Start Download

```bash
POST /api/v1/p2p/transfers/download
Content-Type: application/json

{
  "transfer_info": {
    "transfer_id": "550e8400-e29b-41d4-a716-446655440000",
    "filename": "large-file.zip",
    "total_size": 1073741824,
    "total_chunks": 1024,
    "chunk_size": 1048576,
    "file_hash": "abc123...",
    "chunk_hashes": ["hash1", "hash2", ...]
  },
  "output_path": "/downloads/large-file.zip"
}
```

### List Transfers

```bash
GET /api/v1/p2p/transfers
```

Response:
```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "filename": "large-file.zip",
    "state": "Downloading",
    "is_download": true,
    "progress_percent": 45.5,
    "transferred_bytes": 488636416,
    "total_bytes": 1073741824,
    "chunks_completed": 466,
    "total_chunks": 1024,
    "speed": "12.5 MB/s",
    "eta": "47s",
    "peers": 3,
    "magnet_link": "amail://..."
  }
]
```

### Transfer Control

```bash
# Pause
POST /api/v1/p2p/transfers/{id}/pause

# Resume
POST /api/v1/p2p/transfers/{id}/resume

# Cancel
POST /api/v1/p2p/transfers/{id}/cancel
```

### Get Transfer Status

```bash
GET /api/v1/p2p/transfers/{id}
```

### List Peers

```bash
GET /api/v1/p2p/peers
```

### Node Info

```bash
GET /api/v1/p2p/info
```

Response:
```json
{
  "initialized": true,
  "peer_id": "12D3KooW...",
  "connected_peers": 5,
  "active_downloads": 2,
  "active_uploads": 1,
  "total_transfers": 15
}
```

---

## Mailbox Export/Import

Transfer entire mailboxes between Amail servers:

### Export Mailbox

```bash
POST /api/v1/p2p/mailbox/export
Content-Type: application/json

{
  "mailbox_id": "inbox-uuid",
  "domain": "example.com",
  "include_attachments": true
}
```

### Import Mailbox

```bash
POST /api/v1/p2p/mailbox/import
Content-Type: application/json

{
  "transfer_id": "550e8400-e29b-41d4-a716-446655440000",
  "target_domain": "newdomain.com"
}
```

---

## Peer Discovery

### mDNS (Local Network)

Automatically discovers other Amail instances on the same network:
- Zero configuration
- Works behind NAT
- Instant discovery

### Kademlia DHT (Internet)

For discovering peers across the internet:
- Distributed hash table
- No central server
- Bootstrap nodes for initial connection

---

## Configuration

```env
# P2P Settings (future)
P2P_ENABLED=true
P2P_LISTEN_PORT=4001
P2P_CHUNK_SIZE=1048576  # 1MB
P2P_MAX_PEERS=50
P2P_BOOTSTRAP_NODES=
```

---

## Security

### Integrity
- BLAKE3 hash for each chunk
- Full file hash verification
- Corrupted chunks automatically re-downloaded

### Privacy
- Direct peer-to-peer (no relay servers)
- libp2p encryption
- Transfer IDs are random UUIDs

### Access Control
- Transfers require knowing the magnet link
- Links can be shared via email or other channels
- No public directory of transfers

---

## Use Cases

### Large Attachments
Instead of failing on 25MB+ attachments:
1. Amail detects large attachment
2. Uploads to P2P network
3. Embeds magnet link in email
4. Recipient's Amail downloads automatically

### Server Migration
Moving to a new server:
1. Export mailboxes via P2P
2. Share magnet links with new server
3. Import on new server
4. Direct transfer, no intermediate storage

### Backup Distribution
Distributing backups across multiple servers:
1. Create backup archive
2. Share via P2P
3. Multiple peers can seed
4. Redundant storage

---

## Technical Details

### Chunk Format

```rust
struct FileChunk {
    transfer_id: Uuid,
    index: u32,
    data: Vec<u8>,  // Up to 1MB
    hash: String,   // BLAKE3 hash
}
```

### Protocol Messages

```rust
enum TransferRequest {
    GetInfo { transfer_id: Uuid },
    GetChunk { transfer_id: Uuid, index: u32 },
    Announce { info: ChunkInfo },
}

enum TransferResponse {
    Info { info: ChunkInfo },
    Chunk { chunk: FileChunk },
    NotFound,
    Error { message: String },
}
```

### Transfer States

```
Pending → Downloading → Completed
              ↓
           Paused
              ↓
          Cancelled
              ↓
           Failed
```

---

## Troubleshooting

### No Peers Found

1. Check firewall allows P2P port (default 4001)
2. Ensure mDNS is not blocked on network
3. Try adding bootstrap nodes manually

### Slow Transfers

1. Check network bandwidth
2. Increase max peers setting
3. Verify peers are not behind restrictive NAT

### Hash Mismatch

1. Chunk will be re-downloaded automatically
2. If persistent, check disk for errors
3. Verify source file hasn't changed

### Transfer Stuck

1. Check peer connectivity
2. Try pausing and resuming
3. Check available disk space
