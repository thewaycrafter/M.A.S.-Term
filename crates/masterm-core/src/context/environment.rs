//! Environment type detection (dev/staging/prod)

use std::path::Path;

/// Environment type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EnvironmentType {
    /// Development environment
    #[default]
    Development,

    /// Staging/UAT environment
    Staging,

    /// Production environment
    Production,

    /// Unknown environment
    Unknown,
}

impl EnvironmentType {
    /// Detect environment type from directory and patterns
    pub fn detect(cwd: &Path, prod_patterns: &[String]) -> Self {
        // Check environment variables first (highest precedence)
        if let Some(env_type) = Self::from_env_vars() {
            return env_type;
        }

        // Check path against patterns
        let path_str = cwd.to_string_lossy().to_lowercase().replace('\\', "/");

        // Check production patterns
        for pattern in prod_patterns {
            if Self::matches_pattern(&path_str, pattern) {
                return Self::Production;
            }
        }

        // Check common patterns
        if path_str.contains("/prod")
            || path_str.contains("/production")
            || path_str.contains("/prd-")
        {
            return Self::Production;
        }

        if path_str.contains("/staging") || path_str.contains("/stg") || path_str.contains("/uat") {
            return Self::Staging;
        }

        if path_str.contains("/dev")
            || path_str.contains("/development")
            || path_str.contains("/local")
        {
            return Self::Development;
        }

        // Check for .env files
        if cwd.join(".env.production").exists() {
            return Self::Production;
        }
        if cwd.join(".env.staging").exists() {
            return Self::Staging;
        }

        Self::Unknown
    }

    /// Check environment variables for hints
    fn from_env_vars() -> Option<Self> {
        // Check common environment variables
        let env_vars = [
            "TERMX_ENV",
            "NODE_ENV",
            "RAILS_ENV",
            "RACK_ENV",
            "GO_ENV",
            "APP_ENV",
        ];

        for var in env_vars {
            if let Ok(value) = std::env::var(var) {
                if let Some(env_type) = Self::parse_from_str(&value) {
                    return Some(env_type);
                }
            }
        }

        None
    }

    /// Parse from string (not FromStr trait)
    pub fn parse_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "production" | "prod" | "prd" => Some(Self::Production),
            "staging" | "stage" | "stg" | "uat" | "preprod" => Some(Self::Staging),
            "development" | "dev" | "local" | "test" => Some(Self::Development),
            _ => None,
        }
    }

    /// Simple glob-like pattern matching
    fn matches_pattern(path: &str, pattern: &str) -> bool {
        // Simple pattern matching: ** = any path, * = any segment
        let pattern = pattern.replace("**", ".*").replace('*', "[^/]*");

        regex::Regex::new(&pattern)
            .map(|re| re.is_match(path))
            .unwrap_or(false)
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Development => "DEV",
            Self::Staging => "STAGING",
            Self::Production => "PROD",
            Self::Unknown => "",
        }
    }

    /// Get full name
    pub fn full_name(&self) -> &'static str {
        match self {
            Self::Development => "Development",
            Self::Staging => "Staging",
            Self::Production => "Production",
            Self::Unknown => "Unknown",
        }
    }

    /// Is this a production environment?
    pub fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }

    /// Is this a sensitive environment (prod or staging)?
    pub fn is_sensitive(&self) -> bool {
        matches!(self, Self::Production | Self::Staging)
    }

    /// Get color for display (ANSI color code)
    pub fn color(&self) -> &'static str {
        match self {
            Self::Development => "\x1b[32m", // Green
            Self::Staging => "\x1b[33m",     // Yellow
            Self::Production => "\x1b[31m",  // Red
            Self::Unknown => "\x1b[0m",      // Reset
        }
    }

    /// Get background color for display
    pub fn bg_color(&self) -> &'static str {
        match self {
            Self::Development => "\x1b[42m", // Green bg
            Self::Staging => "\x1b[43m",     // Yellow bg
            Self::Production => "\x1b[41m",  // Red bg
            Self::Unknown => "\x1b[0m",      // Reset
        }
    }
}

impl std::fmt::Display for EnvironmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_from_str() {
        assert_eq!(
            EnvironmentType::parse_from_str("production"),
            Some(EnvironmentType::Production)
        );
        assert_eq!(
            EnvironmentType::parse_from_str("prod"),
            Some(EnvironmentType::Production)
        );
        assert_eq!(
            EnvironmentType::parse_from_str("staging"),
            Some(EnvironmentType::Staging)
        );
        assert_eq!(
            EnvironmentType::parse_from_str("dev"),
            Some(EnvironmentType::Development)
        );
        assert_eq!(EnvironmentType::parse_from_str("unknown"), None);
    }

    #[test]
    fn test_detect_from_path() {
        let temp_dir = TempDir::new().unwrap();
        let prod_path = temp_dir.path().join("production/app");
        std::fs::create_dir_all(&prod_path).unwrap();

        let patterns = vec!["**/production/**".to_string()];
        let env_type = EnvironmentType::detect(&prod_path, &patterns);

        assert_eq!(env_type, EnvironmentType::Production);
    }
}
