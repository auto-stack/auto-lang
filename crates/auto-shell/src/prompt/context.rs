//! Prompt rendering context
//!
//! Git info is discovered by reading .git files directly (branch name = microseconds).
//! A filesystem watcher monitors `.git/` for changes — only when the index, HEAD,
//! or refs change does the cache refresh. No polling, no timers, no disk I/O
//! during typing.
//!
//! Lifecycle:
//! - **cd into git repo** → sync refresh + start watching `.git/`
//! - **`.git/` changes** (index, HEAD, refs) → async refresh in background thread
//! - **command execution** → manual async refresh (fallback for edge cases)
//! - **user typing** → read cache only (zero I/O)

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use notify::EventKind;
use super::config::AshConfig;

// ── Data types ──────────────────────────────────────────────────────

/// Git repository information
#[derive(Debug, Clone, Default)]
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

// ── Global git cache ────────────────────────────────────────────────

/// Shared git info cache with filesystem watcher.
static GIT_CACHE: std::sync::LazyLock<GitCache> =
    std::sync::LazyLock::new(GitCache::new);

/// Git info cache with optional filesystem watcher.
struct GitCache {
    /// Current cached data: (cwd, git_info)
    data: Arc<Mutex<Option<(PathBuf, GitInfo)>>>,
    /// Active filesystem watcher (None = not watching or not in a git repo)
    watcher: Arc<Mutex<Option<notify::RecommendedWatcher>>>,
}

impl GitCache {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(None)),
            watcher: Arc::new(Mutex::new(None)),
        }
    }

    /// Get cached git info for a directory. Returns None if not yet cached
    /// or if the directory doesn't match.
    fn get(&self, cwd: &Path) -> Option<GitInfo> {
        let guard = self.data.lock().unwrap();
        match guard.as_ref() {
            Some((cached_cwd, info)) if cached_cwd == cwd => Some(info.clone()),
            _ => None,
        }
    }

    /// Update the cache with new data.
    fn set(&self, cwd: PathBuf, info: Option<GitInfo>) {
        let mut guard = self.data.lock().unwrap();
        *guard = info.map(|i| (cwd, i));
    }

    /// Start watching `.git/` directory for changes.
    /// When a change is detected, triggers an async cache refresh.
    fn start_watch(&self, cwd: PathBuf) {
        // Find .git directory
        let git_path = match find_git_dir(&cwd) {
            Some(p) => p,
            None => return,
        };

        let git_dir = resolve_git_dir(&git_path);
        let watch_target = if git_dir.join("index").exists() {
            // Watch the .git directory itself for index/HEAD/refs changes
            git_dir.clone()
        } else {
            git_dir.clone()
        };

        let data = self.data.clone();
        let cwd_for_refresh = cwd.clone();

        let result = notify::Config::default()
            .with_poll_interval(std::time::Duration::from_secs(2))
            .with_compare_contents(false);

        let mut watcher = match notify::RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    // Only react to content-changing events (ignore metadata-only)
                    match event.kind {
                        EventKind::Create(_)
                        | EventKind::Modify(notify::event::ModifyKind::Data(_))
                        | EventKind::Modify(notify::event::ModifyKind::Any)
                        | EventKind::Modify(notify::event::ModifyKind::Other) => {}
                        _ => return,
                    }

                    // Async refresh on .git change
                    let data = data.clone();
                    let cwd = cwd_for_refresh.clone();
                    std::thread::spawn(move || {
                        let info = discover_git_info_sync(&cwd);
                        if let Some(info) = info {
                            let mut guard = data.lock().unwrap();
                            *guard = Some((cwd, info));
                        }
                    });
                }
            },
            result,
        ) {
            Ok(w) => w,
            Err(_) => return,
        };

        // Watch the .git directory
        use notify::Watcher;
        if watcher.watch(&watch_target, notify::RecursiveMode::Recursive).is_err() {
            return;
        }

        // Store watcher (drops old one if exists)
        *self.watcher.lock().unwrap() = Some(watcher);
    }

    /// Stop the filesystem watcher.
    fn stop_watch(&self) {
        *self.watcher.lock().unwrap() = None;
    }
}

// ── Public API ──────────────────────────────────────────────────────

/// Called when cd-ing into a new directory.
/// - If inside a git repo: sync refresh + start watching `.git/`
/// - If not in a git repo: clear cache + stop watcher
pub fn on_directory_changed(cwd: PathBuf) {
    if find_git_dir(&cwd).is_some() {
        // Sync refresh so prompt is immediately correct
        let info = discover_git_info_sync(&cwd);
        GIT_CACHE.set(cwd.clone(), info);
        // Start watching .git for future changes
        GIT_CACHE.start_watch(cwd);
    } else {
        // Not a git repo — clear cache and stop watching
        GIT_CACHE.stop_watch();
        GIT_CACHE.data.lock().unwrap().take();
    }
}

/// Trigger a background git info refresh. Called after command execution
/// as a fallback (most changes are caught by the filesystem watcher).
pub fn refresh_git_info_async(cwd: PathBuf) {
    let data = GIT_CACHE.data.clone();
    std::thread::spawn(move || {
        let info = discover_git_info_sync(&cwd);
        if let Some(info) = info {
            let mut guard = data.lock().unwrap();
            *guard = Some((cwd, info));
        }
    });
}

