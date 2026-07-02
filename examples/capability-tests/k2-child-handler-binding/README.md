# k2-child-handler-binding (canary, GREEN)

Status: **GREEN** — `auto build` + `vue-tsc` pass. Parent↔child handler wiring
via a callback prop (`on_select: .Selected`), with the child invoking it from a
handler that receives a **computed** msg arg (`onclick: .Bump(.n + 1)`).

Two earlier blockers for the computed arg are now both fixed:

- **OOM** — `parse_event_arg` didn't consume binary operators, so the caller's
  arg loop spun forever → ~48 GiB OOM. Fixed in the parser (see the
  `oom-event-binop-arg` canary).
- **`this.n`** — the event-arg parser emits standalone `.field` as `this.field`
  (correct for ArkTS). Vue `<script setup>` uses bare state refs, so
  `handler_to_function_call_with_params` now strips a leading `this.` for Vue.
  Generated `@click` is `Bump(n + 1)`, not `Bump(this.n + 1)`.

See [Plan 345](../../../docs/plans/345-gap-canary-tests.md).
