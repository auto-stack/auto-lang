+++
kind = "data-display"
name = "note-list"
palette = ["badge", "button", "input", "separator"]
extension_points = ["empty", "error", "filter_bar", "items", "loading", "toolbar"]
variants = ["default"]

[dataSource]
list = "() -> []Note"
by_folder = "(folder) -> []Note"
by_tag = "(tag) -> []Note"

+++

# Intent

A scrollable, searchable list of notes with explicit loading / error / empty
states, plus folder and tag filtering. The block owns the list+search chrome,
the data state machine, and the filter bar; the consumer wires `dataSource.list`
to their `#[api]` fetcher and `dataSource.by_folder` / `dataSource.by_tag` to the
scoped fetchers. This is the block that backs the notes sidebar in Design 16's
M1 (Plan 338), upgraded for the 015-notes plan (Plan 354 Phase B).

# What this block absorbs (per-app variation)

- item rendering (title + meta + folder/tag badges + selection style)
- search behavior (client filter vs server-side)
- folder / tag filtering (filter bar chips + scoped re-fetch)
- empty / loading / error presentation
- toolbar actions (new, sort, filter)

# Folder and tag filtering

The `filter_bar` EDIT region renders the active scope as removable `badge` chips:
one per active folder and one per active tag, separated from the search input by a
`separator`. Selecting a scope (from a sibling navigation block) sets the
`.active_folder` / `.active_tag` model vars and re-fetches via the matching
dataSource slot:

- no scope set -> `dataSource.list()`
- `.active_folder != ""` -> `dataSource.by_folder(.active_folder)`
- `.active_tag != ""` -> `dataSource.by_tag(.active_tag)`

Each list item carries its `folder` and `tags` so the row can render a folder
`badge` for quick context; clearing a filter re-runs the unscoped fetch.

# Assembly guidance

- render a search `input` bound to `.search` in the `toolbar` EDIT region
- render active scope as removable `badge` chips in the `filter_bar` EDIT region,
  after a `separator` from the toolbar
- iterate `.items` with a `for`; each row is the `items` EDIT region
- always render `loading` / `error` / `empty` branches — they are contract
- emit a `Select(id)` msg; the consumer decides what selection does
- emit `ClearFolder` / `ClearTag` msgs when a filter chip is removed

# References

- `default` — search + filter bar + list + empty/loading/error (reference/default.at)

# Gotchas

See `gotchas.md`.
