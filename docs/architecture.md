# Amail Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          EXTERNAL WORLD                                  │
│  (Email senders, recipients, Telegram, DNS)                             │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    │               │               │
                    ▼               ▼               ▼
            ┌───────────┐   ┌───────────┐   ┌───────────┐
            │   SMTP    │   │  Webhook  │   │ Telegram  │
            │  Server   │   │  Receiver │   │    Bot    │
            │ (Port 25) │   │ (Port 4000)│   │           │
            └─────┬─────┘   └─────┬─────┘   └─────┬─────┘
                  │               │               │
                  └───────────────┼───────────────┘
                                  │
                                  ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         AMAIL CORE                                       │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      Task Queue (Redis/In-Memory)                │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                  │                                       │
│                                  ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                    AGENT SWARM (Actix Actors)                    │   │
│  │                                                                   │   │
│  │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │   │
│  │   │Receptionist │──│   Clerk     │──│  Executive  │             │   │
│  │   │  (Triage)   │  │(Scheduling) │  │ (Decisions) │             │   │
│  │   └─────────────┘  └─────────────┘  └─────────────┘             │   │
│  │          │                │                │                      │   │
│  │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │   │
│  │   │DraftChecker │  │ Accountant  │  │   Lawyer    │             │   │
│  │   │  (Review)   │  │ (Finance)   │  │  (Legal)    │             │   │
│  │   └─────────────┘  └─────────────┘  └─────────────┘             │   │
│  │                                                                   │   │
│  │   + 25 more specialized agents (HR, IT, Sales, Marketing, etc.)  │   │
│  └─────────────────────────────────────────────────────────────────┘   │
│                                  │                                       │
│                                  ▼                                       │
│  ┌─────────────────────────────────────────────────────────────────┐   │
│  │                      LLM Client (Ollama)                         │   │
│  │                    Local Mistral/Llama/Phi                       │   │
│  └─────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                  │
                    ┌─────────────┼─────────────┐
                    │             │             │
                    ▼             ▼             ▼
            ┌───────────┐ ┌───────────┐ ┌───────────┐
            │PostgreSQL │ │   P2P     │ │  SMTP     │
            │ Database  │ │  Network  │ │ Outbound  │
            └───────────┘ └───────────┘ └───────────┘
```

## Component Details

### 1. Email Ingestion Layer

**Embedded SMTP Server** (`amail-mail`)
- Listens on port 25/587 for incoming emails
- Parses MIME messages
- Extracts headers, body, attachments
- Routes to task queue

**Webhook Receiver** (`amail-webhook`)
- Receives POST from Stalwart mail server
- Verifies HMAC-SHA256 signature
- Fetches full email via Stalwart API
- Creates processing task

### 2. Task Queue

**In-Memory Queue** (default)
- Fast, no external dependencies
- Suitable for single-server deployments

**Redis Queue** (optional)
- Distributed task processing
- Persistence across restarts
- Multiple worker support

### 3. Agent Swarm

**Actix Actor System**
- Each agent is an independent actor
- Supervised lifecycle (restart on failure)
- Message-passing communication
- Concurrent task processing

**Agent Registry**
- Discovers available agents
- Routes tasks by capability
- Load balancing across agents

**Supervisor**
- Monitors agent health
- Restarts failed agents
- Erlang/OTP-style supervision tree

### 4. LLM Integration

**Ollama Client** (`amail-llm`)
- Connects to local Ollama server
- Supports multiple models (Mistral, Llama, Phi)
- Tool calling for structured responses
- Streaming for long responses

**Prompt Engineering** (`amail-agents/prompts.rs`)
- Role-specific system prompts
- Context injection (email history, metadata)
- Output formatting instructions

### 5. Data Layer

**PostgreSQL** (`amail-core/repositories`)
- Emails with full-text search
- Task history and status
- Agent states and metrics
- Domain/mailbox configuration
- Human review queue
- Analytics (daily/hourly metrics)

**Migrations** (`amail-core/migrations`)
- Automatic schema updates
- Version tracking
- Rollback support

### 6. P2P Network

**libp2p** (`amail-p2p`)
- Kademlia DHT for peer discovery
- mDNS for local network discovery
- Request-response protocol for chunks

**Chunked Transfer**
- 1MB chunks with BLAKE3 hashing
- Parallel downloads from multiple peers
- Resume interrupted transfers

### 7. External Integrations

**Telegram Bot**
- Human review notifications
- Approve/reject buttons
- Daily digest summaries

**Stalwart Mail Server**
- Full SMTP/IMAP/JMAP support
- Webhook integration
- REST API for mailbox management

---

## Data Flow: Email Processing

```
1. Email arrives via SMTP/Webhook
          │
          ▼
