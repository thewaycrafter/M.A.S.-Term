//! Production guard plugin

use async_trait::async_trait;
use masterm_core::context::EnvironmentType;
use masterm_core::plugin::{
    Plugin, PluginContext, PluginError, PluginManifest, PluginMeta,
    PluginRequirements, PluginPermissions, PluginActivation, PluginPerformance,
    DetectionContext, CommandAction, ActivationTrigger,
};
use masterm_core::prompt::Segment;

/// Production guard plugin
pub struct ProdGuardPlugin {
    manifest: PluginManifest,
    dangerous_commands: Vec<String>,
    blocked_commands: Vec<String>,
}

impl ProdGuardPlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "prod-guard".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Safety guards for production environments".to_string(),
                    author: "MASTerm Team".to_string(),
                    license: "MIT".to_string(),
                    homepage: Some("https://masterm.dev/plugins/prod-guard".to_string()),
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
            dangerous_commands: vec![
                "rm -rf".to_string(),
                "rm -r ".to_string(),
                "DROP DATABASE".to_string(),
                "DROP TABLE".to_string(),
                "kubectl delete".to_string(),
                "terraform destroy".to_string(),
                "docker rm".to_string(),
                "git push --force".to_string(),
                "git push -f".to_string(),
            ],
            blocked_commands: vec![
                "rm -rf /".to_string(),
                "rm -rf /*".to_string(),
            ],
        }
    }

    fn is_production(&self, cwd: &std::path::Path) -> bool {
        let env_type = EnvironmentType::detect(cwd, &[
            "**/prod/**".to_string(),
            "**/production/**".to_string(),
        ]);
        env_type.is_production()
    }

    fn matches_any(&self, cmd: &str, patterns: &[String]) -> bool {
        let cmd_lower = cmd.to_lowercase();
        patterns.iter().any(|p| cmd_lower.contains(&p.to_lowercase()))
    }
}

impl Default for ProdGuardPlugin {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl Plugin for ProdGuardPlugin {
    fn manifest(&self) -> &PluginManifest { &self.manifest }

    async fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        if let Ok(patterns) = ctx.get_string_list("dangerous_commands") {
            self.dangerous_commands.extend(patterns);
        }
        if let Ok(patterns) = ctx.get_string_list("blocked_commands") {
            self.blocked_commands.extend(patterns);
        }
        Ok(())
    }

    fn should_activate(&self, ctx: &DetectionContext) -> bool {
        self.is_production(&ctx.cwd)
    }

    async fn segments(&self, _ctx: &masterm_core::plugin::api::PromptContext) -> Result<Vec<Segment>, PluginError> {
        Ok(vec![])
    }

    fn on_command(&self, cmd: &str) -> CommandAction {
        if self.matches_any(cmd, &self.blocked_commands) {
            return CommandAction::Block(format!(
                "\x1b[1;31m🚫 COMMAND BLOCKED\x1b[0m\n\n\
                 This command is not allowed in production environments:\n\n\
                 \x1b[1m  {}\x1b[0m\n\n\
                 This restriction is enforced by MASTerm safety guards.",
                cmd
            ));
        }

        if self.matches_any(cmd, &self.dangerous_commands) {
            return CommandAction::Confirm(format!(
                "\x1b[1;33m⚠️  PRODUCTION WARNING\x1b[0m\n\n\
                 You are in a \x1b[1;31mPRODUCTION\x1b[0m environment.\n\n\
                 You are about to run a potentially dangerous command:\n\n\
                 \x1b[1m  {}\x1b[0m\n\n\
                 Type '\x1b[1;32myes\x1b[0m' to confirm or '\x1b[1;31mno\x1b[0m' to cancel:",
                cmd
            ));
        }

        CommandAction::Allow
    }

    async fn cleanup(&mut self) -> Result<(), PluginError> { Ok(()) }
}
