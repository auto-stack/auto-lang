# PLAN-827 final repair review R8 — needs_fix (2026-10-25)

Independent reviewer: `/root/plan827_t07_review`, the same separate context as
R7, never an implementation author. User-authorized same-tree repair review;
no extra agents, implementation writes, merge or canonical publication.
Plan revision3; HEAD `ebced8cd5e5f26e609062acdcb08808452af5e20`, branch
`plan-827-dev`, unique tree `D:/autostack/.wt/lang-827/auto-lang`, clean before
and after. Repair code `10c1456e228c9f4bbfd56dc167e52c3cb4f82c20`, prior reviewed
commit `4a069ce7ffeaeb1cdd7491bab65e7838dc3f09ea`, original base
`369e1c349ea9587907f073aaecd3082e2bb486bd`. Baseline/input hashes, toolchain,
dependencies and inherited file blobs are captured in baseline.json.txt and
dependency-metadata.json.txt. Code10c1456e2 -> frozen HEAD changes report
artifacts only, so this round does not invalidate same-code supplied campaigns.

## R7 findings closed

**F-827-R7-01 closed.** Original review replay now returns exit0 with30 rows,
zero null before fields and30 after pairs. Independent artifact audit checks
all actual unchanged pairs: both arrays length2, positive pointers and equal
before/after pointer+generation, zero problems. The complete run has an explicit
pass verdict, P95=412.5422ms, and two immutable named run artifacts are committed;
latest carries run_id. Independent exact tests `raw_shape_rejects_missing_baseline`
and `failing_verdict_still_writes_raw` each pass1/1, no warnings/ignored. The
shape controls cover original null-before, missing-after, short-pair, zero-row
and legitimate complete row. The failure control leaves two observed rows plus
fail/time_gate=false/P95=2100 in its separate negatives file, copied here.

Source audit verifies a real observed revision row is written before that
revision's structural/value assertions; final time-gate verdict is written
before the P95 assertion. Missing actual identity lookups reject rather than
silently becoming null. Earlier partial rows remain on assertion failure;
final P95 failures have their fail verdict persisted. The negative test uses a
controlled recorder input, not an independently forced slow30-edit campaign;
this limitation is explicit. No rerun of the unchanged performance campaign
was needed to inspect its data/producer or replay finite recorder controls.
T09 remains complete; R7's in-memory identity detector evidence remains valid.

**F-827-R7-02 closed for its stated build/content-binding defect.** The resource
scope now builds release worker in the existing frozen workspace, rejects
build failure, records command/HEAD/workspace dirty state and executable SHA256,
then checks stale/foreign, tampered hash, missing executable and exact current
worker before bench. Independent selftest passes its five binding controls
alongside existing selected-ID/skip/child-failure controls. The actual current
release SHA256 matches the committed091045 receipt:
`0A50E300E1B4BC9725C0DAA16E5502306AF3F455D142EC62FF79E01331782261`.
Receipt names code10c1456e2, zero probe dirty entries, and both gate0/30 raw
files; current worker/core/driver/bench/dependencies are unchanged through
frozen ebced8cd5. Build output retained here. No old unbound executable is
accepted as current evidence. The two repaired files are the test recorder and
verification script; R6 production executor is unchanged.

## F-827-R8-01 — P1 — AC05/06, V09, T06: actual peak/summary not retained

This is a remaining original resource-evidence requirement exposed while
checking the repaired receipt, not a recurrence of stale-worker binding and
not a measured memory/performance regression. Existing Plan §5.4/V09/AC05
requires real transient/worker peaks and independently attributable raw;
canonical auto-av/cache-and-conformance.md requires raw/summary of the same run
bound to input and binary, success and failures preserved.

091045 bench raw contains exactly180 sample rows (six scenarios x30). Its fields
are scenario/sample/revision/wall/ok/value/expected/diag/source_hash, cache/stage
counts and staging/hot/slot bytes. None contains `frontend_peak_bytes`,
`private_bytes_peak`, `child_private_bytes_peak`,
`frontend_transient_target_met`, `worker_private_bytes_target_met`, or
`hard_gate_failures`. The binding JSON contains executable/hash/build/HEAD/
dirty-state/certified-filenames/time, with no summary or actual process exit.
Build log proves build completion only. Failed085917 raw and binding are kept
but likewise lack the actual gate/peak summary and recorded exit.

