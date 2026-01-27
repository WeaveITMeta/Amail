# Amail Documentation

**Self-Hosted AI Email Server - 100% Open Source, FREE Forever**

Welcome to the Amail documentation. Amail is a complete, self-hosted email server with AI-powered auto-reply, unlimited domains, and P2P file transfers.

---

## Documentation Index

### Getting Started
- [Quick Start Guide](./quickstart.md) - Get Amail running in 5 minutes
- [Installation](./installation.md) - Detailed installation instructions
- [Configuration](./configuration.md) - Environment variables and settings

### Core Concepts
- [Architecture](./architecture.md) - How Amail works
- [AI Agents](./agents.md) - The 30+ AI agents and their roles
- [Email Processing](./email-processing.md) - How emails flow through the system

### Features
- [Telegram Integration](./telegram.md) - Human review via Telegram
- [P2P File Transfer](./p2p.md) - BitTorrent-style attachments
- [DNS Setup](./dns.md) - Configure MX, SPF, DKIM, DMARC
- [Analytics](./analytics.md) - Dashboard metrics and KPIs

### API Reference
- [REST API](./api.md) - HTTP endpoints
- [WebSocket API](./websocket.md) - Real-time updates

### Deployment
- [Production Deployment](./deployment.md) - Running in production
- [Docker](./docker.md) - Container deployment
- [Reverse Proxy](./reverse-proxy.md) - Nginx/Caddy configuration

### Development
- [Contributing](./contributing.md) - How to contribute
- [Crate Structure](./crates.md) - Code organization

---

## Why Amail?

| Feature | Google Workspace | Microsoft 365 | **Amail** |
|---------|-----------------|---------------|-----------|
| Price | $6-18/user/month | $6-22/user/month | **FREE** |
| Users | Pay per user | Pay per user | **Unlimited** |
| Domains | Limited | Limited | **Unlimited** |
| AI Auto-Reply | ❌ | ❌ | **✅ Local LLM** |
| Attachment Limit | 25MB | 25MB | **No limit (P2P)** |
| Data Privacy | Their servers | Their servers | **Your servers** |
| Open Source | ❌ | ❌ | **✅ MIT License** |

---

## Quick Links

- **GitHub**: [github.com/yourusername/amail](https://github.com/yourusername/amail)
- **Issues**: Report bugs and request features
- **License**: MIT - Use it however you want

---

## Philosophy

Amail exists because email shouldn't cost money per user. It's a basic communication tool that's been monetized into expensive subscriptions.

- **Email is infrastructure** - Like roads, it should be freely accessible
- **Privacy matters** - Your emails shouldn't train someone else's AI
- **Self-hosting is freedom** - Own your data, own your destiny
- **AI should be local** - No cloud APIs, no data leaks
