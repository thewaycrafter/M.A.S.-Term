//! Secret detection patterns
//!
//! Detects hardcoded secrets, API keys, tokens, and passwords in commands.
//!
//! # Supported Secret Types
//!
//! - AWS Access Keys and Secret Keys
//! - GitHub Personal Access Tokens
//! - GitLab Tokens
//! - Slack Tokens
//! - Stripe API Keys
//! - Google Cloud API Keys
//! - Azure Connection Strings
//! - Generic API Keys and Passwords
//! - Private Keys (RSA, DSA, EC)
//! - JWT Tokens

use super::RiskLevel;
use crate::{SecurityError, SecurityResult};
use regex::Regex;
use serde::{Deserialize, Serialize};

/// A secret detection pattern
#[derive(Debug, Clone)]
pub struct SecretPattern {
    /// Pattern name
    pub name: String,

    /// Pattern description
    pub description: String,

    /// Compiled regex pattern
    pub regex: Regex,

    /// Risk level if matched
    pub risk_level: RiskLevel,

    /// Secret type category
    pub category: SecretCategory,
}

impl SecretPattern {
    /// Create a new secret pattern
    pub fn new(
        name: &str,
        description: &str,
        pattern: &str,
        risk_level: RiskLevel,
        category: SecretCategory,
    ) -> SecurityResult<Self> {
        let regex = Regex::new(pattern)
            .map_err(|e| SecurityError::PatternError(format!("{}: {}", name, e)))?;

        Ok(Self {
            name: name.to_string(),
            description: description.to_string(),
            regex,
            risk_level,
            category,
        })
    }
}

/// Secret category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecretCategory {
    AwsCredentials,
    GitHubToken,
    GitLabToken,
    SlackToken,
    StripeKey,
    GoogleCloud,
    Azure,
    PrivateKey,
    JwtToken,
    GenericApiKey,
    GenericPassword,
    DatabaseUrl,
    Other,
}

impl SecretCategory {
    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::AwsCredentials => "AWS Credentials",
            Self::GitHubToken => "GitHub Token",
            Self::GitLabToken => "GitLab Token",
            Self::SlackToken => "Slack Token",
            Self::StripeKey => "Stripe API Key",
            Self::GoogleCloud => "Google Cloud",
            Self::Azure => "Azure Credentials",
            Self::PrivateKey => "Private Key",
            Self::JwtToken => "JWT Token",
            Self::GenericApiKey => "API Key",
            Self::GenericPassword => "Password",
            Self::DatabaseUrl => "Database URL",
            Self::Other => "Secret",
        }
    }

    /// Get icon
    pub fn icon(&self) -> &'static str {
        match self {
            Self::AwsCredentials => "☁️",
            Self::GitHubToken => "🐙",
            Self::GitLabToken => "🦊",
            Self::SlackToken => "💬",
            Self::StripeKey => "💳",
            Self::GoogleCloud => "🌐",
            Self::Azure => "☁️",
            Self::PrivateKey => "🔑",
            Self::JwtToken => "🎫",
            Self::GenericApiKey => "🔐",
            Self::GenericPassword => "🔒",
            Self::DatabaseUrl => "🗄️",
            Self::Other => "🔐",
        }
    }
}

/// A detected secret match
#[derive(Debug, Clone)]
pub struct SecretMatch {
    /// Pattern that matched
    pub pattern_name: String,

    /// Matched text (redacted)
    pub matched_text: String,

    /// Original matched text (for internal use)
    pub original_text: String,

    /// Position in input
    pub position: (usize, usize),

    /// Risk level
    pub risk_level: RiskLevel,

    /// Secret category
    pub category: SecretCategory,

    /// Remediation advice
    pub advice: String,
}

impl SecretMatch {
    /// Get redacted version of the matched text
    pub fn redacted(&self) -> String {
        if self.original_text.len() <= 8 {
            return "*".repeat(self.original_text.len());
        }

        let prefix = &self.original_text[..4];
        let suffix = &self.original_text[self.original_text.len() - 4..];
        format!("{}...{}", prefix, suffix)
    }
}

/// Secret pattern matcher
pub struct SecretPatternMatcher {
    patterns: Vec<SecretPattern>,
}