Established cause: worker `av_probe_worker.rs:593-603` computes the real summary
and hard_gate_failures and writes JSON to stdout. The resource script captures
gate0 stdout in `$g0`, normal stdout in `$b30`, and each exit code, but only
prints these objects. It never persists them or includes summary/exit in its
binding receipt. `$reportDir/bench-*.jsonl` is sample-only output. Thus the
independent reviewer cannot inspect the actual dual-line memory observations
or distinguish the recorded gate0 intended failure from an unrelated nonzero
using the durable submitted files. Work's numerical handoff/Plan summary is a
claim, not the missing raw measurement.

Read-only replay: parse the committed091045 receipt and180 JSONL rows, enumerate
their actual fields and inspect the worker/script output flow. The repeatable
audit is recorded in artifact-audit.json.txt; preserved source/receipt/raw hashes
are in baseline/freeze-hashes. No timing failure is injected or inferred.

The independent formal-session report *does* exist in probe/target and is copied
to reused-formal-report.json.txt:30 samples, all_correct/structural/gate true,
P95=331.9732ms, worker_peak=211812352, input len273575/FNV431bff0efbaabfec,
actual debug worker identity recorded. This supports that distinct formal path;
it cannot reconstruct bench's separate frontend attribution/overall/residual
or its missing actual summary. Likewise later finite-test logs imply the script
continued but do not supply the required peak observations. No unsupported
inference is promoted to AC05 pass.

**Minimal close:** in T06 persist the actual stdout summary and exact exit for
both gate0 and normal bench, naming/binding them with executable/input/config
and raw artifacts; validate expected gate0 rejection and normal hard gates from
those real fields. Prefer recovering an already existing original console
capture if available, without inventing values. No such source was present in
this handoff; parent confirmed it only has the same handoff statements. If it
cannot be recovered, after fixing retention regenerate only the affected
resource measurement/evidence, not unrelated inventory/cache/512-churn or the
entire six-family acceptance workflow merely to change reviewer roles. Reuse
valid formal-session and other unchanged evidence. A bounded retained-summary
positive and missing/tampered/failed-summary negative should prove the evidence
validator and failure propagation. Do not move this work-owned gap to merge.

## Acceptance, Spec reuse and routing

R7 AC01..04 inspected/pass boundaries, repaired R6 allocation/ownership, old
recipe/source/native/parent controls and inherited lifecycle mappings are
unchanged and retain their exact R7 scope. AC09 is now pass within its approved
four-subcase scope: original runtime/detector evidence retained, new actual raw
and finite record controls verified. AC05 remains partial for F-R8-01; AC06
partial only for this resource evidence; AC07 performed without final pass;
AC08 merge-owned pending. No new global gate or language/cap/ABI promise added.

SD01..06 normative text frozen again in spec-delta-frozen.md, unchanged in rule
content from R7. Support layering, actual recipe reuse, current grants, cache
trust, D828 identities and R6 limited frame ownership remain accurate. Source
Spec input hashes are bound; no canonical delta/ledger was published. SD02/03/06
resource-evidence conformity still depends on closing F-R8-01. Impact paths and
GOAL-002 retained without claiming complete VM replacement.

Only two finite recorder tests and script selftest were rerun. No whole cargo
t/tv/tt/tb/tu/taa, root/workspace/acceptance suite, performance matrix or old
heavy campaign rerun. No compiler warnings; diff-check clean. Same-tree HEAD
and cleanliness rechecked at outcome.

Outcome **needs_fix**, reopen **T06 only**, preserve T01..T05 and T09 complete.
Plan executing6/9 (work6/7), T07/T08 unchecked. Old R7 findings close and are not
counted as newly open; F-R8-01 is the residual resource evidence gap. After its
repair, affected independent re-review only. Frozen bare cargo t + compiler
surface tv and finite independent workspace/consumer gates remain merge-owned
pending after final acceptance pass; tt/tb/tu/taa only on actual trigger, tf
batch/due only on main. No merge/archive/cleanup.
