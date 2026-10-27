# PLAN-829 affected final review R18 — PASS

2026-10-22 (Asia/Shanghai). Plan revision1, independent review chat separate
from implementation; no new agent/chat. R17 is the work receipt. Existing
worktree `D:/autostack/.wt/lang-829/auto-lang`, branch `plan-829-dev`.
Frozen commit **ddbde3e62857e6bbfc8b780c6182ee1bb7d4ef74**; previous reviewed
commit ae40d6052cdb7716c7cabe9e3f7e9edb1fe92d74; approved diff base
fdb1fad2ce7afa124adfa3080323496df384e670. Entry Plan SHA256
607878C51DC8EE26C6346744AA8B42393425DA652F9AED37A5B23CA54298265A.
Same HEAD, branch and clean implementation tree verified before verdict.

**PASS. F829-R14-01 and F829-R14-02 are CLOSED. No new blocking finding.**
This is acceptance/code/Spec review, not delivery or merge authorization without
the pending closeout gates below. Plan becomes reviewed10/11; T-11 remains open.

## F829-R14-01 closure — actual normal failure exit

The no-ID terminal branch now returns `iced::exit()`; known-ID still returns
`window::close(id)`. The new permanent lib test exercises the original boot →
Tick deadline → phase99 → finalizer path and checks an actual task is scheduled;
it also retains the known-ID sibling. Source inspection confirms the task is
the runtime exit effect, not an arbitrary nonempty task.

Independent same-source replay retained the preceding clock and readiness
controls, then observed terminal behavior:

| Case | Current observation |
|---|---|
| Initial / after_button / resized pending | Each matching timeout at181 Tick events |
| Frozen resized | wall301, graphics120, phase99; timeout does not depend on animation |
| Valid delayed resized callback | phase6, no failure, graphics still120 |
| Missing window-ID boot | window-id-timeout at302 deliveries (age301 >300) |
| Missing-ID finalizer | finished=true, failure report/verdict1, **exit task units1** |
| Further600 controller Tick events | No repeated finalization or duplicate exit tasks |
| Known-ID finalizer sibling | close task units1 |
| Window becomes ready before deadline | phase1, initial pending, screenshot task units1 |

[Raw controller/test output](replay-output.txt), [driver](timeout-probe.rs),
[private entry](timeout-entry.rs), [replay](replay.ps1).

The decisive native replay uses the original boot/update/view/case-finalizer,
real Iced window/rendering and33ms timer, withholding only Resize event delivery
(the sole source of the missing ID). Both copied modules share the same globals.
It now records the named failure, writes the report, **returns normally from
the native event loop** (`native_loop_returned` marker), then returns the actual
failure verdict **exit1**. No panic, manual close or external termination.
This closes the R16 exit101/present-panic counterexample and the zero-task
controller counterexample. [Exit/status](live-output.txt),
[stderr/normal-return marker](native-stderr.txt),
[failure report](native-failure-report.json.txt),
[native driver](native-missing-id.rs), [bounded runner](native-replay.ps1).

The20s outer bound was only a safety limit and did not terminate the process.
This verifies the declared missing-ID failure model, not a universal hard
wall-clock/GPU-preemption guarantee or a claim that normal hardware loses events.
R16's old raw zero-task/exit101 observations remain the distinguishing red
control; the production change is the exit effect plus its permanent test.

## F829-R14-02 closure — consistent dated historical correction

result.md:117–121 now marks the mislabeled R13/1ff reports [HISTORY], explicitly
NOT commit-bound, with dated CORRECTED2026-10-22 and the finding ID. It cites
R14's actual36 and R16's actualae40 live evidence. E21's existing correction
agrees. Remaining1ff occurrences identify the historical review baseline or
the corrected raw header; none certifies that pair as current bound evidence.
Old raw receipts remain preserved. [Exact committed repair diff](repair-diff.txt).
No build.rs root cause, intentional misstatement or rewritten old report header
is asserted. Current ddb live evidence below is independently verified.

## Current real-window controls and scoped health

Unchanged production `graphics-probe` was rebuilt with the frozen lock and run:

- [Positive](live-positive/report.json.txt): **exit0**, initial / after_button /
  resized all **content ok=true**, real button observed, actual resize accepted.
- [transparent_blit negative](live-transparent/report.json.txt): **exit1**,
  initial / after_button / resized all **content ok=false**, click observed.
  [Successful negative replay](transparent-retry-output.txt).
- Both fingerprint **ddbde3e62857**, lock
  6cd345763c2dd44814db4a367fe636010478f9745577b8b06e3c2c573b08e988,
  asset set983a78f540d9c9b02de2fbe09cad52c162a84ffe92b6458b4fc1cd36845a6d3a.
- **58 targeted tests passed**: composite17 + delivery2 + squash17 + seam5 +
  shared16 =57, plus the one permanent missing-ID lib test. It selects1 test,
  with28 other lib tests filtered out; no whole-lib/workspace pass is claimed.
- fmt/check--tests/repair diff-check all exit0, no compiler warnings.
  [Health and frozen baseline](health-and-baseline.txt),
  [final identity/no leftover probe process](final-identity.txt).

