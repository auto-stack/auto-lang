# vue-gallery — `@auto-ui/widgets` showcase

A Vite + Vue 3 + TypeScript app that consumes the [`@auto-ui/widgets`](../../packages/widgets)
package **directly** and renders every widget with its variants and states. It is
the package's:

1. **visual regression harness** — each generated widget must render correctly,
2. **dogfood consumer** — proves the package works in a real Vite project,
3. **living docs** — shows each widget, its variants, and its props.

## Quick start

```bash
cd examples/vue-gallery
pnpm install
pnpm dev      # http://localhost:5173
```

Build (type-check + bundle):

```bash
pnpm build
```

## How it consumes the package

`vue-gallery` imports widgets straight from the package via its `exports` map:

```ts
import { Button } from '@auto-ui/widgets/registry/button'
import '@auto-ui/widgets/styles.css'   // precompiled stylesheet (zero-Tailwind path)
```

The package is pulled in as a `file:` dependency. Widget styling comes from the
precompiled `dist/styles.css` — this gallery runs **no Tailwind of its own**,
validating the package's "Path A" zero-config stylesheet.

The gallery's own chrome (layout, sidebar, cards, tables) is plain CSS in
[`src/assets/app.css`](src/assets/app.css), reusing the design-token variables
(`--background`, `--border`, `--card`, …) that `styles.css` defines, so the
chrome stays consistent with the widgets. It loads after `styles.css`.

> The package's `exports` map targets each widget's `index.ts`
> (`"./registry/*": "./registry/*/index.ts"`); directory targets don't resolve
> under TypeScript/Rollup, so the map points at the index file explicitly.

## Relation to `examples/gallery`

These are two different things and intentionally coexist:

| | `examples/gallery` | `examples/vue-gallery` (this) |
|---|---|---|
| Shows | **Auto source** → generated Vue (side by side) | the `@auto-ui/widgets` **Vue components** |
| Audience | people learning the Auto language / transpiler | consumers of the npm package |
| Example code | Auto (`button "txt"`) | `<Button variant="…">` |
| Auto↔Vue comparison | yes | no |

`vue-gallery` is **not** generated from Auto source; it hand-writes Vue that
imports the package.

## Adding a widget page

When a new widget lands in `packages/widgets/registry/<widget>/`:

### Automated sync workflow (Plan 337)

1. Add a `library_template` entry + the widget name to `LIBRARY_WIDGETS` in
   `crates/auto-lang/src/ui_gen/vue.rs`. The self-consistency test (Plan 337
   Task 1.1) verifies every listed widget has a template.
2. `auto ui build --target vue --out packages/widgets/registry` — generate the
   package artifacts.
3. `auto ui build --target gallery-stubs --out examples/vue-gallery/src` —
   regenerate `widgets.generated.ts` and create a stub page for any widget
   that doesn't have one yet (existing pages are **not** overwritten).
4. Hand-write the page content (variants/states/code examples) and add a
   blurb + group entry in [`src/widgets.meta.ts`](src/widgets.meta.ts).
5. `auto ui backlog` — see which AURA widgets are still not in the library.

### Manual steps (if not using the sync tooling)

1. Add an entry to [`src/widgets.meta.ts`](src/widgets.meta.ts) (group + blurb).
   The name + route is in [`src/widgets.generated.ts`](src/widgets.generated.ts)
   (generated, do not edit).
2. Create `src/pages/<widget>.vue` following the existing pages (variants →
   sizes → states → `PropTable`). Import the widget from
   `@auto-ui/widgets/registry/<widget>`. Each `DemoBlock` takes a `:code`
   string (a copy-paste-ready `<script setup>` + `<template>` snippet) shown
   beneath the live preview in one card, syntax-highlighted via Prism (markup
   grammar — auto-highlights JS in `<script>` and CSS in `<style>`). Write the
   snippet as a template-literal constant and escape any literal `</script>` as
   `<\/script>`.
3. Add a route in [`src/router.ts`](src/router.ts).
4. `pnpm dev` to eyeball it, `pnpm build` to keep CI green.
