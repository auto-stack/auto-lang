# PLAN-847 final review r1 — PASS

Reviewer: `/root/review_847_r1`, independent delegated context; no implementation edits. Plan revision 1. Sole implementation tree `D:/autostack/.wt/lang-847/auto-lang`, branch `plan-847-dev`, reviewed commit **60f0bb03331e716516701fc8189bfd6e6b7244d9**, base **acbca34e3a0d12b671f230a5a219f5d0a9538398**. Implementation paused and clean at entry and verdict. Dependencies are clean, read-only siblings: auto-ui `aee499e87093ff727f68ac8467c506e9ecf6ecdc`; auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`.

## Acceptance map

| AC → tasks | Result | Actual evidence and scope |
|---|---|---|
| AC-01 → T-01/T-02 | pass | Original feature-off raw log contains 44 E0432/E0433, actual old registration leaves probe unguarded. Actual diff moves only its registration directly under the existing cfg. Independent `cargo check --target-dir D:/autostack/.wt/lang-847/auto-lang/target -p auto-lang --tests` exited 0; [log](feature-off-check.log.txt). Cargo manifest unchanged, dependency remains optional; default features exclude test-cases. |
| AC-02 → T-01/T-02 | pass | Independently parsed every `: test` row from both raw lib lists: exactly 2177 IDs, identical as sorted complete lists; old probe 27 and MIR-native 24. [Audit](audit.json.txt) stores complete IDs and source/evidence hashes. Independently reconcile corrected-run PASS 9 and replay PASS 20: disjoint union exactly equals intended 29, replay equals all prior timeout IDs. Inputs, test source, config, thresholds and workers unchanged by commit. |
| AC-03 → T-03 | pass | Independent source, assertion, legal/illegal boundary, sibling, Specs and evidence audit described below; no open work-owned findings or unapproved deferrals. |
| AC-04 → T-04 | pending merge | Frozen bare cargo t in Plan tree; main-checkout single-instance tf/tt/tb batch recovery, attribution, bookkeeping, archive and guard/cleanup remain merge-owned. This pass does not claim delivery or authorize bypassing these gates. |

## Boundary and assertion audit

The only actual code delta is one removal/insertion in tests.rs. Both `probe_adapter_tests` and `ac_mn_probe_tests` now have their own `#[cfg(feature = "test-cases")]`. Their optional imports, crate::test_cases calls, bodies and assertions are unchanged. This rejects the defective feature-off registration while preserving the feature-on modules; it does not default-enable the dependency or suppress tests. Baseline failure replay is reused from work rather than mutating the frozen tree.

Scanned all ten test files referencing auto_test_case, matching work's inventory. Six directly gated sibling registrations and both repaired registrations remain correctly bound; a2r_tests is gated by test-trans and its optional pilot function additionally by test-cases; vm_file_tests remains available while its optional pilot function alone has test-cases cfg. No optional import is leaked by the changed registration. Inspection covers legal feature-on and feature-off combinations, including these nested sibling guards.

Independently ran only three scoped tests under test-cases with inherited CARGO_TARGET_DIR removed and owned target supplied by command argument: probe_adapter_decode_negatives, ac_mn_probe_decode_negatives, and ac_av_probe_decode_negatives. All three pass ([log](sibling-decode.log.txt)); these exercise invalid recipe/binding/step admission in both changed registration paths and a representative adjacent optional consumer. Existing work corpus and negative-oracle tests cover legal paths, real workers and intentional wrong expected outcomes. Inspected Status::Fail/Error/Pass assertions, restart/stale handling and bounded descendant assertions; success banners are not substituted for these oracles.

Original work failures are retained: inherited target layout made worker discovery invalid; corrected run hit the existing 120-second nextest limit during worker preparation; shared-target replay failed before test execution on missing fingerprint paths. Final replay uses existing original worker paths and the tree-owned target, and passes all 20 affected cases in 7.491 test seconds. This evidence closes the bounded feature registration repair, not a cold-worker latency guarantee. No timeout, input, P95, window, ignore or nextest configuration changed. One formal ABI test is excluded by the pre-existing default filter, remains among the 27 registered IDs, and belongs to the pending full batch tier. The other 21 MIR-native tests remain registered; this review does not claim their runtime execution.

Auxiliary artifact correction: work/test-ids.json.txt's `total: 2099` counts only IDs beginning `tests::`; it omits 78 other root/module IDs. The raw before/after logs each contain 2177 complete IDs and are independently equal. The complete durable list is in review-r1/audit.json.txt. This is an evidence-labeling issue, not missing compiled tests; original evidence is preserved.

## Health, scope and knowledge delta

`git diff --check acbca34e3..HEAD` passes. No production change, debug output, test deletion, test body/fixture/oracle changes, config change or workaround. Compiler warnings are pre-existing unchanged-source warnings, retained in both raw and independent logs (including auto-lang lib-test 41); this review does not claim a warning-free repository. No new warning is attributable to the registration movement. No in-contract debt found; no new debt entry needed.

Read AGENTS.md, overview, auto-lang/project.md, and auto-test-case/project.md and architecture.md. Existing optional/default-off, one-way dependency and complete feature-on/off evidence rules match the repaired code. SD-01 has no normative change: empty supersedes_spec_components/new_spec_components/touched_goals are justified; history bookkeeping is not a new product rule. Frozen [Spec delta](spec-delta.md) SHA256 **95c7c287b1d33c137b4f0880cbbccdfd739c318417450bac7f6fca67021f593e**. Canonical source and work evidence hashes are in audit.json.txt. No canonical Spec or ledger was edited in review.

Final recheck: HEAD remains 60f0bb03331e716516701fc8189bfd6e6b7244d9; implementation and both dependency trees clean. No whole-tier t/tv/tt/tb/tu/taa/tf run in review. Verdict **pass**, route to merge; bare t and main tf/tt/tb are explicitly pending.
