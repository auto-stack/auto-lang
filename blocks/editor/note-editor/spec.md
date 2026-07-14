+++
kind = "editor"
name = "note-editor"
palette = ["autodown_editor", "input", "badge", "button", "separator"]
extension_points = ["title", "editor", "toolbar", "tags"]
variants = ["default"]

[dataSource]
save = "(id: int, title: str, body: str) -> Note"

+++

# Intent

A rich note editor with AutoDown WYSIWYG editing, title input, tag management,
and a toolbar (save/delete/pin/move). The block owns the editor chrome and
save flow; the consumer wires `dataSource.save` to their API fetcher.

# What this block absorbs

- editor rendering (AutoDown Tiptap component or textarea fallback)
- title editing (inline input)
- tag management (add/remove badges)
- toolbar actions (save, delete, pin, move-to-folder)
- read/edit mode toggling

# Assembly guidance

- render an `input` for the title in the `title` EDIT region
- render `autodown_editor` with `content` bound to the note body in the `editor` EDIT region
- wire `@update` / `@save` events to `dataSource.save`
- render tag badges with remove handlers in the `tags` EDIT region
- render save/delete/pin buttons in the `toolbar` EDIT region

# References

- `default` — full editor with AutoDown + tags + toolbar

# Gotchas

See `gotchas.md`.
