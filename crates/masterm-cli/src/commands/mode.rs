//! Mode command for quick switching

use super::output;
use anyhow::Result;
use clap::Args;
use console::style;
use masterm_core::config::{ConfigLoader, Mode};

/// Mode command arguments
#[derive(Args)]
pub struct ModeArgs {
    /// Mode to switch to (minimal, dev, ops)
    mode: Option<String>,
}

/// Run the mode command
pub async fn run(args: ModeArgs) -> Result<()> {
    match args.mode {
        Some(mode_str) => set_mode(&mode_str).await,
        None => show_mode().await,
    }
}

/// Show current mode
async fn show_mode() -> Result<()> {
    let loader = ConfigLoader::new();
    let cwd = std::env::current_dir()?;
    let config = loader.load(&cwd)?;

    let mode = Mode::from_str(&config.core.mode).unwrap_or_default();

    println!("{}", style("Current Mode").bold());
    println!("{}", "─".repeat(40));
    println!();

    for m in [Mode::Minimal, Mode::Dev, Mode::Ops] {
        let indicator = if m == mode { "●" } else { "○" };
        let name = style(m.name()).bold();
        let desc = style(m.description()).dim();

        if m == mode {
            println!("  {} {} - {}", style(indicator).green(), name, desc);
        } else {
            println!("  {} {} - {}", style(indicator).dim(), name, desc);
        }
    }

    println!();
    println!("Switch with: masterm mode <minimal|dev|ops>");

    Ok(())
}

/// Set mode
async fn set_mode(mode_str: &str) -> Result<()> {
    let mode = Mode::from_str(mode_str)
        .ok_or_else(|| anyhow::anyhow!("Invalid mode: {}. Use: minimal, dev, or ops", mode_str))?;

    // Read and update config
    let config_path = dirs::home_dir()
        .map(|h| h.join(".masterm.toml"))
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

    if !config_path.exists() {
        output::error("No configuration file found. Run 'masterm install' first.");
        return Ok(());
    }

    let content = std::fs::read_to_string(&config_path)?;
    let mut config: toml::Value = toml::from_str(&content)?;

    // Update mode
    if let Some(core) = config.get_mut("core") {
        if let Some(table) = core.as_table_mut() {
            table.insert("mode".to_string(), toml::Value::String(mode.name().to_string()));
        }
    }

    // Write back
    std::fs::write(&config_path, toml::to_string_pretty(&config)?)?;

    output::success(&format!("Switched to {} mode", style(mode.name()).cyan()));
    println!("{}", style(mode.description()).dim());

    // Show mode-specific info
    match mode {
        Mode::Minimal => {
            println!("\n{}", style("Minimal mode:").bold());
            println!("  • Fastest startup");
            println!("  • Basic prompt only");
            println!("  • No plugins loaded");
        }
        Mode::Dev => {
            println!("\n{}", style("Dev mode:").bold());
            println!("  • Balanced performance");
            println!("  • Git, language detection enabled");
            println!("  • Standard plugins loaded");
        }
        Mode::Ops => {
            println!("\n{}", style("Ops mode:").bold());
            println!("  • Maximum safety features");
            println!("  • Production guards enabled");
            println!("  • Command confirmations active");
        }
    }

    Ok(())
}
