# Module Path Syntax Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement module path syntax with `pac`, `super`, and dependency imports for AutoLang.

**Architecture:** Extend existing `ModuleResolver` trait and `Use` AST structure. Add `ModulePath` type to distinguish between relative, package, and dependency paths. Parser will produce structured path info, resolver will find files.

**Tech Stack:** Rust, existing resolver.rs, parser.rs, use_scanner.rs

---

## Task 1: Add ModulePath Type to AST

**Files:**
- Create: `crates/auto-lang/src/ast/module_path.rs`
- Modify: `crates/auto-lang/src/ast/mod.rs`

**Step 1: Write the failing test**

Create `crates/auto-lang/src/ast/module_path.rs`:

```rust
//! Plan 131: Module Path Syntax
//!
//! Represents the different ways to reference a module:
//! - `db` → same directory
//! - `super.db` → parent directory
//! - `pac.db` → package root
//! - `pac.api.handlers` → deep path from root
//! - `database.connection` → from dependency

use auto_val::AutoStr;

/// The prefix of a module path
#[derive(Debug, Clone, PartialEq)]
pub enum PathPrefix {
    /// No prefix - same directory: `use db`
    None,
    /// `super.` prefix - parent directory: `use super.db`
    Super,
    /// `pac.` prefix - package root: `use pac.db`
    Pac,
    /// Dependency name - from declared dep: `use database.connection`
    Dep(AutoStr),
}

/// A fully parsed module path
#[derive(Debug, Clone, PartialEq)]
pub struct ModulePath {
    /// The prefix (None, Super, Pac, or Dep name)
    pub prefix: PathPrefix,
    /// The path segments (e.g., ["api", "handlers"] for "pac.api.handlers")
    pub segments: Vec<AutoStr>,
    /// Symbols to import (after `:`)
    pub items: Vec<AutoStr>,
}

impl ModulePath {
    /// Create a new module path
    pub fn new(prefix: PathPrefix, segments: Vec<AutoStr>, items: Vec<AutoStr>) -> Self {
        Self { prefix, segments, items }
    }

    /// Create a simple path (same directory)
    pub fn local(segments: Vec<AutoStr>) -> Self {
        Self::new(PathPrefix::None, segments, Vec::new())
    }

    /// Create a super path (parent directory)
    pub fn super_path(segments: Vec<AutoStr>) -> Self {
        Self::new(PathPrefix::Super, segments, Vec::new())
    }

    /// Create a package path (from root)
    pub fn pac(segments: Vec<AutoStr>) -> Self {
        Self::new(PathPrefix::Pac, segments, Vec::new())
    }

    /// Create a dependency path
    pub fn dep(dep_name: AutoStr, segments: Vec<AutoStr>) -> Self {
        Self::new(PathPrefix::Dep(dep_name), segments, Vec::new())
    }

    /// Add import items
    pub fn with_items(mut self, items: Vec<AutoStr>) -> Self {
        self.items = items;
        self
    }

    /// Get the full path as a string (for display)
    pub fn display(&self) -> String {
        let mut result = String::new();
        match &self.prefix {
            PathPrefix::None => {}
            PathPrefix::Super => result.push_str("super."),
            PathPrefix::Pac => result.push_str("pac."),
            PathPrefix::Dep(name) => {
                result.push_str(name.as_str());
                result.push('.');
            }
        }
        result.push_str(&self.segments.join("."));
        result
    }
}

impl std::fmt::Display for ModulePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_path() {
        let path = ModulePath::local(vec!["db".into()]);
        assert_eq!(path.display(), "db");
        assert_eq!(path.prefix, PathPrefix::None);
    }

    #[test]
    fn test_super_path() {
        let path = ModulePath::super_path(vec!["db".into()]);
        assert_eq!(path.display(), "super.db");
        assert_eq!(path.prefix, PathPrefix::Super);
    }

    #[test]
    fn test_pac_path() {
        let path = ModulePath::pac(vec!["api".into(), "handlers".into()]);
        assert_eq!(path.display(), "pac.api.handlers");
        assert_eq!(path.prefix, PathPrefix::Pac);
    }

    #[test]
    fn test_dep_path() {
        let path = ModulePath::dep("database".into(), vec!["connection".into()]);
        assert_eq!(path.display(), "database.connection");
        assert_eq!(path.prefix, PathPrefix::Dep("database".into()));
    }

    #[test]
    fn test_with_items() {
        let path = ModulePath::local(vec!["db".into()])
            .with_items(vec!["load".into(), "save".into()]);
        assert_eq!(path.items, vec!["load", "save"]);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang module_path::tests --no-run`
