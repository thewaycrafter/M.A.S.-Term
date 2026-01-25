//! Secret Detection Plugin
//!
//! Detects hardcoded secrets, API keys, tokens, and passwords in commands
//! before they are executed.

use async_trait::async_trait;
use masterm_core::plugin::{
    ActivationTrigger, CommandAction, DetectionContext, Plugin, PluginActivation, PluginContext,
    PluginError, PluginManifest, PluginMeta, PluginPerformance, PluginPermissions,
    PluginRequirements, PromptContext,
};
use masterm_core::prompt::Segment;
use masterm_security::patterns::{SecretMatch, SecretPatternMatcher};

/// Secret Detection Plugin
pub struct SecretDetectionPlugin {
    manifest: PluginManifest,
    matcher: SecretPatternMatcher,
    action: SecretAction,
}

/// Action to take when a secret is detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretAction {
    /// Just warn the user
    Warn,
    /// Require confirmation
    Confirm,
    /// Block the command
    Block,
}

impl SecretDetectionPlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "secret-detection".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Detect hardcoded secrets in commands".to_string(),
                    author: "MASTerm Security Team".to_string(),
                    license: "MIT".to_string(),
                    homepage: Some("https://masterm.dev/plugins/secret-detection".to_string()),
                },
                requirements: PluginRequirements::default(),
                permissions: PluginPermissions {
                    filesystem: vec![],
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
            matcher: SecretPatternMatcher::new(),
            action: SecretAction::Confirm,
        }
    }

    /// Set the action to take when secrets are detected
    pub fn with_action(mut self, action: SecretAction) -> Self {
        self.action = action;
        self
    }

    /// Format the warning message for detected secrets
    fn format_warning(&self, secrets: &[SecretMatch]) -> String {
        let mut msg = String::from("\x1b[1;33m🔐 SECRET DETECTED IN COMMAND\x1b[0m\n\n");

        for secret in secrets {
            msg.push_str(&format!(
                "  {} \x1b[1m{}\x1b[0m: {}\n",
                secret.category.icon(),
                secret.category.name(),
                secret.matched_text
            ));
        }

        msg.push_str("\n\x1b[33m⚠️  Never expose secrets in command line!\x1b[0m\n");
        msg.push_str("Use environment variables or credential managers instead.\n");

        if let Some(first) = secrets.first() {
            msg.push_str(&format!("\n💡 Tip: {}\n", first.advice));
        }

        msg
    }

    /// Format the confirmation message
    fn format_confirmation(&self, secrets: &[SecretMatch]) -> String {
        let mut msg = self.format_warning(secrets);
        msg.push_str(
            "\nType '\x1b[1;32myes\x1b[0m' to continue or '\x1b[1;31mno\x1b[0m' to cancel:",
        );
        msg
    }

    /// Format the block message
    fn format_block(&self, secrets: &[SecretMatch]) -> String {
        let mut msg = String::from("\x1b[1;31m🚫 COMMAND BLOCKED - SECRET DETECTED\x1b[0m\n\n");

        for secret in secrets {
            msg.push_str(&format!(
                "  {} \x1b[1m{}\x1b[0m: {}\n",
                secret.category.icon(),
                secret.category.name(),
                secret.matched_text
            ));
        }

        msg.push_str(
            "\n\x1b[31mThis command was blocked because it contains sensitive data.\x1b[0m\n",
        );
        msg.push_str("Remove the secret and use environment variables instead.\n");

        msg
    }
}

impl Default for SecretDetectionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for SecretDetectionPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    async fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        // Load action from config
        if let Some(action) = ctx.get_config_string("action") {
            self.action = match action.as_str() {
                "warn" => SecretAction::Warn,
                "confirm" => SecretAction::Confirm,
                "block" => SecretAction::Block,
                _ => SecretAction::Confirm,
            };
        }
        Ok(())
    }

    fn should_activate(&self, _ctx: &DetectionContext) -> bool {
        true // Always active
    }

    async fn segments(&self, _ctx: &PromptContext) -> Result<Vec<Segment>, PluginError> {
        // No prompt segments
        Ok(vec![])
    }

    fn on_command(&self, cmd: &str) -> CommandAction {
        let secrets = self.matcher.find_all(cmd);

        if secrets.is_empty() {
            return CommandAction::Allow;
        }

        match self.action {
            SecretAction::Warn => CommandAction::Warn(self.format_warning(&secrets)),
            SecretAction::Confirm => CommandAction::Confirm(self.format_confirmation(&secrets)),
            SecretAction::Block => CommandAction::Block(self.format_block(&secrets)),
        }
    }

    async fn cleanup(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_detection() {
        let plugin = SecretDetectionPlugin::new();

        // AWS key should be detected
        let action = plugin.on_command("export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE");
        assert!(matches!(action, CommandAction::Confirm(_)));

        // GitHub token should be detected (ghp_ + 36 chars)
        let action = plugin.on_command(
            "git clone https://ghp_aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789@github.com/repo",
        );
        assert!(matches!(action, CommandAction::Confirm(_)));

        // Normal command should be allowed
        let action = plugin.on_command("ls -la");
        assert!(matches!(action, CommandAction::Allow));
    }

    #[test]
    fn test_action_modes() {
        let plugin = SecretDetectionPlugin::new().with_action(SecretAction::Block);
        // Use a dummy test key
        let action = plugin.on_command("export API_KEY=sk_dummy_000000000000000000000000");
        assert!(matches!(action, CommandAction::Block(_)));

        let plugin = SecretDetectionPlugin::new().with_action(SecretAction::Warn);
        let action = plugin.on_command("export API_KEY=sk_dummy_000000000000000000000000");
        assert!(matches!(action, CommandAction::Warn(_)));
    }
}
