+++
kind = "dashboard"
name = "overview"
palette = ["sidebar", "header", "table", "card", "badge", "skeleton", "text", "button", "image"]
extension_points = ["header_actions", "stat_cards", "charts", "table_region"]
variants = ["default"]
props = ["period"]
actions = ["refresh"]

acceptance = [
    "stat cards render skeleton until dataSource.metrics resolves",
    "the charts region is a declared EDIT slot — the bp never fakes chart data",
    "refresh re-runs dataSource.metrics and preserves the selected period",
    "recent-activity table renders its own empty/loading branches"
]

[dataSource]
metrics = "() -> Stats"


+++

# Intent

The dashboard overview (shadcn dashboard-01 anchor): stat cards, a chart
band, and a recent-activity table under a period-aware header. The blueprint
owns the layout skeleton, the metrics fetch lifecycle, and the loading
states; the consumer supplies `dataSource.metrics`, the period semantics,
and mounts official chart packages into the `charts` EDIT region (charts are
NOT in this bp's palette — see the registry DEBT entry: chart tags are not
WidgetRegistry-registered, PLAN-484 retirement).

# Props

| prop | type | required | default | notes |
| --- | --- | --- | --- | --- |
| `period` | str | no | `"30d"` | reporting window selector (`"7d"` / `"30d"` / `"90d"`); drives the header select |

# What this blueprint absorbs (per-app variation)

- which stat cards render (the `stat_cards` EDIT region)
- which charts mount in the `charts` region (app picks official chart bps)
- the activity table's columns and row rendering
- period vocabulary (week/month/quarter labels)

# Assembly guidance

- header: title + period select + refresh button (`header_actions` region);
  refresh re-runs `dataSource.metrics`
- stat cards bind to the metrics result; render `skeleton` until resolved
- the `charts` region is a single EDIT slot — mount chart bps from the app
  layer; this bp ships no chart widgets (DEBT: chart registry gap)
- the `table_region` region renders recent activity with its own
  loading/empty branches
- mark every extension point with a `// EDIT: <point>` comment

# References

- `default` — cards + chart slot + activity table (reference/default.at)

# Gotchas

See `gotchas.md`.
