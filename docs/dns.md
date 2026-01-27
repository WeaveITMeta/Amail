# DNS & MX Record Setup

This guide covers setting up DNS records for Amail, including migrating from Google Workspace or other email providers.

---

## Quick Start

For Amail to receive email, you need to configure these DNS records:

| Type | Name | Value | Priority | TTL |
|------|------|-------|----------|-----|
| **MX** | @ | `mail.yourdomain.com` | 10 | 3600 |
| **A** | mail | `YOUR_SERVER_IP` | - | 3600 |
| **TXT** | @ | `v=spf1 mx a ip4:YOUR_SERVER_IP -all` | - | 3600 |
| **TXT** | _dmarc | `v=DMARC1; p=quarantine; rua=mailto:dmarc@yourdomain.com` | - | 3600 |
| **TXT** | default._domainkey | `v=DKIM1; k=rsa; p=YOUR_DKIM_PUBLIC_KEY` | - | 3600 |

---

## Step-by-Step Setup

### 1. Get Your Server IP

Your Amail server needs a static public IP address. If using a cloud provider:

```bash
# Check your public IP
curl ifconfig.me
```

### 2. Configure MX Record

The MX record tells other mail servers where to deliver email for your domain.

```
Type: MX
Name: @ (or yourdomain.com)
Value: mail.yourdomain.com
Priority: 10
TTL: 3600
```

**Important**: The MX record points to a hostname, not an IP address.

### 3. Configure A Record for Mail Server

Create an A record for the mail subdomain:

```
Type: A
Name: mail
Value: YOUR_SERVER_IP (e.g., 203.0.113.50)
TTL: 3600
```

### 4. Configure SPF Record

SPF (Sender Policy Framework) prevents email spoofing:

```
Type: TXT
Name: @ (or yourdomain.com)
Value: v=spf1 mx a ip4:YOUR_SERVER_IP -all
TTL: 3600
```

**Explanation**:
- `mx` - Allow servers listed in MX records
- `a` - Allow the domain's A record IP
- `ip4:YOUR_SERVER_IP` - Explicitly allow your server
- `-all` - Reject all other sources (strict)

### 5. Configure DKIM

DKIM (DomainKeys Identified Mail) signs outgoing emails.

First, generate DKIM keys in Amail:

```bash
# Amail generates DKIM keys on first run
# Find your public key in the dashboard or:
cat /var/lib/amail/dkim/default.pub
```

Then add the DNS record:

```
Type: TXT
Name: default._domainkey
Value: v=DKIM1; k=rsa; p=YOUR_PUBLIC_KEY_HERE
TTL: 3600
```

### 6. Configure DMARC

DMARC tells receiving servers what to do with failed SPF/DKIM:

```
Type: TXT
Name: _dmarc
Value: v=DMARC1; p=quarantine; rua=mailto:dmarc@yourdomain.com; pct=100
TTL: 3600
```

**Policy options**:
- `p=none` - Monitor only (start here)
- `p=quarantine` - Send to spam folder
- `p=reject` - Reject failed emails

---

## Migrating from Google Workspace

### Before Migration

1. **Export your data** from Google Takeout
2. **Lower TTL** on existing MX records to 300 (5 minutes) 24 hours before
3. **Document** current MX records for rollback

### Current Google Workspace MX Records

```
Priority 1:  ASPMX.L.GOOGLE.COM
Priority 5:  ALT1.ASPMX.L.GOOGLE.COM
Priority 5:  ALT2.ASPMX.L.GOOGLE.COM
Priority 10: ALT3.ASPMX.L.GOOGLE.COM
Priority 10: ALT4.ASPMX.L.GOOGLE.COM
```

### Migration Steps

1. **Set up Amail server** and verify it's running
2. **Test locally** by sending email to your server directly
3. **Update MX records** to point to Amail:

```
# Remove Google MX records and add:
Type: MX
Name: @
Value: mail.yourdomain.com
Priority: 10
```

4. **Wait for propagation** (up to 48 hours, usually faster)
5. **Monitor** incoming email in Amail dashboard
6. **Disable Google Workspace** email routing after confirming

### Rollback Plan

If issues occur, restore Google MX records:

```
Type: MX, Name: @, Value: ASPMX.L.GOOGLE.COM, Priority: 1
Type: MX, Name: @, Value: ALT1.ASPMX.L.GOOGLE.COM, Priority: 5
Type: MX, Name: @, Value: ALT2.ASPMX.L.GOOGLE.COM, Priority: 5
```

---

## Firewall Configuration

Ensure these ports are open on your server:

| Port | Protocol | Purpose |
|------|----------|---------|
| 25 | TCP | SMTP (receiving mail) |
| 587 | TCP | SMTP Submission (sending mail) |
| 465 | TCP | SMTPS (secure SMTP) |
| 993 | TCP | IMAPS (secure IMAP) |
| 443 | TCP | HTTPS (web dashboard) |
| 80 | TCP | HTTP (Let's Encrypt) |

```bash
# UFW example
sudo ufw allow 25/tcp
sudo ufw allow 587/tcp
sudo ufw allow 465/tcp
sudo ufw allow 993/tcp
sudo ufw allow 443/tcp
sudo ufw allow 80/tcp
```

---

## Verification

### Test MX Records

```bash
# Check MX records
dig MX yourdomain.com +short

# Expected output:
# 10 mail.yourdomain.com.
```

### Test SPF

```bash
dig TXT yourdomain.com +short | grep spf
```

### Test DKIM

```bash
dig TXT default._domainkey.yourdomain.com +short
```

### Test Email Delivery

Use online tools:
- [MXToolbox](https://mxtoolbox.com/)
- [Mail-Tester](https://www.mail-tester.com/)
- [DMARC Analyzer](https://www.dmarcanalyzer.com/)

### Send Test Email

```bash
# From another email provider, send to your domain
# Check Amail dashboard for incoming mail
```

---

## Troubleshooting

### Email Not Arriving

1. **Check MX propagation**: `dig MX yourdomain.com`
2. **Check firewall**: Ensure port 25 is open
3. **Check Amail logs**: `docker logs amail-server`
4. **Check spam folder**: Email might be filtered

### SPF Failures

- Ensure your server IP is in the SPF record
- Check for multiple SPF records (only one allowed)
- Use `~all` (softfail) instead of `-all` during testing

### DKIM Failures

- Verify the public key is correctly formatted
- Check for line breaks in the DNS record
- Ensure the selector matches (`default._domainkey`)

### DMARC Failures

- Start with `p=none` to monitor
- Check DMARC reports at the `rua` address
- Gradually increase to `p=quarantine` then `p=reject`

---

## Reverse DNS (PTR Record)

For best deliverability, set up reverse DNS with your hosting provider:

```
IP: YOUR_SERVER_IP
PTR: mail.yourdomain.com
```

This must be done through your hosting provider's control panel, not your domain registrar.

---

## Multiple Domains

Amail supports unlimited domains. For each additional domain:

1. Add MX record pointing to your Amail server
2. Add SPF, DKIM, and DMARC records
3. Configure the domain in Amail dashboard

```
# Domain 2
Type: MX, Name: @, Value: mail.domain2.com, Priority: 10
Type: A, Name: mail, Value: YOUR_SERVER_IP

# Or use the same mail server
Type: MX, Name: @, Value: mail.primarydomain.com, Priority: 10
```
