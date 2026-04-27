# 09 - Module System and Imports

## Overview

AutoLang's module system evolved through a deliberate architectural transformation: the monolithic `Universe` type -- which originally handled symbol tables, type storage, scope management, module tracking, imports, and runtime values in a single struct -- was decomposed into focused components (`TypeStore`, `InferenceContext`, `ScopeManager`, `AutoCache`). This decomposition enabled clean separation of concerns and made incremental compilation possible. On top of this new foundation, AutoLang built a complete module system with file-based module resolution, folder modules via `mod.at`, public re-exports, wildcard imports, circular dependency detection, cross-module bytecode linking, and multi-file transpilation to Rust.

## Plan Index

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 085 | AIE + AutoCache Use Statement Processing | Done | Replace Universe.import() with AIE framework; add use scanner, TypeStore merge, AutoCache |
| 089 | Type Declaration Storage Migration to Infer | Done | Migrate type declarations from codegen/Database to unified TypeStore via TypeRegistry |
| 090 | Remove Parser's Universe Dependency | Done | Replace Parser's Universe usage with TypeStore + InferenceContext + ModuleTracker |
| 091 | Complete Universe Removal | Done | Delete Universe, eval.rs, interp.rs (~11k lines); introduce ScopeManager and vm/types.rs |
| 131 | Module Path Syntax Design | Done | Design and implement pac/super/dep import prefixes with filesystem resolver |
| 167 | Complete Module System Implementation | Done | pub use re-exports, wildcard imports, circular dependency detection, multi-file a2r transpilation |
| 184 | Cross-Module Function Calls | Done | Compile dependency modules to bytecode and link via existing Linker infrastructure |
| 198 | Native Metadata from Source | Complete | Eliminate hardcoded native metadata by deriving from #[vm] source declarations |
| 203 | Native Registry Namespace Unification | Complete | QualifiedName + resolve_qualified + import_scope; ~137 aliases eliminated; Phase 5f deferred |
| 212b | Rust FFI E2E Dynamic Loading | Complete | dep serde_json -> cargo build cdylib -> AutoVM load .dll -> call |
| 214 | Python FFI (use.py) | Complete | Embed Python interpreter in AutoVM via use.py import syntax |
| 216 | C FFI Build Pipeline Integration | Complete | CLI integration for C FFI bindgen into build pipeline |

## Status

**Implemented**: All seven plans are complete. The module system is fully functional with filesystem resolution, incremental caching, cross-module linking, and multi-file transpilation.

**Partial**: Full `pub` visibility enforcement across module boundaries is not yet enforced at compile time. The `PathPrefix::Dep` variant for external dependency resolution is deferred. String pool merging across modules is not yet implemented -- cross-module string constant references remain a known limitation.

**Planned**: Deep package dependency resolution via `dep` declarations in `pac.at`, module versioning, remote module loading, and IDE integration for auto-import completion.

## Design

### Decomposing the Universe

The original `Universe` type was a god object spanning roughly 2,000 lines that managed six distinct responsibilities: symbol tables, type storage, scope management, module path tracking, module imports, and runtime value storage. Plans 085, 089, 090, and 091 systematically dismantled this monolith.

