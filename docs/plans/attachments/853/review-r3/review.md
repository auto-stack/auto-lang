# PLAN-853 independent affected final review R3

Verdict: **pass** on frozen `c1961665d1a90a8705994085fd4344a1d6a9a564`. The remaining F-853-R2-01 is independently closed at the case-state invariant and both actual evidence consumers. No new in-contract finding remains. T-06 completes; Plan is reviewed,6/9. This is acceptance/Spec review, not delivery: T-07..09 and all merge-owned gates remain pending.

## Baseline and independence

Actual reviewer identity: delegated `/root/review_853_r3`, fresh independent implementation-free context, final affected re-review, third final round. No model/session identity is inferred. Read assigned `D:/autostack/auto-musk/.agents/skills/auto-plan-review/SKILL.md`, its verification-contract reference, main AGENTS, main Plan r1/common contract, both prior reports/persistent inputs, repair-r2 diagnosis/closure/handoff, canonical graphics Specs and actual diff/callers. Hashes and frozen input Plan are in `independent-results.json`/`plan-input.md`.

Sole implementation tree: `D:/autostack/.wt/lang-853/auto-lang`, branch `plan-853-dev`; repair base/R2=`5e3f5b65f145fb127a4e206b626d3e2e3dc0ab8b`, R1=`760c232f4c1185591bfef029435101fa4fd6b594`, overall base/M1=`11911a4a6d809bfd515f745b77d00bc8f04f6ba9`; SDK=`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`. Entry/exit HEAD and branch checked, clean implementation throughout. No implementation writes/new checkout/agent/merge/archive/cleanup. Main's unrelated concurrent changes preserved.

Actual six-path repair diff: m2_evidence.py, test_m2_evidence.py, identity, README and two research documents. Rust/src/tests, SDK, both locks, AGP, API, algorithms, assets, samples, build configuration and source/API/measurement consumers unchanged since R2. Unchanged-function AST comparisons explicitly cover check_selection/validate_run/runner identity helpers, while their changed selected_tests callee is freshly exercised. All twelve canonical Spec hashes match R1/R2.

## Closure and independently observed behavior

| Prior finding / rule | Closing observation | Result |
|---|---|---|
| R1-01 complete source/export closure | Source verifier/package consumers unchanged from independently passed R2; current441 listed inputs verified under documented LF transport normalization; package-root/config/automatic-target repair and prior negative proof retained | closed, unaffected evidence reused |
| R1-02 all five API compatibility credentials | Credential/generation code and all inputs unchanged; original retained producer/consumer runners and each21 actual generated output hashes match R2. R2 five-version LF/CRLF bad-hash controls remain applicable | closed, unaffected evidence reused |
| R1-03 binary interruption | Exact R1 header-no-completion and unframed-failure logs reject through both check_selection and validate_run; complete binary/zeroDoc/nested controls and original logs remain accepted | closed |
| R2-01 every observed case closes exactly once in its own frame | Exact three R2 logs reject through both APIs; original false-green receipt is consumed unchanged with the real measured runner and now rejects. Fresh cases distinguish pending-state preservation from summary/count checks alone | closed |

The two-failure diagnosis is substantiated by actual code. Each frame now owns a separate announced set, completed IDs and pending case. A new case cannot overwrite pending state, reuse an identity, or exceed declared count. Explicit completion requires an owning pending case. Summary requires no pending state plus exact announced/completed/declared equality. Child exit pops only the child; the parent remains pending until its own explicit completion. Required IDs are a separate inclusion check. This is the existing partial/interruption requirement, without a new arbitrary-stdout/security or future-syntax contract.

`replay.py` independently records34 raw controls through BOTH selection and real PE-bound validate_run:24 reject,10 accept (the original false-green receipt is also separately replayed unchanged). Durable inputs and observed diagnostics are adjacent `.log.txt` files or exact referenced R1/R2 originals. Representative new controls include pending with spare declared room, matching-count pending summary, EOF before completion/summary, unfinished child, child overwrite with spare room, second parent case still pending after child exit, duplicate child identity, zero-child with a case, excess start, duplicate/unowned explicit completion, failed explicit completion, ignored child and unattributed nested block. Valid controls include explicit split completion with ordinary output, multi-case, multi-case with child, successive/deeper children, zero child followed by real child, and complete zeroDoc. Parent missing its own completion rejects. This directly checks state closure rather than merely expecting a totals mismatch.

