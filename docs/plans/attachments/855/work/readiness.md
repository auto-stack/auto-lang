# PLAN-855 work readiness

Work-owned T-01..05 / AC-01..05 pass; execution_done5/9. Independent final review and merge gates remain pending. Authoritative tree D:/autostack/.wt/lang-855/auto-lang; plan-855-dev; base523d27b863dbe306c89af0ea546334225a29f8e9, code70f1addf0. Fixed SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 (681 files); AGP/two locks/API/assets/samples unchanged. Source only eventually lands autographics-dev. No root Cargo tiers, full driver suites, merge, canonical/ledger publication, archive or cleanup performed.

## Acceptance / actual observations

| AC / V | Real path and distinguishing oracle | Current evidence |
|---|---|---|
| AC-01 / V-06 | Delivered853/854 -> own worktree -> current complete source/SDK/dependency closure | Pre-change447 sources +681 SDK accepted. Current455 sources all match committed Git blobs; omitted each of six moved modules, SDK manifests/GLB and unlisted source refuse, restore accepts. Own-SDK consumer455 files match producer. |
| AC-02 / V-01/04 | Panel stamped message -> Host commands -> same Editor pending/accepted/history; Play adapter -> same Play VM -> prepared whole-frame publish | responsibility-map.md traced owners, cancellation and last-pin boundaries before edits. Original59 Host/Panel methods and exact Host/Panel fields checked; only private parent visibility/equivalent qualification/rustfmt changed. Play/session/interaction primitives untouched. This textual comparison supplements semantic tests, not a Rust proof. |
| AC-03 / V-01/02 | Existing Editor/storage/project/Host/interaction tests -> actual Auto/GPU/disk reopen | Producer59 cases in14 named binaries pass: stamped stale/busy/cancel/late input, dirty/history/selection, disk/receipt and gesture routes unchanged. Native interaction success plus wrong-target phase1 and missing-release phase4 reject; restored ordinary behavior covered by positive and unchanged tests. |
| AC-04 / V-03/04 | Real Play/Host,128 wide/deep, actual Weak/Drop + explicit external pins | Starting cancel, Pause/neutral Step/Resume, sticky fault/last-good/Stop/new Run, twenty owner cycles pass. Six actual retained code/VM/Descriptor/rescan/fresh-upload/wrong-content env mutations hit original assertions; restored owner case passes. No source mutation persists. |
| AC-05 / V-05 | Original CLI parser/report/exit + App explicit probes -> same real Host/OS | Five original completion unit cases + motion_cli/authoring_probe_exit included in59 pass. Partial states still fail before report; default observation remains legal. Actual native interaction path, positive and original-stage negatives pass. |
| AC-01/05 / V-06 | Current package -> own SDK/initial empty target -> real consumer tests | Independent consumer identity1 + actual motion Host4 pass; no producer target/rlib copied. Metadata/SDK and455 exported hashes equal producer. A verifier message/unused-import-only update was copied explicitly after export; Rust/dependencies/config unchanged (consumer-source.json.txt). |
| AC-04/05 / V-06 | Inherited853 tool/fixtures -> preserved executable ->50 fresh processes/1500 accepted warmticks | Each semantic/source/pixel/resource/owner record equals both853 and854. All21 generated output files byte-identical, ten LF/CRLF/Unicode/local-name fixtures retain independent credentials/regions. Baseline sensitivity controls and raw-data/summary replay pass. |

Actual commands, IDs and raw log hashes: scoped-receipts.json.txt and handoff.json.txt. Current verifier negative/restoration evidence: source-controls.log. Seven permanent evidence-tool cases pass with actual inherited generated fixture; original assertions retained, current identity selection updated. cargo check and package fmt/check/diff pass; fixed SDK146 pre-existing warnings, driver0 new.

## Comparable measurements

| Workload | Tick P50 old853/new855 ms (off) | Tick P95 old853/new855 ms (off) |
|---|---|---|
| move | 2.577/2.702 | 3.636/5.097 |
| maze2d | 3.941/4.156 | 5.165/6.422 |
| corridor3d | 4.175/4.352 | 5.268/5.415 |
| wide128 | 35.520/37.987 | 38.077/55.104 |
| deep128 | 32.937/34.725 | 34.834/49.430 |

Full old853/854 and current named clocks, off/on, per-process values and working-set peaks: performance-comparison.json.txt. All hard budgets, managed bytes and actual ownership/resource/semantic results held. Measurements are somewhat slower; no acceleration claim. Same machine/adapter/toolchain/flags/locks/inputs, five processes per workload/mode with5 warmup+30 measured accepted ticks. Process-cold does not mean disk-cold; working set samples10ms can miss short peaks. Different execution time/background/caches (including consumer compilation overlapping part of sampling) limit causal attribution; no hard performance threshold was relaxed. CPU intervals overlap and are not summed. This is a structural maintainability refactor, not evidence of improved latency.

The sampler began before implementation commit: its preserved receipt honestly records the then-base HEAD. The actual immutable identity and all455 committed blobs match the measured inputs, and no code/dependency/configuration/input changed through commit. The original runner is preserved by SHA; actual child execution environment is recorded separately from parent wrapper environment. No rerun merely for HEAD change.

## Scope, attempts and next role

Original test files and business oracles were not changed. Host child impls avoid a second adapter owner/state/cache; public signatures/field layout are preserved. CLI/probe methods move without bypassing real consumers. Play retains its whole single scheduler; no benefit justified splitting its state machine. New layout and current limitations are in README; proposed SD-01..04 in spec-delta.md remain unpublished.

Initial tool test attempt lacked M2_GENERATED_FIXTURE and copied the old854 closure; corrected by supplying retained actual generated output and choosing current evidence.CURRENT, then all7 passed. Textual move checker initially reported rustfmt trailing commas/braces in two expressions; normalization of these inspected exact changes passed. A receipt helper initially called check_selection without expected and was corrected to selected_tests; no failed helper output was accepted as evidence. These local command failures did not require semantic changes or weaker oracles. Initial tool errors were returned in the session; later successful raw logs are preserved, not invented earlier logs.

Open in-contract findings: []; mandatory work coverage gaps: []; unresolved contract decisions: []. Work implementation is committed and clean, writes paused for independent review. This is implementation self-check, not independent final acceptance. Next role: independent review T-06 in this same tree. T-07 complete producer/fresh consumer and original native/owner/content/AGP/current-source gates remain merge-owned; original85-group/current mapping must be rebound and reviewed for reuse. T-08 publication/Spec+ledger/archive and T-09 guard cleanup remain pending. No new M2 capability or production I2/I3 completion claimed.
