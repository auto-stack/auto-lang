# Plan 207: Enum Multi-Field Destructuring + Named Construction

> **Status: ✅ COMPLETE** — Multi-field destructuring and named-arg construction implemented in parser/codegen/engine
>
> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable `is err { Api(status, msg) -> ... }` multi-field destructuring and `Api(status: 429, msg: "err")` named-arg construction for enum variants with multiple fields.

**Architecture:** Extend `TagCover.elem` to `bindings: Vec<AutoStr>`, update parser to parse comma-separated bindings, replace the single-field extraction loop in codegen with a multi-field loop. For construction, sort `Arg::Pair` named args by `template.fields` index before compiling.

**Tech Stack:** Rust, AutoVM codegen, parser, AST

**Depends on:** Plan 201 Phase 1A-C (✅ completed — `EnumItem.fields`, parser `{ field Type }` syntax, codegen multi-field registration)

---

## Task 1: AST — Change TagCover to support multiple bindings

**Files:**
- Modify: `crates/auto-lang/src/ast/cover.rs:10-15`

**Step 1: Update TagCover struct**

Current:
```rust
pub struct TagCover {
    pub kind: AutoStr,
    pub tag: AutoStr,
    pub elem: AutoStr,
}
```

Change to:
```rust
pub struct TagCover {
    pub kind: AutoStr,
    pub tag: AutoStr,
    pub bindings: Vec<AutoStr>,
}
```

**Step 2: Update all references to `tag_cover.elem`**

Search for `tag_cover.elem` and `.elem` on `TagCover` across the codebase. Every reference needs updating:
- `tag_cover.elem` → `tag_cover.bindings[0]` or `tag_cover.bindings.first()`
- `_` wildcard check: `tag_cover.elem.as_str() != "_"` → `!tag_cover.bindings.is_empty() && tag_cover.bindings[0].as_str() != "_"`
- `Display` impl if it references `.elem`

Key files to check:
- `crates/auto-lang/src/ast/cover.rs` — Display impl
- `crates/auto-lang/src/vm/codegen.rs:2526-2576` — destructuring codegen
- `crates/auto-lang/src/parser.rs:2927-2952` — tag_cover parser
- `crates/auto-lang/src/trans/c.rs` — C transpiler
- `crates/auto-lang/src/trans/rust.rs` — Rust transpiler

**Step 3: Verify compilation**

Run: `cargo build -p auto-lang`
Expected: compile errors only where `bindings` access pattern needs adjustment. Fix all.

**Step 4: Run existing tests**

Run: `cargo test -p auto-lang -- vm_file_tests`
Expected: all existing tests still pass (single-binding patterns still work)

**Step 5: Commit**

```
refactor(ast): change TagCover.elem to TagCover.bindings Vec
```

---

## Task 2: Parser — Multi-binding destructuring syntax

**Files:**
- Modify: `crates/auto-lang/src/parser.rs:2927-2952`

**Step 1: Update `tag_cover` method**

Current (single binding):
```rust
fn tag_cover(&mut self, tag_name: &Name) -> AutoResult<Expr> {
    self.expect(TokenKind::Dot)?;
    let tag_field = self.parse_name()?;

    if self.is_kind(TokenKind::LParen) {
        self.next();
        let elem = self.parse_name()?;
        self.expect(TokenKind::RParen)?;
        return Ok(Expr::Cover(Cover::Tag(TagCover {
            kind: tag_name.clone(),
            tag: tag_field,
            elem,
        })));
    } else {
        return Ok(Expr::Cover(Cover::Tag(TagCover {
            kind: tag_name.clone(),
            tag: tag_field,
            elem: Name::from("_"),
        })));
    }
}
```

New (multi-binding):
```rust
fn tag_cover(&mut self, tag_name: &Name) -> AutoResult<Expr> {
    self.expect(TokenKind::Dot)?;
    let tag_field = self.parse_name()?;

    if self.is_kind(TokenKind::LParen) {
        self.next();
        let mut bindings = vec![];
        if !self.is_kind(TokenKind::RParen) {
            bindings.push(self.parse_name()?);
            while self.is_kind(TokenKind::Comma) {
                self.next();
                bindings.push(self.parse_name()?);
            }
        }
        self.expect(TokenKind::RParen)?;
        return Ok(Expr::Cover(Cover::Tag(TagCover {
            kind: tag_name.clone(),
            tag: tag_field,
            bindings,
        })));
    } else {
        return Ok(Expr::Cover(Cover::Tag(TagCover {
            kind: tag_name.clone(),
            tag: tag_field,
            bindings: vec![Name::from("_")],
        })));
    }
}
```

