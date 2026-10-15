# PLAN-783 r1 — final independent review R1

2026-10-15. **Outcome: needs_fix; return to work in the same tree.**
Reviewer: this Codex review context, independent of the implementation history;
no delegated agent or other chat was created/contacted. No implementation edits.
Review skill SHA256 `0D68BA05276AFF05A143C8F27010EE02B0A9BDED14C27A0133D3A774C20F0CF3`.

## Frozen baseline

- Worktree `D:/autostack/.wt/lang-783/auto-lang`, branch `plan-783-dev`.
- Reviewed commit `62d93dae82546e5685f164382345ad4050690fa2`;
  base `ec934b0006acb9b9838def82d0fcaf438ae0153f`; clean at entry and exit.
- Shared Plan read on master, last Plan commit `6827943464cde767ee216c43ada364e642a38835`,
  revision 1, pre-review SHA256 `D48A93F500D3D228AE0957ABA524769D070BDF638B4F0E5B43ABBF9A3F14417B`.
- Dependencies at reviewed commit (Git tree IDs): ac-core
  `bccb77d30131182129213f234cdcc63f40855fcc`; auto-cache
  `c4524240cf1a0e6c50a9176fe673cdf2527076e9`; shim-metadata
  `6ffceaaa1761cfe722e8c8cf39583a71a313aa12`.
- Probe Cargo.lock Git blob `93582090320f0b07fd6c12d83c15f867ec1d3366`;
  verification script blob `03b42851219caf5718fcc242799a49cb86e5bfe2`.
- Current canonical Spec SHA256: auto-ac/project
  `9336DFD87258DBBB037F2A94C97C40BB7D71F4CC346F51172C7D880E2C9363A8`;
  auto-cache/project `FA52F8DF1F5D8FB9671AE3762DE6EC2CA39EAAD607DE07411E51D28F23DBA8A2`;
  auto-hir/stage-contract `581D0C22408DFCB18667009158ECC364D08AE76F5C4CE3D8EFEB6DCBE9A62C5C`.
- Frozen SD copy: [review-r1-spec-delta.md](review-r1-spec-delta.md), SHA256
  `869089335BEDB72A711A263CEAB75B36CD443864FBA64472FB4B78DE81D63FE3`.
  Canonical Specs and live ledger were not changed.

## Verification and acceptance map

Executed in the implementation tree:
`powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-artifact-probe.ps1 -Scope acceptance`.
It exits 0, but raw totals are **66 passed, 1 ignored**, not 76/0.
The ignored case is the explicitly merge-owned 512 load, which is legitimate pending
closeout; `perf_full_report_contract_when_provided` also silently returns without
checking a report because the acceptance script supplies no report environment.
Raw result: [review-r1-acceptance.txt](review-r1-acceptance.txt).

| AC | Tasks / evidence examined | Result |
|---|---|---|
| AC-01 | T-01/02; registry, identity, codecs, lib/admission/identity tests; R2/R3 controls | partial: bounded HIR restore and reverify work; consumer integrity fails F-04/05 |
| AC-02 | T-03; both builders/views, 6 layout tests; actual 128MiB builds and OS private-memory assertion rerun | partial: normal equality/budget checks pass; malformed/corrupt reader fails F-05 |
| AC-03 | T-04; all four core native oracles fresh/object restart/HIR restart; scalar and invalidation controls | partial: nominal true pipeline passes; CAS and current diagnostic binding fail F-04/11 |
| AC-04 | T-05; reachability/concurrency/recovery/budget/boundary tests, R1/R4/R5/R6 | fail: F-01/02/03/06/07/08 |
| AC-05 | T-06; quick six-scenario execution rerun, committed 30-sample report inspected | partial: sample timings exist; active-GC attribution and required resource evidence fail F-07 |
| AC-06 | T-07; full diff/dependencies, 5 dep_audit tests, SD-01..05 | partial: scope isolation passes, SD claims need correction with F-06/07/08/09 |
| AC-07 | T-08; this review and format check | fail: open findings; no final pass |
| AC-08 | T-09 | pending merge, not executed |

