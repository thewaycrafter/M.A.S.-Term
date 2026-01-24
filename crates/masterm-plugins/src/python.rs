//! Python plugin

use async_trait::async_trait;
use masterm_core::plugin::{
    Plugin, PluginContext, PluginError, PluginManifest, PluginMeta,
    PluginRequirements, PluginPermissions, PluginActivation, PluginPerformance,
    DetectionContext, CommandAction, ActivationTrigger,
};
use masterm_core::prompt::{Segment, SegmentStyle, Color, theme::NamedColor};
use std::process::Command;

pub struct PythonPlugin {
    manifest: PluginManifest,
}

impl PythonPlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "python".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Python version and virtualenv detection".to_string(),
                    author: "MASTerm Team".to_string(),
                    license: "MIT".to_string(),
                    homepage: None,
                },
                requirements: PluginRequirements::default(),
                permissions: PluginPermissions {
                    filesystem: vec!["read".to_string()],
                    environment: vec!["read".to_string()],
                    execute: vec!["python".to_string(), "python3".to_string()],
                    ..Default::default()
                },
                activation: PluginActivation {
                    triggers: vec![
                        ActivationTrigger::FileExists { pattern: "pyproject.toml".to_string() },
                        ActivationTrigger::FileExists { pattern: "requirements.txt".to_string() },
                    ],
                    mode: "auto".to_string(),
                },
                performance: PluginPerformance::default(),
            },
        }
    }

    fn get_version(&self) -> Option<String> {
        Command::new("python3")
            .arg("--version")
            .output()
            .or_else(|_| Command::new("python").arg("--version").output())
            .ok()
            .filter(|o| o.status.success())
            .map(|o| {
                let out = String::from_utf8_lossy(&o.stdout);
                out.split_whitespace().nth(1).unwrap_or("").to_string()
            })
    }

    fn get_venv(&self) -> Option<String> {
        std::env::var("VIRTUAL_ENV").ok().and_then(|p| {
            std::path::Path::new(&p).file_name().map(|n| n.to_string_lossy().to_string())
        })
    }
}

impl Default for PythonPlugin { fn default() -> Self { Self::new() } }

#[async_trait]
impl Plugin for PythonPlugin {
    fn manifest(&self) -> &PluginManifest { &self.manifest }
    async fn init(&mut self, _ctx: &PluginContext) -> Result<(), PluginError> { Ok(()) }

    fn should_activate(&self, ctx: &DetectionContext) -> bool {
        ctx.cwd.join("pyproject.toml").exists() ||
        ctx.cwd.join("requirements.txt").exists() ||
        ctx.cwd.join("setup.py").exists() ||
        std::env::var("VIRTUAL_ENV").is_ok()
    }

    async fn segments(&self, _ctx: &masterm_core::plugin::api::PromptContext) -> Result<Vec<Segment>, PluginError> {
        let mut segs = Vec::new();

        if let Some(venv) = self.get_venv() {
            segs.push(Segment::new("python_venv", format!("({})", venv))
                .with_style(SegmentStyle {
                    fg: Some(Color::Named(NamedColor::Yellow)),
                    icon: Some("".to_string()),
                    icon_fallback: Some("🐍".to_string()),
                    ..Default::default()
                })
                .with_priority(95));
        } else if let Some(version) = self.get_version() {
            segs.push(Segment::new("python", format!("v{}", version))
                .with_style(SegmentStyle {
                    fg: Some(Color::Named(NamedColor::Yellow)),
                    icon: Some("".to_string()),
                    ..Default::default()
                })
                .with_priority(100));
        }

        Ok(segs)
    }

    fn on_command(&self, _cmd: &str) -> CommandAction { CommandAction::Allow }
    async fn cleanup(&mut self) -> Result<(), PluginError> { Ok(()) }
}