Expected: Compilation succeeds (no external dependencies yet)

**Step 3: Add module to ast/mod.rs**

Add to `crates/auto-lang/src/ast/mod.rs`:
```rust
pub mod module_path;
pub use module_path::{ModulePath, PathPrefix};
```

**Step 4: Run tests**

Run: `cargo test -p auto-lang module_path::tests`
Expected: All 5 tests pass

**Step 5: Commit**

```bash
git add crates/auto-lang/src/ast/module_path.rs crates/auto-lang/src/ast/mod.rs
git commit -m "feat(ast): add ModulePath type for Plan 131"
```

---

## Task 2: Add `pac` and `super` Keywords to Lexer

**Files:**
- Modify: `crates/auto-lang/src/token.rs`
- Modify: `crates/auto-lang/src/lexer.rs`

**Step 1: Write the failing test**

Add to `crates/auto-lang/src/lexer.rs` tests:

```rust
#[test]
fn test_pac_keyword() {
    let code = "use pac.db";
    let tokens = parse_token_strings(code);
    assert_eq!(tokens, "<use><pac><.><ident:db>");
}

#[test]
fn test_super_keyword() {
    let code = "use super.db";
    let tokens = parse_token_strings(code);
    assert_eq!(tokens, "<use><super><.><ident:db>");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang test_pac_keyword test_super_keyword`
Expected: FAIL - `pac` and `super` not recognized as keywords

**Step 3: Add keywords to token.rs**

In `crates/auto-lang/src/token.rs`, add to `TokenKind` enum (after `Use`):

```rust
    #[keyword = "pac"]
    Pac,

    #[keyword = "super"]
    Super,
```

**Step 4: Run tests**

Run: `cargo test -p auto-lang test_pac_keyword test_super_keyword`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/src/token.rs crates/auto-lang/src/lexer.rs
git commit -m "feat(lexer): add pac and super keywords for Plan 131"
```

---

## Task 3: Update Use Struct to Include ModulePath

**Files:**
- Modify: `crates/auto-lang/src/ast/use_.rs`

**Step 1: Write the failing test**

Add to `crates/auto-lang/src/ast/use_.rs`:

```rust
#[cfg(test)]
mod plan131_tests {
    use super::*;

    #[test]
    fn test_use_with_pac_prefix() {
        let use_stmt = Use {
            kind: UseKind::Auto,
            module_path: Some(ModulePath::pac(vec!["db".into()])),
            paths: vec![],
            items: vec![],
        };
        assert_eq!(use_stmt.module_path.as_ref().unwrap().display(), "pac.db");
    }

