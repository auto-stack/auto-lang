# 03 - Error Handling

## Overview

Error handling in AutoLang spans three deeply interconnected domains: compiler diagnostics, runtime error reporting, and compile-time type inference. Together, these systems ensure that every error a user encounters -- whether a syntax mistake, a type mismatch, or a division-by-zero at runtime -- comes with a clear message, an exact source location, and an actionable suggestion. The project adopted the `miette` crate early on to provide Rust-compiler-grade colorful diagnostic output, then layered a full type-inference engine on top so that many classes of errors are caught before code ever runs. At the time of writing, the diagnostic system and the type-inference subsystem are complete; runtime error integration in the evaluator remains partially done.

## Plan Index

| Plan | Title | Status | Summary |
|------|-------|--------|---------|
| 008 | Error Message System | Completed | Rust-compiler-grade error reporting with miette, error codes (E0001--E0305), source locations, "did you mean?" suggestions, multi-error display, warning system, and JSON output |
| 009 | Runtime Error Integration | Partial | Replace `panic!` calls in evaluator with `RuntimeError` variants, source location tracking, and stack traces -- planned but not yet started |
| 010 | Type Inference Subsystem | Completed | Full type inference with Hindley-Milner unification, ~2,770 LOC across 7 implementation stages, 285+ unit tests, integrated into parser |
| 191 | Assert and Precise Linker Errors | Planned | Add assert/assert_eq/assert_ne intrinsics and propagate source positions into linker error spans |

## Status

**Implemented**: Comprehensive diagnostic system with miette (error codes, source snippets, multi-error recovery, warnings, JSON output, "did you mean?" suggestions); full type-inference engine with Robinson unification, expression/statement/function inference, and parser integration (~2,770 LOC, 285+ tests).

**Partial**: Runtime error integration in the evaluator (Plan 009). The `RuntimeError` enum is defined (E0301--E0305) and the infrastructure is in place, but the evaluator still contains `panic!` calls and returns raw `Value` instead of `AutoResult<Value>`. Span tracking in AST nodes and stack-trace capture are not yet implemented.

**Planned**: LSP integration for real-time IDE diagnostics; C-implementation port of the error system; generic type parameters and trait-based type inference; assert/assert_eq/assert_ne intrinsics and precise linker error spans (Plan 191).

## Design

### Diagnostic Foundation: miette and the Error Type Hierarchy

The cornerstone of AutoLang's error reporting is the `miette` crate (v7.2 with "fancy" feature), chosen for its rich diagnostic output -- color-coded severity levels, inline source snippets, labeled spans, and structured help text. Built on top of miette, the error type hierarchy in `crates/auto-lang/src/error.rs` defines four primary error categories plus a warning category, each with its own error-code range:

| Category | Codes | Example Variants |
|----------|-------|------------------|
| `SyntaxError` | E0001--E0007 | UnexpectedToken, UnterminatedString, InvalidExpression |
| `TypeError` | E0101--E0105 | TypeMismatch, InvalidOperation, NotCallable |
| `NameError` | E0201--E0204 | UndefinedVariable, DuplicateDefinition |
| `RuntimeError` | E0301--E0305 | DivisionByZero, IndexOutOfBounds, BreakOutsideLoop |
| `Warning` | W0001--W0005 | UnusedVariable, UnusedImport, DeadCode, ImplicitTypeConversion, DeprecatedFeature |

All variants are unified under the `AutoError` enum, which implements miette's `Diagnostic` trait with manual delegation so that each inner error's code, labels, help text, and severity are rendered correctly. The `SyntaxErrorWithSource` struct attaches raw source code to errors, enabling miette to render inline snippets like:

```
Error: auto_syntax_E0007

  x syntax error
   +--[test_error.at:1:3]
 1 | 1 +
   .   +
   .   +-- Expected term, got Newline, pos: 2:0:1, next: <nl>
```

A total of 47 `error_pos!` macro calls in the parser were replaced with structured `SyntaxError` variants, and all parser functions were updated to return `AutoResult<T>`.

### Multi-Error Recovery and Display

Plan 008 implemented parser-level error recovery so that compilation does not stop at the first syntax error. The `Parser` struct carries an `errors: Vec<AutoError>` accumulator and an `error_limit: usize` field (default: 20). When `parse_stmt()` fails, the parser calls `synchronize()`, which skips tokens forward to the next statement boundary (keywords like `fn`, `let`, `var`, `for`, `if`, `return`, or a semicolon). After parsing completes, if multiple errors were collected, the parser returns an `AutoError::MultipleErrors` variant whose `Diagnostic::related()` implementation renders each error as a separate diagnostic block, prefixed by a summary line: "aborting due to N previous errors."

