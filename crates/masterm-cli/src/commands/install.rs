//! Install and uninstall commands

use super::output;
use anyhow::Result;
use clap::Args;
use console::style;
use masterm_core::config::ShellType;
use std::path::{Path, PathBuf};

/// Install command arguments
#[derive(Args)]
pub struct InstallArgs {
    /// Shell to install for (auto-detect if not specified)
    #[arg(short, long)]
    pub shell: Option<String>,

    /// Install system-wide (requires sudo)
    #[arg(long)]
    pub global: bool,

    /// Skip shell integration (just install binary)
    #[arg(long)]
    pub no_shell: bool,
}

/// Uninstall command arguments
#[derive(Args)]
pub struct UninstallArgs {
    /// Shell to uninstall from
    #[arg(short, long)]
    shell: Option<String>,
}

/// Run the install command
pub async fn run(args: InstallArgs) -> Result<()> {
    output::header("MASTerm Installation");

    // Detect shell
    let shell = match &args.shell {
        Some(s) => ShellType::from_path(s),
        None => ShellType::detect(),
    };

    println!("\n{} Detected shell: {}", output::INFO, style(shell).cyan());

    // Create directories
    let masterm_dir = get_masterm_dir()?;
    std::fs::create_dir_all(&masterm_dir)?;
    std::fs::create_dir_all(masterm_dir.join("plugins"))?;
    std::fs::create_dir_all(masterm_dir.join("cache"))?;

    output::success(&format!("Created directory: {}", masterm_dir.display()));

    // Copy shell scripts
    if !args.no_shell {
        install_shell_integration(shell, &masterm_dir)?;
    }

    // Create default config if needed
    let config_path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
        .join(".masterm.toml");

    if !config_path.exists() {
        create_default_config(&config_path)?;
        output::success(&format!("Created config: {}", config_path.display()));
    }

    // Add to shell rc file
    if !args.no_shell {
        add_shell_init(shell)?;
    }

    // Success message
    println!("\n{}", style("Installation complete!").green().bold());
    println!("\nTo apply changes, run:");
    println!("  {} {}", style("exec").cyan(), style("$SHELL").dim());
    println!("\nOr restart your terminal.\n");

    // Run doctor
    println!("Running diagnostics...\n");
    super::doctor::run(super::doctor::DoctorArgs { fix: false }).await?;

    Ok(())
}

/// Run the uninstall command
pub async fn uninstall(args: UninstallArgs) -> Result<()> {
    output::header("MASTerm Uninstallation");

    let shell = match &args.shell {
        Some(s) => ShellType::from_path(s),
        None => ShellType::detect(),
    };

    // Remove shell init line
    remove_shell_init(shell)?;

    output::success("Removed shell integration");

    // Note: We don't remove the binary or config, in case user wants to reinstall

    println!("\n{}", style("Uninstallation complete!").green().bold());
    println!("\nTo remove all MASTerm files, run:");
    println!("  rm -rf ~/.masterm ~/.masterm.toml");
    println!("\nRestart your terminal to apply changes.\n");

    Ok(())
}

/// Get MASTerm directory path
fn get_masterm_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))
        .map(|h| h.join(".masterm"))
}

/// Install shell integration scripts
fn install_shell_integration(shell: ShellType, masterm_dir: &Path) -> Result<()> {
    let shell_dir = masterm_dir.join("shell");
    std::fs::create_dir_all(&shell_dir)?;

    // Write shell scripts
    match shell {
        ShellType::Zsh => {
            std::fs::write(
                shell_dir.join("init.zsh"),
                include_str!("../../shell/init.zsh"),
            )?;
        }
        ShellType::Bash => {
            std::fs::write(
                shell_dir.join("init.bash"),
                include_str!("../../shell/init.bash"),
            )?;
        }
        ShellType::Fish => {
            std::fs::write(
                shell_dir.join("init.fish"),
                include_str!("../../shell/init.fish"),
            )?;
        }
        _ => {}
    }

    output::success("Installed shell scripts");
    Ok(())
}

/// Add initialization line to shell rc file
fn add_shell_init(shell: ShellType) -> Result<()> {
    let (rc_path, init_line) = get_shell_rc_info(shell)?;

    if !rc_path.exists() {
        std::fs::write(&rc_path, "")?;
    }

    let content = std::fs::read_to_string(&rc_path)?;

    if !content.contains("masterm init") {
        let mut file = std::fs::OpenOptions::new().append(true).open(&rc_path)?;

        use std::io::Write;
        writeln!(file, "\n# MASTerm - Master your Terminal")?;
        writeln!(file, "{}", init_line)?;

        output::success(&format!("Added init to {}", rc_path.display()));
    } else {
        output::info("Shell init already configured");
    }

    Ok(())
}

/// Remove initialization line from shell rc file
fn remove_shell_init(shell: ShellType) -> Result<()> {
    let (rc_path, _) = get_shell_rc_info(shell)?;

    if !rc_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&rc_path)?;
    let new_content: String = content
        .lines()
        .filter(|line| !line.contains("masterm") && !line.contains("MASTerm"))
        .collect::<Vec<_>>()
        .join("\n");

    std::fs::write(&rc_path, new_content)?;

    Ok(())
}

/// Get shell RC file path and init line
fn get_shell_rc_info(shell: ShellType) -> Result<(PathBuf, String)> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

    match shell {
        ShellType::Zsh => Ok((
            home.join(".zshrc"),
            r#"eval "$(masterm init zsh)""#.to_string(),
        )),
        ShellType::Bash => Ok((
            home.join(".bashrc"),
            r#"eval "$(masterm init bash)""#.to_string(),
        )),
        ShellType::Fish => Ok((
            home.join(".config/fish/config.fish"),
            "masterm init fish | source".to_string(),
        )),
        _ => Err(anyhow::anyhow!("Unsupported shell: {}", shell)),
    }
}

/// Create default configuration file
fn create_default_config(path: &PathBuf) -> Result<()> {
    let default_config = r#"# MASTerm Configuration
# https://masterm.dev/docs/configuration

[core]
# Shell (auto-detected if not set)
# shell = "zsh"

# Mode: minimal, dev, ops
mode = "dev"

[prompt]
# Prompt style: powerline, simple, minimal
format = "powerline"

# Show newline before prompt
add_newline = true

# Left prompt segments
left = ["directory", "git_branch", "git_status"]

# Right prompt segments
right = ["cmd_duration"]

[prompt.icons]
# Icon mode: auto, nerd, unicode, ascii, none
mode = "auto"

[prompt.colors]
# Theme: catppuccin, dracula, nord
theme = "catppuccin"

[safety]
# Enable production environment detection
prod_detection = true

# Commands requiring confirmation in prod
dangerous_commands = [
    "rm -rf",
    "DROP DATABASE",
    "kubectl delete",
    "terraform destroy"
]
"#;

    std::fs::write(path, default_config)?;
    Ok(())
}
