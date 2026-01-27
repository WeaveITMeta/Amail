# REST API Reference

Amail exposes a REST API for dashboard integration and external applications.

**Base URL**: `http://localhost:4000/api/v1`

---

## Authentication

Currently, the API is designed for local/trusted network use. For production deployments behind a reverse proxy, implement authentication at the proxy level.

---

## Endpoints

### Health Check

```http
GET /health
```

Response:
```json
{
  "status": "ok",
  "version": "0.1.0",
  "uptime_seconds": 3600
}
```

---

## Dashboard Stats

### Get Dashboard Statistics

```http
GET /stats
```

Response:
```json
{
  "active_agents": 6,
  "total_agents": 30,
  "pending_tasks": 3,
  "active_tasks": 1,
  "pending_reviews": 2,
  "emails_today": 47,
  "unread_emails": 12,
  "tasks_week": 234,
  "avg_response_time_ms": 2340.5
}
```

---

## Domains

### List Domains

```http
GET /domains
```

Response:
```json
[
  {
    "id": "d-001",
    "name": "example.com",
    "status": "verified",
    "mx_verified": true,
    "spf_verified": true,
    "dkim_verified": true,
    "dmarc_verified": false,
    "mailbox_count": 5,
    "created_at": "2024-12-15T10:00:00Z"
  }
]
```

### Create Domain

```http
POST /domains
Content-Type: application/json

{
  "name": "newdomain.com"
}
```

Response:
```json
{
  "id": "d-002",
  "name": "newdomain.com",
  "status": "pending",
  "mx_verified": false,
  "spf_verified": false,
  "dkim_verified": false,
  "dmarc_verified": false,
  "mailbox_count": 0,
  "created_at": "2024-12-20T15:30:00Z"
}
```

### Delete Domain

```http
DELETE /domains/{domain_id}
```

Response: `204 No Content`

### Get Domain Mailboxes

```http
GET /domains/{domain_id}/mailboxes
```

---

## Mailboxes

### List All Mailboxes

```http
GET /mailboxes
```

Response:
```json
[
  {
    "id": "m-001",
    "email": "support@example.com",
    "display_name": "Support",
    "domain": "example.com",
    "mailbox_type": "customer_service",
    "is_active": true,
    "unread_count": 5
  }
]
```

### Create Mailbox

```http
POST /mailboxes
Content-Type: application/json

{
  "domain_id": "d-001",
  "local_part": "sales",
  "display_name": "Sales Team"
}
```

---

## Inbox

### List Emails

```http
GET /inbox?mailbox_id={id}&folder={folder}&search={query}&limit={n}&offset={n}
```

Query Parameters:
- `mailbox_id` - Filter by mailbox (optional)
- `folder` - inbox, sent, drafts, trash (default: inbox)
- `search` - Search query (optional)
- `limit` - Results per page (default: 50)
- `offset` - Pagination offset (default: 0)

Response:
```json
[
  {
    "id": "e-001",
    "from_address": "john@example.com",
    "from_name": "John Smith",
    "to": ["support@yourdomain.com"],
    "subject": "Order inquiry",
    "preview": "Hi, I have a question about...",
    "timestamp": "2024-12-20T10:30:00Z",
    "is_read": false,
    "is_starred": true,
    "folder": "inbox",
    "labels": ["important"],
    "mailbox_email": "support@yourdomain.com",
    "mailbox_name": "Support",
    "ai_confidence": 0.85,
    "ai_status": "auto_sent",
    "attachment_count": 2
  }
]
```

### Get Email Details

```http
GET /emails/{email_id}
```

Response:
```json
{
  "id": "e-001",
  "from_address": "john@example.com",
  "from_name": "John Smith",
  "to": ["support@yourdomain.com"],
  "cc": [],
  "subject": "Order inquiry",
  "text_body": "Hi, I have a question...",
  "html_body": "<p>Hi, I have a question...</p>",
  "received_at": "2024-12-20T10:30:00Z",
  "is_read": true,
  "is_starred": true,
  "folder": "inbox",
  "labels": ["important"],
  "attachments": [
    {
      "id": "a-001",
      "filename": "invoice.pdf",
      "content_type": "application/pdf",
      "size": 245000
    }
  ]
}
```

### Update Email

```http
PATCH /emails/{email_id}
Content-Type: application/json

{
  "is_read": true,
  "is_starred": false,
  "folder": "archive"
}
```

---

## Human Reviews

### List Pending Reviews

