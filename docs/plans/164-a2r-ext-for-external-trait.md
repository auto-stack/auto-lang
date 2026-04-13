# Plan 164: a2r — `ext Type for Trait` (External Trait Implementation)

**Date**: 2026-04-13
**Status**: Design approved
**Scope**: a2r transpiler only
**Parent**: Plan 159 Phase 6B-2 (item 6B-4.4)

## Goal

Allow AutoLang code to implement external Rust traits (Display, Debug, Clone, Default, etc.) for Auto-defined types, via a new `ext Type for Trait` syntax.

## Syntax

```auto
// Inherent methods (existing, unchanged)
ext MyType {
    fn helper() { ... }
}

// NEW: External trait implementation
ext MyType for Display {
    fn fmt() String { ... }
}

// With generics on the trait
ext MyType for From<String> {
    fn from(s String) MyType { ... }
}

// With generics on the type
ext List<T> for From<Vec<T>> {
    fn from(v Vec<T>) List<T> { ... }
}
```

Reading: "extend MyType for Display" — type first, then trait.

## What Changes

### 1. AST (`ast/ext.rs`)

Add `trait_name: Option<Name>` to `Ext`:

```rust
pub struct Ext {
    pub target: Name,           // Type being extended (existing)
    pub trait_name: Option<Name>,  // NEW: external trait name (None = inherent impl)
    pub generic_params: Vec<GenericParam>,  // existing
    pub fields: Vec<Member>,    // existing
    pub methods: Vec<Fn>,       // existing
    // ...
}
```

### 2. Parser (`parser.rs` — `parse_ext_stmt()`)

After parsing the target type name, check for `for` keyword:

```
ext <Type> [for <TraitName>] { methods }
         ^^^^^^^^^^^^^^^^^^^^
         new optional clause
```

- No `for` → `trait_name = None` (inherent impl, existing behavior)
- `for` present → parse trait name, store in `trait_name`

### 3. Transpiler (`trans/rust.rs` — `ext_decl()`)

```rust
fn ext_decl(&mut self, ext: &Ext, sink: &mut Sink) {
    match &ext.trait_name {
        Some(trait_name) => {
            // NEW: impl Trait for Type { ... }
            write!(sink.body, "impl {} for {}", trait_name, ext.target)?;
        }
        None => {
            // EXISTING: impl Type { ... }
            write!(sink.body, "impl {}", ext.target)?;
        }
    }
    // ... generics, methods ...
}
```

### 4. Other transpilers (a2c, a2py, a2ts, a2ark)

No changes. When `trait_name` is `Some`, these transpilers ignore it and treat the block as an inherent impl (same as `trait_name = None`). External traits are a Rust-specific concept.

## What Stays the Same

- `ext Type { }` → `impl Type { }` (inherent, unchanged)
- `type X as Spec { }` → `impl Spec for X { }` (unchanged, spec-based impl)
- No validation of trait names — any name passes through
- `impl` keyword remains a synonym for `ext`

## Test Cases

New a2r test directory: `153_ext_for/`

```auto
// 153_ext_for/ext_for.at

type Point {
    x int
    y int
}

ext Point for Default {
    fn default() Point {
        return Point(x: 0, y: 0)
    }
}

ext Point for Display {
    fn fmt() String {
        return f"(${.x}, ${.y})"
    }
}

fn main() {
    let p = Point.default()
    let s = p.fmt()
}
```

Expected Rust output:

```rust
struct Point {
    x: i32,
    y: i32,
}

impl Default for Point {
    fn default() -> Point {
        Point { x: 0, y: 0 }
    }
}

impl Display for Point {
    fn fmt(&self) -> String {
        format!("({}, {})", self.x, self.y)
    }
}

fn main() {
    let p = Point::default();
    let s = p.fmt();
}
```

## Files to Modify

| File | Change |
|------|--------|
| `crates/auto-lang/src/ast/ext.rs` | Add `trait_name: Option<Name>` field |
| `crates/auto-lang/src/parser.rs` | Parse `for TraitName` in `parse_ext_stmt()` |
| `crates/auto-lang/src/trans/rust.rs` | Emit `impl Trait for Type` when `trait_name` is Some |
| `crates/auto-lang/test/a2r/153_ext_for/` | New test case |
| `crates/auto-lang/test/a2r/153_ext_for/ext_for.at` | Input |
| `crates/auto-lang/test/a2r/153_ext_for/ext_for.expected.rs` | Expected output |

## Success Criteria

- [ ] `ext Type for TraitName { }` parses correctly
- [ ] a2r generates `impl TraitName for Type { }`
- [ ] Existing `ext Type { }` (inherent impl) still works
- [ ] Existing `type X as Spec { }` (spec-based impl) still works
- [ ] All existing a2r tests still pass
- [ ] New test `153_ext_for` passes
