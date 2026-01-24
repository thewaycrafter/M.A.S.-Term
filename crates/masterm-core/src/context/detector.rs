//! Main context detector

use super::{ContainerContext, EnvironmentType, GitContext, LanguageContext};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tracing::debug;

/// Aggregated context for the current directory
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// Current working directory
    pub cwd: PathBuf,

    /// Git context (if in a git repo)
    pub git: Option<GitContext>,

    /// Detected languages
    pub languages: Vec<LanguageContext>,

    /// Container context
    pub container: Option<ContainerContext>,

    /// Environment type (dev/staging/prod)
    pub environment: EnvironmentType,

    /// Detection duration
    pub detection_time: Duration,
}

/// Context detector with caching
pub struct ContextDetector {
    /// Cached context per directory
    cache: std::collections::HashMap<PathBuf, (Context, Instant)>,

    /// Cache TTL
    cache_ttl: Duration,

    /// Production patterns for environment detection
    prod_patterns: Vec<String>,
}

impl ContextDetector {
    /// Create a new context detector
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
            cache_ttl: Duration::from_secs(5),
            prod_patterns: vec![
                "**/prod/**".to_string(),
                "**/production/**".to_string(),
            ],
        }
    }

    /// Create detector with custom prod patterns
    pub fn with_prod_patterns(mut self, patterns: Vec<String>) -> Self {
        self.prod_patterns = patterns;
        self
    }

    /// Detect context for the given directory
    pub async fn detect(&mut self, cwd: &Path) -> Result<Context> {
        let start = Instant::now();

        // Check cache
        if let Some((cached, cached_at)) = self.cache.get(cwd) {
            if cached_at.elapsed() < self.cache_ttl {
                debug!("Using cached context for {:?}", cwd);
                return Ok(cached.clone());
            }
        }

        let cwd = cwd.to_path_buf();

        // Detect all contexts in parallel
        let git = self.detect_git(&cwd).await;
        let languages = self.detect_languages(&cwd).await;
        let container = self.detect_container(&cwd).await;
        let environment = self.detect_environment(&cwd);

        let context = Context {
            cwd: cwd.clone(),
            git,
            languages,
            container,
            environment,
            detection_time: start.elapsed(),
        };

        // Cache result
        self.cache.insert(cwd, (context.clone(), Instant::now()));

        debug!("Context detection took {:?}", context.detection_time);
        Ok(context)
    }

    /// Detect git context
    async fn detect_git(&self, cwd: &Path) -> Option<GitContext> {
        GitContext::detect(cwd).await.ok().flatten()
    }

    /// Detect programming languages
    async fn detect_languages(&self, cwd: &Path) -> Vec<LanguageContext> {
        LanguageContext::detect_all(cwd).await
    }

    /// Detect container context
    async fn detect_container(&self, cwd: &Path) -> Option<ContainerContext> {
        ContainerContext::detect(cwd).await.ok().flatten()
    }

    /// Detect environment type
    fn detect_environment(&self, cwd: &Path) -> EnvironmentType {
        EnvironmentType::detect(cwd, &self.prod_patterns)
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Invalidate cache for a specific directory
    pub fn invalidate(&mut self, cwd: &Path) {
        self.cache.remove(cwd);
    }
}

impl Default for ContextDetector {
    fn default() -> Self {
        Self::new()
    }
}
