//! Built-in plugins for MASTerm
//!
//! This crate provides the default plugins that ship with MASTerm:
//! - Git: Branch, status, ahead/behind
//! - Environment: Dev/staging/prod detection
//! - Production Guard: Safety warnings and confirmations
//! - Language detection: Node, Python, Go, Rust, Java

pub mod git;
pub mod env;
pub mod prod_guard;
pub mod node;
pub mod python;
pub mod go;
pub mod rust;
pub mod java;
pub mod docker;
pub mod kubernetes;

use masterm_core::plugin::Plugin;

/// Get all built-in plugins
pub fn builtin_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(git::GitPlugin::new()),
        Box::new(env::EnvPlugin::new()),
        Box::new(prod_guard::ProdGuardPlugin::new()),
        Box::new(node::NodePlugin::new()),
        Box::new(python::PythonPlugin::new()),
        Box::new(go::GoPlugin::new()),
        Box::new(rust::RustPlugin::new()),
    ]
}
