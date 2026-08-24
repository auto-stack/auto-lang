# 041 — model var deep reactivity

Runtime canary for **plan 443** (defineModel downgrade narrowing). The Rust
tests `cap_model_channel_bound_downgrades_child` /
`cap_model_channel_unbound_keeps_ref` (`tests/vue_capabilities.rs`) pin the
emitted code shape; this fixture pins the **runtime** behavior the shape
exists to guarantee.

## The regression (pre-443)

Every widget `model {}` var compiled to `defineModel` (PLAN-037 T4).
`defineModel` is built on `customRef` — its get/set only covers the whole
value, and in the unbound (local-state) case it stores a plain object. Deep
in-place mutations therefore **silently lost reactivity**:

- jade-garden WhiteboardPage: `doc.value.shapes.push()` — data changed,
  view did not re-render (whiteboard e2e went red; DEBTS 015 🔴, 2026-08-24)
- auto-os-config probe B: `.config.provider.api_key = v` — same silent drift

## The fix (plan 443)

Only model channels a parent call-site actually binds (`v-model:name`)
compile to `defineModel` (with factory-wrapped object/array defaults — props
default semantics); every unbound model var emits `ref<T>(...)` again. `ref`
wraps object values in a deep reactive proxy, so nested writes and nested
pushes track.

## What this app pins

Three buttons, three deep mutations — **nothing whole-replaces the model**:

| button | handler | must update |
|---|---|---|
| deep-set | `.doc.provider.api_key = "deep"` | both text lines + `key_len` computed |
| deep-push | `.doc.provider.shapes.push("s2")` | shapes line + `shapes_len` computed |
| nested-set | `.doc.provider.api_key = "nested"` | both text lines |

Any line failing to update in place re-opens the 🔴.

## Verify

```sh
auto run                        # click the three buttons; every line updates
auto build -d . --gen-only      # gen/front/vue/src/App.vue (root widget):
grep 'ref<any>' gen/front/vue/src/App.vue         # the model var is a ref
grep -c defineModel gen/front/vue/src/App.vue     # → 0 (no binding here)
```

Filed from auto-os-config Plan 006 §3 Phase 6 (upstream-fix proposal; fix
itself landed same day as 38adb1ef / 4f64fb6c).
