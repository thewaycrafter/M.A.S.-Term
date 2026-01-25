//! SSH/GPG Key Monitoring Plugin
//!
//! Monitors SSH and GPG key operations:
//! - ssh-keygen (key generation)
//! - ssh-add (key loading)
//! - gpg --gen-key (key generation)
//! - gpg --export (key export)
//! - git push/pull with SSH

use async_trait::async_trait;
use masterm_core::plugin::{
    ActivationTrigger, CommandAction, DetectionContext, Plugin, PluginActivation, PluginContext,
    PluginError, PluginManifest, PluginMeta, PluginPerformance, PluginPermissions,
    PluginRequirements, PromptContext,
};
use masterm_core::prompt::Segment;

/// Key operation type
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyOperation {
    Generate,
    Load,
    Export,
    Import,
    Delete,
    Sign,
    Other,
}

impl KeyOperation {
    fn name(&self) -> &'static str {
        match self {
            Self::Generate => "Generate Key",
            Self::Load => "Load Key",
            Self::Export => "Export Key",
            Self::Import => "Import Key",
            Self::Delete => "Delete Key",
            Self::Sign => "Sign",
            Self::Other => "Key Operation",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::Generate => "🔑",
            Self::Load => "📥",
            Self::Export => "📤",
            Self::Import => "📥",
            Self::Delete => "🗑️",
            Self::Sign => "✍️",
            Self::Other => "🔐",
        }
    }
}

/// Key type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyType {
    Ssh,
    Gpg,
}

impl KeyType {
    fn name(&self) -> &'static str {
        match self {
            Self::Ssh => "SSH",
            Self::Gpg => "GPG",
        }
    }
}

/// Key operation info
#[derive(Debug, Clone)]
struct KeyOpInfo {
    operation: KeyOperation,
    key_type: KeyType,
    details: Option<String>,
}

/// SSH/GPG Monitor Plugin
pub struct SshGpgMonitorPlugin {
    manifest: PluginManifest,
    /// Alert on key generation
    alert_on_generate: bool,
    /// Alert on key export
    alert_on_export: bool,
    /// Alert on key deletion
    alert_on_delete: bool,
    /// Log key operations
    log_operations: bool,
}

impl SshGpgMonitorPlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "ssh-gpg-monitor".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Monitor SSH and GPG key operations".to_string(),
                    author: "MASTerm Security Team".to_string(),
                    license: "MIT".to_string(),
                    homepage: Some("https://masterm.dev/plugins/ssh-gpg-monitor".to_string()),
                },
                requirements: PluginRequirements::default(),
                permissions: PluginPermissions {
                    filesystem: vec![],
                    network: "none".to_string(),
                    environment: vec![],
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
            alert_on_generate: true,
            alert_on_export: true,
            alert_on_delete: true,
            log_operations: true,
        }
    }

    /// Detect key operations in command
    fn detect_operations(&self, cmd: &str) -> Vec<KeyOpInfo> {
        let mut ops = Vec::new();
        let cmd_lower = cmd.to_lowercase();

        // SSH operations
        if cmd_lower.starts_with("ssh-keygen") {
            ops.push(KeyOpInfo {
                operation: KeyOperation::Generate,
                key_type: KeyType::Ssh,
                details: self.extract_ssh_keygen_details(cmd),
            });
        }

        if cmd_lower.starts_with("ssh-add") {
            let operation = if cmd_lower.contains("-d") || cmd_lower.contains("-D") {
                KeyOperation::Delete
            } else {
                KeyOperation::Load
            };
            ops.push(KeyOpInfo {
                operation,
                key_type: KeyType::Ssh,
                details: None,
            });
        }

        // GPG operations
        if cmd_lower.contains("gpg") {
            if cmd_lower.contains("--gen-key") || cmd_lower.contains("--generate-key") 
                || cmd_lower.contains("--full-gen-key") 
            {
                ops.push(KeyOpInfo {
                    operation: KeyOperation::Generate,
                    key_type: KeyType::Gpg,
                    details: None,
                });
            }

            if cmd_lower.contains("--export") || cmd_lower.contains("-o") {
                ops.push(KeyOpInfo {
                    operation: KeyOperation::Export,
                    key_type: KeyType::Gpg,
                    details: if cmd_lower.contains("--armor") || cmd_lower.contains("-a") {
                        Some("ASCII armored".to_string())
                    } else {
                        None
                    },
                });
            }

            if cmd_lower.contains("--import") {
                ops.push(KeyOpInfo {
                    operation: KeyOperation::Import,
                    key_type: KeyType::Gpg,
                    details: None,
                });
            }

            if cmd_lower.contains("--delete-key") || cmd_lower.contains("--delete-secret-key") {
                ops.push(KeyOpInfo {
                    operation: KeyOperation::Delete,
                    key_type: KeyType::Gpg,
                    details: if cmd_lower.contains("--delete-secret-key") {
                        Some("SECRET KEY".to_string())
                    } else {
                        None
                    },
                });
            }

            if cmd_lower.contains("--sign") || cmd_lower.contains("-s") {
                ops.push(KeyOpInfo {
                    operation: KeyOperation::Sign,
                    key_type: KeyType::Gpg,
                    details: None,
                });
            }
        }

        ops
    }

    /// Extract details from ssh-keygen command
    fn extract_ssh_keygen_details(&self, cmd: &str) -> Option<String> {
        let mut details = Vec::new();

        // Key type
        if cmd.contains("-t rsa") {
            details.push("RSA");
        } else if cmd.contains("-t ed25519") {
            details.push("Ed25519");
        } else if cmd.contains("-t ecdsa") {
            details.push("ECDSA");
        } else if cmd.contains("-t dsa") {
            details.push("DSA (insecure!)");
        }

        // Key bits
        if cmd.contains("-b 4096") {
            details.push("4096 bits");
        } else if cmd.contains("-b 2048") {
            details.push("2048 bits");
        }

        if details.is_empty() {
            None
        } else {
            Some(details.join(", "))
        }
    }

    /// Should alert for this operation?
    fn should_alert(&self, op: &KeyOpInfo) -> bool {
        match op.operation {
            KeyOperation::Generate => self.alert_on_generate,
            KeyOperation::Export => self.alert_on_export,
            KeyOperation::Delete => self.alert_on_delete,
            _ => false,
        }
    }

    /// Format alert message
    fn format_alert(&self, ops: &[KeyOpInfo]) -> String {
        let mut msg = String::from("\x1b[1;33m🔐 CRYPTOGRAPHIC KEY OPERATION\x1b[0m\n\n");

        for op in ops {
            msg.push_str(&format!(
                "  {} {} {} Key",
                op.operation.icon(),
                op.operation.name(),
                op.key_type.name()
            ));

            if let Some(ref details) = op.details {
                msg.push_str(&format!(" ({})", details));
            }
            msg.push('\n');
        }

        if ops.iter().any(|o| o.operation == KeyOperation::Export) {
            msg.push_str("\n\x1b[33m⚠️  WARNING: Key export detected.\x1b[0m\n");
            msg.push_str("Ensure exported keys are stored securely and never shared.\n");
        }

        if ops.iter().any(|o| o.operation == KeyOperation::Delete) {
            msg.push_str("\n\x1b[31m⚠️  WARNING: Key deletion is permanent!\x1b[0m\n");
            msg.push_str("Ensure you have backups before deleting keys.\n");
        }

        msg.push_str("\nType '\x1b[1;32myes\x1b[0m' to proceed or '\x1b[1;31mno\x1b[0m' to cancel:");

        msg
    }
}

