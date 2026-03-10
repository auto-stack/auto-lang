# Pipe (`|`) Text Shorthand Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace `>` text syntax with `|` pipe shorthand for cleaner AURA view syntax.

**Architecture:** Modify `parse_view_node()` in parser.rs to handle `|` for both standalone text nodes and element+text one-liners. Parse unquoted text until EOL/`{`. Migrate all example files.

**Tech Stack:** Rust parser, AURA view DSL

---

## Task 1: Add Standalone Pipe Text Node Tests

**Files:**
- Modify: `crates/auto-lang/src/parser.rs` (test section at bottom)

**Step 1: Add test for standalone pipe with unquoted text**

Add to the test module in `parser.rs` (search for `#[test]` in parser.rs to find test section):

```rust
#[test]
fn test_pipe_standalone_unquoted() {
    let code = r#"widget Test { view { col { | Hello } } }"#;
    let result = run(code);
    assert!(result.is_ok());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p auto-lang test_pipe_standalone_unquoted`
Expected: FAIL (test not found or parsing fails)

**Step 3: Add test for standalone pipe with f-string**

```rust
#[test]
fn test_pipe_standalone_fstr() {
    let code = r#"widget Test { model { count int = 0 } view { col { | f"Count: ${.count}" } } }"#;
    let result = run(code);
    assert!(result.is_ok());
}
```

**Step 4: Add test for standalone pipe with quoted text**

```rust
#[test]
fn test_pipe_standalone_quoted() {
    let code = r#"widget Test { view { col { | "Hello World" } } }"#;
    let result = run(code);
    assert!(result.is_ok());
}
```

---

## Task 2: Add Element + Pipe Text Tests

**Files:**
- Modify: `crates/auto-lang/src/parser.rs` (test section)

**Step 1: Add test for element + pipe with unquoted text**

```rust
#[test]
fn test_element_pipe_unquoted() {
    let code = r#"widget Test { view { col { h1 | Input } } }"#;
    let result = run(code);
    assert!(result.is_ok());
}
```

**Step 2: Add test for element + pipe with multi-word text**

```rust
#[test]
fn test_element_pipe_multiword() {
    let code = r#"widget Test { view { col { h1 | Hello World } } }"#;
    let result = run(code);
    assert!(result.is_ok());
}
```

**Step 3: Add test for element + pipe with f-string**

```rust
#[test]
fn test_element_pipe_fstr() {
    let code = r#"widget Test { model { count int = 0 } view { col { h1 | f"Count: ${.count}" } } }"#;
    let result = run(code);
    assert!(result.is_ok());
}
```

**Step 4: Add test for element + pipe error when braces follow**

```rust
#[test]
fn test_element_pipe_with_braces_error() {
    // Should use button "-" { ... } syntax instead
    let code = r#"widget Test { view { col { h1 | Title { onclick: .Test } } } }"#;
    let result = run(code);
    assert!(result.is_err());
}
```

**Step 5: Run all new tests to verify they fail**

Run: `cargo test -p auto-lang test_pipe -- test_element_pipe`
Expected: All FAIL

---

## Task 3: Implement Standalone Pipe Text Parsing

**Files:**
- Modify: `crates/auto-lang/src/parser.rs:7336-7353`

**Step 1: Replace Gt with VBar for standalone text**

In `parse_view_node()`, find the block (around line 7336):

```rust
// Check for text with '>' prefix: > "text" or > f"text ${.state}"
if self.is_kind(TokenKind::Gt) {
    self.next();
    self.skip_empty_lines();

    // Check if it's an f-string
    if self.is_kind(TokenKind::FStrStart) {
        let fstr_expr = self.fstr()?;
        let (template, bindings) = self.extract_fstr_template_and_bindings(&fstr_expr);
        return Ok(ViewNode::Text(ViewText::Interpolated { template, bindings }));
    } else {
        // Plain string
        let text = self.cur.text.to_string();
        self.next();
        return Ok(ViewNode::text(text));
    }
}
```

Replace with:

