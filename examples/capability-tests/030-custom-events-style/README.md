# 030 — Custom Events + style_obj

Two DSL capabilities the editor-menu replicas (SlashMenu-style popups)
need:

- **Quoted custom event names** — `on "autodown:slash-open".document:`
  declares a listener for a `CustomEvent` whose name contains `:`/`-`
  (illegal in identifier event keys). With `.window`/`.document` the
  generator emits `target.addEventListener(...)` in `onMounted` and the
  matching `removeEventListener(...)` in `onUnmounted` (same mechanism as
  standard global listeners); without a target modifier it becomes an
  element-level `@autodown:slash-open` template binding (works for native
  elements and component emits). `$event` is the raw DOM event, so
  `e.detail.*` chains read the CustomEvent payload.
- **`style_obj: { ... }`** — inline-style object binding, generating
  `:style="({ ... } as any)"`. Values are arbitrary expressions (state
  refs, f-string px concatenation like `f"${.menu_top}px"`); CSS property
  names that aren't valid JS identifiers (`z-index`) must be quoted in the
  DSL and are emitted quoted. This is distinct from `style: { class: cond }`,
  which stays a dynamic **class** binding (backward compatible).

## Syntax

```auto
view {
    col {
        on "autodown:slash-open".document: .OnOpen($event),
        on "autodown:slash-close".document: .OnClose

        col {
            style_obj: { top: f"${.menu_top}px", left: f"${.menu_left}px", visibility: .menu_vis, "z-index": 50 }
        }
    }
}
```

## Scenarios

1. **document-level CustomEvents** — a hand-written TS "extension"
   (`src/front/utils/slash.ts`, imported via `use { fn: ... }`) dispatches
   `autodown:slash-open` / `autodown:slash-close` on `document`; the widget
   listens at document level and reads `e.detail.query` / `.top` / `.left`.
2. **element-level custom event** — `demo:poke` is dispatched directly on
   the poke box (via its template ref) and handled by `.Poked($event)`.
3. **style_obj popover** — the SlashMenu popover's `top` / `left` /
   `visibility` / `z-index` are model-driven via `:style`.

## Build

```sh
auto build   # → gen/front/vue, runs vue-tsc + vite build
```

## Verify

- `gen/front/vue/src/App.vue` contains
  `document.addEventListener('autodown:slash-open', __auto_gl_autodown_slash_open_OnOpen)`
  (sanitized wrapper name) and the matching `removeEventListener` pair.
- The poke box carries `@demo:poke="Poked($event)"`.
- The popover carries
  `:style="({ top: `${menu_top}px`, left: `${menu_left}px`, visibility: menu_vis, 'z-index': 50 } as any)"`.
- `pnpm run build` (vue-tsc + vite build) passes.
