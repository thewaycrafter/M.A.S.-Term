//! Security configuration schema
//!
//! Provides comprehensive configuration for all security features.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Master security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SecurityConfig {
    /// Master security toggle
    pub enabled: bool,

    /// Security level: minimal, standard, paranoid
    pub level: String,

    /// Secret detection settings
    pub secrets: SecretsConfig,

    /// Audit logging settings
    pub audit: AuditConfig,

    /// Privilege escalation settings
    pub privilege: PrivilegeConfig,

    /// Suspicious pattern settings
    pub patterns: PatternsConfig,

    /// Network monitoring settings
    pub network: NetworkConfig,

    /// Package audit settings
    pub packages: PackagesConfig,

    /// File integrity settings
    pub files: FilesConfig,

    /// IP/Domain reputation settings
    pub reputation: ReputationConfig,

    /// Sandbox settings
    pub sandbox: SandboxConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: "standard".to_string(),
            secrets: SecretsConfig::default(),
            audit: AuditConfig::default(),
            privilege: PrivilegeConfig::default(),
            patterns: PatternsConfig::default(),
            network: NetworkConfig::default(),
            packages: PackagesConfig::default(),
            files: FilesConfig::default(),
            reputation: ReputationConfig::default(),
            sandbox: SandboxConfig::default(),
        }
    }
}

/// Secret detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SecretsConfig {
    /// Enable secret detection
    pub enabled: bool,

    /// Action when secret detected: warn, confirm, block
    pub action: String,

    /// Enable redaction in output
    pub redact_output: bool,

    /// Custom secret patterns (regex)
    pub custom_patterns: Vec<String>,

    /// Patterns to ignore
    pub ignore_patterns: Vec<String>,
}

impl Default for SecretsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            action: "confirm".to_string(),
            redact_output: true,
            custom_patterns: vec![],
            ignore_patterns: vec![],
        }
    }
}

/// Audit logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled: bool,

    /// Log file path
    pub log_path: Option<PathBuf>,

    /// Maximum log size in MB before rotation
    pub max_size_mb: u64,

    /// Log rotation policy: daily, weekly, size
    pub rotation: String,

    /// Redact detected secrets in logs
    pub redact_secrets: bool,

    /// Include environment variables in logs
    pub include_env: bool,

    /// Enable hash chain verification
    pub hash_chain: bool,

    /// Commands to exclude from logging
    pub exclude_commands: Vec<String>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_path: None, // Will default to ~/.masterm/security/audit.log
            max_size_mb: 100,
            rotation: "daily".to_string(),
            redact_secrets: true,
            include_env: false,
            hash_chain: true,
            exclude_commands: vec![
                "ls".to_string(),
                "cd".to_string(),
                "pwd".to_string(),
                "echo".to_string(),
                "clear".to_string(),
            ],
        }
    }
}

/// Privilege escalation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PrivilegeConfig {
    /// Enable privilege escalation detection
    pub enabled: bool,

    /// Action in dev environment: allow, warn, confirm
    pub dev_action: String,

    /// Action in staging environment
    pub staging_action: String,

    /// Action in production environment
    pub prod_action: String,

    /// Additional privilege commands to monitor
    pub additional_commands: Vec<String>,
}

impl Default for PrivilegeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            dev_action: "allow".to_string(),
            staging_action: "warn".to_string(),
            prod_action: "confirm".to_string(),
            additional_commands: vec![],
        }
    }
}

/// Suspicious pattern configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PatternsConfig {
    /// Enable suspicious pattern detection
    pub enabled: bool,

    /// Block reverse shell attempts
    pub block_reverse_shells: bool,

    /// Block encoded command execution
    pub block_encoded_commands: bool,

    /// Block curl/wget pipe to shell
    pub block_download_execute: bool,

    /// Block history evasion
    pub detect_history_evasion: bool,

    /// Custom threat patterns (regex)
    pub custom_patterns: Vec<String>,

    /// Allow bypass for security testing
    pub allow_bypass: bool,
}

