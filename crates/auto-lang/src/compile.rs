// =============================================================================
// Compile: New compilation API using AIE architecture
// =============================================================================
//
// This module provides the new entry points for compilation using the
// AIE (Auto Incremental Engine) architecture with Database and Indexer.
//
// Phase 1: Demonstrate end-to-end workflow (parse → index → query)
// Phase 2: Add incremental compilation (file hashing, dirty tracking)
// Phase 3: Add fine-grained incremental (fragment hashing, patches)

use crate::database::Database;
use crate::error::{AutoError, AutoResult};
use crate::indexer::Indexer;
use crate::parser::Parser;
use crate::scope::{Sid, SID_PATH_GLOBAL};
use crate::universe::SymbolLocation;
use auto_val::AutoStr;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;

/// Compilation session using the new AIE architecture
///
/// A compilation session manages a Database and provides methods to
/// compile source code with incremental support.
///
/// Phase 4.5: Database is now wrapped in Rc<RefCell<>> for sharing with Evaler
pub struct CompileSession {
    db: Rc<RefCell<Database>>,
}

impl Clone for CompileSession {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
        }
    }
}

impl CompileSession {
    /// Create a new compilation session
    pub fn new() -> Self {
        Self {
            db: Rc::new(RefCell::new(Database::new())),
        }
    }

    /// Get reference to the database (for sharing with Evaler)
    pub fn db(&self) -> Rc<RefCell<Database>> {
        self.db.clone()
    }

    /// Get the underlying database (for advanced usage)
    pub fn database(&self) -> std::cell::Ref<Database> {
        self.db.borrow()
    }

    /// Get mutable access to the database (for advanced usage)
    pub fn database_mut(&mut self) -> std::cell::RefMut<Database> {
        self.db.borrow_mut()
    }

    /// Compile source code and index it into the database
    ///
    /// This is the main entry point for the new architecture.
    /// It parses the source code and indexes all declarations into the database.
    ///
    /// # Arguments
    ///
    /// * `source` - The AutoLang source code to compile
    /// * `path` - The file path (for error reporting and identification)
    ///
    /// # Returns
    ///
    /// A list of fragment IDs that were created.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use auto_lang::compile::CompileSession;
    ///
    /// let mut session = CompileSession::new();
    /// let source = r#"
    ///     fn add(a int, b int) int {
    ///         a + b
    ///     }
    ///
    ///     fn main() int {
    ///         add(10, 20)
    ///     }
    /// "#;
    ///
    /// let frag_ids = session.compile_source(source, "test.at").unwrap();
    /// println!("Indexed {} fragments", frag_ids.len());
    /// ```
    pub fn compile_source(
        &mut self,
        source: &str,
        path: &str,
    ) -> AutoResult<Vec<crate::database::FragId>> {
        // Insert source into database
        let file_id = self.db.borrow_mut().insert_source(path, AutoStr::from(source));

        // Parse source code to AST
        // Note: Phase 1.2 will make parser pure, but for now we use existing parser
        let scope = Rc::new(std::cell::RefCell::new(crate::universe::Universe::new()));
        let mut parser = Parser::new(source, scope.clone());
        let ast = parser.parse()
            .map_err(|e| AutoError::Msg(format!("Parse error: {}", e)))?;

        // Index AST into database
        let mut db = self.db.borrow_mut();
        let mut indexer = Indexer::new(&mut db);
        let frag_ids = indexer.index_ast(&ast, file_id)
            .map_err(|e| AutoError::Msg(format!("Index error: {}", e)))?;

        Ok(frag_ids)
    }

    /// Compile a file and index it into the database
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .at file to compile
    ///
    /// # Returns
    ///
    /// A list of fragment IDs that were created.
    pub fn compile_file(
        &mut self,
        path: &str,
    ) -> AutoResult<Vec<crate::database::FragId>> {
        // Read file
        let source = std::fs::read_to_string(path)
            .map_err(|e| AutoError::Io(format!("Failed to read file: {}", e)))?;

        self.compile_source(&source, path)
    }

    /// Get a fragment by name
    ///
    /// This demonstrates querying the database for a specific function.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the function/fragment to find
    ///
    /// # Returns
    ///
    /// The fragment AST if found.
    pub fn get_fragment_by_name(
        &self,
        name: &str,
    ) -> Option<Arc<crate::ast::Fn>> {
        // Search all fragments for one with matching name
        for file_id in self.db.borrow().get_files() {
            for frag_id in self.db.borrow().get_fragments_in_file(file_id) {
                if let Some(meta) = self.db.borrow().get_fragment_meta(&frag_id) {
                    if meta.name.as_ref() == name {
                        return self.db.borrow().get_fragment(&frag_id);
                    }
                }
            }
        }
        None
    }

