// Transpiler Cache Integration: Cache wrappers for transpilers
//
// **Plan 082**: Phase 4 - Transpiler integration
//
// Features:
// - CTranspilationCache: Cache for a2c transpiler
// - RustTranspilationCache: Cache for a2r transpiler
// - BytecodeCache: Cache for AutoVM bytecode compiler
// - Helper functions for computing cache keys from source code
//
// This module provides high-level cache operations for transpilers,
// building on AutoManCache and AieBridge.

use crate::{AutoManCache, ArtifactType, CacheError};
use crate::fingerprint::{CompilationTarget, TranspilationLang, Fingerprint};
use std::path::{Path, PathBuf};

/// C Transpilation Cache
///
/// Provides caching operations for the a2c (Auto to C) transpiler.
pub struct CTranspilationCache {
    inner: AutoManCache,
}

impl CTranspilationCache {
    /// Create new C transpilation cache
    ///
    /// # Arguments
    /// * `project_name` - Name of the project (for metadata)
    pub fn new(project_name: String) -> Result<Self, CacheError> {
        let inner = AutoManCache::in_home_dir(project_name)?;
        Ok(Self { inner })
    }

    /// Create with custom cache directory
    pub fn with_cache_dir(cache_dir: PathBuf, project_name: String) -> Result<Self, CacheError> {
        let inner = AutoManCache::new(cache_dir, project_name)?;
        Ok(Self { inner })
    }

    /// Query cache for transpiled C code
    ///
    /// Checks if the transpilation result (both .c and .h files) is cached.
    ///
    /// # Arguments
    /// * `module_name` - Name of the module (e.g., "std:io")
    /// * `source_code` - AutoLang source code
    ///
    /// # Returns
    /// - Some((c_path, h_path)) if both .c and .h are cached
    /// - Some((c_path, None)) if only .c is cached
    /// - None if cache miss
    ///
    /// # Cache Key Computation
    /// Uses source code hash + C transpilation target to create cache key.
    pub fn query(
        &self,
        module_name: &str,
        source_code: &str,
    ) -> Option<(PathBuf, Option<PathBuf>)> {
        // Compute content hash from source code
        let content_hash = Fingerprint::compute_content_hash(source_code);

        // Create C transpilation target
        let target = CompilationTarget::transpilation(TranspilationLang::C);

        // Query for C artifact
        let c_path = self.inner.query_transpiled(
            module_name,
            content_hash,
            ArtifactType::TranspiledC,
            &target,
        );

        if c_path.is_none() {
            log::info!("[C Cache Miss] {}", module_name);
            return None;
        }

        let c_path = c_path.unwrap();
        log::info!("[C Cache Hit] {}", module_name);

        // Query for header file (may not exist)
        let h_path = self.inner.query_transpiled(
            module_name,
            content_hash,
            ArtifactType::TranspiledCHeader,
            &target,
        );

        Some((c_path, h_path))
    }

    /// Store transpiled C code in cache
    ///
    /// Stores both .c and .h files after successful transpilation.
    ///
    /// # Arguments
    /// * `module_name` - Name of the module
    /// * `source_code` - AutoLang source code
    /// * `c_path` - Path to generated .c file
    /// * `h_path` - Path to generated .h file (optional)
    ///
    /// # Storage Strategy
    /// Stores .c and .h files as separate cache entries with the same hash key
    /// but different artifact types.
    pub fn store(
        &self,
        module_name: &str,
        source_code: &str,
        c_path: &Path,
        h_path: Option<&Path>,
    ) -> Result<(), CacheError> {
        let content_hash = Fingerprint::compute_content_hash(source_code);
        let target = CompilationTarget::transpilation(TranspilationLang::C);

        // Store C file
        self.inner.store_transpiled(
            module_name,
            content_hash,
            c_path,
            ArtifactType::TranspiledC,
            &target,
        )?;

        // Store header file if provided
        if let Some(h_path) = h_path {
            self.inner.store_transpiled(
                module_name,
                content_hash,
                h_path,
                ArtifactType::TranspiledCHeader,
                &target,
            )?;
            log::debug!("Caching header file: {}", h_path.display());
        }

        let c_size = std::fs::metadata(c_path).map(|m| m.len()).unwrap_or(0);
        let h_size = h_path
            .and_then(|p| std::fs::metadata(p).ok())
            .map(|m| m.len())
            .unwrap_or(0);

        log::info!("[C Cache Store] {} (C: {} bytes, H: {} bytes)",
                  module_name, c_size, h_size);

        Ok(())
    }

