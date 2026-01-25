//! Network Monitor Plugin
//!
//! Tracks outbound network connections from commands like:
//! - curl, wget, httpie
//! - ssh, scp, sftp, rsync
//! - nc, netcat, ncat, socat

use async_trait::async_trait;
use masterm_core::plugin::{
    ActivationTrigger, CommandAction, DetectionContext, Plugin, PluginActivation, PluginContext,
    PluginError, PluginManifest, PluginMeta, PluginPerformance, PluginPermissions,
    PluginRequirements, PromptContext,
};
use masterm_core::prompt::Segment;
use regex::Regex;

/// Network connection info
#[derive(Debug, Clone)]
struct NetworkConnection {
    /// Command type (curl, ssh, etc.)
    command_type: String,
    /// Target host/IP
    target: String,
    /// Port (if detected)
    port: Option<u16>,
    /// Protocol (if detected)
    protocol: Option<String>,
}

/// Network Monitor Plugin
pub struct NetworkMonitorPlugin {
    manifest: PluginManifest,
    /// Log all connections
    log_connections: bool,
    /// Warn on non-standard ports
    warn_nonstandard_ports: bool,
    /// Blocked ports
    blocked_ports: Vec<u16>,
    /// Blocked domains
    blocked_domains: Vec<String>,
}

impl NetworkMonitorPlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "network-monitor".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Monitor outbound network connections".to_string(),
                    author: "MASTerm Security Team".to_string(),
                    license: "MIT".to_string(),
                    homepage: Some("https://masterm.dev/plugins/network-monitor".to_string()),
                },
                requirements: PluginRequirements::default(),
                permissions: PluginPermissions {
                    filesystem: vec![],
                    network: "none".to_string(),
                    environment: vec![],
                    execute: vec![],
                },
                activation: PluginActivation {
                    triggers: vec![ActivationTrigger::Always],
                    mode: "always".to_string(),
                },
                performance: PluginPerformance {
                    startup_cost: "low".to_string(),
                    runtime_cost: "low".to_string(),
                },
            },
            log_connections: true,
            warn_nonstandard_ports: true,
            blocked_ports: vec![],
            blocked_domains: vec![],
        }
    }

    /// Extract network connections from command
    fn extract_connections(&self, cmd: &str) -> Vec<NetworkConnection> {
        let mut connections = Vec::new();

        // Detect command type
        let _cmd_lower = cmd.to_lowercase();
        let words: Vec<&str> = cmd.split_whitespace().collect();

        if words.is_empty() {
            return connections;
        }

        let first_word = words[0];

        // URL pattern
        let url_re = Regex::new(r"(?i)(https?|ftp|ssh)://([^/:]+)(?::(\d+))?").unwrap();
        for cap in url_re.captures_iter(cmd) {
            connections.push(NetworkConnection {
                command_type: first_word.to_string(),
                target: cap
                    .get(2)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default(),
                port: cap.get(3).and_then(|m| m.as_str().parse().ok()),
                protocol: cap.get(1).map(|m| m.as_str().to_string()),
            });
        }

        // SSH pattern: ssh [user@]host [-p port]
        if first_word == "ssh" || first_word == "scp" || first_word == "sftp" {
            // Extract host from user@host or just host
            let host_re = Regex::new(r"(?:[\w-]+@)?([\w.-]+)").unwrap();
            let port_re = Regex::new(r"-p\s*(\d+)").unwrap();

            let port = port_re
                .captures(cmd)
                .and_then(|c| c.get(1)?.as_str().parse().ok());

            for word in &words[1..] {
                if !word.starts_with('-') && !word.contains('/') {
                    if let Some(cap) = host_re.captures(word) {
                        connections.push(NetworkConnection {
                            command_type: first_word.to_string(),
                            target: cap
                                .get(1)
                                .map(|m| m.as_str().to_string())
                                .unwrap_or_default(),
                            port,
                            protocol: Some("ssh".to_string()),
                        });
                        break;
                    }
                }
            }
        }

        // Netcat pattern: nc [host] [port]
        if first_word == "nc" || first_word == "netcat" || first_word == "ncat" {
            let args: Vec<&str> = words
                .iter()
                .skip(1)
                .filter(|w| !w.starts_with('-'))
                .copied()
                .collect();
            if args.len() >= 2 {
                connections.push(NetworkConnection {
                    command_type: first_word.to_string(),
                    target: args[0].to_string(),
                    port: args[1].parse().ok(),
                    protocol: Some("tcp".to_string()),
                });
            }
        }

        // IP:port pattern for direct connections
        let ip_port_re = Regex::new(r"(\d+\.\d+\.\d+\.\d+):(\d+)").unwrap();
        for cap in ip_port_re.captures_iter(cmd) {
            connections.push(NetworkConnection {
                command_type: first_word.to_string(),
                target: cap
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default(),
                port: cap.get(2).and_then(|m| m.as_str().parse().ok()),
                protocol: None,
            });
        }

        connections
    }

    /// Check if connection should be blocked
    fn should_block(&self, conn: &NetworkConnection) -> Option<String> {
        // Check blocked ports
        if let Some(port) = conn.port {
            if self.blocked_ports.contains(&port) {
                return Some(format!("Port {} is blocked", port));
            }
        }

        // Check blocked domains
        for domain in &self.blocked_domains {
            if conn.target.contains(domain) {
                return Some(format!("Domain {} is blocked", domain));
            }
        }

        None
    }

    /// Check if port is non-standard
    fn is_nonstandard_port(&self, port: u16, protocol: Option<&str>) -> bool {
        let standard_ports = match protocol {
            Some("http") | Some("https") => vec![80, 443, 8080, 8443],
            Some("ssh") => vec![22],
            Some("ftp") => vec![21],
            _ => vec![
                80, 443, 22, 21, 25, 53, 110, 143, 993, 995, 3306, 5432, 6379, 27017,
            ],
        };

        !standard_ports.contains(&port)
    }

    /// Format connection info
    fn format_connection(&self, conn: &NetworkConnection) -> String {
        let port_str = conn.port.map(|p| format!(":{}", p)).unwrap_or_default();
        let protocol_str = conn
            .protocol
            .as_ref()
            .map(|p| format!("{} ", p))
            .unwrap_or_default();
        format!(
            "🌐 {} → {}{}{}",
            conn.command_type, protocol_str, conn.target, port_str
        )
    }
}

