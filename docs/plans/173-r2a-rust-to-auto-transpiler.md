# Plan 173: r2a — Rust to AutoLang Transpiler

## Objective

Implement a reverse transpiler (r2a) that converts Rust source code into AutoLang (.at) code. This is a general-purpose Rust code import tool, not limited to a2r-generated code.

## Current State

- a2r (Auto-to-Rust) transpiler is mature with 124 test cases across 17 groups
- a2r test cases provide excellent round-trip validation material
- The transpiler module (`crates/auto-lang/src/trans/`) already has a2c and a2r; r2a will join as a sibling

## Design Decisions

1. **Rust Parser**: Use `syn` crate — the de facto standard Rust AST library in the ecosystem
2. **Output**: Reuse existing `Code`/`Stmt`/`Expr` AST from `ast.rs` (same as a2r input)
3. **Location**: `crates/auto-lang/src/trans/r2a.rs` alongside a2r and a2c
4. **Rust Edition**: 2021 first, 2024 later
5. **Strategy**: Best-effort — features AutoLang can't express get comment markers, not errors

## Architecture

```
Rust Source (.rs)
    ↓
syn::parse_file() → syn::File (full Rust AST)
    ↓
R2aTrans (tree-walking converter)
    ↓
Code (AutoLang AST, reuses ast.rs)
    ↓
Serialize → .at text
```

## Type Mapping (Rust → AutoLang)

| Rust | AutoLang | Notes |
|------|----------|-------|
| `i8` | `i8` | |
| `i32` | `int` | Default integer |
| `i64` | `i64` | |
| `u8` | `byte` | |
| `u32` | `uint` | Default unsigned |
| `u64` | `u64` | |
| `f32` | `f32` | |
| `f64` | `float` | Default float |
| `bool` | `bool` | |
| `char` | `char` | |
| `String` | `str` | |
| `&str` | `cstr` | |
| `()` | `void` | |
| `Vec<T>` | `List<T>` | |
| `Option<T>` | `may T` | |
| `Result<T, E>` | `result T, E` | |
| `[T; N]` | `[N]T` | Fixed array |
| `&[T]` | `[]T` | Slice |
| `*mut T` / `*const T` | `*T` | Raw pointer |
| `Box<T>` | `T` | Ownership implicit |
| `HashMap<K,V>` | `Map<K,V>` | |
| `&T` / `&mut T` | `T` | Value semantics default |

Unknown generic types (e.g. `MyType<T>`) are preserved as-is.

## Syntax Mapping

### Functions

```rust
// Rust
fn add(a: i32, b: i32) -> i32 { a + b }

// AutoLang
fn add(a int, b int) int { a + b }
```

- Params: `name: Type` → `name Type` (space-separated)
- Return: `-> Type` → `Type` (no arrow)

### Variables

```rust
// Rust
let x: i32 = 42;
let mut count = 0;
const MAX: i32 = 100;

// AutoLang
let x int = 42
var count = 0
const MAX int = 100
```

- `let mut` → `var`
- `let` (immutable) → `let`
- `const` → `const`

### Control Flow

```rust
// Rust                  → AutoLang
if x > 0 { ... }        → if x > 0 { ... }
else if x < 0 { ... }   → else if x < 0 { ... }
else { ... }             → else { ... }
for i in 0..10 { ... }  → for i in 0..10 { ... }
while cond { ... }       → for cond { ... }
loop { ... }             → for ever { ... }
match x { ... }          → is x { ... }
```

### Struct / Enum

```rust
// Rust
struct Point { x: i32, y: i32 }
enum Color { Red, Green, Blue }

// AutoLang
type Point { x int, y int }
enum Color { Red, Green, Blue }
```

### Trait / Impl

```rust
// Rust
trait Flyer { fn fly(&self); }
impl Flyer for Bird { fn fly(&self) { ... } }
impl Bird { fn new(name: &str) -> Bird { ... } }

// AutoLang
spec Flyer { fn fly() }
ext Bird for Flyer { fn fly() { ... } }
ext Bird { fn new(name cstr) Bird { ... } }
```

- `trait` → `spec`, `&self` param dropped
- `impl Trait for Type` → `ext Type for Trait`
- `impl Type` → `ext Type`
- `static` methods → `static fn`

## Expression Mapping

### Operators

