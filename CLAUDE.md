# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AutoLang is a multi-purpose programming language designed for automation with one canonical implementation:
- **Rust implementation** (`crates/`): Primary compiler with full features including transpilation to C and Rust

The AutoLang compiler supports multiple execution modes:
- **Evaluator**: Interprets AutoLang code directly (REPL, script execution)
- **C Transpiler** (a2c): Transpiles AutoLang to C for embedded systems
- **Rust Transpiler** (a2r): Transpiles AutoLang to Rust for native applications

### Unified CLI (auto)

```bash
auto hello.at                # Run an Auto script directly
auto new myapp               # Create a new project
auto build                    # Build current project
auto run                      # Run current project
auto fetch                    # Download dependencies
auto                         # Enter REPL
```

### Build & Test

```bash
cargo build                  # Build all crates (debug mode)
cargo test                   # Run all tests
cargo test -p auto-lang      # Run auto-lang crate tests
cargo test -p auto-lang -- trans  # Run transpiler (a2c/a2r) tests
```

- Always run `cargo test` after modifying VM-related code
- Always run `cargo build` after modifying codegen or parser code

## Auto Code Generation Rules

**CRITICAL: When generating AutoLang (.at) code, ALWAYS invoke the `/auto-lang-creator` skill first.** This skill contains the authoritative syntax reference and gotcha checklist verified against a2r test cases. Common mistakes that the skill prevents:

| Wrong (AI tendency) | Correct (AutoLang) | Rule |
|---|---|---|
| `fn foo() -> int` | `fn foo() int` | No `->` for return type |
| `type T { x: int }` | `type T { x int }` | Type fields use space, not colon |
| `enum E { V(field: int) }` | `enum E { V(field int) }` | Enum variant fields use space |
| `let mut x = 5` | `var x = 5` | `var` for mutable, not `let mut` |
| `fn foo(mut self, ...)` | `mut fn foo(...)` | `mut fn` prefix, no self parameter |
| `Option::Some(x)` | `Some(x)` | No module prefix on constructors |
| `Result::Ok(x)` | `Ok(x)` | No module prefix on constructors |
| `is x { 0 => ... }` | `is x { 0 -> ... }` | `->` arrows in pattern matching |
| `while cond { ... }` | `for cond { ... }` | No `while` keyword |
| `async fn foo()` | `fn foo() ~RetType` | `~T` return type = async |
| `say("hello")` | `print("hello")` | Use `print()` |
| `println!("hello")` | `print("hello")` | No macros |

## Language Features

### Storage Types
- `let` - Immutable binding
- `var` - Mutable binding (NOT `let mut`)
- `const` - Global constant

### Control Flow
- `if/else if/else` - Conditional branching
- `for x in start..end` - Range loops
- `loop` - Infinite loops with `break`
- `is` - Pattern matching (uses `->` arrows)

### Key Syntax
- **F-strings**: `f"hello $name"` or `f"result: ${1 + 2}"`
- **Ranges**: `0..10` (exclusive) or `0..=10` (inclusive)
- **Arrays**: `[1, 2, 3]` with indexing `arr[0]`
  - Static arrays: `[N]T` where N is compile-time size (e.g., `[10]int`)
  - Slices: `[]T` for borrowed slice of array
  - **Dynamic lists**: `List` type — see [docs/design/07-data-structures.md](docs/design/07-data-structures.md)
- **Objects**: `{key: value, ...}` with field access `obj.key`
- **Functions**: `fn add(a int, b int) int { a + b }`
- **Imports**: `use math::add` or `use c <stdio.h>`

### AURA Styling Property Location Rule

In AURA widgets, styling properties (especially `class`) should be placed **after children** in the element body.

```auto
col {
    text (text: "Hello, World!") {
        class: "text-2xl font-bold"
    }
    class: "w-full h-full justify-center items-center bg-white"
}
```

### Module Imports

```auto
use db              // Same directory: ./db.at or ./db/mod.at
use super.db         // Parent directory: ../db.at
use pac.db           // Package root: search source dirs
use db: load, save   // Import specific symbols
```

- Error if both `name.at` and `name/mod.at` exist (ambiguous)
- `super` only works one level; for deeper navigation use `pac.`

### Function Annotations & Visibility

AutoLang uses Rust-style `#[...]` annotation syntax. Visibility uses `pub` keyword prefix.

```auto
#[vm]
fn my_function(x int) void;

#[c]
fn c_function(s str) int;

pub fn public_function() int;
```

**Annotation Rules**:
- All annotations MUST start with `#[]` (Rust-style)
- Old `[...]` syntax (without `#`) is **DEPRECATED**
- `pub` is a keyword prefix, NOT an annotation: `pub fn`, not `#[pub] fn`

