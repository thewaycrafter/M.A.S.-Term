//! Go plugin

use async_trait::async_trait;
use masterm_core::plugin::{
    Plugin, PluginContext, PluginError, PluginManifest, PluginMeta,
    PluginRequirements, PluginPermissions, PluginActivation, PluginPerformance,
    DetectionContext, CommandAction, ActivationTrigger,
};
use masterm_core::prompt::{Segment, SegmentStyle, Color, NamedColor};
use std::process::Command;

pub struct GoPlugin { manifest: PluginManifest }

impl GoPlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "go".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Go version detection".to_string(),
                    author: "MASTerm Team".to_string(),
                    license: "MIT".to_string(),
                    homepage: None,
                },
                requirements: PluginRequirements { binaries: vec!["go".to_string()], ..Default::default() },
                permissions: PluginPermissions { execute: vec!["go".to_string()], ..Default::default() },
                activation: PluginActivation {
                    triggers: vec![ActivationTrigger::FileExists { pattern: "go.mod".to_string() }],
                    mode: "auto".to_string(),
                },
                performance: PluginPerformance::default(),
            },
        }
    }
}

impl Default for GoPlugin { fn default() -> Self { Self::new() } }

#[async_trait]
impl Plugin for GoPlugin {
    fn manifest(&self) -> &PluginManifest { &self.manifest }
    async fn init(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }
    fn should_activate(&self, ctx: &DetectionContext) -> bool { ctx.cwd.join("go.mod").exists() }

    async fn segments(&self, _ctx: &masterm_core::plugin::PromptContext) -> Result<Vec<Segment>, PluginError> {
        let version = Command::new("go").arg("version").output().ok()
            .filter(|o| o.status.success())
            .and_then(|o| {
                let s = String::from_utf8_lossy(&o.stdout);
                s.split_whitespace().nth(2).map(|v| v.trim_start_matches("go").to_string())
            });

        if let Some(v) = version {
            Ok(vec![Segment::new("go", format!("v{}", v))
                .with_style(SegmentStyle {
                    fg: Some(Color::Named(NamedColor::Cyan)),
                    icon: Some("".to_string()),
                    icon_fallback: Some("🐹".to_string()),
                    ..Default::default()
                })
                .with_priority(100)])
        } else { Ok(vec![]) }
    }

    fn on_command(&self, _cmd: &str) -> CommandAction { CommandAction::Allow }
    async fn cleanup(&mut self) -> Result<(), PluginError> { Ok(()) }
}
