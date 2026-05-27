# 02 - Type System

## Status

**Implemented**: Postfix type modifiers (`T[]`, `T*`, `T&`, `T?`), `Type` enum with 20+ variants (in `ast/types.rs`), `Option(T)` and `Result(T)` as distinct AST types, unified enum system with three forms (scalar, homogeneous, heterogeneous) parsed in `parser.rs`, `TypeStore` with enum/type/spec/fn registries, Robinson unification in `infer/`, `GenericParam` with optional constraints.

**Partial**: Type inference engine exists (`infer/` module) but is not integrated with the parser. Generic constraint syntax (`#[with(...)]`) is designed but not yet in the parser.

**Planned**: Union types, `#[with(...)]` annotation-based generic constraints, parser integration of type inference.

## Design

### Type Modifiers (Postfix Notation)

AutoLang uses postfix type modifiers, chosen for consistency with C's declaration order. All modifiers attach after the base type:

| Category | Syntax | Example |
|----------|--------|---------|
| Dynamic array | `T[]` | `int[]` |
| Fixed array | `T[N]` | `int[10]` |
| Multi-dimensional | `T[3][10]` | `int[3][10]` |
| Pointer | `T*` | `int*` |
| Multi-level pointer | `T**` | `char**` |
| Reference | `T&` | `int&` |
| Optional | `T?` | `int?` |

Modifiers compose naturally: `int*[3]` is an array of 3 pointers to int; `int[]*` is a pointer to an int array.

Multi-dimensional array dimensions read left-to-right, outer-to-inner, matching C convention:

```auto
let arr int[3][10] = [[0..10], [1..11], [2..12]]
let last = arr[2][9]   // access order matches declaration order
```

### Type Inference

The inference engine (`crates/auto-lang/src/infer/`) implements a hybrid strategy:

- **Local bottom-up inference** for expressions -- infer types from literals and operations.
- **Simplified Hindley-Milner** for function signatures -- unification-based with scope management.

**Module structure**:
- `context.rs` -- Type environment with scoped variable bindings and constraint tracking.
- `unification.rs` -- Robinson's algorithm with occurs check. `Type::Unknown` acts as a wildcard that unifies with anything.
- `constraints.rs` -- Type constraint representations (Equal, Callable, Indexable, Subtype).
- `expr.rs` -- Expression inference covering 20+ expression types.
- `stmt.rs`, `functions.rs` -- Statement and function signature inference (future phases).

**Unification rules** (simplified):
```
(Unknown, T)        -> Ok(T)              // wildcard
(T, T)              -> Ok(T)              // same types
(Int, Uint)         -> Ok(Uint) + warning // coercion
(Int, Bool)         -> Err(Mismatch)      // incompatible
(Array(a), Array(b))-> unify element types and lengths
```

**Current status**: 285 tests passing, >95% coverage, zero warnings. Not yet connected to the parser -- the parser still uses the older `infer_type_expr()` function.

### TypeStore Unification

Historically, type information was scattered across four registries:

| Location | Contents | Issue |
|----------|----------|-------|
| `types.rs` (TypeStore) | TypeDecl, Fn, Spec, GenericTemplate | Original unified storage |
| `type_registry.rs` | `HashMap<String, Type>` | REPL persistence, duplicates TypeStore |
| `infer/registry.rs` | TypeDecl, ClassTemplate | Inference-specific, duplicates TypeStore |
| `Database.type_info_store` | TypeInfo (method names only) | Incomplete data |

The consolidation (Plan 084 follow-up) folds all registries into a single `TypeStore` using `Rc<T>` for shared immutable references:

```rust
pub struct TypeStore {
    type_decls: HashMap<AutoStr, Rc<TypeDecl>>,
    enum_decls: HashMap<AutoStr, Rc<EnumDecl>>,    // unified enum support
    fn_decls: HashMap<Name, Fn>,
    spec_decls: HashMap<AutoStr, SpecDecl>,
    generic_templates: HashMap<String, GenericTemplate>,
    type_aliases: HashMap<AutoStr, AutoStr>,
}
```

All consumers (parser, codegen, inference) read from and write to this shared store via `Arc<RwLock<TypeStore>>`. The `type_registry.rs` and `infer/registry.rs` modules are deprecated in favor of TypeStore.

### Generic Constraints with `#[with(...)]`

Simple generics use the familiar `<T>` syntax. When constraints are needed, AutoLang uses a `#[with(...)]` annotation instead of inline colons or where-clauses:

```auto
// Simple generic (no constraint)
fn identity<T>(x T) T {
    return x
}

// Constrained generic
#[with(I as Iter<T>, T, U)]
fn map(iter I, f T=>U) MapIter<I, T, U> {
    return MapIter { iter: iter, f: f }
}
```

**Design rationale**:
- Avoids `:` in type annotations (AutoLang reserves `:` for key-value pairs).
- Uses `as` for constraints, consistent with `impl X as Y` semantics.
- Keeps function signatures clean -- constraints live on a separate line.
- Integrates with existing `#[...]` annotation infrastructure.

The `TypeParam` struct in the AST already has an optional `constraint` field ready for this syntax.

### String Type System

AutoLang has three string-related types with distinct ownership semantics, mapping to Rust's `&str` / `String` distinction.

#### Three String Types

