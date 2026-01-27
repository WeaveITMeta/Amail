// =============================================================================
// Draft Checker Agent - Output Review and Quality Assurance
// =============================================================================
// Handles: Review drafts, quality assurance, tone checking, error detection
// =============================================================================

use amail_core::agent::AgentRole;
use amail_llm::{ChatMessage, LlmClient, MessageRole};
use crate::workspace_rules;

/// Draft checker agent - reviews high-stakes outputs before sending
pub struct DraftCheckerAgent {
    llm: Option<LlmClient>,
}

impl Default for DraftCheckerAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl DraftCheckerAgent {
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
        
        format!(r#"You are the Draft Checker Agent for Amail. You review all outgoing communications for quality and accuracy.

{}

## Your Responsibilities:
- Review draft emails before sending
- Check for spelling, grammar, and punctuation errors
- Verify tone is appropriate for the recipient
- Ensure accuracy of facts and figures
- Flag potential issues or concerns
- Suggest improvements when needed

## Guidelines:
- Be thorough but efficient in reviews
- Prioritize clarity and professionalism
- Check for sensitive information that shouldn't be shared
- Verify links and attachments are correct
- Ensure compliance with communication policies

When reviewing, provide:
1. Overall Assessment (Approve/Revise/Reject)
2. Issues Found (if any)
3. Suggested Corrections
4. Tone/Style Feedback"#, workspace_rules_section)
    }
    
    /// Review a draft with LLM
    pub async fn review_draft(&self, draft: &str) -> Result<String, String> {
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
                content: format!("Please review this draft email:\n\n{}", draft),
                tool_calls: None,
                tool_call_id: None,
            },
        ];
        
        let response = llm.chat_completion(messages).await.map_err(|e| e.to_string())?;
        Ok(response.content)
    }
}