impl Default for SshGpgMonitorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for SshGpgMonitorPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    async fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        if let Some(gen) = ctx.get_config_bool("alert_on_generate") {
            self.alert_on_generate = gen;
        }
        if let Some(exp) = ctx.get_config_bool("alert_on_export") {
            self.alert_on_export = exp;
        }
        if let Some(del) = ctx.get_config_bool("alert_on_delete") {
            self.alert_on_delete = del;
        }
        Ok(())
    }

    fn should_activate(&self, _ctx: &DetectionContext) -> bool {
        true
    }

    async fn segments(&self, _ctx: &PromptContext) -> Result<Vec<Segment>, PluginError> {
        Ok(vec![])
    }

    fn on_command(&self, cmd: &str) -> CommandAction {
        let operations = self.detect_operations(cmd);

        // Filter to alertable operations
        let alertable: Vec<_> = operations.iter().filter(|o| self.should_alert(o)).cloned().collect();

        if alertable.is_empty() {
            // Log if enabled
            if self.log_operations && !operations.is_empty() {
                for op in &operations {
                    tracing::info!(
                        "Key operation: {} {} key",
                        op.operation.name(),
                        op.key_type.name()
                    );
                }
            }
            return CommandAction::Allow;
        }

        CommandAction::Confirm(self.format_alert(&alertable))
    }

    async fn cleanup(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_keygen_detection() {
        let plugin = SshGpgMonitorPlugin::new();

        let ops = plugin.detect_operations("ssh-keygen -t ed25519 -C 'test@example.com'");
        assert!(!ops.is_empty());
        assert_eq!(ops[0].operation, KeyOperation::Generate);
        assert_eq!(ops[0].key_type, KeyType::Ssh);
    }

    #[test]
    fn test_gpg_export_detection() {
        let plugin = SshGpgMonitorPlugin::new();

        let ops = plugin.detect_operations("gpg --export --armor user@example.com > key.asc");
        assert!(!ops.is_empty());
        assert_eq!(ops[0].operation, KeyOperation::Export);
        assert_eq!(ops[0].key_type, KeyType::Gpg);
    }

    #[test]
    fn test_gpg_delete_detection() {
        let plugin = SshGpgMonitorPlugin::new();

        let ops = plugin.detect_operations("gpg --delete-secret-key KEYID");
        assert!(!ops.is_empty());
        assert_eq!(ops[0].operation, KeyOperation::Delete);
        assert!(ops[0].details.as_ref().unwrap().contains("SECRET"));
    }

    #[test]
    fn test_ssh_add_detection() {
        let plugin = SshGpgMonitorPlugin::new();

        let ops = plugin.detect_operations("ssh-add ~/.ssh/id_ed25519");
        assert!(!ops.is_empty());
        assert_eq!(ops[0].operation, KeyOperation::Load);
    }

    #[test]
    fn test_normal_command() {
        let plugin = SshGpgMonitorPlugin::new();

        let ops = plugin.detect_operations("ls -la");
        assert!(ops.is_empty());
    }
}
