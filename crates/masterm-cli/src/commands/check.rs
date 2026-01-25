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
        CommandAction::Allow => exit(0),
        CommandAction::Warn(msg) => {
            eprintln!("{}", msg);
            exit(0);
        }
        CommandAction::Confirm(msg) => {
            // Interactive confirmation
            eprintln!("{}", msg);
            io::stderr().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if input.trim().eq_ignore_ascii_case("yes") {
                exit(0);
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
