// =============================================================================
// Database: Central storage for Auto Incremental Engine (AIE)
// =============================================================================
//
// This module implements the Database struct that serves as the single source
// of truth for the AIE architecture. It replaces the old Rc<RefCell<Universe>>
// with a query-based, incremental compilation system.
//
// Architecture:
// - LAYER 1 (Storage): Sources, AST fragments, symbols (written by Indexer)
// - LAYER 2 (Cache): Types, bytecodes, dependencies (computed by Query Engine)
//
// Phase 1: Basic structure with file-level support
// Phase 2: Add file hashing and file-level dependency graph
// Phase 3: Add fragment-level hashing and fine-grained dependencies

use crate::ast::{Fn, Type};
use crate::scope::Sid;
use crate::universe::SymbolLocation;
use auto_val::AutoStr;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

// =============================================================================
// Stable Identifiers
// =============================================================================

/// File-level identifier (stable across compilations)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(u64);

impl FileId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Fragment identifier (declaration-level)
///
/// Each top-level declaration (function, struct, const) is a "fragment".
/// Fragments are the unit of incremental compilation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FragId {
    pub file: FileId,
    pub offset: usize,     // Byte offset in source file
    pub generation: u32,   // Increments on modification
}

impl FragId {
    pub fn new(file: FileId, offset: usize) -> Self {
        Self {
            file,
            offset,
            generation: 0,
        }
    }

    pub fn next_generation(&self) -> Self {
        Self {
            file: self.file,
            offset: self.offset,
            generation: self.generation + 1,
        }
    }
}

/// Symbol identifier (stable, human-readable)
///
/// Symbols represent named entities like functions, types, and variables.
/// They are hierarchical (e.g., "main", "math::add", "List::len").
pub type SymbolId = Sid;

// =============================================================================
// Fragment Metadata
// =============================================================================

/// Metadata about a fragment (declaration)
#[derive(Debug, Clone)]
pub struct FragMeta {
    pub name: AutoStr,
    pub span: FragSpan,
    pub file_id: FileId,
    pub kind: FragKind,
}

/// Location of a fragment in source code
#[derive(Debug, Clone, Copy)]
pub struct FragSpan {
    pub offset: usize,
    pub length: usize,
    pub line: usize,
    pub column: usize,
}

/// Kind of fragment
#[derive(Debug, Clone, PartialEq)]
pub enum FragKind {
    Function,
    Struct,
    Enum,
    Const,
    Spec,
    Impl,
}

// =============================================================================
// Dependency Graph (Phase 1: Placeholder, Phase 2: File-level, Phase 3: Fragment-level)
// =============================================================================

/// Dependency graph for incremental compilation
///
/// Phase 1: Placeholder structure (no tracking yet)
/// Phase 2: File-level dependencies (which files import which)
/// Phase 3: Fragment-level dependencies (which functions call which)
#[derive(Debug, Default)]
pub struct DependencyGraph {
    // Phase 2: File-level dependencies
    file_imports: HashMap<FileId, Vec<FileId>>,
    file_imported_by: HashMap<FileId, Vec<FileId>>,

    // Phase 3: Fragment-level dependencies
    frag_deps: HashMap<FragId, Vec<FragId>>,
    frag_dependents: HashMap<FragId, Vec<FragId>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    // Phase 2: File-level dependency methods
    pub fn add_file_import(&mut self, file: FileId, imports: Vec<FileId>) {
        for imported in &imports {
            self.file_imported_by
                .entry(*imported)
                .or_insert_with(Vec::new)
                .push(file);
        }
        self.file_imports.insert(file, imports);
    }

