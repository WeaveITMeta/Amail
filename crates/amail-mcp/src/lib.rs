// =============================================================================
// AMAIL-MCP - Model Context Protocol Client
// =============================================================================
// Table of Contents:
// 1. McpClient - MCP server connection
// 2. Tool discovery and invocation
// 3. Resource access
// =============================================================================

pub mod client;
pub mod types;

pub use client::{ConnectionStatus, LlmToolDefinition, McpClient, McpContent, ServerConnection};
pub use types::{McpTool, McpResource, McpToolResult};
