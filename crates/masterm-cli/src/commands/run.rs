//! Run command - Execute workflows from workflows.toml
use anyhow::{anyhow, Result};
use clap::Args;
use owo_colors::OwoColorize;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// Run command arguments
#[derive(Args)]
pub struct RunArgs {
    /// Name of the workflow to run
    #[arg(required = true)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct WorkflowConfig {
    #[serde(flatten)]
    workflows: HashMap<String, Workflow>,
}

#[derive(Debug, Deserialize)]
struct Workflow {
    description: Option<String>,
    steps: Vec<String>,
}

/// Run the run command
pub async fn run(args: RunArgs) -> Result<()> {
    // 1. Find workflows.toml
    let config_path = Path::new("workflows.toml");
    if !config_path.exists() {
        return Err(anyhow!("No 'workflows.toml' found in current directory."));
    }

    // 2. Parse config
    let content = std::fs::read_to_string(config_path)?;
    let config: WorkflowConfig = toml::from_str(&content)
        .map_err(|e| anyhow!("Failed to parse workflows.toml: {}", e))?;

    // 3. Find workflow
    let workflow = config
        .workflows
        .get(&args.name)
        .ok_or_else(|| anyhow!("Workflow '{}' not found in workflows.toml", args.name))?;

    // 4. Execute
    println!(
        "\n{} Running workflow: {}\n",
        "🚀".cyan(),
        args.name.bold()
    );

    if let Some(desc) = &workflow.description {
        println!("   {}\n", desc.dimmed());
    }

    for (i, step) in workflow.steps.iter().enumerate() {
        println!(
            "{} Step {}/{}: {}",
            "➜".yellow(),
            i + 1,
            workflow.steps.len(),
            step.bold()
        );

        let parts: Vec<&str> = step.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let mut cmd = Command::new(parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }

        let status = cmd.status()?;

        if !status.success() {
            println!("\n{} Step failed with exit code: {}", "✖".red(), status);
            return Err(anyhow!("Workflow failed"));
        }
    }

    println!("\n{} Workflow completed successfully!", "✓".green());

    Ok(())
}
