//! Plugin loader and manager

use super::{ActivationTrigger, DetectionContext, Plugin, PluginContext, PluginManifest};
use crate::prompt::Segment;
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Plugin loader for discovering and loading plugins
pub struct PluginLoader {
    /// Plugin directories to search
    plugin_dirs: Vec<PathBuf>,

    /// Disabled plugins
    disabled: Vec<String>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new() -> Self {
        let mut plugin_dirs = Vec::new();

        // User plugin directory
        if let Some(home) = dirs::home_dir() {
            plugin_dirs.push(home.join(".masterm/plugins"));
        }

        // System plugin directory
        plugin_dirs.push(PathBuf::from("/usr/local/share/masterm/plugins"));

        Self {
            plugin_dirs,
            disabled: Vec::new(),
        }
    }

    /// Set disabled plugins
    pub fn with_disabled(mut self, disabled: Vec<String>) -> Self {
        self.disabled = disabled;
        self
    }

    /// Discover available plugin manifests
    pub fn discover(&self) -> Vec<PluginManifest> {
        let mut manifests = Vec::new();

        for dir in &self.plugin_dirs {
            if !dir.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();

                    // Case 1: Directory with plugin.toml
                    if path.is_dir() {
                        let manifest_path = path.join("plugin.toml");
                        if manifest_path.exists() {
                            if let Ok(manifest) = self.load_manifest(&manifest_path) {
                                if !self.disabled.contains(&manifest.plugin.name) {
                                    manifests.push(manifest);
                                }
                            }
                        }
                    }
                    // Case 2: Standalone .wasm file
                    else if path.extension().is_some_and(|ext| ext == "wasm") {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            if !self.disabled.contains(&name.to_string()) {
                                manifests.push(self.synthetic_manifest(name));
                            }
                        }
                    }
                }
            }
        }

        manifests
    }

    /// Load a plugin manifest from file
    fn load_manifest(&self, path: &Path) -> Result<PluginManifest> {
        let content = std::fs::read_to_string(path)?;
        let manifest: PluginManifest = toml::from_str(&content)?;
        Ok(manifest)
    }

    /// Create a synthetic manifest for a standalone WASM file
    fn synthetic_manifest(&self, name: &str) -> PluginManifest {
        use super::{
            PluginActivation, PluginMeta, PluginPerformance, PluginPermissions, PluginRequirements,
        };

        PluginManifest {
            plugin: PluginMeta {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: "Standalone WASM plugin".to_string(),
                author: "Unknown".to_string(),
                license: "None".to_string(),
                homepage: None,
            },
            requirements: PluginRequirements::default(),
            permissions: PluginPermissions::default(),
            activation: PluginActivation::default(),
            performance: PluginPerformance::default(),
        }
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin manager for managing loaded plugins
pub struct PluginManager {
    /// Loaded plugins
    plugins: HashMap<String, Arc<RwLock<Box<dyn Plugin>>>>,

    /// Plugin configurations
    configs: HashMap<String, HashMap<String, toml::Value>>,

    /// Currently active plugins
    active: Vec<String>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            configs: HashMap::new(),
            active: Vec::new(),
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        let name = plugin.manifest().plugin.name.clone();
        self.plugins.insert(name, Arc::new(RwLock::new(plugin)));
    }

    /// Set configuration for a plugin
    pub fn configure(&mut self, name: &str, config: HashMap<String, toml::Value>) {
        self.configs.insert(name.to_string(), config);
    }

    /// Initialize all registered plugins
    pub async fn init_all(&mut self, cwd: &Path) -> Result<()> {
        for (name, plugin) in &self.plugins {
            let config = self.configs.get(name).cloned().unwrap_or_default();
            let ctx = PluginContext {
                config,
                cwd: cwd.to_path_buf(),
                env_vars: std::env::vars().collect(),
                cache_key: format!("plugin_{}", name),
            };

            let mut plugin = plugin.write().await;
            if let Err(e) = plugin.init(&ctx).await {
                warn!("Failed to initialize plugin {}: {}", name, e);
            } else {
                debug!("Initialized plugin: {}", name);
            }
        }

        Ok(())
    }

    /// Update active plugins based on current context
    pub async fn update_active(&mut self, ctx: &DetectionContext) {
        self.active.clear();

        for (name, plugin) in &self.plugins {
            let plugin = plugin.read().await;
            if plugin.should_activate(ctx) {
                self.active.push(name.clone());
                debug!("Plugin activated: {}", name);
            }
        }
    }

    /// Get segments from all active plugins
    pub async fn get_segments(&self, ctx: &super::api::PromptContext) -> Vec<Segment> {
        let mut segments = Vec::new();

        for name in &self.active {
            if let Some(plugin) = self.plugins.get(name) {
                let plugin = plugin.read().await;
                match plugin.segments(ctx).await {
                    Ok(mut plugin_segments) => segments.append(&mut plugin_segments),
                    Err(e) => warn!("Plugin {} failed to generate segments: {}", name, e),
                }
            }
        }

        segments
    }

    /// Check command against all active plugins for safety
    pub async fn check_command(&self, cmd: &str) -> super::api::CommandAction {
        for name in &self.active {
            if let Some(plugin) = self.plugins.get(name) {
                let plugin = plugin.read().await;
                let action = plugin.on_command(cmd);
                if action != super::api::CommandAction::Allow {
                    return action;
                }
            }
        }

        super::api::CommandAction::Allow
    }

    /// Get list of active plugin names
    pub fn active_plugins(&self) -> &[String] {
        &self.active
    }

    /// Cleanup all plugins
    pub async fn cleanup(&mut self) -> Result<()> {
        for (name, plugin) in &self.plugins {
            let mut plugin = plugin.write().await;
            if let Err(e) = plugin.cleanup().await {
                warn!("Plugin {} cleanup failed: {}", name, e);
            }
        }
        Ok(())
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a trigger matches the detection context
pub fn _trigger_matches(trigger: &ActivationTrigger, ctx: &DetectionContext) -> bool {
    match trigger {
        ActivationTrigger::FileExists { pattern } => {
            ctx.cwd.join(pattern).exists() || ctx.has_file_matching(pattern)
        }
        ActivationTrigger::DirectoryExists { pattern } => ctx.cwd.join(pattern).is_dir(),
        ActivationTrigger::EnvVar { name, value } => match (ctx.env_vars.get(name), value) {
            (Some(actual), Some(expected)) => actual == expected,
            (Some(_), None) => true,
            (None, _) => false,
        },
        ActivationTrigger::Always => true,
    }
}