Transparent negative first attempt was **not** counted as a full successful
negative: initial was correctly rejected, but dispatched input was not observed
(presses0), so bounded click-timeout at901 ticks ended it with later screenshots
missing. [Original raw report](live-transparent-first/report.json.txt),
[stderr](live-transparent-first/stderr.txt) and the failed three-moment assertion
in [the run status](live-first-run-status.txt) remain intact. A single focused retry with unchanged source,
lock, assets, case and sabotage setting completed the real input path and all
three screenshots. The exact external input-delivery cause is not established;
this is recorded as an incomplete interactive attempt, not silently counted
as proof or alleged to be caused by the exit repair. Positive plus the complete
negative retry supply the required current evidence. [Retry entry](transparent-retry.ps1).
The retry validates, rather than removes, the three-moment assertion.

## Reuse boundary and source identity

Only two AGP files changed sinceR16: the terminal branch/new lib test and the
result summary. The analyzer is byte-for-byte identical after newline
normalization (**13414 characters**), verified directly against ae40 Git source.
[Analyzer identity proof](unchanged-analyzer-proof.txt). R16's19/19 perimeter
mutations, R8/R10 axis/clip controls and genuine initial/button/two-resize
accepted controls therefore retain their recorded pass. Their inputs/hashes
were checked again ([input hashes](reused-input-sha256.txt)); they were not
re-executed or added to the58 count. The17 permanent geometry tests were rerun.

The review's entire current iced_spike.rs prefix is **65936 bytes**, SHA256
7B5F3A12710E3C3A3DCC1765B9A17583C86A04C94C17FBDE4FD6DD1C64254E01;
the review copy only appends the private include. The case copy is completely
unchanged, SHA256
BB52114FB6CC687EF09ABBCE7F47E90888ECAAC9C3B0BDEB453FB9ED4D7AD82D.
[Prefix identity](source-copy-identity.txt), [build/source identities](build-identity.txt).
No original function body was edited. Native failure replay changes only its
subscription to inject the accepted missing-ID model; normal controls use the
actual unmodified production CLI and subscription.

## Acceptance and Spec delta

Unchanged dependencies remain wgpu27.0.1/winit0.30.13/iced+iced_wgpu0.14.0/
iced_winit0.14.1/stock iced_widget0.14.2; manifests, lock, assets, game/source
oracles, renderer/model, delivery dependencies and verification configuration
are unchanged. Prior source/game/audio/cache/lifecycle/measurement/consumer
evidence is reused only within the R7/R8/R10/R12/R14/R16 recorded boundaries.
This affected review does not assert a new full matrix execution.

| AC | Disposition and evidence |
|---|---|
| 01 | PASS retained: independent dependency/asset/source closure unchanged; delivery2 rerun |
| 02 | PASS retained: common model/device contracts unchanged; shared16 and seam5 rerun |
| 03 | PASS retained within frozen Bevy profile; game/input/oracle/renderer unchanged |
| 04 | PASS retained within frozen Godot profile; squash17 rerun, y0/death/rotation fixes unchanged |
| 05 | PASS: lifecycle prior evidence retained; deadline/failure finalization now independently closed |
| 06 | PASS: prior measurement bounds retained, failure/skip not passed; current fingerprints/content controls and corrected history verified |
| 07 | PASS: current true-window positive/transparent three moments, resize, same-device controls and native missing-ID normal failure exit |
| 08 | PASS: consistent delivery history; IH owners/entries and I1–I3 pending boundary retained |
| 09 | PASS: independent affected review, unchanged evidence audited, current AC/SD/health verified |
| 10 | PENDING: merge-owned closeout, not accepted as delivered |

Canonical overview/UI/repository-boundary/goals and skill inputs are unchanged
fromR16. Design45 changed: its new§8 explicitly says CAP/AutoShader does not
expand829/830 or turn Rust/Iced probes into production capability. Read and
reconciled; no Plan revision/scope change. [Input hashes](spec-input-sha256.txt),
[frozen Design45](design45-input.md).

SD-01/02/03 proposals are frozen here with [hashes](spec-delta-sha256.txt).
Exact new paths remain docs/specs/auto-graphics/{project,prototype-profile,
integration-contract}.md; supersedes=[], touched_goals=[GOAL-008]; current
canonical module is absent. Proposed independent prototype facts/support
profile and pending integration gates agree with the accepted implementation
and existing repository/UI Specs. No production/current capability is inferred.
SD-03's counter/GDI sentence describes the older skipped-final-draw control;
it must not be read as claiming that counters detect transparent fragments.
The latter requires and now has the independent composite-content gate.
Previously recorded nonblocking R5-03 deposition wording remains a merge
editorial item: include this distinction/current screenshot evidence when
depositing the canonical text, without weakening either gate. This is not a new
implementation requirement or an approval to publish a counter-only oracle.
Canonical Specs and the live ledger were not changed during review.

## Route and pending closeout

T-01..10 completed, **reviewed10/11**. Both remaining findings closed; no new
blocking issue or unauthorized deferral. Keep the sole Plan worktree.

**Merge-owned gates remain PENDING**: on the approved frozen revision, run the
independent AGP workspace applicable full tests/probe matrix (ten cases) and
real independent-consumer relocation; obtain the required migration/write-surface
handoff clearance; deposit canonical Specs/derived ledger; archive and guarded
cleanup. Experimental-only scope does not trigger root crates test tiers or
docs_gen. Any implementation/dependency/config/input change invalidates affected
evidence and requires the corresponding re-review/gates. No merge or claim of
delivered production integration is authorized by this review alone.

No implementation edit/commit, root t/tv/tt/tb/tu/taa/tf/docs_gen, full AGP
suite, canonical publication, merge/archive/cleanup, new agent or external
message was performed. Reversible review artifacts and main Plan/debt bookkeeping
are the only writes.
