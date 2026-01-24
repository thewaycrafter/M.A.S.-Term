//! Git repository context detection

use anyhow::Result;
use std::path::Path;
use std::process::Command;

/// Git repository context
#[derive(Debug, Clone)]
pub struct GitContext {
    /// Current branch name
    pub branch: String,

    /// Is HEAD detached?
    pub detached: bool,

    /// Number of staged files
    pub staged: u32,

    /// Number of modified files
    pub modified: u32,

    /// Number of untracked files
    pub untracked: u32,

    /// Number of deleted files
    pub deleted: u32,

    /// Commits ahead of upstream
    pub ahead: u32,

    /// Commits behind upstream
    pub behind: u32,

    /// Number of stashes
    pub stash_count: u32,

    /// Is repo clean?
    pub is_clean: bool,

    /// In merge conflict?
    pub conflict: bool,

    /// Repository root path
    pub repo_root: String,
}

impl GitContext {
    /// Detect git context for a directory
    pub async fn detect(cwd: &Path) -> Result<Option<Self>> {
        // Quick check: is this a git repo?
        if !Self::is_git_repo(cwd) {
            return Ok(None);
        }

        // Get branch name
        let branch = Self::get_branch(cwd)?;
        let detached = branch.is_none();
        let branch = branch.unwrap_or_else(|| "HEAD".to_string());

        // Get status
        let (staged, modified, untracked, deleted, conflict) = Self::get_status(cwd)?;

        // Get ahead/behind
        let (ahead, behind) = Self::get_ahead_behind(cwd).unwrap_or((0, 0));

        // Get stash count
        let stash_count = Self::get_stash_count(cwd).unwrap_or(0);

        // Get repo root
        let repo_root = Self::get_repo_root(cwd).unwrap_or_else(|| cwd.display().to_string());

        let is_clean = staged == 0 && modified == 0 && untracked == 0 && deleted == 0;

        Ok(Some(Self {
            branch,
            detached,
            staged,
            modified,
            untracked,
            deleted,
            ahead,
            behind,
            stash_count,
            is_clean,
            conflict,
            repo_root,
        }))
    }

    /// Check if directory is inside a git repo
    fn is_git_repo(cwd: &Path) -> bool {
        Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .current_dir(cwd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Get current branch name
    fn get_branch(cwd: &Path) -> Result<Option<String>> {
        let output = Command::new("git")
            .args(["symbolic-ref", "--short", "HEAD"])
            .current_dir(cwd)
            .output()?;

        if output.status.success() {
            Ok(Some(String::from_utf8_lossy(&output.stdout).trim().to_string()))
        } else {
            // Try to get short SHA for detached HEAD
            let output = Command::new("git")
                .args(["rev-parse", "--short", "HEAD"])
                .current_dir(cwd)
                .output()?;

            if output.status.success() {
                Ok(Some(format!(":{}", String::from_utf8_lossy(&output.stdout).trim())))
            } else {
                Ok(None)
            }
        }
    }

    /// Get status counts
    fn get_status(cwd: &Path) -> Result<(u32, u32, u32, u32, bool)> {
        let output = Command::new("git")
            .args(["status", "--porcelain=v1"])
            .current_dir(cwd)
            .output()?;

        if !output.status.success() {
            return Ok((0, 0, 0, 0, false));
        }

        let mut staged = 0;
        let mut modified = 0;
        let mut untracked = 0;
        let mut deleted = 0;
        let mut conflict = false;

        for line in String::from_utf8_lossy(&output.stdout).lines() {
            if line.len() < 2 {
                continue;
            }

            let index = line.chars().next().unwrap_or(' ');
            let worktree = line.chars().nth(1).unwrap_or(' ');

            // Check for conflicts
            if index == 'U' || worktree == 'U' || (index == 'A' && worktree == 'A') || (index == 'D' && worktree == 'D') {
                conflict = true;
            }

            // Index (staged) changes
            match index {
                'A' | 'M' | 'R' | 'C' => staged += 1,
                'D' => deleted += 1,
                _ => {}
            }

            // Worktree (unstaged) changes
            match worktree {
                'M' => modified += 1,
                'D' => deleted += 1,
                '?' => untracked += 1,
                _ => {}
            }
        }

        Ok((staged, modified, untracked, deleted, conflict))
    }

    /// Get ahead/behind counts
    fn get_ahead_behind(cwd: &Path) -> Result<(u32, u32)> {
        let output = Command::new("git")
            .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
            .current_dir(cwd)
            .output()?;

        if !output.status.success() {
            return Ok((0, 0));
        }

        let counts: Vec<&str> = String::from_utf8_lossy(&output.stdout)
            .trim()
            .split('\t')
            .collect();

        if counts.len() == 2 {
            let ahead = counts[0].parse().unwrap_or(0);
            let behind = counts[1].parse().unwrap_or(0);
            Ok((ahead, behind))
        } else {
            Ok((0, 0))
        }
    }

    /// Get stash count
    fn get_stash_count(cwd: &Path) -> Result<u32> {
        let output = Command::new("git")
            .args(["stash", "list"])
            .current_dir(cwd)
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).lines().count() as u32)
        } else {
            Ok(0)
        }
    }

    /// Get repository root path
    fn get_repo_root(cwd: &Path) -> Option<String> {
        Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .current_dir(cwd)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }

    /// Format status for display
    pub fn format_status(&self) -> String {
        let mut parts = Vec::new();

        if self.staged > 0 {
            parts.push(format!("+{}", self.staged));
        }
        if self.modified > 0 {
            parts.push(format!("~{}", self.modified));
        }
        if self.deleted > 0 {
            parts.push(format!("-{}", self.deleted));
        }
        if self.untracked > 0 {
            parts.push(format!("?{}", self.untracked));
        }

        if parts.is_empty() {
            "✓".to_string()
        } else {
            parts.join(" ")
        }
    }

    /// Format ahead/behind for display
    pub fn format_ahead_behind(&self) -> Option<String> {
        if self.ahead == 0 && self.behind == 0 {
            return None;
        }

        let mut parts = Vec::new();
        if self.ahead > 0 {
            parts.push(format!("↑{}", self.ahead));
        }
        if self.behind > 0 {
            parts.push(format!("↓{}", self.behind));
        }

        Some(parts.join(" "))
    }
}
