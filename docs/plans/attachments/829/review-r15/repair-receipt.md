# PLAN-829 post-r14 repair receipt (F829-R14-01/02)

2026-10-22. Worktree `D:/autostack/.wt/lang-829/auto-lang`, branch
`plan-829-dev`, repair commit **`ae40d6052`** on top of R14's
`36f520c104ad`.

## F829-R14-01 — wall-tick deadlines (P2)

`GRAPHICS.tick` freezes with the blank-frame control; phase 5's
`phase_age` stayed 0 and the resized screenshot timeout never fired.
Window-id wait had no deadline.

**Fix**: `SpikeApp::wall_tick` increments on every `Message::Tick`
regardless of freeze. All phase deadlines use `wall_tick`. Phase 0
window-id wait is bounded (`WINDOW_READY_DEADLINE_TICKS=300`,
`window-id-timeout`). Frozen-frame negative control unchanged.

## F829-R14-02 — live receipt binding (P3)

E21/R13 labeled reports `1ff4ee67e061` while their `f_top` fields only
exist in later source. CORRECTION recorded in evidence-notes §E21; those
reports are work context. R14's independent `36f520c104ad` runs stand
for that commit. **This repair's live oracle is bound to `ae40d6052cdb`**:

| Live run (release, windowed) | Result |
|---|---|
| iced-shared-device positive | **exit 0** — three moments `content ok=true` |
| AGP_SABOTAGE_ICED=transparent_blit | **exit 1** (content gate rejects) |

## Health

57/57 targeted (composite 17, delivery 2, squash 17, seam 5, shared 16);
fmt clean. Evidence: scratch `829/r14-live-bound*`.
