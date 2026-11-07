# Amail - Self-Hosted AI Email Server

<p align="center">
  <strong>100% Open Source • FREE Forever • Self-Hosted • No Limits</strong>
</p>

<p align="center">
  <a href="#quick-start">Quick Start</a> •
  <a href="./START.md">Full Setup Guide</a> •
  <a href="./docs/">Documentation</a> •
  <a href="#api-endpoints">API Reference</a>
</p>

---

Break free from expensive Google Workspace and Microsoft 365 subscriptions. Amail is a complete, self-hosted email server with AI-powered auto-reply, unlimited domains, unlimited mailboxes, and P2P file transfers with no size limits.

Amail is a universal email application for Windows, macOS, and Linux. It serves NOVA, Grok Bot, and any compatible client. Build from source for the system you run.

> *"No more expensive per-user email bills. Business emails shouldn't be gatekept."*

## Why Amail?

| Feature | Google Workspace | Microsoft 365 | **Amail** |
|---------|-----------------|---------------|-----------|
| Price | $6-18/user/month | $6-22/user/month | **FREE** |
| Users | Pay per user | Pay per user | **Unlimited** |
| Domains | Limited | Limited | **Unlimited** |
| Storage | 30GB-5TB | 1TB | **Your hardware** |
| AI Auto-Reply | ❌ | ❌ | **✅ Local LLM** |
| Attachment Limit | 25MB | 25MB | **No limit (P2P)** |
| Data Privacy | Their servers | Their servers | **Your servers** |
| Open Source | ❌ | ❌ | **✅ MIT License** |

## Key Features

- 🆓 **100% FREE** - No subscriptions, no pricing tiers, no monetization
- 🏠 **Self-Hosted** - You own your server, your data, your privacy
- 🤖 **AI Auto-Reply** - 30+ specialized agents with local LLM (Ollama/Mistral)
- 📧 **Unlimited Everything** - Domains, mailboxes, users, storage
- 🔗 **P2P Transfers** - Torrent-style file sharing, no 10MB limits
- ⏰ **Auto-Startup** - Configurable daily start time (default 8am)
- 📱 **Telegram Integration** - Human review notifications when AI confidence is low
- 🛡️ **DNS Setup Wizard** - MX, SPF, DKIM, DMARC verification
- 📊 **Real-time Dashboard** - WebSocket-powered analytics
- 🔄 **Mail Migration** - P2P transfer between Amail servers

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     STALWART MAIL SERVER                         │
│  (Rust-native SMTP/IMAP/JMAP with webhook support)              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Webhook POST
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     AMAIL WEBHOOK RECEIVER                       │
│  (Actix-Web endpoint with HMAC signature verification)          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Task Queue
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     AGENT SWARM (Actix Actors)                   │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │ Receptionist │──│    Clerk     │──│  Executive   │          │
│  │   (Triage)   │  │ (Scheduling) │  │ (Decisions)  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│         │                 │                 │                    │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │Draft Checker │  │  Accountant  │  │    Lawyer    │          │
│  │   (Review)   │  │  (Finance)   │  │   (Legal)    │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
│                                                                  │
│  + 25 more specialized agents (HR, IT, Marketing, Sales, etc.)  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ LLM Calls
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     OLLAMA + MISTRAL                             │
│  (Local LLM inference with tool calling)                        │
└─────────────────────────────────────────────────────────────────┘
```

## Technical Features

- **Rust-First**: Memory-safe, blazing fast, production-ready
- **Actix Actors**: Supervised agent lifecycle with Erlang/OTP-style supervision trees
- **A2A Protocol**: Agent-to-agent communication for task delegation
- **Local LLM**: Privacy-preserving inference via Ollama (Mistral, Llama, etc.)
- **Stalwart Integration**: Native webhook support from Rust mail server
- **MCP Support**: Connect to Model Context Protocol servers for extended capabilities
- **PostgreSQL**: Production-grade persistence with auto-migrations
- **libp2p**: Decentralized P2P file transfers with DHT and mDNS discovery

## Quick Start

```bash
# 1. Clone and configure
git clone https://github.com/yourusername/amail.git
cd amail
cp .env.example .env

# 2. Set up database
createdb amail
# Edit .env: DATABASE_URL=postgres://user:pass@localhost:5432/amail

# 3. Install Ollama and pull model
ollama pull mistral:latest

