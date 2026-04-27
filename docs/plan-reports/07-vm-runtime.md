# 07 - VM Runtime

## Overview

The AutoVM bytecode engine is the default execution backend for AutoLang, having fully replaced the legacy tree-walking evaluator. The VM stack spans the complete compilation pipeline: from AST-to-bytecode codegen, through a register-based execution engine with an expanding instruction set, to a unified heap object registry supporting generic collections. The system achieved 23.77x average speedup over the evaluator while reaching 97.4% feature parity (1254/1288 tests passing). Key architectural milestones include a 9-phase AutoVM build-out (Plan 068), full closure support with direct capture semantics (Plan 071), generic type monomorphization (Plan 076), a unified object registry with `HeapObject` trait objects (Plan 077), and a Task/Msg async concurrency framework built on Tokio (Plan 069/127). Three major items remain: a file-based test framework to replace 3000+ lines of inline tests, enum and ext-method codegen for pattern matching on enum variants, and monomorphic dispatch to eliminate type-suffixed API names on generic collections.

## Plan Index

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 039 | VM Tests Migration to AutoVM Tests | In Progress | Migrate vm_tests.rs to autovm_tests.rs by complexity level (10 levels) |
| 068 | AutoVM (AutoVM) Implementation | Complete | Bytecode engine with 9 phases, default execution engine |
| 069 | AutoVM Global Variable Support | Complete | Persistent REPL variables via task reuse and global scope |
| 070 | AutoVM Iterator Implementation | Complete | List.iter(), Iterator.next(), lazy map/filter adapters |
| 071 | AutoVM Closure Implementation | Complete | Full closure support with environment capture across 6 phases |
| 073 | AutoVM Migration Roadmap | Complete | Complete AutoVM replacement of Evaluator with feature parity |
| 074 | Use Statement Multi-Directory Search | In Progress | Multi-directory module lookup for use statements |
| 075 | ConfigCodegen and TemplateCodegen | Complete | CONFIG/TEMPLATE execution modes via pure bytecode |
| 076 | AutoVM Generic Type Support | Complete | Generic type parsing, monomorphization, List<T> support |
| 077 | Unified Object Registry + Generic ListData | In Progress | Single registry for heap objects, generic ListData<T> storage (50%) |
| 078 | AutoMan Integration | Complete | Migrate auto-man into monorepo with dependency resolver |
| 079 | Full AutoMan Migration Strategy | Complete | Complete auto-man build system and package manager migration |
| 080 | AutoVM Stack Frame Bug Fix | Complete | Fix REPL variable accumulation caused by shared stack/local memory |
| 081 | AutoVM as Default Execution Mode | Complete | Make AutoVM the default, support per-dependency execution modes |
| 087 | AutoVM Generics - Type Erasure + Specialization | Complete | Generic types with type-erased storage and specialized access (90%) |
| 117 | VM Runtime Type Coercion | Complete | Fix mixed int/float arithmetic by emitting correct conversion opcodes |
| 118 | VM Test Failures Analysis | In Progress | Systematic fix of 76+ failing VM tests (183/197 passing) |
| 127 | AutoVM TaskSystem Execution | Complete | Bytecode compilation and execution for Task/Msg systems |
| 177 | VM File-Based Test Framework | Planned | Replace inline tests with file-based .expected.out/result/error assertions |
| 191 | Assert and Precise Linker Errors | Complete | Add assert/assert_eq/assert_ne intrinsics and propagate source positions into linker error spans |
| 192 | VM Enum and Ext Codegen | Complete | Enum declaration, ext method codegen, is-match for enum variants (done per Plan 200 ref) |
| 194 | Monomorphic Dispatch for Generic Methods | Complete | Compile-time type-based dispatch for HashMap/HashSet generic APIs (done per Plan 200 ref) |
| 197 | VM Enum/Data, Generic Lists, Pattern Debug | Complete | All 5 phases: string eq, method chaining, struct debug, enum data, List<UserType>, Option<T> |
| 198 | Native Metadata from Source | Complete | Eliminate hardcoded native metadata by deriving from #[vm] source declarations |
| 199 | VM Interactive Debugger | Complete | SOURCE_LINE opcodes, call stack, disassembler, GDB-style debugger, AI agent debug API |
| 200 | VM Missing Features (Examples 14-33) | Complete | loop/continue/tuple/range slicing, .map_err() closure, fs module aliases |
| 201 | VM Four Pillars (Enum/Closure/Result/Spec) | Complete | All 4 pillars: multi-field enum, closure HOF, Result heap objects, spec vtable dispatch |
| 203 | Native Registry Namespace Unification | Partial | QualifiedName-based native function lookup replacing string concatenation |
| 206 | Closure HOF + call_closure API | Complete | call_closure public API, List.map/filter/reduce/find/for_each shims |
| 207 | Enum Multi-Field Destructuring | Complete | Multi-binding destructuring and named arg construction for enum variants |
| 208 | Result Heap Object | Complete | CREATE_OK/CREATE_ERR heap objects, IS_OK, UNWRAP_OK/ERR, ERROR_PROPAGATE |
| 212b | Rust FFI E2E Dynamic Loading | Complete | dep serde_json -> cargo build cdylib -> AutoVM load .dll -> call |
| 221 | Nanboxing Migration | Complete | Migrate VM value representation to NaN-boxing for improved memory and performance |

