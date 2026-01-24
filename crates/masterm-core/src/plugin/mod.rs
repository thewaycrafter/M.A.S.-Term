//! Plugin system for MASTerm
//!
//! Provides a modular plugin architecture with:
//! - Declarative manifests
//! - Permission system
//! - Lazy loading
//! - Lifecycle management

mod loader;
mod api;
mod permissions;

pub use api::{Plugin, PluginContext, PluginError, CommandAction, DetectionContext};
pub use api::PromptContext;
pub use loader::{PluginLoader, PluginManager};
pub use permissions::{Permission, PermissionSet};

use serde::{Deserialize, Serialize};

/// Plugin manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin metadata
    pub plugin: PluginMeta,

    /// Requirements
    #[serde(default)]
    pub requirements: PluginRequirements,

    /// Permissions
    #[serde(default)]
    pub permissions: PluginPermissions,

    /// Activation triggers
    #[serde(default)]
    pub activation: PluginActivation,

    /// Performance hints
    #[serde(default)]
    pub performance: PluginPerformance,
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    /// Plugin name
    pub name: String,

    /// Version
    pub version: String,

    /// Description
    pub description: String,

    /// Author
    #[serde(default)]
    pub author: String,

    /// License
    #[serde(default)]
    pub license: String,

    /// Homepage URL
    #[serde(default)]
    pub homepage: Option<String>,
}

/// Plugin requirements
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PluginRequirements {
    /// Required binaries
    pub binaries: Vec<String>,

    /// Minimum masterm version
    pub masterm_version: Option<String>,

    /// Plugin dependencies
    pub dependencies: Vec<String>,
}

/// Plugin permissions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PluginPermissions {
    /// Filesystem access level
    pub filesystem: Vec<String>,

    /// Network access level
    pub network: String,

    /// Environment variable access
    pub environment: Vec<String>,

    /// Binaries that can be executed
    pub execute: Vec<String>,
}

/// Plugin activation triggers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PluginActivation {
    /// Activation triggers
    pub triggers: Vec<ActivationTrigger>,

    /// Activation mode
    pub mode: String,
}

/// Activation trigger types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActivationTrigger {
    /// File exists
    FileExists { pattern: String },

    /// Directory exists
    DirectoryExists { pattern: String },

    /// Environment variable set
    EnvVar { name: String, value: Option<String> },

    /// Always active
    Always,
}

/// Plugin performance hints
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PluginPerformance {
    /// Startup cost
    pub startup_cost: String,

    /// Runtime cost
    pub runtime_cost: String,
}
