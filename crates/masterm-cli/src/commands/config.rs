//! Config command

use super::output;
use anyhow::Result;
use clap::Subcommand;
use console::style;
use masterm_core::config::ConfigLoader;

/// Config subcommands
#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show {
        /// Show effective (merged) configuration
        #[arg(long)]
        effective: bool,
    },

    /// Open configuration in editor
    Edit,

    /// Reset configuration to defaults
    Reset {
        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },

    /// Validate configuration syntax
    Validate,
}

/// Run the config command
pub async fn run(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Show { effective } => show_config(effective).await,
        ConfigAction::Edit => edit_config().await,
        ConfigAction::Reset { force } => reset_config(force).await,
        ConfigAction::Validate => validate_config().await,
    }
}

/// Show configuration
async fn show_config(effective: bool) -> Result<()> {
    let config_path = dirs::home_dir()
        .map(|h| h.join(".masterm.toml"))
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

    if effective {
        // Show merged configuration
        let loader = ConfigLoader::new();
        let cwd = std::env::current_dir()?;
        let config = loader.load(&cwd)?;

        println!("{}", style("Effective Configuration").bold());
        println!("{}", "─".repeat(40));
        println!("{}", toml::to_string_pretty(&config)?);
    } else {
        // Show raw config file
        if config_path.exists() {
            println!("{}", style(format!("Config: {}", config_path.display())).dim());
            println!("{}", "─".repeat(40));
            println!("{}", std::fs::read_to_string(&config_path)?);
        } else {
            output::warning("No configuration file found.");
            println!("Create one with: masterm config edit");
        }
    }

    Ok(())
}

/// Edit configuration
async fn edit_config() -> Result<()> {
    let config_path = dirs::home_dir()
        .map(|h| h.join(".masterm.toml"))
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

    // Create default if doesn't exist
    if !config_path.exists() {
        output::info("Creating default configuration...");
        super::install::run(super::install::InstallArgs {
            shell: None,
            global: false,
            no_shell: true,
        }).await?;
    }

    // Open in editor
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    output::info(&format!("Opening {} in {}", config_path.display(), editor));

    std::process::Command::new(&editor)
        .arg(&config_path)
        .status()?;

    // Validate after edit
    output::info("Validating configuration...");
    validate_config().await?;

    Ok(())
}

/// Reset configuration
async fn reset_config(force: bool) -> Result<()> {
    let config_path = dirs::home_dir()
        .map(|h| h.join(".masterm.toml"))
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

    if !force {
        println!("This will reset your configuration to defaults.");
        println!("Your current config will be backed up.");
        print!("Continue? [y/N] ");

        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Backup current config
    if config_path.exists() {
        let backup_path = config_path.with_extension("toml.bak");
        std::fs::copy(&config_path, &backup_path)?;
        output::info(&format!("Backed up to {}", backup_path.display()));
    }

    // Remove and recreate
    if config_path.exists() {
        std::fs::remove_file(&config_path)?;
    }

    // This will create a fresh default config
    super::install::run(super::install::InstallArgs {
        shell: None,
        global: false,
        no_shell: true,
    }).await?;

    output::success("Configuration reset to defaults");

    Ok(())
}

/// Validate configuration
async fn validate_config() -> Result<()> {
    let loader = ConfigLoader::new();
    let cwd = std::env::current_dir()?;

    match loader.load(&cwd) {
        Ok(_) => {
            output::success("Configuration is valid");
            Ok(())
        }
        Err(e) => {
            output::error(&format!("Configuration error: {}", e));
            Err(e)
        }
    }
}
