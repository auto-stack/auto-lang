// =============================================================================
// Indexer: Resilient parsing and fragment registration
// =============================================================================
//
// The Indexer is responsible for:
// 1. Scanning source code for declaration boundaries
// 2. Splitting code into independent Fragments (functions, structs, consts)
// 3. Registering symbols in the Database with stable IDs
//
// Phase 1: Basic indexing (parse top-level declarations)
// Phase 2: Add file-level dependency tracking (import statements)
// Phase 3: Add fragment-level dependency tracking (function calls, type usage)

use crate::ast::{Code, Fn, SpecDecl, Stmt, Type};
use crate::database::{Database, FileId, FragId, FragKind, FragSpan};
use crate::error::{AutoError, AutoResult};
use crate::parser::Parser;
use crate::scope::{Sid, SID_PATH_GLOBAL};
use crate::universe::SymbolLocation;
use auto_val::AutoStr;
use std::rc::Rc;
use std::sync::Arc;

/// Indexer: Responsible for fragmenting source code and registering symbols
///
/// The Indexer is the ONLY component that has write access to the Database.
/// All other components (Parser, Query Engine, Transpilers) have read-only access.
pub struct Indexer<'db> {
    db: &'db mut Database,
}

impl<'db> Indexer<'db> {
    /// Create a new indexer for the given database
    pub fn new(db: &'db mut Database) -> Self {
        Self { db }
    }

    /// Index an AST (parsed code) into the database
    ///
    /// This function:
    /// 1. Scans the AST for top-level declarations
    /// 2. Creates Fragments for each declaration
    /// 3. Registers symbols in the Database
    ///
    /// Returns a list of Fragment IDs that were created.
    pub fn index_ast(&mut self, ast: &Code, file_id: FileId) -> Result<Vec<FragId>, String> {
        let mut frag_ids = Vec::new();

        for stmt in &ast.stmts {
            match stmt {
                // Function declarations
                Stmt::Fn(fn_decl) => {
                    let frag_id = self.index_function(fn_decl, file_id)?;
                    frag_ids.push(frag_id);
                }

                // Type declarations (struct, enum)
                Stmt::TypeDecl(type_decl) => {
                    let frag_id = self.index_type_decl(type_decl, file_id)?;
                    frag_ids.push(frag_id);
                }

                // Enum declarations
                Stmt::EnumDecl(enum_decl) => {
                    let frag_id = self.index_enum_decl(enum_decl, file_id)?;
                    frag_ids.push(frag_id);
                }

                // Spec declarations
                Stmt::SpecDecl(spec_decl) => {
                    let frag_id = self.index_spec_decl(spec_decl, file_id)?;
                    frag_ids.push(frag_id);
                }

                // Ext (impl) declarations
                Stmt::Ext(ext) => {
                    let ext_frag_ids = self.index_ext_decl(ext, file_id)?;
                    frag_ids.extend(ext_frag_ids);
                }

                // Import statements (Phase 2: Track dependencies)
                Stmt::Use(use_stmt) => {
                    self.index_use_stmt(use_stmt, file_id)?;
                }

                // Storage declarations (let, mut, const)
                Stmt::Store(store) => {
                    let frag_id = self.index_store_decl(store, file_id)?;
                    frag_ids.push(frag_id);
                }

                // Tag declarations
                Stmt::Tag(tag) => {
                    let frag_id = self.index_tag_decl(tag, file_id)?;
                    frag_ids.push(frag_id);
                }

                // Alias declarations
                Stmt::Alias(alias) => {
                    self.index_alias_decl(alias, file_id)?;
                }

                // Type alias declarations
                Stmt::TypeAlias(type_alias) => {
                    self.index_type_alias_decl(type_alias, file_id)?;
                }

                // Union declarations
                Stmt::Union(union) => {
                    let frag_id = self.index_union_decl(union, file_id)?;
                    frag_ids.push(frag_id);
                }

                // Comments, empty lines, expressions (not declarations)
                Stmt::Comment(_) | Stmt::EmptyLine(_) | Stmt::Expr(_) => {
                    // Skip - these are not top-level declarations
                }

                // Control flow statements (shouldn't appear at top level)
                Stmt::If(_) | Stmt::For(_) | Stmt::Is(_) | Stmt::Break | Stmt::Return(_) => {
                    // These shouldn't appear at top level, but skip them if they do
                }

                // Block statements (shouldn't appear at top level)
                Stmt::Block(_) => {
                    // Skip - blocks are inside functions, not top-level
                }

                // Node statements (UI widget declarations)
                Stmt::Node(_node) => {
                    // For now, treat nodes as expressions (skip)
                    // Phase 2: Could index widget declarations
                }

                // On event handlers
                Stmt::OnEvents(on) => {
                    let frag_id = self.index_on_events(on, file_id)?;
                    frag_ids.push(frag_id);
                }
            }
        }

        Ok(frag_ids)
    }

