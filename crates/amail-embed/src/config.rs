//! Configuration for Amail Embed Widgets
//!
//! Table of Contents:
//! 1. AmailConfig - Main configuration
//! 2. ThemeConfig - Theming options

use serde::{Deserialize, Serialize};

/// Main configuration for Amail widgets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmailConfig {
    /// API URL for the Amail server
    pub api_url: String,
    /// Optional API key for authentication
    pub api_key: Option<String>,
    /// Theme configuration
    pub theme: ThemeConfig,
    /// Enable debug mode
    pub debug: bool,
}

impl Default for AmailConfig {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:7000".to_string(),
            api_key: None,
            theme: ThemeConfig::default(),
            debug: false,
        }
    }
}

/// Theme configuration for widgets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Primary color (hex)
    pub primary_color: String,
    /// Background color (hex)
    pub background_color: String,
    /// Text color (hex)
    pub text_color: String,
    /// Border radius in pixels
    pub border_radius: u32,
    /// Font family
    pub font_family: String,
    /// Compact mode (smaller widgets)
    pub compact: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            primary_color: "#3b82f6".to_string(),
            background_color: "#ffffff".to_string(),
            text_color: "#1e293b".to_string(),
            border_radius: 8,
            font_family: "system-ui, -apple-system, sans-serif".to_string(),
            compact: false,
        }
    }
}

impl ThemeConfig {
    /// Generate CSS variables from theme config
    pub fn to_css_vars(&self) -> String {
        format!(
            "--amail-primary: {}; --amail-bg: {}; --amail-text: {}; --amail-radius: {}px; --amail-font: {};",
            self.primary_color,
            self.background_color,
            self.text_color,
            self.border_radius,
            self.font_family
        )
    }
}
