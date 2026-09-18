# bps-gallery

Browses the AutoUI **bp** catalog (`blueprints/<kind>/<name>/`) as-authored:
the spec, each variant's reference implementation source, and the gotchas.

A bp is a *Skill-tier* unit (Design 17): a natural-language spec + structured
contract that AI assembles from widgets, with reference implementations and
gotchas.

## How the catalog is discovered (PLAN-640)

`src/bps.ts` derives the whole catalog from disk via Vite
`import.meta.glob('?raw', { eager: true })` over
`blueprints/*/*/spec.md`, `…/gotchas.md`, and `…/reference/*.at` — no
generated artifacts, no manifest, zero hand-synced entries. Adding or
removing a package under `blueprints/` is picked up on the next `pnpm dev` /
`pnpm build`. (This replaces Plan 343's `auto bp export-gallery-meta`
outlook: a generator would have introduced a generated-artifact sync
surface; `auto bp list` remains the authoritative registry view.)

## Run

```bash
cd examples/bps-gallery
pnpm install
pnpm dev      # http://localhost:5173
pnpm build    # vue-tsc --noEmit && vite build
```

Builds are gated in CI (`.github/workflows/build-bps-gallery.yml`) on
changes to the gallery or to `blueprints/**`.

## What it shows

- **Sidebar**: bps grouped by kind (form / navigation / …).
- **Blueprint page**: the spec body (NL), a variant switcher, the reference
  `.at` source (Prism-highlighted), and the gotchas list (`—` when a package
  ships none).

## Scope note

Live-rendering a reference `.at` (compiling it via a2vue into a running preview)
is deferred — the gallery currently shows the authored source. Adding live
render is a follow-up (needs the a2vue single-widget compile path wired into a
dev-time preview).

See
[docs/design/blueprints/blueprints-first-class.md](../../docs/design/blueprints/blueprints-first-class.md)
and
[Plan 342](../../docs/plans/archive/342-block-tier-phase-a-package-foundation.md).
