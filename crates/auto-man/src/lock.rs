//! Dependency lock file management
//!
//! This module handles the `pac.lock` file that records the exact versions
//! of all dependencies used in a build, ensuring reproducible builds.

use crate::target::Target;

use auto_val::{AutoPath, AutoStr};
use serde::{Deserialize, Serialize};
use std::fs;

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

/// The complete lock file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockFile {
    /// AutoMan version that created this lock file
    pub automan_version: String,
    /// List of all locked dependencies
    pub dependencies: Vec<LockEntry>,
}

impl LockFile {
    /// Create a new empty lock file
    pub fn new() -> Self {
        Self {
            automan_version: crate::AUTOMAN_VERSION.to_string(),
            dependencies: Vec::new(),
        }
    }

    /// Generate a lock file from resolved targets
    pub fn from_targets(targets: &[Target]) -> Self {
        let mut lock_file = Self::new();

        for target in targets {
            if target.kind == crate::target::TargetKind::Dep
                || target.kind == crate::target::TargetKind::Device
            {
                if let Some(entry) = LockEntry::from_target(target) {
                    lock_file.dependencies.push(entry);
                }
            }
        }

        // Sort by name for consistent ordering
        lock_file.dependencies.sort_by(|a, b| a.name.cmp(&b.name));

        lock_file
    }

    /// Load a lock file from disk
    pub fn load(path: &AutoPath) -> Result<Self, String> {
        let content = fs::read_to_string(path.path())
            .map_err(|e| format!("Failed to read lock file '{}': {}", path, e))?;

        // Handle both JSON and TOML formats
        let path_str = path.to_string();
        if path_str.ends_with(".json") {
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse lock file JSON: {}", e))
        } else {
            // Default to TOML
            toml::from_str(&content).map_err(|e| format!("Failed to parse lock file TOML: {}", e))
        }
    }

    /// Save the lock file to disk
    pub fn save(&self, path: &AutoPath) -> Result<(), String> {
        let path_str = path.to_string();
        let content = if path_str.ends_with(".json") {
            serde_json::to_string_pretty(self)
                .map_err(|e| format!("Failed to serialize lock file JSON: {}", e))?
        } else {
            // Default to TOML for better readability
            toml::to_string_pretty(self)
                .map_err(|e| format!("Failed to serialize lock file TOML: {}", e))?
        };

        // Ensure parent directory exists
        if let Some(parent) = path.path().parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create lock file directory: {}", e))?;
        }

        fs::write(path.path(), content)
            .map_err(|e| format!("Failed to write lock file '{}': {}", path, e))?;

        Ok(())
    }

    /// Verify that all dependencies match the lock file
    pub fn verify(&self, targets: &[Target]) -> Result<(), String> {
        let mut errors = Vec::new();

        for target in targets {
            if target.kind != crate::target::TargetKind::Dep
                && target.kind != crate::target::TargetKind::Device
            {
                continue;
            }

            let name = target.name.as_str();
            let version = target.version.as_str();

            match self.dependencies.iter().find(|d| d.name == name) {
                Some(locked) => {
                    if locked.version != version {
                        errors.push(format!(
                            "Dependency '{}' version mismatch: lockfile has {}, but resolved {}",
                            name, locked.version, version
                        ));
                    }
                }
                None => {
                    errors.push(format!("Dependency '{}' not found in lock file", name));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("\n"))
        }
    }

    /// Get a lock entry by name
    pub fn get(&self, name: &str) -> Option<&LockEntry> {
        self.dependencies.iter().find(|d| d.name == name)
    }
}

impl LockEntry {
    /// Create a lock entry from a resolved target
    pub fn from_target(target: &Target) -> Option<Self> {
        // Only create entries for dependencies
        if target.kind != crate::target::TargetKind::Dep
            && target.kind != crate::target::TargetKind::Device
        {
            return None;
        }

        // Try to get git commit SHA
        let at_str: AutoStr = target.at.to_string().into();
        let commit = get_git_commit(&at_str)?;

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
}
