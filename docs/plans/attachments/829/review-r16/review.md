# PLAN-829 affected final review R16 — needs_fix

2026-10-22 (Asia/Shanghai), Plan revision 1. Independent review chat, separate
from implementation; no new agent/chat. R15 is a work receipt, not an independent
review. Existing tree `D:/autostack/.wt/lang-829/auto-lang`, `plan-829-dev`.
Reviewed commit **ae40d6052cdb7716c7cabe9e3f7e9edb1fe92d74**, prior review
36f520c104ad99e2eaed2e22a5a3f0bb93195391, approved diff base
fdb1fad2ce7afa124adfa3080323496df384e670. Entry Plan SHA256
BF24AF1BB979038A0B075003699D5B5CCA59D086C6CCC9A7C56CF61B06094CAE.
HEAD/branch/clean implementation tree confirmed at entry and verdict.

## Outcome and verified repair

**Needs fix: F829-R14-01/P2 and F829-R14-02/P3 are partially repaired.**
The clock root cause is fixed. The remaining items are terminal failure handling
after the new window-ID deadline and a missed historical assertion in result.md.
They retain their existing finding IDs; the original clock counterexample is
explicitly closed rather than reported as still failing.

- All phase ages now use `wall_tick`, incremented on every Tick independently
  of animation freeze. Resize event entry also records wall_tick.
- Original-function controller replay: initial/after_button/frozen resized
  each diagnoses its matching timeout at **181** Tick events. Frozen resized
  keeps graphics tick120 while wall reaches301, phase99. Active-clock sibling
  does the same. Actual delayed CompositeShot at179 ticks advances phase5→6
  without a failure and leaves animation frozen. [Raw](timeout-final-output.txt).
- Window readiness now diagnoses **window-id-timeout** after302 deliveries
  (first delivery starts age; age301 exceeds300), instead of remaining phase0.
  A Resized event before that deadline advances to phase1 and issues the real
  initial screenshot task. Deadline transition itself is verified.
- E21 now explicitly downgrades the mislabeled R13/1ff reports to work context
  and cites R14's actual36 runs. This part of the evidence correction is sound.
- Independently ran current release **unchanged graphics-probe** with normal
  event subscription: [positive](live-positive/report.json.txt) **exit0**,
  all three moments content ok=true; [transparent_blit](live-transparent/report.json.txt)
  **exit1**, all three content ok=false. Both bind **ae40d6052cdb**, the unchanged
  lock and asset set. [Raw status](live-first-output.txt). Normal-path live
  closure remains valid, including frozen-frame controls and actual resize.
- **57/57 targeted tests** (composite17, delivery2, squash17, seam5, shared16),
  [raw](targeted-tests.txt); fmt/check--tests/diff-check exit0, no warnings.
  Complete-perimeter **19/19** mutations and all R8/R10 placement/overflow
  holdouts still reject; genuine initial/button/both resize positives pass.
  [Raw geometry and tests](replay-output.txt), [inputs](perimeter/).

## F829-R14-01/P2 — timeout is diagnosed, but missing-ID failure has no normal exit

Affected AC-05/06/07/09, T-07/T-08/T-09/T-10. Existing obligations are bounded,
diagnosed failure and normal host shutdown, including the new readiness wait.
This is incomplete closure of the declared failure state, not an added universal
GPU-preemption or hard wall-clock guarantee.

Established path in iced_spike.rs:

1. With no window-ID event, phase0 now records window-id-timeout and enters99
   at1411–1418. This part is correct.
2. Next Tick executes the actual phase99 finalizer at1560–1583: finished=true,
   report written, FINAL_VERDICT=1, then `window_id == None` returns
   **Task::none()**. No exit action is scheduled. Future Tick messages are still
   subscribed, finished prevents any retry, and a later Resized event merely
   records an ID without reopening finalization.
3. The known-ID sibling returns `window::close(id)` (one effect). The no-ID
   sibling returns zero effects. The outer case can return FINAL_VERDICT only
   after `iced_spike::run()` returns; writing the report/atomic is insufficient.

Deterministic controller result, using original update/canonical_step/finalize:

| State | Observed |
|---|---|
| Boot, withhold ID event,302 Tick | phase99, window-id-timeout |
| Next Tick/finalizer | finished=true, report exists, verdict1, **task units0** |
| 600 additional Tick events | phase99/finished, every task units0 |
| Known-ID terminal sibling | finished=true, **close task units1** |
| Ready-ID legitimate control | phase1, pending initial, screenshot task units1 |

[Driver](timeout-probe.rs), [entry](timeout-entry.rs),
[raw](timeout-final-output.txt), [replay](replay.ps1).

Native failure replay also uses original boot/update/view, with **only Resize
event delivery withheld** (the sole source of window_id). It retains the real
33ms timer, native Iced window and real rendering. The matching case finalizer
is copied unchanged and bound to the SAME copied globals as rendering. After
the named timeout and failure report, it **does not return normally**: it panics
in Surface::present, `Parent device is lost`, **exit101**. There is no
`native_loop_returned` marker. The20s external observation bound did not kill it;
the process exited by panic. [Native driver](native-missing-id.rs),
[bounded runner](native-replay.ps1), [exit](native-final-output.txt),
[stderr](native-stderr.txt), [report](native-failure-report.json.txt).

