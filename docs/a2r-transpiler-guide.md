# Auto-to-Rust (a2r) Transpiler Guide

## Overview

The Auto-to-Rust (a2r) transpiler converts AutoLang source code (.at files) into idiomatic Rust code, enabling native compilation with Rust's performance and safety guarantees.

## Quick Start

```bash
# Transpile a single file
auto.exe transpile rust input.at output.rs

# Run tests
cargo test -p auto-lang -- a2r
```

## Language Features

### 1. Functions

**AutoLang:**
```auto
fn add(a int, b int) int {
    return a + b
}
```

**Rust output:**
```rust
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}
```

### 2. Variable Declarations

| AutoLang | Rust |
|----------|------|
| `let x = 42` | `let x: i32 = 42;` |
| `var x = 42` | `let mut x: i32 = 42;` |
| `const x = 42` | `const x: i32 = 42;` |

### 3. Structs and Enums

**AutoLang:**
```auto
type Point {
    x int
    y int
}

tag Option {
    none Nil
    some T
}
```

**Rust output:**
```rust
struct Point {
    x: i32,
    y: i32,
}

enum Option {
    none,
    some,
}
```

### 4. Generics

**AutoLang:**
```auto
type Box<T> {
    value T
}

spec Storage<T> {
    fn data() *T
}

type Heap<T> as Storage<T> {
    ptr *T
    cap u32
}
```

**Rust output:**
```rust
struct Box<T> {
    value: T,
}

trait Storage<T> {
    fn data(&self) -> *mut T;
}

struct Heap<T> {
    ptr: *mut T,
    cap: u32,
}

impl<T> Storage<T> for Heap<T> {
    fn data(&self) -> *mut T {
        // implementation
    }
}
```

### 5. Closures

**AutoLang:**
```auto
fn main() {
    let add = (a int, b int) => a + b
    let result = add(5, 3)
}
```

**Rust output:**
```rust
fn main() {
    let add = |a: i32, b: i32| a + b;
    let result = add(5, 3);
}
```

### 6. Pattern Matching

**AutoLang:**
```auto
is x {
    0 => print("zero")
    1..10 => print("single digit")
    _ => print("other")
}
```

**Rust output:**
```rust
match x {
    0 => println!("zero"),
    1..=10 => println!("single digit"),
    _ => println!("other"),
}
```

## Type Mapping

| AutoLang Type | Rust Type |
|---------------|-----------|
| `int` | `i32` |
| `uint` | `u32` |
| `float` | `f32` |
| `double` | `f64` |
| `bool` | `bool` |
| `str` | `String` |
| `cstr` | `std::ffi::CStr` |
| `char` | `char` |
| `[N]T` | `[T; N]` |
| `[]T` | `[T]` |
| `*T` | `*mut T` |
| `void` | `()` |

## Standard Library Imports

**AutoLang:**
```auto
use auto.io: say
use auto.str: trim
use std::collections: HashMap
```

**Rust output:**
```rust
use crate::io::say;
use crate::str::trim;
use std::collections::HashMap;
```

## Memory Management

AutoLang's ownership system transpiles to Rust's borrow checker:

| AutoLang | Rust | Description |
|----------|------|-------------|
| `x.view` | `&x` | Immutable borrow |
| `x.mut` | `&mut x` | Mutable borrow |
| `x.take` | `x` | Move (default) |
| `x.*` | `*x` | Dereference |
| `x.@` | `x as *mut _` | Address-of |

## Platform-Specific Implementations

AutoLang supports platform-specific code with file extensions:

- `.at` - Interface/definition
- `.rs.at` - Rust-specific implementation
- `.c.at` - C-specific implementation
- `.vm.at` - AutoVM-specific implementation

Example:
```auto
// greeting.at
type Greeter {
    name str
}

ext Greeter {
    #[rs]
    fn say_hello() {
        // Rust implementation
    }

    #[c]
    fn say_hello_c() {
        // C implementation
    }
}
```

## Annotations

AutoLang uses Rust-style `#[...]` annotations:

- `#[rs]` - Rust-specific function
- `#[c]` - C-specific function
- `#[vm]` - VM-specific function
- `#[pub]` - Public visibility
- `#[rs, c]` - Available in both Rust and C

## Testing

Test files are organized as:
```
test/a2r/
├── 000_hello/
│   ├── hello.at           # Input
│   └── hello.expected.rs  # Expected output
├── 006_struct/
│   ├── struct.at
│   └── struct.expected.rs
└── ...
```

Run tests:
```bash
# All a2r tests
cargo test -p auto-lang -- a2r

# Specific test
cargo test -p auto-lang test_006_struct
```

## Implementation Status

- ✅ Phase 1: Parser support for `#[rs]` annotation
- ✅ Phase 2: `.rs.at` file loading
- ✅ Phase 3: Rust transpiler core features
- ✅ Phase 4: Generics support
- ✅ Phase 5: Traits and specs
- ✅ Phase 6: Closures and lambdas
- ✅ Phase 7: Standard library bindings
- ✅ Phase 8: Testing infrastructure
- ✅ Phase 9: Documentation

## Test Results

**Current status: 47/49 tests passing (96%)**

Passing tests cover:
- Basic expressions and statements
- Functions and methods
- Structs and enums
- Generics (types, traits, closures)
- Pattern matching
- Borrow checking
- Closures
- Delegation and inheritance
- Standard library imports

## Architecture

The transpiler is structured as:

1. **Parser**: Converts AutoLang source to AST
2. **AST**: Unified representation for all AutoLang code
3. **RustTrans**: Transpiles AST to Rust code
4. **Sink**: Output buffering

Key files:
- `src/trans/rust.rs` - Main transpiler implementation
- `src/ast/` - AST node definitions
- `src/parser.rs` - Parser implementation
- `test/a2r/` - Test suite

## Limitations

The following features are not yet fully supported:

1. **Advanced tag generics**: `tag May<T> { ... }` requires parser enhancements
2. **Some edge cases**: Whitespace formatting in complex scenarios

## Contributing

When adding new features:

1. Add test cases in `test/a2r/`
2. Update expected output files
3. Ensure type safety is preserved
4. Test with `cargo test -p auto-lang -- a2r`

## See Also

- [C Transpiler (a2c)](../a2c/README.md)
- [AutoVM Execution](../vm/README.md)
- [Language Reference](../../lang/README.md)