    /// Query and retrieve with hard link optimization
    ///
    /// Combines cache lookup with hard link creation for zero-copy cache hits.
    ///
    /// # Arguments
    /// * `module_name` - Name of the module
    /// * `source_code` - AutoLang source code
    /// * `output_c_path` - Where to create hard link for .c file
    /// * `output_h_path` - Where to create hard link for .h file (optional)
    ///
    /// # Returns
    /// - Ok(true) if cache hit and hard links created
    /// - Ok(false) if cache miss
    /// - Err if cache hit but hard link failed
    pub fn get_or_link(
        &self,
        module_name: &str,
        source_code: &str,
        output_c_path: &Path,
        output_h_path: Option<&Path>,
    ) -> Result<bool, CacheError> {
        let content_hash = Fingerprint::compute_content_hash(source_code);
        let target = CompilationTarget::transpilation(TranspilationLang::C);

        // Try to get or link .c file
        let c_hit = self.inner.get_or_link(
            module_name,
            content_hash,
            output_c_path,
            ArtifactType::TranspiledC,
            &target,
        )?;

        if c_hit {
            log::info!("[C Cache Link] .c file cached for {}", module_name);

            // If .c was cached, try to link .h file too
            if let Some(h_path) = output_h_path {
                let h_hit = self.inner.get_or_link(
                    module_name,
                    content_hash,
                    h_path,
                    ArtifactType::TranspiledCHeader,
                    &target,
                )?;

                if h_hit {
                    log::info!("[C Cache Link] .h file cached for {}", module_name);
                } else {
                    log::debug!("[C Cache Link] No .h file in cache for {}", module_name);
                }
            }
        }

        Ok(c_hit)
    }
}

/// Rust Transpilation Cache
///
/// Provides caching operations for the a2r (Auto to Rust) transpiler.
pub struct RustTranspilationCache {
    inner: AutoManCache,
}

impl RustTranspilationCache {
    /// Create new Rust transpilation cache
    ///
    /// # Arguments
    /// * `project_name` - Name of the project (for metadata)
    pub fn new(project_name: String) -> Result<Self, CacheError> {
        let inner = AutoManCache::in_home_dir(project_name)?;
        Ok(Self { inner })
    }

    /// Create with custom cache directory
    pub fn with_cache_dir(cache_dir: PathBuf, project_name: String) -> Result<Self, CacheError> {
        let inner = AutoManCache::new(cache_dir, project_name)?;
        Ok(Self { inner })
    }

    /// Query cache for transpiled Rust code
    ///
    /// # Arguments
    /// * `module_name` - Name of the module
    /// * `source_code` - AutoLang source code
    ///
    /// # Returns
    /// - Some(rs_path) if cache hit
    /// - None if cache miss
    pub fn query(
        &self,
        module_name: &str,
        source_code: &str,
    ) -> Option<PathBuf> {
        let content_hash = Fingerprint::compute_content_hash(source_code);
        let target = CompilationTarget::transpilation(TranspilationLang::Rust);

        let result = self.inner.query_transpiled(
            module_name,
            content_hash,
            ArtifactType::TranspiledRust,
            &target,
        );

        if result.is_some() {
            log::info!("[Rust Cache Hit] {}", module_name);
        } else {
            log::info!("[Rust Cache Miss] {}", module_name);
        }

        result
    }

    /// Store transpiled Rust code in cache
    ///
    /// # Arguments
    /// * `module_name` - Name of the module
    /// * `source_code` - AutoLang source code
    /// * `rs_path` - Path to generated .rs file
    pub fn store(
        &self,
        module_name: &str,
        source_code: &str,
        rs_path: &Path,
    ) -> Result<(), CacheError> {
        let content_hash = Fingerprint::compute_content_hash(source_code);
        let target = CompilationTarget::transpilation(TranspilationLang::Rust);

        self.inner.store_transpiled(
            module_name,
            content_hash,
            rs_path,
            ArtifactType::TranspiledRust,
            &target,
        )?;

        log::info!("[Rust Cache Store] {} ({} bytes)",
                  module_name,
                  std::fs::metadata(rs_path).map(|m| m.len()).unwrap_or(0));

        Ok(())
    }

