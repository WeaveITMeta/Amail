// =============================================================================
// MCP Client - Model Context Protocol Integration
// =============================================================================
// Table of Contents:
// 1. McpClient struct
// 2. Server connection management
// 3. Tool invocation
// 4. Resource access
// 5. Built-in MCP servers (filesystem, memory, etc.)
// =============================================================================

use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::types::{McpResource, McpTool, McpToolResult};

/// MCP Client for connecting to MCP servers
pub struct McpClient {
    /// HTTP client
    client: Client,
    /// Connected servers (name -> connection)
    servers: Arc<RwLock<HashMap<String, ServerConnection>>>,
    /// Tool cache for quick lookup
    tool_cache: Arc<RwLock<HashMap<String, (String, McpTool)>>>,
}

/// Server connection info
#[derive(Debug, Clone)]
pub struct ServerConnection {
    /// Server URL
    pub url: String,
    /// Server name
    pub name: String,
    /// Available tools
    pub tools: Vec<McpTool>,
    /// Available resources
    pub resources: Vec<McpResource>,
    /// Connection status
    pub status: ConnectionStatus,
    /// Last health check
    pub last_health_check: Option<chrono::DateTime<chrono::Utc>>,
}

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()?;

        Ok(Self {
            client,
            servers: Arc::new(RwLock::new(HashMap::new())),
            tool_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Connect to an MCP server
    pub async fn connect(&self, name: &str, url: &str) -> Result<()> {
        info!("Connecting to MCP server '{}' at {}", name, url);

        // Discover tools
        let tools = self.discover_tools(url).await?;
        info!("Discovered {} tools from '{}'", tools.len(), name);
        for tool in &tools {
            debug!("  - {}: {}", tool.name, tool.description.as_deref().unwrap_or("No description"));
        }

        // Discover resources
        let resources = self.discover_resources(url).await.unwrap_or_default();
        debug!("Discovered {} resources from '{}'", resources.len(), name);

        // Update tool cache
        {
            let mut cache = self.tool_cache.write().await;
            for tool in &tools {
                cache.insert(tool.name.clone(), (name.to_string(), tool.clone()));
            }
        }

        // Store connection
        {
            let mut servers = self.servers.write().await;
            servers.insert(
                name.to_string(),
                ServerConnection {
                    url: url.to_string(),
                    name: name.to_string(),
                    tools,
                    resources,
                    status: ConnectionStatus::Connected,
                    last_health_check: Some(chrono::Utc::now()),
                },
            );
        }

        Ok(())
    }

    /// Connect to multiple servers from config
    pub async fn connect_from_config(&self, servers: &[(String, String)]) -> Vec<Result<()>> {
        let mut results = Vec::new();
        for (name, url) in servers {
            results.push(self.connect(name, url).await);
        }
        results
    }

    /// Disconnect from an MCP server
    pub async fn disconnect(&self, name: &str) {
        // Remove from tool cache
        {
            let mut cache = self.tool_cache.write().await;
            cache.retain(|_, (server, _)| server != name);
        }
        // Remove server
        {
            let mut servers = self.servers.write().await;
            servers.remove(name);
        }
        info!("Disconnected from MCP server '{}'", name);
    }

    /// List connected servers
    pub async fn list_servers(&self) -> Vec<String> {
        let servers = self.servers.read().await;
        servers.keys().cloned().collect()
    }

    /// Get server connection info
    pub async fn get_server(&self, name: &str) -> Option<ServerConnection> {
        let servers = self.servers.read().await;
        servers.get(name).cloned()
    }

    /// List tools from a server
    pub async fn list_tools(&self, server: &str) -> Option<Vec<McpTool>> {
        let servers = self.servers.read().await;
        servers.get(server).map(|s| s.tools.clone())
    }

    /// List all tools from all servers
    pub async fn all_tools(&self) -> Vec<(String, McpTool)> {
        let servers = self.servers.read().await;
        servers
            .iter()
            .flat_map(|(name, conn)| conn.tools.iter().map(move |t| (name.clone(), t.clone())))
            .collect()
    }

    /// Find a tool by name (searches all servers)
    pub async fn find_tool(&self, tool_name: &str) -> Option<(String, McpTool)> {
        let cache = self.tool_cache.read().await;
        cache.get(tool_name).cloned()
    }

    /// Get all tools formatted for LLM context
    pub async fn tools_for_llm(&self) -> Vec<LlmToolDefinition> {
        let servers = self.servers.read().await;
        servers
            .iter()
            .flat_map(|(server_name, conn)| {
                conn.tools.iter().map(move |tool| LlmToolDefinition {
                    name: format!("mcp_{}_{}", server_name, tool.name),
                    description: tool.description.clone().unwrap_or_else(|| format!("MCP tool from {}", server_name)),
                    parameters: tool.input_schema.clone().unwrap_or(serde_json::json!({
                        "type": "object",
                        "properties": {}
                    })),
                    server: server_name.clone(),
                    original_name: tool.name.clone(),
                })
            })
            .collect()
    }

    /// Call a tool on a server
    pub async fn call_tool(
        &self,
        server: &str,
        tool: &str,
        arguments: serde_json::Value,
    ) -> Result<McpToolResult> {
        let servers = self.servers.read().await;
        let conn = servers
            .get(server)
            .ok_or_else(|| anyhow!("Server '{}' not connected", server))?;

        let url = format!("{}/tools/{}/call", conn.url, tool);

        debug!("Calling MCP tool {}/{} with args: {:?}", server, tool, arguments);

        let request = McpToolCallRequest {
            name: tool.to_string(),
            arguments,
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to call MCP tool: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("MCP tool call failed {}: {}", status, body);
            return Ok(McpToolResult {
                success: false,
                content: vec![McpContent::Text {
                    text: format!("Error: {}", body),
                }],
                is_error: true,
            });
        }

        let result: McpToolCallResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse MCP response: {}", e))?;

        Ok(McpToolResult {
            success: !result.is_error.unwrap_or(false),
            content: result.content,
            is_error: result.is_error.unwrap_or(false),
        })
    }

    /// Call a tool by its full name (mcp_server_toolname format)
    pub async fn call_tool_by_name(
        &self,
        full_name: &str,
        arguments: serde_json::Value,
    ) -> Result<McpToolResult> {
        // Parse mcp_server_tool format
        if let Some(rest) = full_name.strip_prefix("mcp_") {
            if let Some(idx) = rest.find('_') {
                let server = &rest[..idx];
                let tool = &rest[idx + 1..];
                return self.call_tool(server, tool, arguments).await;
            }
        }
        
        // Try direct lookup in cache
        if let Some((server, _tool)) = self.find_tool(full_name).await {
            return self.call_tool(&server, full_name, arguments).await;
        }
        
        Err(anyhow!("Tool '{}' not found", full_name))
    }

    /// Read a resource from a server
    pub async fn read_resource(&self, server: &str, uri: &str) -> Result<String> {
        let servers = self.servers.read().await;
        let conn = servers
            .get(server)
            .ok_or_else(|| anyhow!("Server '{}' not connected", server))?;

        let url = format!("{}/resources/read", conn.url);

        let request = McpResourceReadRequest {
            uri: uri.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to read MCP resource: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!("MCP resource read failed {}: {}", status, body));
        }

        let result: McpResourceReadResponse = response.json().await?;
        
        // Extract text content
        let text = result
            .contents
            .into_iter()
            .filter_map(|c| match c {
                McpResourceContent::Text { text, .. } => Some(text),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(text)
    }

    /// Discover tools from a server
    async fn discover_tools(&self, url: &str) -> Result<Vec<McpTool>> {
        let tools_url = format!("{}/tools/list", url);

        let response = self
            .client
            .get(&tools_url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to discover tools: {}", e))?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let result: McpToolsListResponse = response.json().await?;
        Ok(result.tools)
    }

    /// Discover resources from a server
    async fn discover_resources(&self, url: &str) -> Result<Vec<McpResource>> {
        let resources_url = format!("{}/resources/list", url);

        let response = self
            .client
            .get(&resources_url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to discover resources: {}", e))?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let result: McpResourcesListResponse = response.json().await?;
        Ok(result.resources)
    }
}

impl Default for McpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create MCP client")
    }
}

// =============================================================================
// MCP Protocol Types
// =============================================================================

#[derive(Debug, Serialize)]
struct McpToolCallRequest {
    name: String,
    arguments: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct McpToolCallResponse {
    content: Vec<McpContent>,
    #[serde(default)]
    is_error: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpContent {
    Text { text: String },
    Image { data: String, mime_type: String },
    Resource { uri: String, text: Option<String> },
}

#[derive(Debug, Serialize)]
struct McpResourceReadRequest {
    uri: String,
}

#[derive(Debug, Deserialize)]
struct McpResourceReadResponse {
    contents: Vec<McpResourceContent>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum McpResourceContent {
    Text { uri: String, text: String },
    Blob { uri: String, blob: String },
}

/// Tool definition for LLM integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmToolDefinition {
    /// Full tool name (mcp_server_toolname)
    pub name: String,
    /// Tool description
    pub description: String,
    /// JSON Schema for parameters
    pub parameters: serde_json::Value,
    /// Server name
    pub server: String,
    /// Original tool name on server
    pub original_name: String,
}

#[derive(Debug, Deserialize)]
struct McpToolsListResponse {
    tools: Vec<McpTool>,
}

#[derive(Debug, Deserialize)]
struct McpResourcesListResponse {
    resources: Vec<McpResource>,
}
