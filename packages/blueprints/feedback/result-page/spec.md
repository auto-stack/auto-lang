+++
kind = "feedback"
name = "result-page"
palette = ["callout", "button", "badge", "separator", "text"]
extension_points = ["icon", "content", "actions"]
variants = ["success", "error"]
props = ["result"]
actions = ["primary", "secondary"]

acceptance = [
    "each variant renders its status icon and title without extra wiring",
    "deep-linked pages load the result via dataSource.fetch and render loading while pending",
    "both action slots render (or degrade gracefully when unwired)",
    "failure details are visible in the content region, not swallowed"
]

[dataSource]
fetch = "(id) -> Result"


+++

# Intent

A full-page operation outcome (AntD Result anchor): `success` confirms a
completed operation; `error` explains a failed one (403/404/500 semantics
fold into this variant's copy). The blueprint owns the centered icon +
title + actions composition; the consumer supplies the operation payload via
the `result` prop (or `dataSource.fetch` for deep links) and the
`actions.primary` / `actions.secondary` semantics.

# Props

| prop | type | required | default | notes |
| --- | --- | --- | --- | --- |
| `result` | Result | yes | — | the operation outcome (`id` / `status` / `title` / `detail`); seed for the page |

# What this blueprint absorbs (per-app variation)

- the action pair (primary: back / view item; secondary: support / retry)
- result copy and failure detail
- icon treatment (the `icon` EDIT region)

# Assembly guidance

- render the `icon` EDIT region (variant's status mark), the title from
  `result.title`, then the `content` EDIT region (`result.detail`)
- `success`: primary navigates to the created/changed resource; secondary
  returns to the origin list
- `error`: primary retries or navigates back; secondary links to support;
  put recoverable detail (request id, reason) in the `content` region
- deep-linked pages fetch via `dataSource.fetch(id)` — render loading while
  pending
- keep the composition centered with a generous top margin
- mark every extension point with a `// EDIT: <point>` comment

# References

- `success` — completed outcome + primary/secondary pair
  (reference/success.at)
- `error` — failed outcome + retry/support pair (reference/error.at)

# Gotchas

See `gotchas.md`.
