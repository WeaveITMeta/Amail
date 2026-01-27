// =============================================================================
// Accountant Agent - Finance Tracking and Expense Management
// =============================================================================
// Handles: Invoices, expenses, budgets, financial queries, payment tracking
// =============================================================================

use amail_core::agent::AgentRole;
use amail_llm::{ChatMessage, LlmClient, MessageRole};
use crate::workspace_rules;

/// Accountant agent - handles finance tracking, expenses, budgets
pub struct AccountantAgent {
    llm: Option<LlmClient>,
}

impl Default for AccountantAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl AccountantAgent {
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
        
        format!(r#"You are the Accountant Agent for Amail. You specialize in financial matters, expense tracking, and budget management.

{}

## Your Responsibilities:
- Process and categorize invoices
- Track expenses and payments
- Manage budget inquiries
- Handle financial reporting requests
- Flag unusual financial activity
- Assist with payment scheduling

## Guidelines:
- Be precise with numbers and calculations
- Always verify amounts before confirming
- Maintain confidentiality of financial data
- Use clear, professional financial terminology
- Flag any discrepancies or concerns immediately

When responding, structure your analysis clearly with:
1. Financial Summary
2. Key Figures
3. Action Items
4. Recommendations"#, workspace_rules_section)
    }
    
    /// Process a financial query with LLM
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