    /// Index a function declaration
    fn index_function(&mut self, fn_decl: &Fn, file_id: FileId) -> Result<FragId, String> {
        let name = fn_decl.name.clone();

        // Get span information (Fn span is (line, col), not (line, col, pos))
        let span = self.extract_span_fn(fn_decl.span.as_ref())?;

        // Create fragment
        let frag_id = self.db.insert_fragment(
            name.clone(),
            file_id,
            span,
            FragKind::Function,
            Arc::new(fn_decl.clone()),
        );

        // Register symbol location (for LSP support)
        if let Some((line, col)) = fn_decl.span.as_ref() {
            let location = SymbolLocation::new(*line, *col, 0);
            let symbol_id = Sid::kid_of(&SID_PATH_GLOBAL, &name);
            self.db.define_symbol_location(symbol_id, location);
        }

        Ok(frag_id)
    }

    /// Index a type declaration (struct)
    fn index_type_decl(&mut self, _type_decl: &crate::ast::TypeDecl, file_id: FileId) -> Result<FragId, String> {
        // TODO: Implement type declaration indexing
        // Phase 1: Create placeholder fragment
        // Phase 2: Extract fields and methods
        let frag_id = FragId::new(file_id, 0);
        Ok(frag_id)
    }

    /// Index an enum declaration
    fn index_enum_decl(&mut self, _enum_decl: &crate::ast::EnumDecl, file_id: FileId) -> Result<FragId, String> {
        // TODO: Implement enum declaration indexing
        let frag_id = FragId::new(file_id, 0);
        Ok(frag_id)
    }

    /// Index a spec declaration
    fn index_spec_decl(&mut self, spec_decl: &SpecDecl, file_id: FileId) -> Result<FragId, String> {
        let name = spec_decl.name.clone();

        // SpecDecl doesn't have span, use default
        let span = FragSpan {
            offset: 0,
            length: 0,
            line: 1,
            column: 1,
        };

        let frag_id = self.db.insert_fragment(
            name.clone(),
            file_id,
            span,
            FragKind::Spec,
            // Note: SpecDecl doesn't implement Clone, so we can't wrap it in Arc
            // For now, we'll store a placeholder Fn - this needs to be fixed
            Arc::new(Fn::new(
                crate::ast::FnKind::Function,
                name.clone(),
                None,
                vec![],
                crate::ast::Body::new(),
                Type::Unknown,
            )),
        );

        // Note: Can't register symbol location since SpecDecl has no span
        // Phase 2: Add span to SpecDecl or track separately

        Ok(frag_id)
    }

    /// Index an ext (impl) declaration
    fn index_ext_decl(&mut self, _ext: &crate::ast::Ext, _file_id: FileId) -> Result<Vec<FragId>, String> {
        // TODO: Implement ext declaration indexing
        // Ext blocks can contain multiple methods
        // Phase 1: Return empty list
        Ok(vec![])
    }

