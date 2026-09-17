+++
kind = "navigation"
name = "sidebar-shell"
palette = ["sidebar", "header", "avatar", "dropdown-menu", "separator", "badge", "button", "text"]
extension_points = ["logo", "nav_items", "content", "user_menu", "footer"]
variants = ["default", "compact"]
props = ["nav_tree", "user"]
actions = ["sign_out"]

[dataSource]
counts = "() -> Counts"

acceptance = [
    "renders the shell (header + sidebar + content slot + user menu) without any app-specific copy",
    "nav badges reflect dataSource.counts and refresh on navigation",
    "user menu exposes actions.sign_out behind a dropdown",
    "compact variant collapses labels to icons with tooltips preserved"
]

+++

# Intent

The application shell (shadcn Sidebar page-level anchor): header with search
and user menu, sidebar with the nav tree, and a content slot. Distinct from
`navigation/sidebar-nav` (the three-section nav *content*): this package is
the *frame* an app mounts routes inside. The consumer supplies the nav tree
and user via props, badge counts via `dataSource.counts`, and the
`actions.sign_out` semantics.

# Props

| prop | type | required | default | notes |
| --- | --- | --- | --- | --- |
| `nav_tree` | []NavNode | yes | — | hierarchical nav entries (`id` / `label` / `children`); drives the `nav_items` region |
| `user` | User | yes | — | display identity (`name` / `email` / `avatar_url`) for the user menu |

# What this blueprint absorbs (per-app variation)

- logo / brand (the `logo` EDIT region)
- nav tree content and grouping
- header actions (search, notifications)
- what sign-out means (the `actions.sign_out` handler)

# Assembly guidance

- compose `header` (brand slot + actions) over `row` (sidebar + content)
- render the `nav_tree` recursively in the `nav_items` EDIT region; badge
  counts come from `dataSource.counts()`
- the user menu is an `avatar` + `dropdown-menu` in the header; sign-out is
  `actions.sign_out`
- the `content` EDIT region is the routed app surface — keep it a single
  slot, no bp-owned chrome
- `compact` collapses sidebar labels to icons; keep `title` attributes for
  tooltips
- mark every extension point with a `// EDIT: <point>` comment

# References

- `default` — full shell with labels + counts (reference/default.at)
- `compact` — icon rail sidebar (reference/compact.at)

# Gotchas

See `gotchas.md`.
