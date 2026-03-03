# Hyphenated Identifiers Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Allow hyphens in identifiers (e.g., `preview-card`) with the rule "subtraction must have spaces on both sides"

**Architecture:** Modify lexer to treat `a-b` as single identifier token. When `-` appears inside an identifier (between valid chars), it's part of the identifier. When `-` appears with whitespace, it's the subtraction operator.

**Tech Stack:** Rust lexer (crates/auto-lang/src/lexer.rs)

---

## Design Rules

| Syntax | Meaning | Reason |
|--------|---------|--------|
| `preview-card` | identifier | `-` between valid chars |
| `a - b` | subtraction | spaces on both sides |
| `a-b` | identifier | `-` between valid chars |
| `a -b` | `a` then unary minus `-b` | space before `-` |
| `a- b` | identifier `a-` then `b` | `-` followed by space (valid identifier `a-`) |

**Key insight:** `-` is part of identifier if the character AFTER `-` is a valid identifier character (letter, digit, underscore).

---

## Task 1: Modify Lexer Identifier Function

**Files:**
- Modify: `crates/auto-lang/src/lexer.rs:482-489`

**Step 1: Write the failing test**

Add test to `crates/auto-lang/src/lexer.rs`:

```rust
#[test]
fn test_hyphenated_identifiers() {
    let code = "preview-card button-primary my-component";
    let tokens = tokenize(code);
    assert_eq!(
        tokens,
        "<ident:preview-card><ident:button-primary><ident:my-component>"
    );
}

#[test]
fn test_subtraction_with_spaces() {
    let code = "a - b";
    let tokens = tokenize(code);
    assert_eq!(tokens, "<ident:a><-><ident:b>");
}

#[test]
fn test_hyphen_identifier_vs_subtraction() {
    // a-b is identifier
    let tokens1 = tokenize("a-b");
    assert_eq!(tokens1, "<ident:a-b>");

    // a - b is subtraction
    let tokens2 = tokenize("a - b");
    assert_eq!(tokens2, "<ident:a><-><ident:b>");

    // a -b is a then unary minus
    let tokens3 = tokenize("a -b");
    assert_eq!(tokens3, "<ident:a><-><-><ident:b>");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p auto-lang test_hyphenated`
Expected: FAIL - identifiers with hyphens not recognized

**Step 3: Modify identifier() function**

Update `crates/auto-lang/src/lexer.rs` around line 482:

```rust
pub fn identifier(&mut self) -> AutoResult<Token> {
    let mut text = String::new();
    let start_pos = self.pos;
    // First character must be alphabetic or underscore
    if let Some(&c) = self.chars.peek() {
        if !c.is_alphabetic() && c != '_' {
            let span = crate::error::span_from(start_pos, 1);
            return Err(LexerError::InvalidIdentifierStart {
                character: c.to_string(),
                span,
            }
            .into());
        } else {
            text.push(c);
            self.chars.next();
        }
    }
    while let Some(&c) = self.chars.peek() {
        if c.is_alphabetic() || c == '_' || c.is_digit(10) {
            text.push(c);
            self.chars.next();
        } else if c == '-' {
            // Hyphen: check if next char is valid identifier char
            // This allows "preview-card" but "a- " stops at "a"
            let mut lookahead = self.chars.clone();
            lookahead.next(); // skip the '-'
            if let Some(&next) = lookahead.peek() {
                if next.is_alphabetic() || next == '_' || next.is_digit(10) {
                    // Hyphen followed by valid identifier char, include in identifier
                    text.push(c);
                    self.chars.next();
                } else {
                    // Hyphen followed by non-identifier char, stop
                    break;
                }
            } else {
                // Hyphen at end of input, stop
                break;
            }
        } else {
            break;
        }
    }
    // ... rest of function unchanged
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p auto-lang test_hyphenated`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/auto-lang/src/lexer.rs
git commit -m "feat(lexer): add hyphen support in identifiers"
```

---

## Task 2: Update Token Kind Detection

**Files:**
- Modify: `crates/auto-lang/src/token.rs`

**Step 1: Verify token tests exist**

Check if there are existing token tests for identifiers.

Run: `cargo test -p auto-lang token`
Expected: Existing tests pass

**Step 2: No changes needed if identifier() handles it**

The `Token::ident()` function already takes any string, so hyphenated identifiers should work automatically.

**Step 3: Commit (if changes were made)**

```bash
git commit -m "feat(token): support hyphenated identifiers" --allow-empty
```

---

## Task 3: Update Parser for Hyphenated Tags

**Files:**
- Modify: `crates/auto-lang/src/parser.rs`

**Step 1: Test hyphenated tag names in view blocks**

Add test to verify hyphenated tags parse correctly:

```rust
#[test]
fn test_hyphenated_tag_names() {
    let code = r#"
view {
    preview-card (id: "test") {
        button-primary {}
    }
}
"#;
    // Should parse without error
    let result = parse(code);
    assert!(result.is_ok());
}
```

Run: `cargo test -p auto-lang test_hyphenated_tag`
Expected: PASS (should work automatically if lexer changes are correct)

**Step 2: Commit**

```bash
git commit -m "test(parser): add hyphenated tag name tests" --allow-empty
```

---

## Task 4: Update Vue Generator for Hyphenated Tags

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue.rs`

