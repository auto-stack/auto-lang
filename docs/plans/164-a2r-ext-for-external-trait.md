# Plan 164: a2r — `ext Type for Trait` (External Trait Implementation)

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Allow AutoLang code to implement external Rust traits (Display, Debug, Clone, Default, etc.) via `ext Type for Trait { }` syntax.

**Architecture:** Add `trait_name: Option<Name>` to the `Ext` AST node. Parser checks for `for` keyword after the target type name. Rust transpiler emits `impl Trait for Type` when present. Other transpilers ignore the field.

**Tech Stack:** Rust, a2r transpiler, existing Ext/ast infrastructure

---

### Task 1: AST — Add `trait_name` field to `Ext`

**Files:**
- Modify: `crates/auto-lang/src/ast/ext.rs`

**Step 1: Add the field**

In `Ext` struct (line 48), add after `target`:

```rust
pub struct Ext {
    /// Type being extended (e.g., "str", "Point")
    pub target: Name,

    /// External trait name (None = inherent impl, Some = trait impl)
    /// e.g., `ext Point for Display` → trait_name = Some("Display")
    pub trait_name: Option<Name>,

    pub generic_params: Vec<GenericParam>,
    // ... rest unchanged
}
```

**Step 2: Update all constructors**

In `Ext::new()` (line 71), add `trait_name: None,` after `target,`.

In `Ext::with_fields()` (line 90), add `trait_name: None,` after `target,`.

In `Ext::with_generic_params()` (line 106), add `trait_name: None,` after `target,`.

**Step 3: Update `PartialEq` impl**

In `PartialEq` (line 117), add comparison:

```rust
fn eq(&self, other: &Self) -> bool {
    self.target == other.target
        && self.trait_name == other.trait_name
        && self.fields.len() == other.fields.len()
        && self.methods == other.methods
        && self.module_path == other.module_path
        && self.is_same_module == other.is_same_module
}
```

**Step 4: Update `Display` impl**

In `fmt` (line 128), add trait_name after target:

```rust
fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "(ext (target {}", self.target)?;

    if let Some(ref trait_name) = self.trait_name {
        write!(f, " (for {})", trait_name)?;
    }

    // ... rest unchanged (generic params, fields, methods)
}
```

**Step 5: Update `ToNode` impl**

In `to_node()` (line 190), add trait_name property:

```rust
fn to_node(&self) -> AutoNode {
    let mut node = AutoNode::new("ext");
    node.set_prop("target", auto_val::Value::Str(self.target.clone()));

    if let Some(ref trait_name) = self.trait_name {
        node.set_prop("trait_name", auto_val::Value::Str(trait_name.clone()));
    }

    // ... rest unchanged
}
```

**Step 6: Update `AtomWriter` impl**

In `write_atom()` (line 162), add trait_name after target:

```rust
fn write_atom(&self, f: &mut impl stdio::Write) -> auto_val::AutoResult<()> {
    write!(f, "ext({}, ", self.target)?;

    if let Some(ref trait_name) = self.trait_name {
        write!(f, "for:{}, ", trait_name)?;
    }

    // ... rest unchanged
}
```

**Step 7: Update existing unit tests**

In `tests` module (line 227), update assertions to account for `trait_name: None`:

No changes needed — existing tests use constructors that default `trait_name` to `None`.

**Step 8: Add new unit test for trait_name**

```rust
#[test]
fn test_ext_with_trait_name() {
    let mut ext = Ext::new("Point".into(), vec![]);
    ext.trait_name = Some("Display".into());
    assert_eq!(ext.target, "Point");
    assert_eq!(ext.trait_name, Some("Display".into()));
}
```

**Step 9: Build to verify**

Run: `cargo build -p auto-lang`
Expected: Compiles with no errors (parser/transpiler not yet using the field).

**Step 10: Commit**

```bash
git add crates/auto-lang/src/ast/ext.rs
git commit -m "feat: add trait_name field to Ext AST node (Plan 164)"
```

---

### Task 2: Parser — Parse `for TraitName` in ext blocks

**Files:**
- Modify: `crates/auto-lang/src/parser.rs:3254-3258`

**Step 1: Parse `for` keyword**

In `parse_ext_stmt()`, after the generic instance skip block (line 3254), and **before** the `expect(TokenKind::LBrace)` (line 3257), insert the `for` parsing:

```rust
        // Plan 164: Parse optional "for TraitName" for external trait implementation
        // e.g., ext Point for Display { ... }
        let mut trait_name: Option<Name> = None;
        if self.is_kind(TokenKind::For) {
            self.next(); // skip 'for'
            trait_name = Some(self.parse_name()?);

            // Skip generic args on trait name, e.g., ext MyType for From<String>
            if self.is_kind(TokenKind::Lt) {
                self.next(); // skip '<'
                if self.next_token_is_type() {
                    let _ = self.parse_type()?;
                }
                while self.is_kind(TokenKind::Comma) {
                    self.next();
                    if self.next_token_is_type() {
                        let _ = self.parse_type()?;
                    }
                }
                self.expect(TokenKind::Gt)?;
            }
        }

        // Expect opening brace
        self.expect(TokenKind::LBrace)?;
```

