# Plan 116: Pipe (`|`) Text Shorthand Syntax

**Date:** 2026-03-10
**Status:** ✅ Complete

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

### Completed Tasks

| Task | Description | Status |
|------|-------------|--------|
| 1 | Add standalone pipe tests | ✅ |
| 2 | Add element + pipe tests | ✅ |
| 3 | Implement standalone pipe parsing | ✅ |
| 4 | Implement element + pipe parsing | ✅ |
| 5 | Remove old Gt syntax | ✅ |
| 6 | Migrate counter_full.at | ✅ |
| 7 | Migrate component gallery | ✅ |
| 8 | Final verification | ✅ |

### Commits

```
9e42779 docs: merge pipe text shorthand plans into Plan 116
dded9ce refactor(parser): replace pipe text syntax with string literal shorthand
cbc5479 fix(gallery): quote special chars in pipe text
d84228f refactor(gallery): migrate all pages/components to pipe text syntax
6460383 refactor(examples): migrate counter_full.at to pipe text syntax
80215fa refactor(parser): add regression tests for old greater-than text syntax
646906c feat(parser): implement element + pipe text shorthand (tag | text)
f031f89 fix(parser): use TokenKind::Newline instead of string comparison
91982ca feat(parser): implement standalone pipe text syntax (| text)
7d296da test(parser): add element + pipe text shorthand tests (Task 2)
```

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
