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

/// Get the project root directory (workspace root)
fn project_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is the crate directory (crates/auto-lang)
    // We need to go up 2 levels to get the workspace root
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("Failed to get project root")
        .to_path_buf()
}

/// Get the test project root
fn test_project_root() -> PathBuf {
    project_root().join("tmp/plan131_test_project")
}

/// Get the test project src directory
fn test_project_src() -> PathBuf {
    test_project_root().join("src")
}

/// Plan 131 Integration Test: Full module resolution pipeline
/// Tests that parsing, ModulePath construction, and file resolution work end-to-end
#[test]
fn test_integration_pac_import_parsing_and_resolution() {
    let src_root = test_project_src();

    // Parse handlers.at
    let handlers_path = src_root.join("api/handlers.at");
    let source = std::fs::read_to_string(&handlers_path)
        .expect("Test project file not found - run setup first");

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
    let src_root = test_project_src();
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
    let src_root = test_project_src();

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
    let src_root = test_project_src();

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
    let src_root = test_project_src();

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
