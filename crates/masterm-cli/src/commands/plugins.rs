//! Plugins command

use super::output;
use anyhow::Result;
use clap::Subcommand;
use console::style;
use masterm_core::plugin::PluginLoader;

/// Plugins subcommands
#[derive(Subcommand)]
pub enum PluginsAction {
    /// List installed plugins
    List {
        /// Show detailed status
        #[arg(long)]
        status: bool,
    },

    /// Search for plugins in registry
    Search {
        /// Search query
        query: String,
    },

    /// Install a plugin
    Install {
        /// Plugin name or URL
        plugin: String,
    },

    /// Remove a plugin
    Remove {
        /// Plugin name
        plugin: String,
    },

    /// Update plugins
    Update {
        /// Specific plugin to update (all if not specified)
        plugin: Option<String>,
    },

    /// Enable a plugin
    Enable {
        /// Plugin name
        plugin: String,
    },

    /// Disable a plugin
    Disable {
        /// Plugin name
        plugin: String,
    },

    /// Show plugin details
    Info {
        /// Plugin name
        plugin: String,
    },
}

/// Run the plugins command
pub async fn run(action: PluginsAction) -> Result<()> {
    match action {
        PluginsAction::List { status } => list_plugins(status).await,
        PluginsAction::Search { query } => search_plugins(&query).await,
        PluginsAction::Install { plugin } => install_plugin(&plugin).await,
        PluginsAction::Remove { plugin } => remove_plugin(&plugin).await,
        PluginsAction::Update { plugin } => update_plugins(plugin).await,
        PluginsAction::Enable { plugin } => enable_plugin(&plugin).await,
        PluginsAction::Disable { plugin } => disable_plugin(&plugin).await,
        PluginsAction::Info { plugin } => show_plugin_info(&plugin).await,
    }
}

/// List installed plugins
async fn list_plugins(show_status: bool) -> Result<()> {
    output::header("Installed Plugins");

    let loader = PluginLoader::new();
    let manifests = loader.discover();

    if manifests.is_empty() {
        println!("\nNo plugins installed.");
        println!("Use 'masterm plugins install <name>' to add plugins.");
        return Ok(());
    }

    // Also show built-in plugins
    println!("\n{}", style("Built-in Plugins").bold());
    println!("{}", "─".repeat(50));

    let builtin = vec![
        ("git", "Git repository context", "1.0.0", true),
        ("env", "Environment detection", "1.0.0", true),
        ("prod-guard", "Production safety", "1.0.0", true),
        ("node", "Node.js detection", "1.0.0", true),
        ("python", "Python detection", "1.0.0", true),
        ("go", "Go detection", "1.0.0", true),
        ("rust", "Rust detection", "1.0.0", true),
    ];

    for (name, desc, version, enabled) in builtin {
        let status = if enabled {
            style("enabled").green()
        } else {
            style("disabled").dim()
        };

        if show_status {
            println!(
                "  {} {} {} - {}",
                output::SUCCESS,
                style(name).cyan().bold(),
                style(format!("v{}", version)).dim(),
                desc
            );
            println!("    Status: {}", status);
        } else {
            println!(
                "  {} {} {} [{}]",
                output::SUCCESS,
                style(name).cyan(),
                style(format!("v{}", version)).dim(),
                status
            );
        }
    }

    if !manifests.is_empty() {
        println!("\n{}", style("External Plugins").bold());
        println!("{}", "─".repeat(50));

        for manifest in manifests {
            println!(
                "  {} {} v{}",
                output::SUCCESS,
                style(&manifest.plugin.name).cyan(),
                manifest.plugin.version
            );

            if show_status {
                println!("    {}", manifest.plugin.description);
            }
        }
    }

    println!();
    Ok(())
}

/// Search plugins
async fn search_plugins(query: &str) -> Result<()> {
    output::header(&format!("Searching for '{}'", query));

    // In reality, this would query a plugin registry
    println!("\n{} Searching plugin registry...\n", output::INFO);

    // Simulated results
    println!("No external plugins available yet.");
    println!("Plugin registry coming in v1.2!");

    Ok(())
}

/// Install a plugin
async fn install_plugin(plugin: &str) -> Result<()> {
    output::header(&format!("Installing plugin: {}", plugin));

    // In reality, this would:
    // 1. Fetch plugin from registry or URL
    // 2. Verify signature/checksum
    // 3. Check permissions
    // 4. Install to ~/.masterm/plugins/

    output::info("Plugin installation coming in v1.2!");

    Ok(())
}

/// Remove a plugin
async fn remove_plugin(plugin: &str) -> Result<()> {
    output::header(&format!("Removing plugin: {}", plugin));

    let plugin_dir = dirs::home_dir()
        .map(|h| h.join(".masterm/plugins").join(plugin))
        .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?;

    if !plugin_dir.exists() {
        output::error(&format!("Plugin '{}' is not installed", plugin));
        return Ok(());
    }

    std::fs::remove_dir_all(&plugin_dir)?;
    output::success(&format!("Removed plugin: {}", plugin));

    Ok(())
}

/// Update plugins
async fn update_plugins(plugin: Option<String>) -> Result<()> {
    match plugin {
        Some(p) => output::header(&format!("Updating plugin: {}", p)),
        None => output::header("Updating all plugins"),
    }

    output::info("Plugin updates coming in v1.2!");

    Ok(())
}

/// Enable a plugin
async fn enable_plugin(plugin: &str) -> Result<()> {
    output::info(&format!("Enabling plugin: {}", plugin));

    // This would update the config to enable the plugin
    // For now, just inform the user

    println!("To enable a plugin, edit your config:");
    println!("  masterm config edit");
    println!("\nAnd add to [plugins]:");
    println!("  enabled = [\"{}\"]", plugin);

    Ok(())
}

/// Disable a plugin
async fn disable_plugin(plugin: &str) -> Result<()> {
    output::info(&format!("Disabling plugin: {}", plugin));

    println!("To disable a plugin, edit your config:");
    println!("  masterm config edit");
    println!("\nAnd add to [plugins]:");
    println!("  disabled = [\"{}\"]", plugin);

    Ok(())
}

/// Show plugin info
async fn show_plugin_info(plugin: &str) -> Result<()> {
    output::header(&format!("Plugin: {}", plugin));

    // Check built-in plugins
    let builtin_info = match plugin {
        "git" | "git-plugin" => Some((
            "Git Plugin",
            "Displays git branch, status, and ahead/behind counts",
            vec!["git_branch", "git_status", "git_ahead_behind"],
            vec!["git"],
        )),
        "env" => Some((
            "Environment Plugin",
            "Detects development, staging, and production environments",
            vec!["env_type"],
            vec![],
        )),
        "prod-guard" => Some((
            "Production Guard Plugin",
            "Warns before running dangerous commands in production",
            vec![],
            vec![],
        )),
        _ => None,
    };

    if let Some((name, desc, segments, deps)) = builtin_info {
        println!("\n{}", style(name).bold());
        println!("{}", desc);

        if !segments.is_empty() {
            println!("\n{}", style("Segments Provided:").dim());
            for seg in segments {
                println!("  • {}", seg);
            }
        }

        if !deps.is_empty() {
            println!("\n{}", style("Dependencies:").dim());
            for dep in deps {
                println!("  • {}", dep);
            }
        }

        println!("\n{}: Built-in", style("Type").dim());
        println!("{}: 1.0.0", style("Version").dim());
    } else {
        output::warning(&format!("Plugin '{}' not found", plugin));
        println!("Use 'masterm plugins list' to see available plugins.");
    }

    Ok(())
}
