//! Application state

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Config,
    Plugins,
}

use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

pub struct App {
    pub tab: Tab,
    pub title: String,
    pub system: System,
    pub config_items: Vec<(String, String)>,
    pub plugin_items: Vec<(String, String, String)>,
}

impl App {
    pub fn new() -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        system.refresh_all();
        
        let config_items = Self::load_config();
        let plugin_items = Self::load_plugins();
        
        Self {
            tab: Tab::Dashboard,
            title: "MASTerm Dashboard".to_string(),
            system,
            config_items,
            plugin_items,
        }
    }

    pub fn on_tick(&mut self) {
        self.system.refresh_cpu();
        self.system.refresh_memory();
    }
    
    fn load_config() -> Vec<(String, String)> {
        let mut items = Vec::new();
        
        if let Some(config_path) = dirs::home_dir().map(|h| h.join(".masterm.toml")) {
            if let Ok(content) = std::fs::read_to_string(config_path) {
                // Simple line-based parsing for display purposes
                // Real parsing would require toml crate dependency
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if line.starts_with('[') {
                        items.push((line.to_string(), "".to_string()));
                    } else if let Some((key, val)) = line.split_once('=') {
                        items.push((key.trim().to_string(), val.trim().to_string()));
                    }
                }
            }
        }
        
        if items.is_empty() {
            items.push(("Status".to_string(), "No config found".to_string()));
        }
        
        items
    }
    
    fn load_plugins() -> Vec<(String, String, String)> {
        let mut plugins = vec![
            ("git".to_string(), "1.0.0".to_string(), "Git repository context".to_string()),
            ("env".to_string(), "1.0.0".to_string(), "Environment detection".to_string()),
            ("prod-guard".to_string(), "1.0.0".to_string(), "Production safety".to_string()),
            ("node".to_string(), "1.0.0".to_string(), "Node.js detection".to_string()),
            ("python".to_string(), "1.0.0".to_string(), "Python detection".to_string()),
            ("go".to_string(), "1.0.0".to_string(), "Go detection".to_string()),
            ("rust".to_string(), "1.0.0".to_string(), "Rust detection".to_string()),
        ];
        
        if let Some(plugin_dir) = dirs::home_dir().map(|h| h.join(".masterm/plugins")) {
            if let Ok(entries) = std::fs::read_dir(plugin_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("?");
                            plugins.push((
                                name.to_string(),
                                "1.0.0".to_string(), // Placeholder version
                                format!("{} plugin", ext.to_uppercase())
                            ));
                        }
                    }
                }
            }
        }
        
        if plugins.is_empty() {
            plugins.push(("No plugins".to_string(), "-".to_string(), "Check ~/.masterm/plugins".to_string()));
        }
        
        plugins
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
