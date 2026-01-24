//! Prompt rendering system
//!
//! Generates beautiful, context-aware prompts with support for:
//! - Multiple segments (left/right)
//! - Icons with fallback
//! - Themes and colors
//! - Terminal capability detection

mod renderer;
mod segment;
pub mod theme;

pub use renderer::{Prompt, PromptRenderer};
pub use segment::{Segment, SegmentStyle, Position};
pub use theme::{Theme, Color, NamedColor};
