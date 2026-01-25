//! Security plugins for MASTerm
//!
//! Provides 10 security-focused plugins:
//! 1. Secret Detection - Detect hardcoded secrets in commands
//! 2. Audit Logging - Forensic-grade command logging
//! 3. Privilege Escalation Alerts - Warn on sudo/su/doas usage
//! 4. Suspicious Pattern Detection - Detect reverse shells, encoded commands
//! 5. Network Monitor - Track outbound connections
//! 6. Package Audit - Warn on risky package installs
//! 7. File Integrity - Alert on sensitive file access
//! 8. SSH/GPG Monitor - Track key usage
//! 9. IP Reputation - Check against threat intelligence
//! 10. Sandbox Mode - Restricted execution environment

mod audit_log;
mod file_integrity;
mod ip_reputation;
mod network_monitor;
mod package_audit;
mod priv_escalation;
mod sandbox_mode;
mod secret_detection;
mod ssh_gpg_monitor;
mod suspicious_pattern;

pub use audit_log::AuditLogPlugin;
pub use file_integrity::FileIntegrityPlugin;
pub use ip_reputation::IpReputationPlugin;
pub use network_monitor::NetworkMonitorPlugin;
pub use package_audit::PackageAuditPlugin;
pub use priv_escalation::PrivEscalationPlugin;
pub use sandbox_mode::SandboxPlugin;
pub use secret_detection::SecretDetectionPlugin;
pub use ssh_gpg_monitor::SshGpgMonitorPlugin;
pub use suspicious_pattern::SuspiciousPatternPlugin;

/// Get all security plugins
pub fn security_plugins() -> Vec<Box<dyn masterm_core::plugin::Plugin + Send + Sync>> {
    vec![
        Box::new(SecretDetectionPlugin::new()),
        Box::new(AuditLogPlugin::new()),
        Box::new(PrivEscalationPlugin::new()),
        Box::new(SuspiciousPatternPlugin::new()),
        Box::new(NetworkMonitorPlugin::new()),
        Box::new(PackageAuditPlugin::new()),
        Box::new(FileIntegrityPlugin::new()),
        Box::new(SshGpgMonitorPlugin::new()),
        Box::new(IpReputationPlugin::new()),
        Box::new(SandboxPlugin::new()),
    ]
}
