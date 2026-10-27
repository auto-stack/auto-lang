# PLAN-829 affected final review R10 — needs_fix

2026-10-22 (Asia/Shanghai). Review revision 1 at
`1fa149873a578dcff44537eb383ee7ef05be6dc8`, branch `plan-829-dev`, existing tree
`D:/autostack/.wt/lang-829/auto-lang`; repair chain `814f9bf01 → be00424c9 → 1fa149873`.
Original approved base `fdb1fad2ce7afa124adfa3080323496df384e670`.
Independent review chat, separate from implementation; no agent/chat created.
R9 is the work repair receipt, not a new independent pass. Entry Plan SHA256:
`B7928560D2AD7CE86577F1ED807A5FD50857D7B92F3E877D8F46FFAA8EE88EF9`.

## Outcome and verified closures

**Needs fix. F829-R8-02 CLOSED. F829-R8-01 original inputs CLOSED, complete
geometry/clip invariant PARTIAL. Two additional P2 failures were verified in
the affected path, including a repair regression on the real resize control.**

- Independently built current library and reran **48/48 targeted tests**:
  composite 8, delivery 2, squash 17, seam 5, shared runtime 16.
  [Actual test output](targeted-tests.txt).
- Replayed R9's exact real-image probe, linked to Cargo-JSON-resolved current
  libraries: original positive green, R6 transparent/side-sky/both-view-shift
  red; R8 three exact corruptions red. The clipped vertical variant retains
  `clip_ok=true`, so the old examples are properly separated.
  [Output](prior-input-output.txt), [probe](prior-input-probe.rs).
- Source-rule audit: corrected sample-manifest, result summary, feasibility
  and risk_slice live note all agree with local y0 and current death2703.
  Older y1/t1402 occurrences in those active files carry dated history or
  correction. [Audit](consistency-audit.txt), [trace](trace-events.json.txt).
  Frozen Main.gd and trace are unchanged from R8's independently verified
  fixture/driver evidence; the new diff changes no game simulation or oracle.
  F829-R8-02 is closed, without re-running or re-freezing the source oracle.
- fmt, check --tests and repair diff-check all pass; no warnings in targeted
  checks. Lock unchanged, SHA256
  `6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988`.

## F829-R8-01 remains OPEN / P2 — horizontal placement and non-sky clipping

Existing V-06 / AC-07 and V-08 / AC-05/06 require correct view placement/clipping
and a discriminating consumed gate, within this exact two-view sample. E18's
per-view × per-edge diagnosis still observes only **row** extents for position,
and only two selected colors outside the rectangles. This is an incomplete
existing invariant, not a new arbitrary-image or arbitrary-nesting requirement.

Independent probe: [geometry-probe.rs](geometry-probe.rs),
[raw output](geometry-output.txt), [replay](replay.ps1). It loads the unchanged
R6 real 2000×1600 @2 screenshot, preserves controls/other view, applies actual
mutations, saves PNGs and feeds the **public consumed function** directly.

| Bound invalid input | Expected | Actual |
|---|---|---|
| Only 2D shifted left/right 80px, clipped to original rect | Position false; clip true | **ok/p2d/p3d/clip all true**, 93,080 changed pixels each |
| Only 3D shifted left/right 80px, clipped to original rect | Position false; clip true | **all true**, 164,683 changed pixels each |
| 25,600 ground-colored side-gutter pixels, RGB224 | Clip false | **all true** |
| 25,600 side-gutter pixels using actual model color from the real view, RGB(86,2,10) | Clip false | **all true** |

The horizontal translations are defined relative to the existing x=[20,1980)
view rect. Correct clipping removes excess; it does not justify the wrong
placement or an 80px blank band. The final color is sampled from the actual
frozen scene image, not a hypothetical future asset. Rendered geometry can
cross the view boundary in colors other than sky/clear; repeating those colors
in the gutters is a direct clip-validator negative control.
These transformed images establish validator false acceptance; they are not
claimed to have been emitted by the renderer. The two separate live failures
below were observed on the unchanged, clean-HEAD production probe path.

