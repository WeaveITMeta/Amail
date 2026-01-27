// =============================================================================
// Executive Agent - High-Level Decisions and Onboarding
// =============================================================================
// Handles: Strategic decisions, escalations, onboarding, high-priority matters
// =============================================================================

use amail_core::agent::AgentRole;
use amail_llm::{ChatMessage, LlmClient, MessageRole};
use crate::workspace_rules;

/// Executive agent - handles onboarding, email provisioning, high-level decisions
pub struct ExecutiveAgent {
    llm: Option<LlmClient>,
}

impl Default for ExecutiveAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutiveAgent {
    pub fn new() -> Self {
        Self { llm: None }
    }
    
    pub fn with_llm(llm: LlmClient) -> Self {
        Self { llm: Some(llm) }
    }
    
    /// Build the system prompt with workspace rules
    pub async fn build_system_prompt(&self) -> String {
        let workspace_rules_content = workspace_rules::load_workspace_rules().await;
        let workspace_rules_section = workspace_rules::format_for_prompt(&workspace_rules_content);
        
        format!(r#"You are the Executive Agent for Amail. You handle high-level decisions, escalations, and strategic matters.

{}

## Your Responsibilities:
- Make high-level decisions on escalated matters
- Handle VIP and priority communications
- Oversee onboarding of new domains and mailboxes
- Coordinate between departments on complex issues
- Approve or reject significant actions
- Set strategic direction for email handling

## Guidelines:
- Consider business impact in all decisions
- Be decisive but thoughtful
- Escalate to human review when stakes are very high
- Maintain executive-level professionalism
- Document reasoning for major decisions

When responding, provide:
1. Decision/Recommendation
2. Rationale
3. Next Steps
4. Risk Assessment (if applicable)"#, workspace_rules_section)
    }
    
    /// Process an executive query with LLM
    pub async fn process_query(&self, query: &str) -> Result<String, String> {
        let llm = self.llm.as_ref().ok_or("LLM not configured")?;
        
        let system_prompt = self.build_system_prompt().await;
        
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: system_prompt,
                tool_calls: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: query.to_string(),
                tool_calls: None,
                tool_call_id: None,
            },
        ];
        
        let response = llm.chat_completion(messages).await.map_err(|e| e.to_string())?;
        Ok(response.content)
    }
}
