# 025-notes-extended

A notes app that showcases AutoUI's **widget/block two-layer architecture** —
**frontend-only, in-memory data, no backend**.

This is the **frontend-richness fork** of [`015-notes`](../015-notes). Per
[Plan 338 §0](../../docs/plans/338-extend-015-notes-m1-benchmark.md), 015 stays
the front-back comms testbed (another agent is stabilizing its Rust comms);
025 carries the richer UI/UX work in isolation, and **merges back into 015
once 015's Rust comms are green**.

## What it demonstrates

- **Widget tier**: `App` composes primitives (`button`/`input`/`text`/`col`/`row`)
  + a presentational `NoteEditor` widget.
- **Block tier**: the sidebar is the intended consumer of the
  [`data-display/note-list`](../../blocks/data-display/note-list) block.
- **In-memory CRUD**: create / edit / delete / search / tag-filter, no backend.

## Build & run

```bash
auto build                              # .at -> gen/front/vue (Vue project)
cd gen/front/vue && pnpm install && pnpm dev
```

## Known deferrals (real AutoUI gaps — see SPEC.md)

These hit **unbuilt** AutoUI features and are intentionally NOT in 025:

- **Routing** — `"/p" -> use <widget>` passes no props; with no backend there's
  no shared state across pages. Needs the Rung-4 shared-store feature
  (Design 16). Single-view only.
- **Block event-wired adoption** — custom widgets are presentational (props
  in, no child→parent event binding), so list interaction stays inline in
  `App`. Wiring the `note-list` block as an interactive child awaits
  child-handler binding.
- **Markdown render** of the body — needs a JS/markdown stdlib bridge.

These three are Design-16 findings feeding the capability ladder.

## Merge back to 015 (task M-merge)

When 015's Rust comms pass: port 025's tag model, editor, search, and block
intent into 015; re-wire the in-memory CRUD to 015's real `#[api]` backend;
delete 025 (or keep as a pure-frontend showcase).
