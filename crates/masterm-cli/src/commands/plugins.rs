//! Plugins command

use super::output;
use anyhow::Result;
use clap::Subcommand;
use comfy_table::{presets::UTF8_HORIZONTAL_ONLY, Cell, Color, ContentArrangement, Table};
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

    // Built-in plugins data
    let builtin = vec![
        ("git", "Git repository context", "1.0.0", true),
        ("env", "Environment detection", "1.0.0", true),
        ("prod-guard", "Production safety", "1.0.0", true),
        ("node", "Node.js detection", "1.0.0", true),
        ("python", "Python detection", "1.0.0", true),
        ("go", "Go detection", "1.0.0", true),
        ("rust", "Rust detection", "1.0.0", true),
    ];

    let mut table = Table::new();
    table
        .load_preset(UTF8_HORIZONTAL_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Name", "Version", "Status", "Description"]);

    // Add built-in plugins
    for (name, desc, version, enabled) in builtin {
        let status_cell = if enabled {
            Cell::new("Active").fg(Color::Green)
        } else {
            Cell::new("Disabled").fg(Color::DarkGrey)
        };

        table.add_row(vec![
            Cell::new(name)
                .fg(Color::Cyan)
                .add_attribute(comfy_table::Attribute::Bold),
            Cell::new(format!("v{}", version)),
            status_cell,
            Cell::new(desc),
        ]);
    }

    // Add external plugins
    for manifest in manifests {
        let status_cell = Cell::new("Active").fg(Color::Green); // External plugins found are active by default for now

        table.add_row(vec![
            Cell::new(&manifest.plugin.name)
                .fg(Color::Cyan)
                .add_attribute(comfy_table::Attribute::Bold),
            Cell::new(format!("v{}", manifest.plugin.version)),
            status_cell,
            Cell::new(&manifest.plugin.description),
        ]);
    }

    println!("{table}");

    if !show_status {
        println!(
            "\n{}",
            style("Tip: Use --status for more details (coming soon in v1.2)").dim()
        );
    }

    Ok(())
}

/// Search plugins
async fn search_plugins(query: &str) -> Result<()> {
    output::header(&format!("Searching for '{}'", query));
    println!("{} Searching plugin registry...", style("ℹ").blue());

    // Registry simulation
    let registry = vec![
        ("docker-context", "Show current Docker context", "1.0.0"),
        ("aws-profile", "Show active AWS profile", "0.9.0"),
        ("kubectl-ns", "Show Kubernetes namespace", "1.2.0"),
    ];

    let mut table = Table::new();
    table
        .load_preset(UTF8_HORIZONTAL_ONLY)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Name", "Version", "Description"]);

    let mut found = false;
    for (name, desc, ver) in registry {
        if name.contains(query) || desc.to_lowercase().contains(&query.to_lowercase()) {
            table.add_row(vec![
                Cell::new(name)
                    .fg(Color::Cyan)
                    .add_attribute(comfy_table::Attribute::Bold),
                Cell::new(ver),
                Cell::new(desc),
            ]);
            found = true;
        }
    }

    if found {
        println!("\n{table}");
        println!(
            "\nTo install: {} install <name>",
            style("masterm plugins").bold()
        );
    } else {
        println!("\nNo plugins found matching '{}'", query);
    }

    Ok(())
}

/// Install a plugin
async fn install_plugin(plugin: &str) -> Result<()> {
    output::header(&format!("Installing plugin: {}", plugin));

    // Simple simulation for now
    let known_plugins = ["docker-context", "aws-profile", "kubectl-ns"];

    if !known_plugins.contains(&plugin) {
        output::error(&format!("Plugin '{}' not found in registry", plugin));
        return Ok(());
    }

    let plugin_dir = dirs::home_dir()
        .unwrap_or_default()
        .join(".masterm/plugins")
        .join(plugin);

    if plugin_dir.exists() {
        output::warning(&format!("Plugin '{}' is already installed", plugin));
        return Ok(());
    }

    std::fs::create_dir_all(&plugin_dir)?;

    // Create dummy manifest
    let manifest = format!(
        r#"
[plugin]
name = "{}"
version = "1.0.0"
description = "WebAssembly plugin for {}"
author = "Community"
license = "MIT"

[requirements]
binaries = []

[permissions]
filesystem = []
network = "none"
"#,
        plugin, plugin
    );

    std::fs::write(plugin_dir.join("plugin.toml"), manifest)?;
    std::fs::write(plugin_dir.join(format!("{}.wasm", plugin)), "")?; // Empty placeholder WASM

    output::success(&format!("Successfully installed {} v1.0.0", plugin));
    println!("  Location: {}", plugin_dir.display());

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
