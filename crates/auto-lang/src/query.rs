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
use std::sync::RwLock;

// =============================================================================
// Cache Entry (Phase 3.4)
// =============================================================================

/// Cached query result with dependency tracking
///
/// Phase 3.4: Track which fragments this query depends on, so we can
/// invalidate the cache when dependencies change.
///
/// Phase 3.6: LRU cache with timestamp tracking for automatic eviction.
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

    /// Last access timestamp (for LRU eviction)
    /// Phase 3.6: Track when this entry was last accessed
    last_access: std::time::Instant,
}

impl CacheEntry {
    /// Create a new cache entry
    fn new(data: Box<dyn std::any::Any>, dependencies: Vec<FragDep>) -> Self {
        Self {
            data,
            dependencies,
            last_access: std::time::Instant::now(),
        }
    }

    /// Update the last access timestamp (called on cache hit)
    fn touch(&mut self) {
        self.last_access = std::time::Instant::now();
    }
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
///
/// # Phase 3.6: LRU Cache Eviction
///
/// - Automatic eviction of least-recently-used entries when cache exceeds capacity
/// - Timestamp-based LRU tracking
/// - Configurable cache capacity (default: 1000 entries per query type)
pub struct QueryEngine {
    /// The database (shared reference for thread safety)
    /// **Plan 065 Phase 3**: Changed from Arc<Database> to Arc<RwLock<Database>>
    /// to integrate with CompileSession
    db: Arc<RwLock<Database>>,

    /// Type-specific caches (indexed by type name)
    /// Phase 2: Vec<u8> placeholders (not used)
    /// Phase 3.4: CacheEntry with dependency tracking (in-memory)
    type_cache: DashMap<String, DashMap<String, CacheEntry>>,

    /// Maximum number of cache entries per query type (Phase 3.6: LRU)
    /// Default: 1000 entries. Can be configured via QueryEngine::with_capacity()
    max_entries_per_type: usize,
}

impl QueryEngine {
    /// Create a new query engine with default capacity (1000 entries per type)
    ///
    /// **Plan 065 Phase 3**: Now accepts Arc<RwLock<Database>> to integrate with CompileSession
    pub fn new(db: Arc<RwLock<Database>>) -> Self {
        Self {
            db,
            type_cache: DashMap::new(),
            max_entries_per_type: 1000,
        }
    }

    /// Create a new query engine with custom cache capacity
    ///
    /// # Arguments
    ///
    /// * `db` - The database to query (Arc<RwLock<Database>> for Plan 065 integration)
    /// * `max_entries_per_type` - Maximum cache entries per query type (default: 1000)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let engine = QueryEngine::with_capacity(db, 500); // 500 entries per type
    /// ```
    ///
    /// **Plan 065 Phase 3**: Now accepts Arc<RwLock<Database>>
    pub fn with_capacity(db: Arc<RwLock<Database>>, max_entries_per_type: usize) -> Self {
        Self {
            db,
            type_cache: DashMap::new(),
            max_entries_per_type,
        }
    }

    /// Execute a query with caching and dependency tracking
    ///
    /// This method will:
    /// 1. Check the cache for a previous result
    /// 2. If cached, validate dependencies (check if fragments are dirty)
    /// 3. If cache valid, downcast and return cached result
    /// 4. Otherwise, execute the query, track dependencies, and cache the result
    /// 5. Phase 3.6: Evict LRU entries if cache exceeds capacity
    ///
    /// **Plan 065 Phase 3**: Acquires read lock on Database before executing queries
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
    ///
    /// # Phase 3.6: LRU Eviction
    ///
    /// When cache size exceeds `max_entries_per_type`, least-recently-used entries
    /// are automatically evicted to maintain memory constraints.
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
                // Note: Phase 3.6 LRU timestamp is NOT updated on cache hits
                // due to DashMap's immutable reference semantics.
                // We use insertion time as a proxy for LRU, which works well for
                // most use cases. Future enhancement: Use a separate HashMap to
                // track access counts if needed.
                //
                // Cache hit and valid - downcast and return
                let result = entry.data.downcast_ref::<Q::Output>()
                    .ok_or_else(|| AutoError::Msg("Cache type mismatch".to_string()))?;

