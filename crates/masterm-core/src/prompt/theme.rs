//! Color and theme support

use serde::{Deserialize, Serialize};

/// Color representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Color {
    /// Named color
    Named(NamedColor),
    /// RGB color
    Rgb { r: u8, g: u8, b: u8 },
    /// Hex color string
    Hex(String),
    /// 256-color index
    Index(u8),
}

impl Color {
    /// Create from hex string
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Self::Rgb { r, g, b })
    }

    /// Convert to ANSI foreground escape sequence
    pub fn to_ansi_fg(&self) -> String {
        match self {
            Color::Named(named) => named.to_ansi_fg(),
            Color::Rgb { r, g, b } => format!("\x1b[38;2;{};{};{}m", r, g, b),
            Color::Hex(hex) => {
                if let Some(Color::Rgb { r, g, b }) = Self::from_hex(hex) {
                    format!("\x1b[38;2;{};{};{}m", r, g, b)
                } else {
                    String::new()
                }
            }
            Color::Index(idx) => format!("\x1b[38;5;{}m", idx),
        }
    }

    /// Convert to ANSI background escape sequence
    pub fn to_ansi_bg(&self) -> String {
        match self {
            Color::Named(named) => named.to_ansi_bg(),
            Color::Rgb { r, g, b } => format!("\x1b[48;2;{};{};{}m", r, g, b),
            Color::Hex(hex) => {
                if let Some(Color::Rgb { r, g, b }) = Self::from_hex(hex) {
                    format!("\x1b[48;2;{};{};{}m", r, g, b)
                } else {
                    String::new()
                }
            }
            Color::Index(idx) => format!("\x1b[48;5;{}m", idx),
        }
    }
}

/// Named colors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NamedColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl NamedColor {
    /// Convert to ANSI foreground code
    pub fn to_ansi_fg(&self) -> String {
        let code = match self {
            Self::Black => 30,
            Self::Red => 31,
            Self::Green => 32,
            Self::Yellow => 33,
            Self::Blue => 34,
            Self::Magenta => 35,
            Self::Cyan => 36,
            Self::White => 37,
            Self::BrightBlack => 90,
            Self::BrightRed => 91,
            Self::BrightGreen => 92,
            Self::BrightYellow => 93,
            Self::BrightBlue => 94,
            Self::BrightMagenta => 95,
            Self::BrightCyan => 96,
            Self::BrightWhite => 97,
        };
        format!("\x1b[{}m", code)
    }

    /// Convert to ANSI background code
    pub fn to_ansi_bg(&self) -> String {
        let code = match self {
            Self::Black => 40,
            Self::Red => 41,
            Self::Green => 42,
            Self::Yellow => 43,
            Self::Blue => 44,
            Self::Magenta => 45,
            Self::Cyan => 46,
            Self::White => 47,
            Self::BrightBlack => 100,
            Self::BrightRed => 101,
            Self::BrightGreen => 102,
            Self::BrightYellow => 103,
            Self::BrightBlue => 104,
            Self::BrightMagenta => 105,
            Self::BrightCyan => 106,
            Self::BrightWhite => 107,
        };
        format!("\x1b[{}m", code)
    }
}

/// Color theme
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme name
    pub name: String,

    /// Directory color
    pub directory: Color,

    /// Git branch color
    pub git_branch: Color,

    /// Git clean status color
    pub git_clean: Color,

    /// Git dirty status color
    pub git_dirty: Color,

    /// Error/exit code color
    pub error: Color,

    /// Success color
    pub success: Color,

    /// Warning color
    pub warning: Color,

    /// Production environment color
    pub prod: Color,

    /// Staging environment color
    pub staging: Color,

    /// Command duration color
    pub duration: Color,
}

impl Theme {
    /// Catppuccin Mocha theme
    pub fn catppuccin() -> Self {
        Self {
            name: "catppuccin".to_string(),
            directory: Color::from_hex("#89b4fa").unwrap(), // Blue
            git_branch: Color::from_hex("#cba6f7").unwrap(), // Mauve
            git_clean: Color::from_hex("#a6e3a1").unwrap(),  // Green
            git_dirty: Color::from_hex("#f38ba8").unwrap(),  // Red
            error: Color::from_hex("#f38ba8").unwrap(),      // Red
            success: Color::from_hex("#a6e3a1").unwrap(),    // Green
            warning: Color::from_hex("#f9e2af").unwrap(),    // Yellow
            prod: Color::from_hex("#f38ba8").unwrap(),       // Red
            staging: Color::from_hex("#f9e2af").unwrap(),    // Yellow
            duration: Color::from_hex("#f5c2e7").unwrap(),   // Pink
        }
    }

    /// Dracula theme
    pub fn dracula() -> Self {
        Self {
            name: "dracula".to_string(),
            directory: Color::from_hex("#bd93f9").unwrap(), // Purple
            git_branch: Color::from_hex("#ff79c6").unwrap(), // Pink
            git_clean: Color::from_hex("#50fa7b").unwrap(),  // Green
            git_dirty: Color::from_hex("#ff5555").unwrap(),  // Red
            error: Color::from_hex("#ff5555").unwrap(),
            success: Color::from_hex("#50fa7b").unwrap(),
            warning: Color::from_hex("#f1fa8c").unwrap(),    // Yellow
            prod: Color::from_hex("#ff5555").unwrap(),
            staging: Color::from_hex("#f1fa8c").unwrap(),
            duration: Color::from_hex("#8be9fd").unwrap(),   // Cyan
        }
    }

    /// Nord theme
    pub fn nord() -> Self {
        Self {
            name: "nord".to_string(),
            directory: Color::from_hex("#81a1c1").unwrap(), // Frost
            git_branch: Color::from_hex("#b48ead").unwrap(), // Aurora purple
            git_clean: Color::from_hex("#a3be8c").unwrap(),  // Aurora green
            git_dirty: Color::from_hex("#bf616a").unwrap(),  // Aurora red
            error: Color::from_hex("#bf616a").unwrap(),
            success: Color::from_hex("#a3be8c").unwrap(),
            warning: Color::from_hex("#ebcb8b").unwrap(),    // Aurora yellow
            prod: Color::from_hex("#bf616a").unwrap(),
            staging: Color::from_hex("#ebcb8b").unwrap(),
            duration: Color::from_hex("#88c0d0").unwrap(),   // Frost
        }
    }

    /// Get theme by name
    pub fn by_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "dracula" => Self::dracula(),
            "nord" => Self::nord(),
            _ => Self::catppuccin(), // Default
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::catppuccin()
    }
}
