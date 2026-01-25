//! Sandbox Mode Plugin
//!
//! Provides a restricted execution environment:
//! - Block privilege escalation
//! - Restrict filesystem access
//! - Optional network restrictions
//! - Time-limited sessions

use async_trait::async_trait;
use masterm_core::plugin::{
    ActivationTrigger, CommandAction, DetectionContext, Plugin, PluginActivation, PluginContext,
    PluginError, PluginManifest, PluginMeta, PluginPerformance, PluginPermissions,
    PluginRequirements, PromptContext,
};
use masterm_core::prompt::{Color, NamedColor, Segment, SegmentStyle};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Sandbox Mode Plugin
pub struct SandboxPlugin {
    manifest: PluginManifest,
    /// Is sandbox mode active
    active: Arc<AtomicBool>,
    /// Allowed directories (empty = all allowed)
    allowed_dirs: Vec<PathBuf>,
    /// Allow network access
    allow_network: bool,
    /// Blocked commands in sandbox
    blocked_commands: Vec<String>,
}

impl SandboxPlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "sandbox".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Restricted execution sandbox mode".to_string(),
                    author: "MASTerm Security Team".to_string(),
                    license: "MIT".to_string(),
                    homepage: Some("https://masterm.dev/plugins/sandbox".to_string()),
                },
                requirements: PluginRequirements::default(),
                permissions: PluginPermissions {
                    filesystem: vec!["read".to_string()],
                    network: "none".to_string(),
                    environment: vec!["read".to_string(), "write".to_string()],
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
            active: Arc::new(AtomicBool::new(false)),
            allowed_dirs: vec![],
            allow_network: false,
            blocked_commands: default_blocked_commands(),
        }
    }

    /// Enter sandbox mode
    pub fn enter(&self) {
        self.active.store(true, Ordering::SeqCst);
        std::env::set_var("MASTERM_SANDBOX", "1");
    }

    /// Exit sandbox mode
    pub fn exit(&self) {
        self.active.store(false, Ordering::SeqCst);
        std::env::remove_var("MASTERM_SANDBOX");
    }

    /// Is sandbox mode active?
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
            || std::env::var("MASTERM_SANDBOX").map(|v| v == "1").unwrap_or(false)
    }

    /// Check if command is blocked in sandbox
    fn is_command_blocked(&self, cmd: &str) -> Option<String> {
        let cmd_lower = cmd.to_lowercase();
        let first_word = cmd_lower.split_whitespace().next().unwrap_or("");

        for blocked in &self.blocked_commands {
            if first_word == blocked.to_lowercase() {
                return Some(blocked.clone());
            }
        }

        // Check for privilege escalation
        if cmd_lower.starts_with("sudo ") || cmd_lower.starts_with("su ") 
            || cmd_lower.starts_with("doas ") || cmd_lower.starts_with("pkexec ") 
        {
            return Some("privilege escalation".to_string());
        }

        None
    }

    /// Check if command accesses forbidden path
    fn accesses_forbidden_path(&self, cmd: &str) -> Option<String> {
        if self.allowed_dirs.is_empty() {
            return None; // No restrictions
        }

        let words: Vec<&str> = cmd.split_whitespace().collect();
        
        for word in &words[1..] {
            if word.starts_with('-') {
                continue;
            }

            // Check if this looks like a path
            if word.starts_with('/') || word.starts_with('~') || word.contains('/') {
                let path = if word.starts_with('~') {
                    dirs::home_dir()
                        .map(|h| h.join(&word[2..]))
                        .unwrap_or_else(|| PathBuf::from(word))
                } else {
                    PathBuf::from(word)
                };

                // Check if path is within allowed directories
                let is_allowed = self.allowed_dirs.iter().any(|allowed| {
                    path.starts_with(allowed)
                });

                if !is_allowed {
                    return Some(word.to_string());
                }
            }
        }

        None
    }

    /// Check for network commands when network is disabled
    fn uses_network(&self, cmd: &str) -> bool {
        if self.allow_network {
            return false;
        }

        let network_commands = [
            "curl", "wget", "ssh", "scp", "sftp", "rsync", "nc", "netcat", 
            "ncat", "telnet", "ftp", "ping", "traceroute", "dig", "nslookup",
        ];

        let first_word = cmd.split_whitespace().next().unwrap_or("").to_lowercase();
        network_commands.contains(&first_word.as_str())
    }
}

