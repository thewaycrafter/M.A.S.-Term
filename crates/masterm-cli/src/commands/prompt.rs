//! Prompt command - generates the actual prompt string

use anyhow::Result;
use clap::Args;
use masterm_core::{
    config::{ConfigLoader, ShellType},
    context::ContextDetector,
    prompt::PromptRenderer,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::PathBuf;
use std::time::Duration;

static ANSI_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\x1b\[[0-9;]*m").expect("Invalid regex"));

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
    ANSI_REGEX.replace_all(s, "%{$0%}").to_string()
}

/// Escape ANSI codes for Bash
fn escape_bash(s: &str) -> String {
    // Bash uses \[ and \] to wrap non-printing characters
    ANSI_REGEX.replace_all(s, "\\[$0\\]").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_zsh_no_ansi() {
        let s = "/home/myuser/music";
        let escaped = escape_zsh(s);
        assert_eq!(escaped, s);
    }

    #[test]
    fn test_escape_zsh_with_ansi() {
        let s = "\x1b[31mmusic\x1b[0m";
        let escaped = escape_zsh(s);
        assert_eq!(escaped, "%{\x1b[31m%}music%{\x1b[0m%}");
    }

    #[test]
    fn test_escape_bash_with_ansi() {
        let s = "\x1b[31mmusic\x1b[0m";
        let escaped = escape_bash(s);
        assert_eq!(escaped, "\\[\x1b[31m\\]music\\[\x1b[0m\\]");
    }

    #[test]
    fn test_escape_mixed_content() {
        let s = "prefix \x1b[1mmiddle\x1b[0m suffix";
        let escaped = escape_zsh(s);
        assert_eq!(escaped, "prefix %{\x1b[1m%}middle%{\x1b[0m%} suffix");
    }
}
