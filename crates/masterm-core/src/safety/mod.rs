//! Safety guards for production environments

mod prod_guard;
mod command_filter;

pub use prod_guard::ProdGuard;
pub use command_filter::CommandFilter;

use crate::plugin::CommandAction;

/// Safety guard trait
pub trait SafetyGuard {
    /// Check a command and return appropriate action
    fn check(&self, command: &str) -> CommandAction;

    /// Get guard name
    fn name(&self) -> &'static str;
}
