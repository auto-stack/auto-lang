# 024-widgets-gallery — Component Gallery (all widgets + docs)

A shadcn-vue documentation gallery covering all widgets, each with an example
preview + properties table. Based on the comprehensive gallery content
(`examples/gallery`) with the Plan 408 §9 codegen fixes applied, so the
previously-broken widgets render natively.

## Layout

Responsive docs-site layout generated from `src/front/app.at`:

- **Header** — sticky, with mobile hamburger + search
- **Sidebar** (desktop) — categorized navigation: Overview / Form / Display /
  Feedback / Navigation / Overlay
- **Content** — `outlet`, each route is one component documentation page
  (`h1` + installation `codeblock` + `preview-card` demos + properties `table`)
- **Mobile** — bottom nav bar + drawer

## Widgets / routes (50)

Every shadcn widget has its own route: accordion, alert, alertdialog,
aspectratio, avatar, badge, breadcrumb, button, calendar, card, carousel,
checkbox, collapsible, combobox, command, contextmenu, datatable, datepicker,
dialog, drawer, dropdownmenu, form, grid, hovercard, input, label, menubar,
navigationmenu, navlink, pagination, popover, progress, radiogroup,
scrollarea, select, separator, sheet, sidebar, skeleton, slider, sonner,
switch, table, tabs, textarea, toast, toggle, togglegroup, tooltip + `/` index.

## Plan 408 §9 fixes exercised here

| Route | Fix |
|-------|-----|
| `/grid` | #1 — `grid` no longer a reserved keyword (usable as a route name) |
| `/slider` | #3 — Slider `value` int → `number[]` via `:default-value` |
| `/drawer` | #4 — Drawer requires `vaul-vue` npm dependency |
| `/toast` | #5 — toast tags → `ui/sonner` scaffolding (`<Toaster/>`) |
| `/navlink` | #6 — NavLink `href` → `router-link` `to` |
| `/pagination` | #7 — correct shadcn-vue export names (PaginationContent…) |

(Fix #2 — Rust-mode `outlet`/`link` placeholders — is codegen-only.)

## How to Run

```bash
cd examples/ui/024-widgets-gallery
auto gen        # generate code for all backends
auto run        # full pipeline: pnpm install + shadcn-vue add + vite dev server
```

`auto run` scaffolds the needed `ui/*` shadcn components (36), applies the
Sonner lucide icon-name compatibility patch, and serves on port 3024.

## Source layout

- `pac.at` — manifest (`render: "vue"`, port 3024)
- `src/front/app.at` — routes + responsive layout
- `src/front/pages/*.at` — one documentation page per widget (50 pages)
