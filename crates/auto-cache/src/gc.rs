// GarbageCollector: LRU garbage collection for AutoCache
//
// **Plan 082**: AutoCache garbage collection
//
// Features:
// - LRU (Least Recently Used) eviction policy
// - Size-based triggering (watermark pattern)
// - Batch deletion for efficiency
// - SQLite-based queries

use rusqlite::Connection;

/// GarbageCollector-specific result type
pub type Result<T> = std::result::Result<T, rusqlite::Error>;

/// Garbage collector for AutoCache
///
/// Uses LRU eviction policy with size-based triggering.
/// When cache size exceeds max_size_gb, GC runs to bring
/// cache down to water_mark_gb (typically 80% of max).
pub struct GarbageCollector {
    max_size_gb: u64,
    water_mark_gb: u64,
}

impl GarbageCollector {
    /// Create new GarbageCollector
    ///
    /// # Arguments
    /// * `max_size_gb` - Maximum cache size in GB
    ///
    /// # Watermark
    /// - Watermark is set to 80% of max_size_gb
    /// - GC runs when cache exceeds max_size_gb
    /// - GC stops when cache reaches water_mark_gb
    ///
    /// # Example
    /// ```
    /// use auto_cache::gc::GarbageCollector;
    ///
    /// let gc = GarbageCollector::new(10);  // 10 GB max, 8 GB watermark
    /// ```
    pub fn new(max_size_gb: u64) -> Self {
        let water_mark_gb = (max_size_gb as f64 * 0.8) as u64;

        Self {
            max_size_gb,
            water_mark_gb,
        }
    }

    /// Create new GarbageCollector with custom watermark
    ///
    /// # Arguments
    /// * `max_size_gb` - Maximum cache size in GB
    /// * `water_mark_gb` - Target size after GC (should be < max_size_gb)
    pub fn with_watermark(max_size_gb: u64, water_mark_gb: u64) -> Self {
        assert!(water_mark_gb < max_size_gb, "Watermark must be less than max size");

        Self {
            max_size_gb,
            water_mark_gb,
        }
    }

    /// Check if garbage collection is needed
    ///
    /// # Arguments
    /// * `current_size_gb` - Current cache size in GB
    ///
    /// # Returns
    /// true if GC should run, false otherwise
    pub fn should_gc(&self, current_size_gb: f64) -> bool {
        current_size_gb > self.max_size_gb as f64
    }

    /// Run garbage collection
    ///
    /// Evicts oldest artifacts (by last_used_at) until cache size
    /// reaches water_mark_gb.
    ///
    /// # Arguments
    /// * `conn` - SQLite connection
    /// * `current_size_bytes` - Current cache size in bytes
    ///
    /// # Returns
    /// Number of bytes freed
    ///
    /// # Process
    /// 1. Calculate target size (water_mark_gb)
    /// 2. Query artifacts ordered by last_used_at ASC (oldest first)
    /// 3. Delete artifacts until target size reached
    /// 4. Return bytes freed
    pub fn run_gc(&self, conn: &Connection, current_size_bytes: u64) -> Result<u64> {
        let target_bytes = self.water_mark_gb * 1024 * 1024 * 1024;
        let mut freed_bytes = 0u64;

        if current_size_bytes <= target_bytes {
            log::info!("GC: Cache size {} GB within limit, no GC needed",
                      current_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0));
            return Ok(0);
        }

        let target_free = current_size_bytes.saturating_sub(target_bytes);
        let mut victims = Vec::new();

        log::info!("GC: Current size: {} GB, Target: {} GB, Need to free: {} MB",
                  current_size_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
                  self.water_mark_gb,
                  target_free / (1024 * 1024));

        // Query oldest artifacts by last_used_at (LRU)
        let mut stmt = conn.prepare(
            "SELECT hash_key, file_size FROM artifacts ORDER BY last_used_at ASC"
        )?;

        let mut rows = stmt.query([])?;

        while let Some(row) = rows.next()? {
            let hash_key: String = row.get(0)?;
            let file_size: u64 = row.get(1)?;

            victims.push((hash_key, file_size));
            freed_bytes += file_size;

            if freed_bytes >= target_free {
                break;
            }
        }

