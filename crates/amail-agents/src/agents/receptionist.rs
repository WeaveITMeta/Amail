// =============================================================================
// Receptionist Agent - Email Triage and Routing
// =============================================================================
// Table of Contents:
// 1. ReceptionistAgent struct
// 2. Email classification logic (keyword-based fallback)
// 3. LLM-based classification (primary)
// 4. Routing rules
// 5. Priority assessment
// 6. Multi-agent orchestration
// =============================================================================

use amail_core::{
    agent::{AgentCapability, AgentRole},
    email::EmailMessage,
    task::{Task, TaskPriority, TaskType},
};
use amail_llm::{ChatMessage, LlmClient, MessageRole};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug};
use crate::workspace_rules;

/// Receptionist agent - first contact for all incoming emails
/// Acts as the chief orchestrator of the agent swarm
pub struct ReceptionistAgent {
    /// Optional LLM client for intelligent classification
    llm: Option<LlmClient>,
}

impl Default for ReceptionistAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl ReceptionistAgent {
    /// Create a new receptionist agent (keyword-based only)
    pub fn new() -> Self {
        Self { llm: None }
    }

    /// Create a receptionist agent with LLM capabilities
    pub fn with_llm(llm: LlmClient) -> Self {
        Self { llm: Some(llm) }
    }

    /// Minimum confidence threshold for automated processing (85%)
    pub const MIN_CONFIDENCE_THRESHOLD: f32 = 0.85;
    /// Maximum retry attempts for low-confidence classifications
    pub const MAX_CLASSIFICATION_RETRIES: u32 = 3;

    /// Classify an email using LLM if available, fallback to keywords
    /// Retries until confidence >= 85% or max retries reached
    pub async fn classify_email_smart(&self, email: &EmailMessage) -> EmailClassification {
        // Try LLM-based classification with retries for low confidence
        if let Some(ref llm) = self.llm {
            let mut best_classification: Option<EmailClassification> = None;
            
            for attempt in 1..=Self::MAX_CLASSIFICATION_RETRIES {
                match self.classify_with_llm(llm, email).await {
                    Ok(classification) => {
                        info!(
                            "LLM classified email '{}' as {:?} with {:.0}% confidence (attempt {})",
                            email.metadata.subject,
                            classification.category,
                            classification.confidence * 100.0,
                            attempt
                        );
                        
                        // If confidence is high enough, use this classification
                        if classification.confidence >= Self::MIN_CONFIDENCE_THRESHOLD {
                            debug!("Confidence {:.0}% >= {:.0}% threshold, proceeding with automation",
                                classification.confidence * 100.0,
                                Self::MIN_CONFIDENCE_THRESHOLD * 100.0
                            );
                            return classification;
                        }
                        
                        // Keep track of best classification so far
                        if best_classification.is_none() || 
                           classification.confidence > best_classification.as_ref().unwrap().confidence {
                            best_classification = Some(classification);
                        }
                        
                        if attempt < Self::MAX_CLASSIFICATION_RETRIES {
                            debug!("Confidence {:.0}% < {:.0}% threshold, retrying ({}/{})",
                                best_classification.as_ref().unwrap().confidence * 100.0,
                                Self::MIN_CONFIDENCE_THRESHOLD * 100.0,
                                attempt,
                                Self::MAX_CLASSIFICATION_RETRIES
                            );
                        }
                    }
                    Err(e) => {
                        warn!("LLM classification attempt {} failed: {}", attempt, e);
                    }
                }
            }
            
            // After all retries, if we have a classification but it's still low confidence,
            // mark it for human review via Telegram notification
            if let Some(mut classification) = best_classification {
                if classification.confidence < Self::MIN_CONFIDENCE_THRESHOLD {
                    warn!(
                        "Classification confidence {:.0}% still below {:.0}% after {} retries - flagging for human review",
                        classification.confidence * 100.0,
                        Self::MIN_CONFIDENCE_THRESHOLD * 100.0,
                        Self::MAX_CLASSIFICATION_RETRIES
                    );
                    classification.requires_human_review = true;
                    classification.reasoning = Some(format!(
                        "{}. [AUTO-FLAGGED: Confidence {:.0}% below {:.0}% threshold after {} retry attempts. Awaiting human review via Telegram.]",
                        classification.reasoning.unwrap_or_default(),
                        classification.confidence * 100.0,
                        Self::MIN_CONFIDENCE_THRESHOLD * 100.0,
                        Self::MAX_CLASSIFICATION_RETRIES
                    ));
                }
                return classification;
            }
            
            warn!("All LLM classification attempts failed, falling back to keywords");
        }

        // Fallback to keyword-based classification
        Self::classify_email_keywords(email)
    }

