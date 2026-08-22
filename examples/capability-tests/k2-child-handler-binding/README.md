# k2-child-handler-binding (canary, GREEN — re-greened by PLAN-037 T3)

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

## PLAN-037 T3 re-green (2026-08-22)

The canary had drifted RED on master (two TS2345s): the callback-prop
contract was half-wired. Fixes:

1. **Source**: `Counter` now declares the `Select(int)` msg variant — the
   callback contract (`on_xxx: msg` prop + matching Pascal variant) is what
   drops the prop from defineProps and declares the emit.
2. **Codegen A** (vue.rs): the `props.on_xxx(` → `emit('<Pascal>',` body
   rewrite now also covers EMITTED-callback props (previously only "real"
   ones) — for those the prop is definitionally absent and the Pascal emit
   definitionally declared, so the rewrite closes the gap.
3. **Codegen B** (vue.rs): callback-contract events are now ALWAYS declared
   in defineEmits with the variant payload type (like quoted events),
   instead of requiring a template binding to reference them.

Verified: `auto build` green; generated Counter.vue = canonical contract
form (empty defineProps, `Select: [number]` emit, `emit('Select', next)`).
