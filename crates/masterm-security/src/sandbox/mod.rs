//! Sandbox execution environment
//!
//! Provides a restricted execution environment for commands.

use std::process::Command;

/// Sandbox level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxLevel {
    /// No network, no file writes (except tmp)
    Strict,
    /// Network allowed, limited file writes
    Network,
    /// Full read access, limited write
    Standard,
}

/// Sandbox runner
pub struct Sandbox;

impl Sandbox {
    /// create a sandboxed command based on OS availability
    pub fn create_command(level: SandboxLevel, cmd: &str, args: &[String]) -> Command {
        // Platform specific sandbox
        #[cfg(target_os = "macos")]
        {
             Self::macos_sandbox(level, cmd, args)
        }

        #[cfg(target_os = "linux")]
        {
            // Use bubblewrap if available, else standard command
            // For now just return standard command with warning or wrapped
            let mut command = Command::new(cmd);
            command.args(args);
            command
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        {
            let mut command = Command::new(cmd);
            command.args(args);
            command
        }
    }

    #[cfg(target_os = "macos")]
    fn macos_sandbox(level: SandboxLevel, cmd: &str, args: &[String]) -> Command {
        // macOS `sandbox-exec` is deprecated but still works, or use `sandbox-init`
        // Here we build a profile string based on level

        let profile = match level {
            SandboxLevel::Strict => "(version 1) (deny default) (allow process-exec) (allow file-read*) (allow file-write* (regex #\"^/private/tmp/.*\"))",
            SandboxLevel::Network => "(version 1) (deny default) (allow process-exec) (allow network*) (allow file-read*)",
            SandboxLevel::Standard => "(version 1) (deny default) (allow process-exec) (allow network*) (allow file-read*) (allow file-write* (regex #\"^/Users/.*\"))",
        };

        let mut command = Command::new("sandbox-exec");
        command.arg("-p").arg(profile).arg(cmd).args(args);
        command
    }
}
