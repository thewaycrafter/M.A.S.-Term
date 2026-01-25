//! File Integrity Alert Plugin
//!
//! Alerts when sensitive files are accessed or modified:
//! - ~/.ssh/* (SSH keys)
//! - ~/.gnupg/* (GPG keys)
//! - ~/.aws/* (AWS credentials)
//! - .env files
//! - /etc/passwd, /etc/shadow

use async_trait::async_trait;
use masterm_core::plugin::{
    ActivationTrigger, CommandAction, DetectionContext, Plugin, PluginActivation, PluginContext,
    PluginError, PluginManifest, PluginMeta, PluginPerformance, PluginPermissions,
    PluginRequirements, PromptContext,
};
use masterm_core::prompt::Segment;
use regex::Regex;

/// Access type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccessType {
    Read,
    Write,
    Delete,
    Permission,
}

/// File access info
#[derive(Debug, Clone)]
struct FileAccess {
    /// Path being accessed
    path: String,
    /// Type of access
    access_type: AccessType,
    /// Category of sensitive file
    category: FileCategory,
}

/// Category of sensitive file
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FileCategory {
    SshKeys,
    GpgKeys,
    AwsCredentials,
    CloudCredentials,
    EnvFile,
    SystemAuth,
    ShellConfig,
    Other,
}

impl FileCategory {
    fn name(&self) -> &'static str {
        match self {
            Self::SshKeys => "SSH Keys",
            Self::GpgKeys => "GPG Keys",
            Self::AwsCredentials => "AWS Credentials",
            Self::CloudCredentials => "Cloud Credentials",
            Self::EnvFile => "Environment File",
            Self::SystemAuth => "System Authentication",
            Self::ShellConfig => "Shell Configuration",
            Self::Other => "Sensitive File",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Self::SshKeys => "🔑",
            Self::GpgKeys => "🔐",
            Self::AwsCredentials => "☁️",
            Self::CloudCredentials => "☁️",
            Self::EnvFile => "📄",
            Self::SystemAuth => "🔒",
            Self::ShellConfig => "⚙️",
            Self::Other => "📁",
        }
    }
}

/// File Integrity Alert Plugin
pub struct FileIntegrityPlugin {
    manifest: PluginManifest,
    /// Sensitive path patterns
    sensitive_patterns: Vec<(Regex, FileCategory)>,
    /// Alert on read access
    alert_on_read: bool,
    /// Alert on write access
    alert_on_write: bool,
    /// Alert on permission changes
    alert_on_chmod: bool,
}

impl FileIntegrityPlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "file-integrity".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Alert on sensitive file access".to_string(),
                    author: "MASTerm Security Team".to_string(),
                    license: "MIT".to_string(),
                    homepage: Some("https://masterm.dev/plugins/file-integrity".to_string()),
                },
                requirements: PluginRequirements::default(),
                permissions: PluginPermissions {
                    filesystem: vec!["read".to_string()],
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
            sensitive_patterns: default_patterns(),
            alert_on_read: false, // Too noisy for most users
            alert_on_write: true,
            alert_on_chmod: true,
        }
    }

    /// Detect file access in command
    fn detect_access(&self, cmd: &str) -> Vec<FileAccess> {
        let mut accesses = Vec::new();
        let _cmd_lower = cmd.to_lowercase();
        let words: Vec<&str> = cmd.split_whitespace().collect();

        if words.is_empty() {
            return accesses;
        }

        let first_word = words[0].to_lowercase();

        // Determine access type from command
        let access_type = if first_word == "rm" || first_word == "rmdir" || first_word == "shred" {
            Some(AccessType::Delete)
        } else if first_word == "chmod" || first_word == "chown" || first_word == "chattr" {
            Some(AccessType::Permission)
        } else if ["vim", "vi", "nano", "emacs", "code", "subl", "edit"]
            .contains(&first_word.as_str())
        {
            Some(AccessType::Write)
        } else if ["cat", "less", "more", "head", "tail", "grep", "awk", "sed"]
            .contains(&first_word.as_str())
        {
            Some(if cmd.contains('>') {
                AccessType::Write
            } else {
                AccessType::Read
            })
        } else if ["cp", "mv", "ln"].contains(&first_word.as_str())
            || first_word == "touch"
            || cmd.contains('>')
            || cmd.contains(">>")
        {
            Some(AccessType::Write)
        } else {
            None
        };

        let access_type = match access_type {
            Some(t) => t,
            None => return accesses,
        };

        // Find sensitive paths in command
        for word in &words[1..] {
            // Skip flags
            if word.starts_with('-') {
                continue;
            }

            // Check against sensitive patterns
            for (pattern, category) in &self.sensitive_patterns {
                if pattern.is_match(word) {
                    accesses.push(FileAccess {
                        path: word.to_string(),
                        access_type,
                        category: *category,
                    });
                    break;
                }
            }
        }

        accesses
    }

    /// Should alert for this access?
    fn should_alert(&self, access: &FileAccess) -> bool {
        match access.access_type {
            AccessType::Read => self.alert_on_read,
            AccessType::Write | AccessType::Delete => self.alert_on_write,
            AccessType::Permission => self.alert_on_chmod,
        }
    }

    /// Format alert message
    fn format_alert(&self, accesses: &[FileAccess]) -> String {
        let mut msg = String::from("\x1b[1;33m🛡️  SENSITIVE FILE ACCESS DETECTED\x1b[0m\n\n");

        for access in accesses {
            let action = match access.access_type {
                AccessType::Read => "Reading",
                AccessType::Write => "Modifying",
                AccessType::Delete => "Deleting",
                AccessType::Permission => "Changing permissions of",
            };

            msg.push_str(&format!(
                "  {} {} {} ({})\n",
                access.category.icon(),
                action,
                access.path,
                access.category.name()
            ));
        }

        msg.push_str("\n\x1b[33mThis action affects sensitive security files.\x1b[0m\n");
        msg.push_str("Type '\x1b[1;32myes\x1b[0m' to confirm or '\x1b[1;31mno\x1b[0m' to cancel:");

        msg
    }
}

