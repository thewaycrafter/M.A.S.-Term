//! Prompt command - generates the actual prompt string

use anyhow::Result;
use clap::Args;
use masterm_core::{
    config::{ConfigLoader, ShellType},
    context::ContextDetector,
    prompt::PromptRenderer,
};
use std::path::PathBuf;
use std::time::Duration;

/// Prompt command arguments
#[derive(Args)]
pub struct PromptArgs {
    /// Shell type
    #[arg(long)]
    shell: String,

    /// Last command exit code
    #[arg(long, default_value = "0")]
    exit_code: i32,

    /// Last command duration in milliseconds
    #[arg(long, default_value = "0")]
    duration: u64,

    /// Current working directory (uses CWD if not specified)
    #[arg(long)]
    cwd: Option<PathBuf>,
}

/// Run the prompt command
pub async fn run(args: PromptArgs) -> Result<()> {
    // Get CWD
    let cwd = args
        .cwd
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    // Load configuration
    let loader = ConfigLoader::new();
    let config = loader.load(&cwd)?;

    // Detect context
    let mut detector =
        ContextDetector::new().with_prod_patterns(config.safety.prod_patterns.clone());

    let context = detector.detect(&cwd).await?;

    // Create renderer
    let renderer = PromptRenderer::new(config.prompt);

    // Get duration
    let duration = Duration::from_millis(args.duration);

    // Render prompt
    let prompt = renderer.render(&context, vec![], args.exit_code, duration);

    // Output based on shell type
    let shell = ShellType::from_path(&args.shell);

    match shell {
        ShellType::Zsh => {
            // Zsh needs %{ %} around escape codes
            let escaped = escape_zsh(&prompt.left);
            print!("{}", escaped);
        }
        ShellType::Bash => {
            // Bash needs \[ \] around escape codes
            let escaped = escape_bash(&prompt.left);
            print!("{}", escaped);
        }
        _ => {
            // Fish and others handle escape codes natively
            print!("{}", prompt.left);
        }
    }

    Ok(())
}

/// Escape ANSI codes for Zsh
fn escape_zsh(s: &str) -> String {
    // Zsh uses %{ and %} to wrap non-printing characters
    // This is a simplified version; a full implementation would parse ANSI codes
    s.replace("\x1b[", "%{\x1b[")
        .replace("m", "m%}")
        .replace("%}%{", "")
}

/// Escape ANSI codes for Bash
fn escape_bash(s: &str) -> String {
    // Bash uses \[ and \] to wrap non-printing characters
    s.replace("\x1b[", "\\[\x1b[")
        .replace("m", "m\\]")
        .replace("\\]\\[", "")
}