    /// Index a use (import) statement
    ///
    /// Phase 2: Track file-level dependencies
    fn index_use_stmt(&mut self, use_stmt: &crate::ast::Use, file_id: FileId) -> Result<(), String> {
        // Phase 2: Only track Auto imports (not C or Rust)
        use crate::ast::UseKind;

        if !matches!(use_stmt.kind, UseKind::Auto) {
            return Ok(());  // Skip C and Rust imports
        }

        // Try to resolve import paths to FileIds
        let mut imported_files = Vec::new();

        // For each path in the use statement, try to find a matching file
        for path in &use_stmt.paths {
            // Convert path like "std::io" to potential file paths
            // Common patterns: "std/io.at", "std.io.at", etc.
            let path_str = path.as_ref();

            // Try different path patterns
            let candidates = vec![
                format!("{}.at", path_str.replace("::", "/")),
                format!("{}.at", path_str.replace("::", ".")),
                format!("{}/index.at", path_str.replace("::", "/")),
            ];

            // Check if any candidate exists in the database
            for candidate in candidates {
                if let Some(imported_file_id) = self.db.get_file_id_by_path(&candidate) {
                    imported_files.push(imported_file_id);
                    break;  // Found a match, stop trying other patterns
                }
            }
        }

        // Add dependencies to the graph
        if !imported_files.is_empty() {
            self.db.dep_graph_mut().add_file_import(file_id, imported_files);
        }

        Ok(())
    }

    /// Index a store declaration (let, mut, const)
    fn index_store_decl(&mut self, _store: &crate::ast::Store, file_id: FileId) -> Result<FragId, String> {
        // Global constants are fragments, local variables are not
        // Phase 1: Treat all stores as potential fragments
        let frag_id = FragId::new(file_id, 0);
        Ok(frag_id)
    }

    /// Index a tag declaration
    fn index_tag_decl(&mut self, _tag: &crate::ast::Tag, file_id: FileId) -> Result<FragId, String> {
        // TODO: Implement tag declaration indexing
        let frag_id = FragId::new(file_id, 0);
        Ok(frag_id)
    }

    /// Index an alias declaration
    fn index_alias_decl(&mut self, _alias: &crate::ast::Alias, _file_id: FileId) -> Result<(), String> {
        // Aliases are not fragments (they're symbol table entries)
        Ok(())
    }

    /// Index a type alias declaration
    fn index_type_alias_decl(&mut self, _type_alias: &crate::ast::TypeAlias, _file_id: FileId) -> Result<(), String> {
        // Type aliases are not fragments (they're symbol table entries)
        Ok(())
    }

    /// Index a union declaration
    fn index_union_decl(&mut self, _union: &crate::ast::Union, file_id: FileId) -> Result<FragId, String> {
        // TODO: Implement union declaration indexing
        let frag_id = FragId::new(file_id, 0);
        Ok(frag_id)
    }

    /// Index an on event handler
    fn index_on_events(&mut self, _on: &crate::ast::OnEvents, file_id: FileId) -> Result<FragId, String> {
        // TODO: Implement on event handler indexing
        let frag_id = FragId::new(file_id, 0);
        Ok(frag_id)
    }

    /// Extract span information from optional (line, col, pos) tuple
    #[allow(dead_code)]  // Phase 2: Will be used for other declaration types
    fn extract_span(&self, span: Option<&(usize, usize, usize)>) -> Result<FragSpan, String> {
        if let Some((line, col, pos)) = span {
            Ok(FragSpan {
                offset: *pos,
                length: 0, // TODO: Calculate length from source
                line: *line,
                column: *col,
            })
        } else {
            // Default span if not available
            Ok(FragSpan {
                offset: 0,
                length: 0,
                line: 1,
                column: 1,
            })
        }
    }

    /// Extract span information from Fn's optional (line, col) tuple
    /// Note: Fn uses (line, col) not (line, col, pos)
    fn extract_span_fn(&self, span: Option<&(usize, usize)>) -> Result<FragSpan, String> {
        if let Some((line, col)) = span {
            Ok(FragSpan {
                offset: 0, // Fn doesn't track offset
                length: 0, // TODO: Calculate length from source
                line: *line,
                column: *col,
            })
        } else {
            // Default span if not available
            Ok(FragSpan {
                offset: 0,
                length: 0,
                line: 1,
                column: 1,
            })
        }
    }

    // =========================================================================
    // Phase 2.3: Incremental Re-Indexing
    // =========================================================================

