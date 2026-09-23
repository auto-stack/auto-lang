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
    /// Content fingerprint of the generator sources (src/ui_gen/**) this
    /// binary was built with (Plan 698 SD-01). A mismatch invalidates all
    /// cached artifacts even when source hashes are unchanged — 692 W-1
    /// root fix (generator upgrade replayed stale outputs as "fresh").
    generator_fingerprint: Option<String>,
}

impl UICache {
    const VERSION: u32 = 2;

    /// Generator fingerprint stamped by build.rs (`AUTO_UI_GEN_FINGERPRINT`).
    /// `None` when the binary was built without it (foreign build).
    pub fn generator_fingerprint() -> Option<String> {
        option_env!("AUTO_UI_GEN_FINGERPRINT").map(|s| s.to_string())
    }

    /// A cached fingerprint mismatches the current one when they differ, or
    /// when one side appeared/disappeared (same semantics as
    /// `api_functions_hash`).
    fn fingerprint_mismatch(cached: &Option<String>, current: &Option<String>) -> bool {
        match (cached, current) {
            (Some(c), Some(n)) => c != n,
            (None, None) => false,
            _ => true,
        }
    }

    fn fresh_with(fingerprint: Option<String>) -> Self {
        Self {
            file_hashes: HashMap::new(),
            artifacts: HashMap::new(),
            version: Self::VERSION,
            api_functions_hash: None,
            generator_fingerprint: fingerprint,
        }
    }

    /// Create a new empty cache
    pub fn new() -> Self {
        Self::fresh_with(Self::generator_fingerprint())
    }

    /// Get cache file path for a project
    pub fn cache_path(project_root: &Path) -> PathBuf {
        project_root.join(".auto").join("ui-cache.json")
    }

    /// Load cache from project root
    pub fn load(project_root: &Path) -> Self {
        Self::load_with(project_root, Self::generator_fingerprint())
    }

    fn load_with(project_root: &Path, current_fp: Option<String>) -> Self {
        let path = Self::cache_path(project_root);
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str::<Self>(&content) {
                        Ok(cache) => {
                            // Version check - invalidate if version mismatch.
                            // The 1→2 bump (Plan 698) also retires old caches
                            // that carry no fingerprint field at all.
                            if cache.version == Self::VERSION {
                                // Generator fingerprint check (Plan 698
                                // SD-01): a generator upgrade invalidates
                                // every entry even when source hashes are
                                // unchanged. Stays loud until the next
                                // generation re-stamps the cache on save.
                                if Self::fingerprint_mismatch(
                                    &cache.generator_fingerprint,
                                    &current_fp,
                                ) {
                                    eprintln!(
                                        "Warning: UI cache invalidated (generator fingerprint changed)"
                                    );
                                    return Self::fresh_with(current_fp);
                                }
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
        Self::fresh_with(current_fp)
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

    /// Plan 015 P0#2: drop entries whose source .at no longer exists, so the
    /// saved cache only lists artifacts that are still live. Returns the
    /// removed artifact output paths (relative, as recorded).
    pub fn retain_existing_sources(&mut self, source_exists: &dyn Fn(&Path) -> bool) -> Vec<PathBuf> {
        let mut removed_outputs = Vec::new();
        let mut stale: Vec<PathBuf> = Vec::new();
        for src in self.file_hashes.keys() {
            if !source_exists(src) {
                stale.push(src.clone());
            }
        }
        for src in &stale {
            if let Some(arts) = self.artifacts.remove(src) {
                for a in arts {
                    removed_outputs.push(a.output_path.clone());
                }
            }
        }
        for src in &stale {
            self.file_hashes.remove(src);
        }
        removed_outputs
    }

    /// Plan 015 P0#2: every artifact output path currently recorded (i.e.
    /// files the generator owns). A .vue in the components dir that is NOT
    /// here but IS in a previous cache snapshot is a stale regen product.
    pub fn all_artifact_outputs(&self) -> std::collections::HashSet<PathBuf> {
        self.artifacts
            .values()
            .flat_map(|v| v.iter().map(|a| a.output_path.clone()))
            .collect()
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

    // --- Plan 698 SD-01: generator fingerprint in the cache key ---

    fn seeded_cache(fp: Option<&str>) -> UICache {
        let mut cache = UICache::new();
        cache.update(PathBuf::from("app.at"), 12345, vec![]);
        cache.generator_fingerprint = fp.map(|s| s.to_string());
        cache
    }

    #[test]
    fn test_fingerprint_match_retained() {
        let temp_dir = TempDir::new().unwrap();
        seeded_cache(Some("F1")).save(temp_dir.path()).unwrap();

        let loaded = UICache::load_with(temp_dir.path(), Some("F1".into()));
        assert_eq!(loaded.file_count(), 1, "matching fingerprint keeps entries");
        assert_eq!(loaded.generator_fingerprint, Some("F1".into()));
    }

    #[test]
    fn test_fingerprint_mismatch_invalidates_all() {
        let temp_dir = TempDir::new().unwrap();
        seeded_cache(Some("F1")).save(temp_dir.path()).unwrap();

        let loaded = UICache::load_with(temp_dir.path(), Some("F2".into()));
        assert_eq!(loaded.file_count(), 0, "changed fingerprint clears entries");
        assert_eq!(loaded.generator_fingerprint, Some("F2".into()), "fresh cache re-stamps current fp");
    }

    #[test]
    fn test_fingerprint_appeared_or_disappeared_invalidates() {
        let temp_dir = TempDir::new().unwrap();
        // Cache written without a fingerprint, binary now has one.
        seeded_cache(None).save(temp_dir.path()).unwrap();
        let loaded = UICache::load_with(temp_dir.path(), Some("F1".into()));
        assert_eq!(loaded.file_count(), 0);

        // Cache written with a fingerprint, binary built without one.
        let temp_dir2 = TempDir::new().unwrap();
        seeded_cache(Some("F1")).save(temp_dir2.path()).unwrap();
        let loaded = UICache::load_with(temp_dir2.path(), None);
        assert_eq!(loaded.file_count(), 0);
    }

    #[test]
    fn test_version1_cache_without_fingerprint_discarded() {
        let temp_dir = TempDir::new().unwrap();
        let legacy = r#"{
  "file_hashes": {},
  "artifacts": {},
  "version": 1,
  "api_functions_hash": null
}"#;
        let cache_path = UICache::cache_path(temp_dir.path());
        std::fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
        std::fs::write(&cache_path, legacy).unwrap();

        let loaded = UICache::load_with(temp_dir.path(), Some("F1".into()));
        assert_eq!(loaded.version, 2, "v1 cache migrates by being discarded");
        assert_eq!(loaded.generator_fingerprint, Some("F1".into()));
    }
}