/// Prompt rendering context, shared by all modules.
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
}

impl AshContext {
    /// Build context from current environment (no I/O — reads global cache only)
    pub fn from_current() -> Self {
        Self {
            cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            home: dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")),
            cmd_duration_ms: None,
            last_status: None,
            config: AshConfig::default(),
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
        }
    }

    /// Get git info from the global cache. Never blocks, never does I/O.
    /// Returns None if not yet cached for this directory.
    pub fn git_info(&self) -> Option<GitInfo> {
        GIT_CACHE.get(&self.cwd)
    }
}

// ── Fast git discovery ──────────────────────────────────────────────

/// Synchronous git discovery — called from background thread or on cd.
fn discover_git_info_sync(cwd: &Path) -> Option<GitInfo> {
    let git_path = find_git_dir(cwd)?;

    // Resolve .git file (worktrees have `gitdir: ...` pointer)
    let git_dir = resolve_git_dir(&git_path);

    // Read branch from HEAD (microseconds — just a file read)
    let head_content = read_file(&git_dir.join("HEAD"))?;
    let head_trimmed = head_content.trim();
    let branch = head_trimmed
        .strip_prefix("ref: refs/heads/")
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // Detached HEAD — use short commit hash
            head_trimmed.chars().take(8).collect()
        });

    if branch.is_empty() {
        return None;
    }

    let root = git_dir.parent()?.to_path_buf();

    // Get status (this is the only subprocess call)
    let status = read_porcelain_status(cwd).unwrap_or_default();
    let (ahead, behind) = read_ahead_behind(&git_dir, cwd);

    Some(GitInfo {
        branch,
        status: GitStatus { ahead, behind, ..status },
        root,
    })
}

/// Find the .git directory by walking up from `cwd`.
fn find_git_dir(cwd: &Path) -> Option<PathBuf> {
    let mut dir = cwd;
    loop {
        let git = dir.join(".git");
        if git.exists() {
            return Some(git);
        }
        dir = dir.parent()?;
    }
}

/// Resolve a `.git` path to the actual git directory.
/// Handles worktree `.git` files that contain `gitdir: ...` pointers.
fn resolve_git_dir(git_path: &Path) -> PathBuf {
    if git_path.is_file() {
        if let Some(content) = read_file(git_path) {
            if let Some(gitdir) = content.trim().strip_prefix("gitdir: ") {
                let resolved = git_path.parent()
                    .map(|p| p.join(gitdir))
                    .unwrap_or_else(|| gitdir.into());
                if resolved.exists() {
                    return resolved;
                }
            }
        }
    }
    git_path.to_path_buf()
}

/// Read a file to string, returning None on any error.
fn read_file(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

/// Read ahead/behind counts by comparing HEAD and upstream refs.
fn read_ahead_behind(git_dir: &Path, cwd: &Path) -> (usize, usize) {
    let head_content = read_file(&git_dir.join("HEAD")).unwrap_or_default();
    let head_ref = head_content.trim().strip_prefix("ref: ").unwrap_or("").to_string();
    let head_hash = if head_ref.starts_with("refs/") {
        read_file(&git_dir.join(&head_ref)).unwrap_or_default()
    } else {
        head_content
    };
    let head_hash = head_hash.trim().to_string();

    if head_hash.is_empty() {
        return (0, 0);
    }

    let branch = head_ref.strip_prefix("refs/heads/").unwrap_or("");
    if branch.is_empty() {
        return (0, 0);
    }

    let upstream_path = git_dir.join(format!("refs/remotes/origin/{}", branch));
    let upstream_hash = match read_file(&upstream_path) {
        Some(s) => s.trim().to_string(),
        None => return (0, 0),
    };

    if upstream_hash.is_empty() || head_hash == upstream_hash {
        return (0, 0);
    }

    // Hashes differ — need git to count. This only runs when actually diverged.
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-list", "--left-right", "--count", "HEAD...@{upstream}"])
        .current_dir(cwd)
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            if let Some((a, b)) = s.trim().split_once('\t') {
                return (
                    a.trim().parse().unwrap_or(0),
                    b.trim().parse().unwrap_or(0),
                );
            }
        }
    }

    (0, 0)
}

/// Run `git status --porcelain` and parse the output.
fn read_porcelain_status(cwd: &Path) -> Option<GitStatus> {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(cwd)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let mut status = GitStatus::default();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if line.len() < 2 {
            continue;
        }
        let x = line.as_bytes()[0];
        let y = line.as_bytes()[1];

        if x != b' ' && x != b'?' {
            status.staged += 1;
        }
        if y != b' ' && y != b'?' {
            status.unstaged += 1;
        }
        if x == b'?' && y == b'?' {
            status.untracked += 1;
        }
        if x == b'U' || y == b'U' || (x == b'A' && y == b'A') || (x == b'D' && y == b'D') {
            status.conflicted += 1;
        }
    }

    Some(status)
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