    /// Get symbol location for a function (for LSP support)
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the symbol
    ///
    /// # Returns
    ///
    /// The symbol location if found.
    pub fn get_symbol_location(&self, name: &str) -> Option<SymbolLocation> {
        let symbol_id = Sid::kid_of(&SID_PATH_GLOBAL, name);
        self.db.borrow().get_symbol_location(&symbol_id).cloned()
    }

    /// List all functions in the database
    ///
    /// # Returns
    ///
    /// A list of function names.
    pub fn list_functions(&self) -> Vec<String> {
        let mut functions = Vec::new();

        for file_id in self.db.borrow().get_files() {
            for frag_id in self.db.borrow().get_fragments_in_file(file_id) {
                if let Some(meta) = self.db.borrow().get_fragment_meta(&frag_id) {
                    if matches!(meta.kind, crate::database::FragKind::Function) {
                        functions.push(meta.name.to_string());
                    }
                }
            }
        }

        functions.sort();
        functions.dedup();
        functions
    }

    /// Clear all data from the database
    pub fn clear(&mut self) {
        self.db = Rc::new(RefCell::new(Database::new()));
    }

    /// Get statistics about the database
    pub fn stats(&self) -> CompileStats {
        let mut total_frags = 0;
        let mut total_functions = 0;
        let mut total_specs = 0;

        for file_id in self.db.borrow().get_files() {
            let frags = self.db.borrow().get_fragments_in_file(file_id);
            total_frags += frags.len();

            for frag_id in &frags {
                if let Some(meta) = self.db.borrow().get_fragment_meta(frag_id) {
                    match meta.kind {
                        crate::database::FragKind::Function => total_functions += 1,
                        crate::database::FragKind::Spec => total_specs += 1,
                        _ => {}
                    }
                }
            }
        }

        CompileStats {
            total_files: self.db.borrow().get_files().len(),
            total_frags,
            total_functions,
            total_specs,
        }
    }

    /// Re-index a file with new source content (incremental compilation)
    ///
    /// This method updates a file's content in the database and re-indexes it.
    /// If the file hash hasn't changed, no recompilation occurs (empty result).
    ///
    /// # Arguments
    ///
    /// * `path` - The file path to re-index
    /// * `source` - The new source content
    ///
    /// # Returns
    ///
    /// A list of new fragment IDs if recompiled, empty if unchanged.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// let mut session = CompileSession::new();
    /// session.compile_source("fn main() int { 42 }", "test.at").unwrap();
    ///
    /// // Re-index with same content (no recompilation)
    /// let frags = session.reindex_source("test.at", "fn main() int { 42 }").unwrap();
    /// assert!(frags.is_empty());
    ///
    /// // Re-index with changed content (recompiles)
    /// let frags = session.reindex_source("test.at", "fn main() int { 100 }").unwrap();
    /// assert_eq!(frags.len(), 1);
    /// ```
    pub fn reindex_source(
        &mut self,
        path: &str,
        source: &str,
    ) -> AutoResult<Vec<crate::database::FragId>> {
        // Update source content (insert_source updates if file exists)
        self.db.borrow_mut().insert_source(path, AutoStr::from(source));

        // Get file ID
        let file_id = self.db.borrow().get_file_id_by_path(path)
            .ok_or_else(|| AutoError::Msg(format!("File not found: {}", path)))?;

        // Re-index using indexer
        let mut db = self.db.borrow_mut();
        let mut indexer = Indexer::new(&mut db);
        let frag_ids = indexer.reindex_file(file_id, source)
            .map_err(|e| AutoError::Msg(format!("Reindex error: {}", e)))?;

        Ok(frag_ids)
    }
}

impl Default for CompileSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about a compilation session
#[derive(Debug, Clone)]
pub struct CompileStats {
    pub total_files: usize,
    pub total_frags: usize,
    pub total_functions: usize,
    pub total_specs: usize,
}

// =============================================================================
// Convenience Functions
// =============================================================================

