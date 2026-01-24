//! Safety guards for production environments

mod command_filter;
mod prod_guard;

pub use command_filter::CommandFilter;
pub use prod_guard::ProdGuard;

use crate::plugin::CommandAction;

/// Safety guard trait
pub trait SafetyGuard {
    /// Check a command and return appropriate action
    fn check(&self, command: &str) -> CommandAction;

    /// Get guard name
    fn name(&self) -> &'static str;
}