## Status

**Implemented**: 068, 069, 070, 071, 073, 075, 076, 078, 079, 080, 081, 087, 117, 127, 191, 192, 194, 197, 198, 199, 200, 201, 206, 207, 208, 212b, 221

**Partial**: 039 (basic levels migrated), 074 (parser works, evaluator partially updated), 077 (8/8 phases done but index marks 50%), 118 (183/197 passing, 11 still failing), 203 (qualified name lookup in progress)

**Not Implemented**: (none remaining)

**Planned**: 177

## Design

### AutoVM Core Engine (Plans 068, 080, 081)

The AutoVM is a stack-based bytecode virtual machine with a variable-length instruction set called AutoByteCode (ABC) v1.0. The architecture centers on three memory regions: VirtualFlash (a read-only byte array for code and constants, simulating XIP), VirtualRAM (a read-write `Vec<i32>` array for stack and heap), and a set of registries (lists, closures, iterators, channels, heap objects) managed through DashMap for thread-safe concurrent access.

The execution engine follows a fetch-decode-execute loop. A critical early bug (Plan 080) revealed that stack and local variables shared the same memory region when bp=0 in the main task, causing REPL variables to accumulate incorrectly (5 becoming 6, then 7). The fix reserved stack space for local variables by pushing dummy CONST_0 values at function entry, ensuring sp starts at n_locals rather than 0.

Plan 081 made AutoVM the default execution mode. The `ExecutionEngine` enum now defaults to AutoVM without requiring a feature flag, though the evaluator remains accessible via the `AUTO_EXECUTION_ENGINE` environment variable for debugging. Performance benchmarks showed AutoVM is 23.77x faster than the evaluator on average, with individual benchmarks ranging from 11.92x to 54.83x speedup. The feature parity check confirmed 1254/1288 tests passing (97.4%).

The VM instruction set has grown to include over 100 opcodes organized into categories: arithmetic (ADD, SUB, MUL, DIV with type-specific variants for i32, f32, f64, i64), control flow (JMP, JMP_IF_Z, JMP_IF_NZ, CALL, RET), memory (CONST_I32, CONST_F32, LOAD_LOC_N, STORE_LOC_N), heap objects (CREATE_LIST_INT, MAKE_OBJ, CREATE_NODE), generic types (NEW_INSTANCE, CONSTRUCT_INSTANCE, GET_GENERIC_FIELD), closures (CLOSURE, LOAD_CAPTURED, STORE_CAPTURED, CALL_CLOSURE), concurrency (SPAWN, CHAN_NEW, SEND, RECV), and string operations (BUILD_FSTR, STR_CAT, TO_STR).

### Closures and Iterators (Plans 070, 071)

AutoVM closures use a direct capture model rather than Lua-style upvalues. The `Closure` struct stores a function address and an `env: HashMap<String, Value>` containing captured variables. This design was chosen because AutoLang has explicit closure syntax (`x => x + n`) with ownership keywords (`.take`, `.view`, `.mut`), making it possible to analyze captures at compile time.

