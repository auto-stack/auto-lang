# Plan 083: Auto-to-Rust (a2r) Transpiler with `.rs.at` and `#[rs]` Support

## Objective

Implement Rust transpiler (a2r) with platform-specific implementation files following the established pattern:
- `.rs.at` files for Rust-specific implementations (similar to `.c.at` for C, `.vm.at` for VM)
- `#[rs]` annotation for Rust-implemented functions
- Support hybrid functions: `#[rs, c, vm]`, `#[rs, c]`, `#[rs, vm]`

## Future Extensibility

Short annotation names enable future backends:
- `#[rs]` - Rust (this plan)
- `#[py]` - Python (future)
- `#[js]` - JavaScript/TypeScript (future)
- `#[go]` - Go (future)
- etc.

## Current State

✅ **Existing**:
- a2c transpiler: `.at` + `.c.at` with `#[c]` annotation
- AutoVM execution: `.at` + `.vm.at` with `#[vm]` annotation
- Hybrid functions: `#[c, vm]` (both C and VM implementations)

⏸️ **a2r Status**:
- Basic a2r transpiler exists but only handles `.at` files
- No `.rs.at` support
- No `#[rs]` annotation support
- Cannot handle Rust-specific features (generics, traits, macros)

## Design

### File Structure

```
stdlib/auto/
├── io.at           # Shared declarations
├── io.c.at         # C-specific implementations
├── io.rs.at        # Rust-specific implementations (NEW)
└── io.vm.at        # VM-specific implementations
```

### Annotation System

```auto
// Single backend
#[rs]
fn to_rust_vec() Vec<u8>;

#[c]
fn fopen(path str) *FILE;

#[vm]
fn run_script(code str) Value;

// Hybrid backends
#[rs, c]
fn read_file(path str) []u8;

#[rs, c, vm]
fn printf(fmt str, args ...) int;

#[rs, vm]
fn get_env(name str) str;
```

### Example: io.rs.at

```auto
// io.rs.at - Rust-specific implementations

use rust.std: fs::File, io::{BufReader, Read, Write}
use rust.std.io: {BufRead, BufReader as StdBufReader}

ext File {
    #[rs, pub]
    static fn open(path str) File {
        let file = File::open(path.to_rust_str())
        File(path: path, file: file)
    }

    #[rs, pub]
    fn read_text() str {
        if .file.is_none() {
            return ""
        }

        let mut reader = BufReader::new(.file.unwrap())
        let mut content = String::new()
        reader.read_to_string(&mut content)
        content.from_rust_string()
    }

    #[rs, pub]
    fn write_line(s str) {
        if .file.is_some() {
            writeln!(.file.unwrap(), "{}", s.to_rust_str())
        }
    }

    #[rs, pub]
    fn close() {
        drop(.file)
        .file = none
    }
}

#[rs, pub]
fn say(msg str) {
    println!("{}", msg.to_rust_str())
}
```

## Implementation Plan

### Phase 1: Parser Support for `#[rs]` Annotation (1-2 days)

**Goal**: Parse `#[rs]` annotation in function declarations

**Tasks**:
1. Extend annotation parser to recognize `#[rs]`
2. Support combined annotations: `#[rs, c]`, `#[rs, vm]`, `#[rs, c, vm]`
3. Store annotation metadata in AST

**Files**:
- `crates/auto-lang/src/parser.rs`: Add `#[rs]` parsing
- `crates/auto-lang/src/ast.rs`: Add `is_rs()` helper to function annotations

**Testing**:
```auto
// test_rs_annotation.at
type Test {
    #[rs]
    fn rust_only() int;

    #[rs, c]
    fn hybrid() int;
}
```

### Phase 2: `.rs.at` File Loading (1 day)

**Goal**: Load and parse `.rs.at` files alongside `.at` files

**Tasks**:
1. Extend file loader to find `.rs.at` files
2. Parse `.rs.at` contents as AutoLang code
3. Merge declarations from `.at` and `.rs.at`

**Files**:
- `crates/auto-lang/src/lib.rs`: Add `.rs.at` file discovery
- `crates/auto-lang/src/module.rs`: Track platform-specific files

**Testing**:
```bash
# Create test file: stdlib/test_rs.rs.at
cargo test test_load_rs_at_file
```

### Phase 3: Rust Transpiler Enhancements (3-5 days)

**Goal**: Transpile AutoLang to idiomatic Rust

**Core Features**:
1. Type mapping
   - AutoLang → Rust type conversion
   - Handle generic types: `List<T>`, `Option<T>`, `Result<T, E>`

