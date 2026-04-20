# 03 - Error Handling

## Status

**Implemented**: `Option(T)` (`?T`) and `Result(T)` (`!T`) as distinct AST types, three-tier error classification (Syntax/Type/Name/Runtime) with miette diagnostic display, error codes (E0001-E0305), source code snippets with labeled spans, `AutoResult<T>` return type across the compiler.

**Partial**: The `May<T>` three-state type is designed but `May` was removed from the AST in favor of separate `Option` and `Result` types. The `!T` unified error handling syntax is designed but the full error propagation operator (`.!`) is not yet in the parser. The `#[nopanic]` annotation is not implemented.

**Planned**: `May<T>` or equivalent three-state unification, `.?` / `.!` / `.!!` postfix propagation operators, `#[nopanic]` static analysis, fallible main (`fn main() !`), Auto Mode (`auto fn`) for implicit error propagation.

## Design

### Three-Tier System: Option, Result, Panic

AutoLang classifies non-normal program states into three strict tiers:

| Tier | Type Syntax | Name | Mental Model | Handling |
|------|-------------|------|--------------|----------|
| L1 | `?T` | Option | Empty box (data absent) | Gentle: expected absence (e.g., key not in map) |
| L2 | `!T` | Result | Broken box (operation failed) | Strict: must be handled (e.g., IO error) |
| L3 | `T` (unwrapped) | Panic | Bomb (unrecoverable) | Emergency: logic errors (e.g., divide by zero) |

This replaces the original `May<T>` three-state design. In the current implementation, `Option(T)` and `Result(T)` are separate AST types (`ast/types.rs`), each wrapping an inner type. The `?T` syntax maps to `Option`, and `!T` maps to `Result`.

### May<T>: The Three-State Vision (Design Only)

The original design proposed `May<T>` (syntax `?T`) as a three-state type merging Option and Result:

| State | Tag | Meaning | C Translation |
|-------|-----|---------|--------------|
| Value | `0x01` | Success with data `T` | `struct.data.value` |
| Empty | `0x00` | Success, no data (nil) | no payload |
| Error | `0x02` | Failure with error info | `struct.data.err` |

This was defined as a tagged union in the raw design:

```auto
tag May<T> {
    nil Nil
    err Err
    val T
}
```

The current implementation chose separate `Option` and `Result` types instead. `May<T>` was removed from `ast/types.rs` with the comment: "use generic tag May<T> from stdlib instead." This leaves open the possibility of a stdlib-based `May` type in the future.

### Postfix Propagation Operators (Designed, Not Implemented)

The designed operator system uses dot-postfix syntax for chained error handling:

**Mnemonic**: `?` for data, `!` for errors, `!!` for panics. No parameter = propagate; with parameter = recover.

| Object | Type | Propagate (no arg) | Recover (with arg) |
|--------|------|--------------------|--------------------|
| Data | `?T` | `val.?` (return None) | `val.?(default)` |
| Error | `!T` | `val.!` (return Err) | `val.!(default)` |
| Panic | `T` | `expr.!!` (panic now) | `expr.!!(default)` |

Usage example:

```auto
let val = list[i].?(0)       // Option: use 0 if absent
let temp = read_sensor(1).!  // Result: propagate error
let speed = (100 / x).!!(0)  // Panic: rescue with 0 if divide-by-zero
```

### `#[nopanic]` Static Safety (Designed, Not Implemented)

For hard real-time and ISR contexts, `#[nopanic]` enforces compile-time safety:

```auto
#[nopanic]
fn interrupt_handler() {
    // Compiler rejects: division, assert, bare .!!, calls to non-nopanic functions
    let speed = (100 / sensor_val).!!(0)  // OK: panic is caught with default
}
```

Rules:
1. **Contagion**: `#[nopanic]` functions can only call other `#[nopanic]` functions (unless panics are locally rescued).
2. **Prohibited**: Division (unless divisor provably non-zero), `assert()`, `panic()`, bare `.!!`.
3. **Rescue**: `.!!(default)` converts a potential panic into a safe fallback, satisfying the `#[nopanic]` contract.

The runtime behavior of `.!!(default)` varies by build profile:
- **Debug**: Ignores the default, panics immediately (fail fast for debugging).
- **Release**: Catches the panic, logs a FATAL, returns the default (fail safe for production).

### Error Message System

The compiler's error reporting is implemented using `miette` and `thiserror`, providing IDE-grade diagnostics.

**Error classification** (in `error.rs`):

| Category | Code Range | Examples |
|----------|-----------|----------|
| SyntaxError | E0001-E0007 | UnexpectedToken, InvalidExpression, UnterminatedString |
| TypeError | E0101-E0105 | TypeMismatch, InvalidOperation, NotCallable |
| NameError | E0201-E0204 | UndefinedVariable, DuplicateDefinition, ImmutableAssignment |
| RuntimeError | E0301-E0305 | DivisionByZero, ModuloByZero, IndexOutOfBounds |

Each error includes: error code, file location (line:column), source code snippet with labeled span, and help text. The `AutoError` enum wraps all error types and manually implements the `Diagnostic` trait to properly delegate `source_code()` and `labels()` to inner errors.

**Example output**:
```
Error: auto_syntax_E0007
  x syntax error
   |-[test_error.at:1:3]
 1 | let x = 1; x = 2
   |          -
   |          -- Syntax error: Assignment not allowed for let store: x
```

### Fallible Main (Designed, Not Implemented)

`fn main() !` would allow the entry point to use error propagation operators. The compiler would perform "entry-point hijacking" -- renaming the user's `main` to `__auto_user_main` and synthesizing a platform-appropriate wrapper:

- **OS target (a2rs)**: Wrapper catches errors and calls `std::process::exit(1)`.
- **Embedded target (a2c)**: Wrapper catches errors and enters an infinite loop (waiting for watchdog reset) rather than returning from `main` on bare metal.

### Auto Mode (Designed, Not Implemented)

Auto Mode is a productivity layer that eliminates error-handling boilerplate through compiler-generated code:

- **`auto fn`**: Function return type implicitly wrapped in `!T`. Error-returning function calls automatically propagate on failure.
- **`auto { ... }`**: Block-level Auto Mode inside system functions.
- **`#auto`**: File-level directive making all functions in the file use Auto Mode.

The 3A Protocol (Automatic Error Propagation, Automatic Dereference, Automatic Type Inference) lowers to standard system-mode code via AST rewriting, requiring no new runtime support.

## Open Questions

- Whether `May<T>` should be revived as a stdlib type or the separate `Option`/`Result` design is final.
- How the `.!` operator interacts with the existing `ParamMode` system in function calls.
- Whether `#[nopanic]` should be part of the initial type checker or a separate analysis pass.
- The interaction between Auto Mode and the ownership system -- automatic dereferencing needs to respect borrow rules.

## Source Documents

- [raw/may-type.md](raw/may-type.md)
- [raw/result-type.md](raw/result-type.md)
- [raw/exceptionals.md](raw/exceptionals.md)
- [raw/error-system.md](raw/error-system.md)
- [raw/enhanced-main.md](raw/enhanced-main.md)
- [raw/auto-mode.md](raw/auto-mode.md)
