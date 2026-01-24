//! Context detection system
//!
//! Automatically detects the current environment context including:
//! - Git repository status
//! - Programming languages
//! - Container environments
//! - Environment type (dev/staging/prod)

mod container;
mod detector;
mod environment;
mod git;
mod language;

pub use container::ContainerContext;
pub use detector::{Context, ContextDetector};
pub use environment::EnvironmentType;
pub use git::GitContext;
pub use language::LanguageContext;
