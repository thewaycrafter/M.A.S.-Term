//! Dashboard command to launch TUI

use anyhow::Result;
use clap::Args;

/// Dashboard command arguments
#[derive(Args, Debug)]
pub struct DashboardArgs {}

/// Run the dashboard command
pub async fn run(_args: DashboardArgs) -> Result<()> {
    masterm_tui::run().await?;
    Ok(())
}
