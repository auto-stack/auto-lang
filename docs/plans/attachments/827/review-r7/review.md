# PLAN-827 final independent review R7 — needs_fix (2026-10-25)

## Identity, scope and baseline

Reviewer is `/root/plan827_t07_review`, a separately delegated context explicitly
authorized by the user via the parent. I did not implement PLAN-827, edit its
implementation, create another tree, or delegate review. No invented model or
session identity. All implementation tests ran in the existing unique tree
`D:/autostack/.wt/lang-827/auto-lang`, branch `plan-827-dev`, frozen HEAD
`4a069ce7ffeaeb1cdd7491bab65e7838dc3f09ea`. Clean before and after. Original
base `369e1c349ea9587907f073aaecd3082e2bb486bd`, repair base
`9f7b5dd6e144b0577a31d9528f667e85bd73c311`, Plan revision 3 unchanged.

Read main AGENTS, auto-plan-review SKILL, current main Plan, R1..R6, contract,
SEM material, acceptance matrix, R6 readiness and relevant canonical Specs.
`baseline.json.txt` records full revisions, toolchain, actual input SHA256 and
head/base Git blobs for inherited implementations/tests/configuration; metadata
captures actual local dependency paths/versions. Main HEAD can advance for
unrelated plans; recorded main Plan hash binds the input read here. No
canonical Spec or ledger write. Only main plan/review bookkeeping was changed.

## R6 allocation finding closes within the stated boundary

The new seam is production `dispatch` root/callee `reserve_slots`, reached by
both loaded modules and snapshot unit views. It uses fallible
`try_reserve_exact`, after existing frame/slot budget checks. For real reserve
failure, attempts have advanced and success has not. The optional ownership
lease lives on Frame after the slots Vec field, so Rust field destruction drops
the actual buffer before release accounting; it is not an outcome-only counter.
Returning a terminal outcome drops LoopState frames; PausedRun retains them and
the Arc snapshot, and resume/drop terminate that ownership. Argument copy now
reads caller slots before pushing the callee, avoiding the old temporary Vec.

Independent semantics run: 14/14 pass. n1=(1,0,0,0), n2=(2,1,1,0),
n3=(3,2,2,0) for attempts/success/released/live_bytes; legal trace prefixes are
empty/A/AB. n4 succeeds value7 with all3 allocations released. Zero frame/slot
budgets have zero attempts; callee frame-budget refusal precedes second reserve.
Cross-pause second-allocation failure releases old frame; cancelled resume after
session close releases slots and Weak snapshot; dropping paused run, trap and
fuel paths release actual buffers. The uninstrumented positive executes. Source
inspection verifies failed call is never pushed/dispatched. Work's bypass-trigger
mutation raw (exit101, original n1 assertion sees Success7) is retained as
supporting historical detector evidence, not falsely claimed independently rerun.
No implementation mutation was permitted in this review. Root/snapshot sibling
executor tests independently 6/6 pass, core check passes with no warnings.

This closes F-R5-01's formerly admitted nth-frame allocation gap and keeps T04
complete. It does not claim process-wide allocation fault coverage; store/root
lease lifecycle retains its separately bound inherited tests.

## F-827-R7-01 — P2 — V13 persisted identity and failure raw incomplete

Existing rule: AC09/V13 and §5.4 require raw per-revision structural/identity
evidence, with failures retained, not merely a successful P95 summary.
`av_source_827_incremental.rs:121-129` keys `identities_before` by unprefixed
function names (`main`); line162 emits `identities_before.get("d_main")`.
All30 persisted `unchanged_sample.before` fields are therefore null, while
after contains a pointer/generation pair. In-memory unchanged-identity assertions
at lines211-230 still exist and are not alleged broken. The artifact cannot
reconstruct its advertised before/after pair. Separately, structural/per-value
assertions occur inside the loop and P95 asserts at249, but raw is written
only at268: a failing required revision/P95 leaves no new durable run, and a
later success overwrites the fixed filename. This falls short of the existing
success/failure raw contract rather than adding a new performance requirement.