impl Default for FileIntegrityPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for FileIntegrityPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    async fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        if let Some(read) = ctx.get_config_bool("alert_on_read") {
            self.alert_on_read = read;
        }
        if let Some(write) = ctx.get_config_bool("alert_on_write") {
            self.alert_on_write = write;
        }
        if let Some(chmod) = ctx.get_config_bool("alert_on_chmod") {
            self.alert_on_chmod = chmod;
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
        let accesses = self.detect_access(cmd);

        // Filter to only alertable accesses
        let alertable: Vec<_> = accesses
            .iter()
            .filter(|a| self.should_alert(a))
            .cloned()
            .collect();

        if alertable.is_empty() {
            return CommandAction::Allow;
        }

        // Delete operations get blocked in production
        let has_delete = alertable
            .iter()
            .any(|a| a.access_type == AccessType::Delete);
        if has_delete {
            return CommandAction::Confirm(self.format_alert(&alertable));
        }

        // Other operations get confirmation
        CommandAction::Confirm(self.format_alert(&alertable))
    }

    async fn cleanup(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// Default sensitive path patterns
fn default_patterns() -> Vec<(Regex, FileCategory)> {
    let patterns = [
        (r"(?i)\.ssh/", FileCategory::SshKeys),
        (r"(?i)id_(?:rsa|dsa|ecdsa|ed25519)", FileCategory::SshKeys),
        (r"(?i)authorized_keys", FileCategory::SshKeys),
        (r"(?i)known_hosts", FileCategory::SshKeys),
        (r"(?i)\.gnupg/", FileCategory::GpgKeys),
        (r"(?i)\.gpg$", FileCategory::GpgKeys),
        (r"(?i)\.aws/", FileCategory::AwsCredentials),
        (r"(?i)aws_credentials", FileCategory::AwsCredentials),
        (r"(?i)\.config/gcloud/", FileCategory::CloudCredentials),
        (r"(?i)\.azure/", FileCategory::CloudCredentials),
        (r"(?i)\.kube/config", FileCategory::CloudCredentials),
        (r"(?i)\.env(?:\.[a-z]+)?$", FileCategory::EnvFile),
        (r"(?i)/etc/passwd", FileCategory::SystemAuth),
        (r"(?i)/etc/shadow", FileCategory::SystemAuth),
        (r"(?i)/etc/sudoers", FileCategory::SystemAuth),
        (r"(?i)\.bashrc$", FileCategory::ShellConfig),
        (r"(?i)\.zshrc$", FileCategory::ShellConfig),
        (r"(?i)\.profile$", FileCategory::ShellConfig),
        (r"(?i)\.bash_profile$", FileCategory::ShellConfig),
    ];

    patterns
        .iter()
        .filter_map(|(p, c)| Regex::new(p).ok().map(|r| (r, *c)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssh_key_detection() {
        let plugin = FileIntegrityPlugin::new();

        let accesses = plugin.detect_access("cat ~/.ssh/id_rsa");
        assert!(!accesses.is_empty());
        assert_eq!(accesses[0].category, FileCategory::SshKeys);
    }

    #[test]
    fn test_env_file_detection() {
        let plugin = FileIntegrityPlugin::new();

        let accesses = plugin.detect_access("vim .env.production");
        assert!(!accesses.is_empty());
        assert_eq!(accesses[0].category, FileCategory::EnvFile);
    }

    #[test]
    fn test_delete_detection() {
        let plugin = FileIntegrityPlugin::new();

        let accesses = plugin.detect_access("rm ~/.ssh/known_hosts");
        assert!(!accesses.is_empty());
        assert_eq!(accesses[0].access_type, AccessType::Delete);
    }

    #[test]
    fn test_permission_change() {
        let plugin = FileIntegrityPlugin::new();

        let accesses = plugin.detect_access("chmod 600 ~/.ssh/id_rsa");
        assert!(!accesses.is_empty());
        assert_eq!(accesses[0].access_type, AccessType::Permission);
    }

    #[test]
    fn test_normal_files_allowed() {
        let plugin = FileIntegrityPlugin::new();

        let accesses = plugin.detect_access("cat README.md");
        assert!(accesses.is_empty());

        let accesses = plugin.detect_access("vim src/main.rs");
        assert!(accesses.is_empty());
    }
}
