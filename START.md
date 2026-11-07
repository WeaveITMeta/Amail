# Amail - Complete Setup Guide

**Self-Hosted AI Email Server - 100% FREE Forever**

This guide walks you through setting up Amail from scratch on your own server.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start (5 minutes)](#quick-start)
3. [Environment Configuration](#environment-configuration)
5. [Ollama LLM Setup](#ollama-llm-setup)
6. [Mail Server Setup (Stalwart)](#mail-server-setup)
7. [DNS Configuration](#dns-configuration)
8. [Telegram Bot Setup](#telegram-bot-setup)
9. [Running Amail](#running-amail)
10. [Auto-Start Configuration](#auto-start-configuration)
11. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required
- **Rust** (latest stable) - [Install Rust](https://rustup.rs/)
- **Ollama** - Local LLM inference - [Install Ollama](https://ollama.ai/)

### Optional
- **Supabase/PostgreSQL** - Cloud or local database (Amail uses JSON files by default)
- **Redis** - For distributed task queue (single-server deployments don't need this)
- **Stalwart Mail Server** - If you want full SMTP/IMAP (Amail has embedded SMTP)
- **Domain name** - For receiving external emails

### System Requirements
- **RAM**: 4GB minimum (8GB+ recommended for LLM)
- **Storage**: 10GB+ for database and attachments
- **OS**: Windows, Linux, or macOS

Amail is used on Windows, macOS, and Linux through any compatible client, including NOVA and Grok Bot. The steps below build from source on the system you choose.

---

## Quick Start

```bash
# 1. Clone the repository
git clone https://github.com/yourusername/amail.git
cd amail

# 2. Copy environment file
cp .env.example .env

# 3. Edit .env with your settings (see Configuration section)
# No database required - Amail uses JSON files by default

# 4. Install Ollama and pull Mistral
ollama pull mistral:latest

# 5. Build and run
cargo build --release
cargo run --release --bin amail-server
```

Amail will be available at `http://localhost:4000`

---

## Storage

Amail uses **JSON files by default** - no database setup required!

Data is stored in `data/config/`:
- `domains.json` - Managed email domains
- `mailboxes.json` - Email addresses per domain  
- `dns.json` - DNS configuration
- `settings.json` - System settings

### Optional: Supabase/PostgreSQL

For cloud sync or multi-server deployments, you can optionally use Supabase:

```env
# Add to .env for Supabase support
DATABASE_URL=postgres://user:password@db.supabase.co:5432/postgres
```

When configured, Amail uses Supabase as primary storage with JSON as fallback.

---

## Environment Configuration

Copy `.env.example` to `.env` and configure:

### Essential Settings

```env
# Server
HOST=127.0.0.1
PORT=4000

# LLM (REQUIRED)
OLLAMA_URL=http://localhost:11434
OLLAMA_MODEL=mistral:latest

# Database (OPTIONAL - uses JSON files if not set)
# DATABASE_URL=postgres://user:password@localhost:5432/amail

# Your email domain
MAIL_DOMAIN=yourdomain.com
MAIL_ORG_NAME=Your Company
```

### Mailbox Configuration

```env
# Define mailboxes to monitor
# Format: name or name:agent_role
MAIL_BOXES=inbox,support:customer_service,hr:human_resources,legal:lawyer,finance:accountant

# Default mailbox for unrouted emails
MAIL_DEFAULT_BOX=inbox
```

### Security

```env
# Generate webhook secret: openssl rand -hex 32
WEBHOOK_SECRET=your_64_character_hex_secret

# Stalwart API (if using external mail server)
STALWART_URL=http://localhost:4040
STALWART_API_KEY=your_api_key
```

### Telegram (Optional but Recommended)

```env
TELEGRAM_BOT_TOKEN=123456789:ABCdefGHIjklMNOpqrsTUVwxyz
TELEGRAM_CHAT_ID=your_chat_id
TELEGRAM_ENABLED=true
TELEGRAM_REVIEW_THRESHOLD=75
```

### Auto-Start Schedule

```env
AMAIL_AUTO_START_ENABLED=true
AMAIL_DAILY_START_TIME=08:00
AMAIL_DAILY_SUMMARY_ENABLED=true
AMAIL_PROCESS_OVERNIGHT_EMAILS=true
AMAIL_ACTIVATE_AGENTS_ON_START=true
```

---

## Ollama LLM Setup

### Install Ollama

**Windows:**
Download from [ollama.ai](https://ollama.ai/) and run installer.

**Linux:**
```bash
curl -fsSL https://ollama.ai/install.sh | sh
```

**macOS:**
```bash
brew install ollama
```

### Pull the Model

```bash
# Recommended: Mistral (best balance of speed/quality)
ollama pull mistral:latest

# Alternative: Llama 3 (better reasoning, slower)
ollama pull llama3:latest

# Alternative: Phi-3 (faster, smaller)
ollama pull phi3:latest
```

### Start Ollama

**Single Instance (Default):**
```bash
ollama serve
```

**Multiple Instances (Recommended for 30+ agents):**

Run this single command to start 3 Ollama instances for parallel LLM processing:

**Windows (PowerShell):**
```powershell
Start-Process powershell -ArgumentList "-Command", "ollama serve" ; Start-Process powershell -ArgumentList "-Command", "`$env:OLLAMA_HOST='0.0.0.0:11435'; ollama serve" ; Start-Process powershell -ArgumentList "-Command", "`$env:OLLAMA_HOST='0.0.0.0:11436'; ollama serve"
```

**Linux/macOS (Bash):**
```bash
ollama serve & OLLAMA_HOST=0.0.0.0:11435 ollama serve & OLLAMA_HOST=0.0.0.0:11436 ollama serve &
```

Then configure in `.env`:
```env
OLLAMA_URLS=http://localhost:11434,http://localhost:11435,http://localhost:11436
```

Ollama runs on `http://localhost:11434` by default (with additional instances on 11435, 11436).

### Verify

```bash
curl http://localhost:11434/api/tags
```

---

## Mail Server Setup

### Option 1: Embedded SMTP (Simplest)

Amail includes an embedded SMTP server. Just configure:

```env
SMTP_HOST=localhost
SMTP_PORT=2525
```

This is sufficient for development and small deployments.

### Option 2: Stalwart Mail Server (Production)

For full SMTP/IMAP/JMAP support, use Stalwart:

**Install Stalwart:**
```bash
# Download from https://stalw.art/
# Or use Docker:
docker run -d -p 25:25 -p 587:587 -p 993:993 -p 4040:8080 \
  -v stalwart-data:/opt/stalwart-mail \
  stalwartlabs/mail-server:latest
```

**Configure Stalwart webhook:**

In Stalwart's `config.toml`:
```toml
[webhook.amail]
url = "http://localhost:4000/api/v1/webhook"
events = ["message.received", "message.delivered"]
secret = "your_webhook_secret_here"  # Same as WEBHOOK_SECRET in .env
```

**Configure Amail:**
```env
STALWART_URL=http://localhost:4040
STALWART_API_KEY=your_stalwart_api_key
STALWART_ADMIN_USER=admin
STALWART_ADMIN_PASS=your_password
```

---

## DNS Configuration

To receive emails from the internet, configure DNS for your domain:

### Required Records

| Type | Name | Value | TTL |
|------|------|-------|-----|
| **MX** | @ | `mail.yourdomain.com` | 3600 |
| **A** | mail | `YOUR_SERVER_IP` | 3600 |

### Recommended Records (Deliverability)

| Type | Name | Value |
|------|------|-------|
| **SPF** | @ | `v=spf1 mx a ip4:YOUR_IP ~all` |
| **DKIM** | `amail._domainkey` | (Generated by Amail) |
| **DMARC** | `_dmarc` | `v=DMARC1; p=quarantine; rua=mailto:dmarc@yourdomain.com` |

### Verify DNS

Use the DNS Wizard in Amail dashboard or:
```bash
# Check MX
dig MX yourdomain.com

# Check SPF
dig TXT yourdomain.com

# Check DKIM
dig TXT amail._domainkey.yourdomain.com
```

---

## Telegram Bot Setup

Telegram integration enables:
- Human review notifications for low-confidence AI responses
- Daily digest summaries
- Quick approve/reject buttons

### Step 1: Create Bot

1. Open Telegram and search for `@BotFather`
2. Send `/newbot`
3. Follow prompts to name your bot
4. Copy the **bot token** (looks like `123456789:ABCdefGHI...`)

### Step 2: Get Your Chat ID

1. Start a chat with your new bot (click Start)
2. Search for `@userinfobot` on Telegram
3. Send `/start` to @userinfobot
4. Copy your **chat ID** (a number like `123456789`)

### Step 3: Configure .env

```env
TELEGRAM_BOT_TOKEN=123456789:ABCdefGHIjklMNOpqrsTUVwxyz
TELEGRAM_CHAT_ID=123456789
TELEGRAM_ENABLED=true

# AI confidence below this triggers Telegram notification
TELEGRAM_REVIEW_THRESHOLD=75

# AI confidence at or above this auto-sends without review
TELEGRAM_AUTO_APPROVE_THRESHOLD=90
```

### Step 4: Test

Start Amail and send a test email. If AI confidence is below threshold, you'll receive a Telegram message with approve/reject buttons.

---

## Running Amail

### Development Mode

```bash
cargo run --bin amail-server
```

### Production Mode

```bash
# Build optimized binary
cargo build --release

# Run
./target/release/amail-server
```

### With Logging

```bash
RUST_LOG=info cargo run --release --bin amail-server
```

### Verify Running

- **Dashboard**: http://localhost:4000
- **API Health**: http://localhost:4000/api/v1/health
- **WebSocket**: ws://localhost:4000/ws/dashboard

---

## Auto-Start Configuration

### Windows (Task Scheduler)

```powershell
# Run as Administrator
$TaskName = "Amail-AutoStart"
$ExePath = "C:\path\to\amail-server.exe"

# Create triggers
$DailyTrigger = New-ScheduledTaskTrigger -Daily -At "08:00"
$BootTrigger = New-ScheduledTaskTrigger -AtStartup

# Register task
Register-ScheduledTask -TaskName $TaskName `
    -Action (New-ScheduledTaskAction -Execute $ExePath) `
    -Trigger $DailyTrigger,$BootTrigger `
    -Description "Start Amail AI Mail Server"
```

### Linux (systemd)

Create `/etc/systemd/system/amail.service`:

```ini
[Unit]
Description=Amail AI Mail Server
After=network.target

[Service]
Type=simple
User=amail
WorkingDirectory=/opt/amail
ExecStart=/opt/amail/amail-server
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable amail
sudo systemctl start amail
```

### macOS (launchd)

Create `~/Library/LaunchAgents/com.amail.server.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.amail.server</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/amail-server</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>
```

Load:
```bash
launchctl load ~/Library/LaunchAgents/com.amail.server.plist
```

---

## Troubleshooting

### Database Connection Failed (Optional)

If you've configured DATABASE_URL and see:
```
Error: connection refused
```

**Solution:**
1. Remove DATABASE_URL from .env to use JSON storage (recommended for single-server)
2. Or ensure PostgreSQL/Supabase is running and credentials are correct

### Ollama Not Responding

```
Error: Failed to connect to Ollama
```

**Solution:**
1. Start Ollama: `ollama serve`
2. Check URL: `curl http://localhost:11434/api/tags`
3. Pull model: `ollama pull mistral:latest`

### Port Already in Use

```
Error: Address already in use (os error 98)
```

**Solution:**
```bash
# Find process using port
lsof -i :4000
# or on Windows
netstat -ano | findstr :4000

# Kill it or change PORT in .env
```

### Webhook Signature Invalid

```
Error: Invalid webhook signature
```

**Solution:**
1. Ensure WEBHOOK_SECRET matches in both Amail and Stalwart
2. Regenerate: `openssl rand -hex 32`

### Emails Not Being Processed

**Check:**
1. Webhook is configured in Stalwart
2. Mailbox exists in MAIL_BOXES
3. Agent is running: check dashboard at http://localhost:4000

### Low AI Confidence

If AI responses consistently have low confidence:
1. Try a larger model: `ollama pull llama3:latest`
2. Increase timeout: `OLLAMA_TIMEOUT=180`
3. Check prompts in `crates/amail-agents/src/prompts.rs`

---

## Next Steps

1. **Add domains** in the dashboard
2. **Configure DNS** for your domain
3. **Set up Telegram** for human review
4. **Customize agents** in settings
5. **Monitor analytics** for performance

---

## Support

- **GitHub Issues**: Report bugs and request features
- **Documentation**: See `/docs` folder
- **Dashboard**: Built-in help at http://localhost:4000

---

**Amail - Your emails, your server, your AI. Free forever.**
