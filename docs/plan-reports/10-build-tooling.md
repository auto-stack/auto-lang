# 10 - Build Tooling and Package Management

## Overview

AutoLang's build infrastructure spans two major domains: the compiler-internal AIE (Auto Incremental Engine) for incremental compilation, and the external tooling chain (AutoMan, Auto CLI, package management) for project lifecycle management. The AIE system, delivered across Plans 063--066, migrated the compiler from a stateful file-based architecture to a query-based incremental model with BLAKE3 hashing, dependency-graph-driven dirty propagation, and a熔断 (circuit-breaker) interface-hash scheme that achieved 1.86--2.67x transpilation speedups. In parallel, the Rust FFI sandbox (Plan 092) enabled the AutoVM to dynamically load Rust crates at runtime. The remaining plans chart the path toward a unified CLI, the B.P.B.E multi-target build architecture, Tauri IPC code generation, structured shell commands, and a bun-based JavaScript toolchain.

## Plan Index

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 063 | AIE Architecture Migration | Partial | Multi-phase migration from file-based to query-based incremental compilation; Phases 1--3.6 (PC-Server) complete, MCU runtime deferred |
| 064 | Split Universe into Compile-time and Runtime | Complete | Separate Universe into AIE Database (compile-time) and ExecutionEngine (runtime); hybrid architecture accepted as stable final state |
| 065 | AIE Integration with lib.rs Entry Points | Complete | Wire QueryEngine, CompileSession, and persistent Database into public API; REPL uses incremental compilation |
| 066 | Incremental Transpilation Database Migration | Complete | Migrate C/Rust transpilers to AIE caching; 2.67x C speedup, 1.86x Rust speedup |
| 092 | Rust FFI via Sandbox Compilation | Complete | Enable AutoVM to dynamically load Rust crates at runtime via sandboxed ABI-stable compilation |
| 093 | AutoMan Redesign for Rust Support | Planned | Extend auto-man to support Auto/Rust mixed projects alongside existing Auto/C projects |
| 111 | Auto CLI Refactor and Unification | Planned | Merge auto-man CLI into universal auto CLI with standardized command structure |
| 112 | AutoMan B.P.B.E Architecture Refactor | Planned | Redesign AutoMan around Backend, Port, Builder, Export architecture for multi-target builds |
| 146 | AutoShell SmartCmd Integration | Complete | Integrate sysinfo/uutils libraries for structured shell command output and cross-platform support |
| 151 | Tauri IPC Mode for api-example | Planned | Generate complete Tauri IPC backend by transpiling api.at + db.at to Rust |
| 186 | Switch from npm to bun for Vue/Web Projects | Planned | Replace npm with bun for faster installs via global cache hard-linking |
| 212b | Rust FFI E2E Dynamic Loading | Complete | dep serde_json -> cargo build cdylib -> AutoVM load .dll -> call |
| 214 | Python FFI (use.py) | Complete | Embed Python interpreter in AutoVM (placeholder, blocked on Plan 212b) |
| 216 | C FFI Build Pipeline Integration | Complete | CLI integration for C FFI bindgen into build pipeline |
| 202 | Auto Playground | Complete | Web-based code editor + VM execution + transpilation viewer (Vue 3 + axum backend) |
| 212a | LSP + VSCode Extension Modernization | Complete | TextMate grammar rewrite, LSP completion sync, Document Symbols, code snippets |
| 219 | Playground Source Map | Complete | Source map generation for playground editor with AST-to-output position mapping |
| 225 | Playground Interactive Debugger | Complete | Browser-based VM debugger with breakpoints, step controls, stack/locals panels via WebSocket |

## Status

**Implemented**: 063 (partial), 064, 065, 066, 092, 146, 202, 212a, 212b, 214, 216, 219, 225
**Partial**: 063 (MCU phases deferred)
**Planned**: 093, 111, 112, 151, 186

## Design

### AIE Incremental Compilation (Plans 063, 064, 065, 066)

The AIE (Auto Incremental Engine) is AutoLang's query-based incremental compilation system, inspired by Rust's query model. Its core philosophy shifts the compiler from a one-shot "process" to a persistent "database": the compiler maintains a Database of sources, fragments, types, and dependencies, and a QueryEngine lazily computes derived results with caching.

