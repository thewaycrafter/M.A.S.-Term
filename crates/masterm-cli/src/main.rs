//! MASTerm CLI - Master your Terminal
//!
//! Usage:
//!   masterm <command> [options]
//!
//! Commands:
//!   install     Install MASTerm for your shell
//!   update      Update MASTerm to the latest version
//!   doctor      Diagnose installation issues
//!   init        Initialize shell (called by shell scripts)
//!   prompt      Generate prompt (called by shell scripts)
//!   config      Configuration management
//!   plugins     Plugin management
//!   mode        Quick mode switching
//!   profile     Performance profiling
//!   security    Security management

mod commands;

use clap::{Parser, Subcommand};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// MASTerm - A fast, intelligent, cross-shell terminal framework
#[derive(Parser)]
#[command(name = "masterm")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable debug output
    #[arg(short, long, global = true)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Ask AI for help
    Ask(commands::ask::AskArgs),

    /// Run a workflow
    Run(commands::run::RunArgs),

    /// Install MASTerm for your shell
    Install(commands::install::InstallArgs),

    /// Uninstall MASTerm from your shell
    Uninstall(commands::install::UninstallArgs),

    /// Update MASTerm to the latest version
    Update(commands::update::UpdateArgs),

    /// Diagnose installation and configuration issues
    Doctor(commands::doctor::DoctorArgs),

    /// Initialize shell integration (called by shell scripts)
    Init(commands::init::InitArgs),

    /// Generate prompt string (called by shell scripts)
    Prompt(commands::prompt::PromptArgs),

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: commands::config::ConfigAction,
    },

    /// Plugin management
    Plugins {
        #[command(subcommand)]
        action: commands::plugins::PluginsAction,
    },

    /// Quick mode switching
    Mode(commands::mode::ModeArgs),

    /// Performance profiling
    Profile {
        #[command(subcommand)]
        action: commands::profile::ProfileAction,
    },

    /// Clear cache
    Cache {
        #[command(subcommand)]
        action: commands::cache::CacheAction,
    },

    /// Launch TUI dashboard
    Dashboard(commands::dashboard::DashboardArgs),

    /// Generate shell completions
    Completions(commands::completions::CompletionsArgs),

    /// Cloud synchronization
    Sync {
        #[command(subcommand)]
        action: commands::sync::SyncAction,
    },

    /// Interactive setup wizard
    Setup(commands::setup::SetupArgs),

    /// Show welcome screen
    Welcome(commands::welcome::WelcomeArgs),

    /// Check if a command is safe to run
    Check(commands::check::CheckArgs),

    /// Security management
    Security {
        #[command(subcommand)]
        action: commands::security::SecurityAction,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Set up logging
    let filter = if cli.debug {
        EnvFilter::new("debug")
    } else if cli.verbose {
        EnvFilter::new("info")
    } else {
        EnvFilter::new("warn")
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(filter)
        .init();

    // Execute command
    match cli.command {
        Commands::Ask(args) => commands::ask::run(args).await,
        Commands::Run(args) => commands::run::run(args).await,
        Commands::Install(args) => commands::install::run(args).await,
        Commands::Uninstall(args) => commands::install::uninstall(args).await,
        Commands::Update(args) => commands::update::run(args).await,
        Commands::Doctor(args) => commands::doctor::run(args).await,
        Commands::Init(args) => commands::init::run(args).await,
        Commands::Prompt(args) => commands::prompt::run(args).await,
        Commands::Config { action } => commands::config::run(action).await,
        Commands::Plugins { action } => commands::plugins::run(action).await,
        Commands::Mode(args) => commands::mode::run(args).await,
        Commands::Profile { action } => commands::profile::run(action).await,
        Commands::Cache { action } => commands::cache::run(action).await,
        Commands::Dashboard(args) => commands::dashboard::run(args).await,
        Commands::Completions(args) => commands::completions::run(args),
        Commands::Sync { action } => commands::sync::run(action).await,
        Commands::Setup(args) => commands::setup::run(args).await,
        Commands::Welcome(args) => commands::welcome::run(args).await,
        Commands::Check(args) => commands::check::run(args).await,
        Commands::Security { action } => commands::security::run(action).await,
    }
}