    /// LLM-based email classification
    async fn classify_with_llm(
        &self,
        llm: &LlmClient,
        email: &EmailMessage,
    ) -> Result<EmailClassification, String> {
        // Load workspace rules to include in the system prompt
        let workspace_rules_content = workspace_rules::load_workspace_rules().await;
        let workspace_rules_section = workspace_rules::format_for_prompt(&workspace_rules_content);
        
        let system_prompt = format!(r#"You are the Receptionist Agent for Amail. Analyze this email and classify it.

{}

Respond with ONLY valid JSON in this exact format:
{{
  "category": "legal|financial|support|hr|sales|marketing|security|compliance|research|scheduling|general",
  "target_agent": "Lawyer|Accountant|CustomerServiceRepresentative|HumanResourcesManager|SalesRepresentative|MarketingCoordinator|SecurityAnalyst|ComplianceOfficer|ResearchAnalyst|Clerk|Executive",
  "priority": "low|normal|high|urgent",
  "confidence": 0.0-1.0,
  "requires_human_review": true|false,
  "reasoning": "Brief explanation of your classification",
  "supporting_agents": ["agent1", "agent2"],
  "plan": [{{"step": 1, "agent": "agent", "action": "what to do"}}]
}}"#, workspace_rules_section);

        let user_message = format!(
            "Classify this email:\n\nFrom: {}\nSubject: {}\n\nBody:\n{}",
            email.metadata.from.address,
            email.metadata.subject,
            email.body_text().unwrap_or("(no body)")
        );

        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: system_prompt,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: user_message,
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        let response = llm
            .chat_completion(messages)
            .await
            .map_err(|e| e.to_string())?;

        // Parse the JSON response
        let json_start = response.content.find('{');
        let json_end = response.content.rfind('}');

        if let (Some(start), Some(end)) = (json_start, json_end) {
            let json_str = &response.content[start..=end];
            let parsed: LlmClassificationResponse =
                serde_json::from_str(json_str).map_err(|e| e.to_string())?;

            return Ok(EmailClassification {
                category: parsed.category.parse().unwrap_or(EmailCategory::General),
                target_role: parse_agent_role(&parsed.target_agent),
                priority: parse_priority(&parsed.priority),
                required_capabilities: vec![AgentCapability::GeneralAssistance],
                requires_human_review: parsed.requires_human_review,
                confidence: parsed.confidence,
                reasoning: Some(parsed.reasoning),
                supporting_agents: parsed.supporting_agents,
                plan: parsed.plan,
            });
        }

        Err("Failed to parse LLM response as JSON".to_string())
    }

    /// Keyword-based email classification (fallback)
    pub fn classify_email_keywords(email: &EmailMessage) -> EmailClassification {
        let subject_lower = email.metadata.subject.to_lowercase();
        let body_lower = email.body_text().unwrap_or("").to_lowercase();

        // Priority keywords
        let is_urgent = subject_lower.contains("urgent")
            || subject_lower.contains("asap")
            || subject_lower.contains("emergency")
            || email.metadata.priority == amail_core::email::EmailPriority::Urgent;

        // Category detection with weighted scoring
        let mut scores: Vec<(EmailCategory, f32)> = vec![
            (EmailCategory::Legal, 0.0),
            (EmailCategory::Financial, 0.0),
            (EmailCategory::Compliance, 0.0),
            (EmailCategory::Support, 0.0),
            (EmailCategory::Scheduling, 0.0),
            (EmailCategory::HumanResources, 0.0),
            (EmailCategory::Sales, 0.0),
            (EmailCategory::Marketing, 0.0),
            (EmailCategory::Security, 0.0),
            (EmailCategory::Research, 0.0),
        ];

        // Legal keywords
        for keyword in &["legal", "contract", "lawsuit", "attorney", "litigation", "subpoena", "court"] {
            if subject_lower.contains(keyword) { scores[0].1 += 2.0; }
            if body_lower.contains(keyword) { scores[0].1 += 1.0; }
        }

        // Financial keywords
        for keyword in &["invoice", "payment", "expense", "budget", "refund", "billing", "receipt"] {
            if subject_lower.contains(keyword) { scores[1].1 += 2.0; }
            if body_lower.contains(keyword) { scores[1].1 += 1.0; }
        }

        // Compliance keywords
        for keyword in &["compliance", "gdpr", "regulation", "audit", "policy", "hipaa", "sox"] {
            if subject_lower.contains(keyword) { scores[2].1 += 2.0; }
            if body_lower.contains(keyword) { scores[2].1 += 1.0; }
        }

        // Support keywords
        for keyword in &["support", "help", "issue", "problem", "bug", "error", "not working"] {
            if subject_lower.contains(keyword) { scores[3].1 += 2.0; }
            if body_lower.contains(keyword) { scores[3].1 += 1.0; }
        }

        // Scheduling keywords
        for keyword in &["meeting", "schedule", "calendar", "appointment", "availability", "reschedule"] {
            if subject_lower.contains(keyword) { scores[4].1 += 2.0; }
            if body_lower.contains(keyword) { scores[4].1 += 1.0; }
        }

        // HR keywords
        for keyword in &["hr", "employee", "hiring", "onboarding", "benefits", "pto", "vacation", "salary"] {
            if subject_lower.contains(keyword) { scores[5].1 += 2.0; }
            if body_lower.contains(keyword) { scores[5].1 += 1.0; }
        }

        // Sales keywords
        for keyword in &["sales", "quote", "proposal", "pricing", "demo", "trial", "purchase"] {
            if subject_lower.contains(keyword) { scores[6].1 += 2.0; }
            if body_lower.contains(keyword) { scores[6].1 += 1.0; }
        }

        // Marketing keywords
        for keyword in &["marketing", "campaign", "promotion", "press", "media", "brand", "newsletter"] {
            if subject_lower.contains(keyword) { scores[7].1 += 2.0; }
            if body_lower.contains(keyword) { scores[7].1 += 1.0; }
        }

        // Security keywords
        for keyword in &["security", "breach", "access", "password", "hack", "vulnerability", "incident"] {
            if subject_lower.contains(keyword) { scores[8].1 += 2.0; }
            if body_lower.contains(keyword) { scores[8].1 += 1.0; }
        }

        // Research keywords
        for keyword in &["research", "analysis", "report", "data", "study", "findings", "statistics"] {
            if subject_lower.contains(keyword) { scores[9].1 += 2.0; }
            if body_lower.contains(keyword) { scores[9].1 += 1.0; }
        }

        // Find highest scoring category
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let (category, top_score) = if scores[0].1 > 0.0 {
            (scores[0].0, scores[0].1)
        } else {
            (EmailCategory::General, 0.0)
        };

        // Calculate confidence based on score differential
        let confidence = if top_score == 0.0 {
            0.5 // Low confidence for general/unclassified
        } else if scores.len() > 1 && scores[1].1 > 0.0 {
            // Higher confidence if clear winner
            let diff = top_score - scores[1].1;
            (0.6 + (diff / top_score) * 0.35).min(0.95)
        } else {
            0.8
        };

        // Determine target role
        let target_role = match category {
            EmailCategory::Legal => AgentRole::Lawyer,
            EmailCategory::Financial => AgentRole::Accountant,
            EmailCategory::Compliance => AgentRole::ComplianceOfficer,
            EmailCategory::Support => AgentRole::CustomerServiceRepresentative,
            EmailCategory::Scheduling => AgentRole::Clerk,
            EmailCategory::HumanResources => AgentRole::HumanResourcesManager,
            EmailCategory::Sales => AgentRole::SalesRepresentative,
            EmailCategory::Marketing => AgentRole::MarketingCoordinator,
            EmailCategory::Security => AgentRole::SecurityAnalyst,
            EmailCategory::Research => AgentRole::ResearchAnalyst,
            EmailCategory::General => AgentRole::Clerk,
        };

        // Determine priority
        let priority = if is_urgent {
            TaskPriority::Urgent
        } else {
            match category {
                EmailCategory::Legal | EmailCategory::Security | EmailCategory::Compliance => {
                    TaskPriority::High
                }
                EmailCategory::Financial | EmailCategory::Support => TaskPriority::Normal,
                _ => TaskPriority::Normal,
            }
        };

        // Determine required capabilities
        let required_capabilities = match category {
            EmailCategory::Legal => vec![AgentCapability::LegalQuery, AgentCapability::ContractReview],
            EmailCategory::Financial => vec![AgentCapability::FinanceTracking, AgentCapability::ExpenseManagement],
            EmailCategory::Compliance => vec![AgentCapability::ComplianceCheck, AgentCapability::RegulatoryAdherence],
            _ => vec![AgentCapability::GeneralAssistance],
        };

        // Flag for human review if confidence is below threshold (85%)
        // Legal and urgent emails always get human review regardless of confidence
        let requires_human_review = category == EmailCategory::Legal 
            || is_urgent 
            || confidence < Self::MIN_CONFIDENCE_THRESHOLD;
        
        EmailClassification {
            category,
            target_role,
            priority,
            required_capabilities,
            requires_human_review,
            confidence,
            reasoning: if requires_human_review && confidence < Self::MIN_CONFIDENCE_THRESHOLD {
                Some(format!(
                    "[AUTO-FLAGGED: Keyword classification confidence {:.0}% below {:.0}% threshold. Awaiting human review via Telegram.]",
                    confidence * 100.0,
                    Self::MIN_CONFIDENCE_THRESHOLD * 100.0
                ))
            } else {
                None
            },
            supporting_agents: vec![],
            plan: vec![],
        }
    }

    /// Legacy sync method for backward compatibility
    pub fn classify_email(email: &EmailMessage) -> EmailClassification {
        Self::classify_email_keywords(email)
    }

    /// Create a task from classified email
    pub fn create_task(email: EmailMessage, classification: &EmailClassification) -> Task {
        let description = format!(
            "[{}] {} - from {}",
            classification.category.as_str(),
            email.metadata.subject,
            email.metadata.from.address
        );

        let mut task = Task::from_email(email);
        task.description = description;
        task.priority = classification.priority;
        task.task_type = match classification.category {
            EmailCategory::Legal => TaskType::LegalReview,
            EmailCategory::Financial => TaskType::FinancialProcessing,
            EmailCategory::Compliance => TaskType::ComplianceCheck,
            EmailCategory::Research => TaskType::Research,
            _ => TaskType::ProcessEmail,
        };

        task
    }
}

/// Email classification result
#[derive(Debug, Clone)]
pub struct EmailClassification {
    /// Detected category
    pub category: EmailCategory,
    /// Target agent role
    pub target_role: AgentRole,
    /// Task priority
    pub priority: TaskPriority,
    /// Required capabilities
    pub required_capabilities: Vec<AgentCapability>,
    /// Whether human review is recommended
    pub requires_human_review: bool,
    /// Classification confidence (0.0 - 1.0)
    pub confidence: f32,
    /// LLM reasoning (if available)
    pub reasoning: Option<String>,
    /// Supporting agents for multi-agent orchestration
    pub supporting_agents: Vec<String>,
    /// Execution plan for complex tasks
    pub plan: Vec<PlanStep>,
}

/// A step in the execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub step: u32,
    pub agent: String,
    pub action: String,
}

