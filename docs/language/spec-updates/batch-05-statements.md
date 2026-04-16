# Spec Update: Batch 05 — Statements & Control Flow

**Date**: 2026-04-16
**Plans Referenced**: Plan 120 (Option/Result patterns), Plan 122 (ownership), Plan 124 (reply), Plan 165 (struct destructuring), Plan 168 (shared)
**Source Files**: `ast.rs` (Stmt enum, 34 variants), `ast/store.rs` (StoreKind, 6 variants)
**Sections Updated**: Statements (Section 7), Control Flow (Section 8)

## Old Content

### Statements
- Listed 4 storage modifiers: `let`, `mut`, `const`, `var`
- Only expression, block statements

### Control Flow
- `if`, `for`, `while` loops, `loop`, `is` pattern matching
- Pattern branches used `->` (incorrect — should be `=>`)
- No struct destructuring, no Option/Result patterns, no `when` blocks
- Used `println()` instead of `print()`

## New Content

### Statements
- Updated to 6 storage modifiers: `let`, `var`, `const`, `mut`, `shared` (Plan 168), `static`
- Added `return` statement documentation
- Added `reply` statement for task RPC (Plan 124)
- Added `break` statement
- Added import statements (`use`, `dep`)
- Added type declaration statements
- Added extension statements (`ext`)
- Added comment statements
- Noted empty lines as statement separators

### Control Flow
- Fixed pattern matching arrow syntax: `->` → `=>`
- Replaced `println()` with `print()` throughout
- Removed `while` — documented `for condition { }` as replacement
- Added struct destructuring in patterns (Plan 165)
- Added Option pattern matching: `Some(x)` / `None` (Plan 120)
- Added Result pattern matching: `Ok(value)` / `Err(msg)` (Plan 120)
- Added `when` blocks

### Stmt Variant Coverage
34 total variants now documented:
- Expr, If, For, Is, Store, Block, Fn, EnumDecl, TypeDecl, Union, Tag
- SpecDecl, Node, Use, Dep, OnEvents, Comment, Alias, TypeAlias
- EmptyLine, Break, Return, Reply, Ext
- WidgetDecl, MsgDecl, ModelBlock, ViewBlock (UI — Batch 12)
- TaskDef (Tasks — Batch 10)
- HashIf, HashFor, HashIs, HashBrace (Compile-time — Batch 11)

## Notes

- `while` is NOT a keyword in Auto. The spec previously documented it incorrectly.
- All `println()` references replaced with `print()` (Auto syntax)
- Pattern matching uses `=>` not `->` (this was a documentation error)
- `StoreKind` has 6 variants: Let, Var, Const, Shared, CVar, Field
