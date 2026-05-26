// AutoCache: Global build cache for AutoLang projects
//
// **Plan 082**: Content-addressable storage system for compiled artifacts
//
// Features:
// - SQLite metadata index with WAL mode for concurrency
// - Filesystem blob storage with 2-level sharding
// - Cross-project artifact sharing
// - LRU garbage collection
// - Hard link optimization for cache hits
//
// **Architecture**:
// ~/.auto/cache/
// ├── index.db         # SQLite metadata
// ├── blobs/           # Binary artifacts (sharded)
// │   ├── a1/          # Hash prefix (first 2 chars)
// │   │   └── a1b2c3...
// │   └── f9/
// └── locks/           # Process locks

// Plan 092: Rust FFI Sandbox
pub mod sandbox;
pub mod registry;

// Plan 212 Phase 3C-v2: sig_code encoding for FFI signatures
pub mod sig_code;
pub mod scanner;

// Re-export main types for convenience
pub use sandbox::{CrateMetadata, CrateSource, Sandbox, SandboxError};
pub use registry::{CrateRegistry, RegistryError};

use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// AutoCache: Global build cache for AutoLang
///
/// Provides content-addressable storage for compiled artifacts across projects.
/// Uses SQLite for metadata and filesystem for binary blobs.
pub struct AutoCache {
    db: Arc<Mutex<Connection>>,
    blobs: BlobStore,
    _gc: GarbageCollector,  // Stored for future use
    max_size_gb: u64,
}

/// Artifact metadata stored in SQLite
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArtifactMetadata {
    pub hash_key: String,        // SHA256 hex (primary key)
    pub blob_path: PathBuf,
    pub artifact_type: ArtifactType,
    pub file_size: u64,
    pub created_at: u64,         // UNIX timestamp
    pub last_used_at: u64,       // UNIX timestamp
    pub access_count: u64,
    pub source_hash: String,      // AIE interface hash
    pub project_name: String,    // Origin project
    pub module_name: String,     // Origin module
}

/// Type of compiled artifact
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[repr(i32)]
pub enum ArtifactType {
    TranspiledC = 0,        // .c files from a2c
    TranspiledCHeader = 1,  // .h files from a2c
    TranspiledRust = 2,     // .rs files from a2r
    Bytecode = 3,           // .bc files from AutoVM
    CompiledObject = 4,     // .o/.obj files from C compilation

    // Plan 092: Rust FFI Sandbox
    RustCrateLibrary = 5,   // Compiled .so/.dylib/.dll
    RustCrateSource = 6,    // Source tarball (.crate)
}

/// Cache integrity verification report
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    pub metadata_entries: u64,
    pub blob_files: u64,
    pub corrupted_entries: u64,
    pub orphaned_files: u64,
    pub is_valid: bool,
}

impl std::fmt::Display for ArtifactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtifactType::TranspiledC => write!(f, "C"),
            ArtifactType::TranspiledCHeader => write!(f, "C Header"),
            ArtifactType::TranspiledRust => write!(f, "Rust"),
            ArtifactType::Bytecode => write!(f, "Bytecode"),
            ArtifactType::CompiledObject => write!(f, "Object"),
            ArtifactType::RustCrateLibrary => write!(f, "Rust Crate Lib"),
            ArtifactType::RustCrateSource => write!(f, "Rust Crate Src"),
        }
    }
}

/// AutoCache errors
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Cache not found at: {0}")]
    NotFound(String),

    #[error("Cache directory error: {0}")]
    CacheDir(String),

    #[error("Storage error: {0}")]
    Storage(String),
}

