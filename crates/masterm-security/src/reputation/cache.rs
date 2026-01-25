//! Reputation cache
//!
//! Local caching of IP/domain reputation data.

use crate::SecurityResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Reputation lookup result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReputationResult {
    /// Clean/safe
    Clean,
    /// Unknown (not in any list)
    Unknown,
    /// Suspicious (low confidence threat)
    Suspicious(ThreatInfo),
    /// Malicious (high confidence threat)
    Malicious(ThreatInfo),
}

impl ReputationResult {
    /// Is this result indicating a threat?
    pub fn is_threat(&self) -> bool {
        matches!(self, Self::Suspicious(_) | Self::Malicious(_))
    }

    /// Is this malicious?
    pub fn is_malicious(&self) -> bool {
        matches!(self, Self::Malicious(_))
    }
}

/// Threat information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreatInfo {
    /// Threat type (malware, phishing, botnet, etc.)
    pub threat_type: String,

    /// Confidence score (0-100)
    pub confidence: u8,

    /// Source of the threat intelligence
    pub source: String,

    /// When this was last updated
    pub last_updated: String,

    /// Additional notes
    pub notes: Option<String>,
}

impl ThreatInfo {
    /// Create new threat info
    pub fn new(
        threat_type: impl Into<String>,
        confidence: u8,
        source: impl Into<String>,
    ) -> Self {
        Self {
            threat_type: threat_type.into(),
            confidence,
            source: source.into(),
            last_updated: chrono::Utc::now().to_rfc3339(),
            notes: None,
        }
    }

    /// Add notes
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

/// Cached entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// IP or domain
    target: String,

    /// Reputation result
    result: CachedResult,

    /// Cache timestamp
    cached_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum CachedResult {
    Clean,
    Unknown,
    Suspicious(ThreatInfo),
    Malicious(ThreatInfo),
}

impl From<CachedResult> for ReputationResult {
    fn from(cached: CachedResult) -> Self {
        match cached {
            CachedResult::Clean => ReputationResult::Clean,
            CachedResult::Unknown => ReputationResult::Unknown,
            CachedResult::Suspicious(info) => ReputationResult::Suspicious(info),
            CachedResult::Malicious(info) => ReputationResult::Malicious(info),
        }
    }
}

impl From<ReputationResult> for CachedResult {
    fn from(result: ReputationResult) -> Self {
        match result {
            ReputationResult::Clean => CachedResult::Clean,
            ReputationResult::Unknown => CachedResult::Unknown,
            ReputationResult::Suspicious(info) => CachedResult::Suspicious(info),
            ReputationResult::Malicious(info) => CachedResult::Malicious(info),
        }
    }
}

/// Reputation cache
#[allow(dead_code)]
pub struct ReputationCache {
    /// In-memory cache
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,

    /// Blocklist (IPs/domains known to be malicious)
    blocklist: Arc<RwLock<Vec<String>>>,

    /// Cache TTL in seconds
    ttl: u64,

    /// Cache file path (for future persistence)
    cache_path: Option<PathBuf>,

    /// Blocklist path (for future file loading)
    blocklist_path: Option<PathBuf>,
}

impl ReputationCache {
    /// Create a new reputation cache
    pub fn new(ttl: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            blocklist: Arc::new(RwLock::new(default_blocklist())),
            ttl,
            cache_path: None,
            blocklist_path: None,
        }
    }

