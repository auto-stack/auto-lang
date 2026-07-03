# 025-notes-extended

A notes app that showcases AutoUI's **widget/block two-layer architecture** plus
**SharedStore (Design 18) + multi-page routing (Plan 105)**.

**Frontend-only, in-memory data, no backend.** This is the frontend-richness
fork of [`015-notes`](../015-notes). Per
[Plan 338 §0](../../docs/plans/338-extend-015-notes-m1-benchmark.md), it merges
back into 015 once 015's Rust comms are green.

## What it demonstrates

- **SharedStore**: `store NotesStore` (Design 18) — module-level composable
  singleton holding notes + CRUD actions.
- **Multi-page routing**: App shell with `routes { "/" -> list; "/editor" -> editor }`
  + `outlet`. Both pages `use store: NotesStore` and share state across routes.
- **Widget tier**: primitives + presentational `NoteEditor`.

## Build & run

```bash
auto build                              # .at -> gen/front/vue (Vue project)
cd gen/front/vue && pnpm install && pnpm dev
```

## Architecture

```
app.at          → App shell (routes + outlet + header + New button)
notes_store.at  → store NotesStore (notes state + CRUD actions, composable singleton)
note_editor.at  → NoteEditor (presentational widget)
types.at        → Note type
pages/
  notes_list.at   → list page (uses store, search + tag filter + note selection)
  editor_page.at  → editor page (uses store, NoteEditor + delete)
```

## Relation to 015-notes

015 = front-back comms testbed (another agent). 025 = frontend richness.
Merge-back task M-merge (Plan 338 §0) ports 025's store + routing + tags into
015 when 015's Rust comms are green.

See [docs/design/16](../../docs/design/16-app-generation-and-ai-authoring.md)
and [docs/design/18](../../docs/design/18-shared-store.md).