    /// Re-index a file that has changed
    ///
    /// This method implements incremental re-indexing by:
    /// 1. Checking if the file actually changed (hash comparison)
    /// 2. If unchanged, returning early (no work needed)
    /// 3. If changed:
    ///    - Updating the source code
    ///    - Clearing old fragments
    ///    - Re-parsing and re-indexing
    ///    - Updating the hash
    ///    - Marking dependents as dirty
    ///
    /// # Arguments
    ///
    /// * `file_id` - The file to re-index
    /// * `new_code` - The new source code
    ///
    /// # Returns
    ///
    /// A list of fragment IDs that were created, or empty if no change detected.
    pub fn reindex_file(
        &mut self,
        file_id: FileId,
        new_code: &str,
    ) -> AutoResult<Vec<FragId>> {
        // Get file path before any mutations
        let file_path = self.db.get_file_path(file_id)
            .map(|p| p.to_string())
            .unwrap_or_else(|| "unknown.at".to_string());

        // Update source first (needed for hash computation)
        self.db.insert_source(&file_path, AutoStr::from(new_code));

        // Check if file actually changed
        if !self.db.is_file_dirty(file_id) {
            return Ok(vec![]);  // No change, skip re-indexing
        }

        // Clear old fragments for this file
        self.db.clear_file_fragments(file_id);

        // Clear symbol locations for this file (Phase 2: more granular clearing)
        // For now, we'll keep all symbol locations

        // Clear cache for this file (types, bytecodes)
        // Note: Fragments have been cleared, so cache entries will be orphaned
        // Phase 2: More granular cache invalidation

        // Re-parse the source code
        let scope = Rc::new(std::cell::RefCell::new(crate::universe::Universe::new()));
        let mut parser = Parser::new(new_code, scope.clone());
        let ast = parser.parse()
            .map_err(|e| AutoError::Msg(format!("Parse error during re-indexing: {}", e)))?;

        // Re-index the AST
        let frag_ids = self.index_ast(&ast, file_id)
            .map_err(|e| AutoError::Msg(format!("Index error during re-indexing: {}", e)))?;

        // Update the hash
        self.db.hash_file(file_id);

        // Mark dependents as dirty (they need recompilation)
        self.db.propagate_dirty(file_id);

        // Clear dirty flag for this file (we just re-indexed it)
        self.db.clear_dirty_flag(file_id);

        Ok(frag_ids)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Body, Fn, FnKind};
    use crate::database::Database;
    use auto_val::AutoStr;

    #[test]
    fn test_indexer_new() {
        let mut db = Database::new();
        let _indexer = Indexer::new(&mut db);
        // Indexer created successfully
    }

    #[test]
    fn test_index_function() {
        let mut db = Database::new();
        let file_id = db.insert_source("test.at", AutoStr::from("fn main() int { 42 }"));

        // Create a simple function AST
        let fn_decl = Fn::new(
            FnKind::Function,
            AutoStr::from("main"),
            None,
            vec![],
            Body::new(),
            crate::ast::Type::Int,
        );

        // Index the function
        let mut indexer = Indexer::new(&mut db);
        let result = indexer.index_function(&fn_decl, file_id);

        assert!(result.is_ok());
        let frag_id = result.unwrap();

        // Verify fragment was created
        let frag = db.get_fragment(&frag_id);
        assert!(frag.is_some());

        // Verify fragment metadata
        let meta = db.get_fragment_meta(&frag_id);
        assert!(meta.is_some());
        let meta = meta.unwrap();
        assert_eq!(meta.name.as_ref(), "main");
        assert_eq!(meta.file_id, file_id);
        assert_eq!(meta.kind, FragKind::Function);
    }

    #[test]
    fn test_index_ast_empty() {
        let mut db = Database::new();
        let file_id = db.insert_source("test.at", AutoStr::from(""));

        let ast = Code::new();
        let mut indexer = Indexer::new(&mut db);
        let result = indexer.index_ast(&ast, file_id);

        assert!(result.is_ok());
        let frag_ids = result.unwrap();
        assert_eq!(frag_ids.len(), 0);
    }

