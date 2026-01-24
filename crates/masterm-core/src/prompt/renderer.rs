//! Prompt renderer

use super::{Position, Segment, SegmentStyle, Theme, Color};
use crate::config::{IconMode, PromptConfig, ColorCapability};
use crate::context::Context;
use std::time::Duration;

/// The generated prompt
#[derive(Debug, Clone)]
pub struct Prompt {
    /// Left side of prompt
    pub left: String,

    /// Right side of prompt
    pub right: String,

    /// Continuation prompt (for multi-line input)
    pub continuation: String,

    /// Transient prompt (shown after command execution)
    pub transient: Option<String>,
}

/// Prompt renderer configuration
pub struct PromptRenderer {
    /// Prompt configuration
    config: PromptConfig,

    /// Color theme
    theme: Theme,

    /// Icon mode
    icon_mode: IconMode,

    /// Terminal color capability
    color_capability: ColorCapability,

    /// Terminal width
    terminal_width: u16,
}

impl PromptRenderer {
    /// Create a new prompt renderer
    pub fn new(config: PromptConfig) -> Self {
        let theme = Theme::by_name(&config.colors.theme);
        let icon_mode = Self::detect_icon_mode(&config.icons.mode);
        let color_capability = ColorCapability::detect();
        let terminal_width = crossterm::terminal::size().map(|(w, _)| w).unwrap_or(80);

        Self {
            config,
            theme,
            icon_mode,
            color_capability,
            terminal_width,
        }
    }

    /// Render the prompt from context and segments
    pub fn render(&self, context: &Context, segments: Vec<Segment>, exit_code: i32, duration: Duration) -> Prompt {
        let mut all_segments = segments;

        // Add built-in segments
        all_segments.extend(self.builtin_segments(context, exit_code, duration));

        // Sort by priority
        all_segments.sort_by_key(|s| s.priority);

        // Split by position
        let (left_segments, right_segments): (Vec<_>, Vec<_>) = all_segments
            .into_iter()
            .partition(|s| s.position == Position::Left);

        // Render each side
        let left = self.render_segments(&left_segments);
        let right = self.render_segments(&right_segments);

        // Build continuation prompt
        let continuation = self.render_continuation();

        // Build transient prompt if enabled
        let transient = if self.config.transient {
            Some(self.render_transient())
        } else {
            None
        };

        Prompt {
            left,
            right,
            continuation,
            transient,
        }
    }

    /// Generate built-in segments
    fn builtin_segments(&self, context: &Context, exit_code: i32, duration: Duration) -> Vec<Segment> {
        let mut segments = Vec::new();

        // Directory segment
        let dir = self.format_directory(&context.cwd);
        segments.push(
            Segment::new("directory", dir)
                .with_style(SegmentStyle::new().fg(self.theme.directory.clone()).icon(""))
                .with_priority(10),
        );

        // Exit code segment (if non-zero)
        if exit_code != 0 {
            segments.push(
                Segment::new("exit_code", format!("✗ {}", exit_code))
                    .with_style(SegmentStyle::new().fg(self.theme.error.clone()))
                    .with_priority(200)
                    .with_position(Position::Right),
            );
        }

        // Duration segment (if > 2 seconds)
        if duration.as_secs() >= 2 {
            let duration_str = self.format_duration(duration);
            segments.push(
                Segment::new("duration", duration_str)
                    .with_style(SegmentStyle::new().fg(self.theme.duration.clone()).icon(""))
                    .with_priority(190)
                    .with_position(Position::Right),
            );
        }

        // Environment segment (if prod/staging)
        if context.environment.is_sensitive() {
            let (content, color) = if context.environment.is_production() {
                ("PROD", self.theme.prod.clone())
            } else {
                ("STAGING", self.theme.staging.clone())
            };

            segments.push(
                Segment::new("environment", content)
                    .with_style(SegmentStyle::new()
                        .fg(Color::Named(super::theme::NamedColor::White))
                        .bg(color)
                        .bold())
                    .with_priority(0), // Highest priority
            );
        }

        // Git segments (if in git repo)
        if let Some(ref git) = context.git {
            // Branch
            let branch_icon = if git.detached { "" } else { "" };
            segments.push(
                Segment::new("git_branch", &git.branch)
                    .with_style(SegmentStyle::new()
                        .fg(self.theme.git_branch.clone())
                        .icon(branch_icon))
                    .with_priority(50),
            );

            // Status
            let status_str = git.format_status();
            let status_color = if git.is_clean {
                self.theme.git_clean.clone()
            } else {
                self.theme.git_dirty.clone()
            };
            segments.push(
                Segment::new("git_status", &status_str)
                    .with_style(SegmentStyle::new().fg(status_color))
                    .with_priority(51),
            );

            // Ahead/behind
            if let Some(ab) = git.format_ahead_behind() {
                segments.push(
                    Segment::new("git_ahead_behind", &ab)
                        .with_style(SegmentStyle::new().fg(self.theme.git_branch.clone()))
                        .with_priority(52),
                );
            }
        }

        // Language segments
        for lang in &context.languages {
            let version = lang.version.as_deref().unwrap_or("");
            let content = if version.is_empty() {
                lang.name.clone()
            } else {
                format!("{} v{}", lang.name, version)
            };

            segments.push(
                Segment::new(format!("lang_{}", lang.name.to_lowercase()), content)
                    .with_style(SegmentStyle::new().icon(lang.icon))
                    .with_priority(100),
            );
        }

        segments
    }

