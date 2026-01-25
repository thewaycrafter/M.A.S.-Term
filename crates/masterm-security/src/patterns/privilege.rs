//! Privilege escalation pattern detection
//!
//! Detects commands that escalate privileges:
//! - sudo, su, doas, pkexec
//! - setuid/setgid modifications
//! - Capability modifications

use super::RiskLevel;
use crate::{SecurityError, SecurityResult};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// A privilege escalation pattern
#[derive(Debug, Clone)]
pub struct PrivilegePattern {
    /// Pattern name
    pub name: String,

    /// Pattern description
    pub description: String,

    /// Compiled regex pattern
    pub regex: Regex,

    /// Risk level
    pub risk_level: RiskLevel,

    /// Type of privilege escalation
    pub priv_type: PrivilegeType,
}

impl PrivilegePattern {
    /// Create a new privilege pattern
    pub fn new(
        name: &str,
        description: &str,
        pattern: &str,
        risk_level: RiskLevel,
        priv_type: PrivilegeType,
    ) -> SecurityResult<Self> {
        let regex = Regex::new(pattern)
            .map_err(|e| SecurityError::PatternError(format!("{}: {}", name, e)))?;

        Ok(Self {
            name: name.to_string(),
            description: description.to_string(),
            regex,
            risk_level,
            priv_type,
        })
    }
}

/// Type of privilege escalation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivilegeType {
    /// sudo command
    Sudo,
    /// su command
    Su,
    /// doas (OpenBSD sudo alternative)
    Doas,
    /// pkexec (PolicyKit)
    Pkexec,
    /// chmod with dangerous permissions
    ChmodDangerous,
    /// chown to root
    ChownRoot,
    /// setcap (capabilities)
    Setcap,
    /// setuid/setgid bit
    SetuidSetgid,
    /// Other privilege escalation
    Other,
}

impl PrivilegeType {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Sudo => "sudo",
            Self::Su => "su",
            Self::Doas => "doas",
            Self::Pkexec => "pkexec",
            Self::ChmodDangerous => "chmod (dangerous)",
            Self::ChownRoot => "chown root",
            Self::Setcap => "setcap",
            Self::SetuidSetgid => "setuid/setgid",
            Self::Other => "privilege escalation",
        }
    }

    /// Get icon
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Sudo | Self::Su | Self::Doas | Self::Pkexec => "👑",
            Self::ChmodDangerous => "🔓",
            Self::ChownRoot => "🔐",
            Self::Setcap => "⚡",
            Self::SetuidSetgid => "⚠️",
            Self::Other => "🔒",
        }
    }

    /// Is this a direct root escalation?
    pub fn is_root_escalation(&self) -> bool {
        matches!(
            self,
            Self::Sudo | Self::Su | Self::Doas | Self::Pkexec | Self::ChownRoot
        )
    }
}

/// A detected privilege escalation
#[derive(Debug, Clone)]
pub struct PrivilegeMatch {
    /// Pattern that matched
    pub pattern_name: String,

    /// Pattern description
    pub description: String,

    /// The command that triggered the match
    pub command: String,

    /// Risk level
    pub risk_level: RiskLevel,

    /// Type of escalation
    pub priv_type: PrivilegeType,

    /// Warning message
    pub warning: String,
}

/// Privilege escalation pattern matcher
pub struct PrivilegePatternMatcher {
    patterns: Vec<PrivilegePattern>,
}

impl PrivilegePatternMatcher {
    /// Create a new matcher with default patterns
    pub fn new() -> Self {
        Self {
            patterns: default_patterns(),
        }
    }

    /// Add a custom pattern
    pub fn add_pattern(&mut self, pattern: PrivilegePattern) -> SecurityResult<()> {
        self.patterns.push(pattern);
        Ok(())
    }

    /// Check if command contains privilege escalation
    pub fn check(&self, input: &str) -> Option<PrivilegeMatch> {
        for pattern in &self.patterns {
            if pattern.regex.is_match(input) {
                return Some(PrivilegeMatch {
                    pattern_name: pattern.name.clone(),
                    description: pattern.description.clone(),
                    command: input.to_string(),
                    risk_level: pattern.risk_level,
                    priv_type: pattern.priv_type,
                    warning: get_warning(pattern.priv_type),
                });
            }
        }
        None
    }

    /// Check if command uses sudo/su/doas
    pub fn uses_root_escalation(&self, input: &str) -> bool {
        self.check(input)
            .map(|m| m.priv_type.is_root_escalation())
            .unwrap_or(false)
    }
}

impl Default for PrivilegePatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Get warning message for privilege type
fn get_warning(priv_type: PrivilegeType) -> String {
    match priv_type {
        PrivilegeType::Sudo => {
            "This command will execute with superuser (root) privileges via sudo."
        }
        PrivilegeType::Su => {
            "This command will switch to another user account (typically root)."
        }
        PrivilegeType::Doas => {
            "This command will execute with elevated privileges via doas."
        }
        PrivilegeType::Pkexec => {
            "This command will execute with elevated privileges via PolicyKit."
        }
        PrivilegeType::ChmodDangerous => {
            "This command sets dangerous file permissions (world-writable or setuid)."
        }
        PrivilegeType::ChownRoot => {
            "This command changes file ownership to root."
        }
        PrivilegeType::Setcap => {
            "This command modifies file capabilities, which can grant special privileges."
        }
        PrivilegeType::SetuidSetgid => {
            "This command sets the setuid or setgid bit, allowing privilege escalation."
        }
        PrivilegeType::Other => {
            "This command involves privilege escalation."
        }
    }
    .to_string()
}