The error limit is configurable via a global `--error-limit N` / `-e N` CLI flag, backed by an atomic global variable so that the parser reads it on initialization without needing to thread the value through every call.

### "Did You Mean?" Suggestions

To help users fix typos quickly, Plan 008 added a Levenshtein-distance-based suggestion system. The `find_best_match()` function computes edit distance against all names in scope (obtained via `Universe::get_defined_names()`, which collects variables, functions, types, and builtins from the current scope and its parents). If a candidate is within the threshold (at most 3 edits or 30% of the target string length), it appears as a `suggested: Option<String>` field on `NameError::UndefinedVariable` and `NameError::UndefinedFunction`. The manual `Diagnostic` implementation for `NameError` renders the suggestion as a note: `"Did you mean 'username'?"`

### Warning System

Five warning variants (W0001--W0005) were added to the `AutoError` enum, each using miette's `severity(warning)` attribute so they render in yellow rather than red. The variants cover unused variables, unused imports, dead code after return/break, implicit type conversions, and deprecated features. The infrastructure is complete and integrated into the `AutoError` dispatcher; however, the parser and evaluator have not yet been wired to emit these warnings during compilation or evaluation.

### JSON Output for IDE Integration

Plan 008 also delivered a `--format json` CLI flag that switches error output from human-readable text to machine-readable JSON. The `format_error_json()` function serializes each `AutoError` into a structured object containing `message`, `code`, `severity`, `spans` (with offset, length, and label), `help`, and `related` errors. This is designed as a foundation for Language Server Protocol integration (Phase 4.3, not yet started). Example output:

```json
{
  "code": "auto_syntax_E0099",
  "severity": "error",
  "message": "aborting due to 1 previous error",
  "related": [
    { "code": "auto_syntax_E0007", "message": "Expected term, got EOF" }
  ]
}
```

### Type Inference Engine

Plan 010 built a complete type-inference subsystem for AutoLang, spanning approximately 2,770 lines of code across seven files under `crates/auto-lang/src/infer/`. The design uses a *hybrid inference strategy*: local expressions are inferred bottom-up with simple type propagation, while functions use a simplified Hindley-Milner approach with constraint generation and Robinson unification. This trade-off keeps the implementation manageable (~1,500 LOC for inference proper vs. ~5,000+ for full HM) while still catching the vast majority of type errors at compile time.

#### Core Architecture

The central data structure is `InferenceContext`, which maintains:

- `type_env: HashMap<Name, Type>` -- the global type environment mapping variable names to their inferred types.
- `scopes: Vec<HashMap<Name, Type>>` -- a scope stack supporting nested blocks and variable shadowing.
- `constraints: Vec<TypeConstraint>` -- accumulated type constraints (Equal, Callable, Indexable, Subtype), each annotated with a `SourceSpan` for error reporting.
- `current_ret: Option<Type>` -- the expected return type of the enclosing function, used to check return statements.
- `errors: Vec<TypeError>` and `warnings: Vec<Warning>` -- accumulators so that inference never aborts on the first error.

The `TypeConstraint` enum supports four constraint kinds. `Equal(t1, t2, span)` asserts two types are identical. `Callable(t, span)` asserts a type can be invoked. `Indexable(t, span)` asserts a type supports indexing. `Subtype(t1, t2, span)` asserts a subtype relationship. Constraints are generated during expression inference and resolved by the unification engine.

#### Robinson Unification

The heart of the type system is the `unify()` function in `infer/unification.rs` (465 lines), implementing the Robinson unification algorithm with an occurs check to prevent infinite types (e.g., `a = List<a>`). Key unification rules:

- `Type::Unknown` unifies with anything (it acts as a type variable / wildcard).
- Identical primitive types unify trivially (`Int` with `Int`).
- Array types unify element types and check matching lengths.
- Coercion pairs (`Int`/`Uint`, `Float`/`Double`) unify successfully but emit an `ImplicitTypeConversion` warning.
- Mismatched types produce a `TypeError::Mismatch` with the expected and found type names.

#### Expression Inference

The `infer/expr.rs` module (552 lines) handles inference for 20+ expression types: integer, float, boolean, and string literals; identifier lookups through the type environment and scope chain; unary operations (Not, Neg); binary arithmetic and comparison operations (where the result type is determined by the operand types); array expressions (element types are unified across all elements); function calls; array indexing; if-else expressions (branch types are unified); and block expressions (the type of the last statement). Each binary operation generates an `Equal` constraint between its operand types, ensuring type consistency.

#### Statement and Function Checking

