# Plan 173: r2a — Rust to AutoLang Transpiler

## Objective

Implement a reverse transpiler (r2a) that converts Rust source code into AutoLang (.at) code. This is a general-purpose Rust code import tool, not limited to a2r-generated code.

## Current State

- a2r (Auto-to-Rust) transpiler is mature with 124 test cases across 17 groups
- a2r test cases provide excellent round-trip validation material
- The transpiler module (`crates/auto-lang/src/trans/`) already has a2c and a2r; r2a will join as a sibling
- **Phase 1 merged** (2025-04-15): 41 tests — core fn, let/var/const, if/for/while, basic types, print, arithmetic, struct/enum
- **Phase 2 merged** (2025-04-16): 57 tests total — impl/trait/spec/ext, &self/&mut self, union, raw pointer, dyn Trait, method calls
- **Phase 3 merged** (2026-04-16): 91 tests total — generics (struct/enum/fn/type alias), trait bounds, enum discriminants, Option/Result round-trips
- **Phase 4 merged** (2026-04-16): 116 tests total — async/comment degradation, derive/serde attrs, modules, Box/Arc/Rc, type casts, collections
- Core architecture: direct syn→.at text conversion (no intermediate AutoLang AST)

## Design Decisions

1. **Rust Parser**: `syn` crate v2 (NOT syn 1.x — API differs significantly)
2. **Output**: Direct syn→.at text conversion via `R2aTrans` struct with `String` output (no intermediate AutoLang AST)
3. **Location**: `crates/auto-lang/src/trans/r2a.rs` alongside a2r and a2c
4. **Rust Edition**: 2021 first, 2024 later
5. **Strategy**: Best-effort — features AutoLang can't express get comment markers, not errors
6. **No `quote` dependency**: Token streams handled via `.to_string()` on syn types, no need for `proc_macro2` direct access

## Architecture

```
Rust Source (.rs)
    ↓
syn::parse_file() → syn::File (full Rust AST)
    ↓
R2aTrans (tree-walking converter)
    ↓
String (direct .at text output)
```

Key implementation detail: syn's `Display` impls on AST types produce Rust debug format, not useful for us. We walk the syn AST and build `.at` text directly.

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
| `Arc<T>` / `Rc<T>` | `T` | Ownership implicit |
| `HashMap<K,V>` | `Map<K,V>` | |
| `&T` | `T.view` | Reference → .view |
| `&mut T` | `T` | Mutable ref dropped |
| `dyn Trait` | `/* dyn Trait */` | Comment degradation |
| `impl Trait` | `/* impl Trait */` | Comment degradation |

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

### Struct / Enum / Union

```rust
// Rust
struct Point { x: i32, y: i32 }
enum Color { Red, Green, Blue }
union Value { int_val: i32, float_val: f32 }

// AutoLang
type Point { x int, y int }
enum Color { Red, Green, Blue }
union Value { int_val int, float_val f32 }
```

### Trait / Impl / Self

```rust
// Rust
trait Flyer { fn fly(&self); }
impl Flyer for Bird { fn fly(&self) { ... } }
impl Bird {
    fn new(name: &str) -> Bird { ... }
    fn speak(&self) { ... }
    fn evolve(&mut self) { ... }
}

// AutoLang
spec Flyer { fn fly() }
ext Bird for Flyer { fn fly() { ... } }
ext Bird {
    static fn new(name cstr) Bird { ... }
    fn speak() { ... }
    mut fn evolve() { ... }
}
```

- `trait` → `spec`, `&self` param dropped
- `impl Trait for Type` → `ext Type for Trait`
- `impl Type` → `ext Type`
- No self parameter → `static fn`
- `&self` → plain `fn` (instance method)
- `&mut self` → `mut fn` (mutating method)

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
| `x.to_string()` | `x.to_str()` |
| `x.clone()` | `x` (identity) |
| `x.into()` | `x` (identity) |

### Macro Conversions

```rust
println!("hello")                    → print("hello")
println!("x = {}", x)                → print(f"x = $x")
println!("{} + {}", a, b)            → print(f"$a + $b")
println!("{}", expr)                  → print(expr)  // simple case
format!("hello {}", name)            → f"hello $name"
vec![1, 2, 3]                       → [1, 2, 3]
```

Note: syn 2 parses macros in statement position as `Stmt::Macro`, not `Stmt::Expr(Expr::Macro)`. Both paths are handled.

### Ownership Simplifications

```rust
String::from("x")                    → "x"
Box::new(val)                        → val
Arc::new(val)                        → val
Rc::new(val)                         → val
x.clone()                            → x
x.into()                             → x
```

