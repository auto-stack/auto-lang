# Plan 165: a2r — Struct Destructuring in `is` Match Arms

**Date**: 2026-04-13
**Status**: Design approved
**Scope**: a2r transpiler only
**Parent**: Plan 159 Phase 6B-4.5

## Goal

Support Rust-style `{field1, field2}` struct destructuring in `is` statement match arms, for both enum variant patterns and plain struct patterns.

## Syntax

```auto
// Existing: tuple variant with single binding
is msg {
    Atom.Int(i) => print(i)
}

// NEW: enum variant with struct destructuring
is msg {
    Message.User { content } => print(content)
    Message.Assistant { content, tool_calls: tc } => process(content, tc)
}

// NEW: plain struct destructuring
is point {
    Point { x, y } => print(x)
}
```

Mapping:

| AutoLang | Rust |
|----------|------|
| `Atom.Int(i)` (existing) | `Atom::Int(i)` |
| `Message.User { content }` (new) | `Message::User { content }` |
| `Message.User { content: c }` (new) | `Message::User { content: c }` |
| `Point { x, y }` (new) | `Point { x, y }` |

## What Changes

### 1. AST (`ast/cover.rs`)

Add `StructCover` for struct destructuring patterns:

```rust
/// Struct field binding in a destructuring pattern
#[derive(Debug, Clone)]
pub struct FieldBinding {
    pub field: AutoStr,       // Field name
    pub binding: AutoStr,     // Binding name (same as field if shorthand)
}

/// Struct destructuring pattern: Type { field1, field2: alias }
/// Used in is branches: is x { Point { x, y } => ... }
#[derive(Debug, Clone)]
pub struct StructCover {
    pub type_name: AutoStr,           // "Point" or "Message" (before the variant)
    pub variant: Option<AutoStr>,     // Some("User") for enum variant, None for plain struct
    pub fields: Vec<FieldBinding>,    // [{ field: "content", binding: "content" }]
}
```

Add new `Expr` variant: `Expr::StructPattern(StructCover)`.

### 2. Parser (`parser.rs` — `is_branch_cond_expr()`)

After parsing the initial identifier/tag expression in `is_branch_cond_expr()`, check if the next token is `{` (LBrace). If so, parse struct destructuring:

```
Name { field1, field2: alias }       → StructCover { type_name, None, fields }
Name.VariantName { field1, field2 }  → StructCover { type_name, Some(variant), fields }
```

Parsing rules:
- `{` opens the destructuring
- Each field: `name` or `name: binding`
- Fields separated by `,`
- `}` closes the destructuring

### 3. Transpiler (`trans/rust.rs`)

Add handling for `Expr::StructPattern(StructCover)`:

```rust
Expr::StructPattern(sc) => {
    match &sc.variant {
        Some(variant) => {
            // Enum variant: Type::Variant { fields }
            write!(out, "{}::{} {{ ", sc.type_name, variant)?;
        }
        None => {
            // Plain struct: Type { fields }
            write!(out, "{} {{ ", sc.type_name)?;
        }
    }
    for (i, fb) in sc.fields.iter().enumerate() {
        if fb.field == fb.binding {
            write!(out, "{}", fb.field)?;  // shorthand
        } else {
            write!(out, "{}: {}", fb.field, fb.binding)?;  // renamed
        }
        if i < sc.fields.len() - 1 { write!(out, ", ")?; }
    }
    write!(out, " }}")
}
```

### 4. Other transpilers

No changes. Struct destructuring is a Rust-specific pattern. Other backends can ignore `StructPattern` or emit a comment.

## Files to Modify

| File | Change |
|------|--------|
| `crates/auto-lang/src/ast/cover.rs` | Add `StructCover`, `FieldBinding` types |
| `crates/auto-lang/src/ast/mod.rs` | Re-export new types |
| `crates/auto-lang/src/parser.rs` | Parse `{...}` in `is_branch_cond_expr()` |
| `crates/auto-lang/src/trans/rust.rs` | Emit struct destructuring patterns |
| `crates/auto-lang/test/a2r/154_struct_destructure/` | New test case |

## Also Marked Done

- **6B-4.6** (`serde_json::json!` macro): Not needed. AutoLang's native object literal `{key: value}` already transpiles to Rust struct syntax, covering the AutoCode use case.

## Test Cases

New a2r test: `154_struct_destructure/`

```auto
// Heterogeneous enum with struct-like variants
enum Message {
    Quit
    Move Point
    Write String
}

type Point {
    x int
    y int
}

fn main() {
    let msg = Message.Move(Point(x: 1, y: 2))

    is msg {
        Message.Quit => print("quit")
        Message.Move(p) => {
            is p {
                Point { x, y } => print(x)
            }
        }
        Message.Write(text) => print(text)
    }
}
```

## Success Criteria

- [ ] `Type { field1, field2 }` parses in is branches
- [ ] `Type.Variant { field1, field2: alias }` parses in is branches
- [ ] a2r generates correct Rust struct destructuring
- [ ] Existing `Type.Variant(binding)` (tuple style) still works
- [ ] All existing a2r tests pass
- [ ] New test `154_struct_destructure` passes