2. Pattern matching
   - `is` expressions → `match` statements
   - `if/else` → `if/else`

3. Ownership & borrowing
   - Transpile AutoLang references to Rust references
   - Handle lifetimes automatically

4. Error handling
   - AutoLang `Result<T>` → Rust `Result<T, E>`
   - `?` operator for error propagation

**Type Mapping Table**:

| AutoLang | Rust |
|----------|------|
| `int` | `i32` |
| `uint` | `u32` |
| `i8` | `i8` |
| `u8` | `u8` |
| `i64` | `i64` |
| `u64` | `u64` |
| `float` | `f32` |
| `double` | `f64` |
| `bool` | `bool` |
| `str` | `String` |
| `cstr` | `std::ffi::CString` |
| `[]T` | `Vec<T>` |
| `[N]T` | `[T; N]` |
| `*T` | `*mut T` or `&mut T` |
| `T?` (optional) | `Option<T>` |
| `Result<T, E>` | `Result<T, E>` |

**Files**:
- `crates/auto-lang/src/trans/rust.rs`: Main transpiler
- `crates/auto-lang/src/trans/rust/types.rs`: Type mapping
- `crates/auto-lang/src/trans/rust/expr.rs`: Expression transpilation
- `crates/auto-lang/src/trans/rust/stmt.rs`: Statement transpilation

### Phase 4: Generics Support (2-3 days)

**Goal**: Transpile AutoLang generics to Rust generics

**Examples**:

AutoLang input:
```auto
type List<T> {
    #[rs, c]
    fn new() List<T> {
        List(data: [])
    }

    #[rs, c]
    fn push(elem T) {
        .data.push(elem)
    }
}
```

Rust output:
```rust
pub struct List<T> {
    data: Vec<T>,
}

impl<T> List<T> {
    pub fn new() -> List<T> {
        List { data: Vec::new() }
    }

    pub fn push(&mut self, elem: T) {
        self.data.push(elem);
    }
}
```

**Files**:
- `crates/auto-lang/src/trans/rust/generics.rs`: Generic type handling

### Phase 5: Traits and Specs (3-4 days)

**Goal**: Transpile AutoLang `spec` and `ext` to Rust traits

**AutoLang spec → Rust trait**:

```auto
spec Storage<T> {
    fn data() *T
    fn capacity() u32
    fn try_grow(min_cap u32) bool
}
```

```rust
pub trait Storage<T> {
    fn data(&mut self) -> *mut T;
    fn capacity(&self) -> u32;
    fn try_grow(&mut self, min_cap: u32) -> bool;
}
```

**AutoLang ext → Rust impl**:

```auto
ext File {
    #[rs]
    fn read_line() str { ... }
}
```

```rust
impl File {
    pub fn read_line(&mut self) -> String {
        // ...
    }
}
```

**Files**:
- `crates/auto-lang/src/trans/rust/traits.rs`: Trait transpilation

### Phase 6: Closures and Lambdas (2 days)

**Goal**: Transpile AutoLang closures to Rust closures

**Example**:

```auto
// AutoLang
let add = |x int, y int| int { x + y }
let result = add(1, 2)
```

```rust
// Rust
let add = |x: i32, y: i32| -> i32 { x + y };
let result = add(1, 2);
```

**Files**:
- `crates/auto-lang/src/trans/rust/closure.rs`: Closure transpilation

### Phase 7: Standard Library Bindings (5-7 days)

**Goal**: Create Rust implementations for stdlib types

**Priority Modules**:
1. **io.rs.at**: File I/O operations
2. **sys.rs.at**: System operations
3. **collections.rs.at**: List, HashMap, HashSet
4. **str.rs.at**: String operations
5. **math.rs.at**: Math functions

**Example: str.rs.at**:

```auto
// str.rs.at - Rust string implementations

use rust.std: string::{String, ToString}
use rust.std.str: FromStr

ext str {
    #[rs, pub]
    fn to_upper() str {
        self.to_rust_str().to_uppercase().from_rust_string()
    }

    #[rs, pub]
    fn contains(pattern str) bool {
        self.to_rust_str().contains(pattern.to_rust_str())
    }

    #[rs, pub]
    fn split(pattern str) []str {
        self.to_rust_str()
            .split(pattern.to_rust_str())
            .map(|s| s.from_rust_string())
            .collect()
    }
}
```

**Files**:
- `stdlib/auto/str.rs.at`
- `stdlib/auto/io.rs.at`
- `stdlib/auto/collections.rs.at`

