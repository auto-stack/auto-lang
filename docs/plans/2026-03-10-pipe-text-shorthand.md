# Pipe (`|`) Text Shorthand Syntax

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

## Migration Plan

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

## Implementation Changes

### Parser (`parser.rs`)

1. Replace `TokenKind::Gt` check with `TokenKind::Pipe` in `parse_view_node()`
2. After `|`, parse content until EOL or `{`
3. For element + `|`: validate no `{` follows (one-liner only)
4. Remove old `>` syntax handling

### Lexer (`lexer.rs`)

- `|` already tokenized as `TokenKind::Pipe` ✓ (no changes needed)

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
