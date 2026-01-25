//! IP/Domain Reputation Plugin
//!
//! Checks connections against threat intelligence:
//! - Local blocklist
//! - Optional external API integration

use async_trait::async_trait;
use masterm_core::plugin::{
    ActivationTrigger, CommandAction, DetectionContext, Plugin, PluginActivation, PluginContext,
    PluginError, PluginManifest, PluginMeta, PluginPerformance, PluginPermissions,
    PluginRequirements, PromptContext,
};
use masterm_core::prompt::Segment;
use masterm_security::reputation::{extract_targets, ReputationCache, ReputationResult};
use std::sync::Arc;
use tokio::sync::RwLock;

/// IP/Domain Reputation Plugin
pub struct IpReputationPlugin {
    manifest: PluginManifest,
    /// Reputation cache
    cache: Arc<RwLock<ReputationCache>>,
    /// Is the plugin enabled
    enabled: bool,
}

impl IpReputationPlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "ip-reputation".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Check IP/domain reputation against threat intelligence".to_string(),
                    author: "MASTerm Security Team".to_string(),
                    license: "MIT".to_string(),
                    homepage: Some("https://masterm.dev/plugins/ip-reputation".to_string()),
                },
                requirements: PluginRequirements::default(),
                permissions: PluginPermissions {
                    filesystem: vec!["read".to_string()],
                    network: "none".to_string(), // Offline mode by default
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
            cache: Arc::new(RwLock::new(ReputationCache::new(86400))), // 24 hour TTL
            enabled: false, // Disabled by default - requires configuration
        }
    }

    /// Check targets in command synchronously (for on_command)
    fn check_targets_sync(&self, cmd: &str) -> Vec<(String, ReputationResult)> {
        let targets = extract_targets(cmd);
        let mut results = Vec::new();

        // For sync context, we use a blocking approach
        // In production, this would be properly async
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let cache = self.cache.clone();
            for target in targets {
                let cache_clone = cache.clone();
                let target_clone = target.clone();
                
                // Use block_in_place for sync context
                let result = handle.block_on(async {
                    let cache = cache_clone.read().await;
                    cache.check(&target_clone).await
                });
                
                results.push((target, result));
            }
        }

        results
    }

    /// Format warning for suspicious targets
    fn format_warning(&self, results: &[(String, ReputationResult)]) -> String {
        let mut msg = String::from("\x1b[1;33m⚠️  SUSPICIOUS DESTINATION DETECTED\x1b[0m\n\n");

        for (target, result) in results {
            match result {
                ReputationResult::Suspicious(info) | ReputationResult::Malicious(info) => {
                    let icon = if matches!(result, ReputationResult::Malicious(_)) {
                        "🚨"
                    } else {
                        "⚠️"
                    };
                    msg.push_str(&format!(
                        "  {} \x1b[1m{}\x1b[0m\n     Threat: {} (confidence: {}%)\n     Source: {}\n",
                        icon, target, info.threat_type, info.confidence, info.source
                    ));
                }
                _ => {}
            }
        }

        msg.push_str("\nThis destination appears in threat intelligence feeds.\n");
        msg
    }

    /// Format block message
    fn format_block(&self, results: &[(String, ReputationResult)]) -> String {
        let mut msg = String::from("\x1b[1;31m🚫 MALICIOUS DESTINATION BLOCKED\x1b[0m\n\n");

        for (target, result) in results {
            if let ReputationResult::Malicious(info) = result {
                msg.push_str(&format!(
                    "  🚨 \x1b[1;31m{}\x1b[0m\n     Threat: {} (confidence: {}%)\n",
                    target, info.threat_type, info.confidence
                ));
            }
        }

        msg.push_str("\n\x1b[31mConnection to this destination has been blocked.\x1b[0m\n");
        msg.push_str("If you believe this is a false positive, update your blocklist.\n");

        msg
    }
}

impl Default for IpReputationPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for IpReputationPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    async fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        // Check if enabled
        if let Some(enabled) = ctx.get_config_bool("enabled") {
            self.enabled = enabled;
        }

        // Load custom blocklist
        if let Some(blocklist_path) = ctx.get_config_string("blocklist_path") {
            let cache = self.cache.read().await;
            let path = std::path::Path::new(&blocklist_path);
            if path.exists() {
                let _ = cache.load_blocklist(path).await;
            }
        }

        // Add custom blocked domains
        if let Ok(domains) = ctx.get_string_list("blocked_domains") {
            let cache = self.cache.read().await;
            for domain in domains {
                cache.add_to_blocklist(&domain).await;
            }
        }

        Ok(())
    }

    fn should_activate(&self, _ctx: &DetectionContext) -> bool {
        self.enabled
    }

    async fn segments(&self, _ctx: &PromptContext) -> Result<Vec<Segment>, PluginError> {
        Ok(vec![])
    }

    fn on_command(&self, cmd: &str) -> CommandAction {
        if !self.enabled {
            return CommandAction::Allow;
        }

        let results = self.check_targets_sync(cmd);

        // Filter to only threats
        let threats: Vec<_> = results
            .iter()
            .filter(|(_, r)| r.is_threat())
            .cloned()
            .collect();

        if threats.is_empty() {
            return CommandAction::Allow;
        }

        // Check for malicious (high confidence) threats
        let has_malicious = threats.iter().any(|(_, r)| r.is_malicious());

        if has_malicious {
            CommandAction::Block(self.format_block(&threats))
        } else {
            CommandAction::Confirm(format!(
                "{}\nType '\x1b[1;32myes\x1b[0m' to proceed or '\x1b[1;31mno\x1b[0m' to cancel:",
                self.format_warning(&threats)
            ))
        }
    }

    async fn cleanup(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = IpReputationPlugin::new();
        assert!(!plugin.enabled); // Disabled by default
    }

    #[test]
    fn test_target_extraction() {
        let targets = extract_targets("curl http://192.168.1.100:8080/api");
        assert!(targets.contains(&"192.168.1.100".to_string()));
    }
}