### Phase 8: Testing Infrastructure (2-3 days)

**Goal**: Comprehensive test coverage for a2r

**Test Structure**:
```
crates/auto-lang/test/a2r/
├── 000_hello/              # Basic hello world
│   ├── hello.at
│   ├── hello.rs.at
│   └── hello.expected.rs
├── 001_generics/           # Generic types
├── 002_traits/             # Trait implementations
├── 003_closures/           # Closure expressions
├── 004_pattern_match/      # Pattern matching
└── 005_stdlib/             # Standard library usage
```

**Test Runner**:
```bash
# Run all a2r tests
cargo test -p auto-lang -- a2r

# Run specific test
cargo test -p auto-lang test_001_generics
```

**Files**:
- `crates/auto-lang/src/trans/rust.rs`: Add `test_a2r()` function
- `crates/auto-lang/test/a2r/`: Test cases

### Phase 9: Documentation (2 days)

**Goal**: User and developer documentation

**Documents**:
1. **User Guide**: `docs/guides/a2r-guide.md`
   - How to use a2r transpiler
   - Writing `.rs.at` files
   - Rust-specific features

2. **CLI Reference**: `docs/cli/a2r-cli.md`
   - `auto rs` command
   - Options and flags

3. **Architecture**: `docs/architecture/a2r.md`
   - Type system mapping
   - Generics implementation
   - Trait/spec translation

**Files**:
- `docs/guides/a2r-guide.md`
- `docs/cli/a2r-cli.md`
- `docs/architecture/a2r.md`

### Phase 10: Integration and Polish (2-3 days)

**Goal**: Integrate a2r into build system

**Tasks**:
1. Add `auto rs` CLI command
2. Build system support for Rust targets
3. Error messages and diagnostics
4. Performance optimization

**CLI Usage**:
```bash
# Transpile to Rust
auto rs input.at -o output.rs

# Build with Rust backend
auto build --backend=rust

# Run with Rust backend
auto run --backend=rust main.at
```

**Files**:
- `crates/auto/src/main.rs`: Add `Rust` subcommand
- `crates/auto-man/src/builder/rust/`: Rust build backend

## Success Criteria

✅ **Completion Checklist**:
- [ ] Parser recognizes `#[rs]` annotation
- [ ] `.rs.at` files load and merge with `.at` declarations
- [ ] All core types transpile correctly (int, str, arrays, etc.)
- [ ] Generics transpile to Rust generics
- [ ] Traits/specs transpile to Rust traits
- [ ] Closures transpile to Rust closures
- [ ] Standard library has Rust implementations
- [ ] Test suite passes (90%+ coverage)
- [ ] Documentation complete
- [ ] CLI integration works

## Estimated Timeline

**Total**: 22-30 days

**Breakdown**:
- Phase 1: 1-2 days (parser)
- Phase 2: 1 day (file loading)
- Phase 3: 3-5 days (core transpiler)
- Phase 4: 2-3 days (generics)
- Phase 5: 3-4 days (traits)
- Phase 6: 2 days (closures)
- Phase 7: 5-7 days (stdlib)
- Phase 8: 2-3 days (testing)
- Phase 9: 2 days (documentation)
- Phase 10: 2-3 days (integration)

## Open Questions

1. **Ownership Model**: How to handle AutoLang's reference counting vs Rust's ownership?
   - *Option A*: Use `Rc<T>` for shared references
   - *Option B*: Use Rust lifetimes and borrowing
   - *Recommendation*: Start with `Rc<T>`, optimize to lifetimes later

2. **Error Handling**: AutoLang uses panic-based errors, Rust uses `Result`
   - *Option A*: Convert all panics to `Result`
   - *Option B*: Keep panics for unrecoverable errors
   - *Recommendation*: Hybrid - `Result` for expected errors, panic for bugs

3. **Async Support**: Should a2r support `async`/`await`?
   - *Decision*: Defer to future plan (Plan 08X)

## Related Plans

- Plan 025: C string support (cstr)
- Plan 052: Storage-based lists with generics
- Plan 059: Generic impl blocks
- Plan 060: Closure implementation
- Plan 073: Object literal support
- Plan 082: AutoCache global build cache

## References

- **a2c Implementation**: `crates/auto-lang/src/trans/c.rs`
- **VM Implementation**: `crates/auto-lang/src/vm/`
- **C .c.at Files**: `stdlib/auto/*.c.at`
- **Rust Book**: https://doc.rust-lang.org/book/