    #[test]
    fn test_index_ast_with_function() {
        let mut db = Database::new();
        let file_id = db.insert_source("test.at", AutoStr::from("fn main() int { 42 }"));

        // Create AST with one function
        let fn_decl = Fn::new(
            FnKind::Function,
            AutoStr::from("main"),
            None,
            vec![],
            Body::new(),
            crate::ast::Type::Int,
        );

        let mut ast = Code::new();
        ast.stmts.push(crate::ast::Stmt::Fn(fn_decl));

        // Index the AST
        let mut indexer = Indexer::new(&mut db);
        let result = indexer.index_ast(&ast, file_id);

        assert!(result.is_ok());
        let frag_ids = result.unwrap();
        assert_eq!(frag_ids.len(), 1);

        // Verify the fragment
        let frag = db.get_fragment(&frag_ids[0]);
        assert!(frag.is_some());
    }

    #[test]
    fn test_index_use_stmt_dependency() {
        let mut db = Database::new();

        // Insert an imported file first (dependency)
        let imported_file = db.insert_source("std/io.at", AutoStr::from("fn say() void {}"));

        // Insert main file that imports std::io
        let main_file = db.insert_source("main.at", AutoStr::from("use std::io\nfn main() int { 42 }"));

        // Create AST with use statement
        let use_stmt = crate::ast::Use {
            kind: crate::ast::UseKind::Auto,
            paths: vec![AutoStr::from("std::io")],
            items: vec![],
        };

        let mut ast = Code::new();
        ast.stmts.push(crate::ast::Stmt::Use(use_stmt));

        // Index the AST
        let mut indexer = Indexer::new(&mut db);
        let result = indexer.index_ast(&ast, main_file);

        assert!(result.is_ok());

        // Verify dependency was tracked
        let imports = db.dep_graph().get_file_imports(main_file);
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0], imported_file);

        // Verify reverse dependency
        let dependents = db.dep_graph().get_file_dependents(imported_file);
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0], main_file);
    }

    #[test]
    fn test_reindex_file_unchanged() {
        let mut db = Database::new();
        let file_id = db.insert_source("test.at", AutoStr::from("fn main() int { 42 }"));

        // Hash the original file
        db.hash_file(file_id);

        // Re-index with same content
        let mut indexer = Indexer::new(&mut db);
        let result = indexer.reindex_file(file_id, "fn main() int { 42 }");

        assert!(result.is_ok());
        let frag_ids = result.unwrap();
        assert_eq!(frag_ids.len(), 0);  // No change detected, skipped

        // File should not be marked as dirty
        assert!(!db.is_marked_dirty(file_id));
    }

    #[test]
    fn test_reindex_file_changed() {
        let mut db = Database::new();
        let file_id = db.insert_source("test.at", AutoStr::from("fn main() int { 42 }"));

        // Hash the original file
        db.hash_file(file_id);

        // Re-index with different content
        let mut indexer = Indexer::new(&mut db);
        let new_code = "fn main() int { 100 }\nfn foo() int { 1 }";
        let result = indexer.reindex_file(file_id, new_code);

        assert!(result.is_ok());
        let frag_ids = result.unwrap();
        assert_eq!(frag_ids.len(), 2);  // Two functions indexed

        // File should not be marked as dirty (we just re-indexed it)
        assert!(!db.is_marked_dirty(file_id));

        // Hash should be updated
        assert!(db.get_file_hash(file_id).is_some());
    }

    #[test]
    fn test_reindex_file_propagates_dirty() {
        let mut db = Database::new();

        // Set up dependency: main imports lib
        let lib_file = db.insert_source("lib.at", AutoStr::from("fn lib_fn() int { 1 }"));
        let main_file = db.insert_source("main.at", AutoStr::from("fn main() int { 42 }"));
        db.dep_graph_mut().add_file_import(main_file, vec![lib_file]);

        // Hash both files
        db.hash_file(lib_file);
        db.hash_file(main_file);

        // Re-index lib_file with changed content
        let mut indexer = Indexer::new(&mut db);
        let result = indexer.reindex_file(lib_file, "fn lib_fn() int { 999 }");

        assert!(result.is_ok());

        // main_file should be marked dirty (it depends on lib_file)
        assert!(db.is_marked_dirty(main_file));

        // lib_file should NOT be marked dirty (we just re-indexed it)
        assert!(!db.is_marked_dirty(lib_file));
    }
}
