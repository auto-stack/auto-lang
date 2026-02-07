// AutoMan Integration: Connect AutoCache with build pipeline
//
// **Plan 082**: Phase 3 - AutoMan build system integration
//
// Features:
// - Query cache before transpilation
// - Store transpiled artifacts in cache
// - Hard link optimization for cache hits
// - Automatic cache key generation
//
// This module provides the integration layer between AutoCache
// and the AutoMan build system (and transpilers).

use crate::{AutoCache, ArtifactMetadata, ArtifactType, CompilationTarget, AieBridge, IntegrityReport};
use std::path::{Path, PathBuf};

/// AutoMan cache manager
///
/// Provides high-level caching operations for AutoMan builds.
/// Wraps AutoCache with build-specific functionality.
pub struct AutoManCache {
    cache: AutoCache,
    project_name: String,
}

impl AutoManCache {
    /// Create new AutoMan cache manager
    ///
    /// # Arguments
    /// * `cache_dir` - Cache directory path
    /// * `project_name` - Name of the project (for metadata)
    pub fn new(cache_dir: PathBuf, project_name: String) -> Result<Self, CacheError> {
        let cache = AutoCache::new(cache_dir)?;

        Ok(Self {
            cache,
            project_name,
        })
    }

    /// Create AutoMan cache at default home directory
    pub fn in_home_dir(project_name: String) -> Result<Self, CacheError> {
        let cache = AutoCache::in_home_dir()?;

        Ok(Self {
            cache,
            project_name,
        })
    }

    /// Query cache for transpiled artifact
    ///
    /// This is the primary cache lookup function used before
    /// transpilation. If the artifact is found, it returns
    /// the path to the cached blob file.
    ///
    /// # Arguments
    /// * `module_name` - Name of the module (e.g., "std:io")
    /// * `interface_hash` - AIE interface hash (L3 hash)
    /// * `artifact_type` - Type of artifact (C, Rust, Bytecode)
    /// * `target` - Compilation target
    ///
    /// # Returns
    /// - Some(path) if cache hit
    /// - None if cache miss
    ///
    /// # Cache Hit Behavior
    /// - Updates last_used_at timestamp
    /// - Increments access_count
    /// - Returns path to cached artifact
    pub fn query_transpiled(
        &self,
        module_name: &str,
        interface_hash: [u8; 32],
        artifact_type: ArtifactType,
        target: &CompilationTarget,
    ) -> Option<PathBuf> {
        // Generate cache key
        let cache_key = self.generate_cache_key(module_name, interface_hash, artifact_type, target);

        // Query cache
        if let Some(blob_path) = self.cache.get(&cache_key) {
            log::info!("[Cache Hit] {} ({})", module_name, artifact_type);
            return Some(blob_path);
        }

        log::info!("[Cache Miss] {} ({})", module_name, artifact_type);
        None
    }

    /// Store transpiled artifact in cache
    ///
    /// Called after successful transpilation to cache the output.
    ///
    /// # Arguments
    /// * `module_name` - Name of the module
    /// * `interface_hash` - AIE interface hash
    /// * `artifact_path` - Path to the transpiled artifact file
    /// * `artifact_type` - Type of artifact
    /// * `target` - Compilation target
    pub fn store_transpiled(
        &self,
        module_name: &str,
        interface_hash: [u8; 32],
        artifact_path: &Path,
        artifact_type: ArtifactType,
        target: &CompilationTarget,
    ) -> Result<(), CacheError> {
        // Generate cache key
        let cache_key = self.generate_cache_key(module_name, interface_hash, artifact_type, target);

        // Get file size
        let file_size = std::fs::metadata(artifact_path)
            .map_err(|e| CacheError::Io(e))?
            .len();

        // Get current timestamp
        let now = chrono::Utc::now().timestamp() as u64;

        // Encode interface hash as hex string for source_hash
        let source_hash = interface_hash.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        // Create metadata
        let metadata = ArtifactMetadata {
            hash_key: cache_key.clone(),
            blob_path: artifact_path.to_path_buf(),
            artifact_type,
            file_size,
            created_at: now,
            last_used_at: now,
            access_count: 1,
            source_hash,
            project_name: self.project_name.clone(),
            module_name: module_name.to_string(),
        };

        // Store in cache
        self.cache.put(&cache_key, artifact_path, &metadata)?;

        log::info!("[Cache Store] {} ({}, {} bytes)",
                  module_name, artifact_type, file_size);

        Ok(())
    }

