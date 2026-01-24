//! Caching layer for expensive computations

use anyhow::Result;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Cache store for MASTerm
pub struct CacheStore {
    /// In-memory cache
    memory: HashMap<String, CacheEntry>,

    /// Cache directory
    cache_dir: PathBuf,

    /// Default TTL
    default_ttl: Duration,

    /// Max memory entries
    max_entries: usize,
}

/// A cached entry
struct CacheEntry {
    /// Cached value
    value: String,

    /// When entry was created
    created_at: Instant,

    /// Time to live
    ttl: Duration,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

impl CacheStore {
    /// Create a new cache store
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("masterm");

        Self {
            memory: HashMap::new(),
            cache_dir,
            default_ttl: Duration::from_secs(300),
            max_entries: 1000,
        }
    }

    /// Create with custom cache directory
    pub fn with_dir(mut self, dir: PathBuf) -> Self {
        self.cache_dir = dir;
        self
    }

    /// Create with custom TTL
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.default_ttl = ttl;
        self
    }

    /// Get a cached value
    pub fn get(&self, key: &str) -> Option<&str> {
        self.memory.get(key).and_then(|entry| {
            if entry.is_expired() {
                None
            } else {
                Some(entry.value.as_str())
            }
        })
    }

    /// Set a cached value with default TTL
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.set_with_ttl(key, value, self.default_ttl);
    }

    /// Set a cached value with custom TTL
    pub fn set_with_ttl(&mut self, key: impl Into<String>, value: impl Into<String>, ttl: Duration) {
        // Evict old entries if needed
        if self.memory.len() >= self.max_entries {
            self.evict_expired();
        }

        self.memory.insert(
            key.into(),
            CacheEntry {
                value: value.into(),
                created_at: Instant::now(),
                ttl,
            },
        );
    }

    /// Remove a cached value
    pub fn remove(&mut self, key: &str) {
        self.memory.remove(key);
    }

    /// Clear all cached values
    pub fn clear(&mut self) {
        self.memory.clear();
    }

    /// Evict expired entries
    pub fn evict_expired(&mut self) {
        self.memory.retain(|_, entry| !entry.is_expired());
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total = self.memory.len();
        let expired = self.memory.values().filter(|e| e.is_expired()).count();

        CacheStats {
            total_entries: total,
            expired_entries: expired,
            active_entries: total - expired,
        }
    }

    /// Get or compute a value
    pub fn get_or_compute<F>(&mut self, key: &str, compute: F) -> Result<String>
    where
        F: FnOnce() -> Result<String>,
    {
        if let Some(value) = self.get(key) {
            return Ok(value.to_string());
        }

        let value = compute()?;
        self.set(key.to_string(), value.clone());
        Ok(value)
    }
}

impl Default for CacheStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of entries
    pub total_entries: usize,

    /// Number of expired entries
    pub expired_entries: usize,

    /// Number of active (non-expired) entries
    pub active_entries: usize,
}
