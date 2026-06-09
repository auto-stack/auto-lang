//! UI Cache for incremental code generation (Plan 134)
//!
//! Manages persistent cache of generated UI files for incremental compilation.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::ui_artifact::UIArtifact;

/// Persistent cache for UI incremental compilation
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UICache {
    /// File path -> content hash
    file_hashes: HashMap<PathBuf, u64>,
    /// File path -> generated artifacts
    artifacts: HashMap<PathBuf, Vec<UIArtifact>>,
    /// Cache version for migration
    version: u32,
    /// Hash of the .api_functions file content — changed API config invalidates all cached artifacts
    api_functions_hash: Option<u64>,
}

impl UICache {
    const VERSION: u32 = 1;

    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            file_hashes: HashMap::new(),
            artifacts: HashMap::new(),
            version: Self::VERSION,
            api_functions_hash: None,
        }
    }

    /// Get cache file path for a project
    pub fn cache_path(project_root: &Path) -> PathBuf {
        project_root.join(".auto").join("ui-cache.json")
    }

    /// Load cache from project root
    pub fn load(project_root: &Path) -> Self {
        let path = Self::cache_path(project_root);
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<Self>(&content) {
                        Ok(cache) => {
                            // Version check - invalidate if version mismatch
                            if cache.version == Self::VERSION {
                                return cache;
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse UI cache: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read UI cache: {}", e);
                }
            }
        }
        Self::new()
    }

    /// Save cache to project root
    pub fn save(&self, project_root: &Path) -> std::io::Result<()> {
        let path = Self::cache_path(project_root);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&path, content)
    }

    /// Check if a file needs regeneration
    pub fn is_dirty(&self, source_path: &Path, current_hash: u64) -> bool {
        match self.file_hashes.get(source_path) {
            Some(&cached_hash) => cached_hash != current_hash,
            None => true,
        }
    }

    /// Check if `.api_functions` config has changed since last cache write.
    /// If so, all cached artifacts are stale (API imports may have changed).
    /// Returns true if the cache was invalidated.
    pub fn invalidate_if_api_functions_changed(&mut self, api_fns_path: &Path) -> bool {
        let current_hash = fs::read_to_string(api_fns_path)
            .ok()
            .map(|content| {
                // Simple FNV-1a-style hash via seahash or just use a basic hash
                let mut hash: u64 = 0xcbf29ce484222325;
                for byte in content.bytes() {
                    hash ^= byte as u64;
                    hash = hash.wrapping_mul(0x100000001b3);
                }
                hash
            });

        let changed = match (&self.api_functions_hash, &current_hash) {
            (Some(cached), Some(current)) => cached != current,
            (None, None) => false,
            _ => true, // appeared or disappeared
        };

        if changed {
            self.clear();
            self.api_functions_hash = current_hash;
        }

        changed
    }

    /// Get artifacts for a source file
    pub fn get_artifacts(&self, source_path: &Path) -> Option<&[UIArtifact]> {
        self.artifacts.get(source_path).map(|v| v.as_slice())
    }

    /// Update cache entry for a source file
    pub fn update(&mut self, source_path: PathBuf, hash: u64, artifacts: Vec<UIArtifact>) {
        self.file_hashes.insert(source_path.clone(), hash);
        self.artifacts.insert(source_path, artifacts);
    }

    /// Remove a file from cache
    pub fn remove(&mut self, source_path: &Path) {
        self.file_hashes.remove(source_path);
        self.artifacts.remove(source_path);
    }

    /// Get all tracked source files
    pub fn tracked_files(&self) -> impl Iterator<Item = &PathBuf> {
        self.file_hashes.keys()
    }

    /// Get number of tracked files
    pub fn file_count(&self) -> usize {
        self.file_hashes.len()
    }

    /// Get total number of artifacts
    pub fn artifact_count(&self) -> usize {
        self.artifacts.values().map(|v| v.len()).sum()
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        self.file_hashes.clear();
        self.artifacts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::UIBackend;
    use tempfile::TempDir;

    #[test]
    fn test_cache_new() {
        let cache = UICache::new();
        assert_eq!(cache.file_count(), 0);
        assert_eq!(cache.artifact_count(), 0);
    }

    #[test]
    fn test_is_dirty_new_file() {
        let cache = UICache::new();
        let path = PathBuf::from("app.at");
        assert!(cache.is_dirty(&path, 12345));
    }

    #[test]
    fn test_is_dirty_unchanged_file() {
        let mut cache = UICache::new();
        let path = PathBuf::from("app.at");
        cache.update(path.clone(), 12345, vec![]);
        assert!(!cache.is_dirty(&path, 12345));
    }

    #[test]
    fn test_is_dirty_changed_file() {
        let mut cache = UICache::new();
        let path = PathBuf::from("app.at");
        cache.update(path.clone(), 12345, vec![]);
        assert!(cache.is_dirty(&path, 99999));
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = UICache::new();

        let path = PathBuf::from("app.at");
        let artifact = UIArtifact {
            source_path: path.clone(),
            widget_name: "App".to_string(),
            output_path: PathBuf::from("src/components/App.vue"),
            source_hash: 12345,
            content_hash: 67890,
            backend: UIBackend::Vue,
        };

        cache.update(path.clone(), 12345, vec![artifact]);
        cache.save(temp_dir.path()).unwrap();

        let loaded = UICache::load(temp_dir.path());
        assert_eq!(loaded.file_count(), 1);
        assert!(!loaded.is_dirty(&path, 12345));
    }
}
