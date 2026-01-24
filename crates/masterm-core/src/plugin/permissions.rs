//! Plugin permission system

use serde::{Deserialize, Serialize};

/// Permission levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    /// No access
    #[default]
    None,
    /// Read-only access
    Read,
    /// Read and write access
    Write,
}



impl Permission {
    /// Check if this permission allows reading
    pub fn can_read(&self) -> bool {
        matches!(self, Self::Read | Self::Write)
    }

    /// Check if this permission allows writing
    pub fn can_write(&self) -> bool {
        matches!(self, Self::Write)
    }

    /// Parse from string
    pub fn parse_from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "read" => Some(Self::Read),
            "write" => Some(Self::Write),
            _ => Some(Self::None), // Assuming Self::None is a valid parsed state for unrecognized strings
        }
    }
}

/// Set of permissions for a plugin
#[derive(Debug, Clone, Default)]
pub struct PermissionSet {
    /// Filesystem permission
    pub filesystem: Permission,

    /// Network permission
    pub network: Permission,

    /// Environment variable permission
    pub environment: Permission,

    /// Allowed executables
    pub executables: Vec<String>,
}

impl PermissionSet {
    /// Create a restricted permission set
    pub fn restricted() -> Self {
        Self::default()
    }

    /// Create a full access permission set
    pub fn full_access() -> Self {
        Self {
            filesystem: Permission::Write,
            network: Permission::Write,
            environment: Permission::Write,
            executables: vec!["*".to_string()],
        }
    }

    /// Check if an executable is allowed
    pub fn can_execute(&self, binary: &str) -> bool {
        self.executables.iter().any(|e| e == "*" || e == binary)
    }

    /// Check if filesystem access is allowed
    pub fn can_access_filesystem(&self, write: bool) -> bool {
        if write {
            self.filesystem.can_write()
        } else {
            self.filesystem.can_read()
        }
    }

    /// Check if network access is allowed
    pub fn can_access_network(&self, write: bool) -> bool {
        if write {
            self.network.can_write()
        } else {
            self.network.can_read()
        }
    }

    /// Check if environment access is allowed
    pub fn can_access_environment(&self, write: bool) -> bool {
        if write {
            self.environment.can_write()
        } else {
            self.environment.can_read()
        }
    }

    /// Create from plugin permissions config
    pub fn from_config(config: &super::PluginPermissions) -> Self {
        let filesystem = config.filesystem
            .first()
            .and_then(|s| Permission::parse_from_str(s))
            .unwrap_or_default();

        let network = Permission::parse_from_str(&config.network).unwrap_or_default();

        let environment = config.environment
            .first()
            .and_then(|s| Permission::parse_from_str(s))
            .unwrap_or_default();

        Self {
            filesystem,
            network,
            environment,
            executables: config.execute.clone(),
        }
    }

    /// Display permission summary
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        match self.filesystem {
            Permission::Write => parts.push("fs:rw"),
            Permission::Read => parts.push("fs:r"),
            Permission::None => {}
        }

        match self.network {
            Permission::Write => parts.push("net:rw"),
            Permission::Read => parts.push("net:r"),
            Permission::None => {}
        }

        if !self.executables.is_empty() {
            let exec_str = if self.executables.contains(&"*".to_string()) {
                "exec:*".to_string()
            } else {
                format!("exec:[{}]", self.executables.join(","))
            };
            parts.push(Box::leak(exec_str.into_boxed_str()));
        }

        if parts.is_empty() {
            "none".to_string()
        } else {
            parts.join(" ")
        }
    }
}
