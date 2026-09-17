+++
kind = "feedback"
name = "empty-state"
palette = ["text", "button", "image", "callout", "separator"]
extension_points = ["headline", "media", "actions"]
variants = ["first_use", "no_result", "error"]
props = ["illustration"]
actions = ["primary"]

[dataSource]
refresh = "() -> ()"

acceptance = [
    "each variant renders its distinct headline and body copy",
    "the primary action is reachable by keyboard and announced with a label",
    "error variant offers a retry wired to dataSource.refresh",
    "illustration falls back to a text marker when the prop is empty"
]

+++

# Intent

The empty-state triad (SAP Fiori taxonomy): `first_use` (nothing here yet —
invite creation), `no_result` (filters matched nothing — invite relief), and
`error` (fetch failed — invite retry). The blueprint owns the centered
media + headline + action composition; the consumer supplies the copy via
EDIT regions, the `illustration` prop, and the `actions.primary` semantics.

# Props

| prop | type | required | default | notes |
| --- | --- | --- | --- | --- |
| `illustration` | str | no | `""` | media source for the `media` region; empty falls back to a text marker |

# What this blueprint absorbs (per-app variation)

- headline / body copy per variant
- the illustration (or none)
- what the primary action does (create / clear filters / retry)

# Assembly guidance

- render the `media` EDIT region (the `illustration` prop, or a text marker
  fallback), then the `headline` EDIT region, then the `actions` EDIT region
- `first_use`: primary action invites the first creation ("New note")
- `no_result`: primary action offers relief ("Clear filters"); a `button`
  works better than a `callout` here
- `error`: primary action retries via `dataSource.refresh`; surface the
  failure detail in a `callout`
- keep the composition centered with a generous top margin
- mark every extension point with a `// EDIT: <point>` comment

# References

- `first_use` — nothing here yet + create CTA (reference/first_use.at)
- `no_result` — no matches + clear-filters CTA (reference/no_result.at)
- `error` — fetch failed + retry CTA (reference/error.at)

# Gotchas

See `gotchas.md`.