2. Parse email (headers, body, attachments)
          │
          ▼
3. Create task in queue
          │
          ▼
4. Receptionist agent triages
   - Classify intent
   - Determine urgency
   - Route to specialist
          │
          ▼
5. Specialist agent processes
   - Generate response via LLM
   - Calculate confidence score
          │
          ▼
6. DraftChecker reviews
   - Quality check
   - Tone verification
          │
          ▼
7. Confidence check
   ├─ High (>90%): Auto-send
   ├─ Medium (75-90%): Send with logging
   └─ Low (<75%): Human review via Telegram
          │
          ▼
8. Send response via SMTP
          │
          ▼
9. Log to database, update metrics
```

---

## Crate Structure

```
amail/
├── src/
│   ├── main.rs           # Server entry point
│   └── worker.rs         # Background worker
│
├── crates/
│   ├── amail-core/       # Core types and database
│   │   ├── email.rs      # Email types
│   │   ├── agent.rs      # Agent types (30+ roles)
│   │   ├── task.rs       # Task types
│   │   ├── db.rs         # Database connection
│   │   ├── repositories/ # Data access layer
│   │   └── migrations/   # SQL migrations
│   │
│   ├── amail-agents/     # AI agent implementations
│   │   ├── actor.rs      # Base Actix actor
│   │   ├── registry.rs   # Agent discovery
│   │   ├── supervisor.rs # Supervision tree
│   │   ├── pipeline.rs   # Processing pipeline
│   │   ├── prompts.rs    # LLM prompts
│   │   └── agents/       # Specialized agents
│   │
│   ├── amail-webhook/    # HTTP API
│   │   ├── handlers.rs   # Webhook handlers
│   │   ├── api.rs        # REST endpoints
│   │   ├── p2p_api.rs    # P2P endpoints
│   │   ├── stalwart.rs   # Stalwart client
│   │   └── websocket.rs  # Dashboard WebSocket
│   │
│   ├── amail-llm/        # LLM integration
│   │   ├── client.rs     # LLM client trait
│   │   ├── ollama.rs     # Ollama implementation
│   │   └── tools.rs      # Tool definitions
│   │
│   ├── amail-mcp/        # Model Context Protocol
│   │   ├── client.rs     # MCP client
│   │   └── types.rs      # MCP types
│   │
│   ├── amail-mail/       # Email handling
│   │   ├── smtp.rs       # Embedded SMTP server
│   │   ├── parser.rs     # MIME parsing
│   │   └── auth.rs       # DKIM signing
│   │
│   ├── amail-p2p/        # P2P file transfer
│   │   ├── node.rs       # libp2p node
│   │   ├── chunk.rs      # File chunking
│   │   ├── transfer.rs   # Transfer management
│   │   ├── protocol.rs   # Wire protocol
│   │   └── discovery.rs  # Peer discovery
│   │
│   └── amail-dashboard/  # Web UI (Leptos/WASM)
│       ├── pages/        # Dashboard pages
│       ├── api.rs        # API client
│       └── websocket.rs  # WebSocket client
```

---

## Security Model

### Authentication
- Webhook signature verification (HMAC-SHA256)
- Stalwart API key authentication
- No external API keys required (local LLM)

### Data Privacy
- All data stays on your server
- No cloud dependencies
- Local LLM inference
- P2P transfers are direct (no relay servers)

### Network Security
- TLS for SMTP (STARTTLS)
- HTTPS for API (configure reverse proxy)
- libp2p encryption for P2P

---

## Scaling Considerations

### Single Server (Recommended for most users)
- In-memory task queue
- Single Ollama instance
- PostgreSQL on same machine
- Handles thousands of emails/day

### Multi-Server (High volume)
- Redis for distributed queue
- Multiple Amail workers
- Separate database server
- Load balancer for API

### Resource Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| RAM | 4GB | 8GB+ |
| CPU | 2 cores | 4+ cores |
| Storage | 10GB | 50GB+ |
| GPU | None | NVIDIA (for faster LLM) |