    /// Query and retrieve with hard link optimization
    pub fn get_or_link(
        &self,
        module_name: &str,
        source_code: &str,
        output_path: &Path,
    ) -> Result<bool, CacheError> {
        let content_hash = Fingerprint::compute_content_hash(source_code);
        let target = CompilationTarget::transpilation(TranspilationLang::Rust);

        self.inner.get_or_link(
            module_name,
            content_hash,
            output_path,
            ArtifactType::TranspiledRust,
            &target,
        )
    }
}

/// Bytecode Cache
///
/// Provides caching operations for AutoVM bytecode compiler.
pub struct BytecodeCache {
    inner: AutoManCache,
}

impl BytecodeCache {
    /// Create new bytecode cache
    ///
    /// # Arguments
    /// * `project_name` - Name of the project (for metadata)
    pub fn new(project_name: String) -> Result<Self, CacheError> {
        let inner = AutoManCache::in_home_dir(project_name)?;
        Ok(Self { inner })
    }

    /// Create with custom cache directory
    pub fn with_cache_dir(cache_dir: PathBuf, project_name: String) -> Result<Self, CacheError> {
        let inner = AutoManCache::new(cache_dir, project_name)?;
        Ok(Self { inner })
    }

    /// Query cache for compiled bytecode
    ///
    /// # Arguments
    /// * `module_name` - Name of the module
    /// * `source_code` - AutoLang source code (or AIE interface hash if available)
    ///
    /// # Returns
    /// - Some(bc_path) if cache hit
    /// - None if cache miss
    pub fn query(
        &self,
        module_name: &str,
        source_code: &str,
    ) -> Option<PathBuf> {
        let content_hash = Fingerprint::compute_content_hash(source_code);
        let target = CompilationTarget::transpilation(TranspilationLang::Bytecode);

        let result = self.inner.query_transpiled(
            module_name,
            content_hash,
            ArtifactType::Bytecode,
            &target,
        );

        if result.is_some() {
            log::info!("[Bytecode Cache Hit] {}", module_name);
        } else {
            log::info!("[Bytecode Cache Miss] {}", module_name);
        }

        result
    }

    /// Store compiled bytecode in cache
    ///
    /// # Arguments
    /// * `module_name` - Name of the module
    /// * `source_code` - AutoLang source code
    /// * `bc_path` - Path to compiled .bc file
    pub fn store(
        &self,
        module_name: &str,
        source_code: &str,
        bc_path: &Path,
    ) -> Result<(), CacheError> {
        let content_hash = Fingerprint::compute_content_hash(source_code);
        let target = CompilationTarget::transpilation(TranspilationLang::Bytecode);

        self.inner.store_transpiled(
            module_name,
            content_hash,
            bc_path,
            ArtifactType::Bytecode,
            &target,
        )?;

        log::info!("[Bytecode Cache Store] {} ({} bytes)",
                  module_name,
                  std::fs::metadata(bc_path).map(|m| m.len()).unwrap_or(0));

        Ok(())
    }

    /// Query and retrieve with hard link optimization
    pub fn get_or_link(
        &self,
        module_name: &str,
        source_code: &str,
        output_path: &Path,
    ) -> Result<bool, CacheError> {
        let content_hash = Fingerprint::compute_content_hash(source_code);
        let target = CompilationTarget::transpilation(TranspilationLang::Bytecode);

        self.inner.get_or_link(
            module_name,
            content_hash,
            output_path,
            ArtifactType::Bytecode,
            &target,
        )
    }

    /// Query cache using AIE interface hash (for AIE integration)
    ///
    /// This is the preferred method when AIE Database is available,
    /// as it reuses the pre-computed interface hash.
    ///
    /// # Arguments
    /// * `module_name` - Name of the module
    /// * `interface_hash` - AIE interface hash (L3 hash)
    ///
    /// # Returns
    /// - Some(bc_path) if cache hit
    /// - None if cache miss
    pub fn query_with_aie_hash(
        &self,
        module_name: &str,
        interface_hash: [u8; 32],
    ) -> Option<PathBuf> {
        let target = CompilationTarget::transpilation(TranspilationLang::Bytecode);

        let result = self.inner.query_transpiled(
            module_name,
            interface_hash,
            ArtifactType::Bytecode,
            &target,
        );

        if result.is_some() {
            log::info!("[Bytecode Cache Hit (AIE)] {}", module_name);
        } else {
            log::info!("[Bytecode Cache Miss (AIE)] {}", module_name);
        }

        result
    }

