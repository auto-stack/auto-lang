# Plan 165: Struct Destructuring in `is` Match Arms

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Support Rust-style `{field1, field2}` struct destructuring in `is` statement match arms.

**Architecture:** Add `StructCover` type to `ast/cover.rs`, new `Expr::StructPattern` variant, parse `{...}` after type/variant names in `is_branch_cond_expr()`, emit `Type::Variant { fields }` in transpiler.

**Tech Stack:** Rust, a2r transpiler, existing is/Cover infrastructure

---

### Task 1: AST — Add `StructCover` and `FieldBinding` types

**Files:**
- Modify: `crates/auto-lang/src/ast/cover.rs` (add new types after line 63)
- Modify: `crates/auto-lang/src/ast.rs` (add `StructPattern` variant at ~line 336)

**Step 1: Add types to cover.rs**

After the existing `ResultUncover` struct (line 63), add:

```rust
// Plan 165: Struct destructuring pattern for is statement
// is x { Point { x, y } => ... } or is x { Message.User { content } => ... }

/// A single field binding in a struct destructuring pattern
#[derive(Debug, Clone)]
pub struct FieldBinding {
    pub field: AutoStr,       // Field name
    pub binding: AutoStr,     // Binding name (same as field when using shorthand)
}

/// Struct destructuring pattern: Type { field1, field2: alias }
#[derive(Debug, Clone)]
pub struct StructCover {
    pub type_name: AutoStr,           // "Point" or "Message"
    pub variant: Option<AutoStr>,     // Some("User") for enum variant, None for plain struct
    pub fields: Vec<FieldBinding>,    // field bindings
}
```

**Step 2: Add Display impls for new types**

```rust
impl fmt::Display for FieldBinding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.field == self.binding {
            write!(f, "{}", self.field)
        } else {
            write!(f, "{}: {}", self.field, self.binding)
        }
    }
}

impl fmt::Display for StructCover {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.variant {
            Some(v) => write!(f, "(struct-cover {}.{} {{", self.type_name, v)?,
            None => write!(f, "(struct-cover {} {{", self.type_name)?,
        }
        for (i, fb) in self.fields.iter().enumerate() {
            if i > 0 { write!(f, " ")?; }
            write!(f, "{}", fb)?;
        }
        write!(f, "}})")
    }
}
```

**Step 3: Add ToNode impls**

```rust
impl ToNode for FieldBinding {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("field-binding");
        node.set_prop("field", Value::Str(self.field.clone()));
        node.set_prop("binding", Value::Str(self.binding.clone()));
        node
    }
}

impl ToNode for StructCover {
    fn to_node(&self) -> AutoNode {
        let mut node = AutoNode::new("struct-cover");
        node.set_prop("type_name", Value::Str(self.type_name.clone()));
        if let Some(ref v) = self.variant {
            node.set_prop("variant", Value::Str(v.clone()));
        }
        for fb in &self.fields {
            node.add_kid(fb.to_node());
        }
        node
    }
}
```

**Step 4: Add `Expr::StructPattern` variant**

In `crates/auto-lang/src/ast.rs`, after line 336 (`ResultUncover`), add:

```rust
    // Plan 165: Struct destructuring pattern for is statement
    StructPattern(crate::ast::cover::StructCover),  // Point { x, y } in is branch
```

**Step 5: Add Display match arm**

In the `Display` impl for `Expr` (~line 449), after `ResultUncover`:

```rust
            // Plan 165: Struct destructuring pattern
            Expr::StructPattern(sc) => write!(f, "{}", sc),
```

**Step 6: Add ToNode match arm**

In the `ToNode` impl for `Expr` (~line 890), after `ResultUncover`:

```rust
            // Plan 165: Struct destructuring pattern
            Expr::StructPattern(sc) => sc.to_node(),
```

**Step 7: Build to verify**

Run: `cargo build -p auto-lang`
Expected: Compiles (parser/transpiler not yet using it, so will fail at match exhaustiveness — see Step 8).

**Step 8: Add stubs in transpiler and evaluator**

In `crates/auto-lang/src/trans/rust.rs`, add a match arm in `expr()` for the new variant (after `ResultPattern`/`ResultUncover` handling, around line 704):

```rust
            // Plan 165: Struct destructuring pattern (handled in is_stmt, stub here)
            Expr::StructPattern(_) => {
                write!(out, "/* struct-pattern */").map_err(Into::into)
            }
```

