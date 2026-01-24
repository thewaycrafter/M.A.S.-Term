//! MASTerm Core Engine
//!
//! The core library for the MASTerm terminal framework.
//! Provides configuration, context detection, prompt rendering,
//! plugin management, and safety features.

pub mod cache;
pub mod config;
pub mod context;
pub mod plugin;
pub mod prompt;
pub mod safety;

// Re-export commonly used types
pub use config::{Config, ConfigLoader};
pub use context::{Context, ContextDetector};
pub use plugin::{Plugin, PluginManager};
pub use prompt::{Prompt, PromptRenderer};

/// MASTerm version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Target startup time in milliseconds
pub const TARGET_STARTUP_MS: u64 = 50;

/// Target prompt render time in milliseconds
pub const TARGET_PROMPT_MS: u64 = 30;
