//! IP/Domain reputation system
//!
//! Provides threat intelligence lookups:
//! - Local blocklist caching
//! - Optional external API integration (AbuseIPDB, VirusTotal)

mod cache;

pub use cache::{extract_targets, ReputationCache, ReputationResult, ThreatInfo};
