# PLAN-829 post-r8 repair receipt (F829-R8-01..02 closing)

2026-10-21. Worktree `D:/autostack/.wt/lang-829/auto-lang`, branch
`plan-829-dev`. R8 baseline was `814f9bf01`; code repair `be00424c9`
added per-view position + full-perimeter clip + the first two bound
fixtures and the three named doc corrections. This receipt covers the
R8 closing oracle the patch itself did not yet demonstrate: the THREE
attached invalid cases on the real captures through the consumed gate,
the third permanent fixture, and the related source-constant audit.

## F829-R8-01 — three bound invalid cases now reject

Consumed path: `analyze_composite_shot` (the same function the Iced
composite case calls). Inputs: R6 durable `pixel-positive.png` (2000×1600
scale 2) and R8's three exact mutations (images regenerated and saved
here; changed-pixel counts match R8: 66,450 / 156,800 / 37,490).

| Input | R8 (baseline 814f9bf01) | Post-repair |
|---|---|---|
| real positive | accept | accept (`v3_obs=754..1392`) |
| R6 transparent / side-bleed / both-shift | reject | reject |
| 3D-only down-80 | **accept (bug)** | **reject** `p3d=false p2d=true` |
| bottom sky bleed 156,800 px | **accept (bug)** | **reject** `clip=false f_below=0.385` |
| 3D-only down-80 clipped | **accept (bug)** | **reject** `p3d=false clip=true` |

The clipped variant keeps `clip_ok=true`, so placement and overflow are
distinguished. Raw facts: [pixel-output.txt](pixel-output.txt). Probe:
[pixel-probe.rs](pixel-probe.rs). Saved mutations:
[3d-only-down80.png](3d-only-down80.png),
[3d-bottom-sky-bleed.png](3d-bottom-sky-bleed.png),
[3d-only-down80-clipped.png](3d-only-down80-clipped.png).

Permanent control: `tests/composite_gate_contract.rs` now has
`r8_3d_only_displacement_clipped_negative` in addition to the two E18
fixtures. Composite-gate suite 8/8; targeted matrix 48/48
(composite 8, delivery 2, squash 17, iced-seam 5, prototype/shared 16).

## F829-R8-02 — related source-constant audit closed

- `result.md` §4 no longer presents "spawn height y=1 resolved per frozen
  path" as the active summary; current rule is LOCAL y=0 first, [HISTORY]
  + dated correction retained.
- `src/host/cases/risk_slice.rs` still taught the y=1 misread as a live
  fact — rewritten to the Main.gd:37 local-position rule.
- Remaining y=1 / t=1402 strings live only inside dated CORRECTION /
  [HISTORY] / provenance paragraphs (sample-manifest, 3d-feasibility,
  generator header, golden-trace meta). Frozen `Main.gd` unchanged.
- Current committed trace agrees: spawn y=0, squashes 995/1938/1990,
  death 2703.

Full audit narrative: `experimental/autographics-prototype/research/evidence-notes.md` §E19.

## Health

- `cargo fmt --all -- --check` clean
- `cargo check --tests` zero warnings
- targeted 48/48 green
- no root tier / docs_gen / merge-owned gate run (work-phase only)

## Route

Return for affected re-review of F829-R8-01 (AC-05/06/07/09) and
F829-R8-02 (AC-01/08). Full live Iced positive/transparent negative,
consumer relocation, Spec deposition, archive and guarded cleanup remain
pending as R8 recorded. No merge / archive / worktree cleanup performed.