impl Default for SandboxPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for SandboxPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    async fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        if let Ok(dirs) = ctx.get_string_list("allowed_dirs") {
            self.allowed_dirs = dirs.into_iter().map(PathBuf::from).collect();
        }
        if let Some(network) = ctx.get_config_bool("allow_network") {
            self.allow_network = network;
        }
        if let Ok(blocked) = ctx.get_string_list("blocked_commands") {
            self.blocked_commands.extend(blocked);
        }

        // Check if sandbox should be active from environment
        if std::env::var("MASTERM_SANDBOX").map(|v| v == "1").unwrap_or(false) {
            self.active.store(true, Ordering::SeqCst);
        }

        Ok(())
    }

    fn should_activate(&self, _ctx: &DetectionContext) -> bool {
        self.is_active()
    }

    async fn segments(&self, _ctx: &PromptContext) -> Result<Vec<Segment>, PluginError> {
        if !self.is_active() {
            return Ok(vec![]);
        }

        // Show sandbox indicator in prompt
        Ok(vec![Segment::new("sandbox", "SANDBOX")
            .with_style(SegmentStyle {
                fg: Some(Color::Named(NamedColor::Black)),
                bg: Some(Color::Named(NamedColor::Cyan)),
                bold: true,
                icon: Some("🧪".to_string()),
                ..Default::default()
            })
            .with_priority(0)])
    }

    fn on_command(&self, cmd: &str) -> CommandAction {
        if !self.is_active() {
            return CommandAction::Allow;
        }

        // Check for blocked commands
        if let Some(blocked) = self.is_command_blocked(cmd) {
            return CommandAction::Block(format!(
                "\x1b[1;31m🧪 SANDBOX: COMMAND BLOCKED\x1b[0m\n\n\
                 Command blocked: \x1b[1m{}\x1b[0m\n\
                 Reason: '{}' is not allowed in sandbox mode.\n\n\
                 Use 'masterm sandbox exit' to leave sandbox mode.\n",
                cmd, blocked
            ));
        }

        // Check for forbidden paths
        if let Some(path) = self.accesses_forbidden_path(cmd) {
            return CommandAction::Block(format!(
                "\x1b[1;31m🧪 SANDBOX: PATH ACCESS DENIED\x1b[0m\n\n\
                 Path blocked: \x1b[1m{}\x1b[0m\n\n\
                 In sandbox mode, you can only access:\n{}\n\n\
                 Use 'masterm sandbox exit' to leave sandbox mode.\n",
                path,
                self.allowed_dirs.iter()
                    .map(|p| format!("  • {}", p.display()))
                    .collect::<Vec<_>>()
                    .join("\n")
            ));
        }

        // Check for network access
        if self.uses_network(cmd) {
            return CommandAction::Block(format!(
                "\x1b[1;31m🧪 SANDBOX: NETWORK ACCESS DENIED\x1b[0m\n\n\
                 Command: \x1b[1m{}\x1b[0m\n\n\
                 Network access is disabled in sandbox mode.\n\
                 Use 'masterm sandbox --allow-net' to enable network access.\n",
                cmd
            ));
        }

        CommandAction::Allow
    }

    async fn cleanup(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// Default commands blocked in sandbox mode
fn default_blocked_commands() -> Vec<String> {
    vec![
        "sudo".to_string(),
        "su".to_string(),
        "doas".to_string(),
        "pkexec".to_string(),
        "mount".to_string(),
        "umount".to_string(),
        "chroot".to_string(),
        "systemctl".to_string(),
        "service".to_string(),
        "reboot".to_string(),
        "shutdown".to_string(),
        "halt".to_string(),
        "poweroff".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_activation() {
        let plugin = SandboxPlugin::new();
        assert!(!plugin.is_active());

        plugin.enter();
        assert!(plugin.is_active());

        plugin.exit();
        assert!(!plugin.is_active());
    }

    #[test]
    fn test_command_blocking() {
        let plugin = SandboxPlugin::new();
        plugin.enter();

        let action = plugin.on_command("sudo apt update");
        assert!(matches!(action, CommandAction::Block(_)));

        let action = plugin.on_command("ls -la");
        assert!(matches!(action, CommandAction::Allow));

        plugin.exit();
    }

    #[test]
    fn test_network_blocking() {
        let plugin = SandboxPlugin::new();
        plugin.enter();

        let action = plugin.on_command("curl https://example.com");
        assert!(matches!(action, CommandAction::Block(_)));

        plugin.exit();
    }

    #[test]
    fn test_inactive_allows_all() {
        let plugin = SandboxPlugin::new();
        // Not in sandbox mode

        let action = plugin.on_command("sudo rm -rf /");
        // When not active, sandbox doesn't interfere
        // (other plugins would catch this)
        assert!(matches!(action, CommandAction::Allow));
    }
}
