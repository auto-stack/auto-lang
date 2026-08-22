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

## Phase 4 — full capability matrix on a widget child (T1) ✅

`item_matrix.at`: widget `MatrixItem(label, on_select: msg)` with use{fn}
(matrix_helpers.at ext import), computed (imported-fn call + chained),
msg/emit contract, per-instance model, watch, .Init/.Destroy, style scoped,
slot outlet — instantiated from the parent loop with the §18.5 loop-var
callback and injected slot content. **First build green.** Generated
MatrixItem.vue carries every capability (import/computed×2/ref/watch/
onMounted/onUnmounted/defineEmits/scoped style/<slot name="extra"/>);
App.vue renders `<template #extra>` injection. Note: "No widget or store
declarations found" warning for pure-fn helper files is non-fatal (ext
import still lands, same as musk forge_helpers.at).

## Phase 5 — callback idiom equivalence (T2) ✅

Both parent-side callback forms work on a WIDGET child:

- `on_select: .ItemSelected(item)` → `@Select="ItemSelected(item)"` (contract form)
- `onselect: .ItemSelected($event)` → `@select="ItemSelected($event)"` (component-fn idiom)

Vue listener camelize (`@select`/`@Select` both compile to the `onSelect`
prop) makes them runtime-equivalent against a `Select` msg-variant emit.
**Migration rule: component fn → widget renames need NO parent-side binding
changes** (child-kind-agnostic event-binding conversion on the parent side).

## Phase 6 — three-layer v-model chain (PLAN-037 T4/T5) ✅

`item_bind.at`: widget `BindChild` with NO props — its only surface is the
model var `value` (→ `defineModel<string>("value", {default: ''})`). The
parent binds `BindChild(value: .draft)` where `.draft` is App's model var.

Generated chain, verified end-to-end:

- App.vue: `<BindChild v-model:value="draft" />` (call-site model addressing —
  a prop name matching the child's model var + a writable state slot folds to
  `v-model:key`; expressions/props/literals on the channel are a HARD build
  error: "model channel `X.y` requires a writable state slot").
- BindChild.vue: `const value = defineModel<string>("value", { default: '' })`
  + `<input v-model="value" />` — the internal input folds to v-model too,
  including the native-HTML path (a bare `input { value: .x }` with no
  oninput handler now still two-ways; previously it silently degraded to a
  one-way `:value` when a style class forced the native element).

Plumbing: sub-widget model vars are collected same-file (api.rs) and
cross-file (auto-man from_workspace prescan →
`ui_build_shadcn_with_sub_widgets_and_stores_full` → ComponentGenOptions →
`VueGenerator.with_sub_widget_models`); both the sub-widget and
AuraNode::Component prop paths enforce the channel contract; a child
declaring a prop and a model var of the same name is a generation error.

## Phase 8 — `use.web` statement pilot (PLAN-037 T8) ✅

`item_matrix.at`'s in-widget `use { fn: matrixGreet from ... }` block became a
top-level statement:

```
use.web matrixGreet from "src/front/matrix_helpers.at"
```

Same ExtImport payload end-to-end: identical generated import
(`import { matrixGreet } from '@/ext/src/front/matrix_helpers'`), identical
ext copy, build green. Grammar (T6): default plain import (fn/object/const),
`component` (instantiable tags), `composable` (+ optional `refs: [...]`);
entries attach to every widget declared in the file. Non-vue backends fail
fast with "use.web requires the vue render target" (T7) instead of silently
dropping the imports. The in-widget use block remains as a deprecated alias.
