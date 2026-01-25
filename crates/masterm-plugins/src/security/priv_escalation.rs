//! Privilege Escalation Alert Plugin
//!
//! Warns when commands use privilege escalation (sudo, su, doas, pkexec)
//! with environment-aware response levels.

use async_trait::async_trait;
use masterm_core::context::EnvironmentType;
use masterm_core::plugin::{
    ActivationTrigger, CommandAction, DetectionContext, Plugin, PluginActivation, PluginContext,
    PluginError, PluginManifest, PluginMeta, PluginPerformance, PluginPermissions,
    PluginRequirements, PromptContext,
};
use masterm_core::prompt::Segment;
use masterm_security::patterns::{PrivilegePatternMatcher, PrivilegeType};

/// Privilege Escalation Alert Plugin
pub struct PrivEscalationPlugin {
    manifest: PluginManifest,
    matcher: PrivilegePatternMatcher,
    /// Action in development environment
    dev_action: ActionLevel,
    /// Action in staging environment
    staging_action: ActionLevel,
    /// Action in production environment
    prod_action: ActionLevel,
}

/// Action level for privilege escalation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionLevel {
    /// Allow without any warning
    Allow,
    /// Show a warning but allow
    Warn,
    /// Require confirmation
    Confirm,
    /// Block entirely
    Block,
}

impl PrivEscalationPlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "priv-escalation".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Alert on privilege escalation commands".to_string(),
                    author: "MASTerm Security Team".to_string(),
                    license: "MIT".to_string(),
                    homepage: Some("https://masterm.dev/plugins/priv-escalation".to_string()),
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
            matcher: PrivilegePatternMatcher::new(),
            dev_action: ActionLevel::Allow,
            staging_action: ActionLevel::Warn,
            prod_action: ActionLevel::Confirm,
        }
    }

    /// Get current environment type
    fn detect_environment(&self) -> EnvironmentType {
        let cwd = std::env::current_dir().unwrap_or_default();
        EnvironmentType::detect(
            &cwd,
            &["**/prod/**".to_string(), "**/production/**".to_string()],
        )
    }

    /// Get action for current environment
    fn get_action(&self) -> ActionLevel {
        match self.detect_environment() {
            EnvironmentType::Production => self.prod_action,
            EnvironmentType::Staging => self.staging_action,
            EnvironmentType::Development | EnvironmentType::Unknown => self.dev_action,
        }
    }

    /// Format warning message
    fn format_warning(&self, priv_type: PrivilegeType, command: &str) -> String {
        let env = self.detect_environment();

        format!(
            "\x1b[1;33m{} PRIVILEGE ESCALATION DETECTED\x1b[0m\n\n\
             Command: \x1b[1m{}\x1b[0m\n\
             Type: {} ({})\n\
             Environment: {}\n\n\
             This command will execute with elevated privileges.",
            priv_type.icon(),
            command,
            priv_type.name(),
            priv_type.icon(),
            env.full_name()
        )
    }

    /// Format confirmation message
    fn format_confirmation(&self, priv_type: PrivilegeType, command: &str) -> String {
        let env = self.detect_environment();

        format!(
            "\x1b[1;33m🔒 PRIVILEGE ESCALATION DETECTED\x1b[0m\n\n\
             You are in a \x1b[1;31m{}\x1b[0m environment.\n\n\
             Command: \x1b[1m{}\x1b[0m\n\
             Type: {} ({})\n\n\
             This command will execute with elevated privileges.\n\n\
             Type '\x1b[1;32myes\x1b[0m' to confirm or '\x1b[1;31mno\x1b[0m' to cancel:",
            env.full_name(),
            command,
            priv_type.name(),
            priv_type.icon()
        )
    }

    /// Format block message
    fn format_block(&self, priv_type: PrivilegeType, command: &str) -> String {
        let env = self.detect_environment();

        format!(
            "\x1b[1;31m🚫 PRIVILEGE ESCALATION BLOCKED\x1b[0m\n\n\
             Privilege escalation is not allowed in {} environment.\n\n\
             Command: \x1b[1m{}\x1b[0m\n\
             Type: {} ({})\n\n\
             Contact your administrator if you need elevated access.",
            env.full_name(),
            command,
            priv_type.name(),
            priv_type.icon()
        )
    }
}

impl Default for PrivEscalationPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for PrivEscalationPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    async fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        // Load action levels from config
        if let Some(action) = ctx.get_config_string("dev_action") {
            self.dev_action = parse_action_level(&action);
        }
        if let Some(action) = ctx.get_config_string("staging_action") {
            self.staging_action = parse_action_level(&action);
        }
        if let Some(action) = ctx.get_config_string("prod_action") {
            self.prod_action = parse_action_level(&action);
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
        let priv_match = match self.matcher.check(cmd) {
            Some(m) => m,
            None => return CommandAction::Allow,
        };

        let action = self.get_action();

        match action {
            ActionLevel::Allow => CommandAction::Allow,
            ActionLevel::Warn => {
                CommandAction::Warn(self.format_warning(priv_match.priv_type, cmd))
            }
            ActionLevel::Confirm => {
                CommandAction::Confirm(self.format_confirmation(priv_match.priv_type, cmd))
            }
            ActionLevel::Block => {
                CommandAction::Block(self.format_block(priv_match.priv_type, cmd))
            }
        }
    }

    async fn cleanup(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// Parse action level from string
fn parse_action_level(s: &str) -> ActionLevel {
    match s.to_lowercase().as_str() {
        "allow" => ActionLevel::Allow,
        "warn" => ActionLevel::Warn,
        "confirm" => ActionLevel::Confirm,
        "block" => ActionLevel::Block,
        _ => ActionLevel::Warn,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sudo_detection() {
        let plugin = PrivEscalationPlugin::new();

        // Default dev action is Allow
        let action = plugin.on_command("sudo apt update");
        // In dev environment, should be allowed
        assert!(matches!(action, CommandAction::Allow));
    }

    #[test]
    fn test_action_parsing() {
        assert_eq!(parse_action_level("allow"), ActionLevel::Allow);
        assert_eq!(parse_action_level("warn"), ActionLevel::Warn);
        assert_eq!(parse_action_level("confirm"), ActionLevel::Confirm);
        assert_eq!(parse_action_level("block"), ActionLevel::Block);
        assert_eq!(parse_action_level("unknown"), ActionLevel::Warn);
    }
}