**In Type Definitions**:
- `static fn` - Type-level methods (e.g., `MyType.new()`)
- Regular `fn` - Instance methods (e.g., `instance.method()`)
- `#[vm]` functions: Implemented in Rust via VM registry
- `#[c]` functions: Transpiled to C

## Detailed Design Documents

Organized as a design reference book (originals preserved in `docs/design/raw/`):
- [docs/design/01-architecture.md](docs/design/01-architecture.md) — Compiler pipeline, core components, AIE
- [docs/design/02-type-system.md](docs/design/02-type-system.md) — Types, inference, generics, enums, unions
- [docs/design/03-error-handling.md](docs/design/03-error-handling.md) — Option/Result/Panic, error messages
- [docs/design/04-memory-ownership.md](docs/design/04-memory-ownership.md) — Lifetimes, ownership, param passing
- [docs/design/05-vm-runtime.md](docs/design/05-vm-runtime.md) — Bytecode ISA, AutoVM, BigVM, concurrency
- [docs/design/06-code-generation.md](docs/design/06-code-generation.md) — a2c, a2ark, a2jet, autogen, C interop
- [docs/design/07-data-structures.md](docs/design/07-data-structures.md) — Node, Atom format, serialization
- [docs/design/08-ui-systems.md](docs/design/08-ui-systems.md) — AURA, design tokens, frontend-backend
- [docs/design/09-compiler.md](docs/design/09-compiler.md) — Incremental compilation, caching, comptime
- [docs/design/10-language-syntax.md](docs/design/10-language-syntax.md) — Dot notation, functions, OOP, stdlib
- [docs/design/11-shell-tools.md](docs/design/11-shell-tools.md) — AutoShell, coreutils, SmartCmd

## Common Development Tasks

### ⚠️ CRITICAL: Test Expectation Rules

**IMPORTANT**: When fixing failing tests, you have two options:

1. **Fix the implementation** to match the expected output (PREFERRED)
2. **Ask for permission** before changing test expectations

**NEVER modify test expected output without explicit user permission.**

### Creating Plans for Complex Tasks

ALL plan files with sequential numbers MUST be created in `docs/plans/` folder with consecutive numbering (e.g., `006-my-plan.md`) and kebab-case naming.

### ⚠️ CRITICAL: Never Edit Generated C Files

**DO NOT manually edit `.c` or `.h` files in `stdlib/auto/`** — these are auto-generated by the C transpiler from `.at` source files. Edit the `.at` files instead.

### ⚠️ CRITICAL: Use `#[rust_fn]` Macro for Stdlib FFI

When implementing VM FFI functions in `crates/auto-lang/src/vm/ffi/stdlib.rs`, use the `#[rust_fn]` macro instead of manual shims whenever possible. Only use manual shims for variadic args or when direct stack/task access is needed.

### Commit Message Guidelines

Keep commit messages concise and focused. Focus on what changed and why, not implementation details.

### Working with Temporary Test Files

Always place temporary test files in the `tmp/` directory (in `.gitignore`). Never create test files in the project root.

### Adding a2c (Auto-to-C) Test Cases

```bash
# 1. Create test directory
mkdir crates/auto-lang/test/a2c/106_my_test

# 2. Create input file and edit
# 3. Run test (creates .wrong.c/.wrong.h)
cargo test -p auto-lang test_106_my_test

# 4. Review and rename to .expected.*
# 5. Add test function in trans/c.rs
```

Test naming: `000-099_*` = core features, `100-199_*` = stdlib. Run all: `cargo test -p auto-lang -- trans`

### Adding a2ark (Auto-to-ArkTS) Test Cases

1. Create directory `XXX_widget_name/` in `crates/auto-lang/test/a2ark/`
2. Add `input.at` with widget test
3. Run test, review `.wrong.ets`, rename to `.expected.ets`
4. Add test function in `generator.rs`

## File Structure Conventions

- `.at` extension - AutoLang source files
- `crates/auto-lang/` - Main compiler implementation (Rust)
- `crates/auto-val/` - Value system and data structures (Rust)
- `crates/auto-lang/src/trans/` - Transpilers (c.rs for C, rust.rs for Rust)
- `crates/auto-lang/test/a2c/` - Auto-to-C transpiler tests
- `crates/auto-lang/test/a2r/` - Auto-to-Rust transpiler tests
- `auto/` - Self-hosted compiler source files (.at files)
- `stdlib/auto/` - Standard library AutoLang code
- `docs/` - Documentation and resources
- `docs/plans/` - Implementation plans
- `docs/design/` - Detailed design documents

## Known Issues and Limitations

1. **For loop variable access** - Accessing loop variable inside loop body may return garbage data
2. **String literal parsing** - Some string edge cases show garbage characters
3. **Unary operations** - Operator representation may be incorrect
4. **If expressions** - Currently parsed as statements, not expressions
5. **F-string prefix** - `f"` is tokenized as `<ident:f>` followed by f-string tokens (not yet unified)
