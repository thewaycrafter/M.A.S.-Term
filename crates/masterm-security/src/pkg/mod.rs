//! Package manager auditing
//!
//! Detects and audits package installation commands.

/// Package manager type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageManager {
    Npm,
    Yarn,
    Pnpm,
    Pip,
    Cargo,
    Gem,
    Go,
    Maven,
    Gradle,
    Unknown,
}

/// Package install request
#[derive(Debug, Clone)]
pub struct PackageInstall {
    pub manager: PackageManager,
    pub packages: Vec<String>,
    pub is_global: bool,
    pub original_command: String,
}

/// Package auditor
pub struct PackageAuditor;

impl PackageAuditor {
    /// Analyze a command for package installation
    pub fn analyze(command: &str) -> Option<PackageInstall> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        let cmd = parts[0];
        let args = &parts[1..];

        match cmd {
            "npm" => Self::parse_npm(args, command),
            "pip" | "pip3" => Self::parse_pip(args, command),
            "cargo" => Self::parse_cargo(args, command),
            _ => None,
        }
    }

    fn parse_npm(args: &[&str], full_cmd: &str) -> Option<PackageInstall> {
        if args.is_empty() {
            return None;
        }

        let subcmd = args[0];
        if subcmd == "install" || subcmd == "i" || subcmd == "add" {
            let mut packages = Vec::new();
            let mut is_global = false;

            for arg in &args[1..] {
                if arg.starts_with('-') {
                    if *arg == "-g" || *arg == "--global" {
                        is_global = true;
                    }
                } else {
                    packages.push(arg.to_string());
                }
            }

            if !packages.is_empty() {
                return Some(PackageInstall {
                    manager: PackageManager::Npm,
                    packages,
                    is_global,
                    original_command: full_cmd.to_string(),
                });
            }
        }
        None
    }

    fn parse_pip(args: &[&str], full_cmd: &str) -> Option<PackageInstall> {
        if args.is_empty() {
            return None;
        }

        if args[0] == "install" {
            let mut packages = Vec::new();

            for arg in &args[1..] {
                if !arg.starts_with('-') {
                    packages.push(arg.to_string());
                }
            }

            if !packages.is_empty() {
                return Some(PackageInstall {
                    manager: PackageManager::Pip,
                    packages,
                    is_global: false, // pip usually is global unless in venv, hard to detect
                    original_command: full_cmd.to_string(),
                });
            }
        }
        None
    }

    fn parse_cargo(args: &[&str], full_cmd: &str) -> Option<PackageInstall> {
        if args.is_empty() {
            return None;
        }

        if args[0] == "install" || args[0] == "add" {
            let mut packages = Vec::new();

            for arg in &args[1..] {
                if !arg.starts_with('-') {
                    packages.push(arg.to_string());
                }
            }

            if !packages.is_empty() {
                return Some(PackageInstall {
                    manager: PackageManager::Cargo,
                    packages,
                    is_global: args[0] == "install",
                    original_command: full_cmd.to_string(),
                });
            }
        }
        None
    }
}
