# PLAN-824 T-07 R3 — pass (final affected repair review)

## Frozen baseline

- stage: review; review_scope: final repair re-review; outcome: **pass**. Reviewer: Codex in this chat, independent of the implementation handoff; same R1/R2 reviewer context, no extra agent/model identity claimed. No implementation changes made during review.
- Plan r7, shared Plan read on main; input state execution_done 9/11. Worktree D:/autostack/.wt/lang-824/auto-lang, branch plan-824-dev, reviewed_commit **29b6eeea98d1b75e2ea18ea5f62eea96866e9d60**. Repair base **6a68240e89bdfd141ecfb893f83c7718bf503056**; implementation repair **2c8b6b0aa5ef5f101e4d0a6b4148b5605245f41b**; final commit only registers required test IDs, so work evidence at the implementation commit has identical code/dependencies/test inputs to the reviewed HEAD.
- Integration base c6c5ddb256089e6b628d7651cb9759664d15f22a, inherited D821/D822/D828/D852; auto-down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4, auto-ui b4fe0000e56cddaf7d3cf70e348431f2258aacde per dependency receipt. No dependency/lock update in repair. Main observed ecc78701124fe6e0ab6f0e262148a1d853ba2af5, not used as implementation baseline. Source Specs unchanged from R2 by SHA256. All input/lock/SD hashes in inputs.json.
- HEAD/cleanliness rechecked repeatedly, including after final holdout. Initial restricted build-directory access handled by approved scoped test commands; no approval rejection/blocker this review. Builds stayed in the existing implementation tree. Evidence and Plan bookkeeping only on main.

## Finding closure and diagnosis

**F-824-R1-03 [P1] closed on reviewed HEAD.** Diagnosis/coverage adjustment in review-r2/repair-diagnosis.md correctly identifies both failed assumptions: node-only bounds do not bound repeated-name bytes; value-only polling misses large type traversals under a tiny value tree. The repair changes the actual production paths rather than only rejecting the prior sample.

- av.rs resolve_dyn_type reserves root/cell/container/name bytes before their allocations, shares the execution data limit with live slots and publishes temporary high water even for is/take operands. Dyn work charges fuel/polls cancellation during construction, pack admission and type comparison. Pack/is/take callsites inspected. Work/refusal/cancel exits leave candidate temporaries owned by normal cleanup.
- portable descriptor emission checks pending bytes before buffer growth; descriptor reading charges cells, containers, names and temporary duplicate-name storage before construction. Type read/write/admit/equality and binding lookup participate in Traversal work/cancel polling. Restore retains one root descriptor, compares and drops observed child descriptors, then releases their temporary logical bytes.
- deep_clone copies the type with the same charged/cancellable traversal and keeps one copied root descriptor while rebuilding values; tests cover source reuse, independent data and the type-depth boundary. Binding producer preflights subtree/domain byte expansion before actual owned type cloning; unsupported declaration control retained.
- F01/F02 remain closed: scalar kind checks and complete current nominal dependency checks are preserved. Changed restore ownership justified re-running their wire/admission/real adapter negative controls. The former unused import remains removed; exact incremental target check has no warnings.

No new in-contract finding or unapproved semantic scope reduction found. Logical peak counters are not claimed to measure RSS. Work's four-fresh-worker parent-sampled memory evidence was generated on the same final implementation code; reused rather than repeated.

## Independent original and new controls

Original R2 probe unchanged, now compiled against reviewed code (replay.txt):

- 1026 type nodes, one empty-list value: freeze **Cancelled, polls=1**, unfreeze **Cancelled, polls=1** (previously both succeeded with zero polls).
- 16011B ABC / 65535 type nodes / 1024-byte repeated name suffix: **SlotBytes**, peak **4055/4096**, before next reservation 5299. Original 17-layer/721B shape also refuses within budget.
- 3-layer small control: **Success Bool(false)**, steps43, peak1867. Bool legal roundtrip remains successful; F01 mutated scalar → Invalid(type), F02 unknown nominal → unknown-type, changed field type → layout-mismatch.

Independently selected holdout (holdout.txt, source probes/src/main.rs):

- Same small legal input at max_slot_bytes=**1866** → SlotBytes for reservation1867; peak1866.
- Same input at **1867** → Success Bool(false), peak1867. Distinguishes exact budget boundary from arbitrary rejection.
- Long-name sample at the actual default **8388608** byte limit → SlotBytes for reservation8389128, peak8387885; no longer only protected by the small test cap.

Both probes print observations and exit0; assertions/verdict derive from the actual output plus production/test inspection, not the process exit alone. Commands from implementation tree:

```powershell
cargo run --quiet --locked --manifest-path D:/autostack/auto-lang/docs/plans/attachments/824/review-r2/probes/Cargo.toml --target-dir D:/autostack/.wt/lang-824/auto-lang/experimental/ac-av-probe/target/review824-r3
cargo run --quiet --manifest-path D:/autostack/auto-lang/docs/plans/attachments/824/review-r3/probes/Cargo.toml --target-dir D:/autostack/.wt/lang-824/auto-lang/experimental/ac-av-probe/target/review824-r3
```

## Independent targeted verification (24 tests)

Raw commands/output in targeted-tests.txt and integration-health.txt, all selected>0, none ignored:

