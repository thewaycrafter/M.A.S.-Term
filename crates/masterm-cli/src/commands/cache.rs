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
    masterm_core::cache::CacheManager::init()?;
    masterm_core::cache::CacheManager::clear()?;
    output::success("Cache cleared");
    Ok(())
}

/// Show cache statistics
async fn show_stats() -> Result<()> {
    println!("{}", style("MASTerm Cache Statistics").bold());
    println!("{}", "═".repeat(60));

    masterm_core::cache::CacheManager::init()?;
    let (count, _size) = masterm_core::cache::CacheManager::stats()?;

    let cache_path = dirs::home_dir()
        .map(|h| h.join(".masterm/cache.db"))
        .unwrap_or_else(|| std::path::PathBuf::from(".masterm_cache.db"));

    println!("{:<25} {:>20}", "Cache path:", cache_path.display());
    println!("{:<25} {:>20}", "Cached entries:", count);
    
    // Check actual file size
    let file_size = std::fs::metadata(&cache_path).map(|m| m.len()).unwrap_or(0);
    println!("{:<25} {:>20}", "Database size:", format_bytes(file_size));

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
