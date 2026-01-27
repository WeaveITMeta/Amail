# AI Agents

Amail uses a swarm of specialized AI agents to process emails. Each agent has a specific role and expertise, allowing for intelligent routing and response generation.

---

## Agent Architecture

### Actix Actor Model

Each agent is an independent **Actix actor** with:
- Own mailbox for receiving tasks
- Isolated state
- Supervised lifecycle (auto-restart on failure)
- Concurrent processing capability

### Supervision Tree

```
                    Supervisor
                        │
        ┌───────────────┼───────────────┐
        │               │               │
   Core Agents    Extended Agents   Custom Agents
   (Always on)    (On demand)       (User defined)
```

---

## Core Agents (6)

These agents are always active and handle the majority of email processing.

### 1. Receptionist
**Role**: Email triage and routing

- First agent to see every email
- Classifies intent and urgency
- Routes to appropriate specialist
- Handles general inquiries

**Triggers**: All incoming emails

**Example**:
```
Email: "Hi, I have a question about my invoice"
Action: Route to Accountant agent
```

### 2. Clerk
**Role**: Administrative tasks and scheduling

- Calendar management
- Meeting scheduling
- Directory lookups
- Document organization

**Triggers**: Scheduling requests, admin tasks

### 3. Executive
**Role**: High-level decisions and escalations

- Strategic decisions
- VIP handling
- Escalation management
- Policy questions

**Triggers**: Executive-level inquiries, escalations

### 4. Draft Checker
**Role**: Quality assurance for outgoing emails

- Reviews all AI-generated responses
- Checks tone and professionalism
- Verifies accuracy
- Suggests improvements

**Triggers**: All draft responses before sending

### 5. Accountant
**Role**: Financial matters

- Invoice processing
- Payment inquiries
- Budget questions
- Financial reports

**Triggers**: Emails mentioning money, invoices, payments

### 6. Lawyer
**Role**: Legal matters

- Contract questions
- Compliance inquiries
- Legal document review
- Liability concerns

**Triggers**: Legal terminology, contracts, compliance

---

## Extended Agents (25+)

Enable with `FULL_SWARM=true` in `.env`

### Human Resources
| Agent | Role |
|-------|------|
| **HR Manager** | Employee relations, policies, benefits |
| **Recruiter** | Job applications, hiring inquiries |
| **Trainer** | Training requests, onboarding |

### Technical
| Agent | Role |
|-------|------|
| **IT Support** | Technical issues, password resets |
| **Security Analyst** | Security concerns, breach reports |
| **Data Analyst** | Data requests, analytics questions |

### Sales & Marketing
| Agent | Role |
|-------|------|
| **Sales Rep** | Product inquiries, pricing, quotes |
| **Marketing Coordinator** | Campaign questions, partnerships |
| **PR Officer** | Media inquiries, press releases |

### Operations
| Agent | Role |
|-------|------|
| **Project Manager** | Project status, timelines |
| **Logistics Coordinator** | Shipping, delivery inquiries |
| **Facilities Manager** | Office issues, maintenance |
| **Procurement Specialist** | Vendor inquiries, purchasing |

### Specialized
| Agent | Role |
|-------|------|
| **Customer Service** | General customer inquiries |
| **Compliance Officer** | Regulatory questions |
| **Research Analyst** | Research requests |
| **Content Creator** | Content requests, copywriting |
| **Translator** | Translation requests |
| **Quality Assurance** | QA inquiries, bug reports |
| **Risk Manager** | Risk assessments |
| **Event Coordinator** | Event planning |
| **Innovation Researcher** | New ideas, R&D |
| **Ethics Reviewer** | Ethical concerns |
| **Internal Auditor** | Audit requests |
| **Negotiator** | Deal negotiations |
| **Strategic Planner** | Strategy questions |
| **Archivist** | Historical data requests |

---

## Agent Routing

### Automatic Routing