No root cargo tier or docs_gen ran: diff has zero crates/** or ac-core/src changes.
`cargo fmt --manifest-path experimental/ac-artifact-probe/Cargo.toml --check` exits 1.
Repeated `shim-metadata/src/classify.rs:213` unused-doc-comment warning comes from an
unchanged dependency; record as baseline, not a probe regression.

Supplemental consumer source [review-counterexamples.rs](review-counterexamples.rs)
was compiled against the frozen release rlib without modifying the implementation.
Replay: `powershell -NoProfile -ExecutionPolicy Bypass -File docs/plans/attachments/783/replay-review-r1.ps1`.
Raw output: [review-r1-counterexamples.txt](review-r1-counterexamples.txt).
The caught layout panic is intentional evidence; the consumer continues to the other controls.

## Open findings

### F-783-R1-01 · P1 · AC-04 / T-05 — newly acquired live lease/root can be deleted

`src/store/gc.rs:126–239` snapshots seeds/closure once, then deletes candidates without
rechecking liveness atomically with lease/root registration (`store/mod.rs:544,580`).
R6 publishes 150 dead chunks, starts GC, waits until its first deletion, then successfully
acquires a lease and hard root for the still-readable last candidate. After GC:
**lease_live=true, target_readable=false, deleted=151**. This violates §5.4's explicit
lease-acquisition/GC interlock; TTL truth alone does not solve acquisition races.
Repair with shared publication/liveness coordination and a final atomic liveness decision;
cover lease acquisition, root replacement, and required-closure publication while GC runs.
The existing reader/GC test pre-roots its live project and cannot distinguish this defect.

### F-783-R1-02 · P1 · AC-04 / T-05/06 — perf deletes unowned directory contents

`src/perf.rs:108` unconditionally `remove_dir_all(root)` before any ownership/canonical
boundary check. `node perf --root <existing directory>` therefore recursively erases it.
Independent disposable control: create a new review-owned temporary directory containing
`external-sentinel.txt`, without any marker; run `node perf --root <dir> --samples 0 --out <temp.json>`.
Observed **marker-before=False, sentinel-after=False**, process exit 0.
Zero samples only makes this control fast; deletion occurs identically with 30.
Require a fresh exclusive owned directory or validated ownership before cleanup, reject
non-owned/reparse paths, preserve outside sentinels. Audit entrypoints, not only store GC.

### F-783-R1-03 · P1 · AC-04 / T-05 — successful republication stays permanently missed

GC leaves a `chunks.state='deleted'` row. `store/mod.rs:407` uses `INSERT OR IGNORE`, so
republication of the same bytes never restores live state. R1: put → readable control →
GC deletes 1 → identical put returns Ok → **readable=false**. Blob bytes are restored but
metadata remains deleted. Recovery tombstones have the same issue. Transition the chunk
state atomically under the same delete/publish coordination and replay repeated put/GC/get
cycles, including recovery and a second key sharing identical content.

### F-783-R1-04 · P1 · AC-01/03/04 / T-02/04/05 — CAS address is never validated on read

`store/mod.rs:492–539` validates a frame's self-checksum but never compares its framed-byte
ChunkId to the requested CAS address. R2 copies a different valid same-kind chunk over the
first chunk's file; lookup of the original key returns **actual=other, expected=first**.
HIR semantic verification also cannot prove that a valid foreign bundle belongs to this key;
object hits go directly to linking. Recompute the domain-separated framed-byte identity
at admission, require its equality to the requested id, and test both HIR/object consumers
with valid-frame substitution as well as stale-checksum bit flips.

### F-783-R1-05 · P1 · AC-01/02 / T-03 — actual layout reader bypasses validation and panics

`layout/mod.rs:491–510` seeks past the frame header and trusts raw payload; only an optional
test helper calls unframe. R3 flips record kind in the page: helper rejects checksum, but
**LayoutView::record returns kind=99**. A checksum-valid A record with offset=65535,len=8
then **panics at line 577** instead of controlled rejection. B has unchecked offset/length
slices at line 598; unknown layout names take the B branch. Validate identity/framing and
manifest/schema/layout/field ranges in the actual bounded page-load/read path. Add direct
view controls for corrupt frame, valid-checksum bad offsets, short B records, unknown
layout, and legal siblings. Calling the helper before some tests is not an admission gate.

### F-783-R1-06 · P2 · AC-04/06 / T-05/07 — reclamation misses committed orphan rows and trash

R4 publishes winner then conflicting payload: conflict commits a chunks row without an
artifact. GC candidates come only from artifacts; recover calls every chunks row committed.
After forced full GC plus exclusive recover, **one unreachable block remains**.
Also place 1000 bytes in trash (the documented failed-delete state), call recover:
**trash remains 1000 bytes**. No recovery pass scans trash. Interrupted tombstone recovery
only updates chunk state and does not cascade artifact/edge/root rows as normal GC does.
Reconcile all orphan/tombstone/trash states, retry physical deletions and report only actual
freed bytes. Add failure-exit controls proving cleanup/idempotence. D-783-D's claim that
explicit recover frees pending trash and SD-04's complete recovery description are false;
these are in-contract repair work, not approved future pack/compaction debt.

### F-783-R1-07 · P1 · AC-04/05/06 / T-05/06/07 — health/perf oracles do not exercise claimed GC

`node.rs:168` varies source only by `i % 7`: 512 iterations reuse seven source/HIR/object
versions, across three projects. `gc_runs` increments even when `GcReceipt.ran=false` and
there is no deletion/watermark-crossing assertion. A targeted 24×300 replay yields
**gc_runs=6, peak=final=2,669,992 bytes, blocks=29**: all calls are below the frozen 64MiB high
watermark. Repeating the same seven keys cannot demonstrate 512-version reclamation.
`perf.rs:323–363` similarly creates tiny junk with default 64MiB GC, ignores build/GC errors
and the thread join result; it does not record or assert a single actual GC deletion.
The committed GC scenario has only a small store. Thus its 695ms tail cannot be attributed
to deletion I/O on this evidence. Report also hardcodes memory delta=0 in four scenarios,
omits per-scenario I/O evidence and a bound source commit/configuration, and its prose
claims P95<400 while its own GC row is 523.3ms. No latency limit is a Plan requirement.
Use genuinely distinct versions within the original frozen budgets, assert actual high
crossings/ran/deletions/low convergence and peak temporary usage; force real overlapping
reclamation during perf, propagate worker failures, measure required metrics and correct
claims. Large 512/30 reruns remain merge-owned after repair/review; targeted proving
controls must establish that those workloads have discriminating oracles before handoff.

### F-783-R1-08 · P2 · AC-04/06 / T-01/05/07 — same-key producers are rejected, not serialized

`store/mod.rs:398–399` performs one nonblocking lock attempt then returns
`Internal("chunk lock unavailable")`. R5 holds the same chunk lock as a producer and
invokes put: contender fails immediately. There is no wait/retry or documented bounded
busy/retry protocol. Staging is left behind on this exit. V-07's alleged two-producer
test calls two processes sequentially with identical source; differing-content conflict
test is also sequential. No actual overlapping same-key production count is measured.
Add bounded coordination/failure cleanup and actual simultaneous-process controls for
identical and conflicting results, actual production/commit counts, and concurrent
recovery (which currently scans/removes orphan/temp files without producer locks).
Correct the feasibility/SD statement that this evidence demonstrated serialized production
and zero reproduced races. Avoid claiming production BlobStore inevitability from this test.

### F-783-R1-09 · P1 · AC-05/06 / T-06 — aggregator accepts skipped or missing verification

`verify-ac-artifact-probe.ps1:67–72` fails only a parsed 0-passed/0-failed summary; ignored
cases and absent summaries are accepted. In an isolated copy with a stub manifest and fake
`cargo.cmd` placed first on PATH, all 13 families reporting `1 passed;0 failed;1 ignored`
produce **exit 0 / PASS**. All 13 reporting no result summary also produce **exit 0 / PASS**
(PowerShell errors do not terminate because ErrorActionPreference=Continue). Negative
control with child exit 1 correctly produces exit 1. This is a real validator mutation,
not replacement evidence for implementation correctness. Require expected case inventory,
summary presence, pass/ignore accounting with an explicit merge-load allowlist, and a real
committed perf-report contract check at acceptance. Full report check must reject missing
or duplicated scenario identities and failed/absent samples. Correct the 76/0 receipt.

### F-783-R1-10 · P2 · AC-07 / T-02..07 — format health gate fails

`cargo fmt --manifest-path experimental/ac-artifact-probe/Cargo.toml --check` exits 1 with
diffs across the new probe sources/tests (first at node.rs:62). Format all probe files and
rerun the nonmutating check. The unchanged shim-metadata warning is separately baseline.

### F-783-R1-11 · P2 · AC-03 / T-04 — diagnostic output loses current project source binding

`driver.rs::render_diags` calls `Diagnostic::render("", "")` without the current path/text.
Control source in a uniquely named `current-project.at`: `fn main() int { return true }`.
Actual CLI: `ERROR {"error":"frontend: :1:1: error[verify/source.type-mismatch]: expected i32, found bool"}`.
The required current-project diagnostic path/span is absent; V-01 only checks successful
compilations and cannot observe it. Bind diagnostics to the consuming source/snapshot
without contaminating content identity, and test two paths plus restored-payload diagnostics.

## Knowledge delta and routing

SD-01/05's intended separation of identity/ref/trust is aligned with canonical Specs, but
actual admission and lease rules do not yet meet it. SD-02/04 cannot be deposited as verified
facts while the above recovery/measurement/concurrency claims are false. SD draft is an
outline, not final add/modify text with before/after anchored to the current Spec versions;
finalize exact target text/versions after repairs. Existing impact paths are correct;
`touched_goals: []` is justified by no language/self-hosting goal closure. No scope expansion
or Plan revision is requested by these findings: all are gaps in existing r1 requirements.

Next owner: work, same tree. T-01's minimal HIR/native/API feasibility observations stand,
but its concurrency conclusion needs correction; reopen T-01..07 (current_step=0/9), retain
the historical execution receipt, mark AC-07 failed and T-08/09 pending. Repairs must commit
a new baseline and replay affected controls plus siblings before final re-review. Do not
run root tiers/full probe closeout while repairing. Future closeout still requires frozen
512 churn + 30-sample perf, Spec/index deposit and guarded cleanup, after a real final pass.
No merge, archive, worktree removal, or implementation repair occurred in this review.
