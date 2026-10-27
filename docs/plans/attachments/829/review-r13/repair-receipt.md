# PLAN-829 post-r12 repair receipt (F829-R8-01 complete clip + live bind)

2026-10-22. Worktree `D:/autostack/.wt/lang-829/auto-lang`, branch
`plan-829-dev`. R12 reviewed `1ff4ee67e`. This receipt closes the complete
clipping complement and binds live evidence to that frozen revision.

## F829-R8-01 — complete background complement

R12's 19 perimeter mutations (real 2000×1600@2 + exact region paints):

| Region / color class | R12 | Post-repair |
|---|---|---|
| pre-2D gap y82..98 × sky/model/ground | **accept (3)** | **reject** `f_pre2d≈0.98` |
| outer control gutters × 3 | **accept (3)** | **reject** `f_left/right≈0.04` |
| bottom outer corners × 3 | **accept (3)** | **reject** `f_left/right≈0.05` |
| top padding RGB(24,38,52) | **accept (1)** | **reject** `f_top≈0.98` |
| inter-view gap / side gutters / below-3D × 3 | already reject | still reject |
| real initial / after_button / both resizes | green | green |

Per-band `frac_rect` (not a diluted whole-image fraction) keeps small
corner regions as sensitive as the large strips. Control interior still
allows button/text via `is_view_sig`.
Raw facts: [perimeter-output.txt](perimeter-output.txt).
Probe: [perimeter-probe.rs](perimeter-probe.rs). Mututations in this dir.

## R10-02 live closure bound to `1ff4ee67e061`

R12 correctly rejected R11's dirty `1fa149873a57-dirty` reports (E20 claim
now corrected). Bounded waits added (screenshot 180 ticks / click 900
ticks / resize) so the case diagnoses failure instead of hanging (also
covers R5-01). Fingerprint in both reports: **`1ff4ee67e061`**.

| Live run (release, windowed) | Result |
|---|---|
| iced-shared-device positive | **exit 0** — three moments `content ok=true` |
| AGP_SABOTAGE_ICED=transparent_blit | **exit 1** — three moments `content ok=false` |

Reports: [live-positive-report.json](live-positive-report.json),
[live-negative-report.json](live-negative-report.json).

## Health

- 57/57 targeted: composite 17, delivery 2, squash 17, seam 5, shared 16.
- R8/R10 prior bound corruptions still reject against this library.
- `cargo fmt --check` clean; `cargo check --tests` zero warnings.

## Route

Affected re-review of F829-R8-01 (AC-05/06/07/09) and R10-02 live
closure (AC-05/06/07). Merge-owned frozen gates, consumer relocation,
migration handoff, Spec/ledger deposition, archive/guard remain pending.