```http
GET /reviews
```

Response:
```json
[
  {
    "id": "r-001",
    "task_id": "t-001",
    "email_id": "e-001",
    "reason": "low_confidence",
    "urgency": "normal",
    "status": "pending",
    "confidence_score": 0.65,
    "created_at": "2024-12-20T10:35:00Z",
    "from_address": "john@example.com",
    "from_name": "John Smith",
    "subject": "Partnership inquiry",
    "email_preview": "Hi, I'm interested in...",
    "draft_response": "Thank you for reaching out...",
    "agent_role": "Sales"
  }
]
```

### Submit Review Decision

```http
POST /reviews/{review_id}/decision
Content-Type: application/json

{
  "decision": "approved",
  "reviewed_by": "admin",
  "reviewer_notes": "Looks good",
  "edited_response": null
}
```

Decision values: `approved`, `rejected`, `edited`

---

## Agents

### List Agents

```http
GET /agents
```

Response:
```json
[
  {
    "id": "agent-001",
    "role": "Receptionist",
    "status": "idle",
    "last_heartbeat": "2024-12-20T15:30:00Z",
    "tasks_processed": 156
  }
]
```

---

## Metrics

### Daily Metrics

```http
GET /metrics/daily?period={period}
```

Period values: `today`, `week`, `month`, `year`

Response:
```json
[
  {
    "date": "2024-12-20",
    "emails_received": 47,
    "emails_sent": 42,
    "emails_auto_replied": 38,
    "tasks_created": 47,
    "tasks_completed": 45,
    "tasks_failed": 2,
    "avg_response_time_ms": 2340,
    "reviews_requested": 6,
    "reviews_approved": 4,
    "reviews_rejected": 1,
    "llm_calls": 89,
    "llm_tokens_used": 45000
  }
]
```

### Hourly Metrics

```http
GET /metrics/hourly?period={period}
```

Response:
```json
[
  {
    "hour": "2024-12-20T10:00:00Z",
    "emails_received": 8,
    "emails_sent": 7,
    "tasks_completed": 8,
    "avg_response_time_ms": 2100
  }
]
```

---

## P2P Transfers

### Get P2P Node Info

```http
GET /p2p/info
```

### List Transfers

```http
GET /p2p/transfers
```

### Share File

```http
POST /p2p/transfers/share
Content-Type: application/json

{
  "file_path": "/path/to/file.zip"
}
```

### Start Download

```http
POST /p2p/transfers/download
Content-Type: application/json

{
  "transfer_info": { ... },
  "output_path": "/downloads/file.zip"
}
```

### Transfer Control

```http
POST /p2p/transfers/{id}/pause
POST /p2p/transfers/{id}/resume
POST /p2p/transfers/{id}/cancel
```

### List Peers

```http
GET /p2p/peers
```

### Mailbox Export/Import

```http
POST /p2p/mailbox/export
POST /p2p/mailbox/import
```

---

## Webhook

### Stalwart Webhook Endpoint

```http
POST /webhook
X-Webhook-Signature: sha256=...
Content-Type: application/json

{
  "event": "message.received",
  "message_id": "...",
  "sender": "user@example.com",
  "recipients": ["inbox@yourdomain.com"],
  "timestamp": "2024-12-20T10:30:00Z"
}
```

The signature is HMAC-SHA256 of the request body using `WEBHOOK_SECRET`.

---

## Error Responses

All errors return JSON:

```json
{
  "error": "Error message here",
  "code": "ERROR_CODE"
}
```

### HTTP Status Codes

| Code | Meaning |
|------|---------|
| 200 | Success |
| 201 | Created |
| 204 | No Content (successful delete) |
| 400 | Bad Request |
| 401 | Unauthorized |
| 404 | Not Found |
| 500 | Internal Server Error |
| 503 | Service Unavailable |

---

## Rate Limiting

Currently no rate limiting is implemented. For production, implement at reverse proxy level.

---

## WebSocket

### Dashboard WebSocket

```
ws://localhost:4000/ws/dashboard
```

Receives real-time updates:
```json
{
  "type": "agent_status",
  "data": {
    "agent_id": "agent-001",
    "status": "busy",
    "current_task": "t-001"
  }
}
```

Event types:
- `agent_status` - Agent state changes
- `task_created` - New task queued
- `task_completed` - Task finished
- `email_received` - New email arrived
- `review_requested` - Human review needed
- `stats_update` - Dashboard stats changed
