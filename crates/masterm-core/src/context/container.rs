//! Container and orchestration context detection

use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Container environment context
#[derive(Debug, Clone)]
pub struct ContainerContext {
    /// Docker context/project
    pub docker: Option<DockerContext>,

    /// Kubernetes context
    pub kubernetes: Option<KubernetesContext>,
}

/// Docker context information
#[derive(Debug, Clone)]
pub struct DockerContext {
    /// Docker is available
    pub available: bool,

    /// Current Docker context
    pub context: Option<String>,

    /// Has Dockerfile in directory
    pub has_dockerfile: bool,

    /// Has docker-compose.yml
    pub has_compose: bool,
}

/// Kubernetes context information
#[derive(Debug, Clone)]
pub struct KubernetesContext {
    /// Current context name
    pub context: String,

    /// Current namespace
    pub namespace: String,
}

impl ContainerContext {
    /// Detect container context
    pub async fn detect(cwd: &Path) -> Result<Option<Self>> {
        let docker = DockerContext::detect(cwd).await;
        let kubernetes = KubernetesContext::detect().await;

        if docker.is_none() && kubernetes.is_none() {
            return Ok(None);
        }

        Ok(Some(Self { docker, kubernetes }))
    }
}

impl DockerContext {
    /// Detect Docker context
    pub async fn detect(cwd: &Path) -> Option<Self> {
        // Check if docker is available
        let available = Command::new("docker")
            .args(["--version"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !available {
            return None;
        }

        // Check for Dockerfile
        let has_dockerfile = cwd.join("Dockerfile").exists()
            || cwd.join("dockerfile").exists()
            || cwd.join("Containerfile").exists();

        // Check for docker-compose
        let has_compose = cwd.join("docker-compose.yml").exists()
            || cwd.join("docker-compose.yaml").exists()
            || cwd.join("compose.yml").exists()
            || cwd.join("compose.yaml").exists();

        // Get current context
        let context = Command::new("docker")
            .args(["context", "show"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

        // Only return if we have something interesting
        if has_dockerfile || has_compose {
            Some(Self {
                available,
                context,
                has_dockerfile,
                has_compose,
            })
        } else {
            None
        }
    }
}

impl KubernetesContext {
    /// Detect Kubernetes context
    pub async fn detect() -> Option<Self> {
        // Get current context
        let context = Command::new("kubectl")
            .args(["config", "current-context"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())?;

        // Get current namespace
        let namespace = Command::new("kubectl")
            .args(["config", "view", "--minify", "-o", "jsonpath={..namespace}"])
            .output()
            .ok()
            .map(|o| {
                let ns = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if ns.is_empty() {
                    "default".to_string()
                } else {
                    ns
                }
            })
            .unwrap_or_else(|| "default".to_string());

        Some(Self { context, namespace })
    }

    /// Check if context looks like production
    pub fn is_production(&self) -> bool {
        let ctx = self.context.to_lowercase();
        let ns = self.namespace.to_lowercase();

        ctx.contains("prod") || ctx.contains("prd") || ns.contains("prod") || ns.contains("prd")
    }
}
