# Telegram Integration

Amail integrates with Telegram to provide human-in-the-loop review for AI-generated email responses. When the AI's confidence is below a threshold, you receive a Telegram notification with approve/reject buttons.

---

## Why Telegram?

- **Instant notifications** - Review emails from your phone
- **Quick actions** - Approve/reject with one tap
- **Daily digests** - Morning summary of overnight activity
- **No app to install** - Use existing Telegram app
- **Secure** - End-to-end encrypted messages

---

## Setup Guide

### Step 1: Create a Telegram Bot

1. Open Telegram and search for **@BotFather**
2. Send `/newbot`
3. Choose a name for your bot (e.g., "My Amail Bot")
4. Choose a username (must end in `bot`, e.g., `myamail_bot`)
5. BotFather will give you a **token** like:
   ```
   123456789:ABCdefGHIjklMNOpqrsTUVwxyz
   ```
6. Save this token - you'll need it for configuration

### Step 2: Get Your Chat ID

1. Start a conversation with your new bot (search for it and click Start)
2. Search for **@userinfobot** on Telegram
3. Send `/start` to @userinfobot
4. It will reply with your **chat ID** (a number like `123456789`)

### Step 3: Configure Amail

Add to your `.env` file:

```env
# Telegram Bot Configuration
TELEGRAM_BOT_TOKEN=123456789:ABCdefGHIjklMNOpqrsTUVwxyz
TELEGRAM_CHAT_ID=123456789
TELEGRAM_ENABLED=true

# Confidence Thresholds (0-100)
# Below this: Send to Telegram for human review
TELEGRAM_REVIEW_THRESHOLD=75

# At or above this: Auto-send without review
TELEGRAM_AUTO_APPROVE_THRESHOLD=90
```

### Step 4: Test the Integration

1. Start Amail
2. Send a test email to one of your mailboxes
3. If AI confidence is below 75%, you should receive a Telegram message

---

## How It Works

### Confidence Levels

```
100% ─────────────────────────────────────────
      │ AUTO-SEND
      │ High confidence, no review needed
 90% ─┼─────────────────────────────────────── TELEGRAM_AUTO_APPROVE_THRESHOLD
      │ SEND WITH LOGGING
      │ Good confidence, logged for audit
 75% ─┼─────────────────────────────────────── TELEGRAM_REVIEW_THRESHOLD
      │ HUMAN REVIEW
      │ Sent to Telegram for approval
  0% ─────────────────────────────────────────
```

### Review Flow

```
Email Received
      │
      ▼
AI Generates Response
      │
      ▼
Calculate Confidence Score
      │
      ├── ≥90%: Auto-send immediately
      │
      ├── 75-89%: Send, log for review
      │
      └── <75%: Send to Telegram
                    │
                    ▼
            ┌───────────────┐
            │   Telegram    │
            │   Message     │
            │               │
            │ [✅ Approve]  │
            │ [❌ Reject]   │
            │ [✏️ Edit]     │
            └───────────────┘
                    │
          ┌─────────┼─────────┐
          │         │         │
          ▼         ▼         ▼
       Approve    Reject     Edit
          │         │         │
          ▼         ▼         ▼
     Send Email  Discard   Open in
                          Dashboard
```

---

## Telegram Message Format

When a review is needed, you'll receive a message like:

```
📧 New Email Review Required

From: john@example.com
To: support@yourdomain.com
Subject: Order inquiry

Original Message:
"Hi, I ordered product #12345 last week..."

AI Draft Response:
"Thank you for contacting us about order #12345.
I can see your order is currently in transit..."

🤖 Agent: Customer Service
📊 Confidence: 68%
⏰ Received: 2 minutes ago

[✅ Approve] [❌ Reject] [✏️ Edit]
```

---

## Button Actions

### ✅ Approve
- Sends the AI-generated response immediately
- Logs approval in database
- Updates metrics

### ❌ Reject
- Discards the draft response
- Email remains in inbox for manual handling
- Logs rejection with timestamp

### ✏️ Edit
- Opens the review in the web dashboard
- Allows editing the response before sending
- Full email context available

---

## Daily Digest

If enabled, Amail sends a daily summary at your configured start time:

```
📊 Amail Daily Digest - Dec 20, 2024

📬 Emails Received: 47
🤖 AI Responses: 38
✅ Auto-Approved: 32
👤 Human Reviews: 6
❌ Rejected: 2

📈 AI Confidence: 78% average
⏱️ Avg Response Time: 2.3 seconds

Top Agents:
1. Receptionist - 15 tasks
2. Customer Service - 12 tasks
3. Sales - 8 tasks

Pending Reviews: 3
[View Dashboard]
```

Enable with:
```env
AMAIL_TELEGRAM_DAILY_DIGEST=true
AMAIL_DAILY_START_TIME=08:00
```

---

## Group Chat Support

You can also use a Telegram group for team reviews:

1. Create a Telegram group
2. Add your Amail bot to the group
3. Make the bot an admin (so it can send messages)
4. Get the group chat ID:
   - Add @RawDataBot to the group temporarily
   - It will show the group ID (negative number like `-123456789`)
   - Remove @RawDataBot after getting the ID
5. Use the group ID as `TELEGRAM_CHAT_ID`

---

## Advanced Configuration

### Multiple Reviewers

For different mailboxes to notify different people:

```env
# Default reviewer
TELEGRAM_CHAT_ID=123456789

# Per-mailbox overrides (future feature)
# TELEGRAM_CHAT_ID_SUPPORT=987654321
# TELEGRAM_CHAT_ID_LEGAL=456789123
```

### Urgency-Based Routing

```env
# High urgency emails always go to Telegram
TELEGRAM_URGENT_ALWAYS=true

# Keywords that trigger immediate review
TELEGRAM_URGENT_KEYWORDS=urgent,asap,emergency,lawsuit
```

### Quiet Hours

```env
# Don't send notifications during these hours
TELEGRAM_QUIET_START=22:00
TELEGRAM_QUIET_END=07:00

# Queue notifications and send summary after quiet hours
TELEGRAM_QUEUE_DURING_QUIET=true
```

---

## Troubleshooting

### Bot Not Responding

1. Verify bot token is correct
2. Ensure you've started a chat with the bot
3. Check Amail logs for Telegram errors

### Not Receiving Messages

1. Verify chat ID is correct
2. Check TELEGRAM_ENABLED=true
3. Ensure confidence is below threshold
4. Check Telegram API rate limits

### Wrong Chat ID

To find your chat ID:
```bash
curl "https://api.telegram.org/bot<YOUR_BOT_TOKEN>/getUpdates"
```

Look for `"chat":{"id":123456789}` in the response.

### Rate Limiting

Telegram limits bots to ~30 messages/second. If you're processing many emails:
- Batch notifications
- Increase review threshold
- Use group chat for team distribution

---

## Security Considerations

- **Bot token** is sensitive - don't commit to git
- **Chat ID** limits who receives messages
- Messages are encrypted by Telegram
- Review actions are logged in Amail database
- Consider using a dedicated bot (not shared with other services)
