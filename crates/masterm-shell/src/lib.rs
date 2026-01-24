//! Shell adapters for MASTerm
//!
//! This crate provides thin integration layers for different shells.

pub mod zsh;
pub mod bash;
pub mod fish;
pub mod powershell;

/// Get initialization script for a shell
pub fn get_init_script(shell: &str) -> &'static str {
    match shell.to_lowercase().as_str() {
        "zsh" => zsh::INIT_SCRIPT,
        "bash" => bash::INIT_SCRIPT,
        "fish" => fish::INIT_SCRIPT,
        "powershell" | "pwsh" => powershell::INIT_SCRIPT,
        _ => "",
    }
}