Similarly, if there's an evaluator that matches on Expr variants, add a stub there too. Search for `Expr::OptionPattern` in non-trans files and add a matching stub for `Expr::StructPattern`.

Run: `cargo build -p auto-lang`
Expected: Compiles with no errors.

**Step 9: Commit**

```bash
git add crates/auto-lang/src/ast/cover.rs crates/auto-lang/src/ast.rs crates/auto-lang/src/trans/rust.rs
git commit -m "feat: add StructCover/FieldBinding AST types for struct destructuring (Plan 165)"
```

---

### Task 2: Parser — Parse `{field1, field2}` in is branches

**Files:**
- Modify: `crates/auto-lang/src/parser.rs` (~lines 2763-2794, `is_branch_cond_expr()`)

**Step 1: Add struct destructuring parser**

In `parser.rs`, add a new helper method after `is_branch_cond_expr()`:

```rust
    /// Plan 165: Parse struct destructuring pattern: Name { field1, field2: alias }
    /// Called when we see `{` after a type name in an is branch.
    fn parse_struct_cover(&mut self, type_name: Name, variant: Option<Name>) -> AutoResult<Expr> {
        self.expect(TokenKind::LBrace)?;
        self.skip_empty_lines();

        let mut fields = Vec::new();
        while !self.is_kind(TokenKind::RBrace) && !self.is_kind(TokenKind::EOF) {
            let field = self.parse_name()?;
            let binding = if self.is_kind(TokenKind::Asn) {
                // field: alias
                self.next(); // skip ':'
                self.parse_name()?
            } else {
                // shorthand: field name = binding name
                field.clone()
            };
            fields.push(crate::ast::cover::FieldBinding {
                field,
                binding,
            });

            // Optional comma separator
            if self.is_kind(TokenKind::Comma) {
                self.next();
            }
            self.skip_empty_lines();
        }
        self.expect(TokenKind::RBrace)?;

        Ok(Expr::StructPattern(crate::ast::cover::StructCover {
            type_name,
            variant,
            fields,
        }))
    }
```

**Step 2: Wire into `is_branch_cond_expr()`**

In `is_branch_cond_expr()` (line 2763), the current flow is:
1. Check Option/Result keywords
2. Parse `lhs` expression
3. Call `expr_pratt_with_left(lhs, 0)` which handles `.Variant` via `tag_cover()`

We need to intercept `{` **after** parsing the initial name but **before** Pratt parsing handles it. Add a check after line 2784 where `lhs` is parsed:

Change the section starting at line 2784 from:

```rust
        // Parse the left-hand side expression (identifier or tag)
        let lhs = if self.is_kind(TokenKind::Ident) {
            self.lhs_expr()?
        } else {
            self.atom()?
        };

        // Continue parsing to handle member access (e.g., Msg.Inc)
        // This allows expressions like "Msg.Inc" in is branches
        self.expr_pratt_with_left(lhs, 0)
```

To:

```rust
        // Parse the left-hand side expression (identifier or tag)
        let lhs = if self.is_kind(TokenKind::Ident) {
            self.lhs_expr()?
        } else {
            self.atom()?
        };

        // Plan 165: Check for struct destructuring { fields } after type name
        // e.g., is x { Point { x, y } => ... }
        if let Expr::Ident(name) = &lhs {
            if self.is_kind(TokenKind::LBrace) {
                let name = name.clone();
                return self.parse_struct_cover(name, None);
            }
        }

        // Continue parsing to handle member access (e.g., Msg.Inc)
        // This allows expressions like "Msg.Inc" in is branches
        let result = self.expr_pratt_with_left(lhs, 0)?;

        // Plan 165: Check for struct destructuring after variant: Msg.User { content }
        // After Pratt parses "Msg.User" into a TagCover, check for { ... }
        if let Expr::Cover(Cover::Tag(tag)) = &result {
            if self.is_kind(TokenKind::LBrace) {
                let type_name = tag.kind.clone();
                let variant = Some(tag.tag.clone());
                return self.parse_struct_cover(type_name, variant);
            }
        }

        Ok(result)
```

**Step 3: Build to verify**

Run: `cargo build -p auto-lang`
Expected: Compiles successfully.

**Step 4: Commit**

```bash
git add crates/auto-lang/src/parser.rs
git commit -m "feat: parse struct destructuring in is branches (Plan 165)"
```

---

### Task 3: Transpiler — Emit struct destructuring patterns

**Files:**
- Modify: `crates/auto-lang/src/trans/rust.rs` (replace the stub added in Task 1)

**Step 1: Replace the stub with real implementation**

Find the `Expr::StructPattern(_) =>` stub added in Task 1 Step 8 and replace with:

```rust
            // Plan 165: Struct destructuring pattern
            Expr::StructPattern(sc) => {
                match &sc.variant {
                    Some(variant) => {
                        write!(out, "{}::{}", sc.type_name, variant)?;
                    }
                    None => {
                        write!(out, "{}", sc.type_name)?;
                    }
                }
                write!(out, " {{ ")?;
                for (i, fb) in sc.fields.iter().enumerate() {
                    if fb.field == fb.binding {
                        write!(out, "{}", fb.field)?;
                    } else {
                        write!(out, "{}: {}", fb.field, fb.binding)?;
                    }
                    if i < sc.fields.len() - 1 {
                        write!(out, ", ")?;
                    }
                }
                write!(out, " }}").map_err(Into::into)
            }
```

**Step 2: Build to verify**

Run: `cargo build -p auto-lang`
Expected: Compiles successfully.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/trans/rust.rs
git commit -m "feat: a2r emit struct destructuring in match arms (Plan 165)"
```

---

### Task 4: Test — Create a2r test case `154_struct_destructure`

**Files:**
- Create: `crates/auto-lang/test/a2r/154_struct_destructure/struct_destructure.at`
- Create: `crates/auto-lang/test/a2r/154_struct_destructure/struct_destructure.expected.rs`
- Modify: `crates/auto-lang/src/tests/a2r_tests.rs` (add test function)

**Step 1: Create test directory and input**

```bash
mkdir -p crates/auto-lang/test/a2r/154_struct_destructure
```

Create `crates/auto-lang/test/a2r/154_struct_destructure/struct_destructure.at`:

```auto
// Plan 165: Struct destructuring in is match arms

type Point {
    x int
    y int
}

enum Shape {
    Circle int
    Rect Point
    None
}

fn main() {
    let p = Point(x: 3, y: 4)

    // Plain struct destructuring
    is p {
        Point { x, y } => print(x)
    }

    // Enum variant with struct destructuring
    let s = Shape.Rect(p)
    is s {
        Shape.Circle(r) => print(r)
        Shape.Rect(pt) => {
            is pt {
                Point { x, y } => print(x)
            }
        }
        Shape.None => print("none")
    }
}
```

**Step 2: Add test function**

In `crates/auto-lang/src/tests/a2r_tests.rs`, add:

```rust
// Plan 165: Struct destructuring in is match arms
#[test]
fn test_154_struct_destructure() {
    test_a2r("154_struct_destructure").unwrap();
}
```

**Step 3: Run test to generate .wrong.rs**

Run: `cargo test -p auto-lang test_154_struct_destructure 2>&1 || true`

This creates `struct_destructure.wrong.rs`.

**Step 4: Create expected output**

Copy `.wrong.rs` to `.expected.rs`:

```bash
cp crates/auto-lang/test/a2r/154_struct_destructure/struct_destructure.wrong.rs \
   crates/auto-lang/test/a2r/154_struct_destructure/struct_destructure.expected.rs
```

Inspect the expected output to verify it looks correct. Expected structure:
- `struct Point { x: i32, y: i32 }`
- `match p { Point { x, y } => ... }`
- `Shape::Rect(pt) => { match pt { Point { x, y } => ... } }`

**Step 5: Run test to verify**

Run: `cargo test -p auto-lang test_154_struct_destructure`
Expected: PASS

**Step 6: Run all a2r tests for regressions**

Run: `cargo test -p auto-lang --lib -- trans`
Expected: All tests pass.

**Step 7: Commit**

```bash
git add crates/auto-lang/test/a2r/154_struct_destructure/ crates/auto-lang/src/tests/a2r_tests.rs
git commit -m "test: add a2r struct destructuring test (Plan 165)"
```

---

### Task 5: Final verification

**Step 1: Run full test suite**

Run: `cargo test -p auto-lang --lib`
Expected: All tests pass, no regressions.

**Step 2: Update Plan 159 status**

In `docs/plans/159-autocode-coding-agent.md`, update line for 6B-4.5:
- Change: `| P1（Plan 165） |` → `| ✅ **已完成** (Plan 165, test 154) | ~~P1~~`

**Step 3: Final commit**

```bash
git add docs/plans/159-autocode-coding-agent.md
git commit -m "docs: update plan 159 — mark 6B-4.5 done via Plan 165"
```
