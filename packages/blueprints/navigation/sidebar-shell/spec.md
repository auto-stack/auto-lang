+++
kind = "navigation"
name = "sidebar-shell"
palette = ["sidebar", "header", "avatar", "dropdown-menu", "separator", "badge", "button", "text"]
extension_points = ["logo", "nav_items", "content", "user_menu", "footer"]
variants = ["default", "compact"]
props = ["nav_tree", "user"]
actions = ["sign_out"]

acceptance = [
    "renders the shell (three-section sidebar + content column with header and user menu) without any app-specific copy",
    "nav badges reflect dataSource.counts and refresh on navigation",
    "user menu exposes actions.sign_out behind a dropdown",
    "collapse toggle switches wide ↔ icon rail at runtime (collapse button / title icon), selected nav state preserved",
    "portrait tablets (<1024 window width) render the rail on first paint and re-widen at ≥1024 without restart (label face flips with the lg breakpoint)",
    "rail icon buttons show their label as a hover tooltip (title: wiring)",
    "compact variant collapses labels to icons with tooltips preserved"
]

[dataSource]
counts = "() -> Counts"


+++

# Intent

The application shell (shadcn Sidebar page-level anchor): a full-height
three-section sidebar (top: brand icon + collapse toggle / mid: nav tree /
bottom: settings footer) beside a content column that owns the header (brand +
user menu) and the routed content slot. Distinct from
`navigation/sidebar-nav` (the three-section nav *content*): this package is
the *frame* an app mounts routes inside. The consumer supplies the nav tree
and user via props, badge counts via `dataSource.counts`, and the
`actions.sign_out` semantics.

Since PLAN-688 this is the anchor example of an **interactive-state BP**
(SD-01): a bp-private `collapsed` model var + `ToggleCollapsed` message drive
the wide ↔ icon-rail switch via two mutually exclusive panes; no window-size
primitive is needed (see gotchas #5 for the pane-visibility recipe).

# Props

| prop | type | required | default | notes |
| --- | --- | --- | --- | --- |
| `nav_tree` | []NavNode | yes | — | nav entries driving the `nav_items` region. NavNode carries the full key set `{ id, label, badge, icon }` (VM missing-key access is a hard error); `icon` is a text glyph (compact.at `⌂▦⚙` precedent) — bake the label-first-char fallback on the consumer side, the bp does no slicing |
| `user` | User | yes | — | display identity (`name` / `email` / `avatar_url`) for the user menu |

# What this blueprint absorbs (per-app variation)

- logo / brand (the `logo` EDIT regions: sidebar title text + header brand)
- nav tree content and grouping
- header actions (search, notifications)
- footer nav items (settings row + one sample additional entry)
- what sign-out means (the `actions.sign_out` handler)

# Assembly guidance

- compose the three-section `sidebar` (top brand/collapse row / mid nav list /
  bottom settings footer) beside `col` (header + content slot)
- render the `nav_tree` in the `nav_items` EDIT region; badge counts come
  from `dataSource.counts()`
- the user menu is an `avatar` + `dropdown-menu` in the content-side header;
  sign-out is `actions.sign_out`
- the `content` EDIT region is the routed app surface — keep it a single
  slot, no bp-owned chrome
- the sidebar renders TWO mutually exclusive panes: the wide pane (each nav
  item one button with icon + title together) and the rail pane (icon-only,
  icons one size larger); the collapse toggle reuses one `ToggleCollapsed`
  message from both the wide chevron and the rail logo; portrait (<1024)
  stays rail regardless of the manual state (see gotchas #4/#5)
- keep `title:` attributes on rail icon buttons for hover tooltips
- mark every extension point with a `// EDIT: <point>` comment

# Non-goals

- **mobile ☰ floating menu** (recorded 2026-09-22, not implemented): on
  narrow/mobile windows a ☰ affordance in the top title bar / bottom tab bar
  should open a floating menu (Sheet) to switch nav sections. The floating
  layer primitives already exist (VM dropdown-menu/sheet/drawer/popover,
  PLAN-533/534); what's missing is an `isMobile` signal and an explicit
  reversal of the "mobile mechanism batch not built" non-goal in
  `docs/design/autoui/sidebar-family-and-nav-retirement.md` §2.2 — deferred
  to the mobile milestone.
- **manual expand override in portrait** (same batch): below 1024 the rail
  form is forced by the `lg:` breakpoint and the collapse button is visually
  a no-op; overriding that needs a window-size primitive readable from `.at`
  (evaluated together with the ☰ reversal — see gotchas #4/#6).
- collapse/expand width transition animation (no VM animation channel).
- RQ track adaptation (user ruling 2026-09-22: RQ unfinished, untouched; new
  visual faces must pass the DisplayList consumer scan when RQ lands —
  PLAN-683 in flight).

# References

- `default` — interactive three-section shell: wide ↔ icon-rail collapse +
  portrait auto-rail (reference/default.at)
- `compact` — static icon rail sidebar (reference/compact.at; kept as the
  pure-rail reference, unchanged by PLAN-688)

# Gotchas

See `gotchas.md`.