Reproduction: from main, `powershell -File
docs/plans/attachments/827/review-r7/replay-evidence.ps1`; expected zero missing
before identities, observed 30/30 missing, exit1. Raw summary and output retained
in raw-identity-replay.json.txt and raw-replay.log.txt. Cause of null baseline is
established by code and committed JSON; failure-retention path is source-audited,
not presented as an independently forced timing failure.

Close in T09: normalize identity key lookup, reject absent baseline/current
pairs, and persist complete per-revision observations and final verdict in a
per-run durable artifact before assertions can discard failure evidence.
Minimal replay: artifact-shape/missing-baseline negative, legitimate finite edit,
and a controlled failing structural/time verdict that still leaves its raw.
Regenerate affected 30-revision raw once on repaired code if that producer changes;
no unrelated six-scenario performance campaign or global tier is required.

## F-827-R7-02 — P1 — V09 bench binary not bound to frozen source

Existing rule: AC05/§5.4 and inherited cache-and-conformance require original
load resource/time evidence bound to current code, inputs and binary; reuse is
only valid when implementation/dependencies/configuration/inputs are unchanged.
`verify-av-source-827.ps1:203-225` executes
`target/release/av_probe_worker.exe` twice without building it, validating its
source/dependency provenance, or checking a revision-bound reusable-evidence
manifest. The script's verdict reads process status only. Its later cargo test
builds the debug formal worker, not this release worker. Thus a valid older
release worker can supply the purported current resource gate. This deterministic
build/content-binding gap is the finding; no measured resource regression is
alleged and no performance threshold is weakened.

Concrete inspected artifact: release exe SHA256
`0B5D0B0F5A4F8F5BB7F4E47A54BA3D4FFE41301A836A770CCDF971A261290F6D`,
write time 2026-10-24 23:14; depinfo captured in release-worker-depinfo.txt lacks
the new `frame_allocation.rs`, whose source exists at frozen R6. Timestamps and
depinfo are supporting evidence of the local old artifact, not sufficient
cryptographic source provenance by themselves. Work-r6 correctly acknowledges
prior performance raw predates the executor change; that cannot be promoted to
current-code AC05 pass or silently moved from work-owned acceptance to merge.
Read-only capture is in resource-worker-binding.json.txt. No old binary was
replaced and no campaign was run against it.

Close in T06: ensure the exercised release worker is built from the frozen
workspace or validate an explicit equivalent build/input/dependency identity;
record binary hash and source/config/input binding with both success and failed
raw. Add a bounded stale-worker/binding negative that fails before campaign
execution and a legitimate current-worker control. Only affected resource/time
evidence invalidated by the R6 executor change needs refresh after repair.
No unrelated cache/GC inventory, full workspace suite or complete six-scenario
performance matrix is requested merely because HEAD changed. Existing frozen
merge-owned whole tiers still run only after final acceptance pass.

## Acceptance map and actual reuse boundaries

