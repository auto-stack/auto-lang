# k2-child-handler-binding (canary, RED)

Status: **RED** — the gap feature is not yet implemented; `auto build` is
expected to fail until it lands. See `src/front/app.at` for the desired
behavior and the "What's needed" note at its top.

This canary is the executable spec for the gap; it flips to GREEN when the
feature in [Plan 345](../../../docs/plans/345-gap-canary-tests.md) is
implemented.

## Note on computed args

The OOM that previously blocked computed msg args (`onclick: .Bump(.n + 1)`)
is **fixed** (root cause: `parse_event_arg` didn't consume binary operators,
so the caller's arg loop spun forever → ~48 GiB OOM; fixed in the parser, see
the `oom-event-binop-arg` canary). A *separate* pre-existing issue remains:
state-ref event args (`.n`) render as `this.n` (wrong for Vue; should be `n`)
— tracked separately. This canary uses a literal arg (`.Bump(1)`) to keep
vue-tsc green regardless.
