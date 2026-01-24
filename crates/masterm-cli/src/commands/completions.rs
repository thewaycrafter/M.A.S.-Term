//! Shell completions command

use anyhow::Result;
use clap::Args;
use clap_complete::{generate, Shell};
use std::io;

/// Completions command arguments
#[derive(Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    shell: String,
}

/// Run the completions command
pub fn run(args: CompletionsArgs) -> Result<()> {
    let shell = match args.shell.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" | "pwsh" => Shell::PowerShell,
        "elvish" => Shell::Elvish,
        _ => {
            eprintln!("Unsupported shell: {}", args.shell);
            eprintln!("Supported shells: bash, zsh, fish, powershell, elvish");
            std::process::exit(1);
        }
    };

    // Build CLI for completions
    let mut cmd = build_cli();

    generate(shell, &mut cmd, "masterm", &mut io::stdout());

    Ok(())
}

/// Build the CLI structure for completions
fn build_cli() -> clap::Command {
    clap::Command::new("masterm")
        .subcommand(
            clap::Command::new("install")
                .about("Install MASTerm for your shell")
                .arg(clap::Arg::new("shell").short('s').long("shell")),
        )
        .subcommand(clap::Command::new("uninstall").about("Uninstall MASTerm from your shell"))
        .subcommand(
            clap::Command::new("update")
                .about("Update MASTerm to the latest version")
                .arg(clap::Arg::new("check").long("check")),
        )
        .subcommand(
            clap::Command::new("doctor")
                .about("Diagnose installation issues")
                .arg(clap::Arg::new("fix").long("fix")),
        )
        .subcommand(
            clap::Command::new("init")
                .about("Initialize shell integration")
                .arg(clap::Arg::new("shell").required(true)),
        )
        .subcommand(
            clap::Command::new("config")
                .about("Configuration management")
                .subcommand(clap::Command::new("show"))
                .subcommand(clap::Command::new("edit"))
                .subcommand(clap::Command::new("reset"))
                .subcommand(clap::Command::new("validate")),
        )
        .subcommand(
            clap::Command::new("plugins")
                .about("Plugin management")
                .subcommand(clap::Command::new("list"))
                .subcommand(
                    clap::Command::new("search").arg(clap::Arg::new("query").required(true)),
                )
                .subcommand(
                    clap::Command::new("install").arg(clap::Arg::new("plugin").required(true)),
                )
                .subcommand(
                    clap::Command::new("remove").arg(clap::Arg::new("plugin").required(true)),
                )
                .subcommand(clap::Command::new("update"))
                .subcommand(
                    clap::Command::new("info").arg(clap::Arg::new("plugin").required(true)),
                ),
        )
        .subcommand(
            clap::Command::new("mode")
                .about("Switch modes")
                .arg(clap::Arg::new("mode").value_parser(["minimal", "dev", "ops"])),
        )
        .subcommand(
            clap::Command::new("profile")
                .about("Performance profiling")
                .subcommand(clap::Command::new("startup"))
                .subcommand(clap::Command::new("prompt"))
                .subcommand(clap::Command::new("plugins")),
        )
        .subcommand(
            clap::Command::new("cache")
                .about("Cache management")
                .subcommand(clap::Command::new("clear"))
                .subcommand(clap::Command::new("stats")),
        )
        .subcommand(
            clap::Command::new("completions")
                .about("Generate shell completions")
                .arg(clap::Arg::new("shell").required(true).value_parser([
                    "bash",
                    "zsh",
                    "fish",
                    "powershell",
                ])),
        )
}