    /// Render a list of segments
    fn render_segments(&self, segments: &[Segment]) -> String {
        let mut result = String::new();

        for (i, segment) in segments.iter().enumerate() {
            // Add separator between segments
            if i > 0 {
                result.push(' ');
            }

            // Render segment
            result.push_str(&self.render_segment(segment));
        }

        // Reset at end
        result.push_str("\x1b[0m");

        if self.config.add_newline {
            result.push_str("\n❯ ");
        } else {
            result.push_str(" ❯ ");
        }

        result
    }

    /// Render a single segment
    fn render_segment(&self, segment: &Segment) -> String {
        let mut result = String::new();

        // Apply style
        if let Some(ref bg) = segment.style.bg {
            result.push_str(&bg.to_ansi_bg());
        }
        if let Some(ref fg) = segment.style.fg {
            result.push_str(&fg.to_ansi_fg());
        }
        if segment.style.bold {
            result.push_str("\x1b[1m");
        }

        // Add icon
        if let Some(ref icon) = segment.style.icon {
            let display_icon = match self.icon_mode {
                IconMode::Nerd => icon.clone(),
                IconMode::Unicode => segment.style.icon_fallback.clone().unwrap_or_else(|| icon.clone()),
                IconMode::Ascii => segment.style.icon_fallback.clone().unwrap_or_default(),
                IconMode::None => String::new(),
                IconMode::Auto => icon.clone(), // Assume Nerd Fonts for auto
            };
            if !display_icon.is_empty() {
                result.push_str(&display_icon);
                result.push(' ');
            }
        }

        // Add content
        result.push_str(&segment.content);

        // Reset styling
        result.push_str("\x1b[0m");

        result
    }

    /// Render continuation prompt
    fn render_continuation(&self) -> String {
        "∙ ".to_string()
    }

    /// Render transient prompt
    fn render_transient(&self) -> String {
        "❯ ".to_string()
    }

    /// Format directory path
    fn format_directory(&self, path: &std::path::Path) -> String {
        // Replace home directory with ~
        if let Some(home) = dirs::home_dir() {
            if let Ok(relative) = path.strip_prefix(&home) {
                return format!("~/{}", relative.display());
            }
        }
        path.display().to_string()
    }

    /// Format duration
    fn format_duration(&self, duration: Duration) -> String {
        let secs = duration.as_secs();
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        }
    }

    /// Detect icon mode
    fn detect_icon_mode(mode: &str) -> IconMode {
        match mode.to_lowercase().as_str() {
            "nerd" => IconMode::Nerd,
            "unicode" => IconMode::Unicode,
            "ascii" => IconMode::Ascii,
            "none" => IconMode::None,
            _ => {
                // Auto-detect: check for Nerd Fonts hint
                if std::env::var("TERMX_ICONS").map(|v| v == "nerd").unwrap_or(false) {
                    IconMode::Nerd
                } else if std::env::var("TERM_PROGRAM").map(|t| t.contains("iTerm") || t.contains("Alacritty")).unwrap_or(false) {
                    IconMode::Nerd // Assume modern terminals have Nerd Fonts
                } else {
                    IconMode::Unicode
                }
            }
        }
    }
}

impl Default for PromptRenderer {
    fn default() -> Self {
        Self::new(PromptConfig::default())
    }
}
