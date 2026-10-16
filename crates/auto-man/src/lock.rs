//! Dependency lock file management
//!
//! This module handles the `pac.lock` file that records the exact versions
//! of all dependencies used in a build, ensuring reproducible builds.
//!
//! PLAN-795: the lock model, codec and verification rules live in the
//! `auto-package-lock` core crate (Target-free). This module is the
//! production adapter and compatibility facade: it projects `Target`s into
//! owned lock entries, observes Git commits for dependency directories, and
//! delegates every rule (ordering, lookup, verification, TOML/JSON load and
//! save) to the single core implementation. The public `LockFile`/
//! `LockEntry` shapes and method surface are frozen historical API; nothing
//! is re-implemented here.

use crate::target::{Target, TargetKind};
use auto_package_lock as lock_core;
use auto_val::{AutoPath, AutoStr};
use serde::{Deserialize, Serialize};

/// Lock file entry for a single dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
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

// The shared lookup/verification rules in the core operate on this
// read-only view, so the facade entry type plugs into them without a DTO
// round-trip (PLAN-795 single-rule requirement).
impl lock_core::LockedEntryView for LockEntry {
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

impl LockEntry {
    fn to_core(&self) -> lock_core::LockEntry {
        lock_core::LockEntry {
            name: self.name.clone(),
            version: self.version.clone(),
            url: self.url.clone(),
            commit: self.commit.clone(),
            path: self.path.clone(),
        }
    }

    fn from_core(entry: lock_core::LockEntry) -> Self {
        Self {
            name: entry.name,
            version: entry.version,
            url: entry.url,
            commit: entry.commit,
            path: entry.path,
        }
    }
}

/// The complete lock file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    /// AutoMan version that created this lock file
    pub automan_version: String,
    /// List of all locked dependencies
    pub dependencies: Vec<LockEntry>,
}

impl LockFile {
    fn to_core(&self) -> lock_core::LockFile {
        lock_core::LockFile {
            automan_version: self.automan_version.clone(),
            dependencies: self.dependencies.iter().map(|d| d.to_core()).collect(),
        }
    }

    fn from_core(lock: lock_core::LockFile) -> Self {
        Self {
            automan_version: lock.automan_version,
            dependencies: lock
                .dependencies
                .into_iter()
                .map(LockEntry::from_core)
                .collect(),
        }
    }

    /// Create a new empty lock file
    pub fn new() -> Self {
        Self::from_core(lock_core::LockFile::new(crate::AUTOMAN_VERSION))
    }

    /// Generate a lock file from resolved targets
    pub fn from_targets(targets: &[Target]) -> Self {
        let entries: Vec<lock_core::LockEntry> = targets
            .iter()
            .filter(|t| matches!(t.kind, TargetKind::Dep | TargetKind::Device))
            .filter_map(LockEntry::from_target)
            .map(|e| e.to_core())
            .collect();

        // Ordering rule lives in the core (sort by name for consistent
        // ordering); the tool version is auto-man's own, never the core's.
        Self::from_core(lock_core::LockFile::from_dependencies(
            crate::AUTOMAN_VERSION,
            entries,
        ))
    }

    /// Load a lock file from disk
    pub fn load(path: &AutoPath) -> Result<Self, String> {
        lock_core::LockFile::load(path).map(Self::from_core)
    }

    /// Save the lock file to disk
    pub fn save(&self, path: &AutoPath) -> Result<(), String> {
        self.to_core().save(path)
    }

    /// Verify that all dependencies match the lock file
    pub fn verify(&self, targets: &[Target]) -> Result<(), String> {
        // Target→projection stays in this adapter: only Dep/Device targets
        // are verified, projected to {name, version}; the rule itself
        // (first-match, version check, PLAN-635 D6 path existence, error
        // aggregation) runs in the core over the frozen entry view.
        let resolved: Vec<lock_core::ResolvedDependency> = targets
            .iter()
            .filter(|t| matches!(t.kind, TargetKind::Dep | TargetKind::Device))
            .map(|t| {
                lock_core::ResolvedDependency::new(
                    t.name.as_str().to_string(),
                    t.version.as_str().to_string(),
                )
            })
            .collect();

        lock_core::verify_dependencies(&self.dependencies, &resolved)
    }

    /// Get a lock entry by name
    pub fn get(&self, name: &str) -> Option<&LockEntry> {
        lock_core::find_entry(&self.dependencies, name)
    }
}

impl LockEntry {
    /// Create a lock entry from a resolved target
    pub fn from_target(target: &Target) -> Option<Self> {
        // Only create entries for dependencies
        if !matches!(target.kind, TargetKind::Dep | TargetKind::Device) {
            return None;
        }

        // PLAN-635 D6: local path deps enter the lock without a git commit —
        // the resolved path IS the reproducibility anchor (junction/symlink
        // materialization is deterministic from the pac.at declaration).
        // Git deps keep their commit pin when the dep dir is a git repo.
        let at_str: AutoStr = target.at.to_string().into();
        let commit = get_git_commit(&at_str).unwrap_or_default();

        Some(Self {
            name: target.name.as_str().to_string(),
            version: target.version.as_str().to_string(),
            url: target.from.to_string(),
            commit,
            path: target.at.to_string(),
        })
    }
}

