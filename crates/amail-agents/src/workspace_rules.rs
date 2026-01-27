// =============================================================================
// Workspace Rules - Global rules for all AI agents
// =============================================================================
// This module provides access to the workspace_rules.md file that contains
// general guidelines all agents should follow in addition to their role-specific
// training.
// =============================================================================

use std::path::Path;
use std::sync::OnceLock;
use tokio::sync::RwLock;
use tracing::{info, warn, debug};

const WORKSPACE_RULES_PATH: &str = "data/config/workspace_rules.md";

static WORKSPACE_RULES_CACHE: OnceLock<RwLock<Option<String>>> = OnceLock::new();

fn get_cache() -> &'static RwLock<Option<String>> {
    WORKSPACE_RULES_CACHE.get_or_init(|| RwLock::new(None))
}

/// Load workspace rules from file
/// Returns the rules content or a default message if not found
pub async fn load_workspace_rules() -> String {
    // Check cache first
    {
        let cache = get_cache().read().await;
        if let Some(ref rules) = *cache {
            debug!("Using cached workspace rules");
            return rules.clone();
        }
    }
    
    // Load from file
    let rules = load_from_file().await;
    
    // Update cache
    {
        let mut cache = get_cache().write().await;
        *cache = Some(rules.clone());
    }
    
    rules
}

/// Force reload workspace rules from file (bypasses cache)
pub async fn reload_workspace_rules() -> String {
    let rules = load_from_file().await;
    
    // Update cache
    {
        let mut cache = get_cache().write().await;
        *cache = Some(rules.clone());
    }
    
    info!("Workspace rules reloaded ({} bytes)", rules.len());
    rules
}

/// Clear the workspace rules cache
pub async fn clear_cache() {
    let mut cache = get_cache().write().await;
    *cache = None;
    debug!("Workspace rules cache cleared");
}

async fn load_from_file() -> String {
    let path = Path::new(WORKSPACE_RULES_PATH);
    
    match tokio::fs::read_to_string(path).await {
        Ok(content) => {
            info!("Loaded workspace rules from {} ({} bytes)", WORKSPACE_RULES_PATH, content.len());
            content
        }
        Err(e) => {
            warn!("Failed to load workspace rules from {}: {}. Using defaults.", WORKSPACE_RULES_PATH, e);
            default_rules()
        }
    }
}

fn default_rules() -> String {
    r#"# Amail Workspace Rules

These rules apply to ALL AI agents in the Amail system.

## Communication Style
- Be professional and concise
- Use clear, actionable language

## Response Guidelines
- Always acknowledge receipt of emails
- Provide estimated response times when applicable
- Escalate urgent matters immediately

## Privacy & Security
- Never share sensitive information externally
- Verify sender identity for sensitive requests
"#.to_string()
}

/// Format workspace rules for inclusion in an LLM system prompt
pub fn format_for_prompt(rules: &str) -> String {
    format!(
        r#"<workspace_rules>
The following are general workspace rules that apply to all agents. Follow these guidelines in addition to your role-specific instructions:

{}
</workspace_rules>"#,
        rules.trim()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rules() {
        let rules = default_rules();
        assert!(rules.contains("Amail Workspace Rules"));
        assert!(rules.contains("Communication Style"));
    }

    #[test]
    fn test_format_for_prompt() {
        let rules = "Test rule 1\nTest rule 2";
        let formatted = format_for_prompt(rules);
        assert!(formatted.contains("<workspace_rules>"));
        assert!(formatted.contains("Test rule 1"));
        assert!(formatted.contains("</workspace_rules>"));
    }
}
