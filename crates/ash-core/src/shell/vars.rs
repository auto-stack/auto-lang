//! Shell variables and environment management
//!
//! Plan 301 / Plan 309 Task 1.2 — Phase 1: scope stack + first-class PATH.

use std::collections::HashMap;
use std::path::Path as FsPath;

/// A single PATH entry, enriched for table display.
#[derive(Debug, Clone)]
pub struct AshPathEntry {
    pub index: usize,
    pub path: String,
    pub exists: bool,
    pub duplicate: bool,
}

/// A single environment variable entry, enriched for table display.
#[derive(Debug, Clone)]
pub struct AshEnvEntry {
    pub name: String,
    pub value: String,
    pub exported: bool,
}

/// Shell variable storage
#[derive(Debug, Clone)]
pub struct ShellVars {
    /// Local shell variables (not exported to child processes)
    locals: HashMap<String, String>,
    /// Environment variables (exported to child processes)
    env: HashMap<String, String>,
    /// Scope stack: each `with env()` block pushes one frame.
    /// The value records the **previous** state of the key so it can be
    /// restored on `pop_scope`: `Some(v)` = it existed with value `v`,
    /// `None` = it did not exist.
    scope_stack: Vec<HashMap<String, Option<String>>>,
}

impl ShellVars {
    /// Create a new variable store
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            env: HashMap::new(),
            scope_stack: Vec::new(),
        }
    }

    /// Set a local variable
    pub fn set_local(&mut self, name: String, value: String) {
        self.locals.insert(name, value);
    }

    /// Get a local variable
    pub fn get_local(&self, name: &str) -> Option<&String> {
        self.locals.get(name)
    }

    /// Set an environment variable (export)
    pub fn set_env(&mut self, name: String, value: String) {
        // Update our copy
        self.env.insert(name.clone(), value.clone());
        // Update actual process environment
        std::env::set_var(name, value);
    }

    /// Get an environment variable
    pub fn get_env(&self, name: &str) -> Option<String> {
        // Check our copy first, then fall back to process environment
        self.env.get(name).cloned().or_else(|| std::env::var(name).ok())
    }

    /// Unset a local variable
    pub fn unset_local(&mut self, name: &str) {
        self.locals.remove(name);
    }

    /// Unset an environment variable
    pub fn unset_env(&mut self, name: &str) {
        self.env.remove(name);
        std::env::remove_var(name);
    }

    /// List all local variables
    pub fn list_locals(&self) -> Vec<(String, String)> {
        self.locals.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// List all environment variables
    pub fn list_env(&self) -> Vec<(String, String)> {
        self.env.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    // ── Plan 301 Phase 1: scope management ──────────────────────────────

    /// Enter a new variable scope (`with env()` block / `K=V cmd` prefix).
    pub fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    /// Exit the current scope, restoring every key overridden within it.
    ///
    /// Both the in-memory map **and** the live process environment are
    /// restored, so child processes spawned after the block see the
    /// pre-block state (fixes the design bug in plan 301 §3.2 which only
    /// restored the map).
    pub fn pop_scope(&mut self) {
        if let Some(overrides) = self.scope_stack.pop() {
            for (key, old_val) in overrides {
                match old_val {
                    Some(val) => {
                        self.env.insert(key.clone(), val.clone());
                        std::env::set_var(key, val);
                    }
                    None => {
                        self.env.remove(&key);
                        std::env::remove_var(key);
                    }
                }
            }
        }
    }

    /// Set an env var within the current scope, recording its previous value
    /// for later restoration. If no scope is active, behaves like `set_env`.
    pub fn set_env_scoped(&mut self, name: String, value: String) {
        if let Some(scope) = self.scope_stack.last_mut() {
            // Record the pre-scope value only on the first override of this key.
            if !scope.contains_key(&name) {
                let old = self.env.get(&name).cloned();
                scope.insert(name.clone(), old);
            }
        }
        self.env.insert(name.clone(), value.clone());
        std::env::set_var(name, value);
    }

    /// Whether at least one scope is currently active.
    pub fn in_scope(&self) -> bool {
        !self.scope_stack.is_empty()
    }

    // ── Plan 301 Phase 1: first-class PATH ──────────────────────────────

    /// Platform-correct path-list separator.
    fn path_sep() -> &'static str {
        if cfg!(windows) { ";" } else { ":" }
    }

    /// Get PATH as a list (split on the platform separator, empties dropped).
    pub fn get_path_list(&self) -> Vec<String> {
        let path_str = self.get_env("PATH").unwrap_or_default();
        path_str
            .split(Self::path_sep())
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Write a list back to PATH (joined with the platform separator).
    pub fn set_path_list(&mut self, paths: Vec<String>) {
        let path_str = paths.join(Self::path_sep());
        self.set_env("PATH".to_string(), path_str);
    }

    /// Append `dir` to the end of PATH (lowest priority).
    pub fn path_add(&mut self, dir: &str) {
        let mut paths = self.get_path_list();
        paths.push(dir.to_string());
        self.set_path_list(paths);
    }

    /// Prepend `dir` to the front of PATH (highest priority).
    pub fn path_prepend(&mut self, dir: &str) {
        let mut paths = self.get_path_list();
        paths.insert(0, dir.to_string());
        self.set_path_list(paths);
    }

    /// Remove every PATH entry equal to `dir`.
    pub fn path_remove(&mut self, dir: &str) {
        let mut paths = self.get_path_list();
        paths.retain(|p| p != dir);
        self.set_path_list(paths);
    }

    /// Remove the PATH entry at `index`. Returns an error if out of range.
    pub fn path_remove_index(&mut self, index: usize) -> Result<(), String> {
        let mut paths = self.get_path_list();
        if index >= paths.len() {
            return Err(format!("序号超出范围，当前共 {} 条", paths.len()));
        }
        paths.remove(index);
        self.set_path_list(paths);
        Ok(())
    }

    /// Move the PATH entry at `from` to position `to` (0-based indices).
    pub fn path_move(&mut self, from: usize, to: usize) -> Result<(), String> {
        let mut paths = self.get_path_list();
        if from >= paths.len() {
            return Err(format!("源序号超出范围，当前共 {} 条", paths.len()));
        }
        let to = to.min(paths.len().saturating_sub(1));
        let item = paths.remove(from);
        paths.insert(to, item);
        self.set_path_list(paths);
        Ok(())
    }

    /// Deduplicate PATH entries (case-insensitive comparison).
    pub fn path_dedup(&mut self) {
        let mut seen = std::collections::HashSet::new();
        let paths = self.get_path_list();
        let deduped: Vec<String> = paths
            .into_iter()
            .filter(|p| seen.insert(p.to_lowercase()))
            .collect();
        self.set_path_list(deduped);
    }

    /// Clean PATH: deduplicate and drop directories that don't exist on disk.
    pub fn path_clean(&mut self) {
        let mut seen = std::collections::HashSet::new();
        let paths = self.get_path_list();
        let cleaned: Vec<String> = paths
            .into_iter()
            .filter(|p| {
                let canonical = p.to_lowercase();
                seen.insert(canonical) && FsPath::new(p).exists()
            })
            .collect();
        self.set_path_list(cleaned);
    }

    /// Get PATH entries enriched with `exists` / `duplicate` flags for display.
    pub fn get_path_entries(&self) -> Vec<AshPathEntry> {
        let paths = self.get_path_list();
        let mut seen = std::collections::HashSet::new();
        paths
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let canonical = p.to_lowercase();
                let dup = !seen.insert(canonical);
                AshPathEntry {
                    index: i,
                    path: p.clone(),
                    exists: FsPath::new(p).exists(),
                    duplicate: dup,
                }
            })
            .collect()
    }
}