**Step 2: Verify compilation + tests**

Run: `cargo test -p auto-lang -- vm_file_tests`
Expected: all pass

**Step 3: Commit**

```
feat(parser): support multi-binding destructuring Variant(a, b, c)
```

---

## Task 3: Codegen — Multi-field destructuring loop

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs:2526-2576`

**Step 1: Replace the TODO single-field extraction with multi-field loop**

Current code (around line 2555-2575) has:
```rust
// TODO: multi-field destructuring when supported by parser
if field_count >= 1 {
    self.emit(OpCode::DUP);
    self.emit(OpCode::GET_GENERIC_FIELD);
    self.emit_u32(0);
    let var_idx = self.add_var(tag_cover.elem.as_str());
    self.emit_store_loc(var_idx);
}
```

Replace with:
```rust
let binding_count = tag_cover.bindings.len().min(field_count);
for i in 0..binding_count {
    let binding = &tag_cover.bindings[i];
    if binding.as_str() != "_" {
        self.emit(OpCode::DUP);
        self.emit(OpCode::GET_GENERIC_FIELD);
        self.emit_u32(i as u32);
        let var_idx = self.add_var(binding.as_str());
        self.emit_store_loc(var_idx);
    }
}
```

Also update the earlier check for whether to extract fields:
- Old: `tag_cover.elem.as_str() != "_"`
- New: `!tag_cover.bindings.is_empty() && tag_cover.bindings.iter().any(|b| b.as_str() != "_")`

**Step 2: Verify compilation + tests**

Run: `cargo test -p auto-lang -- vm_file_tests`
Expected: all pass (single-binding patterns still work via bindings[0])

**Step 3: Commit**

```
feat(codegen): multi-field destructuring in is-match patterns
```

---

## Task 4: Codegen — Named arg ordering for enum construction

**Files:**
- Modify: `crates/auto-lang/src/vm/codegen.rs:4410-4480`

**Step 1: Sort named args by field index before compiling**

In the enum variant construction code (around line 4430-4460), when `call.args.args` contains `Arg::Pair(name, expr)`, we need to sort them by field index.

Find the section that compiles args for enum variant construction. It currently iterates `call.args.args` in source order. Change to:

```rust
// Sort named args by field index for correct field ordering
let mut sorted_args: Vec<(usize, &Arg)> = call.args.args.iter().enumerate().collect();
if !class_type.template.fields.is_empty() {
    // Try to reorder named args to match field definition order
    sorted_args.sort_by_key(|(_, arg)| {
        if let crate::ast::Arg::Pair(name, _) = arg {
            // Find field index by name
            class_type.template.fields.iter()
                .position(|f| f.name.as_str() == name.as_str())
                .unwrap_or(0)
        } else {
            // Positional args keep their original order
            0
        }
    });
}

for (_, arg) in sorted_args {
    match arg {
        crate::ast::Arg::Pos(expr) => { self.compile_expr(expr)?; }
        crate::ast::Arg::Pair(_key, expr) => { self.compile_expr(expr)?; }
        crate::ast::Arg::Name(name) => { self.compile_expr(&Expr::Ident(name.clone()))?; }
    }
}
```

**Note**: The `class_type.template.fields` is a `Vec<FieldDef>` where each `FieldDef` has a `name`. We need to access it — check if `template` is accessible from `class_type`. It may be `class_type.template` directly or through a method.

**Step 2: Verify compilation + tests**

Run: `cargo test -p auto-lang -- vm_file_tests`
Expected: all pass

**Step 3: Commit**

```
feat(codegen): sort named args by field index in enum construction
```

---

## Task 5: Test — multi-field destructuring

**Files:**
- Create: `crates/auto-lang/test/vm/09_functions/016_enum_multi_destruct/enum_multi_destruct.at`
- Create: `crates/auto-lang/test/vm/09_functions/016_enum_multi_destruct/enum_multi_destruct.expected.out`
- Modify: `crates/auto-lang/src/tests/vm_file_tests.rs`

**Step 1: Create test**

`enum_multi_destruct.at`:
```auto
// Test: multi-field enum variant destructuring
enum Shape {
    Point
    Rect { w int, h int }
    Circle { r int }
}

let s = Shape.Rect(10, 20)
is s {
    Shape.Rect(w, h) -> {
        print(w)
        print(h)
    }
    _ -> print("other")
}

// Single-field still works
let c = Shape.Circle(5)
is c {
    Shape.Circle(r) -> print(r)
    _ -> print("other")
}

