# Architecture & Implementation Details

> Extracted from CLAUDE.md for reference. See CLAUDE.md for rules and quick reference.

## Context Management

- When approaching context limits, summarize progress and create a continuation plan before the session ends
- Use concise responses during long debugging sessions to preserve context

## Architecture Overview

### Compilation Pipeline

The AutoLang Rust compiler supports three execution modes:

```
Source Code (.at files)
    ↓
Lexer (lexer.rs) → Tokens
    ↓
Parser (parser.rs) → AST (ast.rs)
    ↓
├─→ Evaluator (eval.rs) → Value (REPL/execution)
├─→ C Transpiler (trans/c.rs) → C code
└─→ Rust Transpiler (trans/rust.rs) → Rust code
```

### Core Components (Rust Implementation)

#### 1. **Lexer** (`crates/auto-lang/src/lexer.rs`)
- Tokenizes AutoLang source code
- Handles f-strings with `$variable` and `${expression}` syntax
- Token types defined in `token.rs`

#### 2. **Parser** (`crates/auto-lang/src/parser.rs`)
- Recursive descent parser consuming lexer tokens
- Builds AST nodes defined in `ast.rs`
- Handles expression precedence and control flow
- Uses `AutoStr` for string memory management

#### 3. **AST** (`crates/auto-lang/src/ast.rs`)
- Unified representation for expressions and statements
- Expression types: `int`, `ident`, `binary`, `unary`, `if`, `array`, `call`, `index`, etc.
- Statement types: `expr`, `store`, `for`, `while`, `break`, `ret`, `use`, etc.

#### 4. **Evaluator** (`crates/auto-lang/src/eval.rs`)
- Interprets AST nodes to produce `Value` results
- Supports multiple evaluation modes (SCRIPT, CONFIG, TEMPLATE)
- Uses `Universe` for variable scoping

#### 5. **Value System** (`crates/auto-val/src/`)
- Dynamic typing with runtime type tags
- Types: `int`, `uint`, `float`, `bool`, `str`, `array`, `object`, `nil`, `func`, `native`
- Node-based data structures for complex values

#### 6. **Transpilers** (`crates/auto-lang/src/trans/`)
- **C Transpiler** (`c.rs`): Transpiles AutoLang to C for embedded systems
- **Rust Transpiler** (`rust.rs`): Transpiles AutoLang to Rust for native apps

#### 7. **AIE (Auto Incremental Engine)** (`crates/auto-lang/src/database.rs`, `compile.rs`, `runtime.rs`)
- **Database** (`database.rs`): Compile-time data storage with dirty tracking
  - Stores source files, fragments, AST nodes, symbol tables
  - Tracks file dependencies and propagation of changes
  - Interface hashing for signature-based cache validation (熔断)
- **Indexer** (`indexer.rs`): Converts AST into Database fragments
- **CompileSession** (`compile.rs`): Incremental compilation session manager
  - `compile_source()`: Compile source with incremental support
  - `reindex_source()`: Re-index modified files
  - Persistent Database across compilations
- **ExecutionEngine** (`runtime.rs`): Runtime state separated from compile-time
  - Stack frames, function calls, VM references
  - Clean separation: Database (compile-time) vs ExecutionEngine (runtime)

**Incremental Compilation Architecture (Plan 063-065)**:
```
┌─────────────────────────────────────────────────────────────┐
│                    User Code (.at files)                    │
│                                                              │
│  fn add(a int, b int) int { a + b }                         │
│  fn main() { print(add(1, 2)) }                               │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│                  CompileSession (Persistent)                 │
│  ┌────────────────────────────────────────────────────┐     │
│  │ Database (Arc<RwLock<Database>>)                   │     │
│  │  - Files: source code storage                      │     │
│  │  - Fragments: parsed functions/types                │     │
│  │  - SymbolTables: compile-time symbols               │     │
│  │  - DepGraph: file dependencies                      │     │
│  │  - HashCache: content hash tracking                 │     │
│  └────────────────────────────────────────────────────┘     │
│                           ↓                                  │
│  QueryEngine (smart caching with 熔断)                       │
└─────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│              ExecutionEngine (Per-Execution)                 │
│  - Stack frames, function calls                             │
│  - VM references (StringBuilder, List, HashMap, etc.)       │
│  - Runtime state only (no compile-time data)                │
└─────────────────────────────────────────────────────────────┘
```

**Key Concepts**:
- **Compile-time vs Runtime**: Database (compile-time) is separate from ExecutionEngine (runtime)
- **Incremental Updates**: Only recompile changed files, reuse cached artifacts
- **熔断 (Circuit Breaker)**: Cache invalidation based on interface hash changes
  - If function signature unchanged → cache valid (reuse bytecode, types, etc.)
  - If function signature changed → cache invalid → recompile dependents
- **Dirty Propagation**: Track file dependencies, mark dependents as dirty

**API Entry Points** (lib.rs):
```rust
// Basic execution (no Database)
run(code: &str) -> AutoResult<String>

// Incremental compilation with persistent session
run_with_session(session: &mut CompileSession, code: &str) -> AutoResult<String>

// REPL session management (repl.rs)
let mut session = ReplSession::new();
session.run(code);  // Uses persistent CompileSession
session.stats();    // Get compilation statistics
session.reset_runtime();  // Clear runtime, keep compile-time data
```

**Implementation Status** (2025):
- ✅ **Phase 1-4 (Plan 065)**: REPL integration with CompileSession
  - `ReplSession` struct with persistent Database
  - `run_with_session()` API for incremental compilation
  - REPL commands: `:stats`, `:reset`, `:help`, `:quit`
- ⏸️ **Phase 3 (Plan 065)**: QueryEngine smart caching
  - Deferred: Needs `Arc<Database>` vs `Arc<RwLock<Database>>` reconciliation
- 🔄 **Plan 064**: Split Universe → Database + ExecutionEngine (60% complete)
- 🔄 **Plan 063**: AIE core architecture (70% complete)

### Test Infrastructure

#### a2c (Auto-to-C) Tests
Located in `crates/auto-lang/test/a2c/`:
- Test cases organized by number (e.g., `000_hello/`, `021_type_error/`)
- Each test has `.at` source file and `.expected.c`/`.expected.h` output files
- Run with: `cargo test -p auto-lang -- trans`

#### a2r (Auto-to-Rust) Tests
Located in `crates/auto-lang/test/a2r/`:
- Test cases organized by number (e.g., `000_hello/`, `029_composition/`)
- Each test has `.at` source file and `.expected.rs` output file
- Run with: `cargo test -p auto-lang -- trans`

## Implementation Strategy

### Primary Implementation: Rust (`crates/`)

The Rust implementation in `crates/` is the canonical AutoLang compiler with:
- Full language feature support
- Three execution modes (evaluator, C transpiler, Rust transpiler)
- Comprehensive error reporting with miette
- Type inference and type checking (see [Type Inference System](#type-inference-system-rust-implementation) below)

### Self-Hosting Strategy (Future)

The self-hosted compiler represents a future bootstrap effort:
1. Rust compiler (`crates/`) → implements full language
2. Auto compiler (`auto/`) → written in AutoLang, compiled by itself

This will create a self-sustaining ecosystem where AutoLang can compile itself.