Do not call this native observation a20s hang: it actually crashed first. The
GPU error's underlying cause is not established, nor asserted to be caused by
Task::none itself. The confirmed controller defect is the absence of a normal
termination action in precisely the state the new deadline handles. The native
failure adds evidence that writing a report is not equivalent to clean case
completion. No claim that normal hardware ordinarily loses Resize events is
made; missing-ID delivery is the failure model the repair explicitly promises
to handle. Normal unchanged-CLI controls above remain green.

Closing oracle: on the resulting commit, missing-ID readiness must diagnose
failure AND finish the native case through its normal cleanup/return path with
the intended nonzero verdict, without relying on a later resize, manual close,
panic or external kill. Retain known-ID close, all screenshot deadlines, valid
delayed completion and actual normal/transparent controls. Implement the exit
strategy in work; this review does not prescribe an unverified workaround.

## F829-R14-02/P3 — result.md still certifies the invalid1ff live binding

Affected AC-06/08/09, T-09/T-10. E21's correction is present and correct, and
current ae40 live evidence is valid. However, result.md:113–118 still says:

> live positive/negative bound to fingerprint `1ff4ee67e061`.

It refers to the same E21/R13 pair now explicitly downgraded to work context.
result.md is unchanged by ae40. R14 explicitly named E21 **and result.md** in
the closing request; correcting one leaves mutually contradictory delivery
documents. Original1ff source/report contradiction is preserved in
[R14 proof](../review-r14/prior-format-proof.txt).

Closing oracle: retain raw historical reports, add the same dated correction to
this summary, and point to actual36/ae40 commit-bound evidence. Search active
delivery assertions, rather than only the evidence-notes paragraph; historical
mentions with clear dates/disclaimers may remain. No build.rs defect or intent
is alleged. This finding concerns inaccurate documentation, not current ae40
fingerprints.

## Probe validity and reproducibility

Full iced_spike.rs original prefix **63788 bytes**, SHA256
B881D3C9B7BFDDB87EEEC53CB3FF1B8BD8B2FCE738FD623B537ED97479DA8A28;
review copy appends only `include!("timeout-entry.rs")`. Original case file
SHA256 BB52114FB6CC687EF09ABBCE7F47E90888ECAAC9C3B0BDEB453FB9ED4D7AD82D
equals its entire review copy. [Prefix identity](source-copy-identity.txt),
[source/executable identities](build-identity.txt). No production function body
is edited; private entry only exposes state and injects the declared event gap.
Driver module aliases keep both copied modules on one evidence/device state.

The first native attempt reused the library case against a separate copied
spike and was invalid for native finalization. Its raw GPU error is preserved
[here](native-first-stderr.txt), not counted as product evidence. The corrected
same-state driver reproduced the failure; the final run records exit101 above.
Two probe compile errors (missing env!/explicit serde_json extern) were fixed
in the harness only; raw compilation outputs remain. They are not product
warnings or failures. Initial controller replay's zero-task result was confirmed
again with the corrected same-state case, in timeout-final-output.txt.

Fresh reproduction: run replay.ps1 for targeted tests/controller/geometry and
native compilation; native-replay.ps1 for bounded native failure observation;
live-replay.ps1 also runs the actual release positive/transparent pair. The
scripts intentionally reproduce/observe the open defect; their outer exit0 is
not an acceptance PASS. See actual task/child/report assertions and native exit.

R15 raw [positive](work-live-positive-report.json.txt) and
[negative](work-live-negative-report.json.txt) were preserved. Negative has only
initial content data and missing later screenshots; its exit1 alone does not
prove all three moments. R16 independently supplied the full three-moment pair;
no new finding is assigned for that limited work receipt.

## Acceptance, Spec delta and routing

Lock SHA256 remains6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988;
wgpu27.0.1/winit0.30.13/iced+iced_wgpu0.14.0/iced_winit0.14.1/stock
iced_widget0.14.2. Repair diff is two AGP files only. Spec/skill/design hashes
are unchanged fromR14 ([frozen hashes](spec-input-sha256.txt)). Three SD proposal
texts are copied here unchanged with hashes. Exact new Spec paths/GOAL-008 and
experimental versus I1–I3 production-pending boundaries remain appropriate.
Carried SD-03 counter/GDI explanation still needs alignment before deposition;
no canonical Spec/live ledger published or new semantic contract approved.

| AC | R16 disposition |
|---|---|
| 01/02/03/04 | Prior scoped pass retained; code/config/inputs unchanged; shared/squash/seam rerun |
| 05/06 | Partial: repaired clock, remaining failure exit and evidence summary |
| 07 | Normal/transparent/resize verified onae40; missing-ID terminal failure needs fix |
| 08 | Boundaries retained; contradictory historical summary needs correction |
| 09 | needs_fix |
| 10 | merge-owned pending |

Unchanged game/oracle/assets/cache/lifecycle/measurement/delivery evidence is
reused only within prior recorded coverage, since only Iced clock/readiness and
E21/E22 notes changed. No full AGP or root tier was run or claimed. The57 tests
are not permanent controller deadline/termination tests: the existing screenshot
barrier test covers readiness classification, not the normal cleanup action.
Work's clock diagnosis was sound; its oracle adjustment still stopped before
checking the new terminal branch and missed the parallel result summary.

Return to same-tree work with both existing findings PARTIAL/OPEN. T-01..06
remain complete; T-07..10 open, **executing6/11**. T-11 and final-pass-dependent
merge-owned frozen AGP tests/probe matrix/real consumer relocation, migration
handoff clearance, Spec/ledger deposition, archive and guarded cleanup remain
pending. No implementation changes/commit/merge/archive/cleanup, root test
tiers, new agent or external message performed by this review.
