//! Node.js plugin

use async_trait::async_trait;
use masterm_core::plugin::{
    ActivationTrigger, CommandAction, DetectionContext, Plugin, PluginActivation, PluginContext,
    PluginError, PluginManifest, PluginMeta, PluginPerformance, PluginPermissions,
    PluginRequirements,
};
use masterm_core::prompt::{Color, NamedColor, Segment, SegmentStyle};
use std::process::Command;

pub struct NodePlugin {
    manifest: PluginManifest,
}

impl NodePlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "node".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Node.js version and package detection".to_string(),
                    author: "MASTerm Team".to_string(),
                    license: "MIT".to_string(),
                    homepage: None,
                },
                requirements: PluginRequirements {
                    binaries: vec!["node".to_string()],
                    ..Default::default()
                },
                permissions: PluginPermissions {
                    filesystem: vec!["read".to_string()],
                    execute: vec!["node".to_string()],
                    ..Default::default()
                },
                activation: PluginActivation {
                    triggers: vec![ActivationTrigger::FileExists {
                        pattern: "package.json".to_string(),
                    }],
                    mode: "auto".to_string(),
                },
                performance: PluginPerformance {
                    startup_cost: "low".to_string(),
                    runtime_cost: "low".to_string(),
                },
            },
        }
    }

    fn get_version(&self) -> Option<String> {
        Command::new("node")
            .arg("--version")
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .trim_start_matches('v')
                    .to_string()
            })
    }
}

impl Default for NodePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for NodePlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
    async fn init(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> {
        Ok(())
    }
    fn should_activate(&self, ctx: &DetectionContext) -> bool {
        ctx.cwd.join("package.json").exists()
    }

    async fn segments(
        &self,
        _ctx: &masterm_core::plugin::PromptContext,
    ) -> Result<Vec<Segment>, PluginError> {
        if let Some(version) = self.get_version() {
            Ok(vec![Segment::new("node", format!("v{}", version))
                .with_style(SegmentStyle {
                    fg: Some(Color::Named(NamedColor::Green)),
                    icon: Some("".to_string()),
                    icon_fallback: Some("⬢".to_string()),
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