    /// Create with custom paths
    pub fn with_paths(
        ttl: u64,
        cache_path: Option<PathBuf>,
        blocklist_path: Option<PathBuf>,
    ) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            blocklist: Arc::new(RwLock::new(default_blocklist())),
            ttl,
            cache_path,
            blocklist_path,
        }
    }

    /// Load blocklist from file
    pub async fn load_blocklist(&self, path: &Path) -> SecurityResult<usize> {
        let content = tokio::fs::read_to_string(path).await?;
        let mut blocklist = self.blocklist.write().await;

        let mut count = 0;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            blocklist.push(line.to_string());
            count += 1;
        }

        Ok(count)
    }

    /// Check IP reputation
    pub async fn check_ip(&self, ip: &IpAddr) -> ReputationResult {
        let ip_str = ip.to_string();
        self.check(&ip_str).await
    }

    /// Check domain reputation
    pub async fn check_domain(&self, domain: &str) -> ReputationResult {
        self.check(domain).await
    }

    /// Check reputation of IP or domain
    pub async fn check(&self, target: &str) -> ReputationResult {
        let target_lower = target.to_lowercase();

        // Check cache first
        if let Some(cached) = self.get_cached(&target_lower).await {
            return cached;
        }

        // Check blocklist
        let blocklist = self.blocklist.read().await;
        for blocked in blocklist.iter() {
            if target_lower.contains(blocked) || blocked.contains(&target_lower) {
                let result = ReputationResult::Malicious(ThreatInfo::new(
                    "blocklisted",
                    100,
                    "local_blocklist",
                ));

                // Cache the result
                drop(blocklist);
                self.cache_result(&target_lower, result.clone()).await;

                return result;
            }
        }

        ReputationResult::Unknown
    }

    /// Get cached result
    async fn get_cached(&self, target: &str) -> Option<ReputationResult> {
        let cache = self.cache.read().await;

        if let Some(entry) = cache.get(target) {
            let now = chrono::Utc::now().timestamp();
            if now - entry.cached_at < self.ttl as i64 {
                return Some(entry.result.clone().into());
            }
        }

        None
    }

    /// Cache a result
    async fn cache_result(&self, target: &str, result: ReputationResult) {
        let mut cache = self.cache.write().await;

        cache.insert(
            target.to_string(),
            CacheEntry {
                target: target.to_string(),
                result: result.into(),
                cached_at: chrono::Utc::now().timestamp(),
            },
        );
    }

    /// Add to blocklist
    pub async fn add_to_blocklist(&self, target: &str) {
        let mut blocklist = self.blocklist.write().await;
        if !blocklist.contains(&target.to_string()) {
            blocklist.push(target.to_string());
        }
    }

    /// Remove from blocklist
    pub async fn remove_from_blocklist(&self, target: &str) {
        let mut blocklist = self.blocklist.write().await;
        blocklist.retain(|x| x != target);
    }

    /// Get blocklist size
    pub async fn blocklist_size(&self) -> usize {
        self.blocklist.read().await.len()
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

/// Default blocklist (known malicious IPs/domains)
fn default_blocklist() -> Vec<String> {
    vec![
        // Common test/attack IPs (examples - not real threats)
        // In production, this would be loaded from threat feeds
    ]
}

/// Extract IPs and domains from a command
pub fn extract_targets(command: &str) -> Vec<String> {
    use regex::Regex;

    let mut targets = Vec::new();

    // IP address pattern
    if let Ok(ip_re) = Regex::new(r"\b(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})\b") {
        for cap in ip_re.captures_iter(command) {
            if let Some(ip) = cap.get(1) {
                // Validate it's a real IP
                if ip.as_str().parse::<IpAddr>().is_ok() {
                    targets.push(ip.as_str().to_string());
                }
            }
        }
    }

    // Domain pattern (simplified)
    if let Ok(domain_re) = Regex::new(r"(?:https?://)?([a-zA-Z0-9][-a-zA-Z0-9]*(?:\.[a-zA-Z0-9][-a-zA-Z0-9]*)+)") {
        for cap in domain_re.captures_iter(command) {
            if let Some(domain) = cap.get(1) {
                let d = domain.as_str().to_lowercase();
                // Skip common safe domains
                if !is_common_safe_domain(&d) {
                    targets.push(d);
                }
            }
        }
    }

    targets
}

/// Check if domain is commonly safe
fn is_common_safe_domain(domain: &str) -> bool {
    let safe_domains = [
        "github.com",
        "githubusercontent.com",
        "gitlab.com",
        "google.com",
        "googleapis.com",
        "microsoft.com",
        "azure.com",
        "amazonaws.com",
        "cloudflare.com",
        "npmjs.org",
        "pypi.org",
        "crates.io",
        "docker.io",
        "docker.com",
        "localhost",
        "127.0.0.1",
    ];

    safe_domains.iter().any(|safe| domain.ends_with(safe))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_creation() {
        let cache = ReputationCache::new(3600);
        assert_eq!(cache.blocklist_size().await, 0);
    }

    #[tokio::test]
    async fn test_blocklist_check() {
        let cache = ReputationCache::new(3600);
        cache.add_to_blocklist("evil.com").await;

        let result = cache.check("evil.com").await;
        assert!(result.is_malicious());
    }

    #[tokio::test]
    async fn test_unknown_target() {
        let cache = ReputationCache::new(3600);
        let result = cache.check("unknown-domain.com").await;
        assert_eq!(result, ReputationResult::Unknown);
    }

    #[test]
    fn test_extract_targets() {
        let targets = extract_targets("curl http://10.0.0.1:8080/api");
        assert!(targets.contains(&"10.0.0.1".to_string()));

        let targets = extract_targets("ssh user@suspicious-server.com");
        assert!(targets.contains(&"suspicious-server.com".to_string()));
    }

    #[test]
    fn test_safe_domain_filtering() {
        let targets = extract_targets("curl https://github.com/user/repo");
        // github.com should be filtered out as safe
        assert!(!targets.contains(&"github.com".to_string()));
    }
}
