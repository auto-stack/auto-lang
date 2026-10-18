// BlobStore: Content-addressable storage for binary artifacts
//
// **Plan 082**: AutoCache blob storage layer
//
// Features:
// - Two-level sharding (a1b2... -> blobs/a1/a1b2...)
// - Atomic writes with temp + rename
// - Hard link support for cache hits
// - Automatic directory creation

use std::fs;
use std::path::{Path, PathBuf};
use std::io;

/// BlobStore-specific result type
pub type Result<T> = std::result::Result<T, Error>;

/// BlobStore errors
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

/// Blob store for binary artifacts
///
/// Uses content-addressable storage with two-level sharding:
/// Hash "a1b2c3d4..." -> "blobs/a1/a1b2c3d4..."
pub struct BlobStore {
    root: PathBuf,
}

impl BlobStore {
    /// Create new BlobStore at specified root directory
    ///
    /// # Arguments
    /// * `root` - Root directory for blob storage
    ///
    /// # Creates
    /// - `root/blobs/` - Main blob storage directory
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Convert hash key to blob path
    ///
    /// Uses two-level sharding to prevent too many files in one directory.
    ///
    /// # Sharding Example
    ///
    /// Hash "a1b2c3d4..." -> "root/blobs/a1/a1b2c3d4..."
    /// Hash "f9e8d7c6..." -> "root/blobs/f9/f9e8d7c6..."
    fn hash_to_path(&self, hash: &str) -> PathBuf {
        let prefix = &hash[0..2.min(hash.len())];
        self.root
            .join("blobs")
            .join(prefix)
            .join(hash)
    }

    /// Store blob from source file
    ///
    /// Uses atomic write pattern:
    /// 1. Write to temporary file
    /// 2. Atomic rename to final location
    ///
    /// # Arguments
    /// * `hash` - Content hash key
    /// * `source` - Source file path
    ///
    /// # Returns
    /// Path to stored blob
    pub fn put(&self, hash: &str, source: &Path) -> Result<PathBuf> {
        let dest = self.hash_to_path(hash);

        // Create parent directories
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write to temp file first (for atomicity)
        let temp = dest.with_extension("tmp");
        fs::copy(source, &temp)?;

        // Atomic rename
        fs::rename(&temp, &dest)?;

        Ok(dest)
    }

