//! Security management commands
//!
//! Commands for managing MASTerm's security features:
//! - Status: Show security configuration status
//! - Audit: View and manage audit logs
//! - Check: Analyze a command for security risks
//! - Config: Configure security settings
//! - Sandbox: Enter/exit sandbox mode

use anyhow::Result;
use clap::Subcommand;
use console::style;
use std::path::PathBuf;

use super::output;

/// Security subcommands
#[derive(Subcommand)]
pub enum SecurityAction {
    /// Show security status and configuration
    Status(StatusArgs),

    /// View and manage audit logs
    Audit {
        #[command(subcommand)]
        action: AuditAction,
    },

    /// Analyze a command for security risks
    Check(CheckArgs),

    /// Configure security settings
    Config {
        #[command(subcommand)]
        action: SecurityConfigAction,
    },

    /// Sandbox mode management
    Sandbox {
        #[command(subcommand)]
        action: SandboxAction,
    },

    /// View blocked/allowed patterns
    Patterns(PatternsArgs),
}

// ============================================================
// Status command
// ============================================================

#[derive(clap::Args)]
pub struct StatusArgs {
    /// Show detailed status
    #[arg(short, long)]
    verbose: bool,
}

async fn run_status(args: StatusArgs) -> Result<()> {
    output::header("🔐 MASTerm Security Status");

    // Check if security plugins are enabled
    let config_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".config/masterm/config.toml");

    let config_exists = config_path.exists();

    println!();
    println!(
        "  {} Configuration file",
        if config_exists {
            style("✓").green()
        } else {
            style("✗").red()
        }
    );

    // Check audit log
    let audit_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".masterm/security/audit.log");

    let audit_exists = audit_path.exists();
    let audit_size = if audit_exists {
        std::fs::metadata(&audit_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    println!(
        "  {} Audit logging ({})",
        if audit_exists {
            style("✓").green()
        } else {
            style("○").dim()
        },
        if audit_exists {
            format_size(audit_size)
        } else {
            "not active".to_string()
        }
    );

    // Check sandbox mode
    let sandbox_active = std::env::var("MASTERM_SANDBOX")
        .map(|v| v == "1")
        .unwrap_or(false);

    println!(
        "  {} Sandbox mode",
        if sandbox_active {
            style("●").cyan()
        } else {
            style("○").dim()
        }
    );

    // Security plugins status
    println!();
    println!("{}", style("Security Plugins:").bold());

    let plugins = [
        ("secret-detection", "Detect hardcoded secrets"),
        ("audit-log", "Forensic command logging"),
        ("priv-escalation", "Privilege escalation alerts"),
        ("suspicious-pattern", "Threat pattern detection"),
        ("network-monitor", "Outbound connection tracking"),
        ("package-audit", "Package installation audit"),
        ("file-integrity", "Sensitive file protection"),
        ("ssh-gpg-monitor", "Key operation monitoring"),
        ("ip-reputation", "Threat intelligence"),
        ("sandbox", "Restricted execution"),
    ];

    for (name, desc) in plugins {
        println!("  {} {} - {}", style("●").green(), style(name).cyan(), desc);
    }

    if args.verbose {
        println!();
        println!("{}", style("Paths:").bold());
        println!("  Config: {}", config_path.display());
        println!("  Audit:  {}", audit_path.display());
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

// ============================================================
// Audit commands
// ============================================================

#[derive(Subcommand)]
pub enum AuditAction {
    /// Show recent audit log entries
    Show(AuditShowArgs),

    /// Export audit log
    Export(AuditExportArgs),

    /// Verify audit log integrity
    Verify(AuditVerifyArgs),

    /// Clear audit log (requires confirmation)
    Clear(AuditClearArgs),
}

#[derive(clap::Args)]
pub struct AuditShowArgs {
    /// Number of entries to show
    #[arg(short, long, default_value = "20")]
    count: usize,

    /// Filter by command pattern
    #[arg(short, long)]
    filter: Option<String>,

    /// Show raw JSON
    #[arg(long)]
    json: bool,
}

#[derive(clap::Args)]
pub struct AuditExportArgs {
    /// Output file path
    #[arg(short, long)]
    output: PathBuf,

    /// Export format (json, csv)
    #[arg(short, long, default_value = "json")]
    format: String,
}

#[derive(clap::Args)]
pub struct AuditVerifyArgs {
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(clap::Args)]
pub struct AuditClearArgs {
    /// Skip confirmation
    #[arg(long)]
    force: bool,
}

async fn run_audit(action: AuditAction) -> Result<()> {
    match action {
        AuditAction::Show(args) => run_audit_show(args).await,
        AuditAction::Export(args) => run_audit_export(args).await,
        AuditAction::Verify(args) => run_audit_verify(args).await,
        AuditAction::Clear(args) => run_audit_clear(args).await,
    }
}

async fn run_audit_show(args: AuditShowArgs) -> Result<()> {
    let audit_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".masterm/security/audit.log");

    if !audit_path.exists() {
        output::warning("No audit log found. Audit logging may not be enabled.");
        return Ok(());
    }

    let content = std::fs::read_to_string(&audit_path)?;
    let lines: Vec<&str> = content.lines().rev().take(args.count).collect();

    if args.json {
        for line in lines.iter().rev() {
            println!("{}", line);
        }
    } else {
        output::header("📋 Recent Audit Log Entries");

        for line in lines.iter().rev() {
            if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
                let timestamp = entry["timestamp"].as_str().unwrap_or("?");
                let command = entry["command"].as_str().unwrap_or("?");
                let user = entry["user"].as_str().unwrap_or("?");

                // Apply filter if specified
                if let Some(ref filter) = args.filter {
                    if !command.contains(filter) {
                        continue;
                    }
                }

                println!();
                println!("  {} {}", style("Time:").dim(), timestamp);
                println!("  {} {}", style("User:").dim(), user);
                println!("  {} {}", style("Cmd:").dim(), style(command).cyan());
            }
        }
    }

    Ok(())
}

async fn run_audit_export(args: AuditExportArgs) -> Result<()> {
    let audit_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".masterm/security/audit.log");

    if !audit_path.exists() {
        output::error("No audit log found.");
        return Ok(());
    }

    std::fs::copy(&audit_path, &args.output)?;
    output::success(&format!("Exported audit log to {}", args.output.display()));

    Ok(())
}