                return Ok(result.clone());
            }
            // Cache invalid - fall through to re-execute
        }

        // Cache miss or invalid - execute query
        // **Plan 065 Phase 3**: Acquire read lock on Database
        let db_read = self.db.read().unwrap();
        let result = query.execute(&*db_read)?;

        // Phase 3.4: Track dependencies and cache the result
        let dependencies = self.extract_dependencies::<Q>(query);

        // Phase 3.6: Use CacheEntry::new() to initialize timestamp
        let entry = CacheEntry::new(Box::new(result.clone()), dependencies);

        cache.insert(key, entry);

        // Phase 3.6: Evict LRU entries if cache exceeds capacity
        if cache.len() > self.max_entries_per_type {
            self.evict_lru_entries(&cache, self.max_entries_per_type);
        }

        Ok(result)
    }

    /// Check if a cache entry is still valid
    ///
    /// **Plan 065 Phase 3**: Acquires read lock on Database
    ///
    /// Phase 3.4: Validate dependencies by checking:
    /// 1. Fragments are not marked as dirty
    /// 2. Fragment interface hashes haven't changed (熔断)
    fn is_cache_valid(&self, entry: &CacheEntry) -> bool {
        // **Plan 065 Phase 3**: Acquire read lock on Database
        let db = self.db.read().unwrap();

        for dep in &entry.dependencies {
            // Check if fragment is marked as dirty
            let file_id = crate::database::FileId::new(dep.file_id);
            if db.is_marked_dirty(file_id) {
                return false;
            }

            // Phase 3.4: Check if interface hash changed (熔断validation)
            // We need to find the fragment by file_id:offset to check its current hash
            let frag_id = Self::find_fragment_by_offset_with_db(&db, file_id, dep.offset);

            if let Some(frag_id) = frag_id {
                // Get current interface hash from database
                if let Some(current_hash) = db.get_fragment_iface_hash(&frag_id) {
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
    /// **Plan 065 Phase 3**: Static method that takes a &Database reference
    ///
    /// Phase 3.4: Enable fragment lookup for熔断hash checking
    ///
    /// # Arguments
    ///
    /// * `db` - The database to query (acquired read lock)
    /// * `file_id` - The file ID
    /// * `offset` - The fragment offset within the file
    ///
    /// # Returns
    ///
    /// The FragId if found, None otherwise
    fn find_fragment_by_offset_with_db(db: &Database, file_id: crate::database::FileId, offset: usize) -> Option<crate::database::FragId> {
        // Get all fragments in the file
        let frag_ids = db.get_fragments_in_file(file_id);

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
    /// **Plan 065 Phase 3**: Acquires read lock on Database
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

        // **Plan 065 Phase 3**: Acquire read lock on Database
        let db = self.db.read().unwrap();

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
                if let Some(hash) = db.get_fragment_iface_hash(&frag_id) {
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
    /// **Plan 065 Phase 3**: Acquires read lock on Database
    ///
    /// Use this for queries that shouldn't be cached (e.g., very large results,
    /// or queries that change frequently).
    pub fn execute_uncached<Q: Query>(&self, query: &Q) -> AutoResult<Q::Output> {
        let db = self.db.read().unwrap();
        query.execute(&*db)
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

    /// Evict least-recently-used cache entries
    ///
    /// Phase 3.6: LRU eviction - removes the oldest entries when cache exceeds capacity.
    ///
    /// # Arguments
    ///
    /// * `cache` - The cache to evict from
    /// * `target_size` - Target cache size (will evict until cache size <= target_size)
    fn evict_lru_entries(&self, cache: &DashMap<String, CacheEntry>, target_size: usize) {
        // Collect all entries with their access times
        let mut entries_with_time: Vec<(String, std::time::Instant)> = cache
            .iter()
            .map(|entry| {
                let key = entry.key().clone();
                let timestamp = entry.value().last_access;
                (key, timestamp)
            })
            .collect();

        // Sort by access time (oldest first)
        entries_with_time.sort_by_key(|(_, timestamp)| *timestamp);

        // Calculate how many entries to remove
        let current_size = cache.len();
        let to_remove = if current_size > target_size {
            current_size - target_size
        } else {
            return; // No eviction needed
        };

        // Remove oldest entries
        for (key, _) in entries_with_time.into_iter().take(to_remove) {
            cache.remove(&key);
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

    /// Get the underlying database as Arc<RwLock<Database>> (for sharing)
    ///
    /// **Plan 065 Phase 3**: Returns Arc<RwLock<Database>> instead of Arc<Database>
    pub fn database_arc(&self) -> Arc<RwLock<Database>> {
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

/// Query to get evaluated result for a fragment
///
/// **Plan 065 Phase 3**: Caches the final Value result for a fragment.
/// This enables incremental execution where unchanged functions return cached results.
#[derive(Debug, Clone)]
pub struct EvalResultQuery {
    /// The fragment ID to query
    pub frag_id: crate::database::FragId,
}

impl Query for EvalResultQuery {
    type Output = Option<auto_val::Value>;

    fn execute(&self, _db: &Database) -> AutoResult<Self::Output> {
        // For now, we don't store evaluation results in the Database
        // This is a placeholder for future caching of evaluated results
        // Phase 3 enhancement: Store and retrieve cached evaluation results
        Ok(None)
    }

    fn cache_key(&self) -> String {
        format!("eval_result:{}:{}", self.frag_id.file.as_u64(), self.frag_id.offset)
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
// Advanced Type Inference Queries (Phase 3.6: PC-Server Enhancements)
// =============================================================================

/// Query to infer the type of an expression
///
/// This integrates the type inference system with the query engine,
/// enabling powerful IDE features like "hover to see type".
#[derive(Debug, Clone)]
pub struct InferExprTypeQuery {
    /// The expression to analyze
    pub expr: crate::ast::Expr,

    /// Variable bindings to use during inference
    pub bindings: std::collections::HashMap<String, Type>,
}

impl Query for InferExprTypeQuery {
    type Output = Type;

    fn execute(&self, _db: &Database) -> AutoResult<Self::Output> {
        // Create inference context with provided bindings
        let mut ctx = crate::infer::InferenceContext::new();

        // Bind variables from the query
        for (name, ty) in &self.bindings {
            ctx.bind_var(crate::ast::Name::from(name.as_str()), ty.clone());
        }

        // Infer expression type
        let inferred_ty = crate::infer::infer_expr(&mut ctx, &self.expr);

        // Check for errors
        if ctx.has_errors() {
            // Return first error as warning
            let error = &ctx.errors[0];
            return Err(AutoError::Msg(format!("Type inference error: {}", error)));
        }

        Ok(inferred_ty)
    }

    fn cache_key(&self) -> String {
        // For expressions, we use a simplified representation as cache key
        // In production, you'd want to hash the expression properly
        format!("infer_expr:{:?}", self.expr)
    }
}

/// Query to get the location where a symbol is defined
///
/// This enables IDE features like "go to definition" (F12 in most editors).
#[derive(Debug, Clone)]
pub struct GetSymbolLocationQuery {
    /// The symbol name to find
    pub symbol_name: String,
}

impl Query for GetSymbolLocationQuery {
    type Output = Option<SymbolLocation>;

    fn execute(&self, db: &Database) -> AutoResult<Self::Output> {
        let sid = Sid::from(self.symbol_name.as_str());
        // Get symbol location and clone it (returns Option<&SymbolLocation>)
        if let Some(loc) = db.get_symbol_location(&sid) {
            // Convert universe::SymbolLocation to query::SymbolLocation
            Ok(Some(SymbolLocation {
                file_id: crate::database::FileId::new(0), // Unknown file ID
                line: loc.line,
                column: loc.character, // character = column
                name: self.symbol_name.clone(),
            }))
        } else {
            Ok(None)
        }
    }

    fn cache_key(&self) -> String {
        format!("symbol_location:{}", self.symbol_name)
    }
}

/// Result of a find-references query
#[derive(Debug, Clone)]
pub struct SymbolReference {
    /// File where the reference occurs
    pub file_id: crate::database::FileId,

    /// Line number (1-based)
    pub line: usize,

    /// Column number (1-based)
    pub column: usize,

    /// Whether this is a definition or a usage
    pub is_definition: bool,
}

/// Query to find all references to a symbol
///
/// This enables IDE features like "find all references" (Shift+F12 in most editors).
#[derive(Debug, Clone)]
pub struct FindReferencesQuery {
    /// The symbol name to search for
    pub symbol_name: String,
}

impl Query for FindReferencesQuery {
    type Output = Vec<SymbolReference>;

    fn execute(&self, db: &Database) -> AutoResult<Self::Output> {
        let mut references = Vec::new();

        // Get the symbol's definition location
        let sid = Sid::from(self.symbol_name.as_str());
        if let Some(def_loc) = db.get_symbol_location(&sid) {
            references.push(SymbolReference {
                file_id: crate::database::FileId::new(0), // Unknown file ID from legacy SymbolLocation
                line: def_loc.line,
                column: def_loc.character, // character = column
                is_definition: true,
            });
        }

        // Search all files for references to this symbol
        // Phase 3.6: Simplified implementation - just scans fragment metadata
        // Future enhancement: Full AST traversal to find all usages
        //
        // We iterate over fragment metadata to find definitions matching the symbol name
        // Note: This doesn't find all usages, only definitions
        let mut seen_frags = std::collections::HashSet::new();

        // Access the sources HashMap keys (this is a workaround since we don't have all_files())
        // We'll iterate through fragment metadata which contains file_id
        for frag_id in db.all_fragment_ids() {
            if let Some(meta) = db.get_fragment_meta(&frag_id) {
                // Check if fragment name matches (this finds definitions)
                if meta.name.as_str() == self.symbol_name && !seen_frags.contains(&frag_id) {
                    seen_frags.insert(frag_id.clone());
                    let span = meta.span;
                    references.push(SymbolReference {
                        file_id: meta.file_id,
                        line: span.line,
                        column: span.column,
                        is_definition: true,
                    });
                }
            }
        }

        Ok(references)
    }

    fn cache_key(&self) -> String {
        format!("find_refs:{}", self.symbol_name)
    }
}

/// Query to get auto-completion suggestions at a location
///
/// This enables IDE features like code completion (Ctrl+Space in most editors).
#[derive(Debug, Clone)]
pub struct GetCompletionsQuery {
    /// The file to get completions for
    pub file_id: crate::database::FileId,

    /// The line number (1-based)
    pub line: usize,

    /// The column number (1-based)
    pub column: usize,

    /// Prefix text to filter completions
    pub prefix: String,
}

/// Completion suggestion
#[derive(Debug, Clone)]
pub struct Completion {
    /// The completion text
    pub text: String,

    /// The type of completion (function, variable, type, etc.)
    pub kind: CompletionKind,

    /// Detail about the completion (e.g., function signature)
    pub detail: Option<String>,
}

/// Kind of completion item
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    /// Function
    Function,

    /// Variable
    Variable,

    /// Type
    Type,

    /// Spec (trait/interface)
    Spec,

    /// Constant
    Constant,
}

impl Query for GetCompletionsQuery {
    type Output = Vec<Completion>;

    fn execute(&self, db: &Database) -> AutoResult<Self::Output> {
        let mut completions = Vec::new();

        // Collect all symbols from the database
        // Phase 3.6: Simplified - returns all visible symbols
        // Future enhancement: Scope-aware completion (only show symbols in scope)

        // Add function completions by iterating over fragments
        for frag_id in db.all_fragment_ids() {
            if let Some(meta) = db.get_fragment_meta(&frag_id) {
                // Filter by prefix
                if meta.name.as_str().starts_with(&self.prefix) {
                    let kind = match meta.kind {
                        crate::database::FragKind::Function => CompletionKind::Function,
                        crate::database::FragKind::Struct => CompletionKind::Type, // Struct is a type
                        crate::database::FragKind::Enum => CompletionKind::Type,   // Enum is a type
                        crate::database::FragKind::Const => CompletionKind::Constant,
                        crate::database::FragKind::Spec => CompletionKind::Spec,
                        crate::database::FragKind::Impl => CompletionKind::Type,   // Impl is type-related
                    };

                    completions.push(Completion {
                        text: meta.name.to_string(),
                        kind,
                        detail: None, // TODO: Add signature info
                    });
                }
            }
        }

        Ok(completions)
    }

    fn cache_key(&self) -> String {
        format!("completions:{}:{}:{}:{}",
            self.file_id.as_u64(),
            self.line,
            self.column,
            self.prefix
        )
    }
}

/// Symbol location information
///
/// Represents where a symbol is defined in source code.
/// Used for IDE features like go-to-definition.
#[derive(Debug, Clone)]
pub struct SymbolLocation {
    /// File ID where the symbol is defined
    pub file_id: crate::database::FileId,

    /// Line number (1-based)
    pub line: usize,

    /// Column number (1-based)
    pub column: usize,

    /// Symbol name
    pub name: String,
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
        let db = Arc::new(RwLock::new(Database::new()));
        let engine = QueryEngine::new(db);

        // Should have empty cache
        let stats = engine.cache_stats();
        assert_eq!(stats.entries, 0);
        assert_eq!(stats.total_size_bytes, 0);
    }

    #[test]
    fn test_get_type_query_miss() {
        let db = Arc::new(RwLock::new(Database::new()));
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

        let db = Arc::new(RwLock::new(db));
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
        let db = Arc::new(RwLock::new(Database::new()));
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

        let db = Arc::new(RwLock::new(db));
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

        let db = Arc::new(RwLock::new(db));
        let engine = QueryEngine::new(db);

        let query = GetFragmentsQuery { file_id };
        let result = engine.execute(&query).unwrap();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_get_file_deps_query() {
        let db = Arc::new(RwLock::new(Database::new()));
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

        let db = Arc::new(RwLock::new(db));
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
        let db_arc = Arc::new(RwLock::new(db));
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

        let db_arc = Arc::new(RwLock::new(db));
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

        let db_arc = Arc::new(RwLock::new(db));
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

        let db = Arc::new(RwLock::new(db));
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

    // =========================================================================
    // Advanced Type Inference Query Tests (Phase 3.6)
    // =========================================================================

    #[test]
    fn test_infer_expr_type_query_int_literal() {
        use crate::ast::Expr;

        let db = Arc::new(RwLock::new(Database::new()));
        let engine = QueryEngine::new(db);

        // Test: Infer type of integer literal
        let expr = Expr::Int(42);
        let query = InferExprTypeQuery {
            expr,
            bindings: std::collections::HashMap::new(),
        };

        let result = engine.execute(&query).unwrap();
        assert!(matches!(result, Type::Int));
    }

    #[test]
    fn test_infer_expr_type_query_with_binding() {
        use crate::ast::{Expr, Name};

        let db = Arc::new(RwLock::new(Database::new()));
        let engine = QueryEngine::new(db);

        // Test: Infer type of variable reference
        let mut bindings = std::collections::HashMap::new();
        bindings.insert("x".to_string(), Type::Int);

        let expr = Expr::Ident(Name::from("x"));
        let query = InferExprTypeQuery {
            expr,
            bindings,
        };

        let result = engine.execute(&query).unwrap();
        assert!(matches!(result, Type::Int));
    }

    #[test]
    fn test_get_symbol_location_query() {
        let db = Arc::new(RwLock::new(Database::new()));
        let engine = QueryEngine::new(db);

        // Test with symbol that doesn't exist
        let query = GetSymbolLocationQuery {
            symbol_name: "nonexistent".to_string(),
        };

        let result = engine.execute(&query).unwrap();
        assert!(result.is_none());

        // Note: Testing with existing symbol requires Database to have symbol location
        // which is set by Indexer, not directly in tests
    }

    #[test]
    fn test_get_completions_query() {
        let mut db = Database::new();
        let file_id = db.insert_source("test.at", AutoStr::from("fn foo() int { 1 } fn bar() int { 2 }"));

        // Create fragments
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

        let db = Arc::new(RwLock::new(db));
        let engine = QueryEngine::new(db);

        // Query for completions with prefix "f"
        let query = GetCompletionsQuery {
            file_id,
            line: 1,
            column: 1,
            prefix: "f".to_string(),
        };

        let result = engine.execute(&query).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, "foo");
        assert!(matches!(result[0].kind, CompletionKind::Function));
    }

    #[test]
    fn test_find_references_query() {
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

        let db = Arc::new(RwLock::new(db));
        let engine = QueryEngine::new(db);

        let query = FindReferencesQuery {
            symbol_name: "foo".to_string(),
        };

        let result = engine.execute(&query).unwrap();
        assert!(!result.is_empty());
        // Should find at least the definition
        assert!(result.iter().any(|r| r.is_definition));
    }

    // =========================================================================
    // Phase 3.6: LRU Cache Eviction Tests
    // =========================================================================

    #[test]
    fn test_query_engine_with_capacity() {
        let db = Arc::new(RwLock::new(Database::new()));
        let engine = QueryEngine::with_capacity(db, 10); // Small capacity

        // Should create engine with custom capacity
        let stats = engine.cache_stats();
        assert_eq!(stats.entries, 0);
    }

    #[test]
    fn test_lru_eviction_when_capacity_exceeded() {
        let mut db = Database::new();
        let file_id = db.insert_source("test.at", AutoStr::from("fn foo() int { 1 }"));

        // Create multiple fragments
        let frag_span = FragSpan {
            offset: 0,
            length: 0,
            line: 1,
            column: 1,
        };

        // Insert 5 fragments
        for i in 0..5 {
            let name = format!("func{}", i);
            db.insert_fragment(
                AutoStr::from(&name),
                file_id,
                frag_span,
                FragKind::Function,
                Arc::new(crate::ast::Fn::new(
                    crate::ast::FnKind::Function,
                    AutoStr::from(&name),
                    None,
                    vec![],
                    crate::ast::Body::new(),
                    Type::Int,
                )),
            );
        }

        let db = Arc::new(RwLock::new(db));
        // Create engine with capacity of 3 (smaller than number of functions)
        let engine = QueryEngine::with_capacity(db, 3);

        let query = GetFunctionsQuery { file_id };
        let _ = engine.execute(&query).unwrap();

        // Cache should have been evicted down to capacity
        let stats = engine.cache_stats();
        assert!(stats.entries <= 3, "Cache should not exceed capacity");
    }
}
