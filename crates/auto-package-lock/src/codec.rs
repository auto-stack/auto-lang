//! Single-sourced `pac.lock` codec: format selection, load and save.
//!
//! Extracted verbatim from `auto-man`'s `LockFile::load`/`save` (PLAN-795).
//! Path semantics are frozen: format selection and error display use
//! `AutoPath`'s unified string (normalized separators → `/`), while file
//! operations use the underlying path. `save` serializes before touching
//! the disk, so a serialization failure never writes new data; the write
//! itself remains the historical plain `fs::write` (no atomic-rename
//! promise).

use crate::model::LockFile;
use auto_val::AutoPath;
use std::fs;

/// Filename suffix selecting the JSON format (everything else is TOML).
pub const LOCK_JSON_SUFFIX: &str = ".json";

/// Format-selection rule: a path whose unified string ends with `.json`
/// selects JSON; any other path defaults to TOML. Case-sensitive, matching
/// historical behavior.
pub fn is_json_path(path: &AutoPath) -> bool {
    path.to_string().ends_with(LOCK_JSON_SUFFIX)
}

impl LockFile {
    /// Load a lock file from disk
    pub fn load(path: &AutoPath) -> Result<Self, String> {
        let content = fs::read_to_string(path.path())
            .map_err(|e| format!("Failed to read lock file '{}': {}", path, e))?;

        // Handle both JSON and TOML formats
        if is_json_path(path) {
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse lock file JSON: {}", e))
        } else {
            // Default to TOML
            toml::from_str(&content).map_err(|e| format!("Failed to parse lock file TOML: {}", e))
        }
    }

    /// Save the lock file to disk
    pub fn save(&self, path: &AutoPath) -> Result<(), String> {
        let content = if is_json_path(path) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::LockEntry;

    #[test]
    fn json_selection_uses_unified_string_and_is_case_sensitive() {
        // Backslashes unify to '/' before the suffix check.
        assert!(is_json_path(&AutoPath::new("deps\\sub\\pac.lock.json")));
        assert!(is_json_path(&AutoPath::new("deps/sub/pac.lock.json")));
        assert!(!is_json_path(&AutoPath::new("deps/sub/pac.lock")));
        // Case-sensitive: '.JSON' falls back to TOML.
        assert!(!is_json_path(&AutoPath::new("deps/sub/pac.lock.JSON")));
    }

    #[test]
    fn load_missing_file_error_stage_is_read_with_unified_path() {
        let path = AutoPath::new("Z:/definitely/missing/pac.lock");
        let err = LockFile::load(&path).unwrap_err();
        assert!(
            err.starts_with("Failed to read lock file 'Z:/definitely/missing/pac.lock': "),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn load_bad_toml_and_bad_json_error_stages() {
        let dir = tempfile::tempdir().unwrap();
        let toml_path = dir.path().join("bad.lock");
        std::fs::write(&toml_path, "not [valid toml").unwrap();
        let err = LockFile::load(&AutoPath::new(
            toml_path.to_string_lossy().to_string().as_str(),
        ))
        .unwrap_err();
        assert!(
            err.starts_with("Failed to parse lock file TOML: "),
            "got: {err}"
        );

        let json_path = dir.path().join("bad.lock.json");
        std::fs::write(&json_path, "{ nope").unwrap();
        let err = LockFile::load(&AutoPath::new(
            json_path.to_string_lossy().to_string().as_str(),
        ))
        .unwrap_err();
        assert!(
            err.starts_with("Failed to parse lock file JSON: "),
            "got: {err}"
        );
    }

    #[test]
    fn save_creates_parent_directories_and_roundtrips_both_formats() {
        let dir = tempfile::tempdir().unwrap();
        let lock = LockFile::from_dependencies(
            "1.2.3",
            vec![LockEntry {
                name: "lib-ünicode".to_string(),
                version: "0.7.1".to_string(),
                url: "https://example.com/lib".to_string(),
                commit: "deadbeef".to_string(),
                path: "deps\\lib-ünicode".to_string(),
            }],
        );

        let toml_path = dir.path().join("nested").join("pac.lock");
        lock.save(&AutoPath::new(
            toml_path.to_string_lossy().to_string().as_str(),
        ))
        .unwrap();
        assert!(toml_path.is_file());

        let json_path = dir.path().join("nested").join("pac.lock.json");
        lock.save(&AutoPath::new(
            json_path.to_string_lossy().to_string().as_str(),
        ))
        .unwrap();
        assert!(json_path.is_file());

        for p in [&toml_path, &json_path] {
            let loaded =
                LockFile::load(&AutoPath::new(p.to_string_lossy().to_string().as_str())).unwrap();
            assert_eq!(loaded.automan_version, "1.2.3");
            assert_eq!(loaded.dependencies.len(), 1);
            assert_eq!(loaded.dependencies[0].name, "lib-ünicode");
            assert_eq!(loaded.dependencies[0].path, "deps\\lib-ünicode");
        }
    }
}