**Step 2: Wire into Ext construction**

At line 3427 where `Ext` is constructed, add `trait_name`:

```rust
        let ext = Ext {
            target,
            trait_name,
            generic_params,
            fields,
            methods,
            module_path,
            is_same_module,
        };
```

**Step 3: Build to verify**

Run: `cargo build -p auto-lang`
Expected: Compiles successfully.

**Step 4: Commit**

```bash
git add crates/auto-lang/src/parser.rs
git commit -m "feat: parse 'ext Type for Trait' syntax (Plan 164)"
```

---

### Task 3: Transpiler — Emit `impl Trait for Type`

**Files:**
- Modify: `crates/auto-lang/src/trans/rust.rs:3391-3394`

**Step 1: Update `ext_decl()` header**

In `ext_decl()` (line 3392), replace the existing `write!(sink.body, "impl {}", ext.target)?;` with:

```rust
    fn ext_decl(&mut self, ext: &Ext, sink: &mut Sink) -> AutoResult<()> {
        // Plan 164: Support "ext Type for Trait" → impl Trait for Type
        match &ext.trait_name {
            Some(trait_name) => {
                write!(sink.body, "impl {} for {}", trait_name, ext.target)?;
            }
            None => {
                write!(sink.body, "impl {}", ext.target)?;
            }
        }

        // Add generic parameters if present (unchanged)
        // ...
```

**Step 2: Build to verify**

Run: `cargo build -p auto-lang`
Expected: Compiles successfully.

**Step 3: Commit**

```bash
git add crates/auto-lang/src/trans/rust.rs
git commit -m "feat: a2r emit impl Trait for Type (Plan 164)"
```

---

### Task 4: Test — Create a2r test case `153_ext_for`

**Files:**
- Create: `crates/auto-lang/test/a2r/153_ext_for/ext_for.at`
- Create: `crates/auto-lang/test/a2r/153_ext_for/ext_for.expected.rs`
- Modify: `crates/auto-lang/src/trans/rust.rs` (add test function)

**Step 1: Create test input**

Create `crates/auto-lang/test/a2r/153_ext_for/ext_for.at`:

```auto
type Point {
    x int
    y int
}

ext Point for Default {
    fn default() Point {
        return Point(x: 0, y: 0)
    }
}

ext Point {
    fn origin() Point {
        return Point(x: 0, y: 0)
    }
}

fn main() {
    let p = Point.default()
    let o = Point.origin()
}
```

**Step 2: Run test to generate initial output**

Run: `cargo test -p auto-lang test_153_ext_for 2>&1 || true`

This will create `ext_for.wrong.rs`. Inspect the output.

**Step 3: Review and create expected output**

Create `crates/auto-lang/test/a2r/153_ext_for/ext_for.expected.rs` based on the `.wrong.rs` output (fixing any issues).

Expected structure:
- `struct Point { x: i32, y: i32 }`
- `impl Default for Point { fn default() -> Point { ... } }`
- `impl Point { fn origin() -> Point { ... } }`
- `fn main() { ... }`

**Step 4: Add test function to `trans/rust.rs`**

In the `#[cfg(test)]` module at the end of `trans/rust.rs`, add:

```rust
#[test]
fn test_153_ext_for() {
    test_a2r("153_ext_for").unwrap();
}
```

**Step 5: Run test to verify**

Run: `cargo test -p auto-lang test_153_ext_for`
Expected: PASS

**Step 6: Run all a2r tests to verify no regressions**

Run: `cargo test -p auto-lang -- trans`
Expected: All tests pass.

**Step 7: Commit**

```bash
git add crates/auto-lang/test/a2r/153_ext_for/ crates/auto-lang/src/trans/rust.rs
git commit -m "test: add a2r ext-for external trait test (Plan 164)"
```

---

### Task 5: Final verification

**Step 1: Run full test suite**

Run: `cargo test -p auto-lang`
Expected: All tests pass, no regressions.

**Step 2: Update Plan 159 status**

In `docs/plans/159-autocode-coding-agent.md`, update Phase 6B-4.4 status:
- `6B-4.4 | impl Trait for Type | ❌ 仅支持 spec | P0` → `6B-4.4 | impl Trait for Type | ✅ Plan 164 | ~~P0~~`

**Step 3: Final commit**

```bash
git add docs/plans/159-autocode-coding-agent.md
git commit -m "docs: update plan 159 — mark 6B-4.4 done via Plan 164"
```