impl SecretPatternMatcher {
    /// Create a new matcher with default patterns
    pub fn new() -> Self {
        Self {
            patterns: default_patterns(),
        }
    }

    /// Add a custom pattern
    pub fn add_pattern(&mut self, pattern: SecretPattern) -> SecurityResult<()> {
        self.patterns.push(pattern);
        Ok(())
    }

    /// Find all secrets in input
    pub fn find_all(&self, input: &str) -> Vec<SecretMatch> {
        let mut matches = Vec::new();

        for pattern in &self.patterns {
            for capture in pattern.regex.find_iter(input) {
                let original = capture.as_str().to_string();
                matches.push(SecretMatch {
                    pattern_name: pattern.name.clone(),
                    matched_text: redact_secret(&original),
                    original_text: original,
                    position: (capture.start(), capture.end()),
                    risk_level: pattern.risk_level,
                    category: pattern.category,
                    advice: get_advice(pattern.category),
                });
            }
        }

        matches
    }

    /// Check if input contains any secrets
    pub fn contains_secrets(&self, input: &str) -> bool {
        self.patterns.iter().any(|p| p.regex.is_match(input))
    }
}

impl Default for SecretPatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Redact a secret for safe display
fn redact_secret(secret: &str) -> String {
    if secret.len() <= 8 {
        return "*".repeat(secret.len());
    }

    let prefix = &secret[..4];
    let stars = "*".repeat(secret.len().saturating_sub(8).min(20));
    let suffix = &secret[secret.len().saturating_sub(4)..];
    format!("{}{}{}", prefix, stars, suffix)
}

/// Get remediation advice for a secret category
fn get_advice(category: SecretCategory) -> String {
    match category {
        SecretCategory::AwsCredentials => {
            "Use AWS IAM roles, environment variables, or AWS credentials file instead."
        }
        SecretCategory::GitHubToken => {
            "Use GitHub CLI authentication (gh auth login) or environment variables."
        }
        SecretCategory::PrivateKey => "Never expose private keys in commands. Use ssh-agent.",
        SecretCategory::DatabaseUrl => {
            "Use environment variables or a secrets manager for database credentials."
        }
        SecretCategory::JwtToken => {
            "Store JWT tokens in secure storage, not in command line arguments."
        }
        _ => "Use environment variables or a secrets manager instead of hardcoding secrets.",
    }
    .to_string()
}

