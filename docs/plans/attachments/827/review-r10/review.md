# PLAN-827 T07 final repair review R10 — pass (2026-10-25)

**Final code, acceptance and proposed Spec-delta review passes.** This is not
merge approval or completed delivery: frozen merge-owned gates, canonical
deposit, actual landing/archive and guarded cleanup remain pending.

## Independent baseline

Reviewer `/root/plan827_t07_review`, same independently delegated context as
R7..R9, explicitly authorized by the user; never an implementation author.
No invented model/session ID. Reviewed main Plan r3 and latest work/acceptance
matrix, prior findings, actual unique-tree diff and evidence. Worktree
`D:/autostack/.wt/lang-827/auto-lang`, branch `plan-827-dev`; frozen HEAD
`9ae7a7a3b6fa118cd94d6ea053ecc7da6d6bab96`, clean before and after. Repair code
`f58fd3382979f013634cc99712643cc9f7b1294c`; previous review
`cce6139e87629edbb251ded6a714a2c0dd5bdb66`; original base
`369e1c349ea9587907f073aaecd3082e2bb486bd`. Code commit changes only the
verification script; following commit adds the provenance annotation only.

baseline.json.txt binds full HEAD/base, Plan/source Spec/input hashes,
toolchain and dependency/file blobs. dependency-metadata.json.txt binds actual
local package versions/paths; reuse-input-hashes.json.txt binds the actual
worker and reused103747 artifacts. No implementation/other-tree writes,
additional agents, canonical Spec/ledger publication, merge or cleanup.

## F-827-R8-01 residual closes

**Expected-failure detector.** Actual `Test-BenchSummary` now requires
gate_ms0, nonzero exit, only threshold-rejection messages belonging to the two
frozen paths at0.0ms, with both paths present. It no longer treats arbitrary
failed bench output as the intended threshold-zero counter-control.

Independent original R9 replay extracts committed functions through PowerShell
AST. Original unrelated-failure mutation now returns **false** (expected),
while actual103747 gate0 positive remains true. Extraction still accepts one
summary, returns null for absent summary and rejects duplicates. Writer replay
changes only the renamed parameter from `-Args` to `-BenchArgs`; exact same
nonempty text is now retained byte-for-byte at the string level. Both resource
call sites use the new parameter. original-controls.log.txt records exit0.

Independently chosen sibling controls exercise the same production predicate:
both-path positive true; signature-only, edit-only, mixed correctness failure,
mixed resource failure, healthy1000ms threshold, exit0 and missing gate field
all false. Fourteen checks including six genuine/recovered artifact judgments
all agree with expected results. Current103747 normal/gate0 remain accepted;
historical085917 normal remains correctly rejected, not relabeled pass.
See replay-siblings.ps1 and sibling-controls.log.txt. Script selftest also
independently passes its selected-ID, binding, summary and writer controls.
No broad campaign or cargo tier was rerun.

**Command provenance.** Real writer now uses a non-reserved `BenchArgs`
parameter and stores that exact value. Existing103747 summaries deliberately
retain historical `args:""`. Sidecar
`bench-summary-args-annotation-20261025-103747.json` records the cause/fix and
two source-derived invocation reconstructions separately. Checked against
historical script311e81a7f call sites, summary gate_ms/sample counts and the
same-run binding/raw filenames. Historical-source-call-sites.txt retains this
source evidence. The sidecar uses repository-relative output notation; it is a
documented source-derived reconstruction, **not a claim of directly captured
historical OS argv**. This qualification preserves provenance rather than
rewriting the original measurement. Original summary hashes match R9 audit.

No residual in-contract finding remains. Earlier R6 allocation/cleanup and
R7 persisted identity/binary-binding closures stay closed.

## Resource evidence and reuse

Runtime source/dependencies/configuration/input are unchanged by f58fd3382 or
the annotation commit, so103747 remains valid without another measurement.
Worker SHA256 is unchanged:
`0A50E300E1B4BC9725C0DAA16E5502306AF3F455D142EC62FF79E01331782261`.
The existing directly captured normal exit0/empty failures, frontend peak
211222528 bytes, overall217563136, child211595264, residual6340608, both
memory targets true, P95(single-body-edit)=966.2128ms and
P95(signature-conflict)=38.0514ms retain R9's verified scope. Gate0 directly
captured exit5 and exactly the two expected zero-threshold failures remains
green as a negative control. Frozen base input is273575B/FNV431bff0efbaabfec;
real edit/conflict revision hashes are retained in raw.

