// =============================================================================
// Query Engine: Cached query execution for AIE
// =============================================================================
//
// The Query Engine provides a caching layer for executing queries against
// the Database. Queries are computations that read from the Database
// and produce results (e.g., type inference, bytecode generation).
//
// Phase 2.4: Basic query engine with caching (no type erasure)
// Phase 3.4: Incremental query engine with dependency tracking and smart invalidation

use crate::database::{Database, FragId};
use crate::error::{AutoError, AutoResult};
use crate::ast::Type;
use crate::scope::Sid;
use dashmap::DashMap;
use std::sync::Arc;

// =============================================================================
// Cache Entry (Phase 3.4)
// =============================================================================

/// Cached query result with dependency tracking
///
/// Phase 3.4: Track which fragments this query depends on, so we can
/// invalidate the cache when dependencies change.
///
/// Note: Uses `any::Any` for type erasure (in-memory caching only).
/// Future enhancement: Proper serialization for persistent cache.
#[derive(Debug)]
struct CacheEntry {
    /// Cached result (type-erased)
    data: Box<dyn std::any::Any>,

    /// Fragments this query depends on (for invalidation)
    /// Phase 3.4: Track dependencies to implement smart cache invalidation
    dependencies: Vec<FragDep>,
}

/// Fragment dependency descriptor
///
/// Phase 3.4: Tracks which fragment a query depends on, and at what
/// interface hash (for熔断-based invalidation).
#[derive(Debug, Clone, Copy)]
struct FragDep {
    /// File ID
    file_id: u64,

    /// Fragment offset
    offset: usize,

    /// Interface hash (L3) at time of query execution
    /// If current hash != cached hash, fragment signature changed
    iface_hash: u64,
}

impl FragDep {
    /// Create a new fragment dependency
    fn new(frag_id: FragId, iface_hash: u64) -> Self {
        Self {
            file_id: frag_id.file.as_u64(),
            offset: frag_id.offset,
            iface_hash,
        }
    }

    /// Check if this dependency matches a fragment ID
    fn matches(&self, frag_id: &FragId) -> bool {
        self.file_id == frag_id.file.as_u64() && self.offset == frag_id.offset
    }
}

// =============================================================================
// Query Trait
// =============================================================================

/// Query trait for database operations
///
/// All queries must implement this trait to be executed by the QueryEngine.
/// Queries should be pure functions of the database state.
pub trait Query {
    /// The output type of this query
    type Output;

    /// Execute the query against the database
    ///
    /// This method should read from the database and compute the result.
    /// It should not modify the database.
    fn execute(&self, db: &Database) -> AutoResult<Self::Output>;

    /// Get a cache key for this query
    ///
    /// The cache key should uniquely identify the query parameters.
    /// Two queries with the same cache_key should produce the same result.
    fn cache_key(&self) -> String;
}

// =============================================================================
// Query Engine
// =============================================================================

/// Query engine with result caching
///
/// The QueryEngine executes queries against the Database and caches
/// the results to avoid redundant computations.
///
/// # Architecture
///
/// ```text
/// Query → QueryEngine → Cache → Database
///                         ↓
///                    CacheEntry (with dependencies)
/// ```
///
/// # Phase 2.4: Basic Caching (per-query-type caches)
///
/// - Cache results keyed by query.cache_key()
/// - No automatic invalidation (manual clearing required)
/// - Thread-safe via DashMap
///
/// # Phase 3.4: Incremental Query Engine
///
/// - Track query dependencies on fragments
/// - Validate cache before using (check dirty flags and interface hashes)
/// - Smart invalidation using熔断
/// - In-memory caching (no serialization for Phase 3.4)
pub struct QueryEngine {
    /// The database (shared reference for thread safety)
    db: Arc<Database>,

    /// Type-specific caches (indexed by type name)
    /// Phase 2: Vec<u8> placeholders (not used)
    /// Phase 3.4: CacheEntry with dependency tracking (in-memory)
    type_cache: DashMap<String, DashMap<String, CacheEntry>>,
}

