//! Build cache for incremental compilation
//!
//! This module provides caching functionality to avoid unnecessary work
//! when source files haven't changed.

use auto_val::AutoPath;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Cached information about a file
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedFile {
    /// Last modification time
    modified: u64,
    /// File size in bytes
    size: u64,
    /// Hash of the file content (for more accurate change detection)
    hash: Option<String>,
}

/// Build cache tracking source files and their states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildCache {
    /// Cached source files by path
    files: HashMap<String, CachedFile>,
    /// Cache format version
    version: u32,
}

impl BuildCache {
    /// Create a new empty cache
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            version: 1,
        }
    }

    /// Load cache from disk
    pub fn load(path: &AutoPath) -> Result<Self, String> {
        let cache_path = path.join(".am/cache.json");
        if !cache_path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(cache_path.path())
            .map_err(|e| format!("Failed to read cache file: {}", e))?;

        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse cache file: {}", e))
    }

    /// Save cache to disk
    pub fn save(&self, path: &AutoPath) -> Result<(), String> {
        let cache_path = path.join(".am/cache.json");

        // Ensure directory exists
        if let Some(parent) = cache_path.path().parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize cache: {}", e))?;

        fs::write(cache_path.path(), content)
            .map_err(|e| format!("Failed to write cache file: {}", e))?;

        Ok(())
    }

    /// Check if a file has been modified since last cache
    pub fn is_dirty(&self, file_path: &str) -> bool {
        // Get file metadata
        let metadata = match fs::metadata(file_path) {
            Ok(meta) => meta,
            Err(_) => return true, // File doesn't exist or can't be accessed
        };

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let size = metadata.len();

        // Check if file is in cache
        match self.files.get(file_path) {
            Some(cached) => {
                // File hasn't changed if modification time and size match
                cached.modified != modified || cached.size != size
            }
            None => true, // New file, needs processing
        }
    }

    /// Mark a file as processed (update cache)
    pub fn mark_processed(&mut self, file_path: &str) {
        let metadata = match fs::metadata(file_path) {
            Ok(meta) => meta,
            Err(_) => return,
        };

        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let size = metadata.len();

        self.files.insert(
            file_path.to_string(),
            CachedFile {
                modified,
                size,
                hash: None, // TODO: Could add content hashing for more accuracy
            },
        );
    }

    /// Check if any transpilation is needed
    pub fn needs_transpilation(&self, auto_files: &[String]) -> bool {
        auto_files.iter().any(|f| self.is_dirty(f))
    }

    /// Get list of dirty (modified) files
    pub fn get_dirty_files(&self, files: &[String]) -> Vec<String> {
        files.iter()
            .filter(|f| self.is_dirty(f))
            .cloned()
            .collect()
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.files.clear();
    }

    /// Remove non-existent files from cache
    pub fn cleanup(&mut self) {
        self.files.retain(|path, _| Path::new(path).exists());
    }
}

impl Default for BuildCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new() {
        let cache = BuildCache::new();
        assert!(cache.files.is_empty());
        assert_eq!(cache.version, 1);
    }

    #[test]
    fn test_cache_default() {
        let cache = BuildCache::default();
        assert!(cache.files.is_empty());
    }

    #[test]
    fn test_cache_roundtrip() {
        let mut cache = BuildCache::new();
        cache.files.insert(
            "test.at".to_string(),
            CachedFile {
                modified: 12345,
                size: 1024,
                hash: None,
            },
        );

        let json = serde_json::to_string(&cache).unwrap();
        let cache2: BuildCache = serde_json::from_str(&json).unwrap();

        assert_eq!(cache2.files.len(), 1);
        assert_eq!(cache2.files["test.at"].modified, 12345);
    }

    #[test]
    fn test_is_dirty() {
        let cache = BuildCache::new();
        // New file is always dirty
        assert!(cache.is_dirty("nonexistent.at"));
    }

    #[test]
    fn test_mark_processed() {
        let mut cache = BuildCache::new();
        // Create a temporary file for testing
        use std::io::Write;
        let temp_file = "test_cache_temp.at";
        let mut file = std::fs::File::create(temp_file).unwrap();
        file.write_all(b"test content").unwrap();
        drop(file);

        cache.mark_processed(temp_file);

        // After marking, file should be in cache
        assert!(cache.files.contains_key(temp_file));

        // Cleanup
        let _ = std::fs::remove_file(temp_file);
    }
}
