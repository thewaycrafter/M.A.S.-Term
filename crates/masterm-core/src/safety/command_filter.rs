//! Command filter for pattern-based command checking

use super::SafetyGuard;
use crate::plugin::CommandAction;
use regex::Regex;

/// Pattern-based command filter
pub struct CommandFilter {
    /// Filter rules
    rules: Vec<FilterRule>,
}

/// A single filter rule
struct FilterRule {
    /// Pattern to match
    pattern: Regex,

    /// Action when matched
    action: CommandAction,

    /// Rule description
    description: String,
}

impl CommandFilter {
    /// Create a new command filter
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Add a confirm rule
    pub fn add_confirm(&mut self, pattern: &str, message: &str) {
        if let Ok(re) = Regex::new(pattern) {
            self.rules.push(FilterRule {
                pattern: re,
                action: CommandAction::Confirm(message.to_string()),
                description: format!("Confirm: {}", pattern),
            });
        }
    }

    /// Add a block rule
    pub fn add_block(&mut self, pattern: &str, message: &str) {
        if let Ok(re) = Regex::new(pattern) {
            self.rules.push(FilterRule {
                pattern: re,
                action: CommandAction::Block(message.to_string()),
                description: format!("Block: {}", pattern),
            });
        }
    }

    /// Add a warn rule
    pub fn add_warn(&mut self, pattern: &str, message: &str) {
        if let Ok(re) = Regex::new(pattern) {
            self.rules.push(FilterRule {
                pattern: re,
                action: CommandAction::Warn(message.to_string()),
                description: format!("Warn: {}", pattern),
            });
        }
    }

    /// Create filter from config patterns
    pub fn from_config(
        dangerous: &[String],
        blocked: &[String],
    ) -> Self {
        let mut filter = Self::new();

        for pattern in blocked {
            let escaped = regex::escape(pattern);
            filter.add_block(
                &escaped,
                &format!("Command containing '{}' is blocked", pattern),
            );
        }

        for pattern in dangerous {
            let escaped = regex::escape(pattern);
            filter.add_confirm(
                &escaped,
                &format!("Command containing '{}' requires confirmation", pattern),
            );
        }

        filter
    }
}

impl SafetyGuard for CommandFilter {
    fn check(&self, command: &str) -> CommandAction {
        for rule in &self.rules {
            if rule.pattern.is_match(command) {
                return rule.action.clone();
            }
        }
        CommandAction::Allow
    }

    fn name(&self) -> &'static str {
        "command_filter"
    }
}

impl Default for CommandFilter {
    fn default() -> Self {
        Self::new()
    }
}