Causes: `iced_spike.rs:949..956` compares only observed y tops/bottoms. No x
extent is checked. At line1057 `outside_any_view` still accepts only near-2D-clear
or near-sky; actual ground/model colors are invisible to clip observation.
The 3D row-position sampler excludes the side gutters, so it cannot compensate.

Closing oracle: reject the attached horizontal translations separately for
each view and both directions, retain correct clipping, and observe visible
out-of-bounds **content** including these actual source colors. Keep initial,
after-click, actual resize and original R6/R8 controls. A finite checker for
this sample is sufficient; no generic perfect image classifier is demanded.
Do not weaken the layout promise or refreeze corrupt outputs.

## F829-R10-01 / P2 — new position check rejects the real canonical resize

Existing AC-07 / T-08 requires the legitimate resize/DPI path to pass; R8's
closing oracle explicitly preserved resize positive controls.

Inputs and observations:

1. Archived real resized composite from `scratch/829/r4/iced-positive5`, saved
   byte-for-byte as [real-resized.png](real-resized.png); original associated
   [report](prior-live-report.json.txt) is preserved with its historical dirty
   fingerprint (it is not misrepresented as a fresh clean-HEAD run).
2. **Fresh clean-HEAD real window run** at 1fa, same canonical command, actual
   click and resize. The second run captures its
   [resized composite](live-positive-repeat/resized_composite.png).

Both images are 1720×1400 physical @2. Real Iced layout gives the 3D target
**840×313 logical**, y=[377,690), i.e. physical y=[754,1380). The target's
actual 313 height is present in both the old and current reports/keyframes.
`Primitive::prepare` uses the received `bounds.height` for the texture size.
This is supported canonical resize to 860×700, not a new small-window promise.

Current analyzer: `v3_obs=754..1378`, p2d=true, p3d=false, clip=true, **ok=false**.
It anchors the nominal second Fixed(320) height to physical bottom1394 and
expects last content row1393±8. But Iced constrains the second child to the
remaining content height inside the parent's 10px bottom padding: its actual
bottom is1380. The 13–15px difference exceeds the tolerance.

Same byte inputs through the **unmodified old analyzer from 814f9bf01** both
pass. [Old output](old-analyzer-output.txt), [old source](old-analyzer.rs),
[adapter](old-analyzer-probe.rs). This isolates a repair regression instead of
blaming an old screenshot or a new renderer. The fresh 1fa screenshot confirms
the old capture's applicability. The producer layout/render path was unchanged
by this repair.

Cause: `iced_spike.rs:868` uses nominal view_h320 and clips only to full h;
line952 compares the new observed extent against that wrong expected bottom.
The permanent resize control manually paints all320 logical pixels to the
nominal bottom, duplicating the wrong assumption and passing an image that
does not represent the constrained Iced layout.

Closing oracle: derive expected rects from the actual constrained producer
layout, preserve independent corruption sensitivity, and accept both genuine
resize inputs (plus initial/click), including a completed new live positive.
Replacing the actual rectangle with observed content extents or merely widening
tolerances would again let corruption redefine its expectation.

Review limitation corrected: earlier R8/R9 constructed resize positives shared
the nominal-height assumption. The actual old/current screenshots and fresh
live target dimensions supersede that insufficient positive control.

## F829-R10-02 / P2 — canonical finalizes before required screenshot completion

Existing AC-06/07 / T-07/T-08 requires the actual canonical composed screenshot
sequence, not only helper fixtures; the three required moments must be consumed
before a successful case can be established. Newly observed existing path gap,
not introduced by the R8 repair (phase code is unchanged).