Plan 089 established a `TypeRegistry` inside the infer module and then unified it into `TypeStore` (via Plan 084), creating a single source of truth for type declarations, function declarations, spec declarations, and generic templates. `TypeStore` is shared across Parser, Codegen, and InferenceContext through `Arc<RwLock<TypeStore>>`. Plan 090 introduced `ModuleTracker` (replacing Universe's `cur_spot`/`enter_mod()`) and `LambdaIdGenerator`, then migrated Parser's symbol lookups to query TypeStore and InferenceContext directly. Plan 085 built the `use_scanner` module for fast pre-processing of use statements, added `TypeStore.merge()` and `import_items()` for symbol import, and implemented `AutoCache` for module-level caching with file-hash validation. Finally, Plan 091 deleted the old interpreter (`eval.rs`, ~7,000 lines), the old `interp.rs` (~1,500 lines), `universe.rs` itself (~2,000 lines), and `repl.rs` (~500 lines) -- approximately 11,000 lines of legacy code removed. A new `ScopeManager` replaced the Universe's scope-management role in the Parser.

The result is a clean architecture where `CompileSession` orchestrates the compilation pipeline, `TypeStore` holds type information, `InferenceContext` manages type environments and inference, `AutoCache` handles incremental caching, and `ScopeManager` tracks lexical scopes in the Parser.

### Module Path Syntax and Resolution

Plan 131 defined the import syntax that AutoLang uses today. Imports support four prefix modes: bare (same directory), `super.` (parent directory), `pac.` (package root), and dependency imports by name. The core data structures live in `crates/auto-lang/src/ast/module_path.rs`:

```rust
pub enum PathPrefix {
    None,           // use db
    Super,          // use super.db
    Pac,            // use pac.db
    Dep(AutoStr),   // use database.connection
}

pub struct ModulePath {
    pub prefix: PathPrefix,
    pub segments: Vec<AutoStr>,
    pub items: Vec<AutoStr>,
}
```

The resolution algorithm, implemented in `FilesystemResolver::find_module()`, tries `<path>.at` first, then `<path>/mod.at`. If both exist, the compiler emits an ambiguity error requiring the user to rename one. The `super` keyword is intentionally limited to one level; deeper navigation is discouraged in favor of `pac.` for refactor-stable absolute paths. Dependency imports are declared in `pac.at` via `dep name(path: "...")` statements and resolved by `AutoManResolver`.

Symbol-level imports use the colon syntax: `use db: load, save` imports specific symbols, while `use db: *` imports all public symbols (restricted to `.as` script files). Namespace imports (`use db`) allow qualified access via `db.load()`.

### Folder Modules, Re-exports, and Wildcards

Plan 167 completed the module system by implementing the remaining pieces. Folder modules use `mod.at` as the entry point for a directory -- the resolver already supported this pattern, but Plan 167 added proper `pub use` re-export parsing. The `Use` AST node gained `is_pub` and `is_wildcard` fields. When a `pub use` statement imports symbols, they are tracked in `TypeStore.pub_exports` so that downstream consumers can see them.

Wildcard imports (`use module: *`) are parsed by checking for the star token after the colon and setting `is_wildcard: true` with an empty items list. The a2r transpiler emits `use crate::module::*;` for wildcards. Circular dependency detection was added to `CompileSession` via a `loading_stack` that tracks modules currently being loaded. If a module appears in its own loading chain, the compiler emits a descriptive error showing the full dependency chain.

### Multi-File Transpilation

Plan 167 also introduced multi-file Rust transpilation through a `MultiSink` structure and `transpile_rust_project()` entry point. In multi-file mode, each `.at` file becomes a separate `.rs` file, `mod.at` maps to `mod.rs` with `pub mod` declarations for submodules, and use statements are remapped appropriately: `use db` becomes `mod db;` when `db.at` is a sibling file, while `use db: connect` becomes `use crate::db::connect;`. The transpiler also generates a `Cargo.toml` by scanning `dep` declarations across all source files. Test coverage includes dedicated a2r tests for pub use re-exports (test 159), wildcard imports (test 160), and multi-file projects (test 161).

### Cross-Module Bytecode Linking

Plan 184 addressed the runtime side of the module system: calling functions defined in external modules. Previously, `resolve_uses()` would load a module's type declarations into TypeStore but generate no bytecode for its function bodies. The plan introduced a `compiled_modules` field on `CompileSession` and a `compile_module_to_bytecode()` method that parses the dependency module, compiles its non-native function declarations to bytecode, and produces a `Module` with exports. These compiled modules are then passed to the existing `Linker` in `execute_autovm()`, which performs multi-module linking in two passes: first registering all exports into a global symbol table, then resolving all relocations. Native functions (`#[vm]`) are excluded from bytecode compilation since they work through the `native_registry`. The Linker already handled same-named function conflicts (duplicate symbol errors) and recursive cross-module calls (the two-pass approach resolves references regardless of direction).

The implementation identified three categories of external functions: `#[vm]` native functions (no bytecode needed), user functions that call only natives (bytecode plus linking), and user functions calling other user functions in the same module (handled automatically by compiling the module as a whole).

## Open Questions

- **String pool merging**: Each module maintains its own string index table. Cross-module string constant references do not yet work, though most practical cross-module calls do not pass string literals across module boundaries.
- **Module initialization code**: Top-level expressions in imported modules do not execute at import time, consistent with Python-style semantics but potentially surprising for users expecting side effects.
- **Visibility enforcement**: `pub` is recognized syntactically but not enforced at compile time. A future pass should validate that private symbols are not accessed from outside their defining module.
- **External dependency resolution**: `PathPrefix::Dep` is defined but not fully wired up. The `dep` declaration in `pac.at` exists but the resolver does not yet search dependency paths for deep module references like `serde.json.from_str`.

## Source Plans

- Plan 085: AIE + AutoCache Use Statement Processing
- Plan 089: Type Declaration Storage Migration to Infer
- Plan 090: Remove Parser's Universe Dependency
- Plan 091: Complete Universe Removal
- Plan 131: Module Path Syntax Design
- Plan 167: Complete Module System Implementation
- Plan 184: Cross-Module Function Calls
- [198-native-metadata-from-source.md](../plans/old/198-native-metadata-from-source.md)
- [203-native-registry-namespace.md](../plans/203-native-registry-namespace.md)
- [212-rust-ffi-e2e.md](../plans/212-rust-ffi-e2e.md)
- [214-python-ffi-use-py.md](../plans/214-python-ffi-use-py.md)
- [216-cffi-bindgen.md](../plans/216-cffi-bindgen.md)
