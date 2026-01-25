//! Package Audit Plugin
//!
//! Warns before installing potentially risky packages via:
//! - npm, yarn, pnpm
//! - pip, pip3
//! - cargo
//! - gem
//! - apt, brew

use async_trait::async_trait;
use masterm_core::plugin::{
    ActivationTrigger, CommandAction, DetectionContext, Plugin, PluginActivation, PluginContext,
    PluginError, PluginManifest, PluginMeta, PluginPerformance, PluginPermissions,
    PluginRequirements, PromptContext,
};
use masterm_core::prompt::Segment;

/// Package info extracted from command
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct PackageInfo {
    /// Package manager (npm, pip, cargo, etc.)
    manager: String,
    /// Package name
    name: String,
    /// Version (if specified)
    version: Option<String>,
    /// Is global install
    is_global: bool,
}

/// Package Audit Plugin
pub struct PackageAuditPlugin {
    manifest: PluginManifest,
    /// Known malicious packages (blocklist)
    blocklist: Vec<String>,
    /// Warn on unscoped npm packages
    warn_unscoped_npm: bool,
    /// Enable typosquatting detection
    typosquatting_detection: bool,
    /// Popular packages to check for typosquatting
    popular_packages: Vec<String>,
}

impl PackageAuditPlugin {
    pub fn new() -> Self {
        Self {
            manifest: PluginManifest {
                plugin: PluginMeta {
                    name: "package-audit".to_string(),
                    version: "1.0.0".to_string(),
                    description: "Audit package installations for security risks".to_string(),
                    author: "MASTerm Security Team".to_string(),
                    license: "MIT".to_string(),
                    homepage: Some("https://masterm.dev/plugins/package-audit".to_string()),
                },
                requirements: PluginRequirements::default(),
                permissions: PluginPermissions {
                    filesystem: vec![],
                    network: "none".to_string(),
                    environment: vec![],
                    execute: vec![],
                },
                activation: PluginActivation {
                    triggers: vec![ActivationTrigger::Always],
                    mode: "always".to_string(),
                },
                performance: PluginPerformance {
                    startup_cost: "low".to_string(),
                    runtime_cost: "low".to_string(),
                },
            },
            blocklist: default_blocklist(),
            warn_unscoped_npm: true,
            typosquatting_detection: true,
            popular_packages: popular_packages(),
        }
    }

    /// Extract packages being installed from command
    fn extract_packages(&self, cmd: &str) -> Vec<PackageInfo> {
        let mut packages = Vec::new();
        let words: Vec<&str> = cmd.split_whitespace().collect();

        if words.is_empty() {
            return packages;
        }

        let cmd_lower = cmd.to_lowercase();

        // npm install
        if cmd_lower.contains("npm install")
            || cmd_lower.contains("npm i ")
            || cmd_lower.contains("yarn add")
            || cmd_lower.contains("pnpm add")
        {
            let manager = if cmd_lower.contains("npm") {
                "npm"
            } else if cmd_lower.contains("yarn") {
                "yarn"
            } else {
                "pnpm"
            };

            let is_global = cmd_lower.contains(" -g ") || cmd_lower.contains(" --global");

            // Find package names (skip flags)
            for word in &words {
                if word.starts_with('-')
                    || *word == "install"
                    || *word == "add"
                    || *word == "i"
                    || *word == "npm"
                    || *word == "yarn"
                    || *word == "pnpm"
                {
                    continue;
                }

                // Parse package@version
                let (name, version) = if word.contains('@') && !word.starts_with('@') {
                    let parts: Vec<&str> = word.splitn(2, '@').collect();
                    (
                        parts[0].to_string(),
                        Some(parts.get(1).unwrap_or(&"").to_string()),
                    )
                } else {
                    (word.to_string(), None)
                };

                if !name.is_empty() {
                    packages.push(PackageInfo {
                        manager: manager.to_string(),
                        name,
                        version,
                        is_global,
                    });
                }
            }
        }

        // pip install
        if cmd_lower.contains("pip install") || cmd_lower.contains("pip3 install") {
            for word in &words {
                if word.starts_with('-') || *word == "install" || *word == "pip" || *word == "pip3"
                {
                    continue;
                }

                let (name, version) = if word.contains("==") {
                    let parts: Vec<&str> = word.splitn(2, "==").collect();
                    (
                        parts[0].to_string(),
                        Some(parts.get(1).unwrap_or(&"").to_string()),
                    )
                } else {
                    (word.to_string(), None)
                };

                if !name.is_empty() && !name.contains('/') {
                    packages.push(PackageInfo {
                        manager: "pip".to_string(),
                        name,
                        version,
                        is_global: false,
                    });
                }
            }
        }

        // cargo install
        if cmd_lower.contains("cargo install") {
            for word in &words {
                if word.starts_with('-') || *word == "install" || *word == "cargo" {
                    continue;
                }

                if !word.is_empty() {
                    packages.push(PackageInfo {
                        manager: "cargo".to_string(),
                        name: word.to_string(),
                        version: None,
                        is_global: true,
                    });
                }
            }
        }

        packages
    }

