+++
kind = "form"
name = "wizard"
palette = ["button", "input", "text", "separator", "badge", "label"]
extension_points = ["step_content", "step_indicator", "summary", "error_display"]
variants = ["default"]
props = ["steps"]
actions = ["next", "back", "finish", "cancel"]

acceptance = [
    "only the active step's fields are rendered and validated",
    "next is gated on the current step passing its validation",
    "back never loses already-entered step data",
    "finish is reachable only from the last step and shows pending state"
]

[dataSource]
submit = "(steps) -> Result"

+++

# Intent

A multi-step onboarding / configuration wizard with a step indicator,
per-step validation gating, and a final review summary. The blueprint owns
the step state machine (step index, visited steps, completion) and the
next/back/finish flow; the consumer supplies per-step field composition and
`dataSource.submit` for the final commit.

# Props

| prop | type | required | default | notes |
| --- | --- | --- | --- | --- |
| `steps` | []Step | yes | — | ordered step descriptors (`id` / `title`); drives the indicator and index bounds |

# What this blueprint absorbs (per-app variation)

- per-step field composition (the `step_content` EDIT region)
- validation rules per step
- linear vs jump-back navigation policy
- review summary layout before finish

# Assembly guidance

- render the step indicator in the `step_indicator` EDIT region: one
  `badge` per `steps` entry (done / active / pending)
- render the active step's fields in the `step_content` EDIT region — only
  the current step, gated validation before `Next`
- keep one model; `back` decrements the index without clearing fields
- the `summary` EDIT region renders the entered data before `Finish`
- `Finish` -> `dataSource.submit`; show pending on the button, failures in
  `error_display`
- mark every extension point with a `// EDIT: <point>` comment

# References

- `default` — three-step shape: fields -> review summary -> finish
  (reference/default.at)

# Gotchas

See `gotchas.md`.
