//! Audit event types
//!
//! Defines the structure of audit log entries.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// An audit log event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Event ID (incrementing counter)
    pub id: u64,

    /// Event timestamp (UTC)
    pub timestamp: DateTime<Utc>,

    /// Command that was executed
    pub command: String,

    /// Current working directory
    pub cwd: PathBuf,

    /// Username
    pub user: String,

    /// Shell type (bash, zsh, etc.)
    pub shell: String,

    /// Process ID
    pub pid: u32,

    /// Parent process ID
    pub ppid: Option<u32>,

    /// Command result (if available)
    pub result: Option<CommandResult>,

    /// Environment type (dev/staging/prod)
    pub env_type: String,

    /// Security flags triggered
    pub security_flags: Vec<String>,

    /// SHA-256 hash of this event
    pub hash: String,

    /// SHA-256 hash of previous event (for chain verification)
    pub prev_hash: String,
}

/// Command execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    /// Exit code
    pub exit_code: i32,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Whether command was blocked by security
    pub blocked: bool,

    /// Whether command required confirmation
    pub required_confirmation: bool,
}

impl AuditEvent {
    /// Create a new audit event builder
    pub fn builder() -> AuditEventBuilder {
        AuditEventBuilder::default()
    }

    /// Calculate hash for this event (excluding the hash fields)
    pub fn calculate_hash(&self) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        // Hash all fields except hash and prev_hash
        hasher.update(self.id.to_le_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        hasher.update(self.command.as_bytes());
        hasher.update(self.cwd.to_string_lossy().as_bytes());
        hasher.update(self.user.as_bytes());
        hasher.update(self.shell.as_bytes());
        hasher.update(self.pid.to_le_bytes());
        if let Some(ppid) = self.ppid {
            hasher.update(ppid.to_le_bytes());
        }
        if let Some(ref result) = self.result {
            hasher.update(result.exit_code.to_le_bytes());
            hasher.update(result.duration_ms.to_le_bytes());
        }
        hasher.update(self.env_type.as_bytes());
        for flag in &self.security_flags {
            hasher.update(flag.as_bytes());
        }
        hasher.update(self.prev_hash.as_bytes());

        hex::encode(hasher.finalize())
    }

    /// Verify that the hash is correct
    pub fn verify_hash(&self) -> bool {
        self.hash == self.calculate_hash()
    }

    /// Get a redacted version of the command (for display)
    pub fn redacted_command(&self) -> String {
        redact_secrets(&self.command)
    }
}

/// Builder for creating audit events
#[derive(Debug, Default)]
pub struct AuditEventBuilder {
    command: Option<String>,
    cwd: Option<PathBuf>,
    user: Option<String>,
    shell: Option<String>,
    pid: Option<u32>,
    ppid: Option<u32>,
    result: Option<CommandResult>,
    env_type: Option<String>,
    security_flags: Vec<String>,
    prev_hash: Option<String>,
}

impl AuditEventBuilder {
    /// Set the command
    pub fn command(mut self, command: impl Into<String>) -> Self {
        self.command = Some(command.into());
        self
    }

    /// Set the current working directory
    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Set the username
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Set the shell type
    pub fn shell(mut self, shell: impl Into<String>) -> Self {
        self.shell = Some(shell.into());
        self
    }

    /// Set the process ID
    pub fn pid(mut self, pid: u32) -> Self {
        self.pid = Some(pid);
        self
    }

    /// Set the parent process ID
    pub fn ppid(mut self, ppid: u32) -> Self {
        self.ppid = Some(ppid);
        self
    }

    /// Set the command result
    pub fn result(mut self, result: CommandResult) -> Self {
        self.result = Some(result);
        self
    }

    /// Set the environment type
    pub fn env_type(mut self, env_type: impl Into<String>) -> Self {
        self.env_type = Some(env_type.into());
        self
    }

    /// Add a security flag
    pub fn add_security_flag(mut self, flag: impl Into<String>) -> Self {
        self.security_flags.push(flag.into());
        self
    }

    /// Set multiple security flags
    pub fn security_flags(mut self, flags: Vec<String>) -> Self {
        self.security_flags = flags;
        self
    }

    /// Set the previous event hash
    pub fn prev_hash(mut self, hash: impl Into<String>) -> Self {
        self.prev_hash = Some(hash.into());
        self
    }

    /// Build the audit event
    pub fn build(self, id: u64) -> AuditEvent {
        let prev_hash = self.prev_hash.unwrap_or_else(|| "0".repeat(64));

        let mut event = AuditEvent {
            id,
            timestamp: Utc::now(),
            command: self.command.unwrap_or_default(),
            cwd: self.cwd.unwrap_or_else(|| PathBuf::from("/")),
            user: self.user.unwrap_or_else(|| whoami()),
            shell: self.shell.unwrap_or_else(|| "unknown".to_string()),
            pid: self.pid.unwrap_or_else(|| std::process::id()),
            ppid: self.ppid,
            result: self.result,
            env_type: self.env_type.unwrap_or_else(|| "unknown".to_string()),
            security_flags: self.security_flags,
            hash: String::new(),
            prev_hash,
        };

        // Calculate and set the hash
        event.hash = event.calculate_hash();
        event
    }
}

/// Get current username
fn whoami() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

/// Redact secrets from a command string
fn redact_secrets(command: &str) -> String {
    use regex::Regex;

    let mut result = command.to_string();

    // Patterns to redact
    let patterns = [
        // AWS Access Key
        (r"(AKIA[A-Z0-9]{16})", "[AWS_KEY_REDACTED]"),
        // GitHub tokens
        (r"(gh[pous]_[A-Za-z0-9]{36})", "[GITHUB_TOKEN_REDACTED]"),
        // Generic API keys
        (r#"(?i)(api[_-]?key[=:]\s*)['"]?([A-Za-z0-9\-_]{20,})['"]?"#, "$1[REDACTED]"),
        // Passwords in URLs
        (r"(://[^:]+:)[^@]+(@)", "$1[REDACTED]$2"),
        // Generic secrets
        (r#"(?i)(secret[=:]\s*)['"]?([A-Za-z0-9\-_]{20,})['"]?"#, "$1[REDACTED]"),
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

    #[test]
    fn test_audit_event_builder() {
        let event = AuditEvent::builder()
            .command("ls -la")
            .cwd("/home/user")
            .user("testuser")
            .shell("zsh")
            .pid(12345)
            .env_type("dev")
            .build(1);

        assert_eq!(event.id, 1);
        assert_eq!(event.command, "ls -la");
        assert_eq!(event.user, "testuser");
        assert!(!event.hash.is_empty());
    }

    #[test]
    fn test_hash_verification() {
        let event = AuditEvent::builder()
            .command("test command")
            .build(1);

        assert!(event.verify_hash());
    }

    #[test]
    fn test_hash_chain() {
        let event1 = AuditEvent::builder()
            .command("first command")
            .build(1);

        let event2 = AuditEvent::builder()
            .command("second command")
            .prev_hash(&event1.hash)
            .build(2);

        assert_eq!(event2.prev_hash, event1.hash);
        assert!(event2.verify_hash());
    }

    #[test]
    fn test_secret_redaction() {
        let redacted = redact_secrets("git clone https://ghp_aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789@github.com/user/repo");
        assert!(redacted.contains("[GITHUB_TOKEN_REDACTED]"));
        assert!(!redacted.contains("ghp_"));

        let redacted = redact_secrets("curl https://user:password123@api.example.com");
        assert!(redacted.contains("[REDACTED]"));
        assert!(!redacted.contains("password123"));
    }
}
