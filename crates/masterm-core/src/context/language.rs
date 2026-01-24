//! Programming language detection

use std::path::Path;
use std::process::Command;

/// Detected programming language context
#[derive(Debug, Clone)]
pub struct LanguageContext {
    /// Language name
    pub name: String,

    /// Language version (if detected)
    pub version: Option<String>,

    /// Package/project name
    pub package_name: Option<String>,

    /// Icon for display
    pub icon: &'static str,

    /// Fallback icon (ASCII)
    pub icon_fallback: &'static str,
}

impl LanguageContext {
    /// Detect all languages in a directory
    pub async fn detect_all(cwd: &Path) -> Vec<Self> {
        let mut languages = Vec::new();

        // Check for each language in parallel (simplified to sequential for MVP)
        if let Some(lang) = Self::detect_node(cwd).await {
            languages.push(lang);
        }
        if let Some(lang) = Self::detect_python(cwd).await {
            languages.push(lang);
        }
        if let Some(lang) = Self::detect_rust(cwd).await {
            languages.push(lang);
        }
        if let Some(lang) = Self::detect_go(cwd).await {
            languages.push(lang);
        }
        if let Some(lang) = Self::detect_java(cwd).await {
            languages.push(lang);
        }

        languages
    }

    /// Detect Node.js project
    async fn detect_node(cwd: &Path) -> Option<Self> {
        let package_json = cwd.join("package.json");
        if !package_json.exists() {
            return None;
        }

        let version = Self::get_command_version("node", &["--version"]);
        let package_name = Self::get_package_json_name(&package_json);

        Some(Self {
            name: "Node.js".to_string(),
            version,
            package_name,
            icon: "",
            icon_fallback: "⬢",
        })
    }

    /// Detect Python project
    async fn detect_python(cwd: &Path) -> Option<Self> {
        let has_python = cwd.join("pyproject.toml").exists()
            || cwd.join("setup.py").exists()
            || cwd.join("requirements.txt").exists()
            || cwd.join("Pipfile").exists();

        if !has_python {
            return None;
        }

        // Check for virtual environment
        let in_venv = std::env::var("VIRTUAL_ENV").is_ok();
        let version = Self::get_command_version("python3", &["--version"])
            .or_else(|| Self::get_command_version("python", &["--version"]));

        let package_name = if in_venv {
            std::env::var("VIRTUAL_ENV")
                .ok()
                .and_then(|p| Path::new(&p).file_name().map(|s| s.to_string_lossy().to_string()))
        } else {
            None
        };

        Some(Self {
            name: "Python".to_string(),
            version,
            package_name,
            icon: "",
            icon_fallback: "🐍",
        })
    }

    /// Detect Rust project
    async fn detect_rust(cwd: &Path) -> Option<Self> {
        let cargo_toml = cwd.join("Cargo.toml");
        if !cargo_toml.exists() {
            return None;
        }

        let version = Self::get_command_version("rustc", &["--version"]);
        let package_name = Self::get_cargo_package_name(&cargo_toml);

        Some(Self {
            name: "Rust".to_string(),
            version,
            package_name,
            icon: "",
            icon_fallback: "🦀",
        })
    }

    /// Detect Go project
    async fn detect_go(cwd: &Path) -> Option<Self> {
        let go_mod = cwd.join("go.mod");
        if !go_mod.exists() {
            return None;
        }

        let version = Self::get_command_version("go", &["version"]);
        let package_name = Self::get_go_module_name(&go_mod);

        Some(Self {
            name: "Go".to_string(),
            version,
            package_name,
            icon: "",
            icon_fallback: "🐹",
        })
    }

    /// Detect Java project
    async fn detect_java(cwd: &Path) -> Option<Self> {
        let has_java = cwd.join("pom.xml").exists()
            || cwd.join("build.gradle").exists()
            || cwd.join("build.gradle.kts").exists();

        if !has_java {
            return None;
        }

        let version = Self::get_command_version("java", &["--version"]);

        Some(Self {
            name: "Java".to_string(),
            version,
            package_name: None,
            icon: "",
            icon_fallback: "☕",
        })
    }

    /// Get version from command output
    fn get_command_version(cmd: &str, args: &[&str]) -> Option<String> {
        Command::new(cmd)
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| {
                let output = String::from_utf8_lossy(&o.stdout);
                // Extract just the version number
                Self::extract_version(&output)
            })
    }

    /// Extract version number from output
    fn extract_version(output: &str) -> String {
        // Common patterns: "v18.17.0", "Python 3.11.4", "rustc 1.74.0"
        let version_pattern = regex::Regex::new(r"v?(\d+\.\d+\.?\d*)").ok();

        version_pattern
            .and_then(|re| re.captures(output))
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| output.trim().to_string())
    }

    /// Get package name from package.json
    fn get_package_json_name(path: &Path) -> Option<String> {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
            .and_then(|json| json["name"].as_str().map(|s| s.to_string()))
    }

    /// Get package name from Cargo.toml
    fn get_cargo_package_name(path: &Path) -> Option<String> {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|content| toml::from_str::<toml::Value>(&content).ok())
            .and_then(|toml| {
                toml.get("package")
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
                    .map(|s| s.to_string())
            })
    }

    /// Get module name from go.mod
    fn get_go_module_name(path: &Path) -> Option<String> {
        std::fs::read_to_string(path).ok().and_then(|content| {
            content
                .lines()
                .find(|line| line.starts_with("module "))
                .map(|line| {
                    line.strip_prefix("module ")
                        .unwrap_or(line)
                        .trim()
                        .to_string()
                })
        })
    }
}
