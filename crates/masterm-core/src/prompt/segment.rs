//! Prompt segment types

use super::Color;
use serde::{Deserialize, Serialize};

/// Segment position in the prompt
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Position {
    #[default]
    Left,
    Right,
}

/// A single prompt segment
#[derive(Debug, Clone)]
pub struct Segment {
    /// Unique segment name
    pub name: String,

    /// Display content
    pub content: String,

    /// Segment style
    pub style: SegmentStyle,

    /// Priority (lower = more important, shown first)
    pub priority: i32,

    /// Position in prompt
    pub position: Position,

    /// Minimum width (for padding)
    pub min_width: Option<usize>,

    /// Maximum width (for truncation)
    pub max_width: Option<usize>,
}

impl Segment {
    /// Create a new segment
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            content: content.into(),
            style: SegmentStyle::default(),
            priority: 100,
            position: Position::Left,
            min_width: None,
            max_width: None,
        }
    }

    /// Set style
    pub fn with_style(mut self, style: SegmentStyle) -> Self {
        self.style = style;
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set position
    pub fn with_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    /// Set foreground color
    pub fn fg(mut self, color: Color) -> Self {
        self.style.fg = Some(color);
        self
    }

    /// Set background color
    pub fn bg(mut self, color: Color) -> Self {
        self.style.bg = Some(color);
        self
    }

    /// Set bold
    pub fn bold(mut self) -> Self {
        self.style.bold = true;
        self
    }

    /// Set icon
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.style.icon = Some(icon.into());
        self
    }

    /// Set icon with fallback
    pub fn icon_with_fallback(mut self, icon: impl Into<String>, fallback: impl Into<String>) -> Self {
        self.style.icon = Some(icon.into());
        self.style.icon_fallback = Some(fallback.into());
        self
    }

    /// Get display width (accounting for ANSI codes)
    pub fn display_width(&self) -> usize {
        // Count visible characters only
        let icon_width = self.style.icon.as_ref().map(|i| i.chars().count() + 1).unwrap_or(0);
        icon_width + self.content.chars().count()
    }

    /// Truncate content if needed
    pub fn truncate(&mut self, max_width: usize) {
        if self.content.chars().count() > max_width {
            let truncated: String = self.content.chars().take(max_width.saturating_sub(1)).collect();
            self.content = format!("{}…", truncated);
        }
    }
}

/// Style for a segment
#[derive(Debug, Clone, Default)]
pub struct SegmentStyle {
    /// Foreground color
    pub fg: Option<Color>,

    /// Background color
    pub bg: Option<Color>,

    /// Bold text
    pub bold: bool,

    /// Italic text
    pub italic: bool,

    /// Underline
    pub underline: bool,

    /// Icon (Nerd Font)
    pub icon: Option<String>,

    /// Fallback icon (Unicode/ASCII)
    pub icon_fallback: Option<String>,

    /// Prefix string
    pub prefix: Option<String>,

    /// Suffix string
    pub suffix: Option<String>,
}

impl SegmentStyle {
    /// Create a new style
    pub fn new() -> Self {
        Self::default()
    }

    /// Set foreground color
    pub fn fg(mut self, color: Color) -> Self {
        self.fg = Some(color);
        self
    }

    /// Set background color
    pub fn bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    /// Set bold
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    /// Set icon
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// Built-in segment types
#[allow(dead_code)]
pub enum BuiltinSegment {
    /// Current directory
    Directory,
    /// Username
    Username,
    /// Hostname
    Hostname,
    /// Current time
    Time,
    /// Last command duration
    CommandDuration,
    /// Last command exit code (if non-zero)
    ExitCode,
    /// Newline
    Newline,
    /// Custom character (prompt symbol)
    Character,
}