// No-field variant
let p = Shape.Point
is p {
    Shape.Point -> print("point")
    _ -> print("other")
}
```

`enum_multi_destruct.expected.out`:
```
10
20
5
point
```

**Step 2: Register test**

In `vm_file_tests.rs` add:
```rust
#[test] fn test_09_functions_016_enum_multi_destruct() { test_vm("09_functions/016_enum_multi_destruct").unwrap(); }
```

**Step 3: Run test**

Run: `cargo test -p auto-lang -- test_09_functions_016_enum_multi_destruct`
Expected: PASS

**Step 4: Commit**

```
test(vm): add multi-field enum destructuring test
```

---

## Task 6: Test — named arg construction

**Files:**
- Create: `crates/auto-lang/test/vm/09_functions/017_enum_named_construct/enum_named_construct.at`
- Create: `crates/auto-lang/test/vm/09_functions/017_enum_named_construct/enum_named_construct.expected.out`
- Modify: `crates/auto-lang/src/tests/vm_file_tests.rs`

**Step 1: Create test**

`enum_named_construct.at`:
```auto
// Test: named arg enum construction (out-of-order)
enum ApiError {
    Http { code int, msg str }
    Timeout { millis int }
}

// Named args in field order
let e1 = ApiError.Http(code: 404, msg: "not found")
is e1 {
    ApiError.Http(c, m) -> {
        print(c)
    }
    _ -> print("other")
}

// Named args out of order (tests field-index sorting)
let e2 = ApiError.Http(msg: "timeout", code: 503)
is e2 {
    ApiError.Http(c, m) -> {
        print(c)
    }
    _ -> print("other")
}

// Positional args (existing behavior)
let e3 = ApiError.Timeout(5000)
is e3 {
    ApiError.Timeout(ms) -> print(ms)
    _ -> print("other")
}
```

`enum_named_construct.expected.out`:
```
404
503
5000
```

**Step 2: Register test**

```rust
#[test] fn test_09_functions_017_enum_named_construct() { test_vm("09_functions/017_enum_named_construct").unwrap(); }
```

**Step 3: Run all tests**

Run: `cargo test -p auto-lang -- vm_file_tests`
Expected: all pass

**Step 4: Commit**

```
test(vm): add named arg enum construction test
```

---

## Task 7: Test — wildcard + partial destructuring edge cases

**Files:**
- Create: `crates/auto-lang/test/vm/09_functions/018_enum_destruct_edge/enum_destruct_edge.at`
- Create: `crates/auto-lang/test/vm/09_functions/018_enum_destruct_edge/enum_destruct_edge.expected.out`
- Modify: `crates/auto-lang/src/tests/vm_file_tests.rs`

**Step 1: Create test**

`enum_destruct_edge.at`:
```auto
// Test: wildcard binding, partial destructuring
enum Color {
    RGB { r int, g int, b int }
    Gray { v int }
}

// Wildcard binding — don't bind any fields
let c1 = Color.RGB(100, 150, 200)
is c1 {
    Color.RGB -> print("rgb")
    _ -> print("other")
}

// Partial binding — only bind first field
let c2 = Color.Gray(128)
is c2 {
    Color.Gray(v) -> print(v)
    _ -> print("other")
}

// Multi-field with wildcard binding
let c3 = Color.RGB(255, 128, 0)
is c3 {
    Color.RGB(r, g, b) -> print(r + g + b)
    _ -> print("other")
}
```

`enum_destruct_edge.expected.out`:
```
rgb
128
383
```

**Step 2: Register test**

```rust
#[test] fn test_09_functions_018_enum_destruct_edge() { test_vm("09_functions/018_enum_destruct_edge").unwrap(); }
```

**Step 3: Run all tests**

Run: `cargo test -p auto-lang -- vm_file_tests`
Expected: all pass

**Step 4: Commit**

```
test(vm): add enum destructuring edge case tests
```

---

## Dependency Graph

```
Task 1 (AST) → Task 2 (Parser) → Task 3 (Codegen destructuring) → Task 5 (Test destructuring)
Task 4 (Codegen construction) → Task 6 (Test named args)
Tasks 5-7 can run in parallel after Tasks 3+4
```

**Critical path**: Task 1 → 2 → 3 → 5

## Risks

1. **Transpiler impact**: `TagCover.elem` change affects C/Rust transpilers. Need to update `.elem` → `.bindings[0]` references in `trans/c.rs` and `trans/rust.rs`.
2. **Field access**: `class_type.template.fields` access pattern needs verification — may need `class_type.template.as_ref().fields` or similar.
3. **Wildcard-only variant**: `Variant()` with empty parens → `bindings: vec![]` needs handling (no fields to extract).