Two bounded actual window runs at **clean 1fa149873a57** completed normally
with **exit1**; neither hit the 90s external timeout. OS click was observed,
both views rendered on Iced's device, resize reached 860×700 and control gates
were exercised. First run: initial and resized composite screenshots missing;
only after_button present. Second run: initial missing; after_button present;
resized present and rejected for R10-01. Exact reports:

- [Run1](live-positive/report.json.txt), [status](live-status.txt),
  [stderr](live-stderr.txt).
- [Run2](live-positive-repeat/report.json.txt), [status](live-repeat-status.txt),
  [stderr](live-repeat-stderr.txt).

`shot_task` returns a Task whose mapped `Message::CompositeShot` stores the
required evidence. But canonical_step sets phase1/3/5 as soon as it **requests**
each screenshot. Subsequent ticks dispatch click/resize and reach phase7,
finalize and close without requiring the matching CompositeShot to have been
processed. The CompositeShot handler appends facts without driving readiness.
Synchronous GPU readback inside Iced's action handler does not establish that
the mapped app callback has been processed before later ticks/finalization.
`shot_task` also returns no task if window_id is unavailable, while the caller
still advances. A completion/readiness guard is absent on either path.

Established: required evidence is missing at finalization in two clean real
runs, and the source has no completion barrier. The precise runtime scheduling
point for each omitted callback was not instrumented; no claim of GPU failure,
host throttling or a specific lost-message mechanism is made. The gate rejects
missing evidence correctly (no false pass); the canonical producer cannot
reliably finish its required positive sequence.

Closing oracle: bind phase progression/close to actual completion or diagnosed
failure of each required moment, including window-id readiness. Demonstrate
delayed callback/readiness controls plus a complete clean-HEAD live positive
with all three screenshots, and the transparent negative rejected for content.
Do not remove the required moments or count requested screenshots as completed.
This is separate from the carried unbounded click-wait debt.

## Acceptance, Spec delta, evidence reuse and route

| AC | Current result |
|---|---|
| 01 | R8-02 doc inconsistency closed; R8 committed export/manifest evidence retained on unchanged assets/deps/game paths |
| 02/03/04 | Existing bounded pass retained; shared/squash/seam targeted controls newly pass; game/render oracle unchanged |
| 05/06 | Partial: R8-01 incomplete discrimination; R10-02 actual canonical evidence failure |
| 07 | Fail: R8-01 + R10-01/02 |
| 08 | R8-02 affected current-rule and research-summary correction passes; I1–I3 remain pending |
| 09 | Needs fix from this independent affected review |
| 10 | Merge-owned pending |

Canonical Spec inputs and SD proposals are hashed/copied here. No changed
canonical input alters the reviewed experimental/production boundary. The
three proposed paths and GOAL-008 metadata remain appropriate, but current
acceptance gaps and the carried SD-03 counter-only wording prevent deposition
approval; no canonical Spec/live ledger edited. Unchanged cache/lifecycle,
measurement, delivery, game and boundary evidence is reused only within R8's
identified scope: no modifications to those implementations, lock, assets,
traces or test configuration in 814..1fa. New checks cover the affected Iced
analyzer/canonical path and changed research/source note. No 107-test/full
probe matrix is claimed for this review; no merge-owned/root tier ran.

Return to work in the same tree. Keep R8-02 and prior source-height/committed
delivery closures. Fix the open R8-01 invariant and two actual path failures,
extend diagnosis to both geometric axes, real constrained layout and callback
completion, preserve distinguishing positive/negative controls, commit and
request affected re-review. No new scope/deferral/oracle weakening authorized.

Plan `executing`, **6/11** complete (T-01..T-06); T-07/T-08/T-09/T-10 open,
T-11 pending. Final pass must precede merge-owned frozen gates, full live
positive/negative, consumer relocation, migration/write-surface handoff
clearance, Spec/ledger deposition, archive and guard cleanup. No implementation
edit, commit, merge, archive, cleanup, new chat/agent or cross-chat message.
