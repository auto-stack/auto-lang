# 024-widgets-gallery — Codegen Fix Verification Gallery (Plan 408 §9)

A small shadcn-vue gallery that verifies the 7 codegen gaps fixed in Plan 408 §9.
Each route exercises one previously-broken widget **natively** — no workarounds —
so a successful `vue-tsc` build + visual check confirms the fix.

## Routes / verified fixes

| Route | Page | Plan 408 fix |
|-------|------|--------------|
| `/` | index | Landing page with links to all demos |
| `/grid` | grid | #1 — `grid` is no longer a reserved keyword (usable as a route name) |
| `/slider` | slider | #3 — Slider `value` int → `number[]` via `:default-value` |
| `/drawer` | drawer | #4 — Drawer requires the `vaul-vue` npm dependency |
| `/toast` | toast | #5 — Toast tags trigger the `ui/sonner` shadcn scaffolding (`<Toaster/>`) |
| `/navlink` | navlink | #6 — NavLink `href` prop forwarded to `router-link`'s `to` |
| `/pagination` | pagination | #7 — Pagination uses shadcn-vue export names (`PaginationContent`/`PaginationItem`/`PaginationPrevious`) |

> Fix #2 (Rust compiled-mode `outlet`/`link` → `View::empty()` / `View::text_styled()`)
> is codegen-only and has no Vue-visible surface; it is covered by the crate build.

## How to Run

```bash
cd examples/ui/024-widgets-gallery
auto gen        # Generate code for all backends
auto run        # Run dev server (installs shadcn components, starts vite)
```

The Vue project is generated into `gen/front/vue/`. `auto run` runs the normal
shadcn-vue install flow (which scaffolds `ui/drawer`, `ui/pagination`,
`ui/slider`, `ui/sonner` and applies the Sonner lucide icon-name compatibility
patch), then starts the dev server.

## Source layout

- `pac.at` — package manifest (`render: "vue"`, port 3024)
- `src/front/app.at` — routes + shared layout (uses `outlet`, `link`, `grid` as a route name)
- `src/front/pages/*.at` — one page per verified fix
