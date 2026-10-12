# Gotchas — feedback/result-page

### Swallowing the failure detail

**Wrong**
The `error` variant renders "Something went wrong" with no reason, request
id, or recovery hint.

**Why**
Users can't distinguish "my fault, retryable" from "broken, contact
support", and support can't correlate the incident. The failure detail is
part of the variant's contract (the `content` EDIT region).

**Right**
Render `result.detail` (or per-status copy) in the `content` region —
recoverable detail as a `callout`, plus a support path in the secondary
action slot.

### Turning the result page into a dead end

**Wrong**
No actions wired — the page shows the outcome and strands the user.

**Why**
A result page is a routing waypoint: forward to the resource (success) or
back/retry (error). Both action slots exist in the contract; an unstyled
dead end fails the acceptance list.

**Right**
Wire `actions.primary` (view item / retry) and `actions.secondary` (back /
support). If a slot genuinely has no destination, render it as a no-op
gracefully — never as a broken button.

### Refetching the whole operation on deep link without a loading state

**Wrong**
Deep-linked result pages read `result` from a global that's empty on a cold
load.

**Why**
The prop doesn't survive a fresh navigation; the page renders empty titles
or crashes. Deep links are part of the spec's acceptance (`dataSource.fetch`).

**Right**
If `result` is missing, load it via `dataSource.fetch(id)` and render a
loading branch while pending; fail into the `error` variant.