The default capture semantics are copy-based: captured values are cloned into the closure environment, which is safe for escaping closures (those that outlive their parent scope). The compiler explicitly blocks `.view` and `.mut` capture in closures to prevent dangling references, with clear error messages directing users to use default copy or explicit `.take` instead. Five new opcodes support closures: CLOSURE (0x90) creates the closure object, LOAD_CAPTURED (0x92) and STORE_CAPTURED (0x93) access the environment, and CALL_CLOSURE (0x94) invokes the closure. The CALL_CLOSURE opcode sets a `current_closure_id` field on the task, and RET restores the previous closure ID, enabling nested closures.

The iterator system (Plan 070) provides `List.iter()` and `Iterator.next()` as native functions, with lazy adapters for `map()` and `filter()`, and terminal operations for `collect()`, `reduce()`, and `find()`. A unified `Iterator` enum with List, Map, and Filter variants handles all adapter chaining. The implementation currently has an MVP limitation where map/filter adapters pass through elements without actually calling the predicate function, which requires further closure integration.

### Type System and Generics (Plans 076, 077, 087)

Generic type support in AutoVM follows a monomorphization approach. When the compiler encounters `List<int>`, it generates type-specific opcodes like `CREATE_LIST_INT` and `LIST_PUSH_INT`. The `GenericTable` tracks all generic instantiations during compilation, and the `Monomorphizer` pass generates specialized bytecode for each unique type parameter combination. This approach yields zero-overhead access for primitive types: `List<int>` uses `Vec<i32>` internally (4 bytes per element) rather than `Vec<Value>` (24 bytes per element).

Plan 077 introduced a unified object registry based on the `HeapObject` trait, replacing per-type registries with a single `DashMap<u64, Arc<RwLock<dyn HeapObject>>>`. The trait provides `type_tag()` for runtime type checking and `as_any()`/`as_any_mut()` for downcasting via `std::any::Any`. Performance analysis showed that while the downcast adds ~15ns per operation, the 6x memory reduction for primitive lists yields a net 1.43x average speedup in real workloads due to improved cache efficiency. The optimized `try_downcast_checked()` helper combines the type tag check and downcast into a single inlined operation, achieving 17% faster performance than separate operations.

Plan 087 extended generics to user-defined types with a `GenericRegistry` and `ClassTemplate`/`ClassType` system. User-defined generic types like `type Pair<K, V> { key K, val V }` are stored with type-erased `Vec<Value>` field storage, while built-in collections (List, HashMap) use specialized storage. The system supports instantiation via object literal syntax (`Pair { key: 1, val: "a" }`), field access (`p.key`), and method calls (`p.get_key()`). Limitations remain: function call syntax with named parameters (`Pair(key: 1, val: "a")`) and generic instance type annotations (`let p Pair<int, string>`) are not yet supported.

### Execution Modes and Codegen Strategies (Plans 073, 075)

The evaluator's three-mode system (SCRIPT, CONFIG, TEMPLATE) was migrated to AutoVM using separate codegen strategies rather than adding mode awareness to the VM itself. This design keeps the VM mode-agnostic: all mode-specific logic lives in the compiler.

`ConfigCodegen` transforms configuration files into bytecode that builds a unified object structure. It parses dotted field paths (`server.host = "localhost"`), creates nested objects, and evaluates expressions before storing. `TemplateCodegen` transforms template files into string concatenation bytecode, supporting variable interpolation, nil filtering, and configurable separators. Three new opcodes support template operations: TO_STR (0x7A) converts any value to its string representation, IS_NIL (0x7B) checks for nil, and STR_CAT (0x7C) concatenates two strings from the pool.

The migration roadmap (Plan 073) tracked the full feature gap between evaluator and AutoVM across 9 phases. Major phases completed include: type system expansion to all primitive types (float, double, uint, i8, u8, i64, u64, byte, char), object literals with field access (MAKE_OBJ, GET_FIELD opcodes), for loops (range, iterator, indexed, conditional, infinite variants), f-string interpolation (BUILD_FSTR opcode), is-pattern matching with EqBranch and IfBranch support, May<T> question operators (NULL_COALESCE and ERROR_PROPAGATE opcodes), array indexing (CREATE_ARRAY, GET_ELEM, SET_ELEM), and type declarations with method calls (CALL_METHOD opcode).