The migration unfolded in three architectural phases. Phase 1 (Plan 063, ~1,450 lines) established the foundation: a `Database` struct with stable `FileId`/`FragId` identifiers, an `Indexer` that splits ASTs into declaration-level fragments and registers them, a `CompileSession` API wrapping the Database, and an `ExecutionEngine` struct for runtime-only state. The parser was refactored into a pure function with no side effects on the old `Universe`.

Phase 2 (Plan 063, ~1,070 lines) introduced file-level incremental compilation. BLAKE3 hashing detects changed files; a `FileDependencyGraph` tracks `use` statement relationships; recursive BFS-based dirty propagation marks transitive dependents; and a `QueryEngine` with `DashMap`-based caching provides trait-based queries (`GetTypeQuery`, `GetBytecodeQuery`, `GetFileDepsQuery`, etc.). Tests confirmed that unchanged files are skipped entirely, import chains propagate correctly, and diamond dependencies are handled.

Phase 3 (Plan 063, ~3,800 lines across multiple modules) added fine-grained fragment-level incremental compilation. Three hash levels -- L1 (source text), L2 (AST structure), L3 (interface/signature) -- enable the熔断 optimization: when a function body changes but its signature does not, dependent recompilation is short-circuited. A `DepScanner` tracks fragment-level call dependencies. The query engine was extended with LRU cache eviction and advanced queries for IDE support (`InferExprTypeQuery`, `GetSymbolLocationQuery`, `FindReferencesQuery`, `GetCompletionsQuery`). Patch structures (`Patch`, `Reloc`, `RelocKind`) were implemented for hot-reload, though MCU runtime integration remains deferred pending hardware availability.

The Universe split (Plan 064) was the critical enabler. The monolithic `Universe` struct mixed 19 fields spanning compile-time (scopes, ASTs, types, specs) and runtime (values, VM refs, call stack). The migration classified every field, extended `ExecutionEngine` with 11 runtime fields (values, VM references, shared values, builtins, evaluator pointer), and extended the AIE Database with 7 compile-time fields (scopes, types, type aliases, specs, code_paks). A `SymbolTable` / `StackFrame` split was designed and partially implemented: `SymbolTable` holds compile-time declarations (symbols, types, hierarchy) while `StackFrame` holds runtime state (variable values, moved variables, execution position). The hybrid architecture was accepted as the stable final state: Evaler bridge methods try `Database`/`ExecutionEngine` first, falling back to Universe for VM modules. VM function signatures were migrated from `fn(Shared<Universe>, ...)` to `fn(&mut Evaler, ...)`, touching ~53 VM implementations. The final test count: 998/1006 passing (99.2%), with zero new regressions.

Plan 065 wired the AIE into the public API. `QueryEngine` was integrated with `CompileSession` using `Arc<RwLock<Database>>`. The REPL's `ReplSession` provides persistent incremental compilation across inputs. The old `run()` API remains for one-shot scripts; `run_with_session()` enables incremental mode.

Plan 066 migrated both transpilers to the Database. The Rust transpiler (2,287 lines, 97% test pass) was migrated first as a lower-risk proof of concept. The C transpiler (3,945 lines) followed using the same hybrid pattern: `with_database()` constructor, unified `lookup_type()`/`lookup_meta()` helpers that try Database first and fall back to Universe. The measured speedups: C transpiler 2.67x (5.5ms to 2.1ms), Rust transpiler 1.86x (3.3ms to 1.8ms), with 100% cache hit rate after hashing. Post-completion, dependency propagation was upgraded from single-level to recursive BFS, ensuring standard library changes automatically trigger recompilation of all indirect dependents.

### Rust FFI Sandbox (Plan 092)

The Rust FFI sandbox solves a fundamental problem: Rust's ABI is not stable across compilations, making it unsafe to pass native types across dynamic library boundaries. The solution controls all compilation through a sandbox at `~/.auto/sandbox/` that ensures the same toolchain, same dependencies, and shared `libstd.so`. When the standard library is shared between AutoVM and loaded libraries, `Vec<T>` has the same memory layout everywhere.