/// Default privilege patterns
fn default_patterns() -> Vec<PrivilegePattern> {
    let mut patterns = Vec::new();

    // sudo command
    if let Ok(p) = PrivilegePattern::new(
        "sudo",
        "Execute command as superuser",
        r"^\s*sudo\s+",
        RiskLevel::High,
        PrivilegeType::Sudo,
    ) {
        patterns.push(p);
    }

    // Also match sudo in pipes
    if let Ok(p) = PrivilegePattern::new(
        "sudo_pipe",
        "Pipe to sudo",
        r"\|\s*sudo\s+",
        RiskLevel::High,
        PrivilegeType::Sudo,
    ) {
        patterns.push(p);
    }

    // su command
    if let Ok(p) = PrivilegePattern::new(
        "su",
        "Switch user",
        r"^\s*su\s+(?:-\s+)?(?:root)?",
        RiskLevel::High,
        PrivilegeType::Su,
    ) {
        patterns.push(p);
    }

    // su with command
    if let Ok(p) = PrivilegePattern::new(
        "su_command",
        "Switch user and execute command",
        r"^\s*su\s+-c\s+",
        RiskLevel::High,
        PrivilegeType::Su,
    ) {
        patterns.push(p);
    }

    // doas command (OpenBSD)
    if let Ok(p) = PrivilegePattern::new(
        "doas",
        "Execute command as another user (doas)",
        r"^\s*doas\s+",
        RiskLevel::High,
        PrivilegeType::Doas,
    ) {
        patterns.push(p);
    }

    // pkexec command
    if let Ok(p) = PrivilegePattern::new(
        "pkexec",
        "Execute command with PolicyKit",
        r"^\s*pkexec\s+",
        RiskLevel::High,
        PrivilegeType::Pkexec,
    ) {
        patterns.push(p);
    }

    // chmod 777 (world-writable)
    if let Ok(p) = PrivilegePattern::new(
        "chmod_777",
        "Set world-writable permissions",
        r"\bchmod\s+(?:-[a-zA-Z]+\s+)*777\b",
        RiskLevel::Medium,
        PrivilegeType::ChmodDangerous,
    ) {
        patterns.push(p);
    }

    // chmod with setuid bit
    if let Ok(p) = PrivilegePattern::new(
        "chmod_setuid",
        "Set setuid bit",
        r"\bchmod\s+(?:-[a-zA-Z]+\s+)*[u+s24][0-7]{3}\b",
        RiskLevel::High,
        PrivilegeType::ChmodDangerous,
    ) {
        patterns.push(p);
    }

    // chmod u+s or g+s
    if let Ok(p) = PrivilegePattern::new(
        "chmod_symbolic_setuid",
        "Set setuid/setgid bit (symbolic)",
        r"\bchmod\s+(?:-[a-zA-Z]+\s+)*[ug]\+s\b",
        RiskLevel::High,
        PrivilegeType::SetuidSetgid,
    ) {
        patterns.push(p);
    }

    // chown to root
    if let Ok(p) = PrivilegePattern::new(
        "chown_root",
        "Change owner to root",
        r"\bchown\s+(?:-[a-zA-Z]+\s+)*root\b",
        RiskLevel::Medium,
        PrivilegeType::ChownRoot,
    ) {
        patterns.push(p);
    }

    // setcap command
    if let Ok(p) = PrivilegePattern::new(
        "setcap",
        "Set file capabilities",
        r"\bsetcap\s+",
        RiskLevel::High,
        PrivilegeType::Setcap,
    ) {
        patterns.push(p);
    }

    patterns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sudo_detection() {
        let matcher = PrivilegePatternMatcher::new();

        let result = matcher.check("sudo rm -rf /var/log/app");
        assert!(result.is_some());
        assert_eq!(result.unwrap().priv_type, PrivilegeType::Sudo);

        let result = matcher.check("cat file | sudo tee /etc/config");
        assert!(result.is_some());
    }

    #[test]
    fn test_su_detection() {
        let matcher = PrivilegePatternMatcher::new();

        let result = matcher.check("su - root");
        assert!(result.is_some());
        assert_eq!(result.unwrap().priv_type, PrivilegeType::Su);

        let result = matcher.check("su -c 'whoami'");
        assert!(result.is_some());
    }

    #[test]
    fn test_chmod_detection() {
        let matcher = PrivilegePatternMatcher::new();

        let result = matcher.check("chmod 777 /tmp/script.sh");
        assert!(result.is_some());
        assert_eq!(result.unwrap().priv_type, PrivilegeType::ChmodDangerous);

        let result = matcher.check("chmod u+s /usr/bin/program");
        assert!(result.is_some());
    }

    #[test]
    fn test_chown_root() {
        let matcher = PrivilegePatternMatcher::new();

        let result = matcher.check("chown root:root /etc/config");
        assert!(result.is_some());
        assert_eq!(result.unwrap().priv_type, PrivilegeType::ChownRoot);
    }

    #[test]
    fn test_no_false_positives() {
        let matcher = PrivilegePatternMatcher::new();

        // Normal commands should not trigger
        assert!(matcher.check("ls -la").is_none());
        assert!(matcher.check("chmod 755 script.sh").is_none());
        assert!(matcher.check("chown user:user file.txt").is_none());
    }

    #[test]
    fn test_root_escalation_check() {
        let matcher = PrivilegePatternMatcher::new();

        assert!(matcher.uses_root_escalation("sudo apt update"));
        assert!(matcher.uses_root_escalation("su -"));
        assert!(!matcher.uses_root_escalation("chmod 777 file"));
        assert!(!matcher.uses_root_escalation("ls -la"));
    }
}
