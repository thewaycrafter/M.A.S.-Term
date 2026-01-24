//! Plugin API trait and types

use super::PluginManifest;
use crate::prompt::Segment;
use async_trait::async_trait;
use std::path::PathBuf;

use std::collections::HashMap;
use thiserror::Error;

/// Plugin error types
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin initialization failed: {0}")]
    InitError(String),

    #[error("Missing required binary: {0}")]
    MissingBinary(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Plugin execution error: {0}")]
    ExecutionError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Context provided to plugins during initialization and execution
#[derive(Debug, Clone)]
pub struct PluginContext {
    /// Plugin-specific configuration
    pub config: HashMap<String, toml::Value>,

    /// Current working directory
    pub cwd: PathBuf,

    /// Environment variables available to plugin
    pub env_vars: HashMap<String, String>,

    /// Cache handle for plugin
    pub cache_key: String,
}

impl PluginContext {
    /// Create a new plugin context
    pub fn new(name: &str, cwd: PathBuf) -> Self {
        Self {
            config: HashMap::new(),
            cwd,
            env_vars: std::env::vars().collect(),
            cache_key: format!("plugin_{}", name),
        }
    }

    /// Get a config value as string
    pub fn get_config_string(&self, key: &str) -> Option<String> {
        self.config
            .get(key)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    /// Get a config value as bool
    pub fn get_config_bool(&self, key: &str) -> Option<bool> {
        self.config.get(key).and_then(|v| v.as_bool())
    }

    /// Get a config value as string list
    pub fn get_string_list(&self, key: &str) -> Result<Vec<String>, PluginError> {
        match self.config.get(key) {
            Some(toml::Value::Array(arr)) => Ok(arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()),
            None => Ok(Vec::new()),
            _ => Err(PluginError::ConfigError(format!(
                "{} is not a string list",
                key
            ))),
        }
    }

    /// Check if a binary is available
    pub fn binary_exists(&self, name: &str) -> bool {
        which::which(name).is_ok()
    }
}

/// Context for prompt generation
#[derive(Debug, Clone)]
pub struct PromptContext {
    /// Current working directory
    pub cwd: PathBuf,

    /// Last command exit code
    pub last_exit_code: i32,

    /// Last command duration
    pub last_command_duration: std::time::Duration,

    /// Shell type
    pub shell: crate::config::ShellType,

    /// Terminal width
    pub terminal_width: u16,

    /// Environment variables
    pub env_vars: HashMap<String, String>,
}

impl PromptContext {
    /// Create new prompt context
    pub fn new(cwd: PathBuf, exit_code: i32, duration: std::time::Duration) -> Self {
        Self {
            cwd,
            last_exit_code: exit_code,
            last_command_duration: duration,
            shell: crate::config::ShellType::detect(),
            terminal_width: crossterm::terminal::size().map(|(w, _)| w).unwrap_or(80),
            env_vars: std::env::vars().collect(),
        }
    }
}

/// Detection context for activation checks
#[derive(Debug, Clone)]
pub struct DetectionContext {
    /// Current working directory
    pub cwd: PathBuf,

    /// Files in cwd
    pub files: Vec<String>,

    /// Environment variables
    pub env_vars: HashMap<String, String>,
}

impl DetectionContext {
    /// Create new detection context
    pub fn new(cwd: PathBuf) -> Self {
        let files = std::fs::read_dir(&cwd)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default();

        Self {
            cwd,
            files,
            env_vars: std::env::vars().collect(),
        }
    }

    /// Check if a file exists in cwd
    pub fn has_file(&self, name: &str) -> bool {
        self.files.iter().any(|f| f == name)
    }

    /// Check if a file matches a pattern
    pub fn has_file_matching(&self, pattern: &str) -> bool {
        let re = regex::Regex::new(pattern).ok();
        re.map(|r| self.files.iter().any(|f| r.is_match(f)))
            .unwrap_or(false)
    }
}

/// Command action for safety guards
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandAction {
    /// Allow command to proceed
    Allow,

    /// Show warning but allow
    Warn(String),

    /// Require confirmation
    Confirm(String),

    /// Block command entirely
    Block(String),
}

/// Core trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin manifest
    fn manifest(&self) -> &PluginManifest;

    /// Called once when plugin is loaded
    async fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError>;

    /// Check if plugin should activate for current context
    fn should_activate(&self, ctx: &DetectionContext) -> bool;

    /// Generate prompt segments
    async fn segments(&self, ctx: &PromptContext) -> Result<Vec<Segment>, PluginError>;

    /// Optional: Handle shell commands (for guards/hooks)
    fn on_command(&self, _cmd: &str) -> CommandAction {
        CommandAction::Allow
    }

    /// Called when plugin is unloaded
    async fn cleanup(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}
