# PLAN-829 post-r16 repair receipt (F829-R14-01/02 residual)

2026-10-22. Worktree `D:/autostack/.wt/lang-829/auto-lang`, repair commit
**`ddbde3e62`** on R16's `ae40d6052`.

## P2 — missing window-id finalizer schedules a real exit (F829-R14-01)

Phase 99 with `window_id == None` returned `Task::none` after writing the
report; the loop stayed subscribed and the native path panicked in
`Surface::present` (exit 101).

**Fix**: `None => iced::exit()` so the runtime returns. Unit holdout
`missing_window_id_finalizer_schedules_exit` asserts `terminal.units() >= 1`
and the known-ID sibling still returns `window::close` (`units == 1`).

## P3 — result.md binding corrected (F829-R14-02)

`result.md` r12 bullet no longer certifies the mislabeled `1ff…` reports as
commit-bound; dated [HISTORY]/CORRECTED points at R14 `36f520c104ad` and
this repair.

## Live bound to `ddbde3e62857`

| Run | Result |
|---|---|
| iced-shared-device positive | **exit 0**, 3 moments `content ok=true` |
| transparent_blit negative | **exit 1** |

Reports in this directory. 57/57 targeted + 1 lib timeout holdout; fmt clean.