The syntax uses `dep` for declaration and `use.rust` for import:

```auto
dep serde(version: "1.0", features: ["derive"])
use.rust serde::json::{from_str, to_string}
```

The implementation spans six phases. The AutoCache was extended with `RustCrateLibrary` and `RustCrateSource` artifact types. A `Sandbox` struct manages toolchain installation, crate compilation, and dynamic loading. A `CrateRegistry` backed by SQLite tracks compiled crates with ABI compatibility metadata. The `RustFfiBridge` in `ffi.rs` provides `load_rust_crate()` with ABI verification, `register_function()` with argument marshaling (i32, f32, f64, pointers, strings, bytes, callbacks), and supports 40+ function signature patterns. Parser support for `dep` and `use.rust` was added to the lexer, parser, and AST. The `CompileSession` tracks declared crates and resolves dependencies. All 9 FFI tests and 10 sandbox/registry tests pass.

### AutoMan and CLI Architecture (Plans 093, 111, 112)

AutoMan (`auto-man` / `am`) is AutoLang's build orchestrator, currently optimized for Auto/C mixed projects. Three planned changes will transform it into a multi-language, multi-target build system.

Plan 093 introduces Rust as a first-class target. Currently `auto-man` assumes everything transpiles to C via CMake/Ninja/IAR/GHS. The plan adds a `lang` field to `pac.at` (defaulting to `"c"` for backward compatibility), a `CargoBuilder` implementation of the `Builder` trait that generates dynamic `Cargo.toml` files, and a virtual Cargo workspace at the build directory root. Each `Target` parsed from `pac.at` becomes a Cargo crate; inter-target dependencies map to Cargo path dependencies. File type registration gains a `RustSource` (`.rs`) variant.

Plan 111 unifies the CLI. The current `auto` and `auto-man` (`am`) CLIs will merge into a single `auto` binary. Old flat commands (`App`, `Lib`, `Capp`, `Vue`, `Tauri`, `Devices`, `Port`) collapse into a subcommand structure: `auto new <name> -t <template>`, `auto build`, `auto run`, `auto fetch`, `auto device list/select`, `auto env reset/install`. The `pac.at` file controls project backend ("smart configuration"), while the CLI provides a minimal and stable set of verbs ("dumb CLI"). The `auto-man` binary will emit a deprecation warning pointing users to `auto`.

Plan 112 redesigns AutoMan around the B.P.B.E (Backend, Port, Builder, Export) architecture. Backend defines the language ecosystem (`c`, `rust`, `vue`, `jet`). Port defines the target environment and hardware (e.g., `stm32` combining MCU, SDK, and toolchain). Builder is the internal orchestration engine (Ninja for C, Cargo for Rust, Vite for Vue). Export generates third-party IDE files (CMake, IAR, GHS, Keil). This resolves ambiguous terminology and overloads: CMake and IAR become "exports" rather than primary builders; SDKs are nested inside port blocks as dependencies; toolchain paths live in global `~/.auto/am.at` rather than project configuration.

### AutoShell SmartCmd (Plan 146)

AutoShell's SmartCmd integration brings structured output and cross-platform compatibility to shell commands. The implementation uses a mixed reuse strategy: `sysinfo` for system and process information, `chrono` for timestamps, and custom implementations for file operations.

The core types (`AshFileEntry`, `AshProcessEntry`, `AshDiskEntry`, `AshCpuInfo`, `AshMemoryInfo`) provide structured representations of command output. A conversion layer (`metadata_to_entry`, `file_entry_to_value`) bridges between filesystem metadata and AutoLang's `Value` type. The `ls` command was refactored to produce structured `AshFileEntry` arrays with sorting (alphabetical, time-based, directories-first) and filtering (hidden files). New commands `ps` (process listing sorted by CPU usage) and `sys` (with `disks`, `cpu`, `mem` subcommands) were implemented using `sysinfo`. File operation commands (`cp`, `mv`, `rm`, `mkdir`) were integrated with recursive and force-mode support. Phase 4 (natural language interface via SmartCmd trait) is deferred pending AutoLang Agent functionality. Nine integration tests cover all structured commands and file operations.

