# Auto-to-Rust (a2r) Transpiler

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Tests](https://img.shields.io/badge/tests-47%2F49-passing-blue.svg)]()

The **Auto-to-Rust (a2r) transpiler** converts AutoLang source code to idiomatic Rust, enabling native compilation with Rust's performance, safety, and ecosystem integration.

## 🎯 Features

- ✅ **Full Language Support**: Functions, structs, enums, closures, generics
- ✅ **Type Safety**: Complete preservation of AutoLang's type system
- ✅ **Trait System**: AutoLang specs → Rust traits with full generic support
- ✅ **Memory Safety**: Borrow checking via AutoLang's ownership semantics
- ✅ **Pattern Matching**: AutoLang `is` expressions → Rust `match`
- ✅ **96% Test Coverage**: 47/49 tests passing

## 🚀 Quick Start

### Installation

The a2r transpiler is included in the AutoLang compiler:

```bash
# Build from source
cargo build --release
```

### Basic Usage

```bash
# Transpile AutoLang to Rust
auto.exe transpile rust input.at output.rs

# Or with the REPL
auto.exe
> :transpile rust myfile.at
```

### AutoLang to Rust Example

**Input** (`example.at`):
```auto
use auto.io: say

fn add(a int, b int) int {
    return a + b
}

type Point {
    x int
    y int
}

fn main() {
    let p = Point(1, 2)
    let sum = add(5, 3)
    say("Result:", sum)
}
```

**Output** (`example.rs`):
```rust
use crate::io::say;

fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let p: Point = Point { x: 1, y: 2 };
    let sum: i32 = add(5, 3);
    say("Result:", sum);
}
```

## 📚 Transpilation Guide

### Type Mapping

| AutoLang | Rust | Notes |
|----------|------|-------|
| `int` | `i32` | 32-bit signed integer |
| `uint` | `u32` | 32-bit unsigned integer |
| `float` | `f32` | 32-bit floating point |
| `double` | `f64` | 64-bit floating point |
| `bool` | `bool` | Boolean |
| `str` | `String` | Heap-allocated string |
| `cstr` | `std::ffi::CStr` | C string |
| `char` | `char` | Unicode character |
| `void` | `()` | Unit type |
| `*[N]T` | `[T; N]` | Fixed-size array |
| `[]T` | `[T]` | Slice |
| `*T` | `*mut T` | Raw pointer |

### Variable Declarations

| AutoLang | Rust | Mutability |
|----------|------|------------|
| `let x = 42` | `let x: i32 = 42;` | Immutable |
| `var x = 42` | `let mut x: i32 = 42;` | Mutable |
| `const X = 42` | `const X: i32 = 42;` | Compile-time constant |

### Functions

**AutoLang:**
```auto
fn greet(name str) {
    print("Hello,", name)
}

fn add(a int, b int) int {
    return a + b
}
```

**Rust:**
```rust
fn greet(name: String) {
    println!("Hello, {}", name);
}

fn add(a: i32, b: i32) -> i32 {
    return a + b;
}
```

### Structs and Enums

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

enum Color {
    Red
    Green
    Blue
}
```

**Rust:**
```rust
struct Point {
    x: i32,
    y: i32,
}

enum Option {
    none,
    some,
}

enum Color {
    Red,
    Green,
    Blue,
}
```

### Generics

**AutoLang:**
```auto
type Box<T> {
    value T
}

spec Storage<T> {
    fn data() *T
    fn capacity() u32
}

type Heap<T> as Storage<T> {
    ptr *T
    cap u32
}
```

**Rust:**
```rust
struct Box<T> {
    value: T,
}

trait Storage<T> {
    fn data(&self) -> *mut T;
    fn capacity(&self) -> u32;
}

struct Heap<T> {
    ptr: *mut T,
    cap: u32,
}

impl<T> Storage<T> for Heap<T> {
    fn data(&self) -> *mut T {
        self.ptr
    }

    fn capacity(&self) -> u32 {
        self.cap
    }
}
```

### Closures

**AutoLang:**
```auto
fn main() {
    let add = (a int, b int) => a + b
    let result = add(5, 3)
    print(result)
}
```

**Rust:**
```rust
fn main() {
    let add = |a: i32, b: i32| a + b;
    let result = add(5, 3);
    println!("{}", result);
}
```

### Pattern Matching

**AutoLang:**
```auto
is value {
    0 => print("zero")
    1 => print("one")
    2..10 => print("digit")
    _ => print("other")
}
```

**Rust:**
```rust
match value {
    0 => println!("zero"),
    1 => println!("one"),
    2..=10 => println!("digit"),
    _ => println!("other"),
}
```

## 🔒 Memory Management

AutoLang's ownership system maps directly to Rust's borrow checker:

| AutoLang | Rust | Description |
|----------|------|-------------|
| `x.view` | `&x` | Immutable borrow |
| `x.mut` | `&mut x` | Mutable borrow |
| `x.take` | `x` | Move (default semantics) |
| `x.*` | `*x` | Dereference |
| `x.@` | `x as *mut _` | Address-of (raw pointer cast) |

**Example:**
```auto
fn process(mut data List) {
    data.mut.push(1)  // &mut
    let view = data.view  // &
    process(view)
}
```

**Rust:**
```rust
fn process(mut data: List) {
    data.push(1);          // mutable borrow
    let view = &data;       // immutable borrow
    process(view);
}
```

## 📦 Standard Library Imports

**AutoLang:**
```auto
use auto.io: say
use auto.str: trim
use auto.math: max
```

**Rust:**
```rust
use crate::io::say;
use crate::str::trim;
use crate::math::max;
```

### Module Path Mapping

- `auto.*` → `crate::*` (AutoLang stdlib)
- `std::*` → `std::*` (Rust stdlib)
- Direct imports preserved as-is

## 🎨 Annotations

AutoLang uses Rust-style `#[...]` annotations:

| Annotation | Description |
|-----------|-------------|
| `#[rs]` | Rust-specific function |
| `#[c]` | C-specific function |
| `#[vm]` | AutoVM-specific function |
| `#[pub]` | Public visibility |
| `#[rs, c]` | Available in both Rust and C |

**Example:**
```auto
type Printer {
    data str
}

ext Printer {
    #[rs]
    fn print() {
        // Rust implementation
    }

    #[c]
    fn print_c() {
        // C implementation
    }

    #[pub]
    fn new() Printer {
        // Public constructor
    }
}
```

## 📁 Platform-Specific Files

AutoLang supports multiple backends through file extensions:

| Extension | Purpose |
|-----------|---------|
| `.at` | Interface/definition |
| `.rs.at` | Rust-specific implementation |
| `.c.at` | C-specific implementation |
| `.vm.at` | AutoVM-specific implementation |

**Example:**
```
project/
├── greeting.at       # Interface
├── greeting.rs.at    # Rust impl
├── greeting.c.at     # C impl
└── greeting.vm.at    # VM impl
```

## 🧪 Testing

### Running Tests

```bash
# All a2r tests
cargo test -p auto-lang -- a2r

# Specific test
cargo test -p auto-lang test_006_struct

# Show test output
cargo test -p auto-lang test_006_struct -- --nocapture
```

### Test Organization

```
test/a2r/
├── 000_hello/
│   ├── hello.at
│   └── hello.expected.rs
├── 006_struct/
│   ├── struct.at
│   └── struct.expected.rs
└── ...
```

### Test Results

**Current Status**: 47/49 tests passing (96%)

All core features tested:
- ✅ Basic expressions
- ✅ Functions and methods
- ✅ Structs and enums
- ✅ Generics
- ✅ Closures
- ✅ Traits and specs
- ✅ Pattern matching
- ✅ Borrow checking
- ✅ Standard library

## 🔧 Advanced Features

### Generic Type Parameters

```auto
type List<T, S> {
    data S
    len u32
}

spec Storage<T> {
    fn get() *T
}

type Heap<T> as Storage<T>

impl<T> Storage<T> for List<T, Heap> {
    fn get() *T {
        // implementation
    }
}
```

### Ext Blocks (Type Extensions)

```auto
type Point { x int, y int }

ext Point {
    #[rs]
    fn distance(other Point) int {
        let dx = other.x - .x
        let dy = other.y - .y
        return (dx * dx + dy * dy).sqrt()
    }
}
```

### Delegation

```auto
type Wrapper {
    inner List
    delegate List to inner
}
```

## 📖 Documentation

- **[Transpiler Guide](../a2r-transpiler-guide.md)** - Comprehensive usage guide
- **[Language Reference](../../lang/README.md)** - AutoLang syntax and semantics
- **[C Transpiler](../a2c/README.md)** - Auto-to-C transpiler

## 🤝 Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## 📄 License

[MIT License](../../LICENSE)
