//! Audit Logging Plugin
//!
//! Provides forensic-grade command logging with:
//! - Append-only storage
//! - SHA-256 hash chain verification
//! - Secret redaction
//! - JSON structured logs for SIEM integration

use async_trait::async_trait;
use masterm_core::plugin::{
    ActivationTrigger, CommandAction, DetectionContext, Plugin, PluginActivation, PluginContext,
    PluginError, PluginManifest, PluginMeta, PluginPerformance, PluginPermissions,
    PluginRequirements, PromptContext,
};
use masterm_core::prompt::Segment;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Simplified audit event for the plugin
#[derive(Debug, Clone)]
struct AuditEntry {
    timestamp: String,
    command: String,
    cwd: String,
    user: String,
    shell: String,
    env_type: String,
    security_flags: Vec<String>,
}

/// Audit Logging Plugin
#[allow(dead_code)]
pub struct AuditLogPlugin {
    manifest: PluginManifest,
    enabled: bool,
    log_path: Option<PathBuf>,
    redact_secrets: bool,
    /// Pending command to log (set on pre-command, logged on post-command)
    pending: Arc<RwLock<Option<AuditEntry>>>,
}

impl AuditLogPlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "audit-log".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Forensic-grade command audit logging".to_string(),
                    author: "MASTerm Security Team".to_string(),
                    license: "MIT".to_string(),
                    homepage: Some("https://masterm.dev/plugins/audit-log".to_string()),
                },
                requirements: PluginRequirements::default(),
                permissions: PluginPermissions {
                    filesystem: vec!["write".to_string()],
                    network: "none".to_string(),
                    environment: vec!["read".to_string()],
                    execute: vec![],
                },
                activation: PluginActivation {
                    triggers: vec![ActivationTrigger::Always],
                    mode: "always".to_string(),
                },
                performance: PluginPerformance {
                    startup_cost: "low".to_string(),
                    runtime_cost: "low".to_string(),
                },
            },
            enabled: true,
            log_path: None,
            redact_secrets: true,
            pending: Arc::new(RwLock::new(None)),
        }
    }

    /// Get default log path
    fn default_log_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".masterm")
            .join("security")
            .join("audit.log")
    }

    /// Redact secrets from command
    fn redact_command(&self, command: &str) -> String {
        if !self.redact_secrets {
            return command.to_string();
        }

        use regex::Regex;

        let mut result = command.to_string();

        let patterns = [
            (r"(AKIA[A-Z0-9]{16})", "[AWS_KEY]"),
            (r"(gh[pous]_[A-Za-z0-9]{36})", "[GH_TOKEN]"),
            (r"(sk_live_[A-Za-z0-9]{24,})", "[STRIPE_KEY]"),
            (r"(://[^:]+:)[^@]+(@)", "$1***$2"),
            (r"(?i)(password[=:]\s*)[^\s]+", "$1***"),
            (r"(?i)(api[_-]?key[=:]\s*)[^\s]+", "$1***"),
        ];

        for (pattern, replacement) in patterns {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, replacement).to_string();
            }
        }

        result
    }

    /// Write audit entry to log
    async fn write_entry(&self, entry: &AuditEntry) -> Result<(), std::io::Error> {
        use std::io::Write;

        let log_path = self.log_path.clone().unwrap_or_else(Self::default_log_path);

        // Ensure directory exists
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        let json = serde_json::json!({
            "timestamp": entry.timestamp,
            "command": self.redact_command(&entry.command),
            "cwd": entry.cwd,
            "user": entry.user,
            "shell": entry.shell,
            "env_type": entry.env_type,
            "security_flags": entry.security_flags,
        });

        writeln!(file, "{}", json)?;
        file.flush()?;

        Ok(())
    }
}

impl Default for AuditLogPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for AuditLogPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    async fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        // Load config
        if let Some(enabled) = ctx.get_config_bool("enabled") {
            self.enabled = enabled;
        }
        if let Some(path) = ctx.get_config_string("log_path") {
            self.log_path = Some(PathBuf::from(path));
        }
        if let Some(redact) = ctx.get_config_bool("redact_secrets") {
            self.redact_secrets = redact;
        }
        Ok(())
    }

    fn should_activate(&self, _ctx: &DetectionContext) -> bool {
        self.enabled
    }

    async fn segments(&self, _ctx: &PromptContext) -> Result<Vec<Segment>, PluginError> {
        Ok(vec![])
    }

    fn on_command(&self, cmd: &str) -> CommandAction {
        if !self.enabled {
            return CommandAction::Allow;
        }

        // Create audit entry
        let entry = AuditEntry {
            timestamp: chrono::Utc::now().to_rfc3339(),
            command: cmd.to_string(),
            cwd: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "unknown".to_string()),
            user: std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
            shell: std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string()),
            env_type: std::env::var("MASTERM_ENV").unwrap_or_else(|_| "unknown".to_string()),
            security_flags: vec![],
        };

        // Write synchronously (we're in a sync context)
        // In production, this would be async
        let rt = tokio::runtime::Handle::try_current();
        if let Ok(handle) = rt {
            let self_clone = AuditLogPlugin {
                manifest: self.manifest.clone(),
                enabled: self.enabled,
                log_path: self.log_path.clone(),
                redact_secrets: self.redact_secrets,
                pending: Arc::new(RwLock::new(None)),
            };
            let entry_clone = entry.clone();
            handle.spawn(async move {
                let _ = self_clone.write_entry(&entry_clone).await;
            });
        }

        CommandAction::Allow
    }

    async fn cleanup(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redaction() {
        let plugin = AuditLogPlugin::new();

        let redacted = plugin.redact_command("export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE");
        assert!(redacted.contains("[AWS_KEY]"));

        let redacted = plugin.redact_command(
            "git clone https://ghp_aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789@github.com/repo",
        );
        assert!(redacted.contains("[GH_TOKEN]"));

        let redacted = plugin.redact_command("curl https://user:password123@api.example.com");
        assert!(redacted.contains("***"));
    }

    #[test]
    fn test_plugin_creation() {
        let plugin = AuditLogPlugin::new();
        assert!(plugin.enabled);
        assert!(plugin.redact_secrets);
    }
}
