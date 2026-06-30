# 025-notes-extended — regeneration SPEC

> Purpose: a notes app that showcases AutoUI's widget/block two-layer
> architecture. **Frontend-only, in-memory data, no backend.** This is the
> frontend-richness fork of `015-notes` (which remains the front-back comms
> testbed). Per Plan 338 §0, the richer frontend built here merges back into
> 015 once 015's Rust comms are stable.

## Functional spec (regenerate from this, no code)

A single-view notes manager:

- **Layout**: top header (title "Notes" + "+ New" button); a tag filter bar
  (All / intro / home / work); a body row with a left sidebar list and a right
  editor pane.
- **Data**: notes are held in app state (in-memory). Seed 4 notes with ids
  0..3, each `{ id, title, body, time, tags }`. Maintain a `next_id` counter.
- **Sidebar list**: a search input (filter by title substring) + a scrollable
  list of note titles. List is also filterable by the active tag from the bar.
  Clicking a title selects it (sets the active index).
- **Editor pane**: shows the active note. Read mode: title, time, tag chips,
  body (preformatted). Edit mode: title input + body textarea, with Save /
  Cancel. A "Delete note" button removes the active note.
- **Tag model**: each note carries `tags` (a list of strings). Tag bar filters
  the list to notes whose tags include the active tag ("All" = no filter).
- **CRUD** (all in-memory): New (append untitled, select it), Save (write edit
  fields back into the note), Delete (remove, clamp active index).

## Data model

```
Note { id int, title str, body str, time str, tags []str }
```

## Architecture notes (what the AI should know)

- Widget tier: `button`, `input`, `text`, `col`, `row`, `h2`, plus a
  presentational `NoteEditor` widget composed by `App`.
- Block tier: the sidebar is *intended* to be the `data-display/note-list`
  block. **Interactive adoption is deferred** — Auto's custom widgets are
  presentational (props in, no child→parent event binding yet), so list
  interaction stays inline in `App` (the proven `015-notes` pattern).

## Known deferrals (do NOT try to add these — they need unbuilt features)

- **Routing** (multi-page): needs shared state across routed pages, which
  requires a Rung-4 shared-store feature not yet built. Single-view only.
- **Markdown rendering** of the body: needs a JS/markdown stdlib bridge.
- **Block event-wired adoption**: needs child-handler binding.
