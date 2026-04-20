# 09 - Compiler Architecture

## Status

**Implemented:**
- AIE (Auto Incremental Engine) core: `Database`, `Indexer`, `QueryEngine` modules are in production (`compile.rs`, `database/mod.rs`, `indexer.rs`, `query.rs`). `CompileSession` manages the full pipeline: parse, index, query.
- AutoCache module-level caching: file hash validation, dependency tracking, interface hash for cache invalidation (`auto_cache.rs`).
- Comptime execution (Phase 1): `#if`, `#for`, `#is` conditional compilation parsed and transformed via the CTEE module (`comptime/mod.rs`, `comptime/transformer.rs`). The CTEE reuses the existing `VmInterpreter`.
- Unified CLI (`auto`): full command set including `new`, `build`, `run`, `clean`, `fetch`, `deps`, `device`, `info`, `env`, `upgrade`, `gen`, `export` (`crates/auto/src/main.rs`). Supports `--ai` mode for JSON-structured diagnostics.
- Multi-mode compilation pipeline: AutoVM, C transpilation, and Rust transpilation modes coexist (`multi_mode.rs`).

**Partial / Planned:**
- Fine-grained (declaration-level) incremental compilation with fragment hashing and interface-hash breakage is designed but not yet fully operational.
- Dead code elimination (DCE) with liveness propagation is not implemented. No dependency graph with reachability tracking exists yet.
- AutoCache global CAS store (SQLite + filesystem blobs) remains a design document. Current implementation uses in-memory module caching only.
- MCU hot reloading is entirely planned.
- AI-native features (typed holes `??`, contract annotations, meta-instructions `#!`) are not implemented.

## Design

### AIE: From Process to Database

The Auto compiler uses a query-based incremental architecture called the Auto Incremental Engine (AIE). Instead of the traditional batch pipeline (`source -> AST -> binary`), the AIE treats the compiler as an in-memory database. Compilation becomes a series of queries against this database.

**Core components:**

1. **Database** -- The single source of truth. It stores two layers:
   - *Storage layer*: source text, AST fragments, symbol tables (written by the Indexer).
   - *Cache layer*: type information, bytecodes, dependency graphs (computed lazily by the QueryEngine).

2. **Indexer** -- The only component with write access to the Database. It performs resilient parsing, fragments source code into declaration-level units (functions, types, constants), and registers each fragment with a stable ID.

3. **QueryEngine** -- All compilation logic (type checking, codegen) is expressed as pure-function queries. Input is a read-only reference to the Database plus a query ID. On cache miss, computation runs and the result is cached.

**Granularity and hashing strategy:**

The AIE operates at declaration level (not file level, not statement level). To prevent cascading recompilation, a three-level hash check is used:

| Level | Check | Effect |
|---|---|---|
| L1 Text Hash | Did the source text change? | No change -> stop. |
| L2 AST Hash | Did the structure change (ignoring whitespace/comments)? | No change -> stop. |
| L3 Interface Hash | Did the function signature (params/return type) change? | No change -> **break the cascade**. Only the function itself recompiles, not its callers. |

**Change lifecycle** (example: user edits `fn calculate()`):

1. File watcher captures the change.
2. Indexer re-parses the file, locates the `fn calculate` fragment, updates its AST.
3. Database invalidates `calculate`'s cached bytecode.
4. Interface hash is computed. If the signature changed, the dependency graph marks all callers as dirty. If unchanged, propagation stops.
5. On the next query for `calculate`'s bytecode, the QueryEngine regenerates it lazily.

### AutoCache

AutoCache is a content-addressable store (CAS) for compilation artifacts. It answers: "have I compiled something with this exact fingerprint before?"

**Fingerprint calculation** combines three hashes:

1. **Content Hash**: Based on canonicalized AST (not raw text), with absolute paths remapped to relative paths for portability.
2. **Context Hash**: Target triple, compiler flags, toolchain version, capabilities configuration.
3. **Dependency Hash**: Transitively includes the hashes of all imported modules, forming a Merkle tree.

The planned storage architecture uses SQLite for metadata indexing and filesystem blobs for binary artifacts, with LRU-based garbage collection. The current in-memory implementation provides file hashing and module-level caching.

### Dead Code Elimination (Prune)

DCE integrates with incremental compilation through a "Graph-State Incremental System." The key insight is separating two questions: "did the source change?" (hash) and "is this symbol still needed?" (reachability).

**Three-step compilation flow:**

