# 028 — DOM Escape Hatch

Imperative DOM access from `on` handlers, the capability the editor
replicas (popover positioning, custom scrollbars, scroll sync) need:

- **Template refs** — `ref: "menuEl"` on a view element emits a static
  `ref="menuEl"` template attribute plus a
  `const menuEl = ref<HTMLElement | null>(null)` declaration in
  `<script setup>`. Works on native elements and on shadcn-mapped
  elements (`col`, `button`, ...).
- **Ref access in `on` blocks** — `.menuEl.xxx` maps to
  `menuEl.value!.xxx`: property reads (`.scrollEl.clientHeight` →
  `scrollEl.value!.clientHeight`), property writes
  (`.scrollEl.scrollTop = n`), method calls
  (`.triggerEl.getBoundingClientRect()`, `.menuEl.querySelector(".x")`),
  and style writes (`.menuEl.style.left = f"${left}px"`).
- **document / window pass-through** — `document.activeElement`,
  `document.querySelector(...)`, `window.innerWidth` / `innerHeight`
  are emitted unchanged.

## Scenarios

1. **Popover positioning** (useMenuBounds style): the trigger button and
   the popover both carry refs. `ToggleMenu` reads
   `triggerEl.getBoundingClientRect()`, clamps against
   `window.innerWidth` / `innerHeight`, writes `left` / `top` back to the
   model (shown in the readout) and applies them imperatively to the
   always-mounted popover element.
2. **Custom scrollbar**: the viewport has `overflow: hidden`; `onwheel`
   adds `deltaY` to `scrollEl.scrollTop` and the thumb drag (window-level
   `mousemove` / `mouseup`) maps pointer delta → scroll ratio →
   `scrollTop`. Thumb position is derived from `scrollTop`,
   `scrollHeight` and `clientHeight` after every change.

## Build

```sh
auto build
```

## Verify

- `gen/front/vue/src/App.vue` declares
  `const {triggerEl,menuEl,scrollEl,trackEl,thumbEl} = ref<HTMLElement | null>(null)`
  and the template carries matching `ref="..."` attributes.
- Handler bodies use `scrollEl.value!.scrollTop`,
  `triggerEl.value!.getBoundingClientRect()`, `window.innerWidth` and
  `document.querySelector(...)`.
- `pnpm run build` (vue-tsc + vite build) passes.
