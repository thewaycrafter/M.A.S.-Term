//! Production environment guard

use super::SafetyGuard;
use crate::context::EnvironmentType;
use crate::plugin::CommandAction;

/// Production environment safety guard
pub struct ProdGuard {
    /// Current environment type
    environment: EnvironmentType,

    /// Dangerous command patterns
    dangerous_patterns: Vec<String>,

    /// Blocked command patterns
    blocked_patterns: Vec<String>,

    /// Whether guard is enabled
    enabled: bool,
}

impl ProdGuard {
    /// Create a new production guard
    pub fn new(environment: EnvironmentType) -> Self {
        Self {
            environment,
            dangerous_patterns: vec![
                "rm -rf".to_string(),
                "rm -r ".to_string(),
                "DROP DATABASE".to_string(),
                "DROP TABLE".to_string(),
                "TRUNCATE TABLE".to_string(),
                "kubectl delete".to_string(),
                "kubectl apply".to_string(),
                "terraform destroy".to_string(),
                "terraform apply".to_string(),
                "docker rm".to_string(),
                "docker rmi".to_string(),
                "git push --force".to_string(),
                "git push -f".to_string(),
            ],
            blocked_patterns: vec![
                "rm -rf /".to_string(),
                "rm -rf /*".to_string(),
                ":(){ :|:& };:".to_string(), // Fork bomb
            ],
            enabled: true,
        }
    }

    /// Create with custom patterns
    pub fn with_patterns(
        mut self,
        dangerous: Vec<String>,
        blocked: Vec<String>,
    ) -> Self {
        self.dangerous_patterns.extend(dangerous);
        self.blocked_patterns.extend(blocked);
        self
    }

    /// Enable or disable the guard
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if command matches any patterns
    fn matches_any(&self, command: &str, patterns: &[String]) -> bool {
        let cmd_lower = command.to_lowercase();
        patterns.iter().any(|p| cmd_lower.contains(&p.to_lowercase()))
    }

    /// Format confirmation message
    fn confirmation_message(&self, command: &str) -> String {
        format!(
            "\x1b[1;33m⚠️  PRODUCTION WARNING\x1b[0m\n\n\
             You are in a \x1b[1;31m{}\x1b[0m environment.\n\n\
             You are about to run a potentially dangerous command:\n\n\
             \x1b[1m  {}\x1b[0m\n\n\
             Type '\x1b[1;32myes\x1b[0m' to confirm or '\x1b[1;31mno\x1b[0m' to cancel:",
            self.environment.full_name(),
            command
        )
    }

    /// Format blocked message
    fn blocked_message(&self, command: &str) -> String {
        format!(
            "\x1b[1;31m🚫 COMMAND BLOCKED\x1b[0m\n\n\
             The following command is not allowed in {} environments:\n\n\
             \x1b[1m  {}\x1b[0m\n\n\
             This restriction is enforced by TermX safety guards.",
            self.environment.full_name(),
            command
        )
    }
}

impl SafetyGuard for ProdGuard {
    fn check(&self, command: &str) -> CommandAction {
        if !self.enabled {
            return CommandAction::Allow;
        }

        // Only apply guards in sensitive environments
        if !self.environment.is_sensitive() {
            return CommandAction::Allow;
        }

        // Check blocked patterns first
        if self.matches_any(command, &self.blocked_patterns) {
            return CommandAction::Block(self.blocked_message(command));
        }

        // Check dangerous patterns
        if self.matches_any(command, &self.dangerous_patterns) {
            // In production: require confirmation
            if self.environment.is_production() {
                return CommandAction::Confirm(self.confirmation_message(command));
            }
            // In staging: just warn
            return CommandAction::Warn(
                "\x1b[33m⚠️  Warning: This command may be dangerous in staging.\x1b[0m".to_string()
            );
        }

        CommandAction::Allow
    }

    fn name(&self) -> &'static str {
        "prod_guard"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prod_guard_blocks_in_prod() {
        let guard = ProdGuard::new(EnvironmentType::Production);

        match guard.check("rm -rf /var/logs") {
            CommandAction::Confirm(_) => {}
            other => panic!("Expected Confirm, got {:?}", other),
        }
    }

    #[test]
    fn test_prod_guard_allows_in_dev() {
        let guard = ProdGuard::new(EnvironmentType::Development);

        assert_eq!(guard.check("rm -rf /tmp/test"), CommandAction::Allow);
    }

    #[test]
    fn test_prod_guard_blocks_fork_bomb() {
        let guard = ProdGuard::new(EnvironmentType::Production);

        match guard.check(":(){ :|:& };:") {
            CommandAction::Block(_) => {}
            other => panic!("Expected Block, got {:?}", other),
        }
    }
}