/// Compile source code in a single call (convenience function)
///
/// This creates a temporary CompileSession, compiles the source,
/// and returns the session for further queries.
///
/// # Example
///
/// ```rust,no_run
/// use auto_lang::compile::compile_once;
///
/// let session = compile_once("fn main() int { 42 }", "test.at").unwrap();
/// let main_fn = session.get_fragment_by_name("main").unwrap();
/// println!("Found main function: {}", main_fn.name);
/// ```
pub fn compile_once(source: &str, path: &str) -> AutoResult<CompileSession> {
    let mut session = CompileSession::new();
    session.compile_source(source, path)?;
    Ok(session)
}

/// Compile a file in a single call (convenience function)
pub fn compile_file_once(path: &str) -> AutoResult<CompileSession> {
    let mut session = CompileSession::new();
    session.compile_file(path)?;
    Ok(session)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Body, Fn, FnKind};

    #[test]
    fn test_compile_session_new() {
        let session = CompileSession::new();
        assert_eq!(session.stats().total_files, 0);
        assert_eq!(session.stats().total_frags, 0);
    }

    #[test]
    fn test_compile_source_simple() {
        let mut session = CompileSession::new();
        let source = "fn main() int { 42 }";

        let result = session.compile_source(source, "test.at");
        assert!(result.is_ok());

        let frag_ids = result.unwrap();
        assert_eq!(frag_ids.len(), 1);

        let stats = session.stats();
        assert_eq!(stats.total_files, 1);
        assert_eq!(stats.total_functions, 1);
    }

    #[test]
    fn test_get_fragment_by_name() {
        let mut session = CompileSession::new();
        let source = "fn add(a int, b int) int { a + b }\nfn main() int { add(10, 20) }";

        session.compile_source(source, "test.at").unwrap();

        // Should find main function
        let main_fn = session.get_fragment_by_name("main");
        assert!(main_fn.is_some());
        assert_eq!(main_fn.unwrap().name.as_ref(), "main");

        // Should find add function
        let add_fn = session.get_fragment_by_name("add");
        assert!(add_fn.is_some());
        assert_eq!(add_fn.unwrap().name.as_ref(), "add");

        // Should not find non-existent function
        let missing_fn = session.get_fragment_by_name("missing");
        assert!(missing_fn.is_none());
    }

    #[test]
    fn test_list_functions() {
        let mut session = CompileSession::new();
        // Functions must be on separate lines for the parser
        let source = "fn foo() int { 1 }\nfn bar() int { 2 }\nfn baz() int { 3 }";

        let frag_ids = session.compile_source(source, "test.at").unwrap();
        println!("Fragment IDs created: {:?}", frag_ids);
        println!("Stats: {:?}", session.stats());

        let functions = session.list_functions();
        println!("Functions found: {:?}", functions);

        assert_eq!(functions.len(), 3);
        assert!(functions.contains(&"foo".to_string()));
        assert!(functions.contains(&"bar".to_string()));
        assert!(functions.contains(&"baz".to_string()));
    }

    #[test]
    fn test_compile_multiple_files() {
        let mut session = CompileSession::new();

        // Compile first file
        let source1 = "fn foo() int { 1 }";
        session.compile_source(source1, "file1.at").unwrap();

        // Compile second file
        let source2 = "fn bar() int { 2 }";
        session.compile_source(source2, "file2.at").unwrap();

        let stats = session.stats();
        assert_eq!(stats.total_files, 2);
        assert_eq!(stats.total_functions, 2);

        // Both functions should be accessible
        assert!(session.get_fragment_by_name("foo").is_some());
        assert!(session.get_fragment_by_name("bar").is_some());
    }

    #[test]
    fn test_clear() {
        let mut session = CompileSession::new();
        session.compile_source("fn test() int { 1 }", "test.at").unwrap();

        assert_eq!(session.stats().total_functions, 1);

        session.clear();

        assert_eq!(session.stats().total_files, 0);
        assert_eq!(session.stats().total_functions, 0);
        assert!(session.get_fragment_by_name("test").is_none());
    }

    #[test]
    fn test_compile_once_convenience() {
        let source = "fn main() int { 42 }";
        let result = compile_once(source, "test.at");

        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.stats().total_functions, 1);
        assert!(session.get_fragment_by_name("main").is_some());
    }

    #[test]
    fn test_empty_source() {
        let mut session = CompileSession::new();
        let source = "";

        let result = session.compile_source(source, "test.at");
        assert!(result.is_ok());

        let frag_ids = result.unwrap();
        assert_eq!(frag_ids.len(), 0);
        assert_eq!(session.stats().total_frags, 0);
    }

    #[test]
    fn test_stats() {
        let mut session = CompileSession::new();
        let source = "fn foo() int { 1 }\nspec MySpec { fn test() void }";

        session.compile_source(source, "test.at").unwrap();

        let stats = session.stats();
        assert_eq!(stats.total_files, 1);
        assert_eq!(stats.total_frags, 2);  // 1 function + 1 spec
        assert_eq!(stats.total_functions, 1);
        assert_eq!(stats.total_specs, 1);
    }

    // =============================================================================
    // Phase 2.5: Incremental Compilation Tests
    // =============================================================================

    #[test]
    fn test_file_no_change() {
        // Test: No recompilation if file unchanged
        let mut session = CompileSession::new();
        let source = "fn main() int { 42 }";

        // First compilation
        let frag_ids1 = session.compile_source(source, "test.at").unwrap();
        assert_eq!(frag_ids1.len(), 1);

        // Get file ID and initial hash
        let file_id = session.database().get_file_id_by_path("test.at").unwrap();
        let hash1 = session.database_mut().hash_file(file_id).unwrap();

        // Re-index same content (should skip)
        let frags = session.reindex_source("test.at", source).unwrap();

        // Should not recompile (no fragments returned)
        assert!(frags.is_empty());

        // Hash should be unchanged
        let hash2 = session.database_mut().hash_file(file_id).unwrap();
        assert_eq!(hash1, hash2);

        // File should not be dirty
        assert!(!session.database().is_file_dirty(file_id));
    }

    #[test]
    fn test_file_changed() {
        // Test: Only changed file recompiled
        let mut session = CompileSession::new();
        let source1 = "fn main() int { 42 }";

        // First compilation
        let frag_ids1 = session.compile_source(source1, "test.at").unwrap();
        assert_eq!(frag_ids1.len(), 1);

        // Get file ID and initial hash
        let file_id = session.database().get_file_id_by_path("test.at").unwrap();
        let hash1 = session.database_mut().hash_file(file_id).unwrap();

        // Change source and re-index
        let source2 = "fn main() int { 100 }";
        let frags = session.reindex_source("test.at", source2).unwrap();

        // Should return new fragments (recompiled)
        assert_eq!(frags.len(), 1);

        // Hash should be changed
        let hash2 = session.database_mut().hash_file(file_id).unwrap();
        assert_ne!(hash1, hash2);

        // File should not be dirty after re-indexing
        assert!(!session.database().is_file_dirty(file_id));
    }

    #[test]
    fn test_import_chain() {
        // Test: A imports B, B changes → A recompiled
        let mut session = CompileSession::new();

        // Compile B first (dependency)
        let source_b = "fn foo() int { 42 }";
        session.compile_source(source_b, "std/b.at").unwrap();

        // Compile A
        let source_a = "fn main() int { 42 }";
        session.compile_source(source_a, "test.a.at").unwrap();

        // Get file IDs
        let (file_b, file_a) = {
            let db = session.database();
            let fb = db.get_file_id_by_path("std/b.at").unwrap();
            let fa = db.get_file_id_by_path("test.a.at").unwrap();
            (fb, fa)
        };

        // Manually add dependency: A imports B
        session.database_mut().dep_graph_mut().add_file_import(file_a, vec![file_b]);

        // Check dependency: A imports B
        {
            let db = session.database();
            let deps_a = db.dep_graph().get_file_imports(file_a);
            assert_eq!(deps_a.len(), 1);
            assert!(deps_a.contains(&file_b));
        }

        // Modify B
        let source_b_new = "fn foo() int { 100 }";
        session.reindex_source("std/b.at", source_b_new).unwrap();

        // Mark B dirty and propagate
        session.database_mut().mark_file_dirty(file_b);
        session.database_mut().propagate_dirty_recursive(file_b);

        // A should be dirty (depends on B)
        {
            let db = session.database();
            assert!(db.is_file_dirty(file_a));
        }
    }

    #[test]
    fn test_import_diamond() {
        // Test: A,B import C, C changes → A,B recompiled
        let mut session = CompileSession::new();

        // Compile C (shared dependency)
        let source_c = "fn shared() int { 42 }";
        session.compile_source(source_c, "std/c.at").unwrap();

        // Compile A
        let source_a = "fn func_a() int { 42 }";
        session.compile_source(source_a, "test/a.at").unwrap();

        // Compile B
        let source_b = "fn func_b() int { 42 }";
        session.compile_source(source_b, "test/b.at").unwrap();

        // Get file IDs
        let (file_c, file_a, file_b) = {
            let db = session.database();
            let fc = db.get_file_id_by_path("std/c.at").unwrap();
            let fa = db.get_file_id_by_path("test/a.at").unwrap();
            let fb = db.get_file_id_by_path("test/b.at").unwrap();
            (fc, fa, fb)
        };

        // Manually add dependencies: A imports C, B imports C
        session.database_mut().dep_graph_mut().add_file_import(file_a, vec![file_c]);
        session.database_mut().dep_graph_mut().add_file_import(file_b, vec![file_c]);

        // Verify diamond dependencies
        {
            let db = session.database();
            let deps_a = db.dep_graph().get_file_imports(file_a);
            let deps_b = db.dep_graph().get_file_imports(file_b);
            assert!(deps_a.contains(&file_c));
            assert!(deps_b.contains(&file_c));
        }

        // Modify C (this will mark C as dirty, propagate to A and B, then clear C's dirty flag)
        let source_c_new = "fn shared() int { 100 }";
        session.reindex_source("std/c.at", source_c_new).unwrap();

        // Both A and B should be dirty (dependents of C)
        {
            let db = session.database();
            assert!(db.is_file_dirty(file_a));
            assert!(db.is_file_dirty(file_b));
        }

        // C should not be dirty (cleared after re-index)
        {
            let db = session.database();
            assert!(!db.is_file_dirty(file_c));
        }
    }

    // =============================================================================
    // Phase 3.2: Fragment Hash熔断Tests
    // =============================================================================

    #[test]
    fn test_fragment_iface_hash_storage() {
        // Test: Fragment interface hashes are computed and stored
        let mut session = CompileSession::new();
        let source = "fn add(a int, b int) int { a + b }";

        session.compile_source(source, "test.at").unwrap();

        // Get the fragment
        let frag_id = session.database().get_fragments_in_file(
            session.database().get_file_id_by_path("test.at").unwrap()
        ).into_iter().next().unwrap();

        // Verify interface hash was computed and stored
        let hash = session.database().get_fragment_iface_hash(&frag_id);
        assert!(hash.is_some(), "Interface hash should be stored");
        assert_ne!(hash.unwrap(), 0, "Interface hash should not be zero");
    }

    #[test]
    fn test_interface_hash_unchanged_body_change() {
        // Test熔断: Function body change doesn't change interface hash
        use crate::hash::FragmentHasher;

        let mut session = CompileSession::new();

        // Initial version
        let source_v1 = "fn add(a int, b int) int { a + b }";
        session.compile_source(source_v1, "test.at").unwrap();

        // Get initial interface hash
        let file_id = session.database().get_file_id_by_path("test.at").unwrap();
        let frag_id_v1 = session.database().get_fragments_in_file(file_id)[0].clone();
        let hash_v1 = session.database().get_fragment_iface_hash(&frag_id_v1).unwrap();

        // Re-index with changed body but same signature
        let source_v2 = "fn add(a int, b int) int { a + b + 0 }";  // Body changed
        session.reindex_source("test.at", source_v2).unwrap();

        // Get new interface hash
        let frag_id_v2 = session.database().get_fragments_in_file(file_id)[0].clone();
        let hash_v2 = session.database().get_fragment_iface_hash(&frag_id_v2).unwrap();

        // Interface hash should be UNCHANGED (熔断!)
        assert_eq!(hash_v1, hash_v2, "Interface hash should be unchanged when only body changes");
    }

    #[test]
    fn test_interface_hash_changed_signature_change() {
        // Test: Signature change DOES change interface hash
        use crate::hash::FragmentHasher;

        let mut session = CompileSession::new();

        // Initial version
        let source_v1 = "fn add(a int, b int) int { a + b }";
        session.compile_source(source_v1, "test.at").unwrap();

        // Get initial interface hash
        let file_id = session.database().get_file_id_by_path("test.at").unwrap();
        let frag_id_v1 = session.database().get_fragments_in_file(file_id)[0].clone();
        let hash_v1 = session.database().get_fragment_iface_hash(&frag_id_v1).unwrap();

        // Re-index with changed signature
        let source_v2 = "fn add(a int, b int, c int) int { a + b + c }";  // Signature changed!
        session.reindex_source("test.at", source_v2).unwrap();

        // Get new interface hash
        let frag_id_v2 = session.database().get_fragments_in_file(file_id)[0].clone();
        let hash_v2 = session.database().get_fragment_iface_hash(&frag_id_v2).unwrap();

        // Interface hash should be CHANGED
        assert_ne!(hash_v1, hash_v2, "Interface hash should change when signature changes");
    }
}