    /// Store bytecode using AIE interface hash
    ///
    /// # Arguments
    /// * `module_name` - Name of the module
    /// * `interface_hash` - AIE interface hash (L3 hash)
    /// * `bc_path` - Path to compiled .bc file
    pub fn store_with_aie_hash(
        &self,
        module_name: &str,
        interface_hash: [u8; 32],
        bc_path: &Path,
    ) -> Result<(), CacheError> {
        let target = CompilationTarget::transpilation(TranspilationLang::Bytecode);

        self.inner.store_transpiled(
            module_name,
            interface_hash,
            bc_path,
            ArtifactType::Bytecode,
            &target,
        )?;

        log::info!("[Bytecode Cache Store (AIE)] {} ({} bytes)",
                  module_name,
                  std::fs::metadata(bc_path).map(|m| m.len()).unwrap_or(0));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_c_transpilation_cache_creation() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join(format!("test_c_cache_{}", std::process::id()));

        let _ = std::fs::remove_dir_all(&cache_dir);

        let cache = CTranspilationCache::with_cache_dir(
            cache_dir.clone(),
            "test_project".to_string(),
        );
        assert!(cache.is_ok());

        let _ = std::fs::remove_dir_all(&cache_dir);
    }

    #[test]
    fn test_rust_transpilation_cache_creation() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join(format!("test_rust_cache_{}", std::process::id()));

        let _ = std::fs::remove_dir_all(&cache_dir);

        let cache = RustTranspilationCache::with_cache_dir(
            cache_dir.clone(),
            "test_project".to_string(),
        );
        assert!(cache.is_ok());

        let _ = std::fs::remove_dir_all(&cache_dir);
    }

    #[test]
    fn test_bytecode_cache_creation() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join(format!("test_bc_cache_{}", std::process::id()));

        let _ = std::fs::remove_dir_all(&cache_dir);

        let cache = BytecodeCache::with_cache_dir(
            cache_dir.clone(),
            "test_project".to_string(),
        );
        assert!(cache.is_ok());

        let _ = std::fs::remove_dir_all(&cache_dir);
    }

    #[test]
    fn test_c_cache_roundtrip() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join(format!("test_c_roundtrip_{}", std::process::id()));

        let cache = CTranspilationCache::with_cache_dir(
            cache_dir.clone(),
            "test_project".to_string(),
        ).unwrap();

        let source_code = r#"
