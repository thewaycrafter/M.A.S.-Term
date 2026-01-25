//! Application state

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Config,
    Plugins,
}

use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Networks, RefreshKind, System};

pub struct App {
    pub tab: Tab,
    pub title: String,
    pub system: System,
    pub networks: Networks,
    pub config_items: Vec<(String, String)>,
    pub plugin_items: Vec<(String, String, String)>,
    // History for charts
    pub cpu_history: Vec<(f64, f64)>,
    pub mem_history: Vec<(f64, f64)>,
    pub rx_history: Vec<(f64, f64)>,
    pub tx_history: Vec<(f64, f64)>,
    pub tick_count: f64,
    // Static info
    pub os_info: String,
    pub host_name: String,
    pub kernel_ver: String,
}

impl App {
    pub fn new() -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        system.refresh_all();
        let networks = Networks::new_with_refreshed_list();

        let config_items = Self::load_config();
        let plugin_items = Self::load_plugins();

        // Static info
        let os_info = System::long_os_version().unwrap_or_else(|| "Unknown".to_string());
        let host_name = System::host_name().unwrap_or_else(|| "localhost".to_string());
        let kernel_ver = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());

        Self {
            tab: Tab::Dashboard,
            title: "MASTerm Dashboard".to_string(),
            system,
            networks,
            config_items,
            plugin_items,
            cpu_history: Vec::new(),
            mem_history: Vec::new(),
            rx_history: Vec::new(),
            tx_history: Vec::new(),
            tick_count: 0.0,
            os_info,
            host_name,
            kernel_ver,
        }
    }

    pub fn on_tick(&mut self) {
        self.system.refresh_cpu();
        self.system.refresh_memory();
        self.networks.refresh();

        self.tick_count += 1.0;

        // Update CPU History
        let global_cpu = self.system.global_cpu_info().cpu_usage() as f64;
        self.cpu_history.push((self.tick_count, global_cpu));

        // Update Memory History
        let used_mem = self.system.used_memory() as f64;
        let total_mem = self.system.total_memory() as f64;
        let mem_percent = (used_mem / total_mem) * 100.0;
        self.mem_history.push((self.tick_count, mem_percent));

        // Update Network History (sum of all interfaces)
        let (rx, tx) = self.networks.iter().fold((0, 0), |acc, (_, data)| {
            (acc.0 + data.received(), acc.1 + data.transmitted())
        });
        // Convert to KB
        self.rx_history.push((self.tick_count, rx as f64 / 1024.0));
        self.tx_history.push((self.tick_count, tx as f64 / 1024.0));

        // Keep last 100 points
        if self.cpu_history.len() > 100 {
            self.cpu_history.remove(0);
            self.mem_history.remove(0);
            self.rx_history.remove(0);
            self.tx_history.remove(0);
        }
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
            (
                "git".to_string(),
                "1.0.0".to_string(),
                "Git repository context".to_string(),
            ),
            (
                "env".to_string(),
                "1.0.0".to_string(),
                "Environment detection".to_string(),
            ),
            (
                "prod-guard".to_string(),
                "1.0.0".to_string(),
                "Production safety".to_string(),
            ),
            (
                "node".to_string(),
                "1.0.0".to_string(),
                "Node.js detection".to_string(),
            ),
            (
                "python".to_string(),
                "1.0.0".to_string(),
                "Python detection".to_string(),
            ),
            (
                "go".to_string(),
                "1.0.0".to_string(),
                "Go detection".to_string(),
            ),
            (
                "rust".to_string(),
                "1.0.0".to_string(),
                "Rust detection".to_string(),
            ),
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
                                format!("{} plugin", ext.to_uppercase()),
                            ));
                        }
                    }
                }
            }
        }

        if plugins.is_empty() {
            plugins.push((
                "No plugins".to_string(),
                "-".to_string(),
                "Check ~/.masterm/plugins".to_string(),
            ));
        }

        plugins
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
