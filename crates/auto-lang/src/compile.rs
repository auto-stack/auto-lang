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
use std::sync::Arc;

/// Compilation session using the new AIE architecture
///
/// A compilation session manages a Database and provides methods to
/// compile source code with incremental support.
pub struct CompileSession {
    db: Database,
}

impl CompileSession {
    /// Create a new compilation session
    pub fn new() -> Self {
        Self {
            db: Database::new(),
        }
    }

    /// Get the underlying database (for advanced usage)
    pub fn database(&self) -> &Database {
        &self.db
    }

    /// Get mutable access to the database (for advanced usage)
    pub fn database_mut(&mut self) -> &mut Database {
        &mut self.db
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
        let file_id = self.db.insert_source(path, AutoStr::from(source));

        // Parse source code to AST
        // Note: Phase 1.2 will make parser pure, but for now we use existing parser
        let scope = Rc::new(std::cell::RefCell::new(crate::universe::Universe::new()));
        let mut parser = Parser::new(source, scope.clone());
        let ast = parser.parse()
            .map_err(|e| AutoError::Msg(format!("Parse error: {}", e)))?;

        // Index AST into database
        let mut indexer = Indexer::new(&mut self.db);
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
        for file_id in self.db.get_files() {
            for frag_id in self.db.get_fragments_in_file(file_id) {
                if let Some(meta) = self.db.get_fragment_meta(&frag_id) {
                    if meta.name.as_ref() == name {
                        return self.db.get_fragment(&frag_id);
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
    pub fn get_symbol_location(&self, name: &str) -> Option<&SymbolLocation> {
        let symbol_id = Sid::kid_of(&SID_PATH_GLOBAL, name);
        self.db.get_symbol_location(&symbol_id)
    }

    /// List all functions in the database
    ///
    /// # Returns
    ///
    /// A list of function names.
    pub fn list_functions(&self) -> Vec<String> {
        let mut functions = Vec::new();

        for file_id in self.db.get_files() {
            for frag_id in self.db.get_fragments_in_file(file_id) {
                if let Some(meta) = self.db.get_fragment_meta(&frag_id) {
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
        self.db = Database::new();
    }

    /// Get statistics about the database
    pub fn stats(&self) -> CompileStats {
        let mut total_frags = 0;
        let mut total_functions = 0;
        let mut total_specs = 0;

        for file_id in self.db.get_files() {
            let frags = self.db.get_fragments_in_file(file_id);
            total_frags += frags.len();

            for frag_id in &frags {
                if let Some(meta) = self.db.get_fragment_meta(frag_id) {
                    match meta.kind {
                        crate::database::FragKind::Function => total_functions += 1,
                        crate::database::FragKind::Spec => total_specs += 1,
                        _ => {}
                    }
                }
            }
        }

        CompileStats {
            total_files: self.db.get_files().len(),
            total_frags,
            total_functions,
            total_specs,
        }
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
}
