+++
kind = "dashboard"
name = "overview"
palette = ["sidebar", "header", "table", "card", "badge", "skeleton", "text", "button", "image", "line-chart", "bar-chart", "area-chart", "donut-chart"]
extension_points = ["header_actions", "stat_cards", "charts", "table_region"]
variants = ["default", "with_charts"]
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
states; the consumer supplies `dataSource.metrics` and the period semantics.
Official chart components mount into the `charts` region: the bp declares
them via the schema `package_origin` vocabulary face (PLAN-643 — the tags
are NOT WidgetRegistry builtins, PLAN-484 retirement stands) and the
`with_charts` variant materializes the slot; the `default` variant keeps it
an empty EDIT slot.

# Promotions

## with_charts — charts region materialization (PLAN-643, 2026-09-18)

- **Difference**: `default` leaves `charts` an empty EDIT slot (consumer
  mounts chart bps at the app layer); L3 assembly showed every real consumer
  immediately mounts official chart components — the slot materialization is
  structural, not per-app styling.
- **Proposal**: declare a `with_charts` variant that mounts the official
  chart package (`use { package: official from "components" }`, consumer
  front convention) into the `charts` region; extend `palette` with the four
  chart tags via the schema `package_origin` vocabulary face
  (`BlueprintRegistry::palette_drift` legal set, PLAN-643 SD-02).
- **Review**: approved within PLAN-643 (chart-tag-unify) — the variant binds
  charts to `dataSource.metrics`-shaped model fields, never hand-draws fake
  chart markup (gotcha unchanged); the DEBT that motivated the empty slot
  ("chart tags are not WidgetRegistry-registered") is resolved by the
  package-origin vocabulary face, not by registry registration (PLAN-484
  "no engine-builtin charts" preserved).

# Props

| prop | type | required | default | notes |
| --- | --- | --- | --- | --- |
| `period` | str | no | `"30d"` | reporting window selector (`"7d"` / `"30d"` / `"90d"`); drives the header select |

# What this blueprint absorbs (per-app variation)

- which stat cards render (the `stat_cards` EDIT region)
- which charts mount in the `charts` region (`default` = app mounts chart
  bps at the app layer; `with_charts` = official chart components prewired)
- the activity table's columns and row rendering
- period vocabulary (week/month/quarter labels)

# Assembly guidance

- header: title + period select + refresh button (`header_actions` region);
  refresh re-runs `dataSource.metrics`
- stat cards bind to the metrics result; render `skeleton` until resolved
- `charts` region: `default` keeps a single EDIT slot (mount chart bps from
  the app layer); `with_charts` materializes it with the official chart
  package (package from `components` — consumer front convention) bound to
  metrics-shaped model fields; never hand-draw fake chart markup
- the `table_region` region renders recent activity with its own
  loading/empty branches
- mark every extension point with a `// EDIT: <point>` comment

# References

- `default` — cards + chart slot + activity table (reference/default.at)
- `with_charts` — charts region materialized with official chart components
  (reference/with_charts.at)

# Gotchas

See `gotchas.md`.
