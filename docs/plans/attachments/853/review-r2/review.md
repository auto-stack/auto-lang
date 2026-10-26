# PLAN-853 independent affected final review R2

Verdict: **needs_fix**. The three original R1 examples now reject, and source/API repairs pass their affected invariants. One adjacent completion-state gap remains in the changed selection parser: an unfinished test case inside a block can be silently discarded. This blocks AC-04/06; it is an evidence validator defect, not a demonstrated gameplay defect. T-04/T-05 reopen, T-01..03 remain complete: executing,3/9. T-06 and all merge-owned gates remain pending.

## Baseline and independence

Actual reviewer context: delegated `/root/review_853_r2`, a fresh independent context without implementation-session history. Scope=final affected re-review, second final round. No model/session identifier is inferred. Same sole tree `D:/autostack/.wt/lang-853/auto-lang`, branch `plan-853-dev`, reviewed HEAD `5e3f5b65f145fb127a4e206b626d3e2e3dc0ab8b`; repair base/R1=`760c232f4c1185591bfef029435101fa4fd6b594`, overall base/M1=`11911a4a6d809bfd515f745b77d00bc8f04f6ba9`, SDK=`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`. Entry and exit HEAD/branch/cleanliness verified; implementation writes stayed frozen. No new checkout, repair, agent, merge, archive or cleanup.

Read the assigned auto-musk review SKILL.md and verification-contract reference, main AGENTS, main Plan r1/common optimization contract, R1 report/probes and repair closure/handoff, relevant canonical graphics Specs, actual repair diff and its callers. `independent-probes.json` binds12 canonical Spec hashes, unchanged from R1. Skill/input hashes and input Plan are in `review-binding.json`/`plan-input.md`.

Repair diff has10 paths: tools, identity and research/README only; no Rust/src/tests, SDK, AGP, Cargo.lock, API, samples/assets, root configuration or native input changes. Thus prior runtime/type/fmt/warnings evidence remains applicable. All441 current source inputs and681 SDK inputs pass in copied fresh consumer; source identity SHA=`2389cd47c06f7948ac3029b2d6cae31848b56f4b41da0dde038959d4beba963a`. Main checkout's unrelated concurrent changes were preserved.

## Finding F-853-R2-01 — P2 — unfinished in-block case disappears at summary or next case

Existing rule: AC-04, V-04, T-04; Plan5.3/common contract require actual completed raw invocations, rejecting partial/interrupted evidence while preserving original IDs. SD-03 likewise promises partial rejection. This is an original requirement gap in the same completion invariant as F-R1-03, not a new product feature or a demand for hostile test-output security.

Bound inputs: `pending-before-summary.log.txt`, `pending-overwritten.log.txt`, `empty-doc-pending.log.txt`, reviewed commit above. All start from the accepted legitimate required-case block. The first inserts `test interrupted ...` before its summary; the second inserts that unfinished case before `test required ... ok`; the third adds a legal zero-count Doc-test block but inserts an unfinished case before its zero summary. These are deliberately incomplete/inconsistent raw evidence controls and must reject rather than silently normalize to the completed subset. They make no claim that a legitimate libtest execution emits contradictory totals.

Replay: from main, `python docs/plans/attachments/853/review-r2/holdouts.py`; focused API is actual `m2_evidence.check_selection(raw, {'Running tests/example.rs':['required']},0)`. `probes.py` records all independent cases and actual two-end generation (replay generation into a new output path if an existing directory is rejected). `false-green-receipt.json` binds the actual unchanged measured runner and SHA of the bad raw input; the real downstream `validate_run(receipt,runner)` also returns True. No mock pass flag stands in for consumption.

Expected: reject all three unfinished-case inputs. Observed: all accepted; first/second return only `required`, third returns that block plus an empty Doc-test. `independent-probes.json` and the durable raw inputs preserve exact results. Legitimate simple completion, proper empty Doc-test, complete extra binary, and nested portable_child complete controls remain accepted; the two original R1 header/unframed-result controls now reject.

Established cause: `selected_tests()` records `current['pending']`, but overwrites it on each new `test ...` line without requiring the previous case to finish; the summary path checks count/IDs but does not require pending=None. Therefore count/required-ID checks can be satisfied by the retained subset while unfinished observed state vanishes. Affected siblings: top-level and nested child block summaries, successive case lines, zero-count Doc-tests. Header-before-count repair does not close these in-block transitions.

Minimal closing oracle: reject the three persistent inputs through both selection and validate_run; add permanent transition assertions, including nested unfinished case and successive case lines; preserve actual nested portable_child, completed zero Doc-test, extra complete binary, original76/280 per-end logs and current scoped selections. Do not rerun performance/native/full suites for this parser fix. Root-cause adjustments belong to work, not this reviewer.

## Old-example closure versus invariant closure

