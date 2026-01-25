//! Audit logging system
//!
//! Provides forensic-grade command logging with:
//! - Append-only log files
//! - SHA-256 hash chain verification
//! - Sensitive data redaction
//! - JSON structured logs for SIEM integration

mod event;
mod logger;
mod storage;

pub use event::{AuditEvent, AuditEventBuilder, CommandResult};
pub use logger::AuditLogger;
pub use storage::AuditStorage;
