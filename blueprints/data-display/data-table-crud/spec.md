+++
kind = "data-display"
name = "data-table-crud"
palette = ["table", "input", "select", "dialog", "button", "badge", "pagination", "dropdown-menu", "text", "separator"]
extension_points = ["toolbar", "filters", "row_actions", "form_dialog", "empty", "loading", "error_display"]
variants = ["minimal", "with_dialog"]
props = ["columns"]
actions = ["create", "update", "delete"]

[dataSource]
query = "(filter, page) -> Rows"

acceptance = [
    "renders loading, empty, and error branches — they are contract",
    "search/filter changes re-run dataSource.query with page reset to 1",
    "row actions (edit/delete) are reachable without leaving the table",
    "pagination reflects the query total, not just the current page slice"
]

+++

# Intent

A query table with search, filter, pagination, and row-level CRUD actions —
the AntD ProTable vocabulary. The blueprint owns the table chrome, the
query/filter/page state machine, and the loading/empty/error branches; the
consumer supplies `dataSource.query` and the `actions.create` / `actions.update`
/ `actions.delete` semantics. `with_dialog` materializes the create/edit
dialog surface.

# Props

| prop | type | required | default | notes |
| --- | --- | --- | --- | --- |
| `columns` | []Column | yes | — | column descriptors (`id` / `title` / `sortable`); drives headers and cell rendering |

# What this blueprint absorbs (per-app variation)

- column set and cell rendering (badges, links, formatted values)
- filter controls (the `filters` EDIT region)
- bulk actions vs per-row actions
- where create/edit happens (dialog page vs side route)

# Assembly guidance

- render the search `input` + filter `select`s in the `filters` EDIT region;
  any change resets `.page` and re-runs `dataSource.query`
- iterate `.rows` with a `for`; each row renders per `columns`
- per-row actions (edit / delete) live in the `row_actions` EDIT region as a
  `dropdown-menu`; destructive entries confirm before firing
- `with_dialog` materializes `form_dialog`: a `dialog` hosting the create/edit
  form; submit fires `actions.create` / `actions.update`
- always render `loading` / `empty` / `error_display` branches
- mark every extension point with a `// EDIT: <point>` comment

# References

- `minimal` — table + search + pagination + row action menu
  (reference/minimal.at)
- `with_dialog` — minimal + create/edit dialog surface
  (reference/with_dialog.at)

# Gotchas

See `gotchas.md`.
