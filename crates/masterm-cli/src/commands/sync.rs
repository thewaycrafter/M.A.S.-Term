//! Configuration synchronization command

use super::output;
use anyhow::{Context, Result};
use clap::Subcommand;
use console::style;
use masterm_core::cloud::GistSync;
use std::io::Write;

/// Sync subcommands
#[derive(Subcommand)]
pub enum SyncAction {
    /// Push configuration to a new Gist (backup)
    Push {
        /// GitHub Personal Access Token (or ensure GITHUB_TOKEN env var is set)
        #[arg(long, env = "GITHUB_TOKEN")]
        token: Option<String>,
        
        /// Description for the Gist
        #[arg(short, long, default_value = "MASTerm Configuration Backup")]
        description: String,
    },

    /// Pull configuration from an existing Gist
    Pull {
        /// Gist ID to download from
        gist_id: String,
        
        /// Overwrite local config without confirmation
        #[arg(long)]
        force: bool,
    },
}

/// Run the sync command
pub async fn run(action: SyncAction) -> Result<()> {
    match action {
        SyncAction::Push { token, description } => push_config(token, &description).await,
        SyncAction::Pull { gist_id, force } => pull_config(&gist_id, force).await,
    }
}

async fn push_config(token: Option<String>, description: &str) -> Result<()> {
    output::header("Sync: Push Configuration");

    // 1. Get token
    let token = match token {
        Some(t) => t,
        None => {
            print!("Enter GitHub Personal Access Token: ");
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    if token.is_empty() {
        return Err(anyhow::anyhow!("Token is required for upload"));
    }

    // 2. Read local config
    let config_path = dirs::home_dir()
        .map(|h| h.join(".masterm.toml"))
        .context("Could not determine config path")?;

    if !config_path.exists() {
        return Err(anyhow::anyhow!("No configuration file found at {}", config_path.display()));
    }

    let content = std::fs::read_to_string(&config_path)
        .context("Failed to read config file")?;

    // 3. Upload
    println!("{} Uploading config to GitHub Gist...", output::INFO);
    
    let client = GistSync::new(Some(token))?;
    let url = client.upload(".masterm.toml", content, description).await?;

    output::success("Successfully uploaded!");
    println!("  URL: {}", style(url).cyan().underlined());
    println!("  Save this URL or the Gist ID to restore later.");

    Ok(())
}

async fn pull_config(gist_id: &str, force: bool) -> Result<()> {
    output::header(&format!("Sync: Pull Configuration ({})", gist_id));

    // 1. Download
    println!("{} Downloading from Gist...", output::INFO);
    
    // Public download doesn't need token usually, but api limits apply. 
    // For simplicity we create client without token for pull
    let client = GistSync::new(None)?;
    let content = client.download(gist_id, ".masterm.toml").await?;

    // 2. Check local file
    let config_path = dirs::home_dir()
        .map(|h| h.join(".masterm.toml"))
        .context("Could not determine config path")?;

    if config_path.exists() && !force {
        println!("{} Local config exists at {}", output::WARNING, config_path.display());
        print!("Overwrite? [y/N] ");
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // 3. Write
    std::fs::write(&config_path, content)?;
    output::success(&format!("Restored config to {}", config_path.display()));

    Ok(())
}