    #[test]
    fn test_use_with_super_prefix() {
        let use_stmt = Use {
            kind: UseKind::Auto,
            module_path: Some(ModulePath::super_path(vec!["utils".into()])),
            paths: vec![],
            items: vec![],
        };
        assert_eq!(use_stmt.module_path.as_ref().unwrap().display(), "super.utils");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang plan131_tests`
Expected: FAIL - `module_path` field does not exist

**Step 3: Update Use struct**

Modify `crates/auto-lang/src/ast/use_.rs`:

```rust
use crate::ast::AtomWriter;
use crate::ast::module_path::ModulePath;
use auto_val::AutoStr;
use std::{fmt, io as stdio};

#[derive(Debug, Clone)]
pub enum UseKind {
    Auto,
    C,
    Rust,
}

#[derive(Debug, Clone)]
pub struct Use {
    pub kind: UseKind,
    /// Plan 131: Structured module path (new syntax)
    pub module_path: Option<ModulePath>,
    /// Legacy: dotted path segments (for backward compat)
    pub paths: Vec<AutoStr>,
    /// Symbols to import (after `:`)
    pub items: Vec<AutoStr>,
}
```

Also update `ToNode` and `AtomWriter` implementations to handle `module_path`.

**Step 4: Run tests**

Run: `cargo test -p auto-lang plan131_tests`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/src/ast/use_.rs
git commit -m "feat(ast): add module_path field to Use struct"
```

---

## Task 4: Update Parser for New Module Path Syntax

**Files:**
- Modify: `crates/auto-lang/src/parser.rs`

**Step 1: Write the failing test**

Add to parser tests:

```rust
#[test]
fn test_parse_pac_import() {
    let code = "use pac.db";
    let ast = parse(code).unwrap();
    assert_eq!(ast.stmts.len(), 1);
    // Verify it parsed as pac.db
}

#[test]
fn test_parse_super_import() {
    let code = "use super.utils";
    let ast = parse(code).unwrap();
    assert_eq!(ast.stmts.len(), 1);
    // Verify it parsed as super.utils
}

#[test]
fn test_parse_dep_import() {
    let code = "use database.connection";
    let ast = parse(code).unwrap();
    assert_eq!(ast.stmts.len(), 1);
    // Verify it parsed as dep.database.connection
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang test_parse_pac_import`
Expected: FAIL - parser doesn't recognize `pac` as special

**Step 3: Update use_stmt() in parser**

In `crates/auto-lang/src/parser.rs`, modify `use_stmt()`:

```rust
pub fn use_stmt(&mut self) -> AutoResult<Stmt> {
    self.next(); // skip 'use'

    // Plan 131: Check for pac. or super. prefix
    let prefix = if self.is_kind(TokenKind::Pac) {
        self.next(); // skip 'pac'
        self.expect(TokenKind::Dot)?;
        PathPrefix::Pac
    } else if self.is_kind(TokenKind::Super) {
        self.next(); // skip 'super'
        self.expect(TokenKind::Dot)?;
        PathPrefix::Super
    } else {
        PathPrefix::None
    };

    // Parse path segments
    let mut segments = Vec::new();
    let first = self.expect_ident_str()?;
    segments.push(first.into());

    while self.is_kind(TokenKind::Dot) {
        self.next(); // skip '.'
        let segment = self.expect_ident_str()?;
        segments.push(segment.into());
    }

    // Parse import items (after ':')
    let items = self.parse_use_items()?;

    // Plan 131: Check if first segment is a dependency name
    let module_path = if prefix == PathPrefix::None && segments.len() > 1 {
        // Could be: dep.module or just nested path
        // For now, treat as local path
        // TODO: Check against declared dependencies
        Some(ModulePath::new(prefix, segments, items.clone()))
    } else {
        Some(ModulePath::new(prefix, segments, items.clone()))
    };

    let uses = Use {
        kind: UseKind::Auto,
        module_path,
        paths: Vec::new(), // Legacy - empty for new syntax
        items,
    };
    Ok(Stmt::Use(uses))
}
```

**Step 4: Run tests**

Run: `cargo test -p auto-lang test_parse_pac_import test_parse_super_import`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/src/parser.rs
git commit -m "feat(parser): parse pac and super module prefixes"
```

---

## Task 5: Implement Module Resolver with PathPrefix Support

**Files:**
- Modify: `crates/auto-lang/src/resolver.rs`

**Step 1: Write the failing test**

Add to resolver tests:

```rust
#[test]
fn test_resolve_pac_path() {
    // Setup: create temp dir structure
    // myproject/
    // ├── pac.at (src: ["src"])
    // └── src/
    //     └── db.at

    let resolver = FilesystemResolver::with_package_root(PathBuf::from("myproject/src"));
    let path = resolver.resolve_with_prefix(&ModulePath::pac(vec!["db".into()]), PathBuf::from("myproject/src/api/handlers.at"));
    assert!(path.is_ok());
    assert_eq!(path.unwrap(), PathBuf::from("myproject/src/db.at"));
}

#[test]
fn test_resolve_super_path() {
    // From myproject/src/api/handlers.at, super.db should resolve to myproject/src/db.at
    let resolver = FilesystemResolver::new(PathBuf::from("stdlib/auto"));
    let current_file = PathBuf::from("myproject/src/api/handlers.at");
    let path = resolver.resolve_with_prefix(&ModulePath::super_path(vec!["db".into()]), current_file);
    assert!(path.is_ok());
    assert_eq!(path.unwrap(), PathBuf::from("myproject/src/db.at"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang test_resolve_pac_path test_resolve_super_path`
Expected: FAIL - `resolve_with_prefix` method doesn't exist

**Step 3: Extend ModuleResolver trait**

In `crates/auto-lang/src/resolver.rs`, add:

```rust
use crate::ast::module_path::{ModulePath, PathPrefix};

impl FilesystemResolver {
    /// Create resolver with package source root
    pub fn with_package_root(package_root: PathBuf) -> Self {
        Self {
            std_root: PathBuf::from("stdlib/auto"),
            search_paths: vec![package_root],
        }
    }

    /// Resolve a module path with prefix awareness
    pub fn resolve_with_prefix(
        &self,
        module_path: &ModulePath,
        current_file: PathBuf,
    ) -> Result<PathBuf, String> {
        let segments = &module_path.segments;

        match &module_path.prefix {
            PathPrefix::Pac => {
                // Search from package root(s)
                for search_path in &self.search_paths {
                    let module_file = self.find_module(search_path, segments)?;
                    if module_file.exists() {
                        return Ok(module_file);
                    }
                }
                Err(format!("Module not found: {}", module_path.display()))
            }
            PathPrefix::Super => {
                // Resolve relative to parent of current file's directory
                let current_dir = current_file.parent()
                    .ok_or("Cannot resolve super: current file has no parent directory")?;
                let parent_dir = current_dir.parent()
                    .ok_or("Cannot resolve super: already at root directory")?;
                let module_file = self.find_module(parent_dir, segments)?;
                Ok(module_file)
            }
            PathPrefix::None => {
                // Same directory as current file
                let current_dir = current_file.parent()
                    .ok_or("Cannot resolve: current file has no parent directory")?;
                let module_file = self.find_module(current_dir, segments)?;
                Ok(module_file)
            }
            PathPrefix::Dep(dep_name) => {
                // Look up dependency - requires dependency map
                Err(format!("Dependency resolution not yet implemented: {}", dep_name))
            }
        }
    }

    /// Find a module file in a base directory
    fn find_module(&self, base_dir: &std::path::Path, segments: &[AutoStr]) -> Result<PathBuf, String> {
        // Build path from segments
        let mut module_path = base_dir.to_path_buf();
        for segment in segments {
            module_path.push(segment.as_str());
        }

        // Try file module first: db.at
        let file_module = module_path.with_extension("at");
        if file_module.exists() {
            // Check for ambiguity with directory module
            let dir_module = module_path.join("mod.at");
            if dir_module.exists() {
                return Err(format!(
                    "Ambiguous module '{}' - both '{}' and '{}' exist",
                    segments.join("."),
                    file_module.display(),
                    dir_module.display()
                ));
            }
            return Ok(file_module);
        }

        // Try directory module: db/mod.at
        let dir_module = module_path.join("mod.at");
        if dir_module.exists() {
            return Ok(dir_module);
        }

        Err(format!("Module not found: {}", segments.join(".")))
    }
}
```

**Step 4: Run tests**

Run: `cargo test -p auto-lang test_resolve_pac_path test_resolve_super_path`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/src/resolver.rs
git commit -m "feat(resolver): add ModulePath resolution with pac/super prefixes"
```

---

## Task 6: Update use_scanner for New Syntax

**Files:**
- Modify: `crates/auto-lang/src/use_scanner.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_scan_pac_import() {
    let source = "use pac.db";
    let uses = scan_use_statements(source);
    assert_eq!(uses.len(), 1);
    assert_eq!(uses[0].module, "pac.db");
}

#[test]
fn test_scan_super_import() {
    let source = "use super.utils";
    let uses = scan_use_statements(source);
    assert_eq!(uses.len(), 1);
    assert_eq!(uses[0].module, "super.utils");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang test_scan_pac_import`
Expected: FAIL - scanner doesn't preserve `pac.` prefix

**Step 3: Update use_scanner**

The scanner already captures the full path, so this should mostly work. Verify and adjust if needed.

**Step 4: Run tests**

Run: `cargo test -p auto-lang test_scan_pac_import test_scan_super_import`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/src/use_scanner.rs
git commit -m "feat(use_scanner): support pac and super prefixes"
```

---

## Task 7: Integration Test - Full Module Resolution

**Files:**
- Create: `crates/auto-lang/tests/module_resolution_test.rs`
- Create: `tmp/test_module_project/pac.at`
- Create: `tmp/test_module_project/src/main.at`
- Create: `tmp/test_module_project/src/db.at`
- Create: `tmp/test_module_project/src/api/mod.at`
- Create: `tmp/test_module_project/src/api/handlers.at`

**Step 1: Create test project structure**

```bash
mkdir -p tmp/test_module_project/src/api
```

**Step 2: Create test files**

`tmp/test_module_project/pac.at`:
```auto
name: "test-project"
src: ["src"]
```

`tmp/test_module_project/src/db.at`:
```auto
fn load(path str) str { "loaded: " + path }
```

`tmp/test_module_project/src/api/handlers.at`:
```auto
use pac.db
use super.utils

fn handle() str { db.load("test") }
```

**Step 3: Write integration test**

```rust
// crates/auto-lang/tests/module_resolution_test.rs
use auto_lang::resolver::{FilesystemResolver, ModuleResolver};
use auto_lang::ast::module_path::{ModulePath, PathPrefix};
use std::path::PathBuf;

#[test]
fn test_resolve_pac_from_nested() {
    let resolver = FilesystemResolver::with_package_root(
        PathBuf::from("tmp/test_module_project/src")
    );

    let path = ModulePath::pac(vec!["db".into()]);
    let current = PathBuf::from("tmp/test_module_project/src/api/handlers.at");

    let result = resolver.resolve_with_prefix(&path, current);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), PathBuf::from("tmp/test_module_project/src/db.at"));
}

#[test]
fn test_ambiguous_module_error() {
    // Create both db.at and db/mod.at
    // Should error
}
```

**Step 4: Run tests**

Run: `cargo test -p auto-lang module_resolution_test`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/tests/module_resolution_test.rs tmp/test_module_project/
git commit -m "test: add module resolution integration tests"
```

---

## Task 8: Update AutoMan Resolver for Dependencies

**Files:**
- Modify: `crates/auto-man/src/resolver.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_resolve_dep_path() {
    // Create test structure:
    // workspace/
    // ├── app/pac.at (dep database(path: "../database"))
    // └── database/
    //     └── connection.at

    let resolver = AutoManResolver::new(
        PathBuf::from("tmp/test_workspace/app"),
        PathBuf::from("stdlib/auto")
    ).prepare_env().unwrap();

    let path = ModulePath::dep("database".into(), vec!["connection".into()]);
    let result = resolver.resolve_with_prefix(&path, PathBuf::from("tmp/test_workspace/app/src/main.at"));
    assert!(result.is_ok());
}
```

**Step 2: Implement dependency path resolution in AutoManResolver**

Extend `AutoManResolver` to handle `PathPrefix::Dep`:

```rust
impl AutoManResolver {
    pub fn resolve_with_prefix(
        &self,
        module_path: &ModulePath,
        current_file: PathBuf,
    ) -> Result<PathBuf, String> {
        match &module_path.prefix {
            PathPrefix::Dep(dep_name) => {
                let dep = self.dependencies.get(dep_name.as_str())
                    .ok_or_else(|| format!("Dependency not declared: {}", dep_name))?;
                let segments = &module_path.segments;
                self.find_module(&dep.path, segments)
            }
            // ... other cases delegate to base resolver
        }
    }
}
```

**Step 3: Run tests**

Run: `cargo test -p auto-man test_resolve_dep_path`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/auto-man/src/resolver.rs
git commit -m "feat(auto-man): resolve module paths from dependencies"
```

---

## Task 9: Error Messages for Common Mistakes

**Files:**
- Modify: `crates/auto-lang/src/resolver.rs`

**Step 1: Write test for error messages**

```rust
#[test]
fn test_error_ambiguous_module() {
    // When both file and dir exist
    let result = resolver.resolve_with_prefix(&path, current);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Ambiguous"));
    assert!(err.contains("db.at"));
    assert!(err.contains("db/mod.at"));
}

#[test]
fn test_error_module_not_found() {
    // When neither file nor dir exist
    let result = resolver.resolve_with_prefix(&path, current);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("not found"));
}
```

**Step 2: Verify error messages are clear**

Run tests and check output.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/resolver.rs
git commit -m "feat(resolver): clear error messages for module resolution"
```

---

## Task 10: Update Documentation

**Files:**
- Modify: `CLAUDE.md`
- Modify: `docs/plans/131-module-path-syntax-design.md`

**Step 1: Update CLAUDE.md with new syntax**

Add section about module path syntax under "Key Syntax".

**Step 2: Mark Plan 131 design doc as implemented**

**Step 3: Commit**

```bash
git add CLAUDE.md docs/plans/131-module-path-syntax-design.md
git commit -m "docs: update module path syntax documentation"
```

---

## Final Verification

Run all tests:
```bash
cargo test -p auto-lang module
cargo test -p auto-man resolver
```

All tests should pass.

---

## Deferred to Future Plans

- **Phase 4:** `pub` visibility and `pub use` re-exports
- **Phase 5:** Wildcard imports (`use db: *`) in `.as` scripts
- **Dependency alias support:** `dep database(path: "../database", as: "db")`
