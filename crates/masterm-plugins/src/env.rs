//! Environment detection plugin

use async_trait::async_trait;
use masterm_core::context::EnvironmentType;
use masterm_core::plugin::{
    Plugin, PluginContext, PluginError, PluginManifest, PluginMeta,
    PluginRequirements, PluginPermissions, PluginActivation, PluginPerformance,
    DetectionContext, CommandAction, ActivationTrigger,
};
use masterm_core::prompt::{Segment, SegmentStyle, Position, Color, theme::NamedColor};

/// Environment detection plugin
pub struct EnvPlugin {
    manifest: PluginManifest,
}

impl EnvPlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "env".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Environment type detection (dev/staging/prod)".to_string(),
                    author: "MASTerm Team".to_string(),
                    license: "MIT".to_string(),
                    homepage: Some("https://masterm.dev/plugins/env".to_string()),
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
        }
    }

    fn detect_env(&self, cwd: &std::path::Path) -> EnvironmentType {
        EnvironmentType::detect(cwd, &[
            "**/prod/**".to_string(),
            "**/production/**".to_string(),
        ])
    }
}

impl Default for EnvPlugin {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl Plugin for EnvPlugin {
    fn manifest(&self) -> &PluginManifest { &self.manifest }
    async fn init(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
    fn should_activate(&self, _ctx: &DetectionContext) -> bool { true }

    async fn segments(&self, ctx: &masterm_core::plugin::api::PromptContext) -> Result<Vec<Segment>, PluginError> {
        let env_type = self.detect_env(&ctx.cwd);

        match env_type {
            EnvironmentType::Production => {
                Ok(vec![Segment::new("env", "PROD")
                    .with_style(SegmentStyle {
                        fg: Some(Color::Named(NamedColor::White)),
                        bg: Some(Color::Named(NamedColor::Red)),
                        bold: true,
                        icon: Some("⚠".to_string()),
                        ..Default::default()
                    })
                    .with_priority(0)])
            }
            EnvironmentType::Staging => {
                Ok(vec![Segment::new("env", "STAGING")
                    .with_style(SegmentStyle {
                        fg: Some(Color::Named(NamedColor::Black)),
                        bg: Some(Color::Named(NamedColor::Yellow)),
                        bold: true,
                        ..Default::default()
                    })
                    .with_priority(1)])
            }
            _ => Ok(vec![]),
        }
    }

    fn on_command(&self, _cmd: &str) -> CommandAction { CommandAction::Allow }
    async fn cleanup(&mut self) -> Result<(), PluginError> { Ok(()) }
}