# 4. Build and run
cargo build --release
cargo run --release --bin amail-server
```

Dashboard available at `http://localhost:4000`

**For complete setup instructions, see [START.md](./START.md)**

### Prerequisites

| Component | Required | Purpose |
|-----------|----------|----------|
| Rust | ✅ | Build the server |
| Ollama | ✅ | Local LLM inference |
| Supabase/PostgreSQL | Optional | Cloud database (uses JSON files if not set) |
| Stalwart | Optional | Full SMTP/IMAP (Amail has embedded SMTP) |
| Redis | Optional | Distributed task queue |

### Storage Options

**Option 1: JSON Files (Default - No Setup Required)**
- Domains and mailboxes stored in `data/config/`
- Perfect for single-server deployments
- No database setup needed

**Option 2: Supabase (Recommended for Cloud)**
```bash
# Add to .env
DATABASE_URL=postgres://postgres.[PROJECT_REF]:[PASSWORD]@aws-0-[REGION].pooler.supabase.com:6543/postgres
```

**Option 3: Local PostgreSQL**
```bash
# Add to .env
DATABASE_URL=postgres://amail:password@localhost:5432/amail
```

### Email & Attachment Storage

| Mode | Emails | Attachments | Best For |
|------|--------|-------------|----------|
| **Local** (default) | `./data/emails/` | `./data/attachments/` | Single server, self-hosted |
| **Supabase** | Supabase DB | Supabase Storage | Cloud, multi-server, backup |

**Local Storage (Default)**
```bash
STORAGE_MODE=local
STORAGE_EMAIL_PATH=./data/emails
STORAGE_ATTACHMENT_PATH=./data/attachments
```

**Supabase Storage (Cloud)**
```bash
STORAGE_MODE=supabase
DATABASE_URL=postgres://postgres.[PROJECT]:[PASS]@aws-0-[REGION].pooler.supabase.com:6543/postgres
SUPABASE_STORAGE_BUCKET=amail-attachments
SUPABASE_ANON_KEY=your_anon_key
```

> **Note**: Raw emails are stored by Stalwart Mail Server (or embedded SMTP). Amail stores processed metadata, AI responses, and attachments.

## Project Structure

```
amail/
├── Cargo.toml                 # Workspace configuration
├── src/
│   ├── main.rs               # Server entry point
│   └── worker.rs             # Background worker
├── crates/
│   ├── amail-core/           # Core types, traits, errors
│   │   ├── src/
│   │   │   ├── email.rs      # Email types
│   │   │   ├── agent.rs      # Agent types (30+ roles)
│   │   │   ├── task.rs       # Task types
│   │   │   ├── a2a.rs        # A2A protocol types
│   │   │   └── traits.rs     # Core traits
│   ├── amail-agents/         # Actix actor implementations
│   │   ├── src/
│   │   │   ├── actor.rs      # Base agent actor
│   │   │   ├── registry.rs   # Agent discovery
│   │   │   ├── supervisor.rs # Supervision tree
│   │   │   └── agents/       # Specialized agents
│   ├── amail-webhook/        # HTTP API & Stalwart integration
│   │   ├── src/
│   │   │   ├── handlers.rs   # Webhook handlers
│   │   │   ├── stalwart.rs   # Stalwart client
│   │   │   └── signature.rs  # HMAC verification
│   ├── amail-llm/            # LLM integration
│   │   ├── src/
│   │   │   ├── client.rs     # LLM client
│   │   │   ├── ollama.rs     # Ollama API
│   │   │   └── tools.rs      # Tool definitions
│   └── amail-mcp/            # MCP client
│       ├── src/
│       │   ├── client.rs     # MCP client
│       │   └── types.rs      # MCP types
└── docker/                   # Docker deployment files
```

## Agent Roles

### Core Agents (6)
1. **Receptionist** - Email triage and routing
2. **Clerk** - Directory management, scheduling
3. **Executive** - Onboarding, high-level decisions
4. **Draft Checker** - Output review and QA
5. **Accountant** - Finance tracking
6. **Lawyer** - Legal queries, compliance

### Extended Agents (25+)
- Human Resources Manager
- IT Support Technician
- Marketing Coordinator
- Sales Representative
- Customer Service Representative
- Project Manager
- Research Analyst
- Content Creator
- Translator
- Data Analyst
- Compliance Officer
- Security Analyst
- Archivist
- Trainer
- Strategic Planner
- Negotiator
- Internal Auditor
- Procurement Specialist
- Logistics Coordinator
- Quality Assurance Tester
- Risk Manager
- Public Relations Officer
- Event Coordinator
- Innovation Researcher
- Facilities Manager
- Ethics Reviewer