impl Default for PatternsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            block_reverse_shells: true,
            block_encoded_commands: true,
            block_download_execute: false, // Too common in legitimate use
            detect_history_evasion: true,
            custom_patterns: vec![],
            allow_bypass: true,
        }
    }
}

/// Network monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    /// Enable network monitoring
    pub enabled: bool,

    /// Log all outbound connections
    pub log_connections: bool,

    /// Warn on non-standard ports
    pub warn_nonstandard_ports: bool,

    /// Blocked ports
    pub blocked_ports: Vec<u16>,

    /// Allowed domains (whitelist mode if non-empty)
    pub allowed_domains: Vec<String>,

    /// Blocked domains
    pub blocked_domains: Vec<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_connections: true,
            warn_nonstandard_ports: true,
            blocked_ports: vec![],
            allowed_domains: vec![],
            blocked_domains: vec![],
        }
    }
}

/// Package manager audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PackagesConfig {
    /// Enable package audit
    pub enabled: bool,

    /// Warn on unscoped npm packages
    pub warn_unscoped_npm: bool,

    /// Warn on pip packages without version pin
    pub warn_unpinned_pip: bool,

    /// Known malicious packages to block
    pub blocklist: Vec<String>,

    /// Typosquatting detection for popular packages
    pub typosquatting_detection: bool,
}

impl Default for PackagesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            warn_unscoped_npm: true,
            warn_unpinned_pip: false,
            blocklist: vec![
                // Known malicious npm packages
                "event-stream".to_string(),
                "flatmap-stream".to_string(),
                "ua-parser-js".to_string(),
            ],
            typosquatting_detection: true,
        }
    }
}

/// File integrity configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FilesConfig {
    /// Enable file integrity monitoring
    pub enabled: bool,

    /// Monitored sensitive paths
    pub sensitive_paths: Vec<String>,

    /// Alert on read access
    pub alert_on_read: bool,

    /// Alert on write access
    pub alert_on_write: bool,

    /// Alert on permission changes
    pub alert_on_chmod: bool,
}

impl Default for FilesConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sensitive_paths: vec![
                "~/.ssh".to_string(),
                "~/.gnupg".to_string(),
                "~/.aws".to_string(),
                "~/.config/gcloud".to_string(),
                ".env".to_string(),
                ".env.*".to_string(),
            ],
            alert_on_read: false,
            alert_on_write: true,
            alert_on_chmod: true,
        }
    }
}

/// IP/Domain reputation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ReputationConfig {
    /// Enable reputation checking
    pub enabled: bool,

    /// Use offline mode (local blocklist only)
    pub offline_mode: bool,

    /// Local blocklist path
    pub blocklist_path: Option<PathBuf>,

    /// Cache TTL in seconds
    pub cache_ttl: u64,

    /// AbuseIPDB API key (optional)
    pub abuseipdb_key: Option<String>,

    /// VirusTotal API key (optional)
    pub virustotal_key: Option<String>,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default, requires setup
            offline_mode: true,
            blocklist_path: None,
            cache_ttl: 86400, // 24 hours
            abuseipdb_key: None,
            virustotal_key: None,
        }
    }
}

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SandboxConfig {
    /// Enable sandbox mode
    pub enabled: bool,

    /// Allowed directories in sandbox
    pub allowed_dirs: Vec<PathBuf>,

    /// Allow network access in sandbox
    pub allow_network: bool,

    /// Maximum session duration in seconds
    pub max_duration: u64,

    /// Blocked commands in sandbox
    pub blocked_commands: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_dirs: vec![],
            allow_network: false,
            max_duration: 3600, // 1 hour
            blocked_commands: vec![
                "sudo".to_string(),
                "su".to_string(),
                "doas".to_string(),
                "pkexec".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SecurityConfig::default();
        assert!(config.enabled);
        assert_eq!(config.level, "standard");
        assert!(config.secrets.enabled);
        assert!(config.audit.enabled);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = SecurityConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: SecurityConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.level, config.level);
    }
}
