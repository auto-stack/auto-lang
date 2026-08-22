# k3-widget-composition (canary, GREEN)

Status: **GREEN** — both phases pass `auto build` (vue-tsc + vite).

Question: can a page-level `widget` compose a child defined as a `component fn`
vs the same child defined as a `widget` — props down, per-instance state,
events up, instantiated inside a `for` loop with the parent closing over the
loop variable?

## Scenario

Parent `App` loops `for item in .items`, instantiates `ItemRow(label: item,
on_select/onselect: .ItemSelected(item))` (the §18.5 form — call-site callback
whose arg is the parent-scope loop var). Child shows the label, keeps a
per-instance click counter, and emits select on "pick".

## Phase 1 — child as `component fn` (`item_cf.at.disabled`) ✅

- Parent binds `onselect: .ItemSelected($event)` (payload form) and
  `onselect: .ItemSelected(item)` (loop-var form) — **both green**.
- Generated: `<ItemRow :label="item" :key="'ItemRow-1-' + (((item as any)?.id ?? item))" @select="ItemSelected(item)" v-for="item in items"/>`.
- Child: `defineProps<{ label }>` + `defineEmits<{ select: [string], bump }>` +
  per-instance `clicks` ref. Fires via `onclick: .select(.label)`.
- No callback prop declaration needed at all — the msg block IS the emit contract.

## Phase 2 — child as `widget` (`item_w.at`) ✅, with the right contract

Contract (Plan 043 M5 R4, see `prop_is_emitted_callback` in vue.rs):

- callback prop MUST be `on_<snake>` (underscore required) — `onselect` is not
  recognized as a callback at all;
- the child MUST declare a matching Pascal msg variant — `on_select: msg` +
  `msg Msg { Select(str) }` — then the prop is dropped from defineProps and
  delivered as `@Select`;
- the child fires the variant (`onclick: .Select(.label)`), not a call to the
  prop as a function.

With that contract the output is byte-equivalent to Phase 1 (emit names Pascal).

### Failures observed along the way (all reproduce)

1. `onselect: msg` (no underscore) + invoking it in the on-block → prop stays a
   REQUIRED prop in defineProps → parent TS2345 (`onselect` missing) + child
   TS2554 (arity).
2. **k2 idiom is RED on current master**: `on_select: msg` WITHOUT a matching
   `Select` variant, child calls `on_select(next)` inside a handler
   (k2-child-handler-binding). The `props.on_select(...)` → `emit('Select', ...)`
   rewrite fires, but the prop remains in defineProps and `Select` is absent
   from defineEmits → two TS2345s. k2's README says GREEN — regression or the
   canary predates the Plan 043 M5 R4 tightening. Follow-up: fix codegen to
   complete this idiom, or retarget k2 to the contract idiom.

## Findings

- Loop instantiation + per-instance props + per-instance state + loop-var
  callbacks work for BOTH `component fn` and `widget` children.
- The widget child needs the double declaration (`on_select: msg` param +
  `Select(str)` variant); the component fn child needs only the msg variant.
- Auto-key synthesis picks `item.id ?? item` for `:key` automatically.

See [Plan 345](../../../docs/plans/345-gap-canary-tests.md) for the canary series.
