//! Cryptographic material usage monitoring
//!
//! Tracks usage of SSH and GPG keys.

use std::path::PathBuf;

/// Key usage event
#[derive(Debug, Clone)]
pub struct KeyUsageEvent {
    pub key_type: KeyType,
    pub operation: KeyOperation,
    pub key_path: Option<PathBuf>,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyType {
    Ssh,
    Gpg,
    Ssl,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyOperation {
    Sign,
    Decrypt,
    AddIdentity,
    Generate,
    List,
    Unknown,
}

/// Crypto material monitor
pub struct CryptoMonitor;

impl CryptoMonitor {
    /// Analyze command for crypto usage
    pub fn analyze(command: &str) -> Option<KeyUsageEvent> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0] {
            "ssh" | "ssh-add" | "ssh-keygen" => Self::analyze_ssh(parts[0], &parts[1..], command),
            "gpg" | "gpg2" => Self::analyze_gpg(parts[0], &parts[1..], command),
            _ => None,
        }
    }

    fn analyze_ssh(cmd: &str, _args: &[&str], full_cmd: &str) -> Option<KeyUsageEvent> {
        let operation = match cmd {
            "ssh-add" => KeyOperation::AddIdentity,
            "ssh-keygen" => KeyOperation::Generate,
            "ssh" => KeyOperation::Sign, // Implicitly uses key
            _ => KeyOperation::Unknown,
        };

        Some(KeyUsageEvent {
            key_type: KeyType::Ssh,
            operation,
            key_path: None, // Hard to extract without parsing all args
            command: full_cmd.to_string(),
        })
    }

    fn analyze_gpg(_cmd: &str, args: &[&str], full_cmd: &str) -> Option<KeyUsageEvent> {
        let mut operation = KeyOperation::Unknown;

        for arg in args {
            match *arg {
                "--sign" | "-s" | "--clearsign" => operation = KeyOperation::Sign,
                "--decrypt" | "-d" => operation = KeyOperation::Decrypt,
                "--gen-key" => operation = KeyOperation::Generate,
                "--list-keys" | "-k" => operation = KeyOperation::List,
                _ => {}
            }
        }

        if operation != KeyOperation::Unknown {
            Some(KeyUsageEvent {
                key_type: KeyType::Gpg,
                key_path: None,
                operation,
                command: full_cmd.to_string(),
            })
        } else {
            None
        }
    }
}