The source-recovered085917/091045 summaries still match their original console
lines. Failed085917 stays retained;091045 normal exit0 remains explicitly
inferred from continuation, not called directly captured. Current103747 has
direct status capture, so that inference does not replace required evidence.
Formal-session report retained in review-r8/reused-formal-report.json.txt
separately passes30 samples, P95=331.9732ms, correct/structural/gate true and
actual worker/input identity. Current predicate accepts the valid summaries;
earlier finite store/churn evidence is not repeated or widened.

## Complete acceptance disposition

| AC / tasks | Final review evidence and result |
|---|---|
| AC01 / T01,T02 | pass: R7 full inventory/SEM audit and R3 independently replayed inventory axes retained. Scanner, input identities, stdlib/member/entry/gap rules unchanged. No future unsupported member is counted as source success. |
| AC02 / T03,T04 | pass: R7 independently ran actual file/native/fresh-hit-new-worker and scalar/trap cases; byte identity, typed0/0 and precise trap, actual shared source pipeline/recipe reviewed. R5 formal parent/corpus evidence retained with unchanged producer/parent code. |
| AC03 / T02,T04 | pass: scalar semantics/order/false-only/required trap/unsupported and SEM boundaries audited and runtime tested in R7; no AVM fallback or new language semantics. |
| AC04 / T03..T05 | pass: R7 V04..08/V10 consume-gate/identity/current grant/span/root/lease source mapping and relevant thin runtime tests; inherited finite code/input evidence same. No new executable trust/cache grant introduced. |
| AC05 / T04..T06 | pass: actual bounded allocation/failure cleanup independently closed R7, current same-code resource/peak/time evidence audited R9 and retained above; expected-failure validator/provenance now closes R10. No memory/time promise waived. |
| AC06 / T01,T03,T05,T06 | pass: exact required IDs, scope inclusion, skip/missing-tool/failure propagation and parent oracle controls previously audited; same-code recipe/817/818 compatibility scope retained. This repair's summary/binding/detector controls independently pass. |
| AC07 / T07 | pass: independent full AC/V/SD audit completed across R7..R10 with explicit scoped reuse, all in-contract findings closed, frozen code clean and health evidence applicable. |
| AC08 / T08 | pending merge-owned, intentionally uncompleted; gates/deposit/landing/archive/index/guard receipts still required. |
| AC09 / T09 | pass: runtime/detector and30-edit structure/time identities from R7/R8; actual per-run before/after raw, failure preservation, precise refusal/dependency/lifetime controls verified; producer/test inputs unchanged. |

Reuse is scoped to the R7/R8/R9 reports and their actual coverage, not a claim
that every inherited upstream test was rerun. Code/dependency/config/input
identities inspected again; only script gate/recording and annotation changed
since the last runtime evidence. No omission, unauthorized deferral or
workaround found in this remaining scope. Deferred full-language/product
features remain explicit non-goals, not accepted debt for unfinished criteria.

## Frozen normative delta and health

SD01..06 frozen in spec-delta-frozen.md; target versions/hashes bound in
baseline.json.txt. SD01/04 limited experimental source-vs-fixture/product
layering; SD02 actual source/snapshot/oracle/resource/evidence contract;
SD03 cache/current authority/recovery; SD05 actual existing
ac-av/core-run-v1 reuse; SD06 D828 identities/lifetime and bounded R6 fault
semantics all match current behavior. SD02 now explicitly states the already
required threshold-zero failure distinction and exact invocation/separate
historical correction rule. This clarifies approved validation, not a new
runtime/language/cap/ABI contract or Plan revision. GOAL-002 only receives the
limited contribution; full VM replacement remains open. No canonical publish.

No Rust changed since previous warning-free checks; relevant R7 core check,
scoped33 runtime tests and work's changed-file formatting remain applicable.
Current script parses, all finite controls pass, diff-check clean; no debug,
fallback or unrelated production code added. No whole cargo t/tv/tt/tb/tu/taa,
performance matrix, acceptance suite or heavy family was run in this review.

## Next and pending merge-owned gates

Outcome **final pass**; mark T07 complete, Plan **reviewed8/9**, retain T08
unchecked. This permits handoff to merge, not landing itself. Merge owns frozen
bare cargo t + compiler surface tv, independent workspace complete finite
contract and actual affected consumer gates; tt/tb/tu/taa only on real trigger
surfaces (none added here), tf due/batch on main only. Then apply frozen
SD01..06/ledger/index, actual landing/archive and receipts, wt-guard clean
before unique-tree cleanup. A semantic change or gate failure returns affected
work/review to its proper stage. No merge/archive/cleanup performed here.
