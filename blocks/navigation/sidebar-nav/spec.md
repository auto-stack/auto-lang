+++
kind = "navigation"
name = "sidebar-nav"
palette = ["button", "text", "badge", "separator"]
extension_points = ["shortcuts", "folders", "tags"]
variants = ["default", "compact"]

[dataSource]
folders = "() -> []Folder"
tags = "() -> []str"

+++

# Intent

A three-section sidebar navigation for note apps: quick-access shortcuts
(All/Pinned/Recent), folder list, and tag cloud. The block owns the
navigation chrome; the consumer wires folder/tag data sources and handles
selection messages.

# What this block absorbs

- shortcut tab rendering (All/Pinned/Recent)
- folder list rendering with icons
- tag cloud rendering
- selection state highlighting

# Assembly guidance

- render shortcut buttons in the `shortcuts` EDIT region
- iterate folders from `dataSource.folders()` in the `folders` EDIT region
- iterate tags from `dataSource.tags()` in the `tags` EDIT region
- emit `SelectShortcut(str)`, `SelectFolder(str)`, `SelectTag(str)` messages

# References

- `default` — three-section layout (shortcuts + folders + tags)
- `compact` — shortcuts + folders only (no tag cloud)

# Gotchas

See `gotchas.md`.
