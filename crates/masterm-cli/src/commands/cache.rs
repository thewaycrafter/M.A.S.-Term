//! Cache management command

use super::output;
use anyhow::Result;
use clap::Subcommand;
use console::style;

/// Cache subcommands
#[derive(Subcommand)]
pub enum CacheAction {
    /// Clear all caches
    Clear,

    /// Show cache statistics
    Stats,
}

/// Run the cache command
pub async fn run(action: CacheAction) -> Result<()> {
    match action {
        CacheAction::Clear => clear_cache().await,
        CacheAction::Stats => show_stats().await,
    }
}

/// Clear cache
async fn clear_cache() -> Result<()> {
    let cache_dir = dirs::home_dir()
        .map(|h| h.join(".masterm/cache"))
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

    if cache_dir.exists() {
        std::fs::remove_dir_all(&cache_dir)?;
        std::fs::create_dir_all(&cache_dir)?;
        output::success("Cache cleared");
    } else {
        output::info("Cache directory doesn't exist");
    }

    Ok(())
}

/// Show cache statistics
async fn show_stats() -> Result<()> {
    println!("{}", style("MASTerm Cache Statistics").bold());
    println!("{}", "═".repeat(60));
    println!();

    let cache_dir = dirs::home_dir()
        .map(|h| h.join(".masterm/cache"))
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

    if !cache_dir.exists() {
        output::info("Cache directory doesn't exist");
        return Ok(());
    }

    // Count files and size
    let mut file_count = 0;
    let mut total_size: u64 = 0;

    for entry in walkdir::WalkDir::new(&cache_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            file_count += 1;
            total_size += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }

    println!("{:<25} {:>20}", "Cache directory:", cache_dir.display());
    println!("{:<25} {:>20}", "Cached entries:", file_count);
    println!("{:<25} {:>20}", "Total size:", format_bytes(total_size));

    // Note: In a full implementation, we'd track hit rates
    println!("\n{}", style("Note:").dim());
    println!("{}", style("Cache hit tracking coming in v1.2").dim());

    Ok(())
}

/// Format bytes as human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}