impl From<storage::Error> for CacheError {
    fn from(err: storage::Error) -> Self {
        CacheError::Storage(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, CacheError>;

impl AutoCache {
    /// Create or open AutoCache at default location
    ///
    /// # Location
    /// - Windows: `C:\Users\<user>\.auto\cache`
    /// - Linux/macOS: `/home/<user>/.auto/cache`
    ///
    /// # Creates
    /// - Cache directory structure
    /// - SQLite database with WAL mode
    /// - Blob storage directories
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        // Ensure cache directory exists
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| CacheError::CacheDir(format!("Failed to create cache directory: {}", e)))?;

        // Initialize SQLite database
        let db_path = cache_dir.join("index.db");
        let db = Self::initialize_database(&db_path)?;

        // Initialize blob store
        let blobs = BlobStore::new(cache_dir.join("blobs"));

        // Initialize GC with 10GB default limit
        let gc = GarbageCollector::new(10);

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            blobs,
            _gc: gc,
            max_size_gb: 10,
        })
    }

    /// Create or open AutoCache at home directory
    pub fn in_home_dir() -> Result<Self> {
        let cache_dir = dirs::home_dir()
            .ok_or_else(|| CacheError::CacheDir("Home directory not found".to_string()))?
            .join(".auto")
            .join("cache");

        Self::new(cache_dir)
    }

    /// Get cached artifact by hash key
    ///
    /// Returns the path to the cached blob file.
    /// Updates last_used_at on cache hit.
    pub fn get(&self, hash_key: &str) -> Option<PathBuf> {
        if let Some(blob_path) = self.blobs.get(hash_key) {
            if blob_path.exists() {
                // Update last_used_at asynchronously
                if let Err(e) = self.update_access(hash_key) {
                    log::warn!("Failed to update access time for {}: {}", hash_key, e);
                }
                return Some(blob_path);
            } else {
                // Blob missing from filesystem (corrupted cache)
                log::warn!("Cache blob missing for {}, removing metadata", hash_key);
                let _ = self.remove(hash_key);
            }
        }
        None
    }

    /// Store an artifact in the cache
    ///
    /// # Arguments
    /// * `hash_key` - Cache key (SHA256 hex)
    /// * `source_path` - Path to artifact file to cache
    /// * `metadata` - Artifact metadata
    pub fn put(&self, hash_key: &str, source_path: &Path, metadata: &ArtifactMetadata) -> Result<()> {
        // Store blob in filesystem (atomic write with temp + rename)
        let blob_path = self.blobs.put(hash_key, source_path)?;

        // Store metadata in SQLite
        let conn = self.db.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO artifacts (
                hash_key, blob_path, artifact_type, file_size,
                created_at, last_used_at, access_count,
                source_hash, project_name, module_name
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                hash_key,
                blob_path.to_str().unwrap(),
                metadata.artifact_type as i32,
                metadata.file_size as i64,
                metadata.created_at as i64,
                metadata.last_used_at as i64,
                metadata.access_count as i64,
                metadata.source_hash,
                metadata.project_name,
                metadata.module_name,
            ],
        )?;

        log::info!("Cached artifact: {} (type: {:?}, size: {} bytes)",
                   hash_key, metadata.artifact_type, metadata.file_size);

        Ok(())
    }

    /// Check if artifact exists in cache
    pub fn contains(&self, hash_key: &str) -> bool {
        self.blobs.get(hash_key).is_some_and(|p| p.exists())
    }

    /// Remove artifact from cache
    pub fn remove(&self, hash_key: &str) -> Result<()> {
        // Remove blob file
        self.blobs.remove(hash_key)?;

        // Remove metadata
        let conn = self.db.lock().unwrap();
        conn.execute("DELETE FROM artifacts WHERE hash_key = ?1", rusqlite::params![hash_key])?;

        log::info!("Removed cached artifact: {}", hash_key);
        Ok(())
    }

    /// Update access statistics for cache hit
    fn update_access(&self, hash_key: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp() as u64;

        let conn = self.db.lock().unwrap();
        conn.execute(
            "UPDATE artifacts SET last_used_at = ?1, access_count = access_count + 1 WHERE hash_key = ?2",
            rusqlite::params![now, hash_key],
        )?;

        Ok(())
    }

    /// Get current cache size in bytes
    pub fn current_size(&self) -> u64 {
        let conn = self.db.lock().unwrap();

        conn.query_row(
            "SELECT COALESCE(SUM(file_size), 0) FROM artifacts",
            [],
            |row| row.get(0)
        ).unwrap_or(0)
    }

    /// Get current cache size in GB
    pub fn current_size_gb(&self) -> f64 {
        self.current_size() as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    /// Get statistics about the cache
    pub fn get_statistics(&self) -> CacheStatistics {
        // NOTE: All queries must be done within a single lock scope.
        // Do NOT call self.current_size() or self.calculate_hit_rate() here
        // because they also acquire self.db.lock(), and Rust's Mutex is non-reentrant.
        let conn = self.db.lock().unwrap();

        let count: u64 = conn.query_row(
            "SELECT COUNT(*) FROM artifacts",
            [],
            |row| row.get(0)
        ).unwrap_or(0);

        let size_bytes: u64 = conn.query_row(
            "SELECT COALESCE(SUM(file_size), 0) FROM artifacts",
            [],
            |row| row.get(0)
        ).unwrap_or(0);

        let total_accesses: u64 = conn.query_row(
            "SELECT COALESCE(SUM(access_count), 0) FROM artifacts",
            [],
            |row| row.get(0)
        ).unwrap_or(0);

        let hit_rate = if total_accesses == 0 {
            0.0
        } else {
            let seven_days_ago = chrono::Utc::now().timestamp() as u64 - (7 * 24 * 60 * 60);
            let recent_accesses: u64 = conn.query_row(
                "SELECT COALESCE(SUM(access_count), 0) FROM artifacts WHERE last_used_at > ?1",
                [&seven_days_ago as &dyn rusqlite::ToSql],
                |row| row.get(0)
            ).unwrap_or(0);
            if recent_accesses == 0 {
                0.0
            } else {
                (recent_accesses as f64) / (total_accesses as f64)
            }
        };

        // Lock is dropped here when conn goes out of scope
        CacheStatistics {
            count,
            size_bytes,
            size_gb: size_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
            max_size_gb: self.max_size_gb,
            hit_rate,
        }
    }

    /// Check if garbage collection is needed
    pub fn should_gc(&self) -> bool {
        self.current_size_gb() > self.max_size_gb as f64
    }

    /// Run garbage collection
    pub fn run_gc(&self) -> Result<u64> {
        let target_bytes = (self.current_size() as f64 * 0.8) as u64; // Target 80% of max
        let mut freed = 0;

        let conn = self.db.lock().unwrap();

        // Query oldest artifacts by last_used_at (LRU)
        let mut stmt = conn.prepare(
            "SELECT hash_key, file_size FROM artifacts ORDER BY last_used_at ASC"
        )?;

        let mut rows = stmt.query([])?;
        let mut victims = Vec::new();

        while let Some(row) = rows.next()? {
            let hash_key: String = row.get(0)?;
            let file_size: u64 = row.get(1)?;

            victims.push((hash_key, file_size));

            freed += file_size;
            if freed >= target_bytes {
                break;
            }
        }

        // Delete artifacts in batch
        if !victims.is_empty() {
            // Start transaction for better performance
            let tx = conn.unchecked_transaction()?;

            // Delete metadata
            for (hash_key, _) in &victims {
                tx.execute("DELETE FROM artifacts WHERE hash_key = ?1", [hash_key])?;
            }

            tx.commit()?;

            // Delete blob files
            for (hash_key, _) in &victims {
                self.blobs.remove(hash_key)?;
            }

            log::info!("GC: Freed {} artifacts ({} MB)", victims.len(), freed / (1024 * 1024));
        }

        Ok(freed)
    }

    /// Clear all cached artifacts
    pub fn clear_all(&self) -> Result<()> {
        let conn = self.db.lock().unwrap();

        // Get all artifacts
        let mut stmt = conn.prepare("SELECT hash_key FROM artifacts")?;
        let mut rows = stmt.query([])?;
        let mut hash_keys = Vec::new();

        while let Some(row) = rows.next()? {
            let hash_key: String = row.get(0)?;
            hash_keys.push(hash_key);
        }

        // Delete all blobs
        for hash_key in &hash_keys {
            self.blobs.remove(hash_key)?;
        }

        // Delete all metadata
        conn.execute("DELETE FROM artifacts", [])?;

        log::info!("Cleared all cache artifacts ({} items)", hash_keys.len());

        Ok(())
    }

    /// List all artifacts with optional filtering
    pub fn list_artifacts(&self, type_filter: Option<ArtifactType>, limit: usize) -> Result<Vec<ArtifactMetadata>> {
        let conn = self.db.lock().unwrap();

        let query = if type_filter.is_some() {
            "SELECT * FROM artifacts WHERE artifact_type = ?1 ORDER BY last_used_at DESC LIMIT ?2"
        } else {
            "SELECT * FROM artifacts ORDER BY last_used_at DESC LIMIT ?1"
        };

        let mut stmt = conn.prepare(query)?;
        let mut artifacts = Vec::new();

        if let Some(artifact_type) = type_filter {
            let mut rows = stmt.query(rusqlite::params![artifact_type as i32, limit as i64])?;
            while let Some(row) = rows.next()? {
                artifacts.push(Self::row_to_metadata(row));
            }
        } else {
            let mut rows = stmt.query(rusqlite::params![limit as i64])?;
            while let Some(row) = rows.next()? {
                artifacts.push(Self::row_to_metadata(row));
            }
        }

        Ok(artifacts)
    }

    /// Get artifact metadata by hash key
    pub fn get_metadata(&self, hash_key: &str) -> Option<ArtifactMetadata> {
        let conn = self.db.lock().unwrap();

        let mut stmt = conn.prepare("SELECT * FROM artifacts WHERE hash_key = ?1").ok()?;

        let result = stmt.query_row(rusqlite::params![hash_key], |row| {
            Ok(Self::row_to_metadata(row))
        });

        result.ok()
    }

    /// Verify cache integrity
    pub fn verify_integrity(&self) -> Result<IntegrityReport> {
        let conn = self.db.lock().unwrap();

        // Count metadata entries
        let metadata_count: u64 = conn.query_row(
            "SELECT COUNT(*) FROM artifacts",
            [],
            |row| row.get(0)
        )?;

        // Count blob files
        let blob_count = self.blobs.count().unwrap_or(0) as u64;

        // Check for corrupted entries (metadata exists but blob missing)
        let mut corrupted = 0;
        let mut stmt = conn.prepare("SELECT hash_key, blob_path FROM artifacts")?;
        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            let hash_key: String = row.get(0)?;
            let blob_path: String = row.get(1)?;
            let path = PathBuf::from(blob_path);

            if !path.exists() {
                log::warn!("Corrupted entry: missing blob for {}", hash_key);
                corrupted += 1;
            }
        }

        // Check for orphaned files (blob exists but no metadata)
        let orphaned = blob_count.saturating_sub(metadata_count);

        Ok(IntegrityReport {
            metadata_entries: metadata_count,
            blob_files: blob_count,
            corrupted_entries: corrupted,
            orphaned_files: orphaned,
            is_valid: corrupted == 0 && orphaned == 0,
        })
    }

    /// Convert database row to ArtifactMetadata
    fn row_to_metadata(row: &rusqlite::Row) -> ArtifactMetadata {
        ArtifactMetadata {
            hash_key: row.get(0).unwrap(),
            blob_path: PathBuf::from(row.get::<_, String>(1).unwrap()),
            artifact_type: match row.get::<_, i32>(2).unwrap() {
                0 => ArtifactType::TranspiledC,
                1 => ArtifactType::TranspiledCHeader,
                2 => ArtifactType::TranspiledRust,
                3 => ArtifactType::Bytecode,
                4 => ArtifactType::CompiledObject,
                5 => ArtifactType::RustCrateLibrary,
                6 => ArtifactType::RustCrateSource,
                _ => ArtifactType::TranspiledC,
            },
            file_size: row.get(3).unwrap(),
            created_at: row.get(4).unwrap(),
            last_used_at: row.get(5).unwrap(),
            access_count: row.get(6).unwrap(),
            source_hash: row.get(7).unwrap(),
            project_name: row.get(8).unwrap(),
            module_name: row.get(9).unwrap(),
        }
    }

    /// Initialize SQLite database with schema
    fn initialize_database(db_path: &Path) -> Result<Connection> {
        let conn = Connection::open(db_path)?;

        // Enable WAL mode for better concurrency
        // Note: PRAGMA journal_mode returns a value, so we use query_row
        conn.query_row(
            "PRAGMA journal_mode=WAL",
            [],
            |_| Ok(())
        )?;

        conn.execute("PRAGMA synchronous=NORMAL", [])?;

        // Create artifacts table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS artifacts (
                hash_key TEXT PRIMARY KEY,
                blob_path TEXT NOT NULL,
                artifact_type INTEGER NOT NULL,
                file_size INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                last_used_at INTEGER NOT NULL,
                access_count INTEGER NOT NULL DEFAULT 1,
                source_hash TEXT NOT NULL,
                project_name TEXT NOT NULL,
                module_name TEXT NOT NULL
            )",
            [],
        )?;

        // Create indexes for LRU and source_hash lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_lru ON artifacts(last_used_at)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_source_hash ON artifacts(source_hash)",
            [],
        )?;

        Ok(conn)
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub count: u64,              // Total number of artifacts
    pub size_bytes: u64,         // Total size in bytes
    pub size_gb: f64,            // Total size in GB
    pub max_size_gb: u64,        // Maximum cache size
    pub hit_rate: f64,           // Cache hit rate (0.0 - 1.0)
}

