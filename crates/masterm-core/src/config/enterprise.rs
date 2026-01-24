//! Enterprise configuration support

use serde::{Deserialize, Serialize};

/// Enterprise configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EnterpriseConfig {
    /// Enterprise settings
    pub enterprise: EnterpriseSettings,

    /// Lockdown settings
    pub lockdown: LockdownSettings,

    /// Plugin controls
    pub plugins: EnterprisePluginSettings,

    /// Audit settings
    pub audit: AuditSettings,

    /// Safety overrides
    pub safety: EnterpriseSafetySettings,
}

/// Enterprise identification and sync settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EnterpriseSettings {
    /// Enable enterprise mode
    pub enabled: bool,

    /// Organization identifier
    pub org_id: String,

    /// Central config server URL
    pub config_server: Option<String>,

    /// Config refresh interval in seconds
    pub refresh_interval: u64,
}

/// Settings lockdown
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LockdownSettings {
    /// Settings that cannot be overridden by users
    pub locked_settings: Vec<String>,

    /// Fully read-only mode
    pub read_only: bool,
}

/// Enterprise plugin controls
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EnterprisePluginSettings {
    /// Allowed plugins (empty = allow all non-denied)
    pub allowlist: Vec<String>,

    /// Blocked plugins
    pub denylist: Vec<String>,

    /// Allowed plugin registries
    pub allowed_registries: Vec<String>,
}

/// Audit logging settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AuditSettings {
    /// Enable audit logging
    pub enabled: bool,

    /// Audit destination: file, syslog, http
    pub destination: String,

    /// File path for file destination
    pub file_path: Option<String>,

    /// HTTP endpoint for http destination
    pub http_endpoint: Option<String>,

    /// Events to audit
    pub events: Vec<String>,
}

/// Enterprise safety overrides
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EnterpriseSafetySettings {
    /// Force prod patterns (merged with user)
    pub force_prod_patterns: Vec<String>,

    /// Force dangerous commands (merged with user)
    pub force_dangerous_commands: Vec<String>,

    /// Completely blocked commands (no confirmation, just block)
    pub blocked_commands: Vec<String>,
}