impl Default for NetworkMonitorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for NetworkMonitorPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    async fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        if let Some(log) = ctx.get_config_bool("log_connections") {
            self.log_connections = log;
        }
        if let Some(warn) = ctx.get_config_bool("warn_nonstandard_ports") {
            self.warn_nonstandard_ports = warn;
        }
        if let Ok(domains) = ctx.get_string_list("blocked_domains") {
            self.blocked_domains = domains;
        }
        Ok(())
    }

    fn should_activate(&self, _ctx: &DetectionContext) -> bool {
        true
    }

    async fn segments(&self, _ctx: &PromptContext) -> Result<Vec<Segment>, PluginError> {
        Ok(vec![])
    }

    fn on_command(&self, cmd: &str) -> CommandAction {
        let connections = self.extract_connections(cmd);

        if connections.is_empty() {
            return CommandAction::Allow;
        }

        // Check for blocked connections
        for conn in &connections {
            if let Some(reason) = self.should_block(conn) {
                return CommandAction::Block(format!(
                    "\x1b[1;31m🚫 NETWORK CONNECTION BLOCKED\x1b[0m\n\n\
                     {}\n\
                     Reason: {}\n",
                    self.format_connection(conn),
                    reason
                ));
            }
        }

        // Check for non-standard ports
        if self.warn_nonstandard_ports {
            for conn in &connections {
                if let Some(port) = conn.port {
                    if self.is_nonstandard_port(port, conn.protocol.as_deref()) {
                        return CommandAction::Warn(format!(
                            "\x1b[1;33m⚠️  NON-STANDARD PORT DETECTED\x1b[0m\n\n\
                             {}\n\
                             Port {} is not a standard port for this protocol.\n",
                            self.format_connection(conn),
                            port
                        ));
                    }
                }
            }
        }

        // Log connection (in production, this would write to audit log)
        if self.log_connections {
            for conn in &connections {
                tracing::info!("Network connection: {}", self.format_connection(conn));
            }
        }

        CommandAction::Allow
    }

    async fn cleanup(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_extraction() {
        let plugin = NetworkMonitorPlugin::new();

        let conns = plugin.extract_connections("curl https://api.example.com/data");
        assert!(!conns.is_empty());
        assert_eq!(conns[0].target, "api.example.com");

        let conns = plugin.extract_connections("wget http://example.com:8080/file.zip");
        assert!(!conns.is_empty());
        assert_eq!(conns[0].port, Some(8080));
    }

    #[test]
    fn test_ssh_extraction() {
        let plugin = NetworkMonitorPlugin::new();

        let conns = plugin.extract_connections("ssh user@server.example.com -p 2222");
        assert!(!conns.is_empty());
        assert_eq!(conns[0].target, "server.example.com");
        assert_eq!(conns[0].port, Some(2222));
    }

    #[test]
    fn test_nc_extraction() {
        let plugin = NetworkMonitorPlugin::new();

        let conns = plugin.extract_connections("nc 192.168.1.100 4444");
        assert!(!conns.is_empty());
        assert_eq!(conns[0].target, "192.168.1.100");
        assert_eq!(conns[0].port, Some(4444));
    }

    #[test]
    fn test_nonstandard_port_detection() {
        let plugin = NetworkMonitorPlugin::new();

        assert!(plugin.is_nonstandard_port(4444, None));
        assert!(plugin.is_nonstandard_port(9999, Some("http")));
        assert!(!plugin.is_nonstandard_port(443, Some("https")));
        assert!(!plugin.is_nonstandard_port(22, Some("ssh")));
    }
}
