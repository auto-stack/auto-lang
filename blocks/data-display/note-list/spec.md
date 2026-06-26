+++
kind = "data-display"
name = "note-list"
palette = ["button", "input"]
extension_points = ["items", "empty", "loading", "error", "toolbar"]
variants = ["default"]

[dataSource]
list = "() -> []Note"

+++

# Intent

A scrollable, searchable list of notes with explicit loading / error / empty
states. The block owns the list+search chrome and the data state machine; the
consumer wires `dataSource.list` to their `#[api]` fetcher. This is the block
that backs the notes sidebar in Design 16's M1 (Plan 338).

# What this block absorbs (per-app variation)

- item rendering (title + meta + selection style)
- search behavior (client filter vs server-side)
- empty / loading / error presentation
- toolbar actions (new, sort, filter)

# Assembly guidance

- render a search `input` bound to `.search` in the `toolbar` EDIT region
- iterate `.items` with a `for`; each row is the `items` EDIT region
- always render `loading` / `error` / `empty` branches — they are contract
- emit a `Select(id)` msg; the consumer decides what selection does

# References

- `default` — search + list + empty/loading/error (reference/default.at)

# Gotchas

See `gotchas.md`.
