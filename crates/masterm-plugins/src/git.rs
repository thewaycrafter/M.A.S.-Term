//! Git plugin - displays git repository context

use async_trait::async_trait;
use masterm_core::plugin::{
    Plugin, PluginContext, PluginError, PluginManifest, PluginMeta,
    PluginRequirements, PluginPermissions, PluginActivation, PluginPerformance,
    DetectionContext, CommandAction, ActivationTrigger,
};
use masterm_core::prompt::{Segment, SegmentStyle, Position, Color, theme::NamedColor};
use std::process::Command;

/// Git plugin for displaying repository information
pub struct GitPlugin {
    manifest: PluginManifest,
    initialized: bool,
}

impl GitPlugin {
    /// Create a new git plugin
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "git".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Git repository context and status".to_string(),
                    author: "MASTerm Team".to_string(),
                    license: "MIT".to_string(),
                    homepage: Some("https://masterm.dev/plugins/git".to_string()),
                },
                requirements: PluginRequirements {
                    binaries: vec!["git".to_string()],
                    masterm_version: Some(">=1.0.0".to_string()),
                    dependencies: vec![],
                },
                permissions: PluginPermissions {
                    filesystem: vec!["read".to_string()],
                    network: "none".to_string(),
                    environment: vec!["read".to_string()],
                    execute: vec!["git".to_string()],
                },
                activation: PluginActivation {
                    triggers: vec![
                        ActivationTrigger::DirectoryExists { pattern: ".git".to_string() },
                    ],
                    mode: "auto".to_string(),
                },
                performance: PluginPerformance {
                    startup_cost: "low".to_string(),
                    runtime_cost: "medium".to_string(),
                },
            },
            initialized: false,
        }
    }

    /// Get current branch
    fn get_branch(&self, cwd: &std::path::Path) -> Option<String> {
        Command::new("git")
            .args(["symbolic-ref", "--short", "HEAD"])
            .current_dir(cwd)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .or_else(|| {
                Command::new("git")
                    .args(["rev-parse", "--short", "HEAD"])
                    .current_dir(cwd)
                    .output()
                    .ok()
                    .filter(|o| o.status.success())
                    .map(|o| format!(":{}", String::from_utf8_lossy(&o.stdout).trim()))
            })
    }

    /// Get status counts
    fn get_status(&self, cwd: &std::path::Path) -> (u32, u32, u32) {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(cwd)
            .output()
            .ok();

        let mut staged = 0u32;
        let mut modified = 0u32;
        let mut untracked = 0u32;

        if let Some(output) = output {
            if output.status.success() {
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    if line.len() < 2 { continue; }
                    let first = line.chars().next().unwrap_or(' ');
                    let second = line.chars().nth(1).unwrap_or(' ');

                    if first != ' ' && first != '?' { staged += 1; }
                    if second == 'M' || second == 'D' { modified += 1; }
                    if first == '?' { untracked += 1; }
                }
            }
        }

        (staged, modified, untracked)
    }

    fn is_git_repo(&self, cwd: &std::path::Path) -> bool {
        Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(cwd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl Default for GitPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for GitPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    async fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        if !ctx.binary_exists("git") {
            return Err(PluginError::MissingBinary("git".into()));
        }
        self.initialized = true;
        Ok(())
    }

    fn should_activate(&self, ctx: &DetectionContext) -> bool {
        ctx.cwd.join(".git").exists() || self.is_git_repo(&ctx.cwd)
    }

    async fn segments(&self, ctx: &masterm_core::plugin::api::PromptContext) -> Result<Vec<Segment>, PluginError> {
        let mut segments = Vec::new();

        if let Some(branch) = self.get_branch(&ctx.cwd) {
            segments.push(
                Segment::new("git_branch", &branch)
                    .with_style(SegmentStyle {
                        fg: Some(Color::Named(NamedColor::Cyan)),
                        bg: None,
                        bold: true,
                        italic: false,
                        underline: false,
                        icon: Some("".to_string()),
                        icon_fallback: Some("⎇".to_string()),
                        prefix: None,
                        suffix: None,
                    })
                    .with_priority(50)
            );
        }

        let (staged, modified, untracked) = self.get_status(&ctx.cwd);
        let is_clean = staged == 0 && modified == 0 && untracked == 0;

        let status_str = if is_clean {
            "✓".to_string()
        } else {
            let mut parts = Vec::new();
            if staged > 0 { parts.push(format!("+{}", staged)); }
            if modified > 0 { parts.push(format!("~{}", modified)); }
            if untracked > 0 { parts.push(format!("?{}", untracked)); }
            parts.join(" ")
        };

        let status_color = if is_clean {
            Color::Named(NamedColor::Green)
        } else {
            Color::Named(NamedColor::Red)
        };

        segments.push(
            Segment::new("git_status", &status_str)
                .with_style(SegmentStyle {
                    fg: Some(status_color),
                    ..Default::default()
                })
                .with_priority(51)
        );

        Ok(segments)
    }

    fn on_command(&self, _cmd: &str) -> CommandAction {
        CommandAction::Allow
    }

    async fn cleanup(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}