```rust
// Check for text with '|' prefix: | text or | f"text ${.state}"
if self.is_kind(TokenKind::VBar) {
    self.next();
    self.skip_empty_lines();

    return self.parse_pipe_text_content();
}
```

**Step 2: Add helper method `parse_pipe_text_content`**

Add this new method to the Parser impl (search for `fn parse_view_node` and add after related view methods):

```rust
/// Parse text content after '|' operator
/// Supports: | text, | "quoted", | f"interpolated ${.state}"
fn parse_pipe_text_content(&mut self) -> AutoResult<ViewNode> {
    // Check if it's an f-string
    if self.is_kind(TokenKind::FStrStart) {
        let fstr_expr = self.fstr()?;
        let (template, bindings) = self.extract_fstr_template_and_bindings(&fstr_expr);
        return Ok(ViewNode::Text(ViewText::Interpolated { template, bindings }));
    }

    // Check for quoted string
    if self.is_kind(TokenKind::Str) {
        let text = self.cur.text.to_string();
        self.next();
        return Ok(ViewNode::text(text));
    }

    // Unquoted text: consume until EOL or '{'
    let mut text_parts = Vec::new();
    while !self.is_kind(TokenKind::LBrace)
        && !self.is_kind(TokenKind::RBrace)
        && !self.is_kind(TokenKind::EOF)
        && self.cur.text.as_str() != "\n"
    {
        text_parts.push(self.cur.text.to_string());
        self.next();
    }

    let text = text_parts.join(" ").trim().to_string();

    if text.is_empty() {
        let span = crate::error::pos_to_span(self.cur.pos);
        return Err(SyntaxError::Generic {
            message: "Expected text after '|'".to_string(),
            span,
        }.into());
    }

    Ok(ViewNode::text(text))
}
```

**Step 3: Run standalone tests to verify they pass**

Run: `cargo test -p auto-lang test_pipe_standalone`
Expected: All PASS

**Step 4: Commit**

```bash
git add crates/auto-lang/src/parser.rs
git commit -m "feat(parser): implement standalone pipe text syntax (| text)"
```

---

## Task 4: Implement Element + Pipe Text Parsing

**Files:**
- Modify: `crates/auto-lang/src/parser.rs:7355-7400` (after tag parsing)

**Step 1: Add pipe check after tag parsing**

In `parse_view_node()`, after the line `self.next();` following `let tag = self.cur.text.to_string();` (around line 7357), add:

```rust
// Check for pipe text shorthand: tag | text (one-liner only, no children)
if self.is_kind(TokenKind::VBar) {
    self.next();
    self.skip_empty_lines();

    // Parse text content
    let text_node = self.parse_pipe_text_content()?;

    // Extract text from the node
    let text_content = match text_node {
        ViewNode::Text(ViewText::Literal(s)) => s,
        ViewNode::Text(ViewText::Interpolated { template, bindings }) => {
            // For interpolated text, return element with text prop
            return Ok(ViewNode::Element {
                tag,
                props: vec![ViewProp {
                    name: "text".to_string(),
                    value: ViewPropValue::Expr(Expr::FStr(AutoStr::from(&template))),
                }],
                events: Vec::new(),
                children: Vec::new(),
            });
        }
        _ => unreachable!(),
    };

    // Error if braces follow (should use tag "text" { ... } syntax)
    if self.is_kind(TokenKind::LBrace) {
        let span = crate::error::pos_to_span(self.cur.pos);
        return Err(SyntaxError::Generic {
            message: format!(
                "Use `{} \"{}\" {{ ... }}` syntax for element with props/children",
                tag, text_content
            ),
            span,
        }.into());
    }

    // Return element with text property only
    return Ok(ViewNode::Element {
        tag,
        props: vec![ViewProp {
            name: "text".to_string(),
            value: ViewPropValue::Expr(Expr::Str(AutoStr::from(&text_content))),
        }],
        events: Vec::new(),
        children: Vec::new(),
    });
}
```

**Step 2: Run element + pipe tests**

Run: `cargo test -p auto-lang test_element_pipe`
Expected: All PASS

**Step 3: Commit**

```bash
git add crates/auto-lang/src/parser.rs
git commit -m "feat(parser): implement element + pipe text shorthand (tag | text)"
```

---

## Task 5: Remove Old Greater-Than Syntax

**Files:**
- Modify: `crates/auto-lang/src/parser.rs`

**Step 1: Verify no Gt usage remains in view parsing**

Search for any remaining `TokenKind::Gt` usage in `parse_view_node` and related methods. Remove if found.

**Step 2: Add regression test that Gt no longer works**

```rust
#[test]
fn test_gt_syntax_removed() {
    let code = r#"widget Test { view { col { > "Hello" } } }"#;
    let result = run(code);
    // Should fail or treat > as unknown
    assert!(result.is_err() || result.unwrap().contains(">"));
}
```

**Step 3: Run all parser tests**

Run: `cargo test -p auto-lang -- parser`
Expected: All PASS

**Step 4: Commit**

```bash
git add crates/auto-lang/src/parser.rs
git commit -m "refactor(parser): remove old greater-than text syntax"
```

---

## Task 6: Migrate examples/counter_full.at

**Files:**
- Modify: `examples/counter_full.at`

**Step 1: Update the text node syntax**

Change line 11 from:
```auto
> f"Count: ${.count}"
```
To:
```auto
| f"Count: ${.count}"
```

**Step 2: Verify the file parses correctly**

Run: `cargo run -p auto-lang -- run examples/counter_full.at`
Expected: No errors

**Step 3: Commit**

```bash
git add examples/counter_full.at
git commit -m "refactor(examples): migrate counter_full.at to pipe text syntax"
```

---

## Task 7: Migrate Component Gallery Pages

**Files:**
- Modify: `examples/component-gallery/source/front/pages/*.at` (all page files)
- Modify: `examples/component-gallery/source/front/components/*.at` (all component files)

**Step 1: Create migration pattern**

For each file, convert:
- `h1 (text: "Input") {}` → `h1 | Input`
- `h2 (text: "Installation") {}` → `h2 | Installation`
- `text (text: "...") {}` → `text | ...`
- `td (text: "type") {}` → `td | type`

**Step 2: Migrate pages/input.at**

Example changes for `input.at`:
```auto
// Before
h1 (text: "Input") {}
text (text: "Displays a form...") {}
h2 (text: "Installation") {}

// After
h1 | Input
text | Displays a form input field or a component that looks like an input field.
h2 | Installation
```

**Step 3: Migrate all other pages**

Apply the same pattern to all files in `pages/` and `components/` directories.

**Step 4: Verify all files parse**

Run: `for f in examples/component-gallery/source/front/**/*.at; do cargo run -p auto-lang -- run "$f" 2>&1 | head -1; done`
Expected: No errors

**Step 5: Commit**

```bash
git add examples/component-gallery/
git commit -m "refactor(gallery): migrate all pages/components to pipe text syntax"
```

---

## Task 8: Final Verification

**Step 1: Run all parser tests**

Run: `cargo test -p auto-lang -- parser`
Expected: All PASS

**Step 2: Run full test suite**

Run: `cargo test -p auto-lang`
Expected: All PASS

**Step 3: Run cargo clippy**

Run: `cargo clippy -p auto-lang -- -D warnings`
Expected: No warnings

**Step 4: Final commit (if any fixes needed)**

```bash
git add -A
git commit -m "fix: resolve test/clippy issues from pipe text syntax"
```

---

## Summary

| Task | Description | Status |
|------|-------------|--------|
| 1 | Add standalone pipe tests | ⬜ |
| 2 | Add element + pipe tests | ⬜ |
| 3 | Implement standalone pipe parsing | ⬜ |
| 4 | Implement element + pipe parsing | ⬜ |
| 5 | Remove old Gt syntax | ⬜ |
| 6 | Migrate counter_full.at | ⬜ |
| 7 | Migrate component gallery | ⬜ |
| 8 | Final verification | ⬜ |