| Rust | AutoLang |
|------|----------|
| `&&` | `and` |
| `\|\|` | `or` |
| `!` | `not` |
| `??` | `??` |
| `?` | `.?` |
| `*ptr` | `ptr.*` |
| `ptr as *mut _` | `ptr.@` |
| `&x` | `x.view` |
| `&mut x` | `x.mut` |

### Method Call Translations

| Rust | AutoLang |
|------|----------|
| `s.to_lowercase()` | `s.to_lower()` |
| `s.len()` | `s.length()` |
| `s.push_str("x")` | `s.append("x")` |
| `v.push(x)` | `v.append(x)` |
| `v.pop()` | `v.pop()` |
| `v.is_empty()` | `v.is_empty()` |
| `s.contains("x")` | `s.has("x")` |
| `s.trim()` | `s.trim()` |
| `s.split(",")` | `s.split(",")` |
| `v.sort()` | `v.sort()` |

### Macro Conversions

```rust
println!("hello")                    → print("hello")
println!("x = {}", x)                → print(f"x = $x")
println!("{} + {}", a, b)            → print(f"$a + $b")
format!("hello {}", name)            → f"hello $name"
vec![1, 2, 3]                       → [1, 2, 3]
```

### Ownership Simplifications

```rust
String::from("x")                    → "x"
x.clone()                            → x
x.into()                             → x
Box::new(val)                        → val
Arc::new(val)                        → val
```

## Features That Cannot Be Expressed (Degradation)

| Rust Feature | Handling |
|--------------|----------|
| Procedural macros `#[derive(...)]` | Comment: `// #[derive] skipped` |
| Attribute/function macros | Preserved in comments |
| `unsafe` blocks | `unsafe` keyword dropped, contents converted |
| Lifetime annotations `'a` | Dropped |
| `const fn` | Downgraded to `fn` |
| Closures `\|x\| x + 1` | Comment marker (Phase 4) |
| async/await | Comment marker until Phase 4 |

## Module System

```rust
// Rust
mod utils;
pub mod network;
use std::collections::HashMap;
use crate::utils::helper;

// AutoLang
use utils
use network
use std.collections: HashMap
use utils: helper
```

- `mod X;` → `use X`
- `pub mod X;` → `use X`
- `crate::` prefix removed
- `super::` preserved

## Test Strategy

### Round-Trip Validation

Leverage existing a2r test cases:
1. Take `.expected.rs` as r2a input
2. r2a converts to `.at`
3. a2r converts back to Rust
4. Compare with original `.expected.rs`

### Independent r2a Test Suite

- Directory: `crates/auto-lang/test/r2a/`
- Each case: `input.rs` (Rust) + `input.expected.at` (AutoLang)
- Naming mirrors a2r: `01_basics/001_hello`, etc.

### Test Infrastructure

```rust
// crates/auto-lang/src/tests/r2a_tests.rs
fn test_r2a("group/case_name") {
    // Read input.rs → transpile_r2a() → compare with .expected.at
}
```

## Public API

```rust
/// Main entry: Rust source → AutoLang source
pub fn transpile_r2a(name: &str, rust_code: &str) -> AutoResult<String>

/// Project-level: Rust project dir → AutoLang project
pub fn transpile_r2a_project(entry_file: &str) -> AutoResult<MultiSink>
```

Consistent with a2r API style for easy integration.

## Implementation Phases

| Phase | Content | Test Coverage |
|-------|---------|---------------|
| Phase 1 | Core: fn, let/var/const, if/for/while, basic types, print, arithmetic | Groups 01-03 (~25 cases) |
| Phase 2 | struct/enum/union, impl/trait/spec/ext, method calls, self | Groups 02, 11-13 (~30 cases) |
| Phase 3 | Generics, Option/Result/may, pattern matching, String methods | Groups 06, 08-09 (~40 cases) |
| Phase 4 | async, modules, ownership annotations, unsafe, HashMap, lifetimes | Groups 07, 10, 14-16 (~30 cases) |

## Key Files

- `crates/auto-lang/src/trans/r2a.rs` — Main transpiler (~4000 lines target)
- `crates/auto-lang/src/trans/mod.rs` — Register r2a module
- `crates/auto-lang/src/tests/r2a_tests.rs` — Test runner
- `crates/auto-lang/test/r2a/` — Test cases directory
- `crates/auto-lang/Cargo.toml` — Add `syn` dependency

## Dependencies

```toml
[dependencies]
syn = { version = "2", features = ["full", "parsing", "extra-traits"] }
quote = "1"  # optional, for reconstructing Rust in comments
```