// Re-export storage and GC modules
pub use storage::BlobStore;
pub use gc::GarbageCollector;
pub use fingerprint::{Fingerprint, CompilationTarget, TranspilationLang};
pub use aie_bridge::{AieBridge, HashUtils};
pub use automan::AutoManCache;
pub use trans::{CTranspilationCache, RustTranspilationCache, BytecodeCache};

// Public modules
pub mod storage;
pub mod gc;
pub mod fingerprint;
pub mod aie_bridge;
pub mod automan;
pub mod trans;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let temp_dir = std::env::temp_dir();
        // Use a unique directory for each test to avoid conflicts
        let cache_dir = temp_dir.join(format!("test_cache_{}", std::process::id()));

        // Cleanup first if exists
        let _ = std::fs::remove_dir_all(&cache_dir);

        let cache = AutoCache::new(cache_dir.clone());
        if let Err(e) = &cache {
            eprintln!("Failed to create cache at {}: {:?}", cache_dir.display(), e);
        }
        assert!(cache.is_ok(), "Failed to create cache");

        let cache = cache.unwrap();
        assert_eq!(cache.max_size_gb, 10);

        // Cleanup
        let _ = std::fs::remove_dir_all(&cache_dir);
    }

    #[test]
    fn test_cache_put_get() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir();
        // Use a unique directory for each test
        let cache_dir = temp_dir.join(format!("test_put_get_{}", std::process::id()));

        // Cleanup first if exists
        let _ = std::fs::remove_dir_all(&cache_dir);

        let cache = match AutoCache::new(cache_dir.clone()) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to create cache: {:?}", e);
                panic!("Failed to create cache: {:?}", e);
            }
        };

        // Create a temporary file to cache
        let test_file = temp_dir.join("test_artifact.txt");
        let mut file = std::fs::File::create(&test_file).unwrap();
        file.write_all(b"Hello, Cache!").unwrap();

        let metadata = ArtifactMetadata {
            hash_key: "test123".to_string(),
            blob_path: PathBuf::from("test_path"),
            artifact_type: ArtifactType::TranspiledC,
            file_size: 13,
            created_at: 1234567890,
            last_used_at: 1234567890,
            access_count: 1,
            source_hash: "source_hash_abc".to_string(),
            project_name: "test_project".to_string(),
            module_name: "test_module".to_string(),
        };

        let result = cache.put("test123", &test_file, &metadata);
        if let Err(e) = &result {
            eprintln!("Failed to put artifact: {:?}", e);
        }
        assert!(result.is_ok(), "Failed to put artifact in cache");

        // Check if contains
        assert!(cache.contains("test123"), "Cache should contain artifact");

        // Get artifact
        let retrieved = cache.get("test123");
        assert!(retrieved.is_some(), "Should retrieve cached artifact");

        // Cleanup
        std::fs::remove_file(&test_file).ok();
        let _ = std::fs::remove_dir_all(&cache_dir);
    }
}
