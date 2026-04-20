# 10 - Language Syntax

## Status

**Implemented:**
- Dot notation for field access: `Expr::Dot(Box<Expr>, Name)` in AST, parsed and codegen'd.
- OOP type-bound methods: `type` blocks contain both `static fn` and `fn` declarations. The `ext` keyword supports type extension (`Stmt::Ext` in AST). Transpilers generate prefixed C functions (e.g., `May_empty()`, `May_is_empty(&self)`).
- Module system (`mod`): file-based and folder-based modules are supported. `use` statements resolve local, `super`, and `pac` paths.
- Basic parameter passing: `ref`, `mut` annotations on function parameters. Default is move/copy depending on type.

**Partial / Planned:**
- Unified dot-notation attributes (`.?`, `.*`, `.@`, `.as`, `.view`, `.mut`, `.take`) are designed but only basic `.field` access is implemented. The symbol-based properties (`.?`, `.*`, `.@`) and keyword properties (`.as`, `.to`, `.view`) are not yet in the parser.
- Bit operations: no bitfield view syntax (`bits()`, `bit()`), no declarative bitfield types, no `shl`/`shr`/`sar`/`rol`/`ror` methods. Only basic arithmetic is available.
- Auto Flow (`|>` pipe operator, `Iter<T>` spec, `!` materialization) is fully designed but not implemented. No iterator adapters or `Iterable` trait exists.
- Potential keywords: a large candidate list is curated but no decisions have been finalized on which additional keywords to adopt.
- Code organization (`lib`/`pac`) beyond module-level is not yet formalized in the build system.

## Design

### Unified Dot Notation

Auto eliminates prefix operators and nested brackets by converging all post-fix operations under the dot (`.`). The design philosophy: *every dot is one step of data refinement or permission change.*

**Attribute symbols** (low-level physical operations):

| Syntax | Name | Purpose |
|---|---|---|
| `.?` | Unwrap | Safe unwrap of `Option`/`Result` |
| `.*` | Deref | Dereference a pointer |
| `.@` | Address | Take address of a value |

**Attribute keywords** (high-level logical operations):

| Syntax | Purpose |
|---|---|
| `.view` / `.mut` / `.take` | Ownership and permission switching |
| `.as` / `.to` | Type conversion (e.g., `500.as.u64`, `val.to.str`) |
| `.fixed` / `.dynamic` | Buffer mode adaptation (MCU vs. PC) |

**Visual consistency rules:**
- Attribute chains: no spaces around dot (e.g., `ptr.*.as.f32`).
- Binary operations: spaces required (e.g., `a * b`).

Example -- pointer dereference, type cast, and arithmetic in one chain:
```auto
let ptr = n.@          // take address
let val = ptr.*.as.f32 * 1.5   // deref, cast, multiply
```

### Function Parameter Passing

Auto defines four parameter passing modes, each combining a data transfer mechanism with a mutability choice:

| Mode | Syntax | Semantics |
|---|---|---|
| Copy | (default for primitive types) | Value is copied; caller retains original |
| Move | (default for non-Copy types) | Ownership transfers to callee; caller loses access |
| Borrow | `ref` parameter | Callee gets read-only reference; caller retains ownership |
| Mutable ref | `mut` parameter | Callee gets read-write reference; caller must declare `var` |

The critical insight from Rust's design is that *move + mut parameter* allows modifying a value inside a function without copying, while keeping the mutability scope precise to the function body. Auto adopts this approach.

### Bit Operations and Bitfield Views

Auto provides a two-layer system for bit manipulation:

**Layer 1: Bitwise methods** -- method-call syntax for all bitwise operations:

| Method | Purpose |
|---|---|
| `val.and(mask)` | Bitwise AND |
| `val.or(mask)` | Bitwise OR |
| `val.xor(mask)` | Bitwise XOR |
| `val.not()` | Bitwise NOT |
| `val.shl(n)` / `val.shr(n)` | Logical shift left/right |
| `val.sar(n)` | Arithmetic shift right (sign-preserving) |
| `val.rol(n)` / `val.ror(n)` | Rotate left/right |

**Layer 2: Bitfield views** -- structured access to register fields:

```auto
type CtrlReg u32 {
    mode    = bits(0, 4)   // bits 0-3
    rate    = bits(4, 4)   // bits 4-7
    enable  = bit(8)       // bit 8
    pending = bit(9)       // bit 9
}
```

Operations on bitfield views: `.read()`, `.write(value)`, `.on()`, `.off()`, `.flip()`, `.test() -> bool`. All view operations expand to optimal bitmask instructions at compile time (zero-cost abstraction).

Bit-scanning utilities: `.count_ones()`, `.leading_zeros()`, `.trailing_zeros()`.

**Design rules:**
- Bit indices and lengths are decimal.
- Bit values (masks, literals) should use `0b` binary notation.
- Endianness is consistent: `bits(0, 4)` always means the lowest 4 bits (LSB).
- Out-of-range operations (e.g., `bit(35)` on `u32`) are compile errors.

### OOP Design (Type-Bound Methods)

Auto is an object-oriented language with Java-style method placement: all methods are defined inside `type` blocks, not in separate `impl` blocks (unlike Rust).

**Method types:**

```auto
type Point<T> {
    x T
    y T

    static fn zero() Point<T> { ... }   // type-level, call as Point<T>.zero()
    fn is_zero() bool { .x == 0 && .y == 0 }  // instance-level, call as p.is_zero()
}
```

