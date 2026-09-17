+++
kind = "data-display"
name = "master-detail"
palette = ["list", "input", "separator", "badge", "skeleton", "text", "button", "image"]
extension_points = ["list_item", "detail", "empty_selection", "loading", "error_display"]
variants = ["default"]
props = ["breakpoint"]
actions = ["preview"]

[dataSource]
fetch_list = "(q) -> []Item"
fetch_detail = "(id) -> Detail"

acceptance = [
    "selection change fetches detail via dataSource.fetch_detail and renders loading while pending",
    "no-selection state is an explicit UI state, never a blank pane",
    "below props.breakpoint the panes stack (list OR detail, not both)",
    "list and detail keep independent loading/error branches"
]

+++

# Intent

A two-pane master-detail browser (PatternFly primary-detail anchor): a
filterable list on one side, the selected item's detail on the other. The
blueprint owns the pane layout, selection state machine, and the two fetch
lifecycles; the consumer supplies `dataSource.fetch_list` /
`dataSource.fetch_detail` and the `actions.preview` semantics. Below the
declared breakpoint the panes stack into list-or-detail navigation.

# Props

| prop | type | required | default | notes |
| --- | --- | --- | --- | --- |
| `breakpoint` | str | no | `"720px"` | min width that keeps the two-pane layout; below it the panes stack |

# What this blueprint absorbs (per-app variation)

- list item rendering (avatar, meta badges, selection accent)
- detail layout (the `detail` EDIT region)
- what selection means (in-place preview vs navigation on small screens)
- list filtering behavior (client vs server)

# Assembly guidance

- render the search `input` above the `list`; each entry is the `list_item`
  EDIT region with the selected accent
- selecting an item emits `Select(id)` and triggers `dataSource.fetch_detail`;
  render `skeleton` in the detail pane while pending
- the `detail` EDIT region renders the fetched detail; the `empty_selection`
  EDIT region renders the no-selection prompt ("Select an item…")
- keep list and detail loading/error branches independent — a detail failure
  must not blank the list
- below `props.breakpoint`, show one pane at a time with an explicit back
  affordance
- mark every extension point with a `// EDIT: <point>` comment

# References

- `default` — two-pane list + detail with independent state branches
  (reference/default.at)

# Gotchas

See `gotchas.md`.
