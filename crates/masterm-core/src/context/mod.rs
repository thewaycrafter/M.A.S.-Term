//! Context detection system
//!
//! Automatically detects the current environment context including:
//! - Git repository status
//! - Programming languages
//! - Container environments
//! - Environment type (dev/staging/prod)

mod detector;
mod git;
mod language;
mod container;
mod environment;

pub use detector::{Context, ContextDetector};
pub use environment::EnvironmentType;
pub use git::GitContext;
pub use language::LanguageContext;
pub use container::ContainerContext;