/// Default secret patterns
fn default_patterns() -> Vec<SecretPattern> {
    let mut patterns = Vec::new();

    // AWS Access Key ID
    if let Ok(p) = SecretPattern::new(
        "aws_access_key",
        "AWS Access Key ID",
        r"(?i)(AKIA|A3T[A-Z0-9]|ABIA|ACCA|ASIA)[A-Z0-9]{16}",
        RiskLevel::Critical,
        SecretCategory::AwsCredentials,
    ) {
        patterns.push(p);
    }

    // AWS Secret Access Key
    if let Ok(p) = SecretPattern::new(
        "aws_secret_key",
        "AWS Secret Access Key",
        r#"(?i)aws[_\-]?secret[_\-]?access[_\-]?key['"]?\s*[:=]\s*['"]?([A-Za-z0-9/+=]{40})"#,
        RiskLevel::Critical,
        SecretCategory::AwsCredentials,
    ) {
        patterns.push(p);
    }

    // GitHub Personal Access Token (classic)
    if let Ok(p) = SecretPattern::new(
        "github_pat_classic",
        "GitHub Personal Access Token (Classic)",
        r"ghp_[A-Za-z0-9]{36}",
        RiskLevel::High,
        SecretCategory::GitHubToken,
    ) {
        patterns.push(p);
    }

    // GitHub Personal Access Token (fine-grained)
    if let Ok(p) = SecretPattern::new(
        "github_pat_fine",
        "GitHub Personal Access Token (Fine-grained)",
        r"github_pat_[A-Za-z0-9]{22}_[A-Za-z0-9]{59}",
        RiskLevel::High,
        SecretCategory::GitHubToken,
    ) {
        patterns.push(p);
    }

    // GitHub OAuth Access Token
    if let Ok(p) = SecretPattern::new(
        "github_oauth",
        "GitHub OAuth Access Token",
        r"gho_[A-Za-z0-9]{36}",
        RiskLevel::High,
        SecretCategory::GitHubToken,
    ) {
        patterns.push(p);
    }

    // GitHub App Token
    if let Ok(p) = SecretPattern::new(
        "github_app",
        "GitHub App Token",
        r"(ghu|ghs)_[A-Za-z0-9]{36}",
        RiskLevel::High,
        SecretCategory::GitHubToken,
    ) {
        patterns.push(p);
    }

    // GitLab Personal Access Token
    if let Ok(p) = SecretPattern::new(
        "gitlab_pat",
        "GitLab Personal Access Token",
        r"glpat-[A-Za-z0-9\-]{20,}",
        RiskLevel::High,
        SecretCategory::GitLabToken,
    ) {
        patterns.push(p);
    }

    // Slack Token
    if let Ok(p) = SecretPattern::new(
        "slack_token",
        "Slack Token",
        r"xox[baprs]-[A-Za-z0-9\-]{10,}",
        RiskLevel::High,
        SecretCategory::SlackToken,
    ) {
        patterns.push(p);
    }

    // Slack Webhook
    if let Ok(p) = SecretPattern::new(
        "slack_webhook",
        "Slack Webhook URL",
        r"https://hooks\.slack\.com/services/T[A-Z0-9]+/B[A-Z0-9]+/[A-Za-z0-9]+",
        RiskLevel::Medium,
        SecretCategory::SlackToken,
    ) {
        patterns.push(p);
    }

    // Stripe Secret Key
    if let Ok(p) = SecretPattern::new(
        "stripe_secret",
        "Stripe Secret Key",
        r"sk_(?:live|test|dummy)_[A-Za-z0-9]{24,}",
        RiskLevel::Critical,
        SecretCategory::StripeKey,
    ) {
        patterns.push(p);
    }

    // Stripe Publishable Key (lower risk but worth noting)
    if let Ok(p) = SecretPattern::new(
        "stripe_publishable",
        "Stripe Publishable Key",
        r"pk_live_[A-Za-z0-9]{24,}",
        RiskLevel::Low,
        SecretCategory::StripeKey,
    ) {
        patterns.push(p);
    }

    // Google API Key
    if let Ok(p) = SecretPattern::new(
        "google_api_key",
        "Google API Key",
        r"AIza[A-Za-z0-9\-_]{35}",
        RiskLevel::High,
        SecretCategory::GoogleCloud,
    ) {
        patterns.push(p);
    }

    // Google Cloud Service Account Key
    if let Ok(p) = SecretPattern::new(
        "gcp_service_account",
        "Google Cloud Service Account",
        r#""type"\s*:\s*"service_account""#,
        RiskLevel::Critical,
        SecretCategory::GoogleCloud,
    ) {
        patterns.push(p);
    }

    // Azure Connection String
    if let Ok(p) = SecretPattern::new(
        "azure_connection_string",
        "Azure Connection String",
        r"(?i)DefaultEndpointsProtocol=https;AccountName=[^;]+;AccountKey=[A-Za-z0-9+/=]{88}",
        RiskLevel::Critical,
        SecretCategory::Azure,
    ) {
        patterns.push(p);
    }

    // RSA Private Key
    if let Ok(p) = SecretPattern::new(
        "rsa_private_key",
        "RSA Private Key",
        r"-----BEGIN RSA PRIVATE KEY-----",
        RiskLevel::Critical,
        SecretCategory::PrivateKey,
    ) {
        patterns.push(p);
    }

    // OpenSSH Private Key
    if let Ok(p) = SecretPattern::new(
        "openssh_private_key",
        "OpenSSH Private Key",
        r"-----BEGIN OPENSSH PRIVATE KEY-----",
        RiskLevel::Critical,
        SecretCategory::PrivateKey,
    ) {
        patterns.push(p);
    }

    // Generic Private Key
    if let Ok(p) = SecretPattern::new(
        "generic_private_key",
        "Private Key",
        r"-----BEGIN (?:EC |DSA )?PRIVATE KEY-----",
        RiskLevel::Critical,
        SecretCategory::PrivateKey,
    ) {
        patterns.push(p);
    }

    // JWT Token
    if let Ok(p) = SecretPattern::new(
        "jwt_token",
        "JWT Token",
        r"eyJ[A-Za-z0-9_-]*\.eyJ[A-Za-z0-9_-]*\.[A-Za-z0-9_-]*",
        RiskLevel::Medium,
        SecretCategory::JwtToken,
    ) {
        patterns.push(p);
    }

    // Generic API Key patterns
    if let Ok(p) = SecretPattern::new(
        "generic_api_key",
        "Generic API Key",
        r#"(?i)(?:api[_\-]?key|apikey)['"]?\s*[:=]\s*['"]?([A-Za-z0-9\-_]{20,})"#,
        RiskLevel::Medium,
        SecretCategory::GenericApiKey,
    ) {
        patterns.push(p);
    }

    // Generic Secret/Token
    if let Ok(p) = SecretPattern::new(
        "generic_secret",
        "Generic Secret",
        r#"(?i)(?:secret|token)['"]?\s*[:=]\s*['"]?([A-Za-z0-9\-_]{20,})"#,
        RiskLevel::Medium,
        SecretCategory::GenericApiKey,
    ) {
        patterns.push(p);
    }

    // Password in URL
    if let Ok(p) = SecretPattern::new(
        "password_in_url",
        "Password in URL",
        r"://[^:]+:([^@]{8,})@",
        RiskLevel::High,
        SecretCategory::GenericPassword,
    ) {
        patterns.push(p);
    }

    // Database URL with credentials
    if let Ok(p) = SecretPattern::new(
        "database_url",
        "Database URL with Credentials",
        r"(?i)(?:postgres|mysql|mongodb|redis)://[^:]+:[^@]+@",
        RiskLevel::High,
        SecretCategory::DatabaseUrl,
    ) {
        patterns.push(p);
    }

    // Heroku API Key
    if let Ok(p) = SecretPattern::new(
        "heroku_api_key",
        "Heroku API Key",
        r#"(?i)heroku[_\-]?api[_\-]?key['"]?\s*[:=]\s*['"]?([a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12})"#,
        RiskLevel::High,
        SecretCategory::GenericApiKey,
    ) {
        patterns.push(p);
    }

    // npm Token
    if let Ok(p) = SecretPattern::new(
        "npm_token",
        "NPM Access Token",
        r"npm_[A-Za-z0-9]{36}",
        RiskLevel::High,
        SecretCategory::GenericApiKey,
    ) {
        patterns.push(p);
    }

    patterns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_access_key_detection() {
        let matcher = SecretPatternMatcher::new();
        let secrets = matcher.find_all("export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE");
        assert!(!secrets.is_empty());
        assert_eq!(secrets[0].category, SecretCategory::AwsCredentials);
    }

    #[test]
    fn test_github_token_detection() {
        let matcher = SecretPatternMatcher::new();
        let secrets = matcher.find_all(
            "git clone https://ghp_aBcDeFgHiJkLmNoPqRsTuVwXyZ1234567890@github.com/user/repo",
        );
        assert!(!secrets.is_empty());
        assert_eq!(secrets[0].category, SecretCategory::GitHubToken);
    }

    #[test]
    fn test_jwt_detection() {
        let matcher = SecretPatternMatcher::new();
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let secrets = matcher.find_all(&format!("curl -H 'Authorization: Bearer {}'", jwt));
        assert!(!secrets.is_empty());
        assert_eq!(secrets[0].category, SecretCategory::JwtToken);
    }

    #[test]
    fn test_private_key_detection() {
        let matcher = SecretPatternMatcher::new();
        let secrets = matcher.find_all("cat -----BEGIN RSA PRIVATE KEY-----");
        assert!(!secrets.is_empty());
        assert_eq!(secrets[0].category, SecretCategory::PrivateKey);
    }

    #[test]
    fn test_redaction() {
        let redacted = redact_secret("AKIAIOSFODNN7EXAMPLE");
        assert!(redacted.starts_with("AKIA"));
        assert!(redacted.ends_with("MPLE"));
        assert!(redacted.contains("*"));
    }

    #[test]
    fn test_no_false_positives() {
        let matcher = SecretPatternMatcher::new();

        // Normal commands should not trigger
        let secrets = matcher.find_all("ls -la");
        assert!(secrets.is_empty());

        let secrets = matcher.find_all("git commit -m 'Added new feature'");
        assert!(secrets.is_empty());

        let secrets = matcher.find_all("npm install express");
        assert!(secrets.is_empty());
    }
}
