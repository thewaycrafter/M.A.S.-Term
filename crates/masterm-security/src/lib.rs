//! MASTerm Security Module
//!
//! Provides comprehensive security features for the MASTerm terminal framework:
//! - Secret detection patterns (API keys, tokens, passwords)
//! - Threat pattern detection (reverse shells, encoded commands)
//! - Command audit logging with cryptographic verification
//! - IP/Domain reputation checking
//!
//! # Architecture
//!
//! ```text
//! masterm-security/
//! ├── patterns/      # Detection patterns
//! │   ├── secrets    # Secret/credential patterns
//! │   ├── threats    # Malicious command patterns
//! │   └── privilege  # Privilege escalation patterns
//! ├── audit/         # Audit logging
//! │   ├── logger     # Append-only logger
//! │   ├── event      # Audit event types
//! │   └── storage    # Secure storage backend
//! ├── reputation/    # Threat intelligence
//! │   ├── cache      # Local blocklist cache
//! │   └── providers  # External threat feeds
//! └── config         # Security configuration
//! ```

pub mod audit;
pub mod config;
pub mod patterns;
pub mod reputation;

pub use audit::{AuditEvent, AuditLogger, AuditStorage};
pub use config::SecurityConfig;
pub use patterns::{
    PrivilegePattern, SecretMatch, SecretPattern, SecurityPatternMatcher, ThreatMatch,
    ThreatPattern,
};
pub use reputation::{ReputationCache, ReputationResult};

/// Result type for security operations
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Security operation errors
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Pattern compilation error: {0}")]
    PatternError(String),

    #[error("Audit log error: {0}")]
    AuditError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Reputation lookup error: {0}")]
    ReputationError(String),
}