1. **Local Update**: Re-parse changed files, recompute dependency edges for modified symbols.
2. **Global Liveness Propagation**: BFS/DFS from roots (main, exports, ISRs) to compute current reachability.
3. **Reconciliation**: A decision matrix combines hash-change and reachability-change to determine the action for each symbol:

| Hash Changed? | Reachability Changed? | Action |
|---|---|---|
| No | False -> True (revived) | GENERATE |
| Any | True -> False (dead) | DELETE |
| Yes | True -> True | REGENERATE |
| Yes | False -> False | IGNORE |
| No | True -> True | SKIP |

This ensures dead code is eliminated without unnecessary I/O, and revived code is regenerated even when its source hasn't changed.

### Compile-Time Execution (Comptime)

Auto provides an explicit metaprogramming system using the `#` prefix. The design philosophy is: *the programmer can see at a glance which code runs at compile time and which runs at runtime.*

**Two-stage compilation:**
- Stage 1 (Meta-Eval): Execute all `#`-marked code, performing AST pruning and expansion.
- Stage 2 (Codegen): Type-check and compile the resulting clean AST.

**Supported constructs:**

| Syntax | Purpose |
|---|---|
| `#if cond { }` | Conditional compilation (whole if/elif/else structure) |
| `#is expr { }` | Compile-time pattern matching |
| `#for i in 0..N { }` | Loop unrolling |
| `#{ expr }` | Compile-time evaluation block (returns a value) |
| `#{var}` | Interpolation of compile-time value into runtime code |

The implementation uses the existing `VmInterpreter` for evaluation rather than building a separate evaluator, ensuring all language features are available at compile time.

### Unified CLI

The `auto` command-line tool replaces the older `am.exe` with a single entry point. The design principle is: *the CLI stays dumb (only verbs), the config file (`pac.at`) stays smart (decides backends).*

| Command | Purpose |
|---|---|
| `auto` (no args) | Enter ASH / Auto REPL |
| `auto file.at` | Run a script via AutoVM |
| `auto new NAME` | Create project (supports `-t c-app`, `-t rs-app`, etc.) |
| `auto build` | Compile based on `pac.at` backend |
| `auto run` | Build and run |
| `auto fetch` | Resolve and download dependencies |
| `auto device list/select` | Hardware management |
| `auto env cache stats/prune/clear` | Cache management |

The `--ai` flag switches output to JSON-structured diagnostics for AI consumption, with error codes, AST node paths, and expected types.

### AI-Native Intermediate Language

Auto is positioned as an "Intent IR" -- a language where code structure directly reflects logical intent, making it an effective intermediate representation for AI code generation. Key design pillars:

- **Explicit intent**: `#if` separates compile-time from runtime; `|>` (Auto Flow) maps to chain-of-thought reasoning.
- **Constraints as prompts**: The type system and contract annotations serve as hard constraints on AI output.
- **Errors as feedback**: Structured diagnostics (JSON mode) enable AI self-correction loops.

Planned features include typed holes (`??`), contract annotations (`#[pre]`, `#[post]`), and meta-instructions (`#!`), none of which are yet implemented.

### MCU Hot Reloading

The goal is sub-second iteration on embedded targets by patching individual functions in RAM rather than reflashing entire firmware.

**Basic approach:**
1. Each symbol (function/variable) is placed in its own section with padding.
2. On source change, only the modified symbol's machine code is regenerated.
3. A debugger writes the new code to the target's RAM, then updates the Global Offset Table to redirect calls.

**Fallback approach** (if padding is impractical): Use indirection tables -- each symbol gets a pointer entry at the module head, and all access goes through these pointers. To hot-swap, update the pointer without moving the code.

## Open Questions

- Should AutoCache use SQLite as designed, or remain in-memory for simplicity?
- What is the minimum MCU runtime support required for hot reloading (halt/resume, RAM write, GOT update)?
- How should DCE interact with the existing C transpiler's `static` function optimization?
- Will comptime evaluation need a target-platform simulator for cross-compilation (e.g., `usize` differs between host and target)?

## Source Documents

- [raw/incremental-compilation.md](raw/incremental-compilation.md)
- [raw/auto-cache.md](raw/auto-cache.md)
- [raw/prune.md](raw/prune.md)
- [raw/compile-time-execution.md](raw/compile-time-execution.md)
- [raw/auto-cli.md](raw/auto-cli.md)
- [raw/ai-native.md](raw/ai-native.md)
- [raw/mcu_hot_reloading.md](raw/mcu_hot_reloading.md)