/// Get the current git commit SHA for a repository
fn get_git_commit(path: &AutoStr) -> Option<String> {
    use std::process::Command;

    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(path.as_str())
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_file_new() {
        let lock = LockFile::new();
        assert!(lock.dependencies.is_empty());
        assert_eq!(lock.automan_version, crate::AUTOMAN_VERSION);
    }

    #[test]
    fn test_lock_file_roundtrip_json() {
        let mut lock = LockFile::new();
        lock.dependencies.push(LockEntry {
            name: "test-lib".to_string(),
            version: "1.2.3".to_string(),
            url: "https://github.com/test/lib".to_string(),
            commit: "abc123".to_string(),
            path: "deps/test-lib".to_string(),
        });

        // Serialize
        let json = serde_json::to_string(&lock).unwrap();

        // Deserialize
        let lock2: LockFile = serde_json::from_str(&json).unwrap();

        assert_eq!(lock2.dependencies.len(), 1);
        assert_eq!(lock2.dependencies[0].name, "test-lib");
        assert_eq!(lock2.dependencies[0].version, "1.2.3");
    }

    #[test]
    fn test_lock_file_get() {
        let mut lock = LockFile::new();
        lock.dependencies.push(LockEntry {
            name: "lib1".to_string(),
            version: "1.0.0".to_string(),
            url: "https://example.com/lib1".to_string(),
            commit: "abc123".to_string(),
            path: "deps/lib1".to_string(),
        });

        let entry = lock.get("lib1");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().version, "1.0.0");

        let entry = lock.get("lib2");
        assert!(entry.is_none());
    }

    // PLAN-795 V04 — the production facade delegates to the core and the
    // old API surface stays byte-compatible. Golden bytes are frozen from
    // the pre-extraction implementation (probe captured at 579f6dec0).

    fn sample_lock() -> LockFile {
        let mut lock = LockFile::new();
        lock.dependencies.push(LockEntry {
            name: "zeta-lib".to_string(),
            version: "2.0.0".to_string(),
            url: "https://example.com/zeta".to_string(),
            commit: "c0ffee".to_string(),
            path: "deps\\zeta-lib".to_string(),
        });
        lock.dependencies.push(LockEntry {
            name: "alpha-依赖".to_string(),
            version: "1.2.3".to_string(),
            url: "".to_string(),
            commit: "".to_string(),
            path: "D:/tmp dir/依赖库".to_string(),
        });
        lock
    }

    #[test]
    fn v04_facade_serde_bytes_match_frozen_golden() {
        let lock = sample_lock();
        assert_eq!(
            toml::to_string_pretty(&lock).unwrap(),
            GOLDEN_TOML,
            "TOML bytes drifted from the frozen pre-extraction format"
        );
        assert_eq!(
            serde_json::to_string_pretty(&lock).unwrap(),
            GOLDEN_JSON,
            "JSON bytes drifted from the frozen pre-extraction format"
        );
    }

    #[test]
    fn v04_facade_save_load_roundtrip_matches_core() {
        let dir = tempfile::tempdir().unwrap();
        let lock = sample_lock();

        let toml_path = AutoPath::new(
            dir.path()
                .join("nested/pac.lock")
                .to_string_lossy()
                .to_string()
                .as_str(),
        );
        lock.save(&toml_path).unwrap();
        assert!(dir.path().join("nested/pac.lock").is_file());

        let json_path = AutoPath::new(
            dir.path()
                .join("nested/pac.lock.json")
                .to_string_lossy()
                .to_string()
                .as_str(),
        );
        lock.save(&json_path).unwrap();

        for path in [toml_path, json_path] {
            let loaded = LockFile::load(&path).unwrap();
            assert_eq!(loaded.automan_version, lock.automan_version);
            assert_eq!(loaded.dependencies.len(), 2);
            // Load preserves the stored order (no re-sort on read).
            assert_eq!(loaded.dependencies[0].name, "zeta-lib");
            assert_eq!(loaded.dependencies[1].name, "alpha-依赖");
            assert_eq!(loaded.to_core(), lock.to_core());

            // Core direct input must produce the identical file: the facade
            // is a projection, not a second codec.
            let core_lock = loaded.to_core();
            core_lock.save(&path).unwrap();
            let reloaded = LockFile::load(&path).unwrap();
            assert_eq!(reloaded.to_core(), core_lock);
        }
    }

    #[test]
    fn v04_from_targets_projects_kinds_versions_and_orders() {
        let dir = tempfile::tempdir().unwrap();
        let local_dep_dir = dir.path().join("local-dep");
        std::fs::create_dir_all(&local_dep_dir).unwrap();

        let mut app = Target::new("my-app", TargetKind::App);
        app.version = "1.0.0".into();
        let mut dep = Target::new("zeta-lib", TargetKind::Dep);
        dep.version = "2.0.0".into();
        dep.from = "https://example.com/zeta".into();
        dep.at = local_dep_dir.to_string_lossy().to_string().into();
        let mut device = Target::new("alpha-dev", TargetKind::Device);
        device.version = "0.3.0".into();
        let mut test_only = Target::new("test-thing", TargetKind::Test);
        test_only.version = "5.0.0".into();

        let targets = [app, dep, device, test_only];
        let lock = LockFile::from_targets(&targets);

        // Only Dep/Device entries, sorted by name. The temp dir is not a git
        // repo, so the local dep's commit is empty (PLAN-635 D6).
        let names: Vec<&str> = lock.dependencies.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, ["alpha-dev", "zeta-lib"]);
        assert_eq!(lock.dependencies[1].path, local_dep_dir.to_string_lossy());
        assert!(lock.dependencies[1].commit.is_empty());

        // verify passes against the same targets (materialized local path).
        assert_eq!(lock.verify(&targets), Ok(()));
    }

    #[test]
    fn v04_verify_delegates_with_frozen_error_text() {
        let mut lock = LockFile::new();
        lock.dependencies.push(LockEntry {
            name: "lib1".to_string(),
            version: "1.0.0".to_string(),
            url: String::new(),
            commit: "abc".to_string(),
            path: String::new(),
        });

        let mut dep = Target::new("lib1", TargetKind::Dep);
        dep.version = "2.0.0".into();
        let mut ghost = Target::new("ghost", TargetKind::Device);
        ghost.version = "0.1.0".into();

        let err = lock.verify(&[dep, ghost]).unwrap_err();
        assert_eq!(
            err,
            "Dependency 'lib1' version mismatch: lockfile has 1.0.0, but resolved 2.0.0\n\
             Dependency 'ghost' not found in lock file"
        );
    }

    #[test]
    fn v04_unmaterialized_local_path_reports_not_materialized() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("never-created");

        let mut lock = LockFile::new();
        lock.dependencies.push(LockEntry {
            name: "local-lib".to_string(),
            version: "1.0.0".to_string(),
            url: String::new(),
            commit: String::new(), // local path dep: no commit pin
            path: missing.to_string_lossy().to_string(),
        });

        let mut dep = Target::new("local-lib", TargetKind::Dep);
        dep.version = "1.0.0".into();
        dep.at = missing.to_string_lossy().to_string().into();
        let targets = [dep];

        let err = lock.verify(&targets).unwrap_err();
        assert_eq!(
            err,
            format!(
                "Dependency 'local-lib' path '{}' is not materialized",
                missing.to_string_lossy()
            )
        );

        // Positive control: materialize the directory and the same check passes.
        std::fs::create_dir_all(&missing).unwrap();
        assert_eq!(lock.verify(&targets), Ok(()));
    }

    #[test]
    fn v04_from_target_observes_real_git_commit() {
        let repo = tempfile::tempdir().unwrap();
        fn git(dir: &std::path::Path, args: &[&str]) {
            let out = std::process::Command::new("git")
                .args(args)
                .current_dir(dir)
                .output()
                .expect("git binary");
            assert!(
                out.status.success(),
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&out.stderr)
            );
        }
        git(repo.path(), &["init", "-q"]);
        git(repo.path(), &["config", "user.email", "t@t"]);
        git(repo.path(), &["config", "user.name", "t"]);
        std::fs::write(repo.path().join("lib.at"), "// lib").unwrap();
        git(repo.path(), &["add", "."]);
        git(repo.path(), &["commit", "-q", "-m", "init"]);

        let mut dep = Target::new("git-lib", TargetKind::Dep);
        dep.version = "1.0.0".into();
        dep.from = "https://example.com/git-lib".into();
        dep.at = repo.path().to_string_lossy().to_string().into();

        let entry = LockEntry::from_target(&dep).expect("dep entry");
        assert_eq!(entry.name, "git-lib");
        assert!(!entry.commit.is_empty());
        // The observed commit is the repository's HEAD.
        let head = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(repo.path())
            .output()
            .unwrap();
        assert_eq!(entry.commit, String::from_utf8_lossy(&head.stdout).trim());
    }

    // Frozen goldens captured from the pre-extraction implementation at
    // 579f6dec0 (probe over the same sample_lock data; probe deleted after
    // capture). TOML keeps toml-rs' literal-string form for the backslash
    // path and a trailing newline; JSON has no trailing newline.
    const GOLDEN_TOML: &str = r#"automan_version = "0.1.0"

[[dependencies]]
name = "zeta-lib"
version = "2.0.0"
url = "https://example.com/zeta"
commit = "c0ffee"
path = 'deps\zeta-lib'

[[dependencies]]
name = "alpha-依赖"
version = "1.2.3"
url = ""
commit = ""
path = "D:/tmp dir/依赖库"
"#;
    const GOLDEN_JSON: &str = r#"{
  "automan_version": "0.1.0",
  "dependencies": [
    {
      "name": "zeta-lib",
      "version": "2.0.0",
      "url": "https://example.com/zeta",
      "commit": "c0ffee",
      "path": "deps\\zeta-lib"
    },
    {
      "name": "alpha-依赖",
      "version": "1.2.3",
      "url": "",
      "commit": "",
      "path": "D:/tmp dir/依赖库"
    }
  ]
}"#;
}
