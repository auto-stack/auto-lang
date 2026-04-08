// Plan 131 Integration Test: Full Module Resolution Pipeline
//
// Tests that parsing, ModulePath construction, and file resolution work end-to-end.
// This validates the complete import syntax and resolution system:
// - `use db` → same directory
// - `use super.db` → parent directory
// - `use pac.db` → package root
// - `use pac.api.handlers` → deep path from root

use auto_lang::ast::{ModulePath, PathPrefix, Stmt, Use};
use auto_lang::parser::Parser;
use auto_lang::resolver::FilesystemResolver;
use std::path::PathBuf;
use tempfile::TempDir;

/// Setup the test project directory structure in a unique temp dir.
/// Returns (TempDir, src_path) — TempDir must be kept alive for duration of test.
fn setup_test_project() -> (TempDir, PathBuf) {
    let tmp = TempDir::new().expect("Failed to create temp dir");
    let src = tmp.path().join("src");

    // Create directory structure
    std::fs::create_dir_all(src.join("api")).unwrap();

    // src/db.at
    std::fs::write(src.join("db.at"), r#"
fn connect() { 1 }
fn query(sql str) { 0 }
"#).unwrap();

    // src/utils.at
    std::fs::write(src.join("utils.at"), r#"
fn helper() { 42 }
"#).unwrap();

    // src/main.at
    std::fs::write(src.join("main.at"), r#"
use pac.db
use pac.api.handlers
"#).unwrap();

    // src/api/mod.at (directory module)
    std::fs::write(src.join("api/mod.at"), r#"
fn api_init() { 1 }
"#).unwrap();

    // src/api/handlers.at
    std::fs::write(src.join("api/handlers.at"), r#"
use pac.db
use super.utils
fn handle_request() { 0 }
"#).unwrap();

    (tmp, src)
}

#[test]
fn test_integration_pac_import_parsing_and_resolution() {
    let (_tmp, src_root) = setup_test_project();

    // Parse handlers.at
    let handlers_path = src_root.join("api/handlers.at");
    let source = std::fs::read_to_string(&handlers_path)
        .expect("Test project file not found");

    let mut parser = Parser::from(source.as_str());
    let ast = parser.parse().expect("Failed to parse handlers.at");

    // Find the use statements
    let use_stmts: Vec<&Use> = ast
        .stmts
        .iter()
        .filter_map(|s| {
            if let Stmt::Use(u) = s {
                Some(u)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(use_stmts.len(), 2, "Should have 2 use statements");

    // Verify pac.db import
    let pac_db = use_stmts
        .iter()
        .find(|u| {
            u.module_path
                .as_ref()
                .map(|mp| mp.display() == "pac.db")
                .unwrap_or(false)
        })
        .expect("pac.db import not found");
    assert!(
        matches!(pac_db.module_path.as_ref().unwrap().prefix, PathPrefix::Pac),
        "pac.db should have Pac prefix"
    );

    // Verify super.utils import
    let super_utils = use_stmts
        .iter()
        .find(|u| {
            u.module_path
                .as_ref()
                .map(|mp| mp.display() == "super.utils")
                .unwrap_or(false)
        })
        .expect("super.utils import not found");
    assert!(
        matches!(
            super_utils.module_path.as_ref().unwrap().prefix,
            PathPrefix::Super
        ),
        "super.utils should have Super prefix"
    );

    // Resolve pac.db to file path
    let resolver = FilesystemResolver::with_package_root(src_root.clone());
    let pac_path = pac_db.module_path.as_ref().unwrap();
    let resolved = resolver
        .resolve_with_prefix(pac_path, handlers_path.clone())
        .expect("Failed to resolve pac.db");
    assert_eq!(
        resolved, src_root.join("db.at"),
        "pac.db should resolve to src/db.at"
    );

    // Resolve super.utils to file path
    let super_path = super_utils.module_path.as_ref().unwrap();
    let resolved = resolver
        .resolve_with_prefix(super_path, handlers_path)
        .expect("Failed to resolve super.utils");
    assert_eq!(
        resolved, src_root.join("utils.at"),
        "super.utils should resolve to src/utils.at"
    );
}

#[test]
fn test_integration_main_imports() {
    let (_tmp, src_root) = setup_test_project();
    let main_path = src_root.join("main.at");

    let source =
        std::fs::read_to_string(&main_path).expect("Test project file not found");

    let mut parser = Parser::from(source.as_str());
    let ast = parser.parse().expect("Failed to parse main.at");

    let use_stmts: Vec<&Use> = ast
        .stmts
        .iter()
        .filter_map(|s| {
            if let Stmt::Use(u) = s {
                Some(u)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(use_stmts.len(), 2, "main.at should have 2 imports");

    // Both should be pac imports
    for u in &use_stmts {
        assert!(
            matches!(
                u.module_path.as_ref().unwrap().prefix,
                PathPrefix::Pac
            ),
            "All imports in main.at should use pac prefix"
        );
    }
}

#[test]
fn test_integration_deep_module_path() {
    let (_tmp, src_root) = setup_test_project();

    // Test resolving pac.api.handlers from main.at
    let resolver = FilesystemResolver::with_package_root(src_root.clone());
    let module_path = ModulePath::pac(vec!["api".into(), "handlers".into()]);
    let current_file = src_root.join("main.at");

    let resolved = resolver
        .resolve_with_prefix(&module_path, current_file)
        .expect("Failed to resolve pac.api.handlers");

    assert_eq!(
        resolved,
        src_root.join("api/handlers.at"),
        "pac.api.handlers should resolve to src/api/handlers.at"
    );
}

#[test]
fn test_integration_module_directory_vs_file() {
    let (_tmp, src_root) = setup_test_project();

    // Test resolving pac.api (should resolve to api/mod.at, not api.at)
    let resolver = FilesystemResolver::with_package_root(src_root.clone());
    let module_path = ModulePath::pac(vec!["api".into()]);
    let current_file = src_root.join("main.at");

    let resolved = resolver
        .resolve_with_prefix(&module_path, current_file)
        .expect("Failed to resolve pac.api");

    assert_eq!(
        resolved,
        src_root.join("api/mod.at"),
        "pac.api should resolve to src/api/mod.at (directory module)"
    );
}

#[test]
fn test_integration_module_not_found() {
    let (_tmp, src_root) = setup_test_project();

    // Test resolving non-existent module
    let resolver = FilesystemResolver::with_package_root(src_root.clone());
    let module_path = ModulePath::pac(vec!["nonexistent".into()]);
    let current_file = src_root.join("main.at");

    let result = resolver.resolve_with_prefix(&module_path, current_file);

    assert!(
        result.is_err(),
        "Should fail to resolve non-existent module"
    );
    assert!(
        result.unwrap_err().contains("not found"),
        "Error message should mention 'not found'"
    );
}