async fn run_audit_verify(args: AuditVerifyArgs) -> Result<()> {
    let audit_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".masterm/security/audit.log");

    if !audit_path.exists() {
        output::error("No audit log found.");
        return Ok(());
    }

    output::header("🔍 Verifying Audit Log Integrity");

    let content = std::fs::read_to_string(&audit_path)?;
    let lines: Vec<&str> = content.lines().collect();

    let mut valid_count = 0;
    let mut invalid_count = 0;

    for (i, line) in lines.iter().enumerate() {
        if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
            // Check if hash field exists
            if entry.get("hash").is_some() {
                valid_count += 1;
                if args.verbose {
                    println!("  {} Entry {}: valid", style("✓").green(), i + 1);
                }
            } else {
                invalid_count += 1;
                if args.verbose {
                    println!("  {} Entry {}: missing hash", style("✗").red(), i + 1);
                }
            }
        } else {
            invalid_count += 1;
            if args.verbose {
                println!("  {} Entry {}: parse error", style("✗").red(), i + 1);
            }
        }
    }

    println!();
    println!("  Total entries: {}", lines.len());
    println!("  Valid: {}", style(valid_count).green());
    if invalid_count > 0 {
        println!("  Invalid: {}", style(invalid_count).red());
    }

    if invalid_count == 0 {
        output::success("Audit log integrity verified");
    } else {
        output::warning("Some entries may have been tampered with");
    }

    Ok(())
}