impl Default for ShellVars {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: run a body with a throwaway PATH, restoring the real one after.
    fn with_test_path<F: FnOnce(&mut ShellVars)>(f: F) {
        let saved = std::env::var("PATH").ok();
        let mut vars = ShellVars::new();
        // Use set_path_list so the separator is platform-correct.
        vars.set_path_list(vec!["/usr/bin".into(), "/bin".into()]);
        f(&mut vars);
        // Restore the process PATH exactly.
        match saved {
            Some(p) => std::env::set_var("PATH", p),
            None => std::env::remove_var("PATH"),
        }
    }

    #[test]
    fn test_set_get_local() {
        let mut vars = ShellVars::new();
        vars.set_local("test".to_string(), "value".to_string());
        assert_eq!(vars.get_local("test"), Some(&"value".to_string()));
    }

    #[test]
    fn test_set_get_env() {
        let mut vars = ShellVars::new();
        vars.set_env("ASH_TEST_VAR_1".to_string(), "test_value".to_string());
        assert_eq!(vars.get_env("ASH_TEST_VAR_1"), Some("test_value".to_string()));
        vars.unset_env("ASH_TEST_VAR_1");
        assert_eq!(vars.get_env("ASH_TEST_VAR_1"), None);
    }

    #[test]
    fn test_unset_local() {
        let mut vars = ShellVars::new();
        vars.set_local("test".to_string(), "value".to_string());
        vars.unset_local("test");
        assert_eq!(vars.get_local("test"), None);
    }

    #[test]
    fn test_list_locals() {
        let mut vars = ShellVars::new();
        vars.set_local("a".to_string(), "1".to_string());
        vars.set_local("b".to_string(), "2".to_string());
        let list = vars.list_locals();
        assert_eq!(list.len(), 2);
    }

    // ── Scope stack ─────────────────────────────────────────────────────

