//! Welcome command

use anyhow::Result;
use clap::Args;
use console::style;
use sysinfo::System;

#[derive(Args)]
pub struct WelcomeArgs;

pub async fn run(_args: WelcomeArgs) -> Result<()> {
    // Clear screen
    print!("\x1b[2J\x1b[1;1H");

    // ASCII Art
    let banner = r#"
    __  ___ ___   _____ ______                     
   /  |/  //   | / ___//_  __/___   _____ ____ ___ 
  / /|_/ // /| | \__ \  / /  / _ \ / ___// __ `__ \
 / /  / // ___ |___/ / / /  /  __// /   / / / / / /
/_/  /_//_/  |_/____/ /_/   \___//_/   /_/ /_/ /_/ 
    "#;

    println!("{}", style(banner).cyan().bold());
    println!("     {}", style("Master your Terminal").italic().dim());
    println!();

    // System Stats
    let mut sys = System::new();
    sys.refresh_memory();
    sys.refresh_cpu();

    // Simulate slight delay for CPU reading or just show static info
    let total_mem = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let used_mem = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;

    println!(
        "  🖥️  {} CPUs | {:.1} GB / {:.1} GB RAM",
        sys.cpus().len(),
        used_mem,
        total_mem
    );
    println!("  🚀 Version: v1.0.0");
    println!("  📢 Tip: Run 'masterm setup' to customize this shell.");
    println!();

    Ok(())
}