### Concurrency and Task System (Plans 069, 127)

AutoVM uses Tokio-based M:N green thread scheduling for concurrency. The VM was split into `AutoVM` (shared runtime with task registry, flash, string pool, and native interface) and `AutoTask` (per-task execution context with stack, frames, IP, BP, and status). Tasks are spawned via `tokio::spawn()` and managed in a `DashMap<TaskId, Arc<Mutex<AutoTask>>>`.

Nine concurrency opcodes support the system: SPAWN (0x80) creates a new task from a function address, TASK_ID (0x81) returns the current task's ID, YIELD (0x82) cooperatively yields execution, SLEEP (0x83) suspends for N milliseconds, JOIN (0x84) waits for a task to complete, CHAN_NEW (0x85) creates a channel, SEND (0x86) and RECV (0x87) handle message passing, and TRY_RECV (0x88) performs non-blocking receive. Channel operations currently use a busy-wait with yield pattern rather than true async await.

Plan 127 extended the task system with a full message-handling framework. Three new opcodes (TASK_LOOP, HANDLE_MSG, REPLY) enable tasks to receive messages and dispatch them to pattern-matched handlers. The `TaskHandlerTable` maps message patterns to bytecode addresses, and `ctx.reply()` is wired as an FFI shim that sends responses through a oneshot channel. The `.go` fire-and-forget execution pattern (SPAWN_GO opcode) was also implemented. Ask/reply synchronization (blocking the caller until a reply arrives) is deferred pending an async/sync bridge.

### Bug Fixes and Test Campaigns (Plans 117, 118)

Plan 117 addressed a class of type coercion bugs affecting 76+ VM tests. When compiling expressions with mixed integer/float operands (e.g., `2 + 3.5`), the codegen emitted float opcodes without first converting integer operands. The VM would then interpret integer bits as float bits, producing garbage values. The fix added two conversion opcodes: I32_TO_F32 (0x46) and I64_TO_F64 (0x47), emitted by the codegen when it detects that one operand is an integer in a float-typed binary operation.

Plan 118 documents a systematic campaign to fix remaining VM test failures. Starting from 121/197 passing, the campaign reached 183/197 by fixing issues across several categories: u8 type inference returning `Type::Uint` instead of `Type::Int`, out-of-bounds array assignment silently failing (fixed by adding `last_error` to `AutoTask`), void functions returning "0" instead of "" (fixed by modifying native print shims to not push return values and updating `last_expr_type` tracking), invalid field access not producing errors (fixed by making GET_FIELD/SET_FIELD return VMError), type instance field access panics (fixed by switching from CREATE_OBJ to NEW_INSTANCE for type constructors), if-in-array filtering (fixed by pushing nil markers for false conditions and filtering during CREATE_ARRAY), and generic field access panics (fixed by adding bounds checks in `Type::substitute()`).

### Package Management Integration (Plans 078, 079)

The auto-man package manager was migrated into the auto-lang monorepo as a separate crate (`crates/auto-man/`). Plan 078 established the foundation: a `ModuleResolver` trait in auto-lang providing `resolve()`, `get_std_root()`, `exists()`, and `search_paths()` methods, with a `FilesystemResolver` reference implementation. Plan 079 completed the full migration of the 6,400-line auto-man codebase including pac.at parsing, dependency resolution, build target configuration, and builder backends (CMake, IAR, GHS, Ninja). The `AutoManResolver` implements the `ModuleResolver` trait, supporting standard library resolution (`std.*`), relative imports (`./module`, `../module`), and third-party package resolution from `pac.at` dependencies.

### Module Resolution (Plan 074)

Plan 074 enhanced `use` statement resolution to search multiple directories rather than only the current directory. The `find_module_file()` utility in `util.rs` searches in order: user local libraries (`~/.auto/libs/`), system-wide libraries (`/usr/local/lib/auto`, `/usr/lib/auto`), and the current directory. The parser and evaluator were both updated to convert dotted paths to directory separators (e.g., `subdir.helpers` to `subdir/helpers.at`). The parser-side implementation is complete and working, but the evaluator-side fix was applied later to ensure consistency across all execution modes.

