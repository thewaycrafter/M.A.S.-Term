//! Ask AI command - Natural language to shell command
use anyhow::Result;
use clap::Args;
use owo_colors::OwoColorize;
use std::thread;
use std::time::Duration;

/// Ask AI command arguments
#[derive(Args)]
pub struct AskArgs {
    /// Natural language query
    #[arg(required = true)]
    query: Vec<String>,
}

/// Run the ask command
pub async fn run(args: AskArgs) -> Result<()> {
    let query = args.query.join(" ");
    
    // 1. Show thinking animation
    let spinner = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    print!("{} Analyzing query...", "🤖".cyan());
    use std::io::Write;
    std::io::stdout().flush()?;
    
    // Simulate network latency/thinking
    for _ in 0..15 {
        for frame in spinner {
            print!("\r{} Analyzing query... {}", "🤖".cyan(), frame.yellow());
            std::io::stdout().flush()?;
            thread::sleep(Duration::from_millis(50));
        }
    }
    print!("\r\x1b[K"); // Clear line

    // 2. "Smart" Matcher (Mock AI)
    let (command, explanation, risk) = match_query(&query);

    // 3. Render Result
    println!("{}", "╭──────────────────────────────────────────────────╮".cyan());
    println!("│                                                  │");
    
    // Query
    println!("│  {}: {:<38} │", "Query".bold(), truncate(&query, 38));
    
    println!("│                                                  │");
    println!("{}", format!("│  {}: {:<36} │", "Result".green().bold(), "").as_str());
    
    // Command code block
    println!("│  {} {:<43} │", "❯".green(), command.white().bold());
    
    println!("│                                                  │");
    
    // Explanation
    println!("│  {} {:<37} │", "Note:".yellow().bold(), truncate(&explanation, 37));
    
    if risk != "Low" {
        println!("│  {} {:<37} │", "Risk:".red().bold(), risk);
    }
    
    println!("│                                                  │");
    println!("{}", "╰──────────────────────────────────────────────────╯".cyan());
    
    Ok(())
}

fn truncate(s: &str, max_width: usize) -> String {
    if s.len() > max_width {
        format!("{}...", &s[0..max_width-3])
    } else {
        s.to_string()
    }
}

fn match_query(query: &str) -> (String, String, String) {
    let q = query.to_lowercase();
    
    if q.contains("undo") && q.contains("commit") {
        return (
            "git reset --soft HEAD~1".to_string(),
            "Undoes last commit, keeps changes".to_string(),
            "Medium".to_string()
        );
    }
    
    if q.contains("list") && (q.contains("file") || q.contains("ls")) {
        return (
            "ls -la".to_string(),
            "Lists all files including hidden ones".to_string(),
            "Low".to_string()
        );
    }
    
    if q.contains("find") && q.contains("text") {
        return (
            "grep -r \"text\" .".to_string(),
            "Recursively finds text in current dir".to_string(),
            "Low".to_string()
        );
    }
    
    if q.contains("port") && q.contains("kill") {
        return (
            "lsof -ti:3000 | xargs kill".to_string(),
            "Kills process on port 3000".to_string(),
            "High".to_string()
        );
    }
    
    if q.contains("docker") && q.contains("prune") {
        return (
            "docker system prune -a".to_string(),
            "Removes all unused containers/images".to_string(),
            "High".to_string()
        );
    }

    // Default fallback
    (
        format!("echo 'Could not understand: {}'", query),
        "Try 'undo commit' or 'find text'".to_string(),
        "Low".to_string()
    )
}