### Tauri IPC Backend Generation (Plan 151)

Plan 151 targets a complete Tauri IPC backend generated from AutoLang source. The architecture transpiles `api.at` (API endpoint definitions with `#[api]` annotations) and `db.at` (business logic with global mutable state) into an independent `rust/` crate that integrates with Tauri's thin shell.

The transpilation maps AutoLang global `var` declarations to Rust's `static X: Lazy<Mutex<T>>` pattern using `once_cell`. Function calls become `X.lock().unwrap()` accessors. The `#[api]` annotation maps to `#[tauri::command]`. The a2r transpiler was extended with method-call translations (`.to_lower()` to `.to_lowercase()`, `.length()` to `.len()`, `.contains(s)` to `.contains(&s)` with auto-borrow). The generated crate structure separates `types.rs` (serde-serializable structs), `db.rs` (global state + business functions), and `commands.rs` (Tauri command wrappers). A new `backend: "rust-tauri"` entry in `pac.at` controls the build flow. The plan includes closure-with-block-body support for filter/map patterns.

### Bun Package Manager Migration (Plan 186)

Plan 186 replaces npm with bun across all Vue/web project tooling in auto-man. Bun provides faster installs via global cache hard-linking: after the first install, projects sharing the same dependencies (shadcn-vue, tailwind, vue) link instantly. A shared `pkg` module in `crates/auto-man/src/pkg.rs` auto-detects bun (falling back to npm silently), replacing 21 hardcoded `npm`/`npx` references across 6 files (vue.rs, tauri.rs, vscode.rs, builder/vue.rs, cmd_vue.rs, cmd_tauri.rs). The module provides `detect()`, `install()`, `run_script()`, `exec()`, and `add_packages()` functions with a Windows `cmd /C` wrapper. Generated `package.json` scripts remain `npm run` compatible since they execute in the user's environment, not ours.

## Open Questions

- **MCU hot-reload**: Phases 3.6b--3.8 of Plan 063 require MCU hardware/infrastructure. The RAM overlay, GOT update mechanism, and debugger protocol are designed but untested on real hardware.
- **Parser/Indexer migration**: Plans 064 Phases 5--6 (migrating Parser from `Shared<Universe>` to `Database`) are deferred as breaking API changes. The corrected migration path requires replacing ~50--100 Parser call sites.
- **B.P.B.E validation**: Plan 112's separation of Builder (internal) from Export (external) needs integration testing across all backend types (c, rust, vue, jet).
- **Rust crate version resolution**: Plan 092 leaves open how to resolve version conflicts and manage garbage collection of unused crates in the sandbox.
- **Tauri closure bodies**: Plan 151's support for complex closure bodies with multiple statements, return statements, and mutable captures is partially implemented.

## Source Plans

- Plan 063: `docs/plans/063-aie-architecture-migration.md`
- Plan 064: `docs/plans/064-split-universe-compile-runtime.md`
- Plan 065: `docs/plans/065-aie-lib-integration.md`
- Plan 066: `docs/plans/066-incremental-transpilation.md`
- Plan 092: `docs/plans/092-rust-ffi-sandbox.md`
- Plan 093: `docs/plans/093-automan-rust-support.md`
- Plan 111: `docs/plans/111-auto-cli-refactor.md`
- Plan 112: `docs/plans/112-automan-bpbe-architecture.md`
- Plan 146: `docs/plans/146-ash-smartcmd-integration.md`
- Plan 151: `docs/plans/151-tauri-ipc-mode.md`
- Plan 186: `docs/plans/186-bun-package-manager.md`
- [212-rust-ffi-e2e.md](../plans/old/212-rust-ffi-e2e.md)
- [214-python-ffi-use-py.md](../plans/214-python-ffi-use-py.md)
- [216-cffi-bindgen.md](../plans/216-cffi-bindgen.md)
- [202-playground-design.md](../plans/old/202-playground-design.md)
- [212-lsp-vscode-modernization.md](../plans/212-lsp-vscode-modernization.md)
- [219-playground-source-map.md](../plans/219-playground-source-map.md)
- [225-playground-interactive-debugger.md](../plans/old/225-playground-interactive-debugger.md)
