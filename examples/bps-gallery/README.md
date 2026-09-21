# bps-gallery

Browses the AutoUI **bp** catalog (`blueprints/<kind>/<name>/`) as-authored:
the spec, each variant's reference implementation source, and the gotchas.

A bp is a *Skill-tier* unit (Design 17): a natural-language spec + structured
contract that AI assembles from widgets, with reference implementations and
gotchas.

PLAN-676 起本画廊是 **Auto 化双臂应用**（`auto run` / `auto run -r vm` 同一
`.at` 源）：壳消费 `layout/gallery-shell` BP（L1 通道零副本），条目点击/
搜索/主题切换全走 BP 的 on_* 回调契约；手写 Vue 壳（App.vue / vue-router /
Prism）已退役。

## How the catalog is discovered (PLAN-676 SD-01)

`src/front/registry.at` is a **generated artifact**: `auto bp list --format at`
scans `blueprints/**` and materializes the catalog as a `.at` static table
(`pub fn all_bps() List`) — committed, dual-arm same-source, drift-checked in
CI (the workflow re-runs the emitter and diffs against the committed file).
This replaces PLAN-640's zero-artifact `import.meta.glob` ruling: the VM arm
cannot glob the disk, so dual-arm parity requires a static table (PLAN-625
ui-gallery registry.at precedent). `auto bp list` remains the authoritative
registry view; the file is a read-only materialization of it.

When a bp package is added/removed/edited under `blueprints/`, regenerate:

```bash
auto bp list --format at > examples/bps-gallery/src/front/registry.at
```

## Run

```bash
cd examples/bps-gallery
auto build        # vue arm full chain (gen + vue-tsc + vite)
auto run          # vue dev server
auto run -r vm    # VM / iced desktop arm
```

Builds and the registry drift check are gated in CI
(`.github/workflows/build-bps-gallery.yml`) on changes to the gallery, to
`blueprints/**`, or to the emitter (`crates/auto`, `crates/auto-lang`).

## What it shows

- **Sidebar**: kind pills (PLAN-676 偏好序，原 bps.ts kindOrder) + bp
  double-line cards (id + first variant).
- **Content**: the spec body (frontmatter stripped), a variant tab switcher,
  the reference `.at` source, and the gotchas list.

## Scope note

Live-rendering a reference `.at` (compiling it via a2vue into a running preview)
is deferred — the gallery currently shows the authored source. Adding live
render is a follow-up (needs the a2vue single-widget compile path wired into a
dev-time preview).

See
[docs/design/blueprints/blueprints-first-class.md](../../docs/design/blueprints/blueprints-first-class.md),
[Plan 342](../../docs/plans/archive/342-block-tier-phase-a-package-foundation.md),
and [Plan 676](../../docs/plans/676-gallery-shell-bp.md).
