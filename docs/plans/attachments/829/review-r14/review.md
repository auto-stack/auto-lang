# PLAN-829 affected final review R14 — needs_fix

2026-10-22 (Asia/Shanghai), plan revision1. Independent review chat, separate
from implementation; no agent/chat created. Existing tree
`D:/autostack/.wt/lang-829/auto-lang`, branch `plan-829-dev`.
Frozen commit `36f520c104ad99e2eaed2e22a5a3f0bb93195391`, previous review
`1ff4ee67e06197180d4ab691c2d5764bdb6c5a61`, approved base
`fdb1fad2ce7afa124adfa3080323496df384e670`. Entry Plan SHA256
`0242D68F9F9DE6D48225E98F58706F8A3987A7754F9EACF4D99EE36138AC3F97`.
Tree clean at entry and verdict. R13 is the work receipt, not an independent pass.

## Outcome and closed findings

**Needs fix: new F829-R14-01/P2 (bounded-wait failure path) and
F829-R14-02/P3 (incorrect historical live-evidence binding).**

The requested repairs are substantially verified:

- **F829-R8-01 CLOSED.** The complete rectangle-complement bands cover the
  declared two-view layout. Current library rejects **19/19** R12 corruptions
  via `clip=false`; initial/after-button/both genuine resize controls stay green.
  [Output](perimeter-output.txt), [probe](perimeter-probe.rs).
- R8/R10 old horizontal/vertical/overflow corruptions remain rejected, with
  clipped placement variants keeping clip=true. R10-01 constrained resize fix
  remains closed. [R8 replay](r8-closing-output.txt),
  [R10 replay](r10-closing-output.txt).
- **R10-02's original premature-finalization/live closure is verified on36.**
  R14 independently rebuilt release and ran the REAL canonical case:
  [positive report](live-positive/report.json.txt) **exit0**, all three moments
  content ok=true; [transparent report](live-transparent/report.json.txt)
  **exit1**, all three content ok=false. Both fingerprints are
  **`36f520c104ad`**. These runs completed normally within the external90s bounds;
  no timeout termination was needed. [Status](live-status.txt).
  This does not cover the new wait-timeout failure path below.
- **57/57 targeted tests** actually rerun: composite17, delivery2, squash17,
  seam5, shared16. [Output](targeted-tests.txt). fmt/check--tests and repair
  diff-check all exit0, no product compiler warning. No full116-test/workspace
  matrix or root tier is claimed.

## F829-R14-01 / P2 — resized screenshot timeout uses the frozen animation clock

Affected existing obligations: AC-05/06/07, T-07/T-08/T-09/T-10. This is a
repair regression/incomplete newly implemented failure path. The work added
and explicitly claims bounded screenshot180-tick/click900-tick/resize waits,
with named diagnosed failures, in E21 and the new code. The original R10
closing contract also requires completion or diagnosed failure and window-ID
readiness. It is not an added hard wall-clock/GPU-preemption guarantee.

Established source path:

1. `update(Message::Tick)` increments `GRAPHICS.tick` only when **!frozen**
   (`iced_spike.rs:1267`). It passes that graphics tick to canonical_step.
2. Phase4 sets **frozen=true** before entering phase5 and issuing the resized
   screenshot (`:1499`). `phase_started_tick` is set to the current graphics tick.
3. `phase_age = tick - phase_started_tick` (`:1384`). In phase5, future Tick
   messages leave the graphics tick unchanged, so **phase_age stays0**.
4. The resized screenshot's Wait timeout requires phase_age>180 (`:1506`),
   which cannot occur while waiting in that normal phase4→5 state.

Direct source-path counterexample:

- [timeout-probe.rs](timeout-probe.rs), [private entry](timeout-entry.rs),
  [raw output](timeout-output.txt), [replay](replay.ps1).
- The full original iced_spike.rs is copied **byte-for-byte**, then only an
  include of the review entry is appended. [Prefix identity](source-copy-identity.txt)
  proves original SHA256 `5366DE199E1DB39B2D5B525EB1145AA32814699C6D4932285A816816A7EA099A`
  equals the copy's original-length prefix. No update/canonical_step/deadline
  function body is changed, extracted into a new algorithm or mocked.
- Review access sets the deadline-relevant state that production phase4
  establishes: phase5, pending resized, graphics tick120, frozen=true,
  phase_started_tick120. Withhold only the matching callback and deliver actual
  `Message::Tick` through original `update`/canonical_step1000 times.
  Unused native-window/GPU state is not needed by the Wait branch. This is an
  accelerated event-count replay, not a claim of1000 real-time33ms intervals.

| State / accepted control | Delivered Tick events | Actual |
|---|---:|---|
| initial pending, active clock | 181 | phase99, `initial-timeout` |
| after_button pending, active clock | 181 | phase99, `after_button-timeout` |
| **resized pending, frozen clock (production state)** | **1000** | **graphics tick120, phase5, no failure** |
| resized pending, force active clock (causal counterfactual) | 181 | phase99, `resized-timeout` |
| Actual CompositeShot delivered to frozen resized waiter | one callback then Tick | received/pending cleared, phase6 |

The callback control uses original CompositeShot handling. No native window,
OS click helper or GPU render is spawned by this controller replay. Finalizer
is not invoked: stop at the first phase99. The frozen-vs-active comparison
isolates the clock dependency; active phase5 is a diagnostic counterfactual,
not misrepresented as a normal production state. A lost/delayed callback's
promised timeout is the failure model; no claim that the current renderer
normally loses this callback is made.

