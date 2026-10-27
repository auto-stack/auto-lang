# [PROVISIONAL — SD-01] docs/specs/auto-graphics/project.md

> Status: proposal drafted by PLAN-829 T-09. Frozen only by the T-10 review
> and published only by merge (auto-plan:merge). Do not treat as current
> Spec until then.

# auto-graphics — project

Module identity: experimental wgpu-based graphics/game research (AGP,
`experimental/autographics-prototype/`), independent workspace with own
lock/assets. Not part of the production UI stack; not a replacement for any
existing default implementation.

## Real entries (PLAN-829 evidence)

- Probe runner `graphics-probe` (10 cases); binaries `graphics-probe`,
  `iced_spike`.
- Frozen samples: AG-2D-01 Bevy Breakout (bevy@efe61cc5), AG-3D-01 Godot
  Squash the Creeps (godot-demo-projects@b761b4cd) — oracles in
  `research/sample-manifest.md`, frozen traces in `testdata/`.
- Machine profile: Windows x64, keyboard/mouse, DPI recorded per run.

## Ownership & boundaries

- Owner stage: work (PLAN-829) → review (T-10) → merge (T-11).
- Repository boundary: no path dependencies into auto-lang/auto-ui/auto-man
  private source; iced_widget 0.14.2 STOCK from crates.io (r1 F09:
  de-vendored after the production PLAN-043 patch proved unnecessary for
  this minimal spike; zero path-sourced packages in the closure); no
  junctions/symlinks; root workspace/lock untouched (V-09 gate).
- Production integration is PENDING (I1–I3); this file must not read as
  production capability. See integration-contract (SD-03).

## Non-goals (unchanged by PLAN-829)

Godot/Bevy engine compat, second render backend, production RQHost wiring,
full physics/animation editors, Tilemap/skeletal/networking/XR.