| AC / task / V | Evidence and verdict |
|---|---|
| AC01 / T01,T02 / V11,V12 | pass for bounded G0 inventory and SEM material. Reviewed scanner's retained errors, per-source/oracle hashes, stdlib file/provider members, symbol anchors and table rows. R3 independent inventory/negative closure retained; inventory implementation/input blobs unchanged by R6. 818 rows are inventory, not AV support. SEM01..08 preserve future owners and AP18 witness; no unsupported program counted as success. |
| AC02 / T03,T04 / V01..03,V07,V10 | pass for original file pipeline/native and typed outcomes. Independent first-case5/5, semantics14/14; AP identities, fresh/hit and actual new-process restore (2 units reverified) pass. Existing formal corpus and parent oracle/fault evidence from R5 retained for unchanged recipe/parent assertions; executor path shared and targeted sibling success/trap tested. No new full root suite claimed. |
| AC03 / T02,T04 / V02,V03,V05,V12 | pass within scalar profile. Argument edits change result; false-only versus required division trap, trace order, unsupported/wrong entry front-door and zero-dispatch are asserted; native exit70 is separate. SEM future gaps remain explicit. No old VM fallback was added; only new production code is slot reserve/ownership. |
| AC04 / T03..T05 / V04..08,V10 | pass within inspected consume boundary. R5 V06 mappings now select cache/ABC/native required members; verified actual corruption/checksum-repaired-inner, owner/body/kind positive/negative assertions and complete single-fn module. Thin consume/grant/span tests independently pass. Existing source freeze/UTF8/path and parent negative oracle/fault controls retained with unchanged parent/worker code. Cache5/5 independently pass. Inherited seven lifecycle cases are exact-required; inspected restore live grant, lease table closure, position-edit required sidecar GC and release assertions. No blanket claim that R6 reran all inherited tests. |
| AC05 / T04,T05,T06 / V07..09 | partial. R6 nth allocation/actual cleanup passes, source cap-at/+1 and fuel/cancel pass. Store lease/churn finite test/input and production cache code remain unchanged and are bound by matrix; no repeated 512-edit churn. Prior timing/resource raw remains historical; current-code binding incomplete F-R7-02. |
| AC06 / T01,T03,T05,T06 / V10..12 | partial F-R7-02. Exact required IDs and acceptance scope inclusion inspected. Independent selftest accepts legitimate selection, rejects skip/ignored/renamed/duplicate/missing and propagates real child exit5; 0 cases/tools cannot be silently green. R4/R5 missing-native independent closure retained, first-case native and missing-tool control independently pass. Existing old recipe/817/818 compatibility faces remain selected, recipe code unchanged; no re-certification of every upstream full suite. |
| AC07 / T07 | review performed; fail due two in-contract evidence gaps. No final pass. |
| AC08 / T08 | pending merge owner, never represented as review/work completion. |
| AC09 / T09 / V13 four subcases | partial F-R7-01. Independent two-function local edit, precise refusal/current-grant reentry and actual forced-rewarm/equal-hash replacement detector checks each1/1 pass. In-memory identities/stages/IO remain asserted. Inherited prepare/cancel/close transaction test source and implementation blobs unchanged; exact required mapping inspected. R2/R5 30-edit structure/time evidence retained only within original boundary, raw defect not waived. |

R5's missing-selection rows are now connected and source-audited; R6 nth gap
closes, while AC05/06 and AC09 remain partial for the two precise new findings.
No admitted omission is registered as permissible debt. No deferred language
features are silently promoted to supported.

## Spec delta and health

All SD01..06 targets, current hashes, rationale and AC bindings inspected.
`spec-delta-frozen.md` freezes concrete proposed text, correcting old candidate
recipe wording to actual `ac-av/core-run-v1` reuse and adding bounded R6
ownership/fault semantics. SD01/04/05 support-boundary text is consistent with
actual code. SD02/03/06 are valid proposed rules, but evidence/raw/resource
conformance still awaits the two fixes before final pass. No canonical Spec,
ledger or goal completion was published; impact metadata remains exact and
GOAL-002 only receives a limited future contribution. Hashes are in freeze-hashes.

Independent tests: semantics14, first/cache10, incremental3, executor6 =33
selected passes, no ignored or new warnings; core check and git diff --check
pass. Test counts are scoped totals, not a whole acceptance verdict. Selftest
also passes its bad/valid controls. Work's changed-file rustfmt evidence reused;
no broad fmt that touches dependencies. No TODO/debug prints introduced in the
production seam; no new fallback, cap increase, legacy semantic expansion or
canonical modification. No full cargo t/tv/tt/tb/tu/taa, whole workspace,
performance matrix or unrelated heavy replay was executed.

## Routing and pending closeout gates

Outcome **needs_fix**. Preserve T01..T05 complete; reopen T06 and T09, T07/T08
remain unchecked. Plan executing5/9 (work5/7). This is implementation/evidence
repair in the same unique tree; no merge/archive/cleanup. Minimal next replay is
the two finding packages, then affected independent re-review. Reconcile the R6
core write-window supplement with the user authorization; original T01's
historical zero-public-write claim is not the final diff inventory.

After a final pass, merge owns frozen bare cargo t + compiler surface tv,
the independent workspace's complete finite contract and actual affected
consumer gates; tt/tb/tu/taa only if their real trigger surface changes (none
added here), tf only due/batch on main. These pending gates cannot close the
currently unfinished work-owned evidence and cannot authorize landing now.
