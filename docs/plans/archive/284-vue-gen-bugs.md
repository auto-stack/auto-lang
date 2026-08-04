# Plan 284: Fix Vue Generator Bugs for Dynamic Data Support

## Background

When testing `015-notes` with List/Object/for-loop/if-else features, several Vue generator bugs were discovered. The Auto source code is correct, but the generated Vue code has issues.

## Bugs Found & Fixed

### Bug 1: Handler - Array index access missing `.value` ✅ FIXED

**Auto source:**
```auto
let note = .notes[.active_id]
note.body = .edit_body
```

**Generated (wrong):**
```javascript
let note = notes[active_id];  // missing .value
note.body = edit_body.value;
```

**Fix:** Added `Expr::Index` handling in `ts_adapter.rs` `transpile_expr()` that delegates to `transpile_expr()` for both array and index, which correctly applies `.value` to state refs.

**File:** `crates/auto-lang/src/ui_gen/ts_adapter.rs`

### Bug 2: Template - Loop variable property access adds space ✅ FIXED

**Auto source:**
```auto
text note.title { ... }
```

**Generated (wrong):**
```html
{{  note .title }}
```

**Root cause:** `expr_to_vue_text()` for `StateRef` wrapped in `{{ name }}`, then `FieldAccess` used `trim_matches` to strip braces, but `{{ note }}` → trim → ` note ` (inner braces remain).

**Fix:** Added `expr_to_vue_text_raw()` that returns bare expression text without `{{ }}` wrapping. `expr_to_vue_text()` now calls the raw version and wraps the final result. This eliminates the nested wrapping/trimming issue.

**File:** `crates/auto-lang/src/ui_gen/vue.rs`

### Bug 3: Condition expression - Method calls stripped ✅ FIXED

**Auto source:**
```auto
if note.title.includes(.search) { ... }
```

**Generated (wrong):**
```html
<template v-if="note.title includes ( search )">
```

**Root cause:** `parse_condition_expr()` in parser.rs only handled one level of `ident.method`, so chained calls like `note.title.includes()` were broken into separate parts. The `.includes` dot was then removed by `convert_condition()`'s state-ref dot stripping.

**Fix:** Rewrote the `Ident` branch in `parse_condition_expr()` to use a `while` loop that supports unlimited chained `.method(args)` calls, building a single chain string like `note.title.includes(.search)`. The existing `convert_condition()` correctly handles this: only dots preceded by non-alphanumeric chars (like `(.search)`) are stripped.

**File:** `crates/auto-lang/src/parser.rs`

## Additional Fix: For-loop onclick parameter passing

To support clicking on a specific note in a `for` loop, the sidebar was changed to:
- `for i, note in .notes` (with index)
- `onclick: .SelectNote(i)` (pass loop index as parameter)
- `.SelectNote(idx int) -> { .active_id = idx; .editing = false }` (handler with parameter)

This leverages existing AURA parser support for `onclick: .Handler(arg)` and `on` block handler parameters.

## Related Files

- `crates/auto-lang/src/ui_gen/vue.rs` — Vue generator (Bug 2 fix)
- `crates/auto-lang/src/ui_gen/ts_adapter.rs` — TypeScript handler transpiler (Bug 1 fix)
- `crates/auto-lang/src/parser.rs` — Parser condition expression (Bug 3 fix)
- `examples/ui/015-notes/` — Test case exercising all three fixes

## Status

✅ All bugs fixed. Verified with full CRUD testing of the notes app.