Four permanent SelectionCompletionTests methods pass, including two newly added methods. The two new methods consume both APIs with real temporary file/hash checks; reviewer probes additionally use the actual retained measured PE. CLI self-check passes. Call path inspected: selected_tests -> check_selection -> validate_run/CLI; rejection exceptions propagate to nonzero CLI. Existing expected-negative oracle/runner/report paths are unchanged; no broad rerun was used to hide the repair's scope.

## Actual commands and retained evidence

From the sole driver directory:

- `python -m unittest discover -s tools -p test_m2_evidence.py -k SelectionCompletionTests -v`:4 methods pass,0 failures,0 skips,0.080s.
- `python tools/m2_evidence.py --self-check`:exit0.
- `python D:/autostack/auto-lang/docs/plans/attachments/853/review-r3/replay.py`:exit0; independent dual-entry controls, adapted bounded repair replay and raw denominator audit.
- `python D:/autostack/auto-lang/docs/plans/attachments/853/review-r3/reuse-proof.py`:exit0; unaffected input/evidence hashes only.
- `git diff --check`, final HEAD/branch/status:clean, same frozen commit.

Command arrays, actual exits/log hashes, expected/actual per-case outcomes and coverage bindings: `independent-results.json`. Bounded `repair-replay-adapted.py` changes only evidence destinations from the repair replay, preserving historical repair evidence. `replay-results.json.txt` confirms exact original per-binary maps:producer and consumer each76 blocks/280 IDs, with nested portable_child excluded; current scoped6/Play5/storage1 selected IDs and raw hashes retain. `runtime-audit.json` verifies all85 historical groups remain exact and rebound, original native14+4 reopens and owner6 named failures+restored pass against retained runners and actual reports/logs. These are artifact replays, not new native experiments.

`reuse-proof.json` independently verifies681 SDK file bytes, all50 original measurement reports and raw-log hashes, all scoped check/fmt/test log hashes and original/rebuilt executable payload equality under only previously recorded link metadata exclusions. The unchanged measurement checker/runtime/input/config and prior R2 validation of1500 accepted ticks/statistics support retaining AC-02/03. There is no resampling or claim of a new performance run. Original type/fmt/warning evidence remains applicable:driver new0, SDK146 inherited. Source/API campaign, actual generation, native/owner experiment, performance matrix, complete Cargo/root tiers and docs_gen were not rerun.

Review harness correction: initial raw README SHA comparison stopped before probing because the manifest intentionally normalizes CRLF to LF. The harness was corrected to the actual documented transport rule and all441 identities then matched. This was a probe error, not a product failure, and no input was modified.

## Acceptance and knowledge delta

| AC / task | Result and evidence boundary |
|---|---|
| AC-01 / T-01,04,05 | pass; frozen tree/dependencies/write set,441 source inputs/681 SDK, R2 source/export invariant retained |
| AC-02/03 / T-02,03,05 | retained pass; unchanged observer/runtime and50 original process reports/1500 ticks, same runner/inputs/config; no new timing claim |
| AC-04 / T-04,05 | pass; prior runner/native/owner/85-group evidence retained; actual raw completion dual-consumer invariant now closed; original exact76/280 each preserved |
| AC-05 / T-04,05 | retained pass; source/API consumers unchanged, actual21 files each/ten fixtures and retained runner hashes match R2; original byte/region identities preserved |
| AC-06 / T-06 | pass; independent affected code/acceptance/SD review complete, no open finding or unapproved deferral |
| AC-07 / T-07 | pending merge-owned; frozen complete producer/fresh own-SDK empty-target consumer suites including new cases, original native/owner/content/new surface and AGP23 complete-input proof or rerun. Reuse is conditional on actual full relevant inputs, not simply unchanged AGP |
| AC-08 / T-08,09 | pending merge-owned; source only autographics-dev; canonical/overview/plans/ADR/ledger/index deposition, archive and guard-clean actual tree/branch cleanup |

Frozen proposed delta `frozen-spec-delta.md`, SHA256 `4b5c7ca7670e9a93420635c7bd90e313a2952dbffaa4f8b53d5a9e16e7f2145f`. SD-01 measurements/bounds and SD-02 current source/all-five credentials remain supported by unchanged evidence. SD-03's new case-ownership detail matches independently verified behavior and the existing partial rejection rule; I2/I3 remain pending. Exact target paths and GOAL-008 metadata are appropriate. No canonical Spec or ledger was edited. No algorithm substitution, budget/tolerance increase, skipped required test, hidden fallback or unapproved scope deferral found; no debt entry is used to excuse unmet work.

Next:merge owner runs or verifies allowed reuse for the frozen full closeout gates before landing. Review pass alone does not authorize claiming M2 delivery. Keep this implementation tree frozen until that step; historical R1/R2 needs_fix and both diagnostic records remain intact.