## Open Questions

- **Escape analysis for closures**: Allowing `.view`/`.mut` capture in non-escaping closures requires dataflow analysis. The current conservative approach (blocking all borrow capture) is safe but restrictive.
- **True async channel operations**: SEND and RECV currently use busy-wait with yield. Moving to true `tokio::sync::mpsc` await would improve efficiency but requires restructuring the execution loop.
- **User-defined generic instantiation**: Object literal syntax (`Pair { key: 1, val: "a" }`) works, but function call syntax with named parameters (`Pair(key: 1, val: "a")`) does not.
- **File-based test framework scope**: Plan 177 needs to define the exact assertion format and migration path for 3000+ lines of inline tests without breaking existing CI.
- **Enum data variants**: Plan 192 must decide whether data-carrying enum variants use the same GenericInstanceData mechanism as type instances or a separate representation.

## Source Plans

- [039-vm-tests-migration.md](../plans/039-vm-tests-migration.md)
- [068-autovm-bigvm.md](../plans/068-autovm-bigvm.md)
- [069-autovm-global-vars.md](../plans/069-autovm-global-vars.md)
- [070-bigvm-iterator.md](../plans/070-bigvm-iterator.md)
- [071-bigvm-closures.md](../plans/071-bigvm-closures.md)
- [073-bigvm-migration-roadmap.md](../plans/073-bigvm-migration-roadmap.md)
- [074-use-statement-multi-dir-search.md](../plans/074-use-statement-multi-dir-search.md)
- [075-config-template-modes.md](../plans/075-config-template-modes.md)
- [076-bigvm-generic-type-support.md](../plans/076-bigvm-generic-type-support.md)
- [077-unified-object-registry.md](../plans/077-unified-object-registry.md)
- [078-automan-integration.md](../plans/078-automan-integration.md)
- [079-automan-full-migration.md](../plans/079-automan-full-migration.md)
- [080-autovm-stack-frame-bug.md](../plans/080-autovm-stack-frame-bug.md)
- [081-autovm-default-mode.md](../plans/081-autovm-default-mode.md)
- [087-autovm-generics-type-erasure-specialization.md](../plans/087-autovm-generics-type-erasure-specialization.md)
- [117-vm-type-coercion.md](../plans/117-vm-type-coercion.md)
- [118-vm-test-failures-analysis.md](../plans/118-vm-test-failures-analysis.md)
- [127-autovm-task-system-execution.md](../plans/127-autovm-task-system-execution.md)
- [177-vm-file-test-framework.md](../plans/177-vm-file-test-framework.md)
- [191-assert-and-precise-linker-errors.md](../plans/191-assert-and-precise-linker-errors.md)
- [192-vm-enum-ext-codegen.md](../plans/192-vm-enum-ext-codegen.md)
- [194-monomorphic-dispatch.md](../plans/194-monomorphic-dispatch.md)
- [197-vm-adt-generic-lists-pattern-debug.md](../plans/197-vm-adt-generic-lists-pattern-debug.md)
- [198-native-metadata-from-source.md](../plans/old/198-native-metadata-from-source.md)
- [199-vm-interactive-debugger.md](../plans/old/199-vm-interactive-debugger.md)
- [200-vm-missing-features-examples-14-33.md](../plans/200-vm-missing-features-examples-14-33.md)
- [201-vm-four-pillars-enum-closure-result-spec.md](../plans/201-vm-four-pillars-enum-closure-result-spec.md)
- [203-native-registry-namespace.md](../plans/203-native-registry-namespace.md)
- [206-closure-hof-call-closure-api.md](../plans/206-closure-hof-call-closure-api.md)
- [207-enum-multi-field-destruct-construction.md](../plans/207-enum-multi-field-destruct-construction.md)
- [208-result-heap-object.md](../plans/208-result-heap-object.md)
- [212-rust-ffi-e2e.md](../plans/212-rust-ffi-e2e.md)
- [221-nanboxing-migration.md](../plans/221-nanboxing-migration.md)