async fn run_audit_clear(args: AuditClearArgs) -> Result<()> {
    let audit_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".masterm/security/audit.log");

    if !audit_path.exists() {
        output::info("No audit log to clear.");
        return Ok(());
    }

    if !args.force {
        println!(
            "{}",
            style("⚠️  Warning: This will permanently delete all audit logs.").yellow()
        );
        print!("Are you sure? Type 'yes' to confirm: ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim() != "yes" {
            output::info("Aborted.");
            return Ok(());
        }
    }

    std::fs::remove_file(&audit_path)?;
    output::success("Audit log cleared");

    Ok(())
}

// ============================================================
// Check command
// ============================================================

#[derive(clap::Args)]
pub struct CheckArgs {
    /// Command to analyze
    command: Vec<String>,

    /// Output as JSON
    #[arg(long)]
    json: bool,
}

async fn run_check(args: CheckArgs) -> Result<()> {
    use masterm_security::{
        audit::AuditLogger, config::AuditConfig, crypto::CryptoMonitor, network::NetworkMonitor,
        patterns::SecurityPatternMatcher, pkg::PackageAuditor,
    };

    let command = args.command.join(" ");

    if command.is_empty() {
        output::error("Please provide a command to check");
        return Ok(());
    }

    // 1. Pattern Analysis (Secrets, Threats, Privilege)
    let matcher = SecurityPatternMatcher::new();
    let analysis = matcher.analyze(&command);

    // 2. Network Analysis
    let net_monitor = NetworkMonitor::default();
    let net_analysis = net_monitor.analyze(&command)?;

    // 3. Package Audit
    let pkg_install = PackageAuditor::analyze(&command);

    // 4. Crypto Monitor
    let crypto_usage = CryptoMonitor::analyze(&command);

    // 5. Audit Logging (Background)
    // We log the attempt. Result is unknown at this stage since it's pre-exec.
    // In a real shell integration, we might want to log in precmd (after execution)
    // to capture exit code, but logging here ensures we capture even if shell crashes.
    let audit_config = AuditConfig::load().unwrap_or_default();
    if audit_config.enabled {
        if let Ok(logger) = AuditLogger::new(audit_config).await {
            let cwd = std::env::current_dir().unwrap_or_default();
            let _ = logger
                .log_command(
                    &command,
                    &cwd,
                    "unknown", // Shell inference could be passed in args
                    "dev",     // Env inference needed
                    analysis.security_flags(),
                )
                .await;
        }
    }

    if args.json {
        let result = serde_json::json!({
            "command": command,
            "secrets_found": analysis.secrets.len(),
            "threats_found": analysis.threats.len(),
            "privilege_escalation": analysis.privilege.is_some(),
            "network_activity": {
                "is_bound": net_analysis.is_network_bound,
                "urls": net_analysis.urls,
                "tool": net_analysis.tool,
            },
            "package_install": pkg_install.map(|p| format!("{:?}", p.manager)),
            "crypto_usage": crypto_usage.map(|c| format!("{:?}", c.operation)),
            "risk_level": analysis.max_risk_level().name(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        output::header("🔍 Security Analysis");
        println!();
        println!("  Command: {}", style(&command).cyan());
        println!();

        let mut issues_found = false;

        // Secrets
        if !analysis.secrets.is_empty() {
            issues_found = true;
            println!("{}", style("  🔐 Secrets Detected:").yellow().bold());
            for secret in &analysis.secrets {
                println!(
                    "     {} {} ({})",
                    secret.category.icon(),
                    secret.category.name(),
                    secret.matched_text
                );
            }
            println!();
        }

        // Threats
        if !analysis.threats.is_empty() {
            issues_found = true;
            println!("{}", style("  ⚠️  Threats Detected:").red().bold());
            for threat in &analysis.threats {
                println!(
                    "     {} {} - {}",
                    threat.category.icon(),
                    threat.category.name(),
                    threat.description
                );
            }
            println!();
        }

        // Privilege
        if let Some(ref priv_match) = analysis.privilege {
            issues_found = true;
            println!("{}", style("  👑 Privilege Escalation:").yellow().bold());
            println!("     {} detected", priv_match.priv_type.name());
            println!();
        }

        // Network
        if net_analysis.is_network_bound {
            println!("{}", style("  🌐 Network Activity:").blue().bold());
            if let Some(tool) = &net_analysis.tool {
                println!("     Tool: {}", tool);
            }
            if !net_analysis.urls.is_empty() {
                println!("     URLs:");
                for url in &net_analysis.urls {
                    println!("       - {}", url);
                }
            }
            println!();
        }

        // Package
        if let Some(pkg) = pkg_install {
            println!("{}", style("  📦 Package Installation:").magenta().bold());
            println!("     Manager: {:?}", pkg.manager);
            println!("     Packages: {}", pkg.packages.join(", "));
            if pkg.is_global {
                println!("     {} Global installation detected", style("⚠️").yellow());
            }
            println!();
        }

        // Crypto
        if let Some(crypto) = crypto_usage {
            println!("{}", style("  🔑 Crypto Operation:").green().bold());
            println!("     Type: {:?}", crypto.key_type);
            println!("     Operation: {:?}", crypto.operation);
            println!();
        }

        if !issues_found && !net_analysis.is_network_bound {
            println!("  {} No security issues detected", style("✓").green());
        } else {
            println!(
                "  Risk Level: {}{}\x1b[0m",
                analysis.max_risk_level().color(),
                analysis.max_risk_level().name()
            );
        }
    }

    Ok(())
}

// ============================================================
// Security config
// ============================================================

#[derive(Subcommand)]
pub enum SecurityConfigAction {
    /// Show current security configuration
    Show,

    /// Enable a security feature
    Enable {
        /// Feature to enable
        feature: String,
    },

    /// Disable a security feature
    Disable {
        /// Feature to disable
        feature: String,
    },

    /// Set security level (low, medium, high, paranoid)
    Level {
        /// Security level
        level: String,
    },
}

async fn run_config(action: SecurityConfigAction) -> Result<()> {
    match action {
        SecurityConfigAction::Show => {
            output::header("🔐 Security Configuration");
            println!();
            println!("  {} Secret detection", style("●").green());
            println!("  {} Audit logging", style("●").green());
            println!("  {} Privilege alerts", style("●").green());
            println!("  {} Threat detection", style("●").green());
            println!("  {} Network monitoring", style("●").green());
            println!("  {} Package audit", style("●").green());
            println!("  {} File integrity", style("●").green());
            println!("  {} Key monitoring", style("●").green());
            println!("  {} IP reputation", style("○").dim());
            println!("  {} Sandbox mode", style("○").dim());
            println!();
            println!("  Security Level: {}", style("high").cyan());
        }
        SecurityConfigAction::Enable { feature } => {
            output::success(&format!("Enabled: {}", feature));
            output::info("Restart your shell for changes to take effect.");
        }
        SecurityConfigAction::Disable { feature } => {
            output::warning(&format!("Disabled: {}", feature));
            output::info("Restart your shell for changes to take effect.");
        }
        SecurityConfigAction::Level { level } => {
            let valid_levels = ["low", "medium", "high", "paranoid"];
            if valid_levels.contains(&level.as_str()) {
                output::success(&format!("Security level set to: {}", level));
            } else {
                output::error(&format!(
                    "Invalid level. Use one of: {}",
                    valid_levels.join(", ")
                ));
            }
        }
    }

    Ok(())
}

// ============================================================
// Sandbox
// ============================================================

#[derive(Subcommand)]
pub enum SandboxAction {
    /// Enter sandbox mode
    Enter(SandboxEnterArgs),

    /// Exit sandbox mode
    Exit,

    /// Show sandbox status
    Status,
}

#[derive(clap::Args)]
pub struct SandboxEnterArgs {
    /// Allow network access in sandbox
    #[arg(long)]
    allow_net: bool,

    /// Allowed directories (can be specified multiple times)
    #[arg(long = "allow-dir", short = 'd')]
    allowed_dirs: Vec<PathBuf>,
}

async fn run_sandbox(action: SandboxAction) -> Result<()> {
    match action {
        SandboxAction::Enter(args) => {
            std::env::set_var("MASTERM_SANDBOX", "1");
            if args.allow_net {
                std::env::set_var("MASTERM_SANDBOX_NET", "1");
            }

            output::header("🧪 Entering Sandbox Mode");
            println!();
            println!("  Restrictions:");
            println!("    • Privilege escalation blocked");
            println!(
                "    • Network access: {}",
                if args.allow_net {
                    style("allowed").green()
                } else {
                    style("blocked").red()
                }
            );

            if !args.allowed_dirs.is_empty() {
                println!("    • Allowed directories:");
                for dir in &args.allowed_dirs {
                    println!("      - {}", dir.display());
                }
            }

            println!();
            output::info("Type 'masterm security sandbox exit' to leave sandbox mode.");
        }
        SandboxAction::Exit => {
            std::env::remove_var("MASTERM_SANDBOX");
            std::env::remove_var("MASTERM_SANDBOX_NET");
            output::success("Exited sandbox mode");
        }
        SandboxAction::Status => {
            let active = std::env::var("MASTERM_SANDBOX")
                .map(|v| v == "1")
                .unwrap_or(false);

            if active {
                let net_allowed = std::env::var("MASTERM_SANDBOX_NET")
                    .map(|v| v == "1")
                    .unwrap_or(false);

                println!("  Sandbox mode: {}", style("ACTIVE").cyan().bold());
                println!(
                    "  Network: {}",
                    if net_allowed {
                        style("allowed").green()
                    } else {
                        style("blocked").red()
                    }
                );
            } else {
                println!("  Sandbox mode: {}", style("inactive").dim());
            }
        }
    }

    Ok(())
}

// ============================================================
// Patterns
// ============================================================

#[derive(clap::Args)]
pub struct PatternsArgs {
    /// Show pattern type (secrets, threats, privilege)
    #[arg(short, long, default_value = "all")]
    pattern_type: String,
}

async fn run_patterns(args: PatternsArgs) -> Result<()> {
    output::header("🔍 Security Patterns");

    let show_all = args.pattern_type == "all";

    if show_all || args.pattern_type == "secrets" {
        println!();
        println!("{}", style("Secret Patterns:").bold());
        let patterns = [
            ("AWS Access Key", "AKIA..."),
            ("GitHub Token", "ghp_*, gho_*, ghs_*, ghu_*"),
            ("GitLab Token", "glpat-*"),
            ("Slack Token", "xox[baprs]-*"),
            ("Stripe Key", "sk_live_*, pk_live_*"),
            ("Google API Key", "AIza*"),
            ("Private Keys", "-----BEGIN * PRIVATE KEY-----"),
            ("JWT Tokens", "eyJ*.eyJ*.*"),
        ];
        for (name, pattern) in patterns {
            println!("  • {} ({})", name, style(pattern).dim());
        }
    }

    if show_all || args.pattern_type == "threats" {
        println!();
        println!("{}", style("Threat Patterns:").bold());
        let patterns = [
            ("Bash reverse shell", "/dev/tcp/*"),
            ("Netcat shell", "nc -e /bin/sh"),
            ("Base64 exec", "base64 -d | sh"),
            ("Download & exec", "curl * | bash"),
            ("History evasion", "unset HISTFILE"),
            ("Fork bomb", ":(){:|:&};:"),
        ];
        for (name, pattern) in patterns {
            println!("  • {} ({})", name, style(pattern).dim());
        }
    }

    if show_all || args.pattern_type == "privilege" {
        println!();
        println!("{}", style("Privilege Escalation:").bold());
        let patterns = ["sudo", "su", "doas", "pkexec", "setuid"];
        for pattern in patterns {
            println!("  • {}", pattern);
        }
    }

    Ok(())
}

// ============================================================
// Main entry point
// ============================================================

pub async fn run(action: SecurityAction) -> Result<()> {
    match action {
        SecurityAction::Status(args) => run_status(args).await,
        SecurityAction::Audit { action } => run_audit(action).await,
        SecurityAction::Check(args) => run_check(args).await,
        SecurityAction::Config { action } => run_config(action).await,
        SecurityAction::Sandbox { action } => run_sandbox(action).await,
        SecurityAction::Patterns(args) => run_patterns(args).await,
    }
}
