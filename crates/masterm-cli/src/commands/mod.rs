//! CLI command modules

pub mod cache;
pub mod completions;
pub mod config;
pub mod dashboard;
pub mod doctor;
pub mod init;
pub mod install;
pub mod mode;
pub mod plugins;
pub mod profile;
pub mod prompt;
pub mod sync;
pub mod update;



/// Output formatting helpers
pub mod output {
    use console::style;

    /// Success indicator
    pub const SUCCESS: &str = "✓";
    /// Failure indicator
    pub const FAILURE: &str = "✗";
    /// Warning indicator
    pub const WARNING: &str = "⚠";
    /// Info indicator
    pub const INFO: &str = "ℹ";

    /// Print a success message
    pub fn success(msg: &str) {
        println!("  {} {}", style(SUCCESS).green(), msg);
    }

    /// Print an error message
    pub fn error(msg: &str) {
        eprintln!("  {} {}", style(FAILURE).red(), msg);
    }

    /// Print a warning message
    pub fn warning(msg: &str) {
        println!("  {} {}", style(WARNING).yellow(), msg);
    }

    /// Print an info message
    pub fn info(msg: &str) {
        println!("  {} {}", style(INFO).blue(), msg);
    }

    /// Print a header
    pub fn header(text: &str) {
        println!();
        println!("{}", style(text).bold());
        println!("{}", "═".repeat(60));
    }


}