## API Endpoints

### Dashboard API
```
GET  /api/v1/stats              # Dashboard statistics
GET  /api/v1/domains            # List domains
POST /api/v1/domains            # Create domain
GET  /api/v1/mailboxes          # List mailboxes
GET  /api/v1/inbox              # Get inbox emails
GET  /api/v1/reviews            # Human review queue
GET  /api/v1/agents             # List AI agents
GET  /api/v1/metrics/daily      # Daily analytics
```

### P2P Transfer API
```
GET  /api/v1/p2p/info           # P2P node info
POST /api/v1/p2p/transfers/share    # Share a file
POST /api/v1/p2p/transfers/download # Start download
GET  /api/v1/p2p/transfers          # List transfers
GET  /api/v1/p2p/peers              # List connected peers
POST /api/v1/p2p/mailbox/export     # Export mailbox for P2P
POST /api/v1/p2p/mailbox/import     # Import mailbox from P2P
```

### Webhook (Stalwart)
```
POST /api/v1/webhook
X-Webhook-Signature: sha256=...
```

## Configuration

See `.env.example` for all configuration options including:
- Database connection (PostgreSQL)
- Stalwart mail server integration
- Ollama LLM settings
- Telegram bot for human review
- Auto-start scheduler settings

## P2P File Transfer

Amail includes BitTorrent-style P2P transfers for:
- **Large attachments** - No 10MB/25MB limits
- **Mailbox migration** - Transfer entire mailboxes between servers
- **Decentralized** - Direct peer-to-peer, no central server

```bash
# Share a file
curl -X POST http://localhost:4000/api/v1/p2p/transfers/share \
  -H "Content-Type: application/json" \
  -d '{"file_path": "/path/to/large-file.zip"}'

# Returns magnet link for sharing
# amail://uuid?xt=urn:blake3:hash&dn=filename&xl=size
```

## Crates

| Crate | Description |
|-------|-------------|
| `amail-core` | Core types, traits, PostgreSQL repositories |
| `amail-agents` | 30+ Actix actor agents |
| `amail-webhook` | HTTP API, Stalwart integration |
| `amail-llm` | Ollama client, tool calling |
| `amail-mcp` | Model Context Protocol client |
| `amail-mail` | Embedded SMTP server |
| `amail-p2p` | libp2p-based file transfers |
| `amail-dashboard` | Leptos WASM dashboard |

## Documentation

| Document | Description |
|----------|-------------|
| [START.md](./START.md) | Complete setup guide |
| [docs/architecture.md](./docs/architecture.md) | System architecture |
| [docs/agents.md](./docs/agents.md) | AI agent details |
| [docs/telegram.md](./docs/telegram.md) | Telegram bot setup |
| [docs/p2p.md](./docs/p2p.md) | P2P file transfer |
| [docs/api.md](./docs/api.md) | REST API reference |

## Environment Variables

Key configuration options (see `.env.example` for full list):

```env
# Required
DATABASE_URL=postgres://amail:password@localhost:5432/amail
OLLAMA_URL=http://localhost:11434
OLLAMA_MODEL=mistral:latest

# Mail
MAIL_DOMAIN=yourdomain.com
MAIL_BOXES=inbox,support:customer_service,hr:human_resources

# Telegram (optional but recommended)
TELEGRAM_BOT_TOKEN=your_bot_token
TELEGRAM_CHAT_ID=your_chat_id
TELEGRAM_ENABLED=true

# Auto-start
AMAIL_AUTO_START_ENABLED=true
AMAIL_DAILY_START_TIME=08:00
```

## License

MIT - Use it, modify it, sell it, whatever. Just don't blame us.

## Philosophy

Amail exists because email shouldn't cost money per user. It's a basic communication tool that's been monetized into expensive subscriptions. We believe:

- **Email is infrastructure** - Like roads, it should be freely accessible
- **Privacy matters** - Your emails shouldn't train someone else's AI
- **Self-hosting is freedom** - Own your data, own your destiny
- **AI should be local** - No cloud APIs, no data leaks

## Contributing

Contributions welcome! This is a community project.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## Star History

If Amail saves you money, consider starring the repo! ⭐