**Instance method rules:**
- `self` keyword references the current instance (optional; fields can be accessed directly with `.field` syntax).
- The `mut fn` prefix declares a method that modifies the instance.

**Transpilation to C** (`a2c`):
- Static methods become `TypeName_methodName()` with no `self` parameter.
- Instance methods become `TypeName_methodName(TypeName* self)`.
- No manual prefixes needed in Auto source; the transpiler adds them automatically.

**Type extension** (`ext`): The `ext` keyword provides two capabilities depending on scope:
- *Intra-module*: Can add private fields to a type (changing memory layout for the current platform).
- *Cross-module*: Can only add methods, not fields (preserves ABI).

This enables the stdlib's multi-platform pattern: a common interface in `io.at` with platform-specific private fields added via `ext` in `io.c.at` or `io.vm.at`.

### Potential Keyword List

Auto maintains a curated list of short (2-3 letter) candidate keywords for future language features. The list is organized by length:

**2-letter candidates** (selected): `as`, `at`, `do`, `if`, `in`, `is`, `my`, `no`, `ok`, `op`, `or`, `to`, `up`, `us`.

**3-letter candidates** (selected notable ones):

| Keyword | Intended meaning |
|---|---|
| `act` | action |
| `def` | define |
| `del` | delete |
| `ext` | extend (implemented) |
| `gen` | generate |
| `get` | getter |
| `let` | immutable binding (implemented) |
| `mut` | mutable (implemented) |
| `pac` | package (implemented) |
| `pub` | public (implemented) |
| `ref` | reference (implemented) |
| `set` | setter |
| `var` | variable binding (implemented) |

The full list contains over 400 candidates. Selection criteria: pronounceable, unambiguous, not conflicting with existing syntax, and meaningful to English speakers.

### Code Organization (mod / lib / pac)

Auto code is organized in three tiers:

| Tier | Name | Analog | Description |
|---|---|---|---|
| 1 | `mod` (Module) | Rust module | A single file or folder. Folder modules have an entry file matching the folder name (`net/net.at`). |
| 2 | `lib` (Library) | C library | Multiple modules forming a complete feature set. |
| 3 | `pac` (Package) | Cargo workspace | One or more libraries plus executables. The unit of dependency management. |

**Module rules:**
- Each `.at` file is a module.
- Folders are modules with sub-modules.
- `use super.X` navigates to parent directory.
- `use pac.X` navigates from package root.

**Import examples:**
```auto
use db              // ./db.at or ./db/mod.at
use super.db        // ../db.at
use pac.db          // package root search
use db: load, save  // specific symbols
```

Ambiguity check: if both `name.at` and `name/mod.at` exist, the compiler raises an error.

### Standard Library Organization

The stdlib uses the `ext`-based multi-platform pattern. Each stdlib module defines a platform-independent interface in its main `.at` file, then fills in platform-specific details via `ext` blocks in target-specific files (e.g., `.c.at` for C transpilation, `.vm.at` for AutoVM).

**Key principle**: *Interface in the main file, implementation in the platform file.*

The compiler collects all `ext` blocks for the active target, merges the fields into the type layout, and generates appropriate code. Users see only the public interface regardless of target.

### Auto Flow (Functional Programming)

Auto Flow is a planned functional programming interface built on lazy iterators. The design follows: *"Lazy by default, eager by bang."*

**Core abstractions:**

- `spec Iter<T>` -- produces elements one at a time via `fn next() ?T`.
- `spec Iterable<T>` -- containers that can produce an iterator via `fn iter() IterT`.

**Iterable auto-forwarding**: `Iterable<T>` provides default implementations for `map`, `filter`, `reduce`, etc. that call `self.iter()` then delegate to the iterator. This means users write `list.map(f).filter(p)!` instead of `list.iter().map(f).filter(p)!`.

**Operator categories:**

| Category | Operators | Behavior |
|---|---|---|
| Lazy adapters | `map`, `filter`, `take`, `skip`, `enumerate`, `zip`, `chain`, `flatten`, `inspect` | No iteration, no allocation; just stack wrapper structs |
| Terminal | `reduce`, `count`, `any`, `all`, `find`, `for_each` | Triggers iteration, returns non-iterator value |
| Materialization | `expr!` | Triggers iteration, collects into default storage (heap on PC, fixed on MCU) |

**Design decision: dot vs. pipe.** Auto uses dot-chaining (Rust/Java style) rather than a pipe operator (`|>`). Reasons: IDE auto-completion is natural after `.`, consistency with struct method calls, and a smaller symbol set. The `!` suffix handles materialization without needing a separate operator.

## Open Questions

- Should `.?`, `.*`, `.@` be parsed as special dot-notation forms, or as standalone postfix operators?
- How should bitfield view types interact with the generic type system?
- Should `lib` and `pac` tiers have manifest files, or is folder convention sufficient?
- Which 2-3 letter keywords from the candidate list should be reserved now vs. later?

## Source Documents

- [raw/dot-notation.md](raw/dot-notation.md)
- [raw/functions.md](raw/functions.md)
- [raw/bit-operations.md](raw/bit-operations.md)
- [raw/OOP.md](raw/OOP.md)
- [raw/potential_keywords.md](raw/potential_keywords.md)
- [raw/organizations.md](raw/organizations.md)
- [raw/stdlib-organization.md](raw/stdlib-organization.md)
- [raw/auto-flow.md](raw/auto-flow.md)
