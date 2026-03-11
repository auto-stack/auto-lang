# Plan 116: Pipe (`|`) Text Shorthand Syntax

**Date:** 2026-03-10
**Status:** Approved

## Summary

Replace the `>` text syntax with `|` (pipe) for a cleaner, unambiguous text shorthand in AURA views.

## Motivation

The current `>` syntax is visually ambiguous with comparison operators (`h1 > Input` could be read as "h1 greater than Input"). We need a syntax that is:
1. Technically unambiguous in the parser
2. Visually distinct from comparison operators
3. Simple and concise for common cases

## Design

### Syntax Specification

**Standalone text nodes:**
```auto
| Input                  → Text node "Input"
| Hello World            → Text node "Hello World"
| f"Count: ${.count}"    → Text node with interpolation
```

**Element + text (one-liner, NO children/props):**
```auto
h1 | Input               → h1 with text="Input", no children
h1 | Hello World         → h1 with text="Hello World", no children
h1 | f"Count: ${.count}" → h1 with interpolated text, no children
```

**Element with props/children (use existing syntax):**
```auto
button "-" { onclick: .Dec }     → button with text AND event
col { h1 | Title }               → col with child h1
```

### Key Principle

`|` behaves like a virtual `text` tag:
- `| Hello` ≡ `text | Hello` (both produce text node "Hello")

`|` is a **one-liner shorthand** only — no children, no props, no braces.

### Parsing Rules

In view context (`view { }` blocks), the `|` operator:

1. **Standalone**: `| <content>` → creates `ViewNode::Text`
2. **After tag with nothing else**: `tag | <content>` → element with text only

**Content parsing (until EOL or `{`):**
- `f"..."` → interpolated f-string
- `"..."` → quoted literal
- Otherwise → unquoted literal (consume to EOL/`{`)

**Validation:**
- If `{` follows `tag | <text>` → error (use `tag "text" { ... }` instead)
- Empty text after `|` → error

### Edge Cases

| Case | Behavior |
|------|----------|
| `| ` (empty) | Error - text required |
| `| ` (whitespace only) | Error - text required |
| `h1 | ` (empty after pipe) | Error - text required |
| `h1 |` at end of line | Error - text required |

## Migration

### Removed Syntax

```auto
// OLD (no longer supported)
> "text"
> f"text ${.state}"
```

### Migration Rules

| Before | After |
|--------|-------|
| `> "Hello"` | `\| Hello` |
| `> f"Count: ${.count}"` | `\| f"Count: ${.count}"` |
| `h1 (text: "Input") {}` | `h1 \| Input` |
| `h2 (text: "Title") {}` | `h2 \| Title` |
| `p (text: "Description") {}` | `p \| Description` |

### Migration Scope

- All `*.at` files with view blocks
- `examples/counter_full.at` - uses `> f"Count: ${.count}"`
- `examples/component-gallery/**/*.at` - all pages and widgets with simple text-only nodes

### Detection Rule

Any element with:
- Only `text` property (no other props)
- No events
- Empty body `{}`

→ Convert to `tag | text_content`

---

## Implementation

### Task 1: Add Tests for Pipe Syntax

**Files:** `crates/auto-lang/src/parser.rs` (test section)

Add tests for:
- Standalone pipe with unquoted text: `| Hello`
- Standalone pipe with f-string: `| f"Count: ${.count}"`
- Standalone pipe with quoted text: `| "Hello World"`
- Element + pipe unquoted: `h1 | Input`
- Element + pipe multi-word: `h1 | Hello World`
- Element + pipe f-string: `h1 | f"Count: ${.count}"`
- Element + pipe error when braces follow: `h1 | Title { onclick: .Test }`

### Task 2: Implement Standalone Pipe Text Parsing

**Files:** `crates/auto-lang/src/parser.rs`

In `parse_view_node()`, replace `TokenKind::Gt` check with `TokenKind::VBar`:

```rust
// Check for text with '|' prefix: | text or | f"text ${.state}"
if self.is_kind(TokenKind::VBar) {
    self.next();
    self.skip_empty_lines();
    return self.parse_pipe_text_content();
}
```

Add helper method `parse_pipe_text_content()`:
- Check for f-string (`FStrStart`) → return interpolated text
- Check for quoted string (`Str`) → return literal text
- Otherwise → consume unquoted text until EOL/`{`/`EOF`
- Error if empty text

### Task 3: Implement Element + Pipe Text Parsing

**Files:** `crates/auto-lang/src/parser.rs`

In `parse_view_node()`, after parsing tag name, check for `VBar`:

```rust
// Check for pipe text shorthand: tag | text (one-liner only, no children)
if self.is_kind(TokenKind::VBar) {
    self.next();
    self.skip_empty_lines();

    let text_node = self.parse_pipe_text_content()?;

    // Extract text and create element with text prop only
    // Error if LBrace follows (should use tag "text" { ... } syntax)

    return Ok(ViewNode::Element { tag, props: vec![text_prop], events: vec![], children: vec![] });
}
```

### Task 4: Remove Old Greater-Than Syntax

**Files:** `crates/auto-lang/src/parser.rs`

- Verify no `TokenKind::Gt` usage remains in view parsing
- Add regression test that `>` no longer works for text nodes

### Task 5: Migrate Example Files

**Files:**
- `examples/counter_full.at`
- `examples/component-gallery/source/front/pages/*.at`
- `examples/component-gallery/source/front/components/*.at`

Apply migration pattern:
- `h1 (text: "Input") {}` → `h1 | Input`
- `> f"Count: ${.count}"` → `| f"Count: ${.count}"`

### Task 6: Final Verification

- Run all parser tests: `cargo test -p auto-lang -- parser`
- Run full test suite: `cargo test -p auto-lang`
- Run clippy: `cargo clippy -p auto-lang -- -D warnings`

---

## Examples

### Before

```auto
view {
    col {
        > f"Count: ${.count}"
        h1 (text: "Input") {}
        button (text: "-") { onclick: .Dec }
    }
}
```

### After

```auto
view {
    col {
        | f"Count: ${.count}"
        h1 | Input
        button "-" { onclick: .Dec }
    }
}
```
