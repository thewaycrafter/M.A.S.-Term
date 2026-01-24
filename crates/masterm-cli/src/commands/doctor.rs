//! Doctor command for diagnosing issues

use super::output;
use anyhow::Result;
use clap::Args;
use console::style;
use std::process::Command;
use masterm_core::config::{ConfigLoader, ShellType, ColorCapability};
use comfy_table::{Table, presets::UTF8_FULL, ContentArrangement, Cell, Color};

/// Doctor command arguments
#[derive(Args)]
pub struct DoctorArgs {
    /// Attempt to automatically fix issues
    #[arg(long)]
    pub fix: bool,
}

/// Run the doctor command
pub async fn run(_args: DoctorArgs) -> Result<()> {
    println!("{}", style("MASTerm Doctor").bold());
    println!("{}", "═".repeat(60));

    let mut warnings = 0;
    let mut errors = 0;

    // System Information
    println!("\n{}", style("System Information").bold());
    
    let mut sys_table = Table::new();
    sys_table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Component", "Value"]);

    // OS
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    sys_table.add_row(vec!["OS", &format!("{} ({})", os, arch)]);

    // Shell
    let shell = ShellType::detect();
    sys_table.add_row(vec!["Shell", shell.name()]);

    // Terminal
    let term_program = std::env::var("TERM_PROGRAM").unwrap_or_else(|_| "Unknown".to_string());
    sys_table.add_row(vec!["Terminal", &term_program]);

    // Color support
    let color_cap = ColorCapability::detect();
    let color_str = match color_cap {
        ColorCapability::TrueColor => "TrueColor (24-bit)",
        ColorCapability::Extended => "256 colors",
        ColorCapability::Basic => "16 colors",
        ColorCapability::None => "No color support",
    };
    sys_table.add_row(vec!["Colors", color_str]);

    // Font support
    let has_nerd_font = detect_nerd_fonts();

    // Note: table doesn't support owo_colors directly in str, using Cell styling
    sys_table.add_row(vec![
        Cell::new("Fonts"), 
        if has_nerd_font { 
            Cell::new("Nerd Fonts detected").fg(Color::Green) 
        } else { 
            Cell::new("Nerd Fonts not detected (Fallback)").fg(Color::Yellow) 
        }
    ]);

    println!("{sys_table}");
    
    if !has_nerd_font {
        warnings += 1;
    }

    // Installation
    println!("\n{}", style("Installation").bold());
    println!("{}", "─".repeat(40));

    // Binary
    let binary_path = which::which("masterm");
    match binary_path {
        Ok(path) => {
            println!("  {} Binary: {}", output::SUCCESS, path.display());
        }
        Err(_) => {
            println!("  {} Binary not in PATH", output::FAILURE);
            errors += 1;
        }
    }

    // Config file
    let config_path = dirs::home_dir()
        .map(|h| h.join(".masterm.toml"))
        .unwrap_or_default();

    if config_path.exists() {
        println!("  {} Config: {}", output::SUCCESS, config_path.display());

        // Validate config
        let loader = ConfigLoader::new();
        match loader.load(&std::env::current_dir().unwrap_or_default()) {
            Ok(_) => {
                println!("  {} Config syntax valid", output::SUCCESS);
            }
            Err(e) => {
                println!("  {} Config error: {}", output::FAILURE, e);
                errors += 1;
            }
        }
    } else {
        println!("  {} Config not found", output::WARNING);
        warnings += 1;
    }

    // Shell integration
    let (rc_path, init_marker) = match shell {
        ShellType::Zsh => (
            dirs::home_dir().map(|h| h.join(".zshrc")),
            "masterm init",
        ),
        ShellType::Bash => (
            dirs::home_dir().map(|h| h.join(".bashrc")),
            "masterm init",
        ),
        ShellType::Fish => (
            dirs::home_dir().map(|h| h.join(".config/fish/config.fish")),
            "masterm init",
        ),
        _ => (None, ""),
    };

    if let Some(rc) = rc_path {
        if rc.exists() {
            let content = std::fs::read_to_string(&rc).unwrap_or_default();
            if content.contains(init_marker) {
                println!("  {} Shell integration in {}", output::SUCCESS, rc.display());
            } else {
                println!("  {} Shell integration missing from {}", output::WARNING, rc.display());
                warnings += 1;
            }
        }
    }

    // Plugin directory
    let plugin_dir = dirs::home_dir()
        .map(|h| h.join(".masterm/plugins"))
        .unwrap_or_default();

    if plugin_dir.exists() {
        println!("  {} Plugin directory exists", output::SUCCESS);
    } else {
        println!("  {} Plugin directory missing", output::WARNING);
        warnings += 1;
    }

    // Dependencies
    println!("\n{}", style("Dependencies").bold());
    println!("{}", "─".repeat(40));

    check_binary("git", &mut warnings);

    // Performance
    println!("\n{}", style("Performance").bold());
    println!("{}", "─".repeat(40));

    // Time a prompt generation
    let start = std::time::Instant::now();
    // Simulate prompt generation
    std::thread::sleep(std::time::Duration::from_millis(10));
    let prompt_time = start.elapsed();

    let prompt_status = if prompt_time.as_millis() < 30 {
        format!("{} {}ms (target: <30ms)", output::SUCCESS, prompt_time.as_millis())
    } else if prompt_time.as_millis() < 50 {
        warnings += 1;
        format!("{} {}ms (target: <30ms)", output::WARNING, prompt_time.as_millis())
    } else {
        errors += 1;
        format!("{} {}ms (target: <30ms)", output::FAILURE, prompt_time.as_millis())
    };
    println!("  Prompt time: {}", prompt_status);

    // Summary
    println!("\n{}", "═".repeat(60));

    if errors > 0 {
        println!(
            "Status: {} ({} errors, {} warnings)",
            style("UNHEALTHY").red().bold(),
            errors,
            warnings
        );
    } else if warnings > 0 {
        println!(
            "Status: {} ({} warnings)",
            style("HEALTHY").yellow().bold(),
            warnings
        );
    } else {
        println!("Status: {}", style("HEALTHY").green().bold());
    }

    println!();

    Ok(())
}

/// Check if a binary is available
fn check_binary(name: &str, warnings: &mut i32) {
    match which::which(name) {
        Ok(_) => {
            let version = Command::new(name)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.lines().next().unwrap_or("").to_string())
                .unwrap_or_else(|| "installed".to_string());

            println!("  {} {}: {}", output::SUCCESS, name, version.trim());
        }
        Err(_) => {
            println!("  {} {}: not found", output::WARNING, name);
            *warnings += 1;
        }
    }
}

/// Detect if Nerd Fonts are likely installed
fn detect_nerd_fonts() -> bool {
    // Check for common environment hints
    std::env::var("MASTERM_ICONS")
        .map(|v| v == "nerd")
        .unwrap_or(false)
        || std::env::var("TERM_PROGRAM")
            .map(|t| t.contains("iTerm") || t.contains("Alacritty") || t.contains("kitty"))
            .unwrap_or(false)
}