/// LLM classification response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LlmClassificationResponse {
    category: String,
    target_agent: String,
    priority: String,
    confidence: f32,
    requires_human_review: bool,
    reasoning: String,
    #[serde(default)]
    supporting_agents: Vec<String>,
    #[serde(default)]
    plan: Vec<PlanStep>,
}

/// Email categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailCategory {
    Legal,
    Financial,
    Compliance,
    Support,
    Scheduling,
    HumanResources,
    Sales,
    Marketing,
    Security,
    Research,
    General,
}

impl EmailCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Legal => "Legal",
            Self::Financial => "Financial",
            Self::Compliance => "Compliance",
            Self::Support => "Support",
            Self::Scheduling => "Scheduling",
            Self::HumanResources => "HR",
            Self::Sales => "Sales",
            Self::Marketing => "Marketing",
            Self::Security => "Security",
            Self::Research => "Research",
            Self::General => "General",
        }
    }
}

impl std::str::FromStr for EmailCategory {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "legal" => Ok(Self::Legal),
            "financial" => Ok(Self::Financial),
            "compliance" => Ok(Self::Compliance),
            "support" => Ok(Self::Support),
            "scheduling" => Ok(Self::Scheduling),
            "hr" | "humanresources" => Ok(Self::HumanResources),
            "sales" => Ok(Self::Sales),
            "marketing" => Ok(Self::Marketing),
            "security" => Ok(Self::Security),
            "research" => Ok(Self::Research),
            _ => Ok(Self::General),
        }
    }
}