    /// Get blob path if exists
    ///
    /// # Arguments
    /// * `hash` - Content hash key
    ///
    /// # Returns
    /// Some(path) if blob exists, None otherwise
    pub fn get(&self, hash: &str) -> Option<PathBuf> {
        let path = self.hash_to_path(hash);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Check if blob exists
    ///
    /// # Arguments
    /// * `hash` - Content hash key
    pub fn contains(&self, hash: &str) -> bool {
        self.get(hash).is_some()
    }

    /// Remove blob from storage
    ///
    /// # Arguments
    /// * `hash` - Content hash key
    pub fn remove(&self, hash: &str) -> Result<()> {
        let path = self.hash_to_path(hash);
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Create hard link from blob to target
    ///
    /// Hard links are zero-copy and instantaneous.
    /// Falls back to copy if hard link fails (e.g., cross-device).
    ///
    /// # Arguments
    /// * `hash` - Content hash key
    /// * `target` - Target path for hard link
    pub fn link_or_copy(&self, hash: &str, target: &Path) -> Result<()> {
        let source = self.hash_to_path(hash);

        // Try hard link first (zero-copy)
        match fs::hard_link(&source, target) {
            Ok(_) => {
                log::debug!("Created hard link: {} -> {}", source.display(), target.display());
                Ok(())
            }
            Err(e) => {
                // Cross-device or other error: fall back to copy
                log::debug!("Hard link failed: {}, copying file", e);
                fs::copy(&source, target)?;
                log::debug!("Copied: {} -> {}", source.display(), target.display());
                Ok(())
            }
        }
    }

    /// Get total size of all blobs in bytes
    ///
    /// Scans all blob directories recursively.
    pub fn total_size(&self) -> Result<u64> {
        let blobs_dir = self.root.join("blobs");

        if !blobs_dir.exists() {
            return Ok(0);
        }

        let mut total = 0u64;

        for entry in fs::read_dir(blobs_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip non-directories
            if !path.is_dir() {
                continue;
            }

            // Scan each prefix directory
            for blob_entry in fs::read_dir(path)? {
                let blob_entry = blob_entry?;
                let blob_path = blob_entry.path();

                // Only count files, not directories
                if blob_path.is_file() {
                    total += blob_entry.metadata()?.len();
                }
            }
        }

        Ok(total)
    }

    /// Count total number of blobs
    pub fn count(&self) -> Result<usize> {
        let blobs_dir = self.root.join("blobs");

        if !blobs_dir.exists() {
            return Ok(0);
        }

        let mut count = 0usize;

        for entry in fs::read_dir(blobs_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            for blob_entry in fs::read_dir(path)? {
                let blob_entry = blob_entry?;
                if blob_entry.path().is_file() {
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    /// Clear all blobs
    ///
    /// Removes the entire `blobs/` directory.
    pub fn clear_all(&self) -> Result<()> {
        let blobs_dir = self.root.join("blobs");

        if blobs_dir.exists() {
            fs::remove_dir_all(&blobs_dir)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_hash_to_path() {
        let temp_dir = std::env::temp_dir();
        let store = BlobStore::new(temp_dir.clone());

        let hash = "a1b2c3d4e5f6";
        let path = store.hash_to_path(hash);

        assert_eq!(path, temp_dir.join("blobs").join("a1").join(hash));
    }

    #[test]
    fn test_put_and_get() {
        let temp_dir = std::env::temp_dir().join("test_blob_store");
        std::fs::create_dir_all(&temp_dir).ok();

        let store = BlobStore::new(temp_dir.clone());

        // Create test file
        let test_file = temp_dir.join("test.txt");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"Hello, BlobStore!").unwrap();

        // Put blob
        let hash = "test123";
        let blob_path = store.put(hash, &test_file).unwrap();
        assert!(blob_path.exists());

        // Get blob
        let retrieved = store.get(hash);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), blob_path);

        // Check contains
        assert!(store.contains(hash));

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_remove() {
        let temp_dir = std::env::temp_dir().join("test_blob_remove");
        std::fs::create_dir_all(&temp_dir).ok();

        let store = BlobStore::new(temp_dir.clone());

        // Create test file
        let test_file = temp_dir.join("test.txt");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"Test content").unwrap();

        // Put and verify
        let hash = "test456";
        store.put(hash, &test_file).unwrap();
        assert!(store.contains(hash));

        // Remove and verify
        store.remove(hash).unwrap();
        assert!(!store.contains(hash));

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_sharding() {
        let temp_dir = std::env::temp_dir().join("test_sharding");
        std::fs::create_dir_all(&temp_dir).ok();

        let store = BlobStore::new(temp_dir.clone());

        // Create test file
        let test_file = temp_dir.join("test.txt");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"Shard test").unwrap();

        // Store multiple blobs with different prefixes
        for hash in &["a10001", "a20002", "b30003", "b40004", "c50005"] {
            store.put(hash, &test_file).unwrap();
        }

        // Verify sharding structure
        let blobs_dir = temp_dir.join("blobs");
        assert!(blobs_dir.join("a1").exists());
        assert!(blobs_dir.join("a2").exists());
        assert!(blobs_dir.join("b3").exists());
        assert!(blobs_dir.join("b4").exists());
        assert!(blobs_dir.join("c5").exists());

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_total_size() {
        let temp_dir = std::env::temp_dir().join("test_size");
        std::fs::create_dir_all(&temp_dir).ok();

        let store = BlobStore::new(temp_dir.clone());

        // Create test files
        for i in 1..=3 {
            let test_file = temp_dir.join(format!("test{}.txt", i));
            let mut file = File::create(&test_file).unwrap();
            file.write_all(vec![b'a'; 100 * i].as_slice()).unwrap();
            store.put(&format!("hash{}", i), &test_file).unwrap();
        }

        // Total size: 100 + 200 + 300 = 600
        let size = store.total_size().unwrap();
        assert_eq!(size, 600);

        // Cleanup
        store.clear_all().unwrap();
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_count() {
        let temp_dir = std::env::temp_dir().join("test_count");
        std::fs::create_dir_all(&temp_dir).ok();

        let store = BlobStore::new(temp_dir.clone());

        // Create test file
        let test_file = temp_dir.join("test.txt");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"Count test").unwrap();

        // Store multiple blobs
        for i in 1..=5 {
            store.put(&format!("hash{}", i), &test_file).unwrap();
        }

        // Count should be 5
        let count = store.count().unwrap();
        assert_eq!(count, 5);

        // Cleanup
        store.clear_all().unwrap();
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_link_or_copy() {
        let temp_dir = std::env::temp_dir().join("test_link");
        std::fs::create_dir_all(&temp_dir).ok();

        let store = BlobStore::new(temp_dir.clone());

        // Create and store blob
        let test_file = temp_dir.join("test.txt");
        let mut file = File::create(&test_file).unwrap();
        file.write_all(b"Link test").unwrap();
        store.put("link123", &test_file).unwrap();

        // Create hard link
        let target = temp_dir.join("link.txt");
        store.link_or_copy("link123", &target).unwrap();
        assert!(target.exists());

        // Verify content
        let content = std::fs::read_to_string(&target).unwrap();
        assert_eq!(content, "Link test");

        // Cleanup
        store.clear_all().unwrap();
        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
