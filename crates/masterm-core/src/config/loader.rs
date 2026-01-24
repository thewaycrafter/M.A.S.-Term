//! Configuration loader with multi-tier support

use super::{Config, EnterpriseConfig};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

/// Configuration loader with support for enterprise, user, and project configs
pub struct ConfigLoader {
    /// Enterprise config path (system-wide)
    enterprise_path: PathBuf,

    /// User config path
    user_path: PathBuf,

    /// Project config filename
    project_filename: String,
}

impl ConfigLoader {
    /// Create a new config loader with default paths
    pub fn new() -> Self {
        Self {
            enterprise_path: PathBuf::from("/etc/masterm/enterprise.toml"),
            user_path: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("~"))
                .join(".masterm.toml"),
            project_filename: ".masterm.toml".to_string(),
        }
    }

    /// Create a config loader with custom paths (for testing)
    pub fn with_paths(enterprise: PathBuf, user: PathBuf, project_filename: String) -> Self {
        Self {
            enterprise_path: enterprise,
            user_path: user,
            project_filename,
        }
    }

    /// Load and merge all configuration tiers
    pub fn load(&self, cwd: &Path) -> Result<Config> {
        let mut config = Config::default();

        // Load user config (lowest precedence for overrides)
        if self.user_path.exists() {
            debug!("Loading user config from {:?}", self.user_path);
            let user_config = self.load_file(&self.user_path)?;
            config = self.merge_config(config, user_config);
        }

        // Load project config (overrides user)
        let project_path = self.find_project_config(cwd);
        if let Some(ref path) = project_path {
            debug!("Loading project config from {:?}", path);
            let project_config = self.load_file(path)?;
            config = self.merge_config(config, project_config);
        }

        // Load enterprise config (highest precedence, applies lockdowns)
        if self.enterprise_path.exists() {
            debug!("Loading enterprise config from {:?}", self.enterprise_path);
            let enterprise = self.load_enterprise_config()?;
            config = self.apply_enterprise_lockdown(config, &enterprise);
        }

        Ok(config)
    }

    /// Load a single config file
    fn load_file(&self, path: &Path) -> Result<Config> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))
    }

    /// Load enterprise configuration
    fn load_enterprise_config(&self) -> Result<EnterpriseConfig> {
        let content = std::fs::read_to_string(&self.enterprise_path)
            .with_context(|| "Failed to read enterprise config")?;

        toml::from_str(&content).with_context(|| "Failed to parse enterprise config")
    }

    /// Find project config by walking up the directory tree
    fn find_project_config(&self, start: &Path) -> Option<PathBuf> {
        let mut current = start.to_path_buf();

        loop {
            let config_path = current.join(&self.project_filename);
            if config_path.exists() {
                return Some(config_path);
            }

            if !current.pop() {
                break;
            }
        }

        None
    }

    /// Merge two configs (later overrides earlier)
    fn merge_config(&self, base: Config, overlay: Config) -> Config {
        // For MVP, we do a simple field-level merge
        // In production, this would be more sophisticated
        Config {
            core: if overlay.core.shell != "auto" {
                overlay.core
            } else {
                base.core
            },
            prompt: overlay.prompt,
            plugins: PluginsConfigMerge::merge(base.plugins, overlay.plugins),
            safety: overlay.safety,
            cache: overlay.cache,
            telemetry: overlay.telemetry,
        }
    }

    /// Apply enterprise lockdown settings
    fn apply_enterprise_lockdown(&self, mut config: Config, enterprise: &EnterpriseConfig) -> Config {
        if !enterprise.enterprise.enabled {
            return config;
        }

        info!("Enterprise mode enabled for org: {}", enterprise.enterprise.org_id);

        // Apply locked settings
        for setting in &enterprise.lockdown.locked_settings {
            match setting.as_str() {
                "safety.prod_detection" => {
                    warn!("Enterprise locked: safety.prod_detection");
                    // Keep enterprise value, ignore user override
                }
                "telemetry.enabled" => {
                    warn!("Enterprise locked: telemetry.enabled");
                }
                _ => {}
            }
        }

        // Merge plugin controls
        for denied in &enterprise.plugins.denylist {
            if !config.plugins.disabled.contains(denied) {
                config.plugins.disabled.push(denied.clone());
            }
        }

        // Force safety patterns
        for pattern in &enterprise.safety.force_prod_patterns {
            if !config.safety.prod_patterns.contains(pattern) {
                config.safety.prod_patterns.push(pattern.clone());
            }
        }

        for cmd in &enterprise.safety.force_dangerous_commands {
            if !config.safety.dangerous_commands.contains(cmd) {
                config.safety.dangerous_commands.push(cmd.clone());
            }
        }

        config
    }

    /// Get the user config path
    pub fn user_config_path(&self) -> &Path {
        &self.user_path
    }

    /// Create a default config file if it doesn't exist
    pub fn create_default_config(&self) -> Result<()> {
        if self.user_path.exists() {
            return Ok(());
        }

        let default_config = include_str!("../../assets/default_config.toml");

        if let Some(parent) = self.user_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&self.user_path, default_config)
            .with_context(|| "Failed to create default config")?;

        info!("Created default config at {:?}", self.user_path);
        Ok(())
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper for merging plugin configs
struct PluginsConfigMerge;

impl PluginsConfigMerge {
    fn merge(base: super::PluginsConfig, overlay: super::PluginsConfig) -> super::PluginsConfig {
        super::PluginsConfig {
            registry: overlay.registry.or(base.registry),
            enabled: if overlay.enabled.is_empty() {
                base.enabled
            } else {
                overlay.enabled
            },
            disabled: {
                let mut disabled = base.disabled;
                disabled.extend(overlay.disabled);
                disabled
            },
            plugin_configs: {
                let mut configs = base.plugin_configs;
                configs.extend(overlay.plugin_configs);
                configs
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_load_default_config() {
        let loader = ConfigLoader::new();
        let temp_dir = TempDir::new().unwrap();
        let config = loader.load(temp_dir.path()).unwrap();

        assert_eq!(config.core.mode, "dev");
        assert!(config.safety.prod_detection);
    }

    #[test]
    fn test_find_project_config() {
        let temp_dir = TempDir::new().unwrap();
        let nested = temp_dir.path().join("a/b/c");
        std::fs::create_dir_all(&nested).unwrap();

        let config_path = temp_dir.path().join(".masterm.toml");
        std::fs::write(&config_path, "[core]\nmode = \"minimal\"").unwrap();

        let loader = ConfigLoader::new();
        let found = loader.find_project_config(&nested);

        assert!(found.is_some());
        assert_eq!(found.unwrap(), config_path);
    }
}
