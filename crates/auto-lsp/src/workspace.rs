//! Workspace-level state management for cross-file resolution
//!
//! This module builds a combined view of all modules in the workspace
//! by resolving `use` statements and merging their TypeStores.

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use auto_lang::ast::ModulePath;
use auto_lang::resolver::ModuleResolver;
use auto_lang::database::Database;
use auto_lang::indexer::Indexer;
use auto_lang::parser::Parser;
use auto_lang::resolver::FilesystemResolver;
use auto_lang::types::TypeStore;
use auto_lang::use_scanner::scan_use_statements;
use auto_val::AutoStr;

/// Build workspace state for a given document
///
/// Parses the document, scans its `use` statements, resolves them to file paths,
/// parses dependency modules, and merges all symbols into a unified TypeStore
/// and Database.
pub fn build_workspace_state(
    content: &str,
    document_path: &Path,
    resolver: &FilesystemResolver,
) -> WorkspaceState {
    let mut workspace_db = Database::new();
    let mut workspace_store = TypeStore::new();

    // Insert the current document
    let doc_path_str = document_path.to_string_lossy().to_string();
    let doc_file_id = workspace_db.insert_source(&doc_path_str, AutoStr::from(content));

    // Parse current document and register its symbols
    let mut parser = Parser::from(content);
    if let Ok(ast) = parser.parse() {
        // Register symbols in workspace store
        register_ast_symbols(&ast, &mut workspace_store);

        // Index current document
        let mut indexer = Indexer::new(&mut workspace_db);
        let _ = indexer.index_ast(&ast, doc_file_id);

        // Scan and resolve use statements
        let use_statements = scan_use_statements(content);
        for use_stmt in use_statements {
            if use_stmt.is_c_import || use_stmt.is_rust_import || use_stmt.is_python_import {
                continue;
            }

            // Resolve module path to file path
            let resolved = resolve_use_statement(resolver, &use_stmt.module, document_path);

            if let Some(module_file_path) = resolved {
                if let Ok(module_source) = std::fs::read_to_string(&module_file_path) {
                    let module_path_str = module_file_path.to_string_lossy().to_string();
                    let module_file_id = workspace_db.insert_source(&module_path_str, AutoStr::from(&module_source));

                    let mut module_parser = Parser::from(module_source.as_str());
                    if let Ok(module_ast) = module_parser.parse() {
                        // Register module symbols
                        register_ast_symbols(&module_ast, &mut workspace_store);

                        // Index module
                        let mut module_indexer = Indexer::new(&mut workspace_db);
                        let _ = module_indexer.index_ast(&module_ast, module_file_id);
                    }
                }
            }
        }
    }

    WorkspaceState {
        db: workspace_db,
        type_store: Arc::new(RwLock::new(workspace_store)),
    }
}

/// Register all top-level symbols from an AST into a TypeStore
fn register_ast_symbols(ast: &auto_lang::ast::Code, store: &mut TypeStore) {
    for stmt in &ast.stmts {
        match stmt {
            auto_lang::ast::Stmt::Fn(fn_decl) => store.register_fn_decl(fn_decl),
            auto_lang::ast::Stmt::TypeDecl(type_decl) => store.register_type_decl(type_decl),
            auto_lang::ast::Stmt::SpecDecl(spec_decl) => store.register_spec_decl(spec_decl),
            auto_lang::ast::Stmt::Ext(ext) => store.register_ext_methods(ext),
            _ => {}
        }
    }
}

/// Resolve a use statement module string to a file path
fn resolve_use_statement(
    resolver: &FilesystemResolver,
    module: &str,
    current_file: &Path,
) -> Option<PathBuf> {
    // Handle std.* imports using the legacy resolve method
    if module.starts_with("std.") {
        return resolver.resolve(module).ok();
    }

    // Handle pac.* imports
    if let Some(rest) = module.strip_prefix("pac.") {
        let segments: Vec<AutoStr> = rest.split('.').map(AutoStr::from).collect();
        let module_path = ModulePath::pac(segments);
        return resolver.resolve_with_prefix(&module_path, current_file.to_path_buf()).ok();
    }

    // Handle super.* imports
    if let Some(rest) = module.strip_prefix("super.") {
        let segments: Vec<AutoStr> = rest.split('.').map(AutoStr::from).collect();
        let module_path = ModulePath::super_path(segments);
        return resolver.resolve_with_prefix(&module_path, current_file.to_path_buf()).ok();
    }

    // Handle local imports (no prefix)
    let segments: Vec<AutoStr> = module.split('.').map(AutoStr::from).collect();
    let module_path = ModulePath::local(segments);
    resolver.resolve_with_prefix(&module_path, current_file.to_path_buf()).ok()
}

/// Workspace state containing merged symbols from all resolved modules
pub struct WorkspaceState {
    pub db: Database,
    pub type_store: Arc<RwLock<TypeStore>>,
}

/// Create a FilesystemResolver for the given workspace root
pub fn create_resolver(workspace_root: &Path) -> FilesystemResolver {
    let std_root = find_std_root(workspace_root);
    let mut resolver = FilesystemResolver::new(std_root);
    resolver.add_search_path(workspace_root.to_path_buf());
    resolver
}

/// Find the stdlib root path
fn find_std_root(workspace_root: &Path) -> PathBuf {
    // Check if stdlib exists relative to workspace
    let local_std = workspace_root.join("stdlib").join("auto");
    if local_std.exists() {
        return local_std;
    }

    // Check if we're inside the auto-lang repo itself
    let repo_std = workspace_root.join("..").join("stdlib").join("auto");
    if repo_std.exists() {
        return repo_std.canonicalize().unwrap_or(repo_std);
    }

    // Fallback: just return workspace root
    workspace_root.to_path_buf()
}
