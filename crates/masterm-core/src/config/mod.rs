//! Configuration system for MASTerm
//!
//! Supports three tiers of configuration (in order of precedence):
//! 1. Enterprise config (`/etc/masterm/enterprise.toml`)
//! 2. User config (`~/.masterm.toml`)
//! 3. Project config (`.masterm.toml` in current directory)

mod enterprise;
mod loader;
mod schema;

pub use enterprise::EnterpriseConfig;
pub use loader::ConfigLoader;
pub use schema::*;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Core settings
    pub core: CoreConfig,

    /// Prompt configuration
    pub prompt: PromptConfig,

    /// Plugin settings
    pub plugins: PluginsConfig,

    /// Safety features
    pub safety: SafetyConfig,

    /// Cache settings
    pub cache: CacheConfig,

    /// Telemetry settings
    pub telemetry: TelemetryConfig,
}

/// Core configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CoreConfig {
    /// Shell to configure
    pub shell: String,

    /// Startup mode: minimal, dev, ops
    pub mode: String,

    /// Enabled features
    pub features: Vec<String>,

    /// Log level
    pub log_level: String,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            shell: "auto".to_string(),
            mode: "dev".to_string(),
            features: vec![
                "prompt".to_string(),
                "plugins".to_string(),
                "safety".to_string(),
            ],
            log_level: "warn".to_string(),
        }
    }
}

/// Prompt configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PromptConfig {
    /// Prompt format: simple, powerline, minimal
    pub format: String,

    /// Use transient prompt
    pub transient: bool,

    /// Add newline before prompt
    pub add_newline: bool,

    /// Left prompt segments
    pub left: Vec<String>,

    /// Right prompt segments
    pub right: Vec<String>,

    /// Icon settings
    pub icons: IconsConfig,

    /// Color theme
    pub colors: ColorsConfig,
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self {
            format: "powerline".to_string(),
            transient: true,
            add_newline: true,
            left: vec![
                "context".to_string(),
                "directory".to_string(),
                "git_branch".to_string(),
                "git_status".to_string(),
            ],
            right: vec!["cmd_duration".to_string(), "time".to_string()],
            icons: IconsConfig::default(),
            colors: ColorsConfig::default(),
        }
    }
}

/// Icon configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IconsConfig {
    /// Icon mode: auto, nerd, unicode, ascii, none
    pub mode: String,

    /// Icon overrides
    #[serde(default)]
    pub overrides: std::collections::HashMap<String, String>,
}

impl Default for IconsConfig {
    fn default() -> Self {
        Self {
            mode: "auto".to_string(),
            overrides: std::collections::HashMap::new(),
        }
    }
}

/// Color configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorsConfig {
    /// Theme name: catppuccin, dracula, nord, gruvbox
    pub theme: String,

    /// Color overrides
    #[serde(default)]
    pub overrides: std::collections::HashMap<String, String>,
}

impl Default for ColorsConfig {
    fn default() -> Self {
        Self {
            theme: "catppuccin".to_string(),
            overrides: std::collections::HashMap::new(),
        }
    }
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PluginsConfig {
    /// Plugin registry URL
    pub registry: Option<String>,

    /// Explicitly enabled plugins
    pub enabled: Vec<String>,

    /// Explicitly disabled plugins
    pub disabled: Vec<String>,

    /// Per-plugin configuration
    #[serde(flatten)]
    pub plugin_configs: std::collections::HashMap<String, toml::Value>,
}

/// Safety configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SafetyConfig {
    /// Enable production detection
    pub prod_detection: bool,

    /// Patterns that indicate production
    pub prod_patterns: Vec<String>,

    /// Commands that require confirmation
    pub dangerous_commands: Vec<String>,

    /// Visual warning style
    pub warning_style: String,
}

impl Default for SafetyConfig {
    fn default() -> Self {
        Self {
            prod_detection: true,
            prod_patterns: vec![
                "**/prod/**".to_string(),
                "**/production/**".to_string(),
                "/var/www/**".to_string(),
            ],
            dangerous_commands: vec![
                "rm -rf".to_string(),
                "DROP DATABASE".to_string(),
                "kubectl delete".to_string(),
                "terraform destroy".to_string(),
            ],
            warning_style: "banner".to_string(),
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    /// Cache directory
    pub directory: Option<PathBuf>,

    /// Cache TTL in seconds
    pub ttl: u64,

    /// Max cache size in MB
    pub max_size: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            directory: None,
            ttl: 300,
            max_size: 100,
        }
    }
}

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TelemetryConfig {
    /// Enable anonymous telemetry
    pub enabled: bool,
}
