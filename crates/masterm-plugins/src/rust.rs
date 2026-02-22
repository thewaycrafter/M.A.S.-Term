//! Rust plugin

use async_trait::async_trait;
use masterm_core::plugin::{
    ActivationTrigger, CommandAction, DetectionContext, Plugin, PluginActivation, PluginContext,
    PluginError, PluginManifest, PluginMeta, PluginPerformance, PluginPermissions,
    PluginRequirements,
};
use masterm_core::prompt::{Color, NamedColor, Segment, SegmentStyle};
use std::process::Command;

pub struct RustPlugin {
    manifest: PluginManifest,
}

impl RustPlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "rust".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Rust version detection".to_string(),
                    author: "theWayCrafter".to_string(),
                    license: "MIT".to_string(),
                    homepage: None,
                },
                requirements: PluginRequirements {
                    binaries: vec!["rustc".to_string()],
                    ..Default::default()
                },
                permissions: PluginPermissions {
                    execute: vec!["rustc".to_string()],
                    ..Default::default()
                },
                activation: PluginActivation {
                    triggers: vec![ActivationTrigger::FileExists {
                        pattern: "Cargo.toml".to_string(),
                    }],
                    mode: "auto".to_string(),
                },
                performance: PluginPerformance::default(),
            },
        }
    }
}

impl Default for RustPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for RustPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
    async fn init(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        Ok(())
    }
    fn should_activate(&self, ctx: &DetectionContext) -> bool {
        ctx.cwd.join("Cargo.toml").exists()
    }

    async fn segments(
        &self,
        _ctx: &masterm_core::plugin::PromptContext,
    ) -> Result<Vec<Segment>, PluginError> {
        let version = Command::new("rustc")
            .arg("--version")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .and_then(|o| {
                let s = String::from_utf8_lossy(&o.stdout);
                s.split_whitespace().nth(1).map(|v| v.to_string())
            });

        if let Some(v) = version {
            Ok(vec![Segment::new("rust", format!("v{}", v))
                .with_style(SegmentStyle {
                    fg: Some(Color::Named(NamedColor::Red)),
                    icon: Some("".to_string()),
                    icon_fallback: Some("🦀".to_string()),
                    ..Default::default()
                })
                .with_priority(100)])
        } else {
            Ok(vec![])
        }
    }

    fn on_command(&self, _cmd: &str) -> CommandAction {
        CommandAction::Allow
    }
    async fn cleanup(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}