Constructor detection checks full path (e.g. `String::from`, `Box::new`), not just the last segment.

## Features That Cannot Be Expressed (Degradation)

| Rust Feature | Handling |
|--------------|----------|
| Procedural macros `#[derive(...)]` | Comment: `// #[derive] skipped` |
| Attribute/function macros | Preserved in comments |
| `unsafe` blocks | `/* unsupported expr */` for block contents |
| Lifetime annotations `'a` | Dropped |
| `const fn` | Downgraded to `fn` |
| Closures `\|x\| x + 1` | Comment marker (Phase 4) |
| async/await | Comment marker until Phase 4 |
| `dyn Trait` types | `/* dyn TraitName */` comment |
| `impl Trait` types | `/* impl TraitName */` comment |

## Known Limitations

1. **Type prefix lost on static calls**: `Counter::new()` → `new()` (type prefix dropped)
2. **Token stream spacing**: Method calls inside macros may have extra spaces (e.g. `c . get_count ()`) due to syn token stream representation
3. **Struct literal format**: `Point { x, y }` → `Point(x: x, y: y)` (constructor syntax instead of struct literal)

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

### File-Based Tests

- Directory: `crates/auto-lang/test/r2a/`
- Each case: `input.rs` (Rust) + `input.expected.at` (AutoLang)
- Runner: `test_r2a_file("group/case_name")` — reads .rs, transpiles, compares with .expected.at, writes .wrong.at on mismatch

### Round-Trip Validation

Leverage existing a2r test cases:
1. Take `.expected.rs` as r2a input
2. r2a converts to `.at`
3. Verify output is non-empty and contains no `/* unknown */` markers
4. Does NOT compare with original (structural correctness only)

### Test Counts

| Category | Phase 1 | Phase 2 | Phase 3 | Phase 4 | Total |
|----------|---------|---------|---------|---------|-------|
| Unit tests | 22 | 3 | 5 | 5 | 35 |
| File-based tests | 7 | 6 | 11 | 11 | 35 |
| Round-trip tests | 12 | 7 | 18 (+2 ignored) | 10 (+1 ignored) | 47 |
| **Total** | **41** | **16** | **34** | **25** | **116** |

## Public API

```rust
/// Main entry: Rust source → AutoLang source
pub fn transpile_r2a(name: &str, rust_code: &str) -> AutoResult<String>
```

Consistent with a2r API style. Project-level API deferred to Phase 4.

## Implementation Phases

| Phase | Content | Status | Tests |
|-------|---------|--------|-------|
| Phase 1 | Core: fn, let/var/const, if/for/while, basic types, print, arithmetic, struct/enum | ✅ Merged | 41 |
| Phase 2 | impl/trait/spec/ext, &self/&mut self, union, raw pointer, dyn Trait, method calls | ✅ Merged | +16 = 57 |
| Phase 3 | Generics (struct/enum/fn/type alias), trait bounds, enum discriminants, Option/Result round-trips | ✅ Merged | +34 = 91 |
| Phase 4 | async/comment degradation, derive/serde attrs, modules, Box/Arc/Rc, type casts, collections | ✅ Merged | +25 = 116 |

## Key Files

- `crates/auto-lang/src/trans/r2a.rs` — Main transpiler (~2500 lines)
- `crates/auto-lang/src/trans.rs` — Module registration (`pub mod r2a`)
- `crates/auto-lang/test/r2a/` — Test cases directory
  - `01_basics/` — Hello, func
  - `02_types/` — Struct, enum
  - `03_control/` — If, for, match, while, loop
  - `04_methods/` — mut self methods
  - `05_traits/` — impl for, struct methods, dyn trait, union, raw pointer
  - `06_pattern_matching/` — Hetero enum, struct destructure, discriminant, generic enum
  - `08_generics/` — Type alias, generic struct, generic fn, map type
  - `09_option_result/` — Option construct, try operator, unwrap_or
  - `10_collections/` — Array, method chain
  - `14_modules/` — Use, pub visibility, const decl, derive attr
  - `15_type_conversion/` — Type cast, Box/Arc/Rc, string methods
  - `16_interop/` — Async fn, field attrs (serde)
- `crates/auto-lang/Cargo.toml` — `syn` dependency

## Dependencies

```toml
[dependencies]
syn = { version = "2", features = ["full", "parsing", "extra-traits"] }
```

No `quote` or `proc_macro2` dependency needed — token streams handled via `.to_string()`.