| Finding / invariant | Independent observation | Result |
|---|---|---|
| F-853-R1-01 source closure | Exact original unlisted root build.rs literal rejected by actual verifier, full source and export before metadata; independent AGP extra-root .rs also rejects; permanent driver/AGP/shared config/examples/benches/missing identity/verifier controls; restored consumer own-SDK/own-target closure and legitimate AGP build.rs export | closed |
| F-853-R1-02 all five API credentials | Persisted bad-v1..5 inputs reject; actual independently built producer/consumer each regenerate21 byte-identical files;10 LF/CRLF fixtures and50 metadata mutations reject/restore; Python assembly compared with unchanged Rust construction and raw API inputs, separate from namespaced region hashes | closed |
| F-853-R1-03 new announced header | Original header/no-completion and unframed-failure reject; valid extra binary/emptyDoc/nested child retained | original example closed; full completion invariant remains partial due R2-01 |

Permanent regression command: in driver `M2_GENERATED_FIXTURE=D:/autostack/plan-artifacts/plan-853/baseline-r2/generated`, `M2_TEST_ARTIFACTS=D:/autostack/plan-artifacts/plan-853/review-r2`, `python -m unittest discover -s tools -p test_m2_evidence.py -v`:5 unique tests pass,11.081s. The source methods retain30 actual subprocess calls in `permanent-source-results.json`. These green tests omit the now-durable pending-state controls; they do not overrule the finding. `source-holdouts.json` adds six actual rejection calls plus restored positive, including original R1 literal and AGP sibling.

## Acceptance and retained evidence

| AC | Result / evidence |
|---|---|
| AC-01 | pass affected source/revision/write-set check; current441 identity and681 SDK closure, root/config/automatic targets and actual export admission controls above |
| AC-02/03 | retained pass with explicit unchanged Rust/SDK/locks/API/assets/config/input proof; original50 process/1500 accepted tick raw reports replay via validate_existing; clock/statistics/raw hashes/runner/semantics pass; no resampling |
| AC-04 | partial/fail: original85 groups and original76 blocks/280 IDs per end exact; native/owner/runner identity retained, original header rejection closed; F-R2-01 blocks complete raw admission |
| AC-05 | pass affected API/source/consumer checks; actual10 fixtures/21 files per end identical, original raw user bytes and emitted regions bound; actual consumer runtime evidence reusable because runtime/build inputs unchanged |
| AC-06 | fail pending F-R2-01, T-06 not complete |
| AC-07/08 | pending merge-owned complete frozen two-end suites, original native/owner/content/new surface and AGP23 complete-input proof or rerun; source only autographics-dev; canonical/ledger/archive/guard cleanup |

`runtime-denominator-audit.json` re-executes the R1 direct raw evidence audit:85 groups equal historical inventory, unique rebound IDs/commands, exact76/280 original blocks each, native14 cases+4 reopens bound to immutable runner and original pixel oracle, owner6 named failures+restored success bound to immutable runner. This is raw artifact replay, not a new native experiment or permission to reuse future merge gates automatically. `historical-replay.json` and `independent-probes.json` retain raw hashes/current scoped IDs. Prior check/fmt logs hashes match; repair changes no Rust so SDK146 inherited/driver0 new warning evidence is retained, no Cargo test/check/full tier executed in R2.

## Spec delta and convergence

Frozen current SD copy `frozen-spec-delta.md`, SHA256 `222a98c0c38b3f3d478f04370946c548c4ca66922a57229e6b32efcbf9d0e98e`; impact paths/GOAL-008 unchanged. SD-01 measurement boundaries remain supported. SD-02 source/all-five credential clarification agrees with observed implementation. SD-03 partial rejection remains an unmet implementation claim until F-R2-01 closes. Do not weaken the contract or publish canonical/ledger in review. No gameplay fallback, budget/tolerance increase, new skipped tests, SDK/asset mutation or out-of-scope implementation found.

This is the **second consecutive final needs_fix** for Plan853, across commits760c232f4 and5e3f5b65f. Before the next execution_done handoff, work must explicitly diagnose the completion-state oracle and coverage adjustment:

| Remaining invariant | Why earlier checks missed it | Affected states | Required diagnosis/oracle adjustment |
|---|---|---|---|
| Every observed announced case must finish exactly once inside its owning block | R1 focused pending binary headers; repair tests cover header/count/summary and unowned blocks but not pending case at summary/new case | parent/child case pending, zeroDoc, next case, block exit | explain transition ownership and reject discarded pending states; replay R2 cases plus valid nested/zero/new binary and original raw denominator before readiness |

Next: work in the same tree, repair and commit the bounded parser/oracle gap, diagnose the repeated miss, then affected independent re-review. Original source/API repairs stay closed unless their inputs change. Unified gates remain merge-owned pending; no landing or delivery pass.
