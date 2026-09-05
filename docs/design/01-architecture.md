---
status: active
describes: current-state
last_verified_at: 2026-09-04
verified_by_plan: PLAN-543
---

# 01 - Architecture

## Status

**Implemented**: Lexer, parser, AST, TypeStore and semantic passes, AutoVM bytecode execution,
multi-target transpilation, AutoUI generation/runtime, incremental compilation (AIE) with Database,
CompileSession and QueryEngine, error reporting, and persistent sessions.

**Partial**: OS abstraction layer（task/actor/runtime bridges 已有实现，Process/Thread/Task 的
最终语言、标准库与 host 边界仍待 target-state 设计收敛）。

**Experimental**: `auto/lib/` 中的 self-hosted compiler 已形成 token、lexer、parser、typeinfo、
codegen、engine、a2r 分阶段链路；Rust compiler 仍是 canonical reference。

> 本文描述仓库级 current-state。模块细节以 `docs/specs/` 为准；未来方案必须进入带
> `describes: target-state` 的 Design/RFC。

## Design

### Compilation Pipeline

The AutoLang compiler supports multiple execution and generation modes from a single source:

```
Source Code (.at files)
    |
Lexer (lexer.rs) -> Tokens
    |
Parser (parser.rs) -> AST (ast.rs)
    |
+-> AutoVM (vm/) -> Bytecode -> engine (canonical script/REPL execution)
+-> Transpilers (trans/) -> C/Rust/JS/TS/Python/GDScript/r2a
+-> AutoUI -> AURA/ui_gen/VTree -> iced/gpui/Vue/headless
```

The pipeline has four major stages:

1. **Lexing** (`lexer.rs`, `token.rs`): Tokenizes source code including f-string interpolation (`$var` and `${expr}`).
2. **Parsing** (`parser.rs`): Recursive descent parser that builds AST nodes. Handles expression precedence, control flow, and the unified enum/type/spec/task declaration syntax.
3. **AST** (`ast.rs` and submodules): The central data structure. Expression types cover literals, binary/unary ops, calls, indexing, arrays, if-blocks, and lambda. Statement types cover storage bindings, loops, returns, use/import, and type/enum/spec declarations.
4. **Backend dispatch**: The semantic model feeds AutoVM, the multi-target transpiler family, or
   AutoUI generation/runtime. The historical direct evaluator has been removed; public execution
   uses AutoVM.

### Core Components

**Value System** (`crates/auto-val/`): Runtime values with dynamic type tags -- `int`, `uint`, `float`, `bool`, `str`, `array`, `object`, `nil`, `func`, `native`. Node-based data structures for complex values.

**TypeStore** (`types.rs`): A unified type registry serving as the single source of truth for type declarations, enum declarations, function declarations, spec declarations, generic templates, and type aliases. Consumers (parser, codegen, inference) all read from and write to this shared store. Implemented with `Rc<T>` for cheap shared references behind `Arc<RwLock<TypeStore>>`.

**Inference Engine** (`infer/`): Modular type inference and unification used together with resolver,
type checking, ownership and comptime passes. Detailed coverage is maintained in the types Spec.

**Transpilers** (`trans/`): Multi-target code generation for C, Rust, JavaScript, TypeScript, Python,
GDScript and r2a; targets share the language frontend while retaining target-specific lowering/runtime.

### Incremental Compilation (AIE)

The AIE (Auto Incremental Engine) architecture separates compile-time from runtime state:

- **Database** (`database.rs`): Stores source files, parsed fragments (functions, types), symbol tables, dependency graphs, and content hashes. Wrapped in `Arc<RwLock<Database>>` for safe sharing.
- **Indexer** (`indexer.rs`): Converts AST into Database fragments.
- **CompileSession** (`compile.rs`): Manages incremental compilation. Exposes `compile_source()` and `reindex_source()` with a persistent Database across compilations.
- **QueryEngine** (`query/`): Integrated query/cache layer created on demand from the shared Database,
  reused during a session and reset with session cache invalidation.
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

- Self-hosting maturity: when the experimental Auto implementation can assume bootstrap/release duties.
- OS abstraction: Whether Os.Process and Os.Thread should be language-level concepts or library-level abstractions.
- Backend parity: which cross-backend semantic guarantees should become executable architecture fitness functions.

## Historical Corrections

- `eval.rs`/direct Evaluator is historical and is not a current execution backend.
- “QueryEngine integration deferred” was resolved by the current CompileSession/Database integration.
- VM slot width and opcode counts are deliberately not duplicated here; current VM Spec and code are
  the authoritative references.

## Source Documents

- [raw/architecture.md](raw/architecture.md)
- [raw/os.md](raw/os.md)
