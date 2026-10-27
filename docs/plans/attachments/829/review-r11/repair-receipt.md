# PLAN-829 post-r10 repair receipt (F829-R8-01 residual, F829-R10-01/02)

2026-10-22. Worktree `D:/autostack/.wt/lang-829/auto-lang`, branch
`plan-829-dev`. R10 reviewed `1fa149873`; this receipt covers the three
P2 closures and the live positive/negative the R10 closing oracle required.

## F829-R8-01 residual — horizontal placement + real content clip

R10 bound inputs through the consumed `analyze_composite_shot` (real
2000×1600@2 capture + exact mutations):

| Input | R10 (`1fa149873`) | Post-repair |
|---|---|---|
| real initial / after_button | accept | accept |
| 2D-only dx=±80 clipped | **accept (bug)** | **reject** `p2d=false p3d=true clip=true` |
| 3D-only dx=±80 clipped | **accept (bug)** | **reject** `p3d=false p2d=true clip=true` |
| ground RGB(224) side bleed 25,600px | **accept (bug)** | **reject** `clip=false pad=0.494` |
| model RGB(86,2,10) side bleed 25,600px | **accept (bug)** | **reject** `clip=false pad=0.494` |

Expected: horizontal variants keep `clip=true` (placement ≠ overflow).
Permanent fixtures added in `tests/composite_gate_contract.rs`.
Raw facts: [geometry-output.txt](geometry-output.txt).
Mutations: [2d_only_dx-80_clipped.png](2d_only_dx-80_clipped.png) and siblings,
[3d_ground_color_side_bleed.png](3d_ground_color_side_bleed.png),
[3d_real_model_color_side_bleed.png](3d_real_model_color_side_bleed.png).

## F829-R10-01 — constrained layout regression closed

Real canonical resize 860×700@2 = 1720×1400 physical: Iced squeezes the
last Fixed(320) to **313 logical** (physical y=[754,1380)). Expected rects
now come from that constrained column math. Both genuine resize captures
accept (`v3_h=626`, `v3_obs=754..1378`); the constructed resize positive
paints the real 313 height. Not a widened tolerance.

## F829-R10-02 — screenshot completion barrier + live evidence

`ScreenshotBarrier` / `try_shot_task`: phase progression binds to actual
`CompositeShot` completion (or diagnosed Missing); window-id readiness
gates the first request. Unit control `r10_screenshot_barrier_waits_for_completion`.

| Live run (clean HEAD, windowed) | Result |
|---|---|
| iced-shared-device positive | **exit 0** — initial + after_button + resized all `content ok=true` |
| AGP_SABOTAGE_ICED=transparent_blit | **exit 1** — all three moments `content ok=false` |

Reports: [live-positive](live-positive/report.json.txt),
[live-negative](live-negative/report.json.txt).

## Regression / health

- R8 prior bound cases (down-80, bottom sky bleed, clipped down-80) still
  reject against the **new** library.
- 53/53 targeted: composite 13, delivery 2, squash 17, seam 5, shared 16.
- `cargo fmt --check` clean; `cargo check --tests` zero warnings.

## Route

Return for affected re-review of F829-R8-01 (AC-05/06/07/09) and
F829-R10-01/02 (AC-05/06/07). Merge-owned frozen gates, consumer
relocation, migration handoff clearance, Spec/ledger deposition, archive
and guard cleanup remain pending as before. No merge / archive / cleanup.
