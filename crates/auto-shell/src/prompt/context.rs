//! Prompt rendering context
//!
//! Built once per prompt render, shared across all modules.
/// Heavy computations (git info) use `OnceLock` for lazy evaluation.

use std::path::PathBuf;
use std::sync::OnceLock;

use super::config::AshConfig;

/// Git repository information (lazily discovered)
#[derive(Debug, Clone)]
pub struct GitInfo {
    /// Current branch name
    pub branch: String,
    /// Working tree status counts
    pub status: GitStatus,
    /// Repository root path
    pub root: PathBuf,
}

/// Git working tree status summary
#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    pub staged: usize,
    pub unstaged: usize,
    pub untracked: usize,
    pub conflicted: usize,
    pub ahead: usize,
    pub behind: usize,
}

/// Prompt rendering context, shared by all modules.
///
/// Constructed once per prompt render. Expensive lookups (git) are
/// lazily computed via `OnceLock` — if no module asks for git info,
/// the discovery never runs.
pub struct AshContext {
    /// Current working directory
    pub cwd: PathBuf,
    /// User HOME directory
    pub home: PathBuf,
    /// Last command duration in milliseconds (None = first prompt)
    pub cmd_duration_ms: Option<u64>,
    /// Last command exit code (None = first prompt)
    pub last_status: Option<i32>,
    /// Prompt configuration
    pub config: AshConfig,

    // Lazy caches (shared across modules in one render pass)
    git_info: OnceLock<Option<GitInfo>>,
}

impl AshContext {
    /// Build context from current environment
    pub fn from_current() -> Self {
        Self {
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            home: dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")),
            cmd_duration_ms: None,
            last_status: None,
            config: AshConfig::default(),
            git_info: OnceLock::new(),
        }
    }

    /// Build context with explicit shell state
    pub fn new(
        cwd: PathBuf,
        home: PathBuf,
        cmd_duration_ms: Option<u64>,
        last_status: Option<i32>,
        config: AshConfig,
    ) -> Self {
        Self {
            cwd,
            home,
            cmd_duration_ms,
            last_status,
            config,
            git_info: OnceLock::new(),
        }
    }

    /// Get git info (lazy — computed at most once per context)
    pub fn git_info(&self) -> Option<&GitInfo> {
        self.git_info
            .get_or_init(|| discover_git_info(&self.cwd))
            .as_ref()
    }
}

/// Discover git repository information by running `git` commands.
///
/// Uses `std::process::Command` with a 200ms timeout to avoid blocking.
/// Returns `None` if not in a git repo or git is unavailable.
fn discover_git_info(cwd: &std::path::Path) -> Option<GitInfo> {
    use std::process::Command;
    use std::time::Duration;

    let timeout = Duration::from_millis(200);

    // Get branch name
    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()
        .filter(|o| o.status.success())?;

    let branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();
    if branch.is_empty() {
        return None;
    }

    // Get repo root
    let root_output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(cwd)
        .output()
        .ok()
        .filter(|o| o.status.success());

    let root = root_output
        .map(|o| PathBuf::from(String::from_utf8_lossy(&o.stdout).trim().to_string()))
        .unwrap_or_else(|| cwd.to_path_buf());

    // Get status (porcelain format)
    let status = get_git_status(cwd, timeout);

    Some(GitInfo {
        branch,
        status,
        root,
    })
}

/// Parse `git status --porcelain` output into status counts
fn get_git_status(cwd: &std::path::Path, _timeout: std::time::Duration) -> GitStatus {
    use std::process::Command;

    let output = match Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(cwd)
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return GitStatus::default(),
    };

    let mut status = GitStatus::default();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if line.len() < 2 {
            continue;
        }
        let x = line.as_bytes()[0];
        let y = line.as_bytes()[1];

        // Staged changes (index status)
        if x != b' ' && x != b'?' {
            status.staged += 1;
        }
        // Unstaged changes (worktree status)
        if y != b' ' && y != b'?' {
            status.unstaged += 1;
        }
        // Untracked files
        if x == b'?' && y == b'?' {
            status.untracked += 1;
        }
        // Conflicts
        if x == b'U' || y == b'U' || (x == b'A' && y == b'A') || (x == b'D' && y == b'D') {
            status.conflicted += 1;
        }
    }

    // Ahead/behind (skip if no upstream)
    if let Ok(output) = Command::new("git")
        .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
        .current_dir(cwd)
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            if let Some((ahead, behind)) = s.trim().split_once('\t') {
                status.ahead = ahead.trim().parse().unwrap_or(0);
                status.behind = behind.trim().parse().unwrap_or(0);
            }
        }
    }

    status
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_from_current() {
        let ctx = AshContext::from_current();
        assert!(ctx.cwd.exists());
        assert!(ctx.home.exists());
        assert!(ctx.cmd_duration_ms.is_none());
        assert!(ctx.last_status.is_none());
    }

    #[test]
    fn test_context_with_explicit_state() {
        let ctx = AshContext::new(
            PathBuf::from("/tmp"),
            PathBuf::from("/home/user"),
            Some(1500),
            Some(0),
            AshConfig::default(),
        );
        assert_eq!(ctx.cwd, PathBuf::from("/tmp"));
        assert_eq!(ctx.cmd_duration_ms, Some(1500));
        assert_eq!(ctx.last_status, Some(0));
    }
}
