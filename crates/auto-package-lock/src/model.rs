//! Lock model and the single-sourced ordering/lookup rules.
//!
//! Extracted verbatim from `auto-man`'s `lock.rs` (PLAN-795); field sets,
//! serde shapes and rule behavior are frozen historical contracts — see the
//! crate docs for the honest limits.

use serde::{Deserialize, Serialize};

/// Lock file entry for a single dependency
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockEntry {
    /// Name of the dependency
    pub name: String,
    /// Version that was resolved/used
    pub version: String,
    /// Git repository URL
    pub url: String,
    /// Git commit SHA for reproducibility
    pub commit: String,
    /// Where the dependency is located locally
    pub path: String,
}

/// The complete lock file structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockFile {
    /// AutoMan version that created this lock file
    pub automan_version: String,
    /// List of all locked dependencies
    pub dependencies: Vec<LockEntry>,
}

impl LockFile {
    /// Create a new empty lock file.
    ///
    /// The tool version is caller-supplied: `auto-man` passes its own
    /// `AUTOMAN_VERSION`; this crate never substitutes its package version.
    pub fn new(automan_version: &str) -> Self {
        Self {
            automan_version: automan_version.to_string(),
            dependencies: Vec::new(),
        }
    }

    /// Create a lock file from already-projected dependency entries.
    ///
    /// Callers project their own dependency representation (Target, package
    /// descriptor, ...) into [`LockEntry`] and hand the complete list here;
    /// the canonical name ordering is applied by the single sort rule.
    pub fn from_dependencies(automan_version: &str, mut dependencies: Vec<LockEntry>) -> Self {
        sort_dependencies(&mut dependencies);
        Self {
            automan_version: automan_version.to_string(),
            dependencies,
        }
    }

    /// Get a lock entry by name (first match — see [`find_entry`]).
    pub fn get(&self, name: &str) -> Option<&LockEntry> {
        find_entry(&self.dependencies, name)
    }
}

/// Sort entries by name for consistent ordering (single ordering rule,
/// verbatim from the historical `from_targets`).
pub fn sort_dependencies(dependencies: &mut [LockEntry]) {
    dependencies.sort_by(|a, b| a.name.cmp(&b.name));
}

/// Read-only view of a locked entry as consumed by the shared
/// lookup/verification rules.
///
/// `auto-man`'s compatibility facade implements this over its legacy
/// [`LockEntry`]-shaped type so the rules stay single-sourced without a DTO
/// round-trip; implementations must expose exactly the corresponding fields.
pub trait LockedEntryView {
    fn lock_name(&self) -> &str;
    fn lock_version(&self) -> &str;
    fn lock_commit(&self) -> &str;
    fn lock_path(&self) -> &str;
}

impl LockedEntryView for LockEntry {
    fn lock_name(&self) -> &str {
        &self.name
    }
    fn lock_version(&self) -> &str {
        &self.version
    }
    fn lock_commit(&self) -> &str {
        &self.commit
    }
    fn lock_path(&self) -> &str {
        &self.path
    }
}

/// First-match lookup by name (single lookup rule).
///
/// Historical behavior on duplicate names: the first entry wins for both
/// `get` and `verify`; sorting is stable, so duplicates keep insertion
/// order.
pub fn find_entry<'a, L: LockedEntryView>(dependencies: &'a [L], name: &str) -> Option<&'a L> {
    dependencies.iter().find(|d| d.lock_name() == name)
}

/// Target-free projection of a resolved dependency as seen by verification.
///
/// `auto-man` builds these from `Target` (after Dep/Device kind filtering);
/// the core never sees `Target` itself.
#[derive(Debug, Clone)]
pub struct ResolvedDependency {
    pub name: String,
    pub version: String,
}

impl ResolvedDependency {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stamps_caller_supplied_version_not_crate_version() {
        let lock = LockFile::new("9.9.9-tool");
        assert_eq!(lock.automan_version, "9.9.9-tool");
        assert!(lock.dependencies.is_empty());
        // The core must never substitute its own package version.
        assert_ne!(lock.automan_version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn from_dependencies_sorts_by_name_stably() {
        let entries = vec![
            entry("zeta", "2.0.0"),
            entry("alpha", "1.0.0"),
            entry("mid", "0.1.0"),
        ];
        let lock = LockFile::from_dependencies("v", entries);
        let names: Vec<&str> = lock.dependencies.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, ["alpha", "mid", "zeta"]);
    }

    #[test]
    fn from_dependencies_keeps_duplicate_insertion_order_among_equals() {
        let entries = vec![entry("same", "1.0.0"), entry("same", "2.0.0")];
        let lock = LockFile::from_dependencies("v", entries);
        assert_eq!(lock.dependencies.len(), 2);
        assert_eq!(lock.get("same").unwrap().version, "1.0.0");
    }

    #[test]
    fn get_returns_first_match_and_none_for_missing() {
        let lock = LockFile::from_dependencies("v", vec![entry("a", "1"), entry("b", "2")]);
        assert_eq!(lock.get("a").unwrap().version, "1");
        assert_eq!(lock.get("b").unwrap().version, "2");
        assert!(lock.get("c").is_none());
    }

    fn entry(name: &str, version: &str) -> LockEntry {
        LockEntry {
            name: name.to_string(),
            version: version.to_string(),
            url: String::new(),
            commit: String::new(),
            path: String::new(),
        }
    }
}
