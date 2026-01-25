use clap::Args;
use masterm_core::context::EnvironmentType;
use masterm_core::plugin::CommandAction;
use masterm_core::safety::{ProdGuard, SafetyGuard};
use std::io::{self, Write};
use std::process::exit;

#[derive(Debug, Args)]
pub struct CheckArgs {
    /// The command to check
    #[arg(trailing_var_arg = true)]
    command: Vec<String>,
}

pub async fn run(args: CheckArgs) -> anyhow::Result<()> {
    // Reconstruct command string
    let command = args.command.join(" ");
    if command.trim().is_empty() {
        return Ok(());
    }

    // Load configuration
    // We need config to check safety patterns and mode
    let cwd = std::env::current_dir()?;
    let loader = masterm_core::config::ConfigLoader::new();
    let config = loader.load(&cwd).unwrap_or_default();

    // Detect environment type using config patterns
    let env = EnvironmentType::detect(&cwd, &config.safety.prod_patterns);

    // Check if we are in OPS mode
    let mode_str = config.core.mode.to_lowercase();

    // Determine effective environment
    // If we are in 'ops' mode, we treat it as Production for safety checks if implied,
    // OR just rely on 'env' being accurate.
    // However, the issue description says "Ops Mode" should trigger guards.
    // If ops mode is ON, and we run a dangerous command, we might want to FORCE checks even if not strictly in a prod folder.
    // But ProdGuard usually only checks if environment.is_sensitive().
    // Let's coerce 'ops' mode to act like Staging at minimum if not checking paths?
    // Actually, let's just use the detected env. If the user wants to test it, they need to be in a prod folder OR force it.
    // BUT the user said `masterm mode ops` was active.
    // If `mode == ops`, maybe we force `EnvironmentType::Production` for safety?
    let effective_env = if mode_str == "ops" {
        EnvironmentType::Production
    } else {
        env
    };

    let guard = ProdGuard::new(effective_env);

    match guard.check(&command) {
        CommandAction::Allow => {
            // Check patterns even if environment allows
            check_patterns(&command).await;
            Ok(())
        }
        CommandAction::Warn(msg) => {
            eprintln!("{}", msg);
            check_patterns(&command).await;
            Ok(())
        }
        CommandAction::Confirm(msg) => {
            // Interactive confirmation
            eprintln!("{}", msg);
            io::stderr().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if input.trim().eq_ignore_ascii_case("yes") {
                check_patterns(&command).await;
                Ok(())
            } else {
                eprintln!("\n\x1b[31m❌ Command cancelled by user\x1b[0m");
                exit(1);
            }
        }
        CommandAction::Block(msg) => {
            eprintln!("{}", msg);
            exit(1);
        }
    }
}

async fn check_patterns(command: &str) {
    use masterm_security::audit::AuditLogger;
    use masterm_security::config::AuditConfig;
    use masterm_security::patterns::{RiskLevel, SecurityPatternMatcher};

    let matcher = SecurityPatternMatcher::new();
    let analysis = matcher.analyze(command);
    let risk = analysis.max_risk_level();

    // Audit Logging
    let audit_config = AuditConfig::load().unwrap_or_default();
    if audit_config.enabled {
        if let Ok(logger) = AuditLogger::new(audit_config).await {
            let cwd = std::env::current_dir().unwrap_or_default();
            // Best effort logging
            let _ = logger
                .log_command(command, &cwd, "unknown", "dev", analysis.security_flags())
                .await;
        }
    }

    if risk >= RiskLevel::High {
        eprintln!();
        eprintln!("\x1b[31m🚨 SECURITY ALERT: High Risk Command Detected\x1b[0m");

        for secret in &analysis.secrets {
            eprintln!(
                "   🔐 Secret: {} ({})",
                secret.category.name(),
                secret.matched_text
            );
        }
        for threat in &analysis.threats {
            eprintln!(
                "   ⚠️  Threat: {} - {}",
                threat.category.name(),
                threat.description
            );
        }

        if risk == RiskLevel::Critical {
            eprintln!("\n\x1b[31m❌ Command BLOCKED due to critical risk.\x1b[0m");
            exit(1);
        } else {
            // Confirm for High risk
            eprintln!("\nAre you sure you want to execute this? (yes/no)");
            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_ok()
                && input.trim().eq_ignore_ascii_case("yes")
            {
                return;
            }
            eprintln!("cancelled.");
            exit(1);
        }
    }
}
