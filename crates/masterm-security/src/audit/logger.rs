//! Audit logger implementation
//!
//! Provides a high-level interface for audit logging.

use super::{AuditEvent, AuditStorage, CommandResult};
use crate::config::AuditConfig;
use crate::SecurityResult;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Audit logger
pub struct AuditLogger {
    /// Storage backend
    storage: Arc<RwLock<AuditStorage>>,

    /// Configuration
    config: AuditConfig,

    /// Event counter
    counter: AtomicU64,

    /// Last event hash (for chain)
    last_hash: Arc<RwLock<String>>,

    /// Commands to exclude from logging
    excluded_commands: Vec<String>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub async fn new(config: AuditConfig) -> SecurityResult<Self> {
        let log_path = config.log_path.clone().unwrap_or_else(default_log_path);

        // Ensure parent directory exists
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let storage = AuditStorage::new(&log_path).await?;

        // Get the last event to continue the hash chain
        let (counter, last_hash) = storage.get_last_event_info().await?;

        Ok(Self {
            storage: Arc::new(RwLock::new(storage)),
            config: config.clone(),
            counter: AtomicU64::new(counter),
            last_hash: Arc::new(RwLock::new(last_hash)),
            excluded_commands: config.exclude_commands,
        })
    }

    /// Log a command execution
    pub async fn log_command(
        &self,
        command: &str,
        cwd: &PathBuf,
        shell: &str,
        env_type: &str,
        security_flags: Vec<String>,
    ) -> SecurityResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        // Check if command should be excluded
        if self.should_exclude(command) {
            return Ok(());
        }

        let id = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
        let prev_hash = self.last_hash.read().await.clone();

        let event = AuditEvent::builder()
            .command(self.maybe_redact(command))
            .cwd(cwd.clone())
            .shell(shell)
            .env_type(env_type)
            .security_flags(security_flags)
            .prev_hash(prev_hash)
            .build(id);

        // Update last hash
        {
            let mut hash = self.last_hash.write().await;
            *hash = event.hash.clone();
        }

        // Write to storage
        let storage = self.storage.read().await;
        storage.append(&event).await?;

        Ok(())
    }

    /// Log a command with its result
    pub async fn log_command_with_result(
        &self,
        command: &str,
        cwd: &PathBuf,
        shell: &str,
        env_type: &str,
        security_flags: Vec<String>,
        result: CommandResult,
    ) -> SecurityResult<()> {
        if !self.config.enabled {
            return Ok(());
        }

        if self.should_exclude(command) {
            return Ok(());
        }

        let id = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
        let prev_hash = self.last_hash.read().await.clone();

        let event = AuditEvent::builder()
            .command(self.maybe_redact(command))
            .cwd(cwd.clone())
            .shell(shell)
            .env_type(env_type)
            .security_flags(security_flags)
            .result(result)
            .prev_hash(prev_hash)
            .build(id);

        // Update last hash
        {
            let mut hash = self.last_hash.write().await;
            *hash = event.hash.clone();
        }

        // Write to storage
        let storage = self.storage.read().await;
        storage.append(&event).await?;

        Ok(())
    }

    /// Get recent events
    pub async fn get_recent_events(&self, count: usize) -> SecurityResult<Vec<AuditEvent>> {
        let storage = self.storage.read().await;
        storage.get_recent(count).await
    }

    /// Verify the integrity of the audit log
    pub async fn verify_integrity(&self) -> SecurityResult<IntegrityReport> {
        let storage = self.storage.read().await;
        storage.verify_integrity().await
    }

    /// Should this command be excluded from logging?
    fn should_exclude(&self, command: &str) -> bool {
        let cmd_lower = command.to_lowercase();
        let first_word = cmd_lower.split_whitespace().next().unwrap_or("");

        self.excluded_commands
            .iter()
            .any(|exc| first_word == exc.to_lowercase())
    }

    /// Maybe redact secrets from command
    fn maybe_redact(&self, command: &str) -> String {
        if self.config.redact_secrets {
            redact_secrets(command)
        } else {
            command.to_string()
        }
    }
}

/// Integrity verification report
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    /// Total events checked
    pub total_events: u64,

    /// Number of valid events
    pub valid_events: u64,

    /// Number of invalid events
    pub invalid_events: u64,

    /// Chain is intact
    pub chain_valid: bool,

    /// First invalid event ID (if any)
    pub first_invalid_id: Option<u64>,

    /// Errors encountered
    pub errors: Vec<String>,
}

impl IntegrityReport {
    /// Is the log fully valid?
    pub fn is_valid(&self) -> bool {
        self.invalid_events == 0 && self.chain_valid && self.errors.is_empty()
    }
}

/// Get default log path
fn default_log_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".masterm")
        .join("security")
        .join("audit.log")
}

/// Redact secrets from command
fn redact_secrets(command: &str) -> String {
    use regex::Regex;

    let mut result = command.to_string();

    let patterns = [
        (r"(AKIA[A-Z0-9]{16})", "[AWS_KEY]"),
        (r"(gh[pous]_[A-Za-z0-9]{36})", "[GH_TOKEN]"),
        (r"(://[^:]+:)[^@]+(@)", "$1***$2"),
    ];

    for (pattern, replacement) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            result = re.replace_all(&result, replacement).to_string();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_logger_creation() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");

        let config = AuditConfig {
            enabled: true,
            log_path: Some(log_path),
            ..Default::default()
        };

        let logger = AuditLogger::new(config).await.unwrap();
        assert!(logger.config.enabled);
    }

    #[tokio::test]
    async fn test_command_exclusion() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");

        let config = AuditConfig {
            enabled: true,
            log_path: Some(log_path),
            exclude_commands: vec!["ls".to_string(), "cd".to_string()],
            ..Default::default()
        };

        let logger = AuditLogger::new(config).await.unwrap();

        assert!(logger.should_exclude("ls -la"));
        assert!(logger.should_exclude("cd /home"));
        assert!(!logger.should_exclude("rm -rf /tmp"));
    }
}
