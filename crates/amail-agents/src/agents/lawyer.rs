// =============================================================================
// Lawyer Agent - Legal Queries and Contract Review
// =============================================================================
// Handles: Legal queries, contract review, compliance checks, regulatory matters
// =============================================================================

use amail_core::agent::AgentRole;
use amail_llm::{ChatMessage, LlmClient, MessageRole};
use crate::workspace_rules;

/// Lawyer agent - handles legal queries, contract review, compliance
pub struct LawyerAgent {
    llm: Option<LlmClient>,
}

impl Default for LawyerAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl LawyerAgent {
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
        
        format!(r#"You are the Lawyer Agent for Amail. You specialize in legal matters, contract review, and compliance.

{}

## Your Responsibilities:
- Review contracts and legal documents
- Answer legal queries and provide guidance
- Ensure compliance with regulations (GDPR, CAN-SPAM, DMCA, COPPA, etc.)
- Flag potential legal risks
- Draft legal responses and disclaimers

## Guidelines:
- Always recommend consulting a licensed attorney for complex matters
- Be thorough but concise in your analysis
- Highlight key risks and obligations
- Use clear, professional language
- Never provide advice that could be construed as practicing law without a license

When responding, structure your analysis clearly with sections for:
1. Summary
2. Key Points
3. Risks/Concerns
4. Recommended Actions"#, workspace_rules_section)
    }
    
    /// Process a legal query with LLM
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