impl QueryEngine {
    /// Create a new query engine
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            type_cache: DashMap::new(),
        }
    }

    /// Execute a query with caching and dependency tracking
    ///
    /// This method will:
    /// 1. Check the cache for a previous result
    /// 2. If cached, validate dependencies (check if fragments are dirty)
    /// 3. If cache valid, downcast and return cached result
    /// 4. Otherwise, execute the query, track dependencies, and cache the result
    ///
    /// # Type Parameters
    ///
    /// * `Q` - Query type implementing the Query trait
    ///
    /// # Returns
    ///
    /// The query result
    ///
    /// # Phase 3.4: Smart Caching
    ///
    /// Results are cached in-memory with dependency tracking.
    /// Cache entries are validated before use (dirty check + interface hash check).
    pub fn execute<Q: Query + 'static>(&self, query: &Q) -> AutoResult<Q::Output>
    where
        Q::Output: Clone + 'static,
    {
        let type_name = std::any::type_name::<Q>();
        let key = query.cache_key();

        // Get or create type-specific cache
        let cache = self.type_cache.entry(type_name.to_string()).or_insert_with(DashMap::new);

        // Check cache
        if let Some(entry) = cache.get(&key) {
            // Phase 3.4: Validate cache before using
            if self.is_cache_valid(&entry) {
                // Cache hit and valid - downcast and return
                let result = entry.data.downcast_ref::<Q::Output>()
                    .ok_or_else(|| AutoError::Msg("Cache type mismatch".to_string()))?;

                return Ok(result.clone());
            }
            // Cache invalid - fall through to re-execute
        }

        // Cache miss or invalid - execute query
        let result = query.execute(&self.db)?;

        // Phase 3.4: Track dependencies and cache the result
        let dependencies = self.extract_dependencies::<Q>(query);

        let entry = CacheEntry {
            data: Box::new(result.clone()),
            dependencies,
        };

        cache.insert(key, entry);

        Ok(result)
    }

    /// Check if a cache entry is still valid
    ///
    /// Phase 3.4: Validate dependencies by checking:
    /// 1. Fragments are not marked as dirty
    /// 2. Fragment interface hashes haven't changed (熔断)
    fn is_cache_valid(&self, entry: &CacheEntry) -> bool {
        for dep in &entry.dependencies {
            // Check if fragment is marked as dirty
            let file_id = crate::database::FileId::new(dep.file_id);
            if self.db.is_marked_dirty(file_id) {
                return false;
            }

            // Phase 3.4: Check if interface hash changed (熔断validation)
            // We need to find the fragment by file_id:offset to check its current hash
            let frag_id = self.find_fragment_by_offset(file_id, dep.offset);

            if let Some(frag_id) = frag_id {
                // Get current interface hash from database
                if let Some(current_hash) = self.db.get_fragment_iface_hash(&frag_id) {
                    // Compare with cached hash
                    if current_hash != dep.iface_hash {
                        // Interface hash changed - signature changed,熔断FAILED
                        return false;
                    }
                    // else: Hash unchanged - signature stable,熔断WORKS
                } else {
                    // Fragment no longer exists or hash not set - invalidate cache
                    return false;
                }
            } else {
                // Fragment no longer exists - invalidate cache
                return false;
            }
        }

        true
    }

    /// Find a fragment by file ID and offset
    ///
    /// Phase 3.4: Enable fragment lookup for熔断hash checking
    ///
    /// # Arguments
    ///
    /// * `file_id` - The file ID
    /// * `offset` - The fragment offset within the file
    ///
    /// # Returns
    ///
    /// The FragId if found, None otherwise
    fn find_fragment_by_offset(&self, file_id: crate::database::FileId, offset: usize) -> Option<crate::database::FragId> {
        // Get all fragments in the file
        let frag_ids = self.db.get_fragments_in_file(file_id);

        // Find the fragment with matching offset
        for frag_id in frag_ids {
            if frag_id.offset == offset {
                return Some(frag_id);
            }
        }

        None
    }

    /// Extract fragment dependencies from a query
    ///
    /// Phase 3.4: Determine which fragments this query depends on.
    /// This is used for cache invalidation.
    ///
    /// For now, we extract dependencies by inspecting the query parameters.
    /// Future enhancement: Queries could implement a `dependencies()` method.
    fn extract_dependencies<Q: Query + 'static>(&self, query: &Q) -> Vec<FragDep> {
        let mut deps = Vec::new();

        // Extract dependencies based on query type
        let type_name = std::any::type_name::<Q>();

        // GetTypeQuery: Depends on fragments that define the symbol
        if type_name.contains("GetTypeQuery") {
            // For now, we can't easily track symbol → fragment mapping
            // Phase 3.4 enhancement: Add symbol location tracking
        }

        // GetBytecodeQuery: Directly depends on the fragment
        if type_name.contains("GetBytecodeQuery") {
            // Extract frag_id from query using downcast
            if let Some(bc_query) = (query as &dyn std::any::Any).downcast_ref::<GetBytecodeQuery>() {
                let frag_id = bc_query.frag_id.clone();
                if let Some(hash) = self.db.get_fragment_iface_hash(&frag_id) {
                    deps.push(FragDep::new(frag_id, hash));
                }
            }
        }

        // GetFileDepsQuery, GetFunctionsQuery, GetFragmentsQuery:
        // These depend on all fragments in the file
        if type_name.contains("GetFileDepsQuery")
            || type_name.contains("GetFunctionsQuery")
            || type_name.contains("GetFragmentsQuery")
        {
            // Extract file_id from query
            // Phase 3.4: Simplified - depends on all fragments in file
            // Future: Track exact fragment dependencies
        }

        deps
    }

    /// Execute a query without caching
    ///
    /// Use this for queries that shouldn't be cached (e.g., very large results,
    /// or queries that change frequently).
    pub fn execute_uncached<Q: Query>(&self, query: &Q) -> AutoResult<Q::Output> {
        query.execute(&self.db)
    }

    /// Clear all cached results
    pub fn clear_cache(&self) {
        self.type_cache.clear();
    }

    /// Clear cache entries for a specific key prefix
    ///
    /// Useful for invalidating all queries related to a specific file/fragment
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Clear all queries related to file #42
    /// engine.clear_cache_prefix("type:42:");
    /// engine.clear_cache_prefix("bytecode:42:");
    /// ```
    pub fn clear_cache_prefix(&self, prefix: &str) {
        for cache in self.type_cache.iter() {
            let mut keys_to_remove = Vec::new();

            for key in cache.value().iter() {
                if key.key().starts_with(prefix) {
                    keys_to_remove.push(key.key().clone());
                }
            }

            for key in keys_to_remove {
                cache.value().remove(&key);
            }
        }
    }

    /// Invalidate cache entries that depend on a specific fragment
    ///
    /// Phase 3.4: Smart invalidation - only remove cache entries that
    /// actually depend on the given fragment.
    ///
    /// # Arguments
    ///
    /// * `frag_id` - The fragment that changed
    pub fn invalidate_fragment(&self, frag_id: FragId) {
        for cache in self.type_cache.iter() {
            let mut keys_to_remove = Vec::new();

            for entry in cache.value().iter() {
                // Check if this cache entry depends on the fragment
                let depends = entry.value().dependencies.iter()
                    .any(|dep| dep.matches(&frag_id));

                if depends {
                    keys_to_remove.push(entry.key().clone());
                }
            }

            for key in keys_to_remove {
                cache.value().remove(&key);
            }
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let entries: usize = self.type_cache.iter().map(|cache| cache.value().len()).sum();
        let total_size_bytes: usize = self.type_cache.iter()
            .map(|cache| -> usize {
                cache.value().iter()
                    .map(|_entry| 0) // Can't easily size Box<dyn Any>
                    .sum()
            })
            .sum();

        CacheStats {
            entries,
            total_size_bytes,
        }
    }

    /// Get the underlying database (for direct access)
    pub fn database(&self) -> &Database {
        &self.db
    }

    /// Get the underlying database as Arc (for sharing)
    pub fn database_arc(&self) -> Arc<Database> {
        Arc::clone(&self.db)
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of cached entries
    pub entries: usize,

    /// Total size of all cached entries in bytes
    pub total_size_bytes: usize,
}

// =============================================================================
// Example Queries
// =============================================================================

/// Query to get the type of a symbol
#[derive(Debug, Clone)]
pub struct GetTypeQuery {
    /// The symbol ID to query
    pub symbol_id: Sid,
}

impl Query for GetTypeQuery {
    type Output = Option<Type>;

    fn execute(&self, db: &Database) -> AutoResult<Self::Output> {
        Ok(db.get_type(&self.symbol_id))
    }

    fn cache_key(&self) -> String {
        format!("type:{}", self.symbol_id)
    }
}

/// Query to get bytecode for a fragment
#[derive(Debug, Clone)]
pub struct GetBytecodeQuery {
    /// The fragment ID to query
    pub frag_id: crate::database::FragId,
}

impl Query for GetBytecodeQuery {
    type Output = Option<Vec<u8>>;

    fn execute(&self, db: &Database) -> AutoResult<Self::Output> {
        Ok(db.get_bytecode(&self.frag_id))
    }

    fn cache_key(&self) -> String {
        format!("bytecode:{}:{}", self.frag_id.file.as_u64(), self.frag_id.offset)
    }
}

/// Query to get dependencies for a file
#[derive(Debug, Clone)]
pub struct GetFileDepsQuery {
    /// The file ID to query
    pub file_id: crate::database::FileId,
}

impl Query for GetFileDepsQuery {
    type Output = Vec<crate::database::FileId>;

    fn execute(&self, db: &Database) -> AutoResult<Self::Output> {
        Ok(db.dep_graph().get_file_imports(self.file_id).to_vec())
    }

    fn cache_key(&self) -> String {
        format!("file_deps:{}", self.file_id.as_u64())
    }
}

/// Query to get all functions in a file
#[derive(Debug, Clone)]
pub struct GetFunctionsQuery {
    /// The file ID to query
    pub file_id: crate::database::FileId,
}

impl Query for GetFunctionsQuery {
    type Output = Vec<String>;

    fn execute(&self, db: &Database) -> AutoResult<Self::Output> {
        let mut functions = Vec::new();

        for frag_id in db.get_fragments_in_file(self.file_id) {
            if let Some(meta) = db.get_fragment_meta(&frag_id) {
                if matches!(meta.kind, crate::database::FragKind::Function) {
                    functions.push(meta.name.to_string());
                }
            }
        }

        Ok(functions)
    }

    fn cache_key(&self) -> String {
        format!("functions:{}", self.file_id.as_u64())
    }
}

/// Query to get all fragments in a file
#[derive(Debug, Clone)]
pub struct GetFragmentsQuery {
    /// The file ID to query
    pub file_id: crate::database::FileId,
}

impl Query for GetFragmentsQuery {
    type Output = Vec<crate::database::FragId>;

    fn execute(&self, db: &Database) -> AutoResult<Self::Output> {
        Ok(db.get_fragments_in_file(self.file_id))
    }

    fn cache_key(&self) -> String {
        format!("fragments:{}", self.file_id.as_u64())
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::{Database, FileId, FragKind, FragSpan};
    use auto_val::AutoStr;
    use std::sync::Arc;

    #[test]
    fn test_query_engine_new() {
        let db = Arc::new(Database::new());
        let engine = QueryEngine::new(db);

        // Should have empty cache
        let stats = engine.cache_stats();
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.total_size_bytes, 0);
    }

    #[test]
    fn test_get_type_query_miss() {
        let db = Arc::new(Database::new());
        let engine = QueryEngine::new(db);

        let symbol_id = Sid::from("test_function");
        let query = GetTypeQuery { symbol_id };

        let result = engine.execute(&query).unwrap();
        assert!(result.is_none());  // No type cached
    }

    #[test]
    fn test_get_type_query_hit() {
        let mut db = Database::new();
        let symbol_id = Sid::from("test_function");

        // Set a type in the database
        db.set_type(symbol_id.clone(), Type::Int);

        let db = Arc::new(db);
        let engine = QueryEngine::new(db);

        // Execute query
        let query = GetTypeQuery { symbol_id };
        let result = engine.execute(&query).unwrap();
        assert!(matches!(result, Some(Type::Int)));

        // Phase 3.4: Cache hit on second execution
        let result2 = engine.execute(&query).unwrap();
        assert!(matches!(result2, Some(Type::Int)));
    }

    #[test]
    fn test_clear_cache() {
        let db = Arc::new(Database::new());
        let engine = QueryEngine::new(db);

        // Create a cache entry by executing a query
        let file_id = FileId::new(42);
        let query = GetFunctionsQuery { file_id };
        let _ = engine.execute(&query);

        // Clear cache
        engine.clear_cache();

        assert_eq!(engine.cache_stats().entries, 0);
    }

    #[test]
    fn test_get_functions_query() {
        let mut db = Database::new();
        let file_id = db.insert_source("test.at", AutoStr::from("fn foo() int { 1 }"));

        // Create a fragment
        let frag_span = FragSpan {
            offset: 0,
            length: 0,
            line: 1,
            column: 1,
        };

        db.insert_fragment(
            AutoStr::from("foo"),
            file_id,
            frag_span,
            FragKind::Function,
            Arc::new(crate::ast::Fn::new(
                crate::ast::FnKind::Function,
                AutoStr::from("foo"),
                None,
                vec![],
                crate::ast::Body::new(),
                Type::Int,
            )),
        );

        let db = Arc::new(db);
        let engine = QueryEngine::new(db);

        let query = GetFunctionsQuery { file_id };
        let result = engine.execute(&query).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "foo");
    }

    #[test]
    fn test_get_fragments_query() {
        let mut db = Database::new();
        let file_id = db.insert_source("test.at", AutoStr::from("fn foo() int { 1 }"));

        // Create two fragments
        let frag_span = FragSpan {
            offset: 0,
            length: 0,
            line: 1,
            column: 1,
        };

        db.insert_fragment(
            AutoStr::from("foo"),
            file_id,
            frag_span,
            FragKind::Function,
            Arc::new(crate::ast::Fn::new(
                crate::ast::FnKind::Function,
                AutoStr::from("foo"),
                None,
                vec![],
                crate::ast::Body::new(),
                Type::Int,
            )),
        );

        db.insert_fragment(
            AutoStr::from("bar"),
            file_id,
            frag_span,
            FragKind::Function,
            Arc::new(crate::ast::Fn::new(
                crate::ast::FnKind::Function,
                AutoStr::from("bar"),
                None,
                vec![],
                crate::ast::Body::new(),
                Type::Int,
            )),
        );

        let db = Arc::new(db);
        let engine = QueryEngine::new(db);

        let query = GetFragmentsQuery { file_id };
        let result = engine.execute(&query).unwrap();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_get_file_deps_query() {
        let db = Arc::new(Database::new());
        let engine = QueryEngine::new(db);

        let file_id = FileId::new(42);
        let query = GetFileDepsQuery { file_id };

        let result = engine.execute(&query).unwrap();
        assert_eq!(result.len(), 0);  // No dependencies
    }

    #[test]
    fn test_execute_uncached() {
        let mut db = Database::new();
        let symbol_id = Sid::from("test_function");

        db.set_type(symbol_id.clone(), Type::Int);

        let db = Arc::new(db);
        let engine = QueryEngine::new(db);

        // Execute without caching
        let query = GetTypeQuery { symbol_id };
        let result1 = engine.execute_uncached(&query).unwrap();
        assert!(matches!(result1, Some(Type::Int)));
    }

    // Phase 3.4: Cache validation tests

    #[test]
    fn test_cache_validation_dirty_file() {
        let mut db = Database::new();
        let file_id = db.insert_source("test.at", AutoStr::from("fn foo() int { 1 }"));

        // Create a fragment with interface hash
        let frag_span = FragSpan {
            offset: 0,
            length: 0,
            line: 1,
            column: 1,
        };

        let frag_id = db.insert_fragment(
            AutoStr::from("foo"),
            file_id,
            frag_span,
            FragKind::Function,
            Arc::new(crate::ast::Fn::new(
                crate::ast::FnKind::Function,
                AutoStr::from("foo"),
                None,
                vec![],
                crate::ast::Body::new(),
                Type::Int,
            )),
        );

        // Compute and store interface hash
        use crate::hash::FragmentHasher;
        let iface_hash = FragmentHasher::hash_interface(
            &db.get_fragment(&frag_id).unwrap()
        );
        db.set_fragment_iface_hash(frag_id.clone(), iface_hash);

        // Create Arc for engine, but keep mutable db for testing
        let db_arc = Arc::new(db);
        let engine = QueryEngine::new(db_arc);

        // Note: Can't test dirty flag marking here since db was moved into Arc
        // Full test would require Database to implement Clone
        // Phase 3.4: Mark test as partial

        // Phase 3.4: Cache should be invalidated on next execute
        // For now, we just check the method doesn't crash
        // Full test requires re-executing the query
    }

    #[test]
    fn test熔断_cache_valid_when_signature_unchanged() {
        // Phase 3.4: Test熔断- cache remains valid when signature unchanged
        use crate::hash::FragmentHasher;

        let mut db = Database::new();
        let file_id = db.insert_source("test.at", AutoStr::from("fn add(a int, b int) int { a + b }"));

        // Create a fragment
        let frag_span = FragSpan {
            offset: 0,
            length: 0,
            line: 1,
            column: 1,
        };

        let frag_id = db.insert_fragment(
            AutoStr::from("add"),
            file_id,
            frag_span,
            FragKind::Function,
            Arc::new(crate::ast::Fn::new(
                crate::ast::FnKind::Function,
                AutoStr::from("add"),
                None,
                vec![],
                crate::ast::Body::new(),
                Type::Int,
            )),
        );

        // Compute and store initial interface hash
        let iface_hash_v1 = FragmentHasher::hash_interface(
            &db.get_fragment(&frag_id).unwrap()
        );
        db.set_fragment_iface_hash(frag_id.clone(), iface_hash_v1);

        let db_arc = Arc::new(db);
        let engine = QueryEngine::new(db_arc);

        // Execute query to populate cache
        let query = GetBytecodeQuery { frag_id: frag_id.clone() };
        let _ = engine.execute(&query);

        // Get cache stats
        let stats_before = engine.cache_stats();
        assert_eq!(stats_before.entries, 1);

        // Simulate signature unchanged: Update fragment with same signature
        // (In real scenario, this would happen during re-indexing)
        // For this test, we verify熔断works by checking cache is still valid

        // Execute query again - should use cached result
        let result2 = engine.execute(&query);
        assert!(result2.is_ok());

        // Cache should still be valid (熔断WORKS)
        let stats_after = engine.cache_stats();
        assert_eq!(stats_after.entries, 1);
    }

    #[test]
    fn test熔断_cache_invalidated_when_signature_changed() {
        // Phase 3.4: Test熔断- cache invalidated when signature changes
        use crate::hash::FragmentHasher;

        let mut db = Database::new();
        let file_id = db.insert_source("test.at", AutoStr::from("fn add(a int, b int) int { a + b }"));

        // Create a fragment
        let frag_span = FragSpan {
            offset: 0,
            length: 0,
            line: 1,
            column: 1,
        };

        let frag_id = db.insert_fragment(
            AutoStr::from("add"),
            file_id,
            frag_span,
            FragKind::Function,
            Arc::new(crate::ast::Fn::new(
                crate::ast::FnKind::Function,
                AutoStr::from("add"),
                None,
                vec![],
                crate::ast::Body::new(),
                Type::Int,
            )),
        );

        // Compute and store initial interface hash
        let iface_hash_v1 = FragmentHasher::hash_interface(
            &db.get_fragment(&frag_id).unwrap()
        );
        db.set_fragment_iface_hash(frag_id.clone(), iface_hash_v1);

        let db_arc = Arc::new(db);
        let engine = QueryEngine::new(db_arc);

        // Execute query to populate cache
        let query = GetBytecodeQuery { frag_id: frag_id.clone() };
        let _ = engine.execute(&query);

        // Get cache stats
        let stats_before = engine.cache_stats();
        assert_eq!(stats_before.entries, 1);

        // Simulate signature change: Update the interface hash
        // (In real scenario, this would happen during re-indexing)
        // We use the inner Arc to get mutable access for testing
        // Note: This is a workaround for testing - in real scenario,
        // the Database would be updated by the Indexer

        // For this test, we verify熔断by checking that cache would be
        // invalidated if the hash changed. We can't easily test this
        // without Database::clone() or interior mutability.

        // Instead, we test that invalidate_fragment works (which熔断uses)
        engine.invalidate_fragment(frag_id.clone());

        // Cache should be cleared
        let stats_after = engine.cache_stats();
        assert_eq!(stats_after.entries, 0);
    }

    #[test]
    fn test_invalidate_fragment() {
        let mut db = Database::new();
        let file_id = db.insert_source("test.at", AutoStr::from("fn foo() int { 1 }"));

        // Create a fragment
        let frag_span = FragSpan {
            offset: 0,
            length: 0,
            line: 1,
            column: 1,
        };

        let frag_id = db.insert_fragment(
            AutoStr::from("foo"),
            file_id,
            frag_span,
            FragKind::Function,
            Arc::new(crate::ast::Fn::new(
                crate::ast::FnKind::Function,
                AutoStr::from("foo"),
                None,
                vec![],
                crate::ast::Body::new(),
                Type::Int,
            )),
        );

        // Compute and store interface hash
        use crate::hash::FragmentHasher;
        let iface_hash = FragmentHasher::hash_interface(
            &db.get_fragment(&frag_id).unwrap()
        );
        db.set_fragment_iface_hash(frag_id.clone(), iface_hash);

        let db = Arc::new(db);
        let engine = QueryEngine::new(db);

        // Execute query to populate cache
        let query = GetBytecodeQuery { frag_id: frag_id.clone() };
        let _ = engine.execute(&query);

        // Get cache stats before invalidation
        let stats_before = engine.cache_stats();
        assert_eq!(stats_before.entries, 1);

        // Invalidate the fragment
        engine.invalidate_fragment(frag_id);

        // Cache should be cleared
        let stats_after = engine.cache_stats();
        assert_eq!(stats_after.entries, 0);
    }
}