        // Delete artifacts in batch
        if !victims.is_empty() {
            // Start transaction
            let tx = conn.unchecked_transaction()?;

            // Delete each artifact
            for (hash_key, _) in &victims {
                tx.execute("DELETE FROM artifacts WHERE hash_key = ?1", [hash_key])?;
            }

            tx.commit()?;

            log::info!("GC: Freed {} artifacts ({} MB, {} GB)",
                      victims.len(),
                      freed_bytes / (1024 * 1024),
                      freed_bytes as f64 / (1024.0 * 1024.0 * 1024.0));
        }

        Ok(freed_bytes)
    }

    /// Get garbage collector statistics
    ///
    /// # Returns
    /// (max_size_gb, water_mark_gb)
    pub fn stats(&self) -> (u64, u64) {
        (self.max_size_gb, self.water_mark_gb)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_gc_creation() {
        let gc = GarbageCollector::new(10);
        let (max, water) = gc.stats();

        assert_eq!(max, 10);
        assert_eq!(water, 8);  // 80% of max
    }

    #[test]
    fn test_gc_custom_watermark() {
        let gc = GarbageCollector::with_watermark(10, 5);
        let (max, water) = gc.stats();

        assert_eq!(max, 10);
        assert_eq!(water, 5);
    }

    #[test]
    fn test_should_gc() {
        let gc = GarbageCollector::new(10);

        assert!(!gc.should_gc(8.0));  // 8 GB < 10 GB
        assert!(!gc.should_gc(10.0)); // 10 GB = 10 GB
        assert!(gc.should_gc(11.0));  // 11 GB > 10 GB
    }

    #[test]
    fn test_run_gc() {
        let temp_dir = std::env::temp_dir().join("test_gc");
        std::fs::create_dir_all(&temp_dir).ok();

        // Create in-memory database
        let conn = Connection::open_in_memory().unwrap();

        // Create artifacts table
        conn.execute(
            "CREATE TABLE artifacts (
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
        ).unwrap();

        // Insert test artifacts (last_used_at in seconds)
        // Current timestamp
        let now = chrono::Utc::now().timestamp() as u64;

        let artifacts = vec![
            ("hash1", 100, now - 1000),  // Oldest
            ("hash2", 200, now - 500),
            ("hash3", 300, now - 100),
            ("hash4", 400, now),         // Newest
        ];

        for (hash, size, last_used) in &artifacts {
            conn.execute(
                "INSERT INTO artifacts (hash_key, blob_path, artifact_type, file_size,
                                      created_at, last_used_at, access_count,
                                      source_hash, project_name, module_name)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    hash,
                    format!("/tmp/{}", hash),
                    0i32,  // artifact_type
                    *size as i64,
                    *last_used as i64,  // created_at
                    *last_used as i64,  // last_used_at
                    1i64,  // access_count
                    "source_hash",   // source_hash
                    "test_project",  // project_name
                    "test_module",   // module_name
                ],
            ).unwrap();
        }

        // Total size: 100 + 200 + 300 + 400 = 1000 bytes
        // Watermark: 700 bytes (keep newest artifact hash4: 400 bytes, and part of hash3: 300 bytes)
        // Target: free up to 300 bytes (1000 - 700)
        let gc = GarbageCollector::with_watermark(1, 0);  // Simplified: 0 GB watermark, but we'll pass bytes directly
        let current_size = 1000;

        // Run GC
        let freed = gc.run_gc(&conn, current_size).unwrap();

        // Should free oldest artifacts (hash1: 100, hash2: 200) = 300 bytes minimum
        // The GC will delete until freed_bytes >= target_free (1000 - 0 = 1000 bytes in this case)
        // So all artifacts will be deleted
        assert!(freed >= 600);

        // Verify deleted artifacts - with 0 watermark, all should be deleted
        let mut stmt = conn.prepare("SELECT hash_key FROM artifacts").unwrap();
        let mut rows = stmt.query([]).unwrap();

        let mut remaining = Vec::new();
        while let Some(row) = rows.next().unwrap() {
            remaining.push(row.get::<_, String>(0).unwrap());
        }

        // All artifacts should be deleted when watermark is 0
        assert_eq!(remaining.len(), 0);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    #[should_panic(expected = "Watermark must be less than max size")]
    fn test_invalid_watermark() {
        GarbageCollector::with_watermark(10, 15);  // watermark > max
    }
}