| Manifest/target/filter | Passed | Invariant |
|---|---:|---|
| parallel/824 --lib av_dynamic_824_r2_ | 5 | type-only cancellation, third-poll admission cancellation, byte-limited copy/restore, retry/source preservation, depth256 copy |
| parallel/824 --lib av_dynamic_824_wire_ | 5 | F01 and changed restore wire/parent-child consistency, malformed checksum/header |
| parallel/824 --lib av_dynamic_824_admission_ | 3 | F02 empty/inactive/changed types and legal fresh index |
| probe --test av_dynamic_824_resources av_dynamic_824_resources_r2_ | 6 | temporary bytes default/small caps, three Dyn callsites, binding bytes/nonerasable control, fuel, cancellation/ledger drain and retry |
| same resource binary, av_dynamic_824_resources_deep_chain_roundtrip_within_bound | 1 | changed root descriptor ownership on depth255 actual value chain |
| probe semantics exact restore_revalidates_current_field_types | 1 | real binding adapter |
| probe semantics exact production_same_source_byte_equivalence | 1 | production/harness same-source byte identity |
| probe first_case exact av_dynamic_824_restore_reverifies | 1 | real managed-store restoration and re-admission |
| probe cache exact av_dynamic_824_cache_portable_good_checksum_bad_structure_rejected | 1 | good checksum cannot admit bad inner structure |

Additional health: exact incremental test-target cargo check passed without warnings; core/probe package cargo fmt checks and harness fmt passed; git diff --check 6a68240e8..HEAD passed; GuardSelfTest passed. New 11 repair tests are in required-ID lists, harness list is actually passed to Run-Harness-Tests/Assert-Selected. Existing root-workspace baseline warnings documented by work are outside this repair and are not hidden as a zero-warning root build claim.

No whole-tier t/tv/etc, full workspace suite, aggregate acceptance/baseline script or unrelated scale/performance matrix run in review. The 43-test work receipt and the reused R1/R2 evidence are retained at their stated input/revision boundary. R3 independently tests the paths changed by this repair; no global green banner substituted for these checks.

## Acceptance and evidence reuse

| AC | Final review disposition / basis |
|---|---|
| AC-01 | pass: retained bounded first-case/formal parent-oracle evidence; same implementation work formal corpus green, real restore and same-source producer independently rechecked |
| AC-02/03/04 | pass: F01/F02 closure retained and replayed; F03 typed operation budget/cancel, refusal/ownership and source-preserving retry now pass above |
| AC-05 | pass: F03 default/small/exact byte caps + fuel/cancel/cleanup checks, source-inspected pre-allocation checks; worker memory receipt reused at identical code |
| AC-06 | pass: retained prior consumer/cache/oracle evidence plus rechecked store restore and self-consistent malformed portable refusal; affected binding/restore identity controls pass |
| AC-07 | pass: no open findings; revision/hash/Spec delta/health reviewed, closeout gates explicitly pending |
| AC-08 | pending merge (not a work/review failure) |
| AC-09 | pass: retained D828 scalar campaign and unchanged incremental contracts; work final-code body identity test and prior T09 controls retained, no new full performance claim |
| AC-10 | pass: same-source production/harness, budget/refusal/cancel and actual byte equivalence checked |
| AC-11 | pass: existing typed D/W/I bindings retain their honest source/native exclusions; affected resource/admission/recovery axes now close; required regression registration verified |

V-06/07/11/14/15 F03 portions updated to pass; V-05/08/12 F01/F02 remain closed; unrelated V09/V10/V13 historical evidence remains its actual scope. Dependency, locks and non-Dyn producer/consumer inputs unchanged, so unrelated checks were not repeated merely for another review. The historical 798 aggregate timing red is **not relabeled a passing aggregate run**: preserve the recorded same-shape baseline-red comparison and isolated pass, and unchanged non-Dyn path attribution. R3 changes Dyn-only execution/size/admission branches and private dynamic modules, not the scalar source campaign. This supports reusing existing scalar compatibility evidence; merge still must execute/adjudicate required frozen closeout gates, no timing requirement waived. Prior admitted platform gaps are outside this Plan's typed profile; no new gap/debt created to excuse an AC.

## Spec delta and remaining closeout

SD-01..07 frozen copy spec-delta-frozen.md (hash in inputs.json), canonical Spec targets unchanged from R2. Frontmatter paths correct; touched_goals=[] remains justified because this experimental profile adds no repository-level goal ID. Knowledge delta is consistent with verified current semantics: safe scalar kind/complete nominal restoration, before-allocation type bytes, work-based type cancellation, existing code-vs-portable domain separation and typed-only capability scope. Merge should deposit these enduring rules, not older node-only implementation notes. No canonical Spec or live ledger publication in review.

**Final pass applies only to 29b6eeea98d1b75e2ea18ea5f62eea96866e9d60.** Plan becomes reviewed 10/11, T07 checked, T08 pending. Next owner merge must execute frozen bare cargo t + applicable cargo tv + finite new-workspace contract gates, and retain/adjudicate required compatibility timing evidence. tf remains batch-only on main when due. 823 profile/wire/opcode collision must be reconciled against actual landing order; any renumbering/integration code changes require committed affected re-review and gate evidence on the resulting baseline before landing. No merge, archive, cleanup or implementation edits performed here.