Statement-level type checking lives in `infer/stmt.rs` (822 lines). It provides `check_store()` for variable declarations (validating that the initializer's type matches the declared type), `check_if()` and `check_for()` for control flow, and `check_return()` for return-statement consistency against the current function's return type.

Function signature inference in `infer/functions.rs` (662 lines) creates a new scope, infers parameter types (from explicit annotations or default values), adds parameters to the type environment, infers the body type, and then either checks it against an explicit return type annotation or uses the inferred body type as the return type. The result is a `Type::Fn(params, ret)` representing the complete function signature.

#### Error Recovery in the Inference Engine

The inference engine follows a "fail-open" strategy: when type inference fails for an expression, the result degrades to `Type::Unknown` rather than halting compilation. All type errors are collected into `ctx.errors` and reported at the end. This is complemented by a suggestion system in `infer/errors.rs` (475 lines) that uses the same Levenshtein-distance algorithm from Plan 008 to offer "did you mean?" hints for misspelled type names, variable names, and primitive types. The module also provides helpers for constructing coercion warnings and dead-code warnings.

#### Parser Integration (Hybrid Strategy)

A key design decision in Plan 010 was to use a *hybrid integration strategy* rather than replacing the old inference system outright. The old `infer_type_expr()` function in the parser relies on `Universe`-based symbol lookups that have access to runtime type information (such as pre-computed function return types stored in `call.ret`). The new system, operating purely on the AST, lacks this runtime context. The hybrid approach tries the old system first and falls back to the new inference engine:

```rust
fn infer_type_expr(&mut self, expr: &Expr) -> Type {
    // 1. Old Universe-based lookup (has runtime type info)
    if let Expr::Ident(name) = expr {
        if let Some(sym) = self.scope.borrow().lookup(name) {
            if let Some(ty) = &sym.ty {
                if !matches!(ty, Type::Unknown) {
                    return ty.clone();
                }
            }
        }
    }
    // 2. New inference engine (full derivation capability)
    self.infer_ctx.infer_expr(expr)
}
```

Scope synchronization was added in Phase 5B: every `enter_scope()` / `exit_scope()` call in the parser is mirrored in the inference context, and every `define()` / `define_rc()` call binds the variable's type into the inference environment. Function scopes and import scopes are similarly synchronized. This integration yielded measurable improvements: five existing tests showed better type inference -- for example, `unknown y = x` became `int y = x` in the C transpiler output, and incorrect `printf("%d\n", v1)` format strings were corrected to `printf("%s\n", v1)` after the inference engine correctly identified string types.

The integration passed 1,048 of 1,064 project tests (98.5%), with remaining failures attributable to pre-existing issues unrelated to type inference. The 98 infer-module tests complete in under 10 milliseconds.

### Runtime Error Integration (Plan 009 -- Not Yet Started)

Plan 009 outlines the work needed to bring the `RuntimeError` enum (E0301--E0305) into the evaluator. Currently, `eval.rs` uses `panic!` calls for errors like division by zero and invalid assignment, and returns raw `Value` instead of `AutoResult<Value>`. The plan involves five steps:

1. **Add span tracking to AST nodes** -- an optional `span: Option<SourceSpan>` field on `Expr` and `Stmt`, populated by the parser.
2. **Change evaluator signatures** -- every `eval_expr` and `eval_stmt` function returns `AutoResult<Value>`, enabling error propagation with the `?` operator.
3. **Replace `panic!` calls** -- each panic site is mapped to a specific `RuntimeError` variant with the appropriate span.
4. **Implement stack-trace capture** -- a `StackFrame` struct tracks function name and source location; the interpreter pushes/pops frames during function calls.
5. **Enhanced error display** -- miette's `related()` notes render the call chain as "Error occurred in function X / Called from 'main' at file.at:line:col."

The estimated effort is 12--18 hours. This is the primary remaining gap in the error-handling story: once complete, every error path through the compiler and runtime will produce structured, source-annotated diagnostics.

## Open Questions

- **Error recovery priority**: Should parser-level error recovery be expanded before completing runtime error integration in the evaluator, or should the evaluator work come first?
- **Warning aggressiveness**: Should warnings like unused variables be opt-in or opt-out by default? No CLI flags for warning control have been implemented yet.
- **Error documentation**: Should the error-code reference in `docs/errors.md` be auto-generated from doc comments or maintained manually? The `--explain E0001` flag is planned but not started.
- **C-implementation port timing**: The error system is Rust-only. Porting to the C implementation (`autoc/`) is deferred until the Rust version is complete, but no firm milestone has been set.
- **Generic type inference**: The current inference engine is monomorphic. Adding generics (Plan 010, Phase 8) and traits/interfaces (Phase 9) would require significant extensions to the constraint system and unification algorithm.

## Source Plans

- docs/plans/008-error-message-system.md
- docs/plans/009-runtime-error-integration.md
- docs/plans/010-type-inference-subsystem.md
- [191-assert-and-precise-linker-errors.md](../plans/191-assert-and-precise-linker-errors.md)
