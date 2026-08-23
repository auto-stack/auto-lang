# 031 — Dynamic Components (`dyn`) + Reactive Watchers (`watch`)

Two DSL capabilities the editor-menu replicas need:

- **`dyn (expr) { ... }`** — dynamic component, generating
  `<component :is="(expr) as any" ... />`. The parenthesized expression is
  the component value: a for-loop iterator field (`dyn (.item.icon)`), a
  model field, or a computed. Brace props/class/`style_obj`/events behave
  like on plain elements (`dyn (.item.icon) { size: 16, class: "..." }`).
  The paren-less form `dyn { is: .current_icon }` is equivalent. This is
  the SlashMenu shape: 30 menu items whose `icon` field is a lucide
  component passed in from hand-written TS.
- **`watch { ... }`** — widget-level reactive watchers, generating Vue
  `watch(source, () => { ... }, opts)` in `<script setup>` (after
  state/computed/defineProps, so watched refs are initialized). Sources
  are `.field` names: model fields and computed are watched as refs
  directly, props become getters (`() => props.x`). Comma-separated
  sources emit a multi-source array watch; `.immediate` / `.deep`
  modifiers map to watch options. The handler body uses the same
  conventions as `on` handlers (ts_adapter: `.field` state access via
  `.value`, template refs via `.value!`, props via `props.`).

## Syntax

```auto
widget SlashMenu {
    watch {
        .filtered_items -> { .selected_index = 0 }
        .ratio, .viewport_h.immediate -> { ... }
        .items.deep -> { ... }
    }
    view {
        col {
            for item in .items {
                dyn (.item.icon) { size: 16, class: "w-4 h-4" }
            }
        }
    }
}
```

## Scenarios

1. **dyn from list data** — `menuItems()` (hand-written TS) returns items
   whose `icon` is a lucide component; each row renders
   `<component :is="(item.icon) as any" :size="16" />`.
2. **search + watch reset** — the search box `v-model`s `.query`,
   `filtered` recomputes, and `watch { .filtered -> { .selected_index = 0 } }`
   resets the selection.
3. **watch prop → imperative geometry** — the parent's zoom buttons change
   `ratio`; the child `ThumbBar` watches its prop
   (`watch(() => props.ratio, ..., { immediate: true })`) and recomputes
   the thumb height from the track element's live DOM box — the
   CustomScrollbar scenario, without the `onscroll.window.capture`
   workaround.

## Build

```sh
auto build   # → gen/front/vue, runs vue-tsc + vite build
```

## Verify

- `gen/front/vue/src/App.vue` contains
  `<component :is="(item.icon) as any"` and
  `watch(filtered, () => {` with `selected_index.value = 0` in the body.
- `gen/front/vue/src/components/ThumbBar.vue` contains
  `watch(() => props.ratio, () => {` with
  `thumb_h.value = thumbHeight(trackEl.value!, props.ratio)` and
  `{ immediate: true }`.
- `pnpm run build` (vue-tsc + vite build) passes.