    pub fn get_file_imports(&self, file: FileId) -> &[FileId] {
        self.file_imports.get(&file)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_file_dependents(&self, file: FileId) -> &[FileId] {
        self.file_imported_by.get(&file)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    // Phase 3: Fragment-level dependency methods
    pub fn add_frag_deps(&mut self, frag: FragId, deps: Vec<FragId>) {
        for dep in &deps {
            self.frag_dependents
                .entry(dep.clone())
                .or_insert_with(Vec::new)
                .push(frag.clone());
        }
        self.frag_deps.insert(frag, deps);
    }

    pub fn get_frag_deps(&self, frag: &FragId) -> &[FragId] {
        self.frag_deps.get(frag)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_frag_dependents(&self, frag: &FragId) -> &[FragId] {
        self.frag_dependents.get(frag)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

// =============================================================================
// Database
// =============================================================================

/// Central storage for AIE (Auto Incremental Engine)
///
/// Replaces the old `Rc<RefCell<Universe>>` with a query-based database.
///
/// # Architecture
///
/// ```text
/// Database
/// ├── LAYER 1: Storage (written by Indexer only)
/// │   ├── sources: HashMap<FileId, String>
/// │   ├── frag_asts: HashMap<FragId, Arc<Fn>>
/// │   ├── frag_meta: HashMap<FragId, FragMeta>
/// │   └── symbols: HashMap<SymbolId, SymbolMeta>
/// │
/// └── LAYER 2: Cache (computed by Query Engine)
///     ├── types: DashMap<SymbolId, Type>
///     ├── bytecodes: DashMap<FragId, Blob>
///     └── dep_graph: DependencyGraph
/// ```
///
/// # Thread Safety
///
/// The storage layer (HashMap) requires `&mut` access (single writer).
/// The cache layer (DashMap) allows concurrent reads.
///
/// In Phase 1, only the Indexer has `&mut Database` access.
/// In Phase 2+, Query Engine can read cache concurrently.
pub struct Database {
    // =========================================================================
    // LAYER 1: STORAGE (written by Indexer only)
    // =========================================================================

    // Source code storage
    sources: HashMap<FileId, AutoStr>,
    file_paths: HashMap<FileId, AutoStr>,  // FileId -> path

    // Fragment storage (declaration-level)
    frag_asts: HashMap<FragId, Arc<Fn>>,
    frag_meta: HashMap<FragId, FragMeta>,

    // Fragment ID counters (per-file)
    frag_counters: HashMap<FileId, u64>,  // FileId -> next fragment index

    // Symbol metadata (for LSP support)
    symbol_locations: HashMap<SymbolId, SymbolLocation>,

    // =========================================================================
    // LAYER 2: CACHE (computed by Query Engine)
    // =========================================================================

    // Type cache (symbol -> inferred type)
    types: DashMap<SymbolId, Type>,

    // Bytecode cache (fragment -> compiled bytecode)
    bytecodes: DashMap<FragId, Vec<u8>>,

    // Dependency graph
    dep_graph: DependencyGraph,

    // =========================================================================
    // COUNTERS (for ID generation)
    // =========================================================================

    file_counter: AtomicU64,
    #[allow(dead_code)]  // Phase 2: Used for fragment ID generation
    frag_counter: AtomicU64,
}

impl Database {
    /// Create a new empty database
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            file_paths: HashMap::new(),
            frag_asts: HashMap::new(),
            frag_meta: HashMap::new(),
            symbol_locations: HashMap::new(),
            types: DashMap::new(),
            bytecodes: DashMap::new(),
            dep_graph: DependencyGraph::new(),
            file_counter: AtomicU64::new(0),
            frag_counter: AtomicU64::new(0),
            frag_counters: HashMap::new(),
        }
    }

    // =========================================================================
    // File Management
    // =========================================================================

    /// Insert source code for a file
    ///
    /// Returns the FileId assigned to this file.
    /// If the file already exists, updates the source code.
    pub fn insert_source(&mut self, path: &str, code: AutoStr) -> FileId {
        // Check if file already exists
        for (&file_id, file_path) in &self.file_paths {
            if file_path.as_ref() == path {
                // Update existing file
                self.sources.insert(file_id, code);
                return file_id;
            }
        }

        // Create new file
        let id = FileId(self.file_counter.fetch_add(1, Ordering::SeqCst));
        self.file_paths.insert(id, AutoStr::from(path));
        self.sources.insert(id, code);
        id
    }

    /// Get source code by FileId
    pub fn get_source(&self, file_id: FileId) -> Option<&AutoStr> {
        self.sources.get(&file_id)
    }

    /// Get file path by FileId
    pub fn get_file_path(&self, file_id: FileId) -> Option<&AutoStr> {
        self.file_paths.get(&file_id)
    }

    /// Get all file IDs
    pub fn get_files(&self) -> Vec<FileId> {
        self.file_paths.keys().copied().collect()
    }

    /// Remove a file and all its fragments
    pub fn remove_file(&mut self, file_id: FileId) {
        self.sources.remove(&file_id);
        self.file_paths.remove(&file_id);
        self.frag_counters.remove(&file_id);

        // Remove all fragments belonging to this file
        let frags_to_remove: Vec<_> = self.frag_meta
            .iter()
            .filter(|(_, meta)| meta.file_id == file_id)
            .map(|(frag_id, _)| frag_id.clone())
            .collect();

        for frag_id in frags_to_remove {
            self.remove_fragment(&frag_id);
        }
    }

    // =========================================================================
    // Fragment Management
    // =========================================================================

    /// Insert a fragment (top-level declaration)
    ///
    /// Returns the FragId assigned to this fragment.
    pub fn insert_fragment(
        &mut self,
        name: AutoStr,
        file_id: FileId,
        span: FragSpan,
        kind: FragKind,
        ast: Arc<Fn>,
    ) -> FragId {
        // Generate unique fragment ID using file-specific counter
        let frag_index = self.frag_counters.entry(file_id).or_insert(0);
        let frag_id = FragId {
            file: file_id,
            offset: *frag_index as usize,  // Use counter instead of span offset
            generation: 0,
        };
        *frag_index += 1;

        let meta = FragMeta {
            name: name.clone(),
            span,
            file_id,
            kind,
        };

        self.frag_meta.insert(frag_id.clone(), meta);
        self.frag_asts.insert(frag_id.clone(), ast);

        frag_id
    }

    /// Get fragment AST by FragId
    pub fn get_fragment(&self, frag_id: &FragId) -> Option<Arc<Fn>> {
        self.frag_asts.get(frag_id).cloned()
    }

    /// Get fragment metadata by FragId
    pub fn get_fragment_meta(&self, frag_id: &FragId) -> Option<&FragMeta> {
        self.frag_meta.get(frag_id)
    }

    /// Get all fragments for a file
    pub fn get_fragments_in_file(&self, file_id: FileId) -> Vec<FragId> {
        self.frag_meta
            .iter()
            .filter(|(_, meta)| meta.file_id == file_id)
            .map(|(frag_id, _)| frag_id.clone())
            .collect()
    }

    /// Remove a fragment
    pub fn remove_fragment(&mut self, frag_id: &FragId) {
        self.frag_asts.remove(frag_id);
        self.frag_meta.remove(frag_id);
        self.bytecodes.remove(frag_id);
        self.dep_graph.frag_deps.remove(frag_id);
        self.dep_graph.frag_dependents.remove(frag_id);
    }

    /// Clear all fragments for a file (for re-indexing)
    pub fn clear_file_fragments(&mut self, file_id: FileId) {
        let frags: Vec<_> = self.get_fragments_in_file(file_id);
        for frag_id in frags {
            self.remove_fragment(&frag_id);
        }
    }

    // =========================================================================
    // Symbol Management (for LSP support)
    // =========================================================================

    /// Register a symbol's definition location
    pub fn define_symbol_location(&mut self, name: SymbolId, location: SymbolLocation) {
        self.symbol_locations.insert(name, location);
    }

    /// Lookup a symbol's definition location
    pub fn get_symbol_location(&self, name: &SymbolId) -> Option<&SymbolLocation> {
        self.symbol_locations.get(name)
    }

    /// Get all symbol locations (for LSP workspace symbols)
    pub fn get_all_symbol_locations(&self) -> &HashMap<SymbolId, SymbolLocation> {
        &self.symbol_locations
    }

    /// Clear all symbol locations (when re-parsing a file)
    pub fn clear_symbol_locations(&mut self) {
        self.symbol_locations.clear();
    }

    // =========================================================================
    // Type Cache (Layer 2: Cache)
    // =========================================================================

    /// Get cached type for a symbol
    pub fn get_type(&self, symbol_id: &SymbolId) -> Option<Type> {
        self.types.get(symbol_id).map(|entry| entry.clone())
    }

    /// Set cached type for a symbol
    pub fn set_type(&self, symbol_id: SymbolId, ty: Type) {
        self.types.insert(symbol_id, ty);
    }

    /// Clear type cache for a symbol
    pub fn clear_type(&self, symbol_id: &SymbolId) {
        self.types.remove(symbol_id);
    }

    /// Clear entire type cache
    pub fn clear_all_types(&self) {
        self.types.clear();
    }

    // =========================================================================
    // Bytecode Cache (Layer 2: Cache)
    // =========================================================================

    /// Get cached bytecode for a fragment
    pub fn get_bytecode(&self, frag_id: &FragId) -> Option<Vec<u8>> {
        self.bytecodes.get(frag_id).map(|entry| entry.clone())
    }

    /// Set cached bytecode for a fragment
    pub fn set_bytecode(&self, frag_id: FragId, bytecode: Vec<u8>) {
        self.bytecodes.insert(frag_id, bytecode);
    }

    /// Clear bytecode cache for a fragment
    pub fn clear_bytecode(&self, frag_id: &FragId) {
        self.bytecodes.remove(frag_id);
    }

    /// Clear entire bytecode cache
    pub fn clear_all_bytecodes(&self) {
        self.bytecodes.clear();
    }

    // =========================================================================
    // Dependency Graph Access
    // =========================================================================

    /// Get the dependency graph (mutable access)
    pub fn dep_graph_mut(&mut self) -> &mut DependencyGraph {
        &mut self.dep_graph
    }

    /// Get the dependency graph (read-only access)
    pub fn dep_graph(&self) -> &DependencyGraph {
        &self.dep_graph
    }

    // =========================================================================
    // Query Methods (Phase 2: Incremental Compilation)
    // =========================================================================

    /// Check if a file needs recompilation
    ///
    /// Phase 2: Compare file hashes
    /// Phase 1: Always returns true (no hashing yet)
    pub fn is_file_dirty(&self, _file_id: FileId) -> bool {
        // Phase 1: No hashing yet, always dirty
        true
    }

    /// Mark a file as dirty (needs recompilation)
    ///
    /// Phase 2: Update dirty flags
    /// Phase 1: No-op
    pub fn mark_file_dirty(&mut self, _file_id: FileId) {
        // Phase 1: No dirty tracking yet
    }

    /// Get all dirty files
    ///
    /// Phase 2: Scan dirty flags
    /// Phase 1: Returns all files
    pub fn get_dirty_files(&self) -> Vec<FileId> {
        // Phase 1: All files are dirty
        self.get_files()
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_new() {
        let db = Database::new();
        assert_eq!(db.get_files().len(), 0);
    }

    #[test]
    fn test_insert_source() {
        let mut db = Database::new();

        let code = AutoStr::from("fn main() int { 42 }");
        let file_id = db.insert_source("test.at", code);

        assert_eq!(file_id.as_u64(), 0);

        let retrieved = db.get_source(file_id).unwrap();
        assert_eq!(retrieved.as_ref(), "fn main() int { 42 }");

        let path = db.get_file_path(file_id).unwrap();
        assert_eq!(path.as_ref(), "test.at");
    }

    #[test]
    fn test_insert_source_update_existing() {
        let mut db = Database::new();

        let code1 = AutoStr::from("fn main() int { 42 }");
        let file_id1 = db.insert_source("test.at", code1);

        let code2 = AutoStr::from("fn main() int { 100 }");
        let file_id2 = db.insert_source("test.at", code2);

        // Should return same FileId
        assert_eq!(file_id1, file_id2);

        // Should have updated code
        let retrieved = db.get_source(file_id1).unwrap();
        assert_eq!(retrieved.as_ref(), "fn main() int { 100 }");
    }

    #[test]
    fn test_get_files() {
        let mut db = Database::new();

        db.insert_source("test1.at", AutoStr::from("code1"));
        db.insert_source("test2.at", AutoStr::from("code2"));

        let files = db.get_files();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_remove_file() {
        let mut db = Database::new();

        let file_id = db.insert_source("test.at", AutoStr::from("code"));
        db.remove_file(file_id);

        assert!(db.get_source(file_id).is_none());
        assert!(db.get_file_path(file_id).is_none());
        assert_eq!(db.get_files().len(), 0);
    }

    #[test]
    fn test_fragment_id() {
        let file_id = FileId::new(42);
        let frag_id = FragId::new(file_id, 100);

        assert_eq!(frag_id.file.as_u64(), 42);
        assert_eq!(frag_id.offset, 100);
        assert_eq!(frag_id.generation, 0);

        let next_id = frag_id.next_generation();
        assert_eq!(next_id.generation, 1);
        assert_eq!(next_id.file.as_u64(), 42);
        assert_eq!(next_id.offset, 100);
    }

    #[test]
    fn test_dep_graph_file_level() {
        let mut graph = DependencyGraph::new();

        let file_a = FileId::new(0);
        let file_b = FileId::new(1);
        let file_c = FileId::new(2);

        // A imports B and C
        graph.add_file_import(file_a, vec![file_b, file_c]);

        // Check imports
        let imports = graph.get_file_imports(file_a);
        assert_eq!(imports.len(), 2);
        assert!(imports.contains(&file_b));
        assert!(imports.contains(&file_c));

        // Check dependents (reverse)
        let dependents = graph.get_file_dependents(file_b);
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0], file_a);
    }

    #[test]
    fn test_dep_graph_fragment_level() {
        let mut graph = DependencyGraph::new();

        let frag_a = FragId::new(FileId::new(0), 0);
        let frag_b = FragId::new(FileId::new(0), 10);
        let frag_c = FragId::new(FileId::new(0), 20);

        // A depends on B and C
        graph.add_frag_deps(frag_a.clone(), vec![frag_b.clone(), frag_c.clone()]);

        // Check dependencies
        let deps = graph.get_frag_deps(&frag_a);
        assert_eq!(deps.len(), 2);

        // Check dependents (reverse)
        let dependents = graph.get_frag_dependents(&frag_b);
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0], frag_a);
    }

    #[test]
    fn test_type_cache() {
        let db = Database::new();
        let symbol_id = SymbolId::from("test_function");

        // Cache miss
        assert!(db.get_type(&symbol_id).is_none());

        // Set cache
        db.set_type(symbol_id.clone(), Type::Int);

        // Cache hit
        let cached = db.get_type(&symbol_id).unwrap();
        assert!(matches!(cached, Type::Int));

        // Clear cache
        db.clear_type(&symbol_id);
        assert!(db.get_type(&symbol_id).is_none());
    }

    #[test]
    fn test_bytecode_cache() {
        let db = Database::new();
        let frag_id = FragId::new(FileId::new(0), 100);

        // Cache miss
        assert!(db.get_bytecode(&frag_id).is_none());

        // Set cache
        let bytecode = vec![0x01, 0x02, 0x03];
        db.set_bytecode(frag_id.clone(), bytecode.clone());

        // Cache hit
        let cached = db.get_bytecode(&frag_id).unwrap();
        assert_eq!(cached, bytecode);

        // Clear cache
        db.clear_bytecode(&frag_id);
        assert!(db.get_bytecode(&frag_id).is_none());
    }
}
