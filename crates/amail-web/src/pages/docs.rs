//! Documentation Page - Web-based documentation viewer with floating TOC
//!
//! Table of Contents:
//! 1. DocsPage - Main documentation viewer
//! 2. DocsSidebar - Floating table of contents
//! 3. DocsContent - Markdown content renderer
//! 4. DocSection - Individual documentation section

use leptos::*;

/// Documentation structure
#[derive(Clone, Debug, PartialEq)]
pub struct DocItem {
    pub id: &'static str,
    pub title: &'static str,
    pub icon: &'static str,
    pub sections: Vec<DocSection>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DocSection {
    pub id: &'static str,
    pub title: &'static str,
    pub content: &'static str,
}

/// Get all documentation items
fn get_docs() -> Vec<DocItem> {
    vec![
        DocItem {
            id: "getting-started",
            title: "Getting Started",
            icon: "M13 10V3L4 14h7v7l9-11h-7z",
            sections: vec![
                DocSection {
                    id: "introduction",
                    title: "Introduction",
                    content: r#"
<h2>Welcome to Amail</h2>
<p><strong>Amail</strong> is a self-hosted AI email server that's 100% open source and free forever. It combines a full-featured mail server with AI-powered auto-reply capabilities.</p>

<h3>Key Features</h3>
<ul>
    <li><strong>30+ AI Agents</strong> - Specialized agents for customer service, sales, HR, legal, and more</li>
    <li><strong>Self-Hosted</strong> - Complete privacy, your data stays on your servers</li>
    <li><strong>Unlimited Domains</strong> - No per-domain or per-user fees</li>
    <li><strong>P2P File Transfer</strong> - No attachment size limits via BitTorrent-style transfers</li>
    <li><strong>Human Review</strong> - Telegram integration for approving AI responses</li>
</ul>
"#,
                },
                DocSection {
                    id: "quick-start",
                    title: "Quick Start",
                    content: r#"
<h2>Quick Start Guide</h2>
<p>Get Amail running in 5 minutes:</p>

<h3>1. Clone the Repository</h3>
<pre><code>git clone https://github.com/yourusername/amail.git
cd amail</code></pre>

<h3>2. Configure Environment</h3>
<pre><code>cp .env.example .env
# Edit .env with your settings</code></pre>

<h3>3. Start with Docker</h3>
<pre><code>docker-compose up -d</code></pre>

<h3>4. Access the Dashboard</h3>
<p>Open <code>http://localhost:4000</code> in your browser.</p>
"#,
                },
                DocSection {
                    id: "requirements",
                    title: "Requirements",
                    content: r#"
<h2>System Requirements</h2>

<h3>Minimum</h3>
<ul>
    <li>2 CPU cores</li>
    <li>4GB RAM</li>
    <li>20GB disk space</li>
    <li>Docker & Docker Compose</li>
</ul>

<h3>Recommended (for AI features)</h3>
<ul>
    <li>4+ CPU cores</li>
    <li>16GB RAM</li>
    <li>100GB SSD</li>
    <li>Ollama with Mistral/Llama model</li>
</ul>

<h3>For GPU Acceleration</h3>
<ul>
    <li>NVIDIA GPU with 8GB+ VRAM</li>
    <li>CUDA 11.8+</li>
</ul>
"#,
                },
            ],
        },
        DocItem {
            id: "architecture",
            title: "Architecture",
            icon: "M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM9 17H7v-7h2v7zm4 0h-2V7h2v10zm4 0h-2v-4h2v4z",
            sections: vec![
                DocSection {
                    id: "overview",
                    title: "System Overview",
                    content: r#"
<h2>System Architecture</h2>
<p>Amail is built on a modular architecture with several key components:</p>

<pre><code>┌─────────────────────────────────────────────────────────────┐
│                      EXTERNAL WORLD                          │
│  (Email senders, recipients, Telegram, DNS)                 │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              │               │               │
              ▼               ▼               ▼
      ┌───────────┐   ┌───────────┐   ┌───────────┐
      │   SMTP    │   │  Webhook  │   │ Telegram  │
      │  Server   │   │  Receiver │   │    Bot    │
      └─────┬─────┘   └─────┬─────┘   └─────┬─────┘
              │               │               │
              └───────────────┼───────────────┘
                              │
                              ▼
              ┌───────────────────────────────┐
              │         AGENT SWARM           │
              │   (Actix Actors + LLM)        │
              └───────────────────────────────┘</code></pre>
"#,
                },
                DocSection {
                    id: "crates",
                    title: "Crate Structure",
                    content: r#"
<h2>Crate Organization</h2>

<table>
    <thead>
        <tr><th>Crate</th><th>Purpose</th></tr>
    </thead>
    <tbody>
        <tr><td><code>amail-core</code></td><td>Core types, email, agents, tasks</td></tr>
        <tr><td><code>amail-mail</code></td><td>SMTP server, DKIM, SPF, DMARC</td></tr>
        <tr><td><code>amail-webhook</code></td><td>HTTP API, Stalwart integration</td></tr>
        <tr><td><code>amail-agents</code></td><td>AI agent implementations</td></tr>
        <tr><td><code>amail-llm</code></td><td>LLM client (Ollama)</td></tr>
        <tr><td><code>amail-mcp</code></td><td>Model Context Protocol</td></tr>
        <tr><td><code>amail-p2p</code></td><td>P2P file transfers</td></tr>
        <tr><td><code>amail-telegram</code></td><td>Telegram bot integration</td></tr>
        <tr><td><code>amail-dashboard</code></td><td>Web UI (Leptos)</td></tr>
    </tbody>
</table>
"#,
                },
                DocSection {
                    id: "data-flow",
                    title: "Data Flow",
                    content: r#"
<h2>Email Processing Flow</h2>

<ol>
    <li><strong>Ingestion</strong> - Email arrives via SMTP or webhook</li>
    <li><strong>Parsing</strong> - MIME message parsed, attachments extracted</li>
    <li><strong>Triage</strong> - Receptionist agent classifies intent</li>
    <li><strong>Routing</strong> - Task routed to specialist agent</li>
    <li><strong>Processing</strong> - Agent generates response via LLM</li>
    <li><strong>Review</strong> - Draft sent to Telegram for approval (if configured)</li>
    <li><strong>Delivery</strong> - Approved response sent via SMTP</li>
</ol>
"#,
                },
            ],
        },
        DocItem {
            id: "dns",
            title: "DNS & MX Setup",
            icon: "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z",
            sections: vec![
                DocSection {
                    id: "dns-overview",
                    title: "Overview",
                    content: r#"
<h2>DNS Configuration</h2>

<p>For Amail to receive email, configure these DNS records for your domain:</p>

<table>
    <thead>
        <tr><th>Type</th><th>Name</th><th>Value</th><th>Priority</th></tr>
    </thead>
    <tbody>
        <tr><td><strong>MX</strong></td><td>@</td><td>mail.yourdomain.com</td><td>10</td></tr>
        <tr><td><strong>A</strong></td><td>mail</td><td>YOUR_SERVER_IP</td><td>-</td></tr>
        <tr><td><strong>TXT</strong></td><td>@</td><td>v=spf1 mx a ip4:YOUR_IP -all</td><td>-</td></tr>
        <tr><td><strong>TXT</strong></td><td>_dmarc</td><td>v=DMARC1; p=quarantine; rua=mailto:dmarc@yourdomain.com</td><td>-</td></tr>
        <tr><td><strong>TXT</strong></td><td>default._domainkey</td><td>v=DKIM1; k=rsa; p=YOUR_DKIM_KEY</td><td>-</td></tr>
    </tbody>
</table>
"#,
                },
                DocSection {
                    id: "mx-records",
                    title: "MX Records",
                    content: r#"
<h2>MX Record Setup</h2>

<p>The MX record tells other mail servers where to deliver email for your domain.</p>

<h3>Step 1: Create A Record</h3>
<pre><code>Type: A
Name: mail
Value: YOUR_SERVER_IP (e.g., 203.0.113.50)
TTL: 3600</code></pre>

<h3>Step 2: Create MX Record</h3>
<pre><code>Type: MX
Name: @ (or yourdomain.com)
Value: mail.yourdomain.com
Priority: 10
TTL: 3600</code></pre>

<p><strong>Note:</strong> MX records point to hostnames, not IP addresses.</p>
"#,
                },
                DocSection {
                    id: "email-auth",
                    title: "Email Authentication",
                    content: r#"
<h2>SPF, DKIM & DMARC</h2>

<h3>SPF (Sender Policy Framework)</h3>
<p>Prevents email spoofing by specifying which servers can send email for your domain.</p>
<pre><code>Type: TXT
Name: @
Value: v=spf1 mx a ip4:YOUR_SERVER_IP -all</code></pre>

<h3>DKIM (DomainKeys Identified Mail)</h3>
<p>Signs outgoing emails cryptographically. Amail generates keys automatically.</p>
<pre><code>Type: TXT
Name: default._domainkey
Value: v=DKIM1; k=rsa; p=YOUR_PUBLIC_KEY</code></pre>

<h3>DMARC (Domain-based Message Authentication)</h3>
<p>Tells receiving servers what to do with failed SPF/DKIM checks.</p>
<pre><code>Type: TXT
Name: _dmarc
Value: v=DMARC1; p=quarantine; rua=mailto:dmarc@yourdomain.com</code></pre>

<p><strong>Tip:</strong> Start with <code>p=none</code> to monitor, then move to <code>p=quarantine</code> or <code>p=reject</code>.</p>
"#,
                },
                DocSection {
                    id: "google-migration",
                    title: "Google Workspace Migration",
                    content: r#"
<h2>Migrating from Google Workspace</h2>

<h3>Before Migration</h3>
<ol>
    <li>Export data from Google Takeout</li>
    <li>Lower TTL on existing MX records to 300 (24 hours before)</li>
    <li>Document current records for rollback</li>
</ol>

<h3>Current Google MX Records</h3>
<pre><code>Priority 1:  ASPMX.L.GOOGLE.COM
Priority 5:  ALT1.ASPMX.L.GOOGLE.COM
Priority 5:  ALT2.ASPMX.L.GOOGLE.COM
Priority 10: ALT3.ASPMX.L.GOOGLE.COM
Priority 10: ALT4.ASPMX.L.GOOGLE.COM</code></pre>

<h3>Migration Steps</h3>
<ol>
    <li>Set up and verify Amail server is running</li>
    <li>Test by sending email directly to your server</li>
    <li>Remove Google MX records</li>
    <li>Add Amail MX record: <code>mail.yourdomain.com</code> priority 10</li>
    <li>Wait for propagation (up to 48 hours)</li>
    <li>Monitor incoming email in Amail dashboard</li>
</ol>

<h3>Rollback</h3>
<p>If issues occur, restore Google MX records immediately.</p>
"#,
                },
                DocSection {
                    id: "firewall",
                    title: "Firewall & Ports",
                    content: r#"
<h2>Required Ports</h2>

<table>
    <thead>
        <tr><th>Port</th><th>Protocol</th><th>Purpose</th></tr>
    </thead>
    <tbody>
        <tr><td><strong>25</strong></td><td>TCP</td><td>SMTP (receiving mail)</td></tr>
        <tr><td><strong>587</strong></td><td>TCP</td><td>SMTP Submission (sending)</td></tr>
        <tr><td><strong>465</strong></td><td>TCP</td><td>SMTPS (secure SMTP)</td></tr>
        <tr><td><strong>993</strong></td><td>TCP</td><td>IMAPS (secure IMAP)</td></tr>
        <tr><td><strong>443</strong></td><td>TCP</td><td>HTTPS (web dashboard)</td></tr>
        <tr><td><strong>80</strong></td><td>TCP</td><td>HTTP (Let's Encrypt)</td></tr>
    </tbody>
</table>

<h3>UFW Example</h3>
<pre><code>sudo ufw allow 25/tcp
sudo ufw allow 587/tcp
sudo ufw allow 465/tcp
sudo ufw allow 993/tcp
sudo ufw allow 443/tcp</code></pre>

<h3>Verification</h3>
<pre><code># Check MX records
dig MX yourdomain.com +short

# Check SPF
dig TXT yourdomain.com +short | grep spf

# Test with online tools
# - mxtoolbox.com
# - mail-tester.com</code></pre>
"#,
                },
            ],
        },
        DocItem {
            id: "agents",
            title: "AI Agents",
            icon: "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 3c1.66 0 3 1.34 3 3s-1.34 3-3 3-3-1.34-3-3 1.34-3 3-3zm0 14.2c-2.5 0-4.71-1.28-6-3.22.03-1.99 4-3.08 6-3.08 1.99 0 5.97 1.09 6 3.08-1.29 1.94-3.5 3.22-6 3.22z",
            sections: vec![
                DocSection {
                    id: "core-agents",
                    title: "Core Agents",
                    content: r#"
<h2>Core Agents (Always Active)</h2>

<h3>1. Receptionist</h3>
<p><strong>Role:</strong> Email triage and routing</p>
<ul>
    <li>First agent to see every email</li>
    <li>Classifies intent and urgency</li>
    <li>Routes to appropriate specialist</li>
</ul>

<h3>2. Clerk</h3>
<p><strong>Role:</strong> Administrative tasks</p>
<ul>
    <li>Calendar management</li>
    <li>Meeting scheduling</li>
    <li>Document organization</li>
</ul>

<h3>3. Executive</h3>
<p><strong>Role:</strong> High-level decisions</p>
<ul>
    <li>Strategic decisions</li>
    <li>VIP handling</li>
    <li>Escalation management</li>
</ul>

<h3>4. Draft Checker</h3>
<p><strong>Role:</strong> Quality assurance</p>
<ul>
    <li>Reviews all AI responses</li>
    <li>Checks tone and accuracy</li>
    <li>Suggests improvements</li>
</ul>
"#,
                },
                DocSection {
                    id: "specialist-agents",
                    title: "Specialist Agents",
                    content: r#"
<h2>Specialist Agents</h2>

<div class="agent-grid">
    <div><strong>Accountant</strong> - Financial inquiries, invoices</div>
    <div><strong>Lawyer</strong> - Legal questions, contracts</div>
    <div><strong>HR Manager</strong> - Employment, benefits</div>
    <div><strong>IT Support</strong> - Technical issues</div>
    <div><strong>Sales Rep</strong> - Product inquiries, quotes</div>
    <div><strong>Marketing</strong> - Campaigns, partnerships</div>
    <div><strong>Customer Service</strong> - General support</div>
    <div><strong>Project Manager</strong> - Status updates, timelines</div>
</div>

<p>Each agent has specialized prompts and can be customized via configuration.</p>
"#,
                },
                DocSection {
                    id: "agent-config",
                    title: "Configuration",
                    content: r#"
<h2>Agent Configuration</h2>

<p>Agents are configured in <code>.env</code>:</p>

<pre><code># Enable/disable specific agents
AGENT_RECEPTIONIST_ENABLED=true
AGENT_ACCOUNTANT_ENABLED=true
AGENT_LAWYER_ENABLED=false

# Agent behavior
AGENT_CONFIDENCE_THRESHOLD=0.7
AGENT_AUTO_REPLY_ENABLED=true
AGENT_REQUIRE_REVIEW=true</code></pre>

<h3>Custom System Prompts</h3>
<p>Override default prompts in <code>config/agents/</code>:</p>
<pre><code>config/agents/
├── receptionist.txt
├── accountant.txt
└── custom_agent.txt</code></pre>
"#,
                },
            ],
        },
        DocItem {
            id: "p2p",
            title: "P2P Transfers",
            icon: "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z",
            sections: vec![
                DocSection {
                    id: "p2p-overview",
                    title: "Overview",
                    content: r#"
<h2>P2P File Transfer</h2>

<p>Amail includes BitTorrent-style peer-to-peer file transfers, eliminating attachment size limits.</p>

<table>
    <thead>
        <tr><th>Traditional Email</th><th>Amail P2P</th></tr>
    </thead>
    <tbody>
        <tr><td>25MB attachment limit</td><td><strong>No limit</strong></td></tr>
        <tr><td>Files stored on mail server</td><td><strong>Direct peer transfer</strong></td></tr>
        <tr><td>Single point of failure</td><td><strong>Distributed</strong></td></tr>
        <tr><td>Slow for large files</td><td><strong>Parallel chunk downloads</strong></td></tr>
    </tbody>
</table>
"#,
                },
                DocSection {
                    id: "p2p-how-it-works",
                    title: "How It Works",
                    content: r#"
<h2>Transfer Process</h2>

<ol>
    <li><strong>Chunking</strong> - File split into 1MB chunks</li>
    <li><strong>Hashing</strong> - Each chunk gets BLAKE3 hash</li>
    <li><strong>Announcement</strong> - Transfer announced to DHT</li>
    <li><strong>Discovery</strong> - Peers find each other via mDNS/DHT</li>
    <li><strong>Transfer</strong> - Chunks downloaded in parallel</li>
    <li><strong>Verification</strong> - Hashes verified on receipt</li>
    <li><strong>Assembly</strong> - Chunks reassembled into file</li>
</ol>

<h3>Magnet Links</h3>
<pre><code>amail://transfer-uuid?xt=urn:blake3:filehash&dn=filename.zip&xl=filesize</code></pre>
"#,
                },
                DocSection {
                    id: "p2p-config",
                    title: "Configuration",
                    content: r#"
<h2>P2P Configuration</h2>

<pre><code># Enable P2P for large attachments
P2P_ATTACHMENTS_ENABLED=true

# Size threshold (attachments larger than this use P2P)
P2P_SIZE_THRESHOLD_BYTES=10485760  # 10MB

# Maximum attachment size
P2P_MAX_ATTACHMENT_SIZE_BYTES=10737418240  # 10GB

# Retention period
P2P_RETENTION_DAYS=30

# Storage path
P2P_STORAGE_PATH=./data/p2p_attachments</code></pre>
"#,
                },
            ],
        },
        DocItem {
            id: "api",
            title: "API Reference",
            icon: "M9.4 16.6L4.8 12l4.6-4.6L8 6l-6 6 6 6 1.4-1.4zm5.2 0l4.6-4.6-4.6-4.6L16 6l6 6-6 6-1.4-1.4z",
            sections: vec![
                DocSection {
                    id: "api-overview",
                    title: "Overview",
                    content: r#"
<h2>REST API</h2>

<p>Amail exposes a REST API on port 4000 (configurable).</p>

<h3>Base URL</h3>
<pre><code>http://localhost:4000/api/v1</code></pre>

<h3>Authentication</h3>
<p>API requests require an API key in the header:</p>
<pre><code>Authorization: Bearer YOUR_API_KEY</code></pre>
"#,
                },
                DocSection {
                    id: "api-domains",
                    title: "Domains API",
                    content: r#"
<h2>Domains</h2>

<h3>List Domains</h3>
<pre><code>GET /api/v1/domains</code></pre>

<h3>Create Domain</h3>
<pre><code>POST /api/v1/domains
{
    "name": "example.com"
}</code></pre>

<h3>Verify DNS</h3>
<pre><code>POST /api/v1/domains/{id}/verify</code></pre>
"#,
                },
                DocSection {
                    id: "api-emails",
                    title: "Emails API",
                    content: r#"
<h2>Emails</h2>

<h3>List Emails</h3>
<pre><code>GET /api/v1/emails?folder=inbox&limit=50</code></pre>

<h3>Get Email</h3>
<pre><code>GET /api/v1/emails/{id}</code></pre>

<h3>Send Email</h3>
<pre><code>POST /api/v1/emails/send
{
    "to": "recipient@example.com",
    "subject": "Hello",
    "body": "Email content"
}</code></pre>
"#,
                },
            ],
        },
        DocItem {
            id: "compliance",
            title: "Compliance",
            icon: "M12 1L3 5v6c0 5.55 3.84 10.74 9 12 5.16-1.26 9-6.45 9-12V5l-9-4zm0 10.99h7c-.53 4.12-3.28 7.79-7 8.94V12H5V6.3l7-3.11v8.8z",
            sections: vec![
                DocSection {
                    id: "compliance-overview",
                    title: "Overview",
                    content: r#"
<h2>Legal Compliance</h2>

<p>Amail includes built-in compliance modules for:</p>

<ul>
    <li><strong>CSAM Reporting</strong> - Mandatory reporting to NCMEC</li>
    <li><strong>DMCA</strong> - Takedown request handling</li>
    <li><strong>CAN-SPAM</strong> - Unsubscribe and opt-out compliance</li>
    <li><strong>COPPA</strong> - Children's privacy protection</li>
    <li><strong>GDPR</strong> - EU data protection</li>
    <li><strong>CCPA</strong> - California privacy rights</li>
</ul>

<p class="warning"><strong>Important:</strong> This documentation is for informational purposes only. Consult with a qualified attorney for your specific situation.</p>
"#,
                },
                DocSection {
                    id: "gdpr",
                    title: "GDPR",
                    content: r#"
<h2>GDPR Compliance</h2>

<h3>Data Subject Rights</h3>
<ul>
    <li><strong>Right to Access</strong> - Export user data</li>
    <li><strong>Right to Erasure</strong> - Delete user data</li>
    <li><strong>Right to Portability</strong> - Download in standard format</li>
    <li><strong>Right to Rectification</strong> - Correct inaccurate data</li>
</ul>

<h3>Configuration</h3>
<pre><code>GDPR_DPO_EMAIL=dpo@yourdomain.com
GDPR_AUTO_DSR_ENABLED=true
RETENTION_DEFAULT_DAYS=365</code></pre>
"#,
                },
                DocSection {
                    id: "spam-filtering",
                    title: "Spam Filtering",
                    content: r#"
<h2>Spam Filtering</h2>

<p>Amail includes multi-layer spam detection:</p>

<ul>
    <li><strong>SPF/DKIM/DMARC</strong> - Authentication checks</li>
    <li><strong>Content Analysis</strong> - Keyword and pattern matching</li>
    <li><strong>Reputation</strong> - Sender reputation scoring</li>
    <li><strong>Rate Limiting</strong> - Prevent flooding</li>
</ul>

<h3>Configuration</h3>
<pre><code>SPAM_THRESHOLD=5.0
SPAM_AUTO_DELETE_THRESHOLD=10.0
SPAM_BLOCKED_DOMAINS=spam.com,phishing.net</code></pre>
"#,
                },
            ],
        },
        DocItem {
            id: "telegram",
            title: "Telegram Bot",
            icon: "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.64 6.8c-.15 1.58-.8 5.42-1.13 7.19-.14.75-.42 1-.68 1.03-.58.05-1.02-.38-1.58-.75-.88-.58-1.38-.94-2.23-1.5-.99-.65-.35-1.01.22-1.59.15-.15 2.71-2.48 2.76-2.69a.2.2 0 00-.05-.18c-.06-.05-.14-.03-.21-.02-.09.02-1.49.95-4.22 2.79-.4.27-.76.41-1.08.4-.36-.01-1.04-.2-1.55-.37-.63-.2-1.12-.31-1.08-.66.02-.18.27-.36.74-.55 2.92-1.27 4.86-2.11 5.83-2.51 2.78-1.16 3.35-1.36 3.73-1.36.08 0 .27.02.39.12.1.08.13.19.14.27-.01.06.01.24 0 .38z",
            sections: vec![
                DocSection {
                    id: "telegram-setup",
                    title: "Setup",
                    content: r#"
<h2>Telegram Bot Setup</h2>

<h3>1. Create Bot</h3>
<ol>
    <li>Message <code>@BotFather</code> on Telegram</li>
    <li>Send <code>/newbot</code></li>
    <li>Follow prompts to name your bot</li>
    <li>Copy the API token</li>
</ol>

<h3>2. Configure Amail</h3>
<pre><code>TELEGRAM_BOT_TOKEN=your_bot_token_here
TELEGRAM_CHAT_ID=your_chat_id
TELEGRAM_ENABLED=true</code></pre>

<h3>3. Get Chat ID</h3>
<p>Send a message to your bot, then visit:</p>
<pre><code>https://api.telegram.org/bot{TOKEN}/getUpdates</code></pre>
"#,
                },
                DocSection {
                    id: "telegram-commands",
                    title: "Commands",
                    content: r#"
<h2>Bot Commands</h2>

<table>
    <thead>
        <tr><th>Command</th><th>Description</th></tr>
    </thead>
    <tbody>
        <tr><td><code>/start</code></td><td>Initialize bot</td></tr>
        <tr><td><code>/status</code></td><td>System status</td></tr>
        <tr><td><code>/pending</code></td><td>List pending reviews</td></tr>
        <tr><td><code>/approve {id}</code></td><td>Approve draft</td></tr>
        <tr><td><code>/reject {id}</code></td><td>Reject draft</td></tr>
        <tr><td><code>/edit {id}</code></td><td>Edit before sending</td></tr>
    </tbody>
</table>
"#,
                },
                DocSection {
                    id: "telegram-notifications",
                    title: "Notifications",
                    content: r#"
<h2>Notification Settings</h2>

<pre><code># What to notify about
TELEGRAM_NOTIFY_NEW_EMAIL=true
TELEGRAM_NOTIFY_AI_DRAFT=true
TELEGRAM_NOTIFY_ERRORS=true

# Quiet hours (no notifications)
TELEGRAM_QUIET_START=22:00
TELEGRAM_QUIET_END=08:00

# Priority filtering
TELEGRAM_MIN_PRIORITY=normal</code></pre>
"#,
                },
            ],
        },
    ]
}

#[component]
pub fn DocsPage() -> impl IntoView {
    let docs = get_docs();
    let docs_for_nav = docs.clone();
    let docs_for_content = docs.clone();
    
    let (active_doc, set_active_doc) = create_signal(docs[0].id);
    let (active_section, set_active_section) = create_signal(docs[0].sections[0].id);
    
    let current_section = move || {
        docs_for_content.iter()
            .find(|d| d.id == active_doc.get())
            .and_then(|doc| {
                doc.sections.iter().find(|s| s.id == active_section.get()).cloned()
            })
    };

    view! {
        <div class="docs-page">
            // Left sidebar - Table of Contents
            <aside class="docs-sidebar">
                <div class="docs-sidebar-header">
                    <h2>"Documentation"</h2>
                    <span class="docs-version">"v0.1.0"</span>
                </div>
                
                <nav class="docs-nav">
                    {docs_for_nav.iter().map(|doc| {
                        let doc_id = doc.id;
                        let doc_title = doc.title;
                        let doc_icon = doc.icon;
                        let sections_for_click = doc.sections.clone();
                        let sections_for_show = doc.sections.clone();
                        let is_active = move || active_doc.get() == doc_id;
                        
                        view! {
                            <div class="docs-nav-group">
                                <button 
                                    class=move || if is_active() { "docs-nav-item active" } else { "docs-nav-item" }
                                    on:click=move |_| {
                                        set_active_doc.set(doc_id);
                                        if let Some(first) = sections_for_click.first() {
                                            set_active_section.set(first.id);
                                        }
                                    }
                                >
                                    <svg viewBox="0 0 24 24" fill="currentColor" class="docs-nav-icon">
                                        <path d=doc_icon/>
                                    </svg>
                                    <span>{doc_title}</span>
                                </button>
                                
                                <Show when=is_active>
                                    <div class="docs-nav-sections">
                                        {sections_for_show.iter().map(|section| {
                                            let section_id = section.id;
                                            let section_title = section.title;
                                            let is_section_active = move || active_section.get() == section_id;
                                            
                                            view! {
                                                <button 
                                                    class=move || if is_section_active() { "docs-section-link active" } else { "docs-section-link" }
                                                    on:click=move |_| set_active_section.set(section_id)
                                                >
                                                    {section_title}
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                </Show>
                            </div>
                        }
                    }).collect_view()}
                </nav>
                
                <div class="docs-sidebar-footer">
                    <a href="https://github.com/yourusername/amail" target="_blank" class="docs-github-link">
                        <svg viewBox="0 0 24 24" fill="currentColor">
                            <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                        </svg>
                        "View on GitHub"
                    </a>
                </div>
            </aside>
            
            // Main content area
            <main class="docs-content">
                {move || {
                    if let Some(section) = current_section() {
                        view! {
                            <article class="docs-article" inner_html=section.content></article>
                        }.into_view()
                    } else {
                        view! {
                            <div class="docs-empty">
                                <p>"Select a topic from the sidebar"</p>
                            </div>
                        }.into_view()
                    }
                }}
            </main>
        </div>
    }
}
