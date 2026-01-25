//! Audit storage backend
//!
//! Provides append-only file storage for audit events.

use super::event::AuditEvent;
use super::logger::IntegrityReport;
use crate::SecurityResult;
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Audit storage backend
pub struct AuditStorage {
    /// Log file path
    path: PathBuf,
}

impl AuditStorage {
    /// Create a new storage backend
    pub async fn new(path: &Path) -> SecurityResult<Self> {
        // Ensure the file exists
        if !path.exists() {
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent).await?;
            }
            File::create(path).await?;
        }

        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    /// Append an event to the log
    pub async fn append(&self, event: &AuditEvent) -> SecurityResult<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;

        let json = serde_json::to_string(event)?;
        file.write_all(json.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.flush().await?;

        Ok(())
    }

    /// Get the last event info (ID and hash)
    pub async fn get_last_event_info(&self) -> SecurityResult<(u64, String)> {
        let file = File::open(&self.path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut last_id = 0u64;
        let mut last_hash = "0".repeat(64);

        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(event) = serde_json::from_str::<AuditEvent>(&line) {
                last_id = event.id;
                last_hash = event.hash;
            }
        }

        Ok((last_id, last_hash))
    }

    /// Get recent events
    pub async fn get_recent(&self, count: usize) -> SecurityResult<Vec<AuditEvent>> {
        let file = File::open(&self.path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut events = Vec::new();

        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(event) = serde_json::from_str::<AuditEvent>(&line) {
                events.push(event);
            }
        }

        // Return the last `count` events
        let start = events.len().saturating_sub(count);
        Ok(events[start..].to_vec())
    }

    /// Get all events
    pub async fn get_all(&self) -> SecurityResult<Vec<AuditEvent>> {
        let file = File::open(&self.path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        let mut events = Vec::new();

        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(event) = serde_json::from_str::<AuditEvent>(&line) {
                events.push(event);
            }
        }

        Ok(events)
    }

    /// Verify integrity of the log
    pub async fn verify_integrity(&self) -> SecurityResult<IntegrityReport> {
        let events = self.get_all().await?;

        let mut report = IntegrityReport {
            total_events: events.len() as u64,
            valid_events: 0,
            invalid_events: 0,
            chain_valid: true,
            first_invalid_id: None,
            errors: Vec::new(),
        };

        if events.is_empty() {
            return Ok(report);
        }

        let mut prev_hash = "0".repeat(64);

        for event in &events {
            // Verify hash
            if !event.verify_hash() {
                report.invalid_events += 1;
                if report.first_invalid_id.is_none() {
                    report.first_invalid_id = Some(event.id);
                }
                report
                    .errors
                    .push(format!("Event {}: Invalid hash", event.id));
            } else {
                report.valid_events += 1;
            }

            // Verify chain
            if event.prev_hash != prev_hash {
                report.chain_valid = false;
                report
                    .errors
                    .push(format!("Event {}: Chain broken", event.id));
            }

            prev_hash = event.hash.clone();
        }

        Ok(report)
    }

    /// Get log file size in bytes
    pub async fn size(&self) -> SecurityResult<u64> {
        let metadata = tokio::fs::metadata(&self.path).await?;
        Ok(metadata.len())
    }

    /// Check if rotation is needed
    pub async fn needs_rotation(&self, max_size_mb: u64) -> SecurityResult<bool> {
        let size = self.size().await?;
        Ok(size > max_size_mb * 1024 * 1024)
    }

    /// Rotate the log file
    pub async fn rotate(&self) -> SecurityResult<PathBuf> {
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let rotated_name = format!(
            "{}.{}",
            self.path.file_name().unwrap().to_string_lossy(),
            timestamp
        );
        let rotated_path = self.path.with_file_name(rotated_name);

        tokio::fs::rename(&self.path, &rotated_path).await?;
        File::create(&self.path).await?;

        Ok(rotated_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::AuditEventBuilder;

    #[tokio::test]
    async fn test_storage_append_and_read() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_audit.log");

        let storage = AuditStorage::new(&log_path).await.unwrap();

        // Append some events
        let event1 = AuditEvent::builder().command("ls -la").build(1);
        storage.append(&event1).await.unwrap();

        let event2 = AuditEvent::builder()
            .command("pwd")
            .prev_hash(&event1.hash)
            .build(2);
        storage.append(&event2).await.unwrap();

        // Read back
        let events = storage.get_all().await.unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].command, "ls -la");
        assert_eq!(events[1].command, "pwd");
    }

    #[tokio::test]
    async fn test_integrity_verification() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_audit.log");

        let storage = AuditStorage::new(&log_path).await.unwrap();

        // Build a valid chain
        let event1 = AuditEvent::builder().command("first").build(1);
        storage.append(&event1).await.unwrap();

        let event2 = AuditEvent::builder()
            .command("second")
            .prev_hash(&event1.hash)
            .build(2);
        storage.append(&event2).await.unwrap();

        let event3 = AuditEvent::builder()
            .command("third")
            .prev_hash(&event2.hash)
            .build(3);
        storage.append(&event3).await.unwrap();

        // Verify
        let report = storage.verify_integrity().await.unwrap();
        assert!(report.is_valid());
        assert_eq!(report.total_events, 3);
        assert_eq!(report.valid_events, 3);
        assert!(report.chain_valid);
    }

    #[tokio::test]
    async fn test_get_recent() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let log_path = temp_dir.path().join("test_audit.log");

        let storage = AuditStorage::new(&log_path).await.unwrap();

        // Add 5 events
        let mut prev_hash = "0".repeat(64);
        for i in 1..=5 {
            let event = AuditEvent::builder()
                .command(format!("command {}", i))
                .prev_hash(&prev_hash)
                .build(i);
            prev_hash = event.hash.clone();
            storage.append(&event).await.unwrap();
        }

        // Get last 2
        let recent = storage.get_recent(2).await.unwrap();
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].command, "command 4");
        assert_eq!(recent[1].command, "command 5");
    }
}