The Receptionist agent analyzes each email and routes based on:

1. **Keywords** - Specific terms trigger specific agents
2. **Sender** - Known contacts route to their usual handler
3. **Mailbox** - `legal@` goes to Lawyer, `hr@` goes to HR
4. **Intent** - NLP classification of email purpose
5. **Urgency** - High urgency may escalate to Executive

### Routing Rules

```rust
// Example routing logic
match classify_email(&email) {
    Intent::Invoice => route_to("Accountant"),
    Intent::Legal => route_to("Lawyer"),
    Intent::Technical => route_to("IT Support"),
    Intent::Hiring => route_to("Recruiter"),
    Intent::General => route_to("Customer Service"),
    Intent::Urgent => route_to("Executive"),
    _ => route_to("Receptionist"),
}
```

### Manual Override

In the dashboard, you can:
- Reassign emails to different agents
- Set routing rules per mailbox
- Create custom routing based on sender domain

---

## Agent Configuration

### Per-Mailbox Agent Assignment

```env
# Format: mailbox:agent_role
MAIL_BOXES=inbox,support:customer_service,hr:human_resources,legal:lawyer
```

### Agent Behavior Settings

In the dashboard Settings page:
- **Response tone**: Professional, friendly, formal
- **Response length**: Concise, detailed, comprehensive
- **Confidence threshold**: When to request human review
- **Auto-reply**: Enable/disable per agent

---

## Agent Prompts

Each agent has a specialized system prompt. Located in `crates/amail-agents/src/prompts.rs`:

```rust
pub fn receptionist_prompt() -> String {
    r#"You are a professional receptionist for {org_name}.
    
    Your responsibilities:
    - Greet visitors and callers professionally
    - Route inquiries to the appropriate department
    - Answer general questions about the organization
    - Take messages when staff are unavailable
    
    Always be:
    - Polite and welcoming
    - Clear and concise
    - Helpful and patient
    
    If you're unsure how to help, offer to connect them
    with someone who can assist."#.to_string()
}
```

### Customizing Prompts

1. Edit `prompts.rs` for permanent changes
2. Or use the dashboard to override per-agent
3. Variables available: `{org_name}`, `{sender_name}`, `{mailbox}`

---

## Agent Metrics

The dashboard shows per-agent statistics:

- **Tasks processed** - Total emails handled
- **Average confidence** - AI confidence score
- **Response time** - Average processing time
- **Approval rate** - % of responses approved
- **Escalation rate** - % escalated to human

---

## Creating Custom Agents

### 1. Define the Role

Add to `crates/amail-core/src/agent.rs`:

```rust
pub enum AgentRole {
    // ... existing roles
    MyCustomAgent,
}
```

### 2. Create the Agent

Add to `crates/amail-agents/src/agents/`:

```rust
// my_custom_agent.rs
use crate::prelude::*;

pub struct MyCustomAgent {
    config: AgentConfig,
}

impl Agent for MyCustomAgent {
    fn role(&self) -> AgentRole {
        AgentRole::MyCustomAgent
    }
    
    async fn process(&self, task: Task) -> TaskResult {
        // Your logic here
    }
}
```

### 3. Add Prompt

In `prompts.rs`:

```rust
pub fn my_custom_agent_prompt() -> String {
    "You are a specialized agent for...".to_string()
}
```

### 4. Register

In `crates/amail-agents/src/agents/mod.rs`:

```rust
pub mod my_custom_agent;
pub use my_custom_agent::MyCustomAgent;
```

---

## Best Practices

### Agent Design
- Keep agents focused on one domain
- Use clear, specific prompts
- Set appropriate confidence thresholds
- Monitor and adjust based on metrics

### Routing
- Start with core agents, add extended as needed
- Use mailbox-based routing for clear separation
- Review routing logs to optimize

### Performance
- Disable unused extended agents
- Use smaller LLM models for simple agents
- Batch similar tasks when possible
