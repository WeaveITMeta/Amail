# Amail Docker Setup

This directory contains Docker configuration for the Amail development stack.

## Services

| Service | Port | Description |
|---------|------|-------------|
| Stalwart | 4040 | Mail server REST API & Admin UI |
| Stalwart | 25, 587, 465 | SMTP (plain, submission, TLS) |
| Stalwart | 143, 993 | IMAP (plain, TLS) |
| PostgreSQL | 5432 | Database |
| Redis | 6379 | Task queue |

## Quick Start

```powershell
# Start all services
.\setup.ps1 start

# Initialize mailboxes from .env config
.\setup.ps1 init

# View logs
.\setup.ps1 logs

# Stop all services
.\setup.ps1 stop
```

## Configuration

1. Copy `..\.env.example` to `..\.env`
2. Configure your mail domain and mailboxes:
   ```env
   MAIL_DOMAIN=yourdomain.com
   MAIL_BOXES=inbox,support,hr,legal,finance
   ```
3. Run `.\setup.ps1 init` to create mailboxes

## Stalwart Admin

- URL: http://localhost:4040
- Default credentials: `admin` / `admin123`

## Webhook Setup

Stalwart is pre-configured to send webhooks to Amail at:
```
http://host.docker.internal:4000/api/v1/webhook/stalwart
```

This triggers when emails are received, allowing Amail agents to process them.

## Testing Email Flow

1. Start the stack: `.\setup.ps1 start`
2. Start Amail server: `cargo run --bin amail-server`
3. Send a test email to `inbox@localhost` using any mail client
4. Watch the Amail dashboard for agent activity

## Volumes

Data is persisted in Docker volumes:
- `stalwart-data` - Mail storage
- `postgres-data` - Database
- `redis-data` - Queue data

To reset all data:
```powershell
docker-compose down -v
```