Related readiness gap: actual boot with no window-ID event,1000 Tick messages,
reaches graphics tick1000 but remains phase0 with no diagnosed failure. The
early window_id-none return at1394 has no deadline. If claiming the case's
waits are bounded, this declared readiness path needs an explicit bounded
failure policy too; it is not covered by Missing in try_shot_task because the
caller returns before issuing a task.

Closing oracle: deadline time must continue independently of frozen animation
and required frozen-frame negative controls. Replay pending resized after
phase4 beyond the declared limit and require a named bounded failure, retain
initial/after-button timeout and valid delayed-completion controls, and cover
window-ID readiness failure. Do not merely enlarge180 or unfreeze graphics in
a way that breaks the canonical frozen-frame checks. Recheck full normal
positive/transparent negative on the resulting committed revision.

## F829-R14-02 / P3 — R13/E21 live receipts are not evidence of the SHA they claim

Affected existing AC-06/08/09, T-09. Required commit/lock/asset evidence binding
must describe the actual inspected payload; a clean-looking string is not
sufficient when the report itself contradicts the frozen source.

- Both R13 live reports say **`agp_commit=1ff4ee67e061`**.
- Both contain the new `f_top`, `f_pre2d`, `f_left`, `f_right` content-note
  fields. The frozen **1ff source has no `f_top=` format**; those fields are
  introduced by36's analyzer. [Direct old/current Git source check](prior-format-proof.txt).
- Current E21 and result.md still claim those reports establish the clean
  R12/1ff live oracle. The header/body pair cannot prove that old revision's
  execution. It may be a precommit or mislabeled payload; the exact build/copy
  cause is not established. No accusation of intent or unverified build.rs
  implementation prescription is made.
- Copies: [work positive](work-live-positive-report.json.txt),
  [work negative](work-live-negative-report.json.txt).

R14's independent **36** positive/negative pair above now supplies correctly
bound current normal-path evidence. Correct R13/E21/result's historical binding
with a dated explanation and reference the actual new frozen evidence; do not
silently rewrite old headers to the desired SHA or retain the unsupported1ff
claim. Preserve the raw receipts as history.

Investigation boundary: two tiny isolated Git fixtures ran the **original**
build.rs to explore one stale-fingerprint hypothesis; both actually reported
the modified source as **dirty**, so that code-root-cause hypothesis was NOT
reproduced and is not a product finding. The first queried status before the
second build, the second after it. Raw attempts are preserved as
[first](fingerprint-attempt-index-refresh.txt),
[second](fingerprint-attempt-after-build-status.txt), with
[source identities](fingerprint-exploration-identity.txt). Their exploratory
scripts assert the hypothesized false-clean behavior and therefore fail under
these observed controls; they are deliberately excluded from the main replay.
The confirmed finding is the receipt/source-version contradiction, not the
failed exploratory hypothesis. Test repositories exist only under ignored
`scratch/829/review-r14/fingerprint-*`, not as Plan checkouts/worktrees; no real
repository history was changed by their fixture baseline commits.

## Trailing replacement error and evidence audit

The user's `String to replace not found` target was checked against actual
files. The intended old clean-HEAD wording is already replaced with the dirty
R11 disclaimer in result.md/E20. No leftover exact old sentence was found;
that edit error itself is not a finding. F14-02 concerns the separate newly
added E21/1ff claim, not that already-corrected R11 paragraph.

Dependency lock unchanged (SHA256
`6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`):
wgpu27.0.1/winit0.30.13/iced and iced_wgpu0.14.0/iced_winit0.14.1/stock
iced_widget0.14.2. Four repair files remain AGP-only; no root workspace/lock
change. Spec inputs/SD proposals are frozen in this directory with hashes.
Current canonical inputs and proposed experimental/production boundaries
remain unchanged; exact three new Spec paths and GOAL-008 remain appropriate.
The carried SD-03 counter-only wording must be reconciled before deposition;
no canonical Spec or live ledger was edited or approved for publication.

Unchanged game, source oracle, asset, cache, lifecycle, measurement, delivery
and boundary evidence is reused only within R8/R10/R12's recorded scope:
this diff changes only Iced analyzer/phase code, tests and research notes.
Those unchanged paths/dependencies/configuration are not recertified by a new
full suite; new evidence covers all affected geometry, phase and live paths.

| AC | R14 affected disposition |
|---|---|
| 01/02/03/04 | Bounded prior pass retained; source/game/asset contracts unchanged; shared/squash/seam rerun |
| 05/06 | Partial: new bounded-failure path and evidence binding findings |
| 07 | Normal live/transparent+resize pass on36; new timeout/readiness failure path needs fix |
| 08 | Experimental/I1–I3 boundary retained; historical live claim needs correction |
| 09 | needs_fix |
| 10 | Merge-owned pending |

Route to same-tree work: fix both findings, diagnose/test clock ownership and
success/failure/readiness states, keep all closed geometry/source/delivery
repairs, commit and request affected re-review. T-01..06 complete,
`executing 6/11`; T-07..10 open, T-11 pending. No scope reduction or deferral
authorized. Final pass precedes merge-owned frozen AGP gates/consumer relocation,
migration/write-surface handoff clearance, Spec/ledger deposition, archive and
guard cleanup. No implementation edit/commit/merge/archive/cleanup or root
t/tv/tt/tb/tu/taa/tf/docs_gen; no new agent/chat or external message.
