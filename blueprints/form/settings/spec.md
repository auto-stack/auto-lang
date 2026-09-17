+++
kind = "form"
name = "settings"
palette = ["form", "tabs", "switch", "select", "input", "button", "separator", "alert-dialog", "label", "text"]
extension_points = ["custom_section", "danger_zone", "success", "error_display"]
variants = ["default"]
props = ["sections"]
actions = ["save", "reset"]

[dataSource]
load = "() -> Config"
save = "(Config) -> Config"

acceptance = [
    "loads current config via dataSource.load before first paint of values",
    "dirty state gates navigation away with a confirm (alert-dialog)",
    "danger_zone actions require an alert-dialog confirmation",
    "save shows pending state and lands outcome in success/error_display"
]

+++

# Intent

A settings screen with tabbed sections, per-field editors (switch / select /
input), explicit dirty state, and a guarded danger zone. The blueprint owns
the section chrome, the dirty/clean state machine, and the save flow; the
consumer supplies `dataSource.load` / `dataSource.save` and the `actions.save`
/ `actions.reset` semantics (what "saved" means for their domain).

# Props

| prop | type | required | default | notes |
| --- | --- | --- | --- | --- |
| `sections` | []Section | yes | — | ordered section descriptors (`id` / `title`); one tab per entry |

# What this blueprint absorbs (per-app variation)

- which controls render per setting (switch vs select vs input)
- section grouping and ordering
- danger-zone actions (delete account, revoke sessions…)
- unsaved-changes policy (block navigation vs autosave)

# Assembly guidance

- render one `tabs` pane per `sections` entry; controls live in the
  `custom_section` EDIT region
- bind each control to the loaded config draft; any edit flips `.dirty`
- save button -> `dataSource.save(draft)`; show pending on the button and the
  outcome in `success` / `error_display`
- reset restores the last loaded config (confirm first when dirty)
- the `danger_zone` EDIT region renders destructive actions behind an
  `alert-dialog` confirmation
- mark every extension point with a `// EDIT: <point>` comment

# References

- `default` — tabbed sections + save/reset bar + guarded danger zone
  (reference/default.at)

# Gotchas

See `gotchas.md`.
