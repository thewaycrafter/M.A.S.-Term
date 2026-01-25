//! File integrity monitoring
//!
//! Monitors critical files for changes.

use crate::SecurityResult;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// File integrity monitor
pub struct IntegrityMonitor {
    /// Monitored files and their hashes
    hashes: HashMap<PathBuf, String>,
}

impl IntegrityMonitor {
    /// Create a new integrity monitor
    pub fn new() -> Self {
        Self {
            hashes: HashMap::new(),
        }
    }

    /// Add a file to monitor
    pub fn add_file(&mut self, path: impl Into<PathBuf>) -> SecurityResult<()> {
        let path = path.into();
        if path.exists() {
            let hash = self.calculate_hash(&path)?;
            self.hashes.insert(path, hash);
        }
        Ok(())
    }

    /// Check integrity of all monitored files
    pub fn check_integrity(&self) -> SecurityResult<IntegrityCheckResult> {
        let mut violations = Vec::new();
        let mut missing_files = Vec::new();

        for (path, stored_hash) in &self.hashes {
            if !path.exists() {
                missing_files.push(path.clone());
                continue;
            }

            let current_hash = self.calculate_hash(path)?;
            if *stored_hash != current_hash {
                violations.push((path.clone(), stored_hash.clone(), current_hash));
            }
        }

        Ok(IntegrityCheckResult {
            violations,
            missing_files,
            total_checked: self.hashes.len(),
        })
    }

    /// Calculate SHA-256 hash of a file
    fn calculate_hash(&self, path: &Path) -> SecurityResult<String> {
        let mut file = fs::File::open(path)?;
        let mut hasher = Sha256::new();
        std::io::copy(&mut file, &mut hasher)?;
        Ok(hex::encode(hasher.finalize()))
    }
}

impl Default for IntegrityMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of integrity check
#[derive(Debug, Clone)]
pub struct IntegrityCheckResult {
    /// Files with changed hashes: (Path, OldHash, NewHash)
    pub violations: Vec<(PathBuf, String, String)>,

    /// Files that went missing
    pub missing_files: Vec<PathBuf>,

    /// Total files checked
    pub total_checked: usize,
}