**Step 1: Test Vue generation with hyphenated tags**

The Vue generator should handle hyphenated tags correctly. Verify:

```rust
#[test]
fn test_hyphenated_tag_vue_generation() {
    // preview-card should work as a custom element
    // The generator may need to map it appropriately
}
```

**Step 2: Update tag mapping if needed**

Check `map_tag()` function in vue.rs to ensure hyphenated tags are handled:

```rust
// In map_tag(), ensure custom elements pass through
// "preview-card" -> "preview-card" (not modified)
```

**Step 3: Commit**

```bash
git commit -m "feat(vue): support hyphenated component names"
```

---

## Task 5: Update Component Gallery Example

**Files:**
- Modify: `examples/component-gallery/source/front/pages/*.at`

**Step 1: Rename previewcard to preview-card**

Update all page files to use hyphenated tag names:

```auto
// Before
previewcard (id: "button-basic") {
    button (text: "Button") {}
}

// After
preview-card (id: "button-basic") {
    button (text: "Button") {}
}
```

**Step 2: Regenerate Vue project**

Run: `./target/release/auto.exe vue examples/component-gallery/source/front --no-install`

**Step 3: Verify generated output**

Check that `preview-card` generates correctly in Vue output.

**Step 4: Commit**

```bash
git add examples/component-gallery/source/front/pages/*.at
git commit -m "refactor(examples): use hyphenated tag names"
```

---

## Task 6: Documentation

**Files:**
- Modify: `docs/router.md` or create `docs/syntax.md`

**Step 1: Document hyphenated identifier rules**

Add documentation:

```markdown
## Identifier Naming Rules

Identifiers can contain hyphens (`-`) when the hyphen is followed by a valid
identifier character (letter, digit, or underscore).

| Syntax | Meaning |
|--------|---------|
| `preview-card` | Single identifier |
| `a - b` | Subtraction (spaces required) |
| `a-b` | Single identifier |
| `a -b` | `a` then unary minus `-b` |

**Rule:** Subtraction must have spaces on both sides.
```

**Step 2: Commit**

```bash
git add docs/syntax.md
git commit -m "docs: add hyphenated identifier syntax documentation"
```

---

## Verification

**Final verification steps:**

1. Run all tests: `cargo test -p auto-lang`
2. Generate Vue project: `./target/release/auto.exe vue examples/component-gallery/source/front`
3. Start dev server and verify UI renders correctly

---

## Summary

This plan adds hyphen support in identifiers with the rule that subtraction must have spaces on both sides. The key change is in the lexer's `identifier()` function to continue consuming when `-` is followed by a valid identifier character.
