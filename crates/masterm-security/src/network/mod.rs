//! Network activity monitoring
//!
//! Detects and analyzes network-bound commands.

use crate::SecurityResult;
use regex::Regex;
use std::collections::HashSet;
use url::Url;

/// Network monitor
pub struct NetworkMonitor {
    /// Known network tools
    network_tools: HashSet<String>,

    /// URL extraction regex
    url_regex: Regex,
}

impl Default for NetworkMonitor {
    fn default() -> Self {
        let tools = vec![
            "curl",
            "wget",
            "ssh",
            "nc",
            "netcat",
            "telnet",
            "ftp",
            "sftp",
            "scp",
            "rsync",
            "git",
            "ping",
            "traceroute",
            "dig",
            "nslookup",
        ];

        Self {
            network_tools: tools.into_iter().map(String::from).collect(),
            // Simple regex for URLs, improved one can be used
            url_regex: Regex::new(r"https?://[^\s/$.?#].[^\s]*").unwrap(),
        }
    }
}

impl NetworkMonitor {
    /// Check if a command is network-bound
    pub fn is_network_bound(&self, command: &str) -> bool {
        let first_word = command.split_whitespace().next().unwrap_or("");
        self.network_tools.contains(first_word)
    }

    /// Extract URLs from command
    pub fn extract_urls(&self, command: &str) -> Vec<Url> {
        let mut urls = Vec::new();

        for capture in self.url_regex.captures_iter(command) {
            if let Some(m) = capture.get(0) {
                if let Ok(url) = Url::parse(m.as_str()) {
                    urls.push(url);
                }
            }
        }

        urls
    }

    /// Analyze a network command
    pub fn analyze(&self, command: &str) -> SecurityResult<NetworkAnalysis> {
        let is_bound = self.is_network_bound(command);
        let urls = self.extract_urls(command);

        // Potential check for IP addresses
        // Potential check for blocked domains (via Reputation module)

        Ok(NetworkAnalysis {
            is_network_bound: is_bound,
            urls,
            tool: command.split_whitespace().next().map(String::from),
        })
    }
}

/// Result of network analysis
#[derive(Debug, Clone)]
pub struct NetworkAnalysis {
    /// Is the command network-bound?
    pub is_network_bound: bool,

    /// Extracted URLs
    pub urls: Vec<Url>,

    /// detected tool
    pub tool: Option<String>,
}