/// Parse agent role from string
fn parse_agent_role(s: &str) -> AgentRole {
    match s.to_lowercase().as_str() {
        "receptionist" => AgentRole::Receptionist,
        "clerk" => AgentRole::Clerk,
        "executive" => AgentRole::Executive,
        "draftchecker" | "draft_checker" => AgentRole::DraftChecker,
        "lawyer" => AgentRole::Lawyer,
        "accountant" => AgentRole::Accountant,
        "humanresourcesmanager" | "hr" | "hrmanager" => AgentRole::HumanResourcesManager,
        "itsupporttechnician" | "it" | "itsupport" => AgentRole::ITSupportTechnician,
        "customerservicerepresentative" | "customerservice" | "support" => AgentRole::CustomerServiceRepresentative,
        "salesrepresentative" | "sales" => AgentRole::SalesRepresentative,
        "marketingcoordinator" | "marketing" => AgentRole::MarketingCoordinator,
        "projectmanager" => AgentRole::ProjectManager,
        "researchanalyst" | "research" => AgentRole::ResearchAnalyst,
        "complianceofficer" | "compliance" => AgentRole::ComplianceOfficer,
        "securityanalyst" | "security" => AgentRole::SecurityAnalyst,
        "dataanalyst" | "data" => AgentRole::DataAnalyst,
        _ => AgentRole::Clerk,
    }
}

/// Parse task priority from string
fn parse_priority(s: &str) -> TaskPriority {
    match s.to_lowercase().as_str() {
        "urgent" => TaskPriority::Urgent,
        "high" => TaskPriority::High,
        "low" => TaskPriority::Low,
        _ => TaskPriority::Normal,
    }
}