fn add(a int, b int) int {
    a + b
}
"#;

        let module_name = "test:add";

        // Create test .c and .h files
        let c_path = temp_dir.join("test_add.c");
        let h_path = temp_dir.join("test_add.h");

        let mut c_file = std::fs::File::create(&c_path).unwrap();
        c_file.write_all(b"int add(int a, int b) { return a + b; }").unwrap();

        let mut h_file = std::fs::File::create(&h_path).unwrap();
        h_file.write_all(b"int add(int a, int b);").unwrap();

        // Store in cache
        let result = cache.store(module_name, source_code, &c_path, Some(&h_path));
        assert!(result.is_ok());

        // Query cache
        let found = cache.query(module_name, source_code);
        assert!(found.is_some());

        let (found_c, found_h) = found.unwrap();
        assert!(found_c.exists());
        assert!(found_h.is_some()); // Header should be cached
        assert!(found_h.unwrap().exists());

        // Cleanup
        std::fs::remove_file(&c_path).ok();
        std::fs::remove_file(&h_path).ok();
        let _ = std::fs::remove_dir_all(&cache_dir);
    }

    #[test]
    fn test_rust_cache_roundtrip() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join(format!("test_rust_roundtrip_{}", std::process::id()));

        let cache = RustTranspilationCache::with_cache_dir(
            cache_dir.clone(),
            "test_project".to_string(),
        ).unwrap();

        let source_code = "fn add(a int, b int) int { a + b }";
        let module_name = "test:add";

        // Create test .rs file
        let rs_path = temp_dir.join("test_add.rs");
        let mut file = std::fs::File::create(&rs_path).unwrap();
        file.write_all(b"pub fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();

        // Store in cache
        let result = cache.store(module_name, source_code, &rs_path);
        assert!(result.is_ok());

        // Query cache
        let found = cache.query(module_name, source_code);
        assert!(found.is_some());
        assert!(found.unwrap().exists());

        // Cleanup
        std::fs::remove_file(&rs_path).ok();
        let _ = std::fs::remove_dir_all(&cache_dir);
    }

    #[test]
    fn test_bytecode_cache_roundtrip() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join(format!("test_bc_roundtrip_{}", std::process::id()));

        let cache = BytecodeCache::with_cache_dir(
            cache_dir.clone(),
            "test_project".to_string(),
        ).unwrap();

        let source_code = "fn main() { 42 }";
        let module_name = "test:main";

        // Create test .bc file
        let bc_path = temp_dir.join("test_main.bc");
        let mut file = std::fs::File::create(&bc_path).unwrap();
        file.write_all(b"[bytecode data]").unwrap();

        // Store in cache
        let result = cache.store(module_name, source_code, &bc_path);
        assert!(result.is_ok());

        // Query cache
        let found = cache.query(module_name, source_code);
        assert!(found.is_some());
        assert!(found.unwrap().exists());

        // Cleanup
        std::fs::remove_file(&bc_path).ok();
        let _ = std::fs::remove_dir_all(&cache_dir);
    }

    #[test]
    fn test_bytecode_cache_with_aie_hash() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join(format!("test_bc_aie_{}", std::process::id()));

        let cache = BytecodeCache::with_cache_dir(
            cache_dir.clone(),
            "test_project".to_string(),
        ).unwrap();

        let module_name = "test:module";
        let interface_hash = [42u8; 32];

        // Create test .bc file
        let bc_path = temp_dir.join("test.bc");
        let mut file = std::fs::File::create(&bc_path).unwrap();
        file.write_all(b"[compiled bytecode]").unwrap();

        // Store with AIE hash
        let result = cache.store_with_aie_hash(module_name, interface_hash, &bc_path);
        assert!(result.is_ok());

        // Query with AIE hash
        let found = cache.query_with_aie_hash(module_name, interface_hash);
        assert!(found.is_some());
        assert!(found.unwrap().exists());

        // Cleanup
        std::fs::remove_file(&bc_path).ok();
        let _ = std::fs::remove_dir_all(&cache_dir);
    }

    #[test]
    fn test_c_cache_get_or_link() {
        let temp_dir = std::env::temp_dir();
        let cache_dir = temp_dir.join(format!("test_c_link_{}", std::process::id()));

        let cache = CTranspilationCache::with_cache_dir(
            cache_dir.clone(),
            "test_project".to_string(),
        ).unwrap();

        let source_code = "fn test() int { 42 }";
        let module_name = "test:link";

        // Create test files
        let c_path = temp_dir.join("test_link.c");
        let h_path = temp_dir.join("test_link.h");

        let mut c_file = std::fs::File::create(&c_path).unwrap();
        c_file.write_all(b"int test() { return 42; }").unwrap();

        let mut h_file = std::fs::File::create(&h_path).unwrap();
        h_file.write_all(b"int test();").unwrap();

        // Store in cache
        cache.store(module_name, source_code, &c_path, Some(&h_path)).unwrap();

        // Test get_or_link
        let output_c = temp_dir.join("output.c");
        let output_h = temp_dir.join("output.h");

        let result = cache.get_or_link(module_name, source_code, &output_c, Some(&output_h));
        assert!(result.is_ok());
        assert!(result.unwrap()); // Should be true (cache hit)
        assert!(output_c.exists());
        assert!(output_h.exists());

        // Verify content
        let c_content = std::fs::read_to_string(&output_c).unwrap();
        assert_eq!(c_content, "int test() { return 42; }");

        let h_content = std::fs::read_to_string(&output_h).unwrap();
        assert_eq!(h_content, "int test();");

        // Cleanup
        std::fs::remove_file(&c_path).ok();
        std::fs::remove_file(&h_path).ok();
        std::fs::remove_file(&output_c).ok();
        std::fs::remove_file(&output_h).ok();
        let _ = std::fs::remove_dir_all(&cache_dir);
    }
}
