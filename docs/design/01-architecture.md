# 01 - Architecture

## Status

**Implemented**: Lexer, parser, AST, evaluator, C transpiler (a2c), Rust transpiler (a2r), AutoVM bytecode interpreter, incremental compilation (AIE) with Database and CompileSession, TypeStore, error reporting with miette, REPL with persistent sessions.

**Partial**: QueryEngine smart caching (deferred), OS abstraction layer (task concept exists but Os.Process/Os.Thread not implemented).

**Planned**: Self-hosting compiler (auto/ directory), OS-level process/thread/task unification.

## Design

### Compilation Pipeline

The AutoLang compiler supports three execution modes from a single source:

```
Source Code (.at files)
    |
Lexer (lexer.rs) -> Tokens
    |
Parser (parser.rs) -> AST (ast.rs)
    |
+-> Evaluator (eval.rs) -> Value (REPL/execution)
+-> AutoVM (vm/) -> Bytecode (VM execution)
+-> C Transpiler (trans/c.rs) -> C code
+-> Rust Transpiler (trans/rust.rs) -> Rust code
```

The pipeline has four major stages:

1. **Lexing** (`lexer.rs`, `token.rs`): Tokenizes source code including f-string interpolation (`$var` and `${expr}`).
2. **Parsing** (`parser.rs`): Recursive descent parser that builds AST nodes. Handles expression precedence, control flow, and the unified enum/type/spec/task declaration syntax.
3. **AST** (`ast.rs` and submodules): The central data structure. Expression types cover literals, binary/unary ops, calls, indexing, arrays, if-blocks, and lambda. Statement types cover storage bindings, loops, returns, use/import, and type/enum/spec declarations.
4. **Backend dispatch**: The AST feeds into one of four backends -- the direct evaluator, the AutoVM bytecode compiler, the C transpiler, or the Rust transpiler.

### Core Components

**Value System** (`crates/auto-val/`): Runtime values with dynamic type tags -- `int`, `uint`, `float`, `bool`, `str`, `array`, `object`, `nil`, `func`, `native`. Node-based data structures for complex values.

**TypeStore** (`types.rs`): A unified type registry serving as the single source of truth for type declarations, enum declarations, function declarations, spec declarations, generic templates, and type aliases. Consumers (parser, codegen, inference) all read from and write to this shared store. Implemented with `Rc<T>` for cheap shared references behind `Arc<RwLock<TypeStore>>`.

**Inference Engine** (`infer/`): A modular type inference system implementing Robinson unification with occurs check. Supports 20+ expression types, scope management, and type coercion. Currently standalone -- parser integration is deferred.

**Transpilers** (`trans/`): The C transpiler targets embedded systems (no heap allocation required). The Rust transpiler targets native applications. Both share the same AST input.

### Incremental Compilation (AIE)

The AIE (Auto Incremental Engine) architecture separates compile-time from runtime state:

- **Database** (`database.rs`): Stores source files, parsed fragments (functions, types), symbol tables, dependency graphs, and content hashes. Wrapped in `Arc<RwLock<Database>>` for safe sharing.
- **Indexer** (`indexer.rs`): Converts AST into Database fragments.
- **CompileSession** (`compile.rs`): Manages incremental compilation. Exposes `compile_source()` and `reindex_source()` with a persistent Database across compilations.
- **ExecutionEngine** (`runtime.rs`): Runtime state (stack frames, function calls, VM references) completely separated from compile-time data.

The "circuit breaker" (熔断) mechanism invalidates caches when function signatures change. If a signature is unchanged, cached bytecode and types are reused. If changed, dependents are marked dirty and recompiled.

**API entry points** (in `lib.rs`):
- `run(code)` -- basic one-shot execution
- `run_autovm(code)` -- AutoVM-based execution
- `run_with_session(session, code)` -- incremental compilation
- `run_file(path)` -- file-based execution

### OS Abstraction Layer

AutoLang follows a "Language as OS" (LaOS) philosophy, providing virtual OS concepts at the language level:

| Concept | Auto Keyword | Analog |
|---------|-------------|--------|
| Os.Process | Future | OS process |
| Os.Thread | Future | OS thread |
| Task | `task` | Coroutine/fiber |

The `task` keyword defines a concurrency unit with `@Task` lifetime scope. Tasks support `on <duration>` event handlers and are managed via `.start()` and `.end()` methods. Variables declared inside a task have `@Task` lifetime -- they live until the task ends.

```auto
task blink {
  mut color = Red
  on 10ms {
    // toggle color every 10ms
  }
}

fn main {
  let t = blink.start()
  t.end()
}
```

Task definitions are parsed as AST nodes (`ast/task.rs`) and compiled to AutoVM bytecode with handler registration.

### Test Infrastructure

- **a2c tests** (`test/a2c/`): Numbered directories with `.at` input and `.expected.c/.expected.h` output. Ranges 000-099 for core features, 100-199 for stdlib.
- **a2r tests** (`test/a2r/`): Same structure with `.expected.rs` output.
- **VM tests** (`test/vm/`): Organized by feature area (control flow, strings, generics, etc.).
- All transpiler tests run via `cargo test -p auto-lang -- trans`.

## Open Questions

- QueryEngine integration: How to reconcile `Arc<Database>` with `Arc<RwLock<Database>>` for smart caching.
- Self-hosting strategy: The `auto/` directory exists but the bootstrap compiler is not yet functional.
- OS abstraction: Whether Os.Process and Os.Thread should be language-level concepts or library-level abstractions.

## Source Documents

- [raw/architecture.md](raw/architecture.md)
- [raw/os.md](raw/os.md)
