# Plan 279: AURA Source-Style MCP Snapshot

## Status: ✅ Completed

## Summary

Rebuilt MCP snapshot output from AuraNode tree instead of View tree, producing output that closely resembles the original AURA source code with all variables evaluated.

## Problem

Plan 278's MCP snapshot built from `View<DynamicMessage>` tree lost original AURA information:
- Tag names changed (`center` → missing, `col` → `Column`)
- Tailwind classes converted to iced `Style` enum
- Event names changed (`onclick` → `press`, `oninput` → `type`)

## Solution

### Core Architecture Change

```
Old path (lossy):  AuraNode → AuraViewBuilder → View → SnapshotBuilder → UiNode
New path (faithful): AuraNode → AuraSnapshotBuilder → AURA text snapshot
                              ↑ state evaluation
                              ↑ ForLoop expansion
                              ↑ Conditional evaluation
```

### Key Component: `aura_snapshot_builder.rs`

New file that traverses AuraNode tree directly, preserving:
- Original tag names (`center`, `col`, `button`, `text`)
- Original style strings (`style: "w-128 p-8 bg-white"`)
- Original event attribute names (`onclick`, `oninput`)
- Stable AuraNodeIds (`#aura_0`, `#aura_3`)

### State Evaluation

- **StateRef props**: `value: .input` → `value: ""` (evaluated from state)
- **Interpolated text**: `` `Active: ${.active_count}` `` → `"Active: 1"` (regex-based fallback)
- **Style bindings**: `{ completed: todo.done }` → evaluates condition
- **Conditionals**: `if .show { ... }` → evaluates, shows only matching branch
- **ForLoop**: Expands from state array when possible

### Reverse Mapping

Parser transforms `center` → `col` + `"w-full h-full justify-center items-center"` at parse time.
Snapshot builder reverses this: detects the pattern and restores `center` tag.

## Files Modified

| File | Change |
|------|--------|
| `crates/auto-lang/src/ui/aura_snapshot_builder.rs` | **New** — AuraNode → AURA text snapshot builder |
| `crates/auto-lang/src/ui/dynamic.rs` | Added `view_template()` public accessor |
| `crates/auto-lang/src/ui/mcp_server.rs` | SharedState stores view_template; snapshot/inspect use new builder |
| `crates/auto-lang/src/ui/iced/renderer.rs` | MCP sync passes view_template from DynamicComponent |
| `crates/auto-lang/src/ui/mod.rs` | Export aura_snapshot_builder module |

## Example Output

**Input (013-todo AURA source):**
```
center {
    col {
        row {
            input { placeholder: "Add todo", value: .input, oninput: .InputChanged }
            button "Add" { onclick: .AddTodo, style: "px-4 py-2 bg-blue-500 text-white rounded" }
            style: "w-full max-w-xs gap-2 items-center"
        }
        text "__TODO_LIST__"
        text `Active: ${.active_count}`
        style: "w-128 p-8 bg-white"
    }
}
```

**MCP Snapshot (after adding 3 TODOs via autoui_action):**
```
AURA Snapshot v2
widget: "App"

state:
  active_count: 4 (int)
  input: "" (str)
  todo_count: 5 (int)

tree:
center #aura_0 {
  col #aura_1 {
    style: "w-128 p-8 bg-white"
    row #aura_2 {
      style: "w-full max-w-xs gap-2 items-center"
      input #aura_3 {
        placeholder: "Add todo"
        value: ""
        oninput: .InputChanged
      }
      button #aura_4 "Add" {
        style: "px-4 py-2 bg-blue-500 text-white rounded"
        onclick: .AddTodo
      }
    }
    text #aura_5 "__TODO_LIST__"
    text #aura_6 "Active: 4"
  }
}
```

## Bugs Fixed During Implementation

1. **Double-escaping**: `format_value` used `{:?}` (debug) for strings → `""` became `"\"\""`. Fixed with `format_eval_value` that doesn't add quotes.
2. **Text prop duplication**: `text` prop was extracted as label AND shown in props. Fixed by skipping `"text"` in `emit_props`.
3. **Interpolation not evaluated**: `bindings` Vec could be empty for f-string templates. Fixed with regex-based `${...}` extraction fallback.
4. **Center tag missing**: Parser converts `center` → `col` + centering style. Fixed with reverse mapping in snapshot builder.
5. **Empty braces on text nodes**: `text "foo" {}` with empty body. Fixed by checking `has_body` before emitting braces.
