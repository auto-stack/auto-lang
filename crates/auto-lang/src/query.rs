// =============================================================================
// Query Engine: Cached query execution for AIE
// =============================================================================
//
// The Query Engine provides a caching layer for executing queries against
// the Database. Queries are computations that read from the Database
// and produce results (e.g., type inference, bytecode generation).
//
// Phase 2.4: Basic query engine with caching (no type erasure)
// Phase 3: Type-erased cache with proper invalidation

use crate::database::Database;
use crate::error::AutoResult;
use crate::ast::Type;
use crate::scope::Sid;
use dashmap::DashMap;
use std::sync::Arc;

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
///                    CacheEntry
/// ```
///
/// # Phase 2.4: Basic Caching (per-query-type caches)
///
/// - Cache results keyed by query.cache_key()
/// - No automatic invalidation (manual clearing required)
/// - Thread-safe via DashMap
pub struct QueryEngine {
    /// The database (shared reference for thread safety)
    db: Arc<Database>,

    /// Type-specific caches (indexed by type name)
    /// Phase 2: One cache per query type for simplicity
    /// Phase 3: Unified type-erased cache
    type_cache: DashMap<String, DashMap<String, Vec<u8>>>,
}

impl QueryEngine {
    /// Create a new query engine
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            type_cache: DashMap::new(),
        }
    }

    /// Execute a query with caching
    ///
    /// This method will:
    /// 1. Check the cache for a previous result
    /// 2. If cached and not dirty, return the cached result
    /// 3. Otherwise, execute the query and cache the result
    ///
    /// # Type Parameters
    ///
    /// * `Q` - Query type implementing the Query trait
    ///
    /// # Returns
    ///
    /// The query result
    ///
    /// # Note
    ///
    /// Phase 2: Results are NOT cached (serialization issues with complex types)
    /// This is a placeholder for Phase 3 which will add proper serialization.
    pub fn execute<Q: Query>(&self, query: &Q) -> AutoResult<Q::Output>
    where
        Q::Output: Clone,
    {
        let type_name = std::any::type_name::<Q>();
        let key = query.cache_key();

        // Get or create type-specific cache
        let cache = self.type_cache.entry(type_name.to_string()).or_insert_with(DashMap::new);

        // Check cache
        if let Some(_serialized) = cache.get(&key) {
            // Phase 2: For now, skip cache hit (serialization issues)
            // Phase 3: Deserialize and return
            // Cache hit
        }

        // Cache miss - execute query
        let result = query.execute(&self.db)?;

        // Phase 2: Don't cache (serialization issues with complex types)
        // Phase 3: Cache the serialized result

        Ok(result)
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

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let entries: usize = self.type_cache.iter().map(|cache| cache.value().len()).sum();
        let total_size_bytes: usize = self.type_cache.iter()
            .map(|cache| -> usize {
                cache.value().iter()
                    .map(|entry| entry.value().len())
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

        // Phase 2: No actual caching yet, just executing
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
}
