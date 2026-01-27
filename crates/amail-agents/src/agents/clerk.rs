// =============================================================================
// Clerk Agent - Directory Management and Scheduling
// =============================================================================
// Handles: Scheduling, calendar management, meeting coordination, reminders
// =============================================================================

use amail_core::agent::AgentRole;
use amail_llm::{ChatMessage, LlmClient, MessageRole};
use crate::workspace_rules;

/// Clerk agent - manages directory, scheduling, and agent registry
pub struct ClerkAgent {
    llm: Option<LlmClient>,
}

impl Default for ClerkAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl ClerkAgent {
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
        
        format!(r#"You are the Clerk Agent for Amail. You specialize in scheduling, calendar management, and coordination.

{}

## Your Responsibilities:
- Schedule meetings and appointments
- Manage calendar conflicts
- Send meeting reminders
- Coordinate between parties for scheduling
- Handle rescheduling requests
- Maintain the agent directory

## Guidelines:
- Always confirm time zones when scheduling
- Check for conflicts before confirming appointments
- Provide clear meeting details (time, location/link, agenda)
- Be polite and professional in all communications
- Offer alternative times when conflicts arise

When responding, include:
1. Proposed time(s)
2. Duration
3. Participants
4. Any conflicts or considerations"#, workspace_rules_section)
    }
    
    /// Process a scheduling query with LLM
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