| Auto Keyword | Compiler Internal | Rust Equivalent | Semantics |
|-------------|-------------------|-----------------|-----------|
| `str` | `StrSlice` | `&str` | Borrowed string slice. Default type for string variables and function parameters. |
| `Str` | `StrOwned` | `String` | Owned heap-allocated string. Required inside containers. |
| (literal) `"hello"` | `StrFixed(N)` / `CStrLit` | `"hello"` | String literal. Not a named type — cannot be used in type annotations. Auto-converts to `str` or `Str` as needed. |

#### Auto Conversion Rules

Conversions flow in one direction: from smaller to larger ownership scope.

```
StrLit ──auto──> StrSlice ──auto──> StrOwned
("hello")         (str)             (Str)
```

| # | Rule | Auto Example | Effect |
|---|------|-------------|--------|
| 1 | StrLit → StrSlice | `let s = "hello"` | Literal automatically becomes `str` |
| 2 | StrLit → StrOwned | `entries.push("hello")` | Literal auto-adapts to `Str` where needed |
| 3 | StrSlice → StrOwned | `entries.push(s)` where `s: str` | `str` auto-adapts to `Str` (equivalent to `s.to(Str)`) |

Reverse conversions require explicit syntax: `s.to(str)` or `s.as_str()` in Rust terms.

#### Container Restriction

**Slice types (`str`) are not allowed as container element types.** Containers require owned data to avoid lifetime issues.

```auto
// WRONG — str in container not allowed
var entries List<str> = List.new()
var seen HashSet<str> = HashSet.new()

// CORRECT — use Str (owned)
var entries List<Str> = List.new()
var seen HashSet<Str> = HashSet.new()
```

When the type parameter is omitted, containers default to `Str`:

```auto
var entries = List.new()     // defaults to List<Str> → Vec<String>
var scores = HashMap.new()   // defaults to HashMap<Str, Str> → HashMap<String, String>
```

#### Rust Transpilation Mapping (a2r)

The a2r transpiler maps Auto string types to Rust according to context:

| Context | Auto `str` | Auto `Str` |
|---------|-----------|------------|
| Function parameter | `&str` | `String` |
| Return type | `String` | `String` |
| Variable/field | `String` | `String` |
| Container element | `String` | `String` |
| Literal expression | `"hello"` | `"hello".to_string()` |

This means `str` maps to `String` in most storage contexts (variables, fields, containers) to avoid Rust lifetime issues, but maps to `&str` in parameter position where borrowing is idiomatic. The transpiler inserts `.to_string()` only when the Rust type system requires it (StrLit/StrSlice assigned to a `String`-typed slot).

### Unified Enum System

AutoLang merges traditional enums and tagged unions into a single `enum` keyword with three physical forms:

**1. Scalar Enum** (C-style): Pure state with optional integer values and optional representation type.

```auto
enum Color { Red, Green, Blue }                  // default: repr u8, auto-increment
enum HttpCode u16 { OK = 200, NotFound = 404 }   // explicit repr and values
```

**2. Homogeneous Enum**: All variants share a single payload type. Supports direct field access without pattern matching.

```auto
type Point { x int, y int }
enum Vertex Point { LeftTop, RightTop }

fn reset(v Vertex) {
    v.x = 0   // direct O(1) offset access to shared payload
}
```

**3. Heterogeneous Enum** (ADT/sum type): Each variant may carry a different payload type.

```auto
enum Msg {
    Quit
    Move Point
    Write string
    Pair (string, string)
    Update { id int, val float }
}
```

**Implementation** (`ast/enums.rs`): `EnumKind` discriminates the three forms. `EnumDecl` holds the variant list and kind. The parser dispatches based on whether a type name follows `enum Name` and whether variants carry payloads.

**Built-in methods**: All enum instances have `.tag()` (integer discriminant) and `.name()` (string variant name).

**Migration from `tag`**: The `tag` keyword is deprecated. `tag Msg { Quit, Move Point }` becomes `enum Msg { Quit, Move Point }`.

### Union Types

AutoLang provides two union mechanisms:

**Raw `union`**: Memory-overlapping fields, mirroring C semantics. Used for low-level memory reinterpretation.

```auto
union MyUnion {
    i int
    f float
    c char
}
```

**`tag` (tagged union)**: Deprecated in favor of heterogeneous enum. Previously provided Rust-style algebraic data types. Pattern matching uses `is`:

```auto
is my_data {
    Int(i) -> print(i)
    Float(f) -> print(f)
}
```

The `is` keyword also provides direct `.tag` access to the discriminant, which Rust hides.

## Open Questions

- Whether `infer/registry.rs` has been fully removed or is still referenced by any code path.
- Multi-constraint syntax (`T as Clone + Serialize`) -- single constraint per type parameter is the current design, but compound constraints may be needed.
- How union types interact with the unified enum system in the type checker.
- Whether the `tag` keyword should be a hard error or a soft deprecation warning.

## Source Documents

- [raw/types.md](raw/types.md)
- [raw/type-inference.md](raw/type-inference.md)
- [raw/typestore-unification-design.md](raw/typestore-unification-design.md)
- [raw/generic-constraints.md](raw/generic-constraints.md)
- [raw/unified-enum.md](raw/unified-enum.md)
- [raw/union.md](raw/union.md)