    /// Query and retrieve artifact with hard link optimization
    ///
    /// This is a convenience function that combines query and hard link
    /// creation. Useful when you want to create a local copy of the cached
    /// artifact with zero-copy (hard link) if possible.
    ///
    /// # Arguments
    /// * `module_name` - Name of the module
    /// * `interface_hash` - AIE interface hash
    /// * `output_path` - Where to create the hard link
    /// * `artifact_type` - Type of artifact
    /// * `target` - Compilation target
    ///
    /// # Returns
    /// - Ok(true) if cache hit and hard link created
    /// - Ok(false) if cache miss
    /// - Err if cache hit but hard link failed
    pub fn get_or_link(
        &self,
        module_name: &str,
        interface_hash: [u8; 32],
        output_path: &Path,
        artifact_type: ArtifactType,
        target: &CompilationTarget,
    ) -> Result<bool, CacheError> {
        // Query cache
        let cache_key = self.generate_cache_key(module_name, interface_hash, artifact_type, target);

        if let Some(blob_path) = self.cache.get(&cache_key) {
            // Try to create hard link
            match std::fs::hard_link(&blob_path, output_path) {
                Ok(_) => {
                    log::info!("[Cache Link] {} -> {} (hard link)",
                              blob_path.display(), output_path.display());
                    return Ok(true);
                }
                Err(_) => {
                    // Cross-device or other error: fall back to copy
                    log::debug!("[Cache Link] Hard link failed, copying: {} -> {}",
                               blob_path.display(), output_path.display());
                    std::fs::copy(&blob_path, output_path)?;
                    log::info!("[Cache Link] {} -> {} (copy)",
                              blob_path.display(), output_path.display());
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Generate cache key from module information
    ///
    /// Uses AieBridge to create a deterministic cache key.
    ///
    /// # Format
    /// ```text
    /// {module_name}:{target_hash}
    /// ```
    ///
    /// For example:
    /// ```text
    /// std:io:a1b2c3d4e5f6...
    /// ```
    fn generate_cache_key(
        &self,
        module_name: &str,
        interface_hash: [u8; 32],
        artifact_type: ArtifactType,
        target: &CompilationTarget,
    ) -> String {
        // Use AieBridge to create fingerprint
        let fp = AieBridge::fingerprint_from_aie(interface_hash, target, &[]);

        // Sanitize module name: replace colons with underscores
        let safe_module_name = module_name.replace(':', "_");

        // Combine module name, artifact type, and target hash
        // This ensures the same module can have different cached artifacts
        // for different output types (C, Rust, Bytecode)
        format!("{}_{}_{}", safe_module_name, artifact_type, fp.target_hash())
    }

    /// Check if garbage collection is needed
    pub fn should_gc(&self) -> bool {
        self.cache.should_gc()
    }

    /// Run garbage collection
    pub fn run_gc(&self) -> Result<u64, CacheError> {
        self.cache.run_gc()
    }

    /// Get cache statistics
    pub fn get_statistics(&self) -> CacheStatistics {
        self.cache.get_statistics()
    }

    /// Clear all cached artifacts
    pub fn clear_all(&self) -> Result<(), CacheError> {
        self.cache.clear_all()
    }

    /// List all artifacts with optional filtering
    pub fn list_artifacts(&self, type_filter: Option<ArtifactType>, limit: usize) -> Result<Vec<ArtifactMetadata>, CacheError> {
        self.cache.list_artifacts(type_filter, limit)
    }

    /// Get artifact metadata by hash key
    pub fn get_metadata(&self, hash_key: &str) -> Option<ArtifactMetadata> {
        self.cache.get_metadata(hash_key)
    }

    /// Verify cache integrity
    pub fn verify_integrity(&self) -> Result<IntegrityReport, CacheError> {
        self.cache.verify_integrity()
    }
}

/// Cache error type
pub use crate::CacheError;

/// Re-export cache statistics
pub use crate::CacheStatistics;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_automan_cache_creation() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join(format!("test_automan_{}", std::process::id()));

        // Cleanup first
        let _ = std::fs::remove_dir_all(&cache_dir);

        let cache = AutoManCache::new(cache_dir.clone(), "test_project".to_string());
        assert!(cache.is_ok());

        let cache = cache.unwrap();
        assert_eq!(cache.project_name, "test_project");

        // Cleanup
        let _ = std::fs::remove_dir_all(&cache_dir);
    }

    #[test]
    fn test_cache_key_generation() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join(format!("test_key_gen_{}", std::process::id()));

        let cache = AutoManCache::new(cache_dir.clone(), "test_project".to_string()).unwrap();

        let interface_hash = [42u8; 32];
        let target = CompilationTarget::current();

        let key1 = cache.generate_cache_key("std:io", interface_hash, ArtifactType::TranspiledC, &target);
        let key2 = cache.generate_cache_key("std:io", interface_hash, ArtifactType::TranspiledC, &target);

        // Same inputs should produce same key
        assert_eq!(key1, key2);

        // Different module names should produce different keys
        let key3 = cache.generate_cache_key("std:fs", interface_hash, ArtifactType::TranspiledC, &target);
        assert_ne!(key1, key3);

        // Cleanup
        let _ = std::fs::remove_dir_all(&cache_dir);
    }

    #[test]
    fn test_query_store_roundtrip() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join(format!("test_roundtrip_{}", std::process::id()));

        let cache = AutoManCache::new(cache_dir.clone(), "test_project".to_string()).unwrap();
        let interface_hash = [123u8; 32];
        let target = CompilationTarget::current();

        // Create test artifact file
        let artifact_path = temp_dir.join("test_artifact.c");
        let mut file = std::fs::File::create(&artifact_path).unwrap();
        file.write_all(b"// Generated C code\nint main() { return 0; }").unwrap();

        // Store artifact
        let result = cache.store_transpiled(
            "test_module",
            interface_hash,
            &artifact_path,
            ArtifactType::TranspiledC,
            &target,
        );
        assert!(result.is_ok());

        // Query artifact
        let found = cache.query_transpiled(
            "test_module",
            interface_hash,
            ArtifactType::TranspiledC,
            &target,
        );
        assert!(found.is_some());

        // Cleanup
        std::fs::remove_file(&artifact_path).ok();
        let _ = std::fs::remove_dir_all(&cache_dir);
    }

