# blocks-gallery

Browses the AutoUI **block** catalog (`/blocks/<kind>/<name>/`) as-authored:
the spec, each variant's reference implementation source, and the gotchas.

A block is a *Skill-tier* unit (Design 17): a natural-language spec + structured
contract that AI assembles from widgets, with reference implementations and
gotchas. This gallery reads the real package files via Vite `?raw` imports, so
it always reflects `blocks/` on disk.

## Run

```bash
cd examples/blocks-gallery
pnpm install
pnpm dev      # http://localhost:5173
pnpm build    # vue-tsc --noEmit && vite build
```

## What it shows

- **Sidebar**: blocks grouped by kind (form / data-display / …).
- **Block page**: the spec body (NL), a variant switcher, the reference `.at`
  source (Prism-highlighted), and the gotchas list.

## Scope note

Live-rendering a reference `.at` (compiling it via a2vue into a running preview)
is deferred — the gallery currently shows the authored source. Adding live
render is a follow-up (needs the a2vue single-widget compile path wired into a
dev-time preview).

See [docs/design/17-blocks-first-class.md](../../docs/design/17-blocks-first-class.md)
and [Plan 342](../../docs/plans/342-block-tier-phase-a-package-foundation.md).
