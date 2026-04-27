# 09 - Modules

## Overview
AutoLang's module system evolved from a monolithic Universe-based architecture to a clean separation using TypeStore, InferenceContext, and AIE-based incremental compilation. The system supports folder modules (mod.at), public re-exports (pub use), symbol-level imports, and cross-module function calls via bytecode linking.

## Plan Summary

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 085 | AIE + AutoCache Use Statement Processing | ✅ | Replace Universe.import() with AIE framework for incremental module resolution |
| 089 | Type Declaration Storage Migration to Infer | ✅ | Migrate all type declarations from codegen/Database to unified infer module |
| 090 | Remove Parser's Universe Dependency | ✅ | Replace Parser's Universe dependency with TypeStore + InferenceContext |
| 091 | Complete Universe Removal | ✅ | Remove Universe type entirely, migrate to TypeStore + Database + AutoCache |
| 131 | Module Path Syntax Design | ✅ | Design for relative (super), package-relative (pac), and dependency imports |
| 167 | Complete Module System Implementation | ✅ | Folder modules, pub use re-exports, wildcard imports, circular dependency detection |
| 184 | Cross-Module Function Calls | ✅ | Compile dependency modules to bytecode and link via existing Linker infrastructure |
| 198 | Native Metadata from Source | ✅ | Eliminate hardcoded native metadata by deriving from #[vm] source declarations |
| 203 | Native Registry Namespace Unification | 🔧 | QualifiedName-based native function lookup replacing string concatenation |
| 212b | Rust FFI E2E Dynamic Loading | ✅ | dep serde_json -> cargo build cdylib -> AutoVM load .dll -> call |
| 214 | Python FFI (use.py) | ✅ | Embed Python interpreter in AutoVM |
| 216 | C FFI Bindgen | ✅ | Auto-bindgen for C headers with libloading runtime, a2c auto-bind, CLI integration |

## Status Summary
- Completed: 7 | Partial: 0 | Planned: 2 | Deprecated: 0

## Key Achievements
- Complete removal of the monolithic Universe type, replaced by TypeStore + InferenceContext + Database + AutoCache
- Full module system with folder modules (mod.at), pub re-exports, wildcard imports, and circular dependency detection
- Cross-module function calls work by compiling dependency modules to bytecode and linking them at runtime via the existing Linker

## Remaining Work
- pub visibility enforcement and pub use re-export expansion (Phase 4-5 from Plan 131)
- Wildcard imports (use db: *) limited to .as scripts; full .at support pending
- Deep module path resolution for package dependencies via dep declarations in pac.at
- Plan 203: Unify native registry with QualifiedName-based lookup
- Plan 203: Unify native registry with QualifiedName-based lookup
