//! Configuration schema types (re-exported from mod.rs)
//!
//! This module contains additional schema types and validation logic.

use serde::{Deserialize, Serialize};

/// Shell type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    Zsh,
    Bash,
    Fish,
    PowerShell,
    Unknown,
}

impl ShellType {
    /// Detect shell from environment
    pub fn detect() -> Self {
        std::env::var("SHELL")
            .map(|s| Self::from_path(&s))
            .unwrap_or(Self::Unknown)
    }

    /// Parse shell type from path
    pub fn from_path(path: &str) -> Self {
        if path.ends_with("zsh") {
            Self::Zsh
        } else if path.ends_with("bash") {
            Self::Bash
        } else if path.ends_with("fish") {
            Self::Fish
        } else if path.contains("pwsh") || path.contains("powershell") {
            Self::PowerShell
        } else {
            Self::Unknown
        }
    }

    /// Get shell name as string
    pub fn name(&self) -> &'static str {
        match self {
            Self::Zsh => "zsh",
            Self::Bash => "bash",
            Self::Fish => "fish",
            Self::PowerShell => "powershell",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Mode enumeration for quick switching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// Minimal: fastest startup, basic prompt
    Minimal,
    /// Dev: balanced features (default)
    #[default]
    Dev,
    /// Ops: maximum safety features
    Ops,
}

impl Mode {
    /// Parse mode from string
    pub fn parse_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "minimal" | "min" => Some(Self::Minimal),
            "dev" | "development" => Some(Self::Dev),
            "ops" | "operations" | "prod" => Some(Self::Ops),
            _ => None,
        }
    }

    /// Get mode name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::Dev => "dev",
            Self::Ops => "ops",
        }
    }

    /// Get mode description
    pub fn description(&self) -> &'static str {
        match self {
            Self::Minimal => "Fastest startup, basic prompt only",
            Self::Dev => "Balanced features for development",
            Self::Ops => "Maximum safety features for operations",
        }
    }
}



impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Icon mode enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum IconMode {
    /// Auto-detect based on terminal
    #[default]
    Auto,
    /// Nerd Fonts icons
    Nerd,
    /// Unicode symbols
    Unicode,
    /// ASCII only
    Ascii,
    /// No icons
    None,
}

/// Terminal color capability
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorCapability {
    /// No color support
    None,
    /// 16 colors
    Basic,
    /// 256 colors
    Extended,
    /// 24-bit true color
    TrueColor,
}

impl ColorCapability {
    /// Detect color capability from environment
    pub fn detect() -> Self {
        // Check COLORTERM for true color
        if let Ok(colorterm) = std::env::var("COLORTERM") {
            if colorterm == "truecolor" || colorterm == "24bit" {
                return Self::TrueColor;
            }
        }

        // Check TERM for 256 color
        if let Ok(term) = std::env::var("TERM") {
            if term.contains("256color") {
                return Self::Extended;
            }
            if term.contains("color") || term.starts_with("xterm") {
                return Self::Basic;
            }
        }

        Self::None
    }
}