    #[test]
    fn test_get_or_link() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join(format!("test_link_{}", std::process::id()));

        let cache = AutoManCache::new(cache_dir.clone(), "test_project".to_string()).unwrap();
        let interface_hash = [99u8; 32];
        let target = CompilationTarget::current();

        // Create test artifact file
        let artifact_path = temp_dir.join("test_link.c");
        let mut file = std::fs::File::create(&artifact_path).unwrap();
        file.write_all(b"int test() { return 42; }").unwrap();

        // Store artifact
        cache.store_transpiled(
            "link_test",
            interface_hash,
            &artifact_path,
            ArtifactType::TranspiledC,
            &target,
        ).unwrap();

        // Test get_or_link
        let output_path = temp_dir.join("linked.c");
        let result = cache.get_or_link(
            "link_test",
            interface_hash,
            &output_path,
            ArtifactType::TranspiledC,
            &target,
        );

        assert!(result.is_ok());
        assert!(result.unwrap()); // Should be true (cache hit)
        assert!(output_path.exists());

        // Verify content
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert_eq!(content, "int test() { return 42; }");

        // Cleanup
        std::fs::remove_file(&artifact_path).ok();
        std::fs::remove_file(&output_path).ok();
        let _ = std::fs::remove_dir_all(&cache_dir);
    }
}
