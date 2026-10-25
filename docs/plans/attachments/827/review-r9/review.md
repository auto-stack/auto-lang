# PLAN-827 final repair R9 — needs_fix (2026-10-25)

Reviewer `/root/plan827_t07_review`: same independent context as R7/R8, no
implementation participation. Plan r3, unique tree
`D:/autostack/.wt/lang-827/auto-lang`, branch `plan-827-dev`, clean before/after.
Frozen HEAD `cce6139e87629edbb251ded6a714a2c0dd5bdb66`; repair code
`311e81a7f2a0f6f2043a965212b10e5e798fb06a`; previous review
`ebced8cd5e5f26e609062acdcb08808452af5e20`; original base
`369e1c349ea9587907f073aaecd3082e2bb486bd`. Exact input/Spec/tool/dependency
hashes and file blobs recorded in baseline.json.txt. The repair changes only
the verification script; subsequent commit adds evidence only. No production
executor, source input, dependency or test configuration change. No new agents,
implementation edits, merge or canonical publication.

## F-827-R8-01: retention/measurement closes, bounded validator/provenance residual remains

The original missing actual summary/peak/exit data is now supplied. Independently
read103747 summaries/raw/binding: gate0 exit5, exactly two0ms threshold failure
messages; normal exit0, no hard failures, single-body-edit P95=966.2128ms,
signature-conflict P95=38.0514ms. Normal frontend peak=211222528 bytes,
overall=217563136, child=211595264, residual=6340608; both targets true.
The gate0 peaks/targets likewise exist. There are18/180 raw rows; the varying
source hashes on edit/conflict scenarios are retained, not collapsed into the
base source identity. Binary SHA256 is
`0A50E300E1B4BC9725C0DAA16E5502306AF3F455D142EC62FF79E01331782261`, matching
the same-code built worker. Receipt binds code311e81a7f, zero workspace dirty
entries and both raw/summary pairs. These actual observations are valid and
must be retained/reused; no new performance run is requested for this review.

Recovered085917/091045 summaries each match exactly one parsed original console
JSON line (all four matches independently checked). Failed085917 is retained.
091045 normal exit0 is explicitly **inferred from script continuation**, not a
directly captured exit line; this review preserves that qualification. The new
103747 run records the actual process status directly, so no historical exit
inference is needed as its substitute. Audits and hashes are in
current-resource-audit.json.txt and recovered-source-audit.json.txt.

Real stdout extraction was independently exercised using the actual script
functions through PowerShell AST: one JSON summary extracts, no summary gives
null, duplicate summary throws. Source audit confirms real summary+exit is
saved before gate validation/exit. Script selftest passes existing/new controls.

Two bounded residuals prevent full F-R8-01 closure (now **P2**, original P1
missing measurement data remedied):

1. **Unrelated failure still satisfies intended gate0 rejection.** Existing
   V09 threshold0 counter-control and R8 closing rule require the expected
   threshold rejection, not merely an arbitrary failed bench. Actual
   `Test-BenchSummary` only tests nonzero exit plus nonempty failure array;
   it never checks `summary.gate_ms==0` or the two frozen P95 paths' threshold
   rejection. Independent mutation starts with the real103747 gate0 artifact,
   preserves its complete memory fields/identity/exit5, changes gate_ms to1000
   (both actual P95 values are below1000), and replaces failures with
   `fresh: failing samples [0]`. Expected false: no0ms gate was exercised and
   the failure is an unrelated correctness failure. Actual **true**. This is
   the same required expected-failure distinction from R8, not a new runtime
   defect or new performance promise. The selftest's unrelated-nonzero control
   only empties failures, so misses this legitimate sibling failure shape.

2. **Actual invocation argument field is empty.** Both new103747 artifacts
   contain `args:""` despite nonempty call-site `-Args` values. Independent
   invocation of actual `Save-BenchSummaryArtifact` with
   `-Args 'bench-turnaround --samples 3 --gate-ms 0'` also writes empty args.
   `$Args` collides with PowerShell's automatic `$args` handling in this function;
   nonempty invocation text does not survive to its output. The actual summary
   gate/sample fields and script reconstruct the current configuration, so
   this does not invalidate measured peaks; it leaves the promised durable
   exact command provenance incomplete. Use a non-reserved parameter name and
   verify exact retained text, rather than treating the field's presence as
   proof it was bound.

Minimal reproduction from main:
`powershell -File docs/plans/attachments/827/review-r9/replay-summary-controls.ps1`.
Exit1, real positive true, single/missing/duplicate controls correct,
unrelated expected false/actual true, writer expected nonempty/actual empty.
This extracts the **committed production functions** and writes only review
attachments; it never mutates implementation or launches a campaign. Durable
inputs and output: unrelated-failure.json.txt, writer-args-control.json.txt,
summary-controls.log.txt.

Minimal close in T06: validate actual0 threshold and expected frozen-path
threshold failures, and reject unrelated correctness/resource/infra failures
as the intended gate0 control; keep real gate0 positive and normal positive.
Rename the argument parameter, preserve exact actual invocation, and add the
above finite controls against those same functions. Correct existing provenance
from its recorded real call/source with an explicit annotation if needed.
Current raw/summary/peak evidence and all prior valid campaigns remain reusable
when code/dependencies/config/input are unchanged. **No performance matrix or
full acceptance rerun is required for these script-only residuals.**

## Acceptance/SD/state

AC01..04 and AC09 remain within R7/R8 passed/verified boundaries; R6 nth actual
allocation/budget/ownership cleanup and R7 raw/binary-binding findings remain
closed. Actual AC05 current resource observations now pass their measurement
boundary; AC05/06 remain partial only for this required gate control/provenance
residual. AC07 is performed without final pass; AC08 stays merge-owned pending.

SD01..06 rule text remains accurate and frozen again in spec-delta-frozen.md;
only the review-status preamble changes. No cap/ABI/language/profile/default or
GOAL-002 completion claim changes. No canonical Spec/ledger publication.
The existing actual resource evidence is no longer labeled missing; the
remaining blocker is the two finite T06 issues above.

Outcome **needs_fix**, **only T06 reopened**, Plan executing6/9 (work6/7),
T09 complete, T07/T08 unchecked. Keep all historical records and other-plan WIP.
This round ran script selftest and finite AST controls only, no cargo/whole tier,
performance matrix, acceptance suite or heavy family. Merge-owned frozen bare
cargo t + applicable compiler tv/finite workspace/consumer gates remain pending
after acceptance pass; other surface tiers only on actual trigger, tf batch-only.
No merge/archive/cleanup or implementation change.