    #[test]
    fn test_scope_push_pop_restores_value() {
        let mut vars = ShellVars::new();
        vars.set_env("ASH_SCOPE".to_string(), "outer".to_string());
        assert_eq!(vars.get_env("ASH_SCOPE"), Some("outer".to_string()));

        vars.push_scope();
        vars.set_env_scoped("ASH_SCOPE".to_string(), "inner".to_string());
        assert_eq!(vars.get_env("ASH_SCOPE"), Some("inner".to_string()));
        // Process env must reflect the scoped value too.
        assert_eq!(std::env::var("ASH_SCOPE").unwrap(), "inner");

        vars.pop_scope();
        assert_eq!(vars.get_env("ASH_SCOPE"), Some("outer".to_string()));
        assert_eq!(std::env::var("ASH_SCOPE").unwrap(), "outer");
        vars.unset_env("ASH_SCOPE");
    }

    #[test]
    fn test_scope_pop_restores_absence() {
        let mut vars = ShellVars::new();
        // Key does not exist before scope.
        vars.push_scope();
        vars.set_env_scoped("ASH_WAS_ABSENT".to_string(), "temp".to_string());
        assert!(vars.get_env("ASH_WAS_ABSENT").is_some());
        vars.pop_scope();
        // After pop, it should be gone again (process env too).
        assert!(vars.get_env("ASH_WAS_ABSENT").is_none());
        assert!(std::env::var("ASH_WAS_ABSENT").is_err());
    }

    #[test]
    fn test_nested_scopes() {
        let mut vars = ShellVars::new();
        vars.set_env("ASH_N".to_string(), "base".to_string());

        vars.push_scope();
        vars.set_env_scoped("ASH_N".to_string(), "L1".to_string());

        vars.push_scope();
        vars.set_env_scoped("ASH_N".to_string(), "L2".to_string());
        assert_eq!(vars.get_env("ASH_N"), Some("L2".to_string()));
        vars.pop_scope();
        assert_eq!(vars.get_env("ASH_N"), Some("L1".to_string()));
        vars.pop_scope();
        assert_eq!(vars.get_env("ASH_N"), Some("base".to_string()));
        assert!(!vars.in_scope());
        vars.unset_env("ASH_N");
    }

    #[test]
    fn test_set_env_scoped_without_scope_is_plain_set() {
        let mut vars = ShellVars::new();
        vars.set_env_scoped("ASH_NOSCOPE".to_string(), "v".to_string());
        assert_eq!(vars.get_env("ASH_NOSCOPE"), Some("v".to_string()));
        // No scope active → pop_scope is a no-op, value stays.
        vars.pop_scope();
        assert_eq!(vars.get_env("ASH_NOSCOPE"), Some("v".to_string()));
        vars.unset_env("ASH_NOSCOPE");
    }

    // ── PATH operations ─────────────────────────────────────────────────

    #[test]
    fn test_path_get_set_list() {
        with_test_path(|vars| {
            let list = vars.get_path_list();
            assert_eq!(list, vec!["/usr/bin".to_string(), "/bin".to_string()]);
            vars.set_path_list(vec!["/a".into(), "/b".into(), "/c".into()]);
            assert_eq!(vars.get_path_list(), vec!["/a", "/b", "/c"]);
        });
    }

    #[test]
    fn test_path_add_prepend_remove() {
        with_test_path(|vars| {
            vars.path_add("/new");
            assert!(vars.get_path_list().last().unwrap() == "/new");
            vars.path_prepend("/front");
            assert!(vars.get_path_list().first().unwrap() == "/front");
            vars.path_remove("/bin");
            assert!(!vars.get_path_list().contains(&"/bin".to_string()));
        });
    }

    #[test]
    fn test_path_remove_index() {
        with_test_path(|vars| {
            assert!(vars.path_remove_index(5).is_err()); // out of range
            assert!(vars.path_remove_index(0).is_ok());
            assert_eq!(vars.get_path_list(), vec!["/bin"]);
        });
    }

    #[test]
    fn test_path_move() {
        with_test_path(|vars| {
            vars.set_path_list(vec!["/a".into(), "/b".into(), "/c".into(), "/d".into()]);
            vars.path_move(3, 0).unwrap();
            assert_eq!(vars.get_path_list(), vec!["/d", "/a", "/b", "/c"]);
            assert!(vars.path_move(99, 0).is_err());
        });
    }

    #[test]
    fn test_path_dedup() {
        with_test_path(|vars| {
            vars.set_path_list(vec![
                "/x".into(),
                "/X".into(), // case-insensitive dup of /x
                "/y".into(),
                "/x".into(), // exact dup
            ]);
            vars.path_dedup();
            assert_eq!(vars.get_path_list(), vec!["/x", "/y"]);
        });
    }

    #[test]
    fn test_path_entries_flags() {
        with_test_path(|vars| {
            vars.set_path_list(vec![
                "/usr/bin".into(), // exists on most unix; flagged if not
                "/usr/bin".into(), // duplicate
            ]);
            let entries = vars.get_path_entries();
            assert_eq!(entries.len(), 2);
            assert!(entries[1].duplicate);
            assert!(!entries[0].duplicate);
        });
    }
}
