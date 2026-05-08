# 07 - VM Runtime

## Overview
The AutoVM bytecode engine is the default execution backend for AutoLang, having fully replaced the legacy tree-walking evaluator. This topic covers the complete VM stack: bytecode compilation, runtime execution, type system integration, closures, generics, iterators, task system, enum codegen, and the monomorphic dispatch mechanism for generic method calls.

## Plan Summary

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 039 | VM Tests Migration to AutoVM Tests | 🔧 | Migrate vm_tests.rs tests to autovm_tests.rs by complexity level |
| 068 | AutoVM (AutoVM) Implementation | ✅ | AutoVM bytecode engine with 9 phases, now the default execution engine |
| 069 | AutoVM Global Variable Support | ✅ | Persistent REPL variables via global scope in AutoVM |
| 070 | AutoVM Iterator Implementation | ✅ | List.iter(), Iterator.next(), lazy map/filter adapters |
| 071 | AutoVM Closure Implementation | ✅ | Full closure support with environment capture across 6 phases |
| 073 | AutoVM Migration Roadmap | ✅ | Complete AutoVM replacement of Evaluator with feature parity |
| 074 | Use Statement Multi-Directory Search | 🔧 | Multi-directory module lookup for use statements |
| 075 | ConfigCodegen and TemplateCodegen | ✅ | CONFIG/TEMPLATE execution modes via pure bytecode |
| 076 | AutoVM Generic Type Support | ✅ | Generic type parsing, monomorphization, List<T> support |
| 077 | Unified Object Registry + Generic ListData | 🔧 | Single registry for heap objects, generic ListData<T> storage (50%) |
| 078 | AutoMan Integration | ✅ | Migrate auto-man into monorepo with dependency resolver |
| 079 | Full AutoMan Migration Strategy | ✅ | Complete auto-man build system and package manager migration |
| 080 | AutoVM Stack Frame Bug Fix | ✅ | Fix REPL variable accumulation caused by shared stack/local memory |
| 081 | AutoVM as Default Execution Mode | ✅ | Make AutoVM the default, support per-dependency execution modes |
| 087 | AutoVM Generics - Type Erasure + Specialization | ✅ | Generic types with type-erased storage and specialized access (90%) |
| 117 | VM Runtime Type Coercion | ✅ | Fix mixed int/float arithmetic by emitting correct conversion opcodes |
| 118 | VM Test Failures Analysis | 🔧 | Systematic fix of 76+ failing VM tests (183/197 passing) |
| 127 | AutoVM TaskSystem Execution | ✅ | Bytecode compilation and execution for Task/Msg systems |
| 177 | VM File-Based Test Framework | ⏳ | Replace inline tests with file-based .expected.out/result/error assertions |
| 191 | Assert and Precise Linker Errors | ✅ | Add assert/assert_eq/assert_ne intrinsics and propagate source positions into linker error spans |
| 192 | VM Enum & Ext Codegen | ✅ | Enum declaration, ext method codegen, is-match for enum variants (done per Plan 200 ref) |
| 194 | Monomorphic Dispatch for Generic Methods | ✅ | Compile-time type-based dispatch for HashMap/HashSet generic APIs (done per Plan 200 ref) |
| 197 | VM Enum/Data, Generic Lists, Pattern Debug | ✅ | All 5 phases done: string eq, method chaining, struct debug, enum data, List<UserType>, pattern destruct, Option<T> |
| 198 | Native Metadata from Source | ✅ | Eliminate hardcoded native metadata by deriving from #[vm] source declarations |
| 199 | VM Interactive Debugger | ✅ | SOURCE_LINE opcodes, call stack, disassembler, GDB-style debugger, AI agent debug API |
| 200 | VM Missing Features (Examples 14-33) | ✅ | loop/continue/tuple/range slicing, .map_err() closure, fs module aliases — all done |
| 201 | VM Four Pillars (Enum/Closure/Result/Spec) | ✅ | All 4 pillars complete: multi-field enum, closure HOF, Result heap objects, spec vtable dispatch |
| 203 | Native Registry Namespace Unification | ✅ | QualifiedName + resolve_qualified + import_scope; ~137 short-name aliases eliminated; monomorphic dispatch refactored |
| 206 | Closure HOF + call_closure API | ✅ | call_closure public API, List.map/filter/reduce/find/for_each shims |
| 207 | Enum Multi-Field Destructuring | ✅ | Multi-binding destructuring and named arg construction for enum variants |
| 208 | Result Heap Object | ✅ | CREATE_OK/CREATE_ERR heap objects, IS_OK, UNWRAP_OK/ERR, ERROR_PROPAGATE |
| 212b | Rust FFI E2E Dynamic Loading | ✅ | dep serde_json -> cargo build cdylib -> AutoVM load .dll -> call |
| 216 | C FFI Build Pipeline Integration | ✅ | CLI integration for C FFI bindgen into build pipeline (Phase 4 of Plan 216) |
| 221 | Nanboxing Migration | ✅ | Migrate VM value representation to NaN-boxing for compact tagged values |
| 224 | VM Async Runtime | ✅ | TaskSystem.run bridge, AWAIT_FUTURE reentrant execution, async FFI shim support |
| 226 | ABT Bytecode Text Format | ✅ | ABC↔ABT assembler/disassembler, Playground bytecode tab |
| 229a | IS_VARIANT Primitive Fix | ✅ | Engine-level i32 Option compatibility |
| 230 | f64 Struct Literal Fix | ✅ | PROMOTE_F64 in 5 codegen paths |
| 231 | Nested mut fn Stack Fix | ✅ | SET_GENERIC_FIELD Void marking + BUILD_FSTR formatting |

## Status Summary
- Completed: 26 | Partial: 4 | Planned: 0 | Deprecated: 0

## Key Achievements
- AutoVM fully replaced the tree-walking Evaluator with 1.00-1.10x performance improvement and feature parity
- Complete closure implementation with environment capture, iterators with lazy adapters, and generic type monomorphization
- Type coercion fixes resolved 76+ mixed arithmetic bugs; systematic test fix campaign reached 183/197 passing

## Remaining Work
- File-based test framework (Plan 177) to replace 3000+ lines of inline tests with maintainable .expected.* files