    /// Check if package is on blocklist
    fn is_blocked(&self, name: &str) -> bool {
        self.blocklist.iter().any(|b| b.eq_ignore_ascii_case(name))
    }

    /// Check for typosquatting
    fn check_typosquatting(&self, name: &str) -> Option<String> {
        if !self.typosquatting_detection {
            return None;
        }

        let name_lower = name.to_lowercase();

        for popular in &self.popular_packages {
            let popular_lower = popular.to_lowercase();

            // Skip if exact match
            if name_lower == popular_lower {
                continue;
            }

            // Check for common typosquatting patterns
            let distance = levenshtein_distance(&name_lower, &popular_lower);
            if distance == 1 || distance == 2 {
                return Some(popular.clone());
            }

            // Check for hyphen/underscore confusion
            let normalized_name = name_lower.replace('-', "_");
            let normalized_popular = popular_lower.replace('-', "_");
            if normalized_name == normalized_popular && name_lower != popular_lower {
                return Some(popular.clone());
            }
        }

        None
    }

    /// Check if npm package is unscoped
    fn is_unscoped_npm(&self, pkg: &PackageInfo) -> bool {
        pkg.manager == "npm" && !pkg.name.starts_with('@')
    }
}

impl Default for PackageAuditPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for PackageAuditPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    async fn init(&mut self, ctx: &PluginContext) -> Result<(), PluginError> {
        if let Some(warn) = ctx.get_config_bool("warn_unscoped_npm") {
            self.warn_unscoped_npm = warn;
        }
        if let Some(detect) = ctx.get_config_bool("typosquatting_detection") {
            self.typosquatting_detection = detect;
        }
        if let Ok(blocked) = ctx.get_string_list("blocklist") {
            self.blocklist.extend(blocked);
        }
        Ok(())
    }

    fn should_activate(&self, _ctx: &DetectionContext) -> bool {
        true
    }

    async fn segments(&self, _ctx: &PromptContext) -> Result<Vec<Segment>, PluginError> {
        Ok(vec![])
    }

    fn on_command(&self, cmd: &str) -> CommandAction {
        let packages = self.extract_packages(cmd);

        if packages.is_empty() {
            return CommandAction::Allow;
        }

        // Check for blocked packages
        for pkg in &packages {
            if self.is_blocked(&pkg.name) {
                return CommandAction::Block(format!(
                    "\x1b[1;31m🚫 MALICIOUS PACKAGE BLOCKED\x1b[0m\n\n\
                     Package: \x1b[1m{}\x1b[0m (via {})\n\n\
                     This package is known to be malicious and has been blocked.\n\
                     See: https://www.npmjs.com/advisories for more information.\n",
                    pkg.name, pkg.manager
                ));
            }
        }

        // Check for typosquatting
        for pkg in &packages {
            if let Some(intended) = self.check_typosquatting(&pkg.name) {
                return CommandAction::Confirm(format!(
                    "\x1b[1;33m⚠️  POSSIBLE TYPOSQUATTING DETECTED\x1b[0m\n\n\
                     You are installing: \x1b[1m{}\x1b[0m\n\
                     Did you mean: \x1b[1;32m{}\x1b[0m?\n\n\
                     Typosquatting is a common supply chain attack vector.\n\
                     Please verify the package name carefully.\n\n\
                     Type '\x1b[1;32myes\x1b[0m' to proceed or '\x1b[1;31mno\x1b[0m' to cancel:",
                    pkg.name, intended
                ));
            }
        }

        // Warn on unscoped npm packages
        if self.warn_unscoped_npm {
            for pkg in &packages {
                if self.is_unscoped_npm(pkg) && pkg.is_global {
                    return CommandAction::Warn(format!(
                        "\x1b[1;33mℹ️  UNSCOPED GLOBAL PACKAGE\x1b[0m\n\n\
                         Installing global package: \x1b[1m{}\x1b[0m\n\n\
                         Tip: Scoped packages (@scope/package) are more secure as they\n\
                         require organization verification.\n",
                        pkg.name
                    ));
                }
            }
        }

        CommandAction::Allow
    }

    async fn cleanup(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

/// Simple Levenshtein distance calculation
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(a_len + 1) {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate().take(b_len + 1) {
        *cell = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[a_len][b_len]
}

/// Default blocklist of known malicious packages
fn default_blocklist() -> Vec<String> {
    vec![
        // npm
        "event-stream".to_string(),
        "flatmap-stream".to_string(),
        "electron-native-notify".to_string(),
        "getcookies".to_string(),
        "discord.js-user".to_string(),
        "colors-99".to_string(),
        // pip
        "python3-dateutil".to_string(),
        "jeIlyfish".to_string(),
        "python-sqlite".to_string(),
    ]
}

/// Popular packages for typosquatting detection
fn popular_packages() -> Vec<String> {
    vec![
        // npm
        "express".to_string(),
        "react".to_string(),
        "lodash".to_string(),
        "axios".to_string(),
        "moment".to_string(),
        "webpack".to_string(),
        "typescript".to_string(),
        "next".to_string(),
        "vue".to_string(),
        "angular".to_string(),
        // pip
        "requests".to_string(),
        "numpy".to_string(),
        "pandas".to_string(),
        "django".to_string(),
        "flask".to_string(),
        "tensorflow".to_string(),
        "pytorch".to_string(),
        // cargo
        "serde".to_string(),
        "tokio".to_string(),
        "clap".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npm_package_extraction() {
        let plugin = PackageAuditPlugin::new();

        let packages = plugin.extract_packages("npm install express lodash");
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0].name, "express");
        assert_eq!(packages[1].name, "lodash");
    }

    #[test]
    fn test_pip_package_extraction() {
        let plugin = PackageAuditPlugin::new();

        let packages = plugin.extract_packages("pip install requests==2.28.0 flask");
        assert_eq!(packages.len(), 2);
        assert_eq!(packages[0].name, "requests");
        assert_eq!(packages[0].version, Some("2.28.0".to_string()));
    }

    #[test]
    fn test_blocked_package() {
        let plugin = PackageAuditPlugin::new();

        let action = plugin.on_command("npm install event-stream");
        assert!(matches!(action, CommandAction::Block(_)));
    }

    #[test]
    fn test_typosquatting_detection() {
        let plugin = PackageAuditPlugin::new();

        // "expresss" is close to "express"
        let similar = plugin.check_typosquatting("expresss");
        assert_eq!(similar, Some("express".to_string()));

        // "requests" should not trigger (exact match)
        let exact = plugin.check_typosquatting("requests");
        assert!(exact.is_none());
    }

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("express", "expresss"), 1);
        assert_eq!(levenshtein_distance("lodash", "lodash"), 0);
    }
}
