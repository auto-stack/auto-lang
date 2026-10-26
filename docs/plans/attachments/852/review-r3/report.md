# PLAN-852 r2 — T-07 final re-review R3

**outcome: needs_fix**. Reviewer: same independent review context as R1/R2, no implementation work or subagents. This review used the existing tree; no code was changed.

## Revision-bound baseline

HEAD `545b727dd17b6c19232a744b01e231ad5f83f8ef`, branch `codex/plan-852-dev`, tree `D:/autostack/.wt/lang-852/auto-lang`. Repair baseline `17a959ba097dd7fd247171f7ce43568c75b937ba`, implementation repair `81b474145`; later merge contains bookkeeping. Dependencies unchanged: auto-ui `b4fe0000e56cddaf7d3cf70e348431f2258aacde`, auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`. Tracked tree clean before/after; existing untracked probe-t01 preserved.

Reviewed the two-failure diagnosis in the main Plan before re-review. The separation of actual launched compiler identity from harness build identity addresses the diagnosed R2-A cause. Exact typed receipt checks address filter_map's R2-B cause on the Rust side; the remaining defect is incomplete application to the PowerShell consumer, not a new acceptance requirement.

## Confirmed closures / reused evidence

- **R2-A closed:** replayed the reviewer's unchanged R2 replay.ps1. Fresh RUSTC=nightly certification now records actual 1.90 nightly, not build stable 1.98. Valid receipt reuses; --rustc shim identity/cleanup targeted test passes. Read the new identity_source/default_identity_at_certify axes and daily frozen-profile handling. Actual compiler identity now comes from the contained -vV channel.
- **R2-B original samples closed, invariant still partial:** original flags-null, external [null,17] and unknown-root samples now reject; schema and string flags remain red. `test_cases::ace_certify::tests::invalid_receipts_are_rejected_in_memory` ran exactly 1 test and passed (schema-test.txt). Expanded independent consumer probe below still fails the strict contract.
- **R2-C closed:** original missing --rustc invocation returns exit2 and scratch_still_exists=False (cleanup-replay.txt). The explicit shim/failed-identity cleanup test also passed exactly 1 test (identity-cleanup-test.txt).
- **Formatting closed:** rustfmt --check --edition 2021 over build.rs, ace_certify.rs, stale.rs, ace-certify bin and ace_m1_tests.rs exits0 (fmt.txt). Changed implementation diff --check clean.
- Preserve R1/R2 implicit-certification removal, stream reader/cap/wall evidence, full fixed-source/reference/oracle audit (43 values,9 diagnostics,6 traps), denominator and lifecycle evidence: those mechanisms/inputs are unchanged. Do not rerun those whole families. Current consumer probe also executed all 97 baseline checks successfully.

An initial schema test command used a short name with --exact and selected 0 tests; its raw log is retained as schema-test-initial.txt and is **not** PASS evidence. Corrected full test path selected and passed exactly 1 test.

## F-852-R3-01 [P2] R2-B remains open in daily receipt consumer

Affected AC-04/06/08; T-03 and T-06. Existing rule: frozen contract-decision §5 and R2-B require unknown/missing/wrong-typed nested fields to reject, with consistent Rust and PowerShell consumers. Daily output explicitly claims typed exact-key strict validity.

**Bound input:** committed B01 certificate in an isolated COPY of the corpus; add only `produced_by.unknown_field = "strict schema must reject"`. All source/golden/oracle/context and other receipt values remain unchanged.

**Replay:** `powershell -File docs/plans/attachments/852/review-r3/replay-consumer.ps1` (runs the script from main; default Tree points at frozen worktree). It invokes frozen ace-certify --only b01 on the copy, then documented Windows PowerShell5.1 snapshot with AUTO_CASE_BIN pointing at the frozen worktree binary. No tracked implementation or receipt is mutated.

**Expected:** both consumers reject the malformed receipt; the daily certification-validity axis cannot be green.

**Observed (consumer-output.txt):** Rust returns `REJECTED ... unknown field "unknown_field" in produced_by`, exit1. Snapshot returns PASS, 26 a2r checks, “10 certificates STRICTLY valid”, full suite97 green, exit0. This is a concrete acceptance-gate false positive even though language/source checks legitimately remain green.

**Established cause:** verify-ace-m1-852.ps1:336 checks only that produced_by exists at root; the nested Assert-Keys list at337–342 omits produced_by entirely, and no produced_by value/type checks follow. Rust validates its exact two keys and string values at ace_certify.rs:305 onward. Thus the two declared strict validators disagree.

**Sibling audit boundary:** the PS validator also accepts any signed int created_at without rejecting negatives (Rust as_u64 rejects); Assert-Keys uses PowerShell's case-insensitive -contains comparisons, unlike Rust's exact JSON key matching; PS hash regex/equality are case-insensitive despite a lowercase-hex contract. These are source-inspection concerns within the same typed-schema consistency invariant, not additional runtime findings. Close the shared invariant rather than only adding this one unknown-field example. No new language semantics or capability scope is requested.

**Minimal closing oracle:** valid receipt accepted by both; malformed produced_by unknown/missing/null/wrong-type controls rejected by both; exercise type/range/key-spelling/hash-case boundaries across both validators. Test receipt validation directly where possible instead of replaying all 97 source checks for every field. Preserve no-overwrite-on-invalid and explicit-only certification controls. No whole-tier regression before final review pass.

## Spec delta / omission audit

Frozen draft copies in this directory, hashes in spec-hashes.txt:

- ace-m1-conformance.md `9f668f89f179ee63979d40b11eea0c106f124302d0a1ea5808248f6bc2f405b6`.
- a2r-golden-certification.md `a70e18b9bc7e6eb2b905da23d0a3665e515898f108bad4224c196ea052d65255`.
- contract-decision.md `c0f6fe811c57adfe28fddd7d481440c88aa0865a6414d5af5df49496d186b75ea`.

SD-01/04/05 directions,58-primary/43-reference/5-typed counts and empty-goal rationale retain R2 acceptance. SD-02/03 strict daily-receipt wording cannot pass F-R3-01. Additionally the SD-02 resource paragraph still says default identity comes from build.rs and only --rustc uses contained capture (lines56–57), contradicting the repaired actual identity flow and the same document's earlier new section. Correct this surviving R1/R2 wording before freezing the delta. This is closeout text consistency, not another implementation task.

Comments atop ace_certify.rs also retain the old zero-spawn default identity description; align them with actual explicit certification behavior. Daily sampler prose now correctly says periodic sampling rather than lossless process creation count. No canonical Specs or ledger were published by review. No unsupported language/native/HMR gap was promoted into a new 852 blocker.

## Acceptance and routing

| AC | R3 result | Evidence |
|---|---|---|
| AC-01 | pass | unchanged source/typed denominator and current97-check baseline; actual compiler identity now corrected |
| AC-02 | pass (retained) | R2 per-case independent oracle/reference/diagnostic/trap audit; inputs unchanged |
| AC-03 | pass | R2 actual compiler counterexample closed; current receipts/default profile and real actual/reference certification observed |
| AC-04 | partial | no implicit certification; daily invalid-receipt false positive F-R3-01 |
| AC-05 | pass | denominator/full-M1 distinction unchanged and inspected |
| AC-06 | fail | new certificate consumers disagree on nested strict schema |
| AC-07 | pass (scoped/retained) | original cleanup replay closed + identity/cleanup test1pass; R2 stream controls unchanged |
| AC-08 | partial | independent revision-bound evidence and SD frozen; daily/schema and stale documentation prevent final pass |
| AC-09 | pending merge | bare cargo t + actual applicable surface gates after final review passes; tf batch-only main |

Next: work repairs daily strict-schema consistency and corresponding SD/comments, commits in same tree, then hands off only affected replay/controls. Reopen T-03/T-06; preserve completed T-01/T-02/T-04/T-05 (current_step4). No merge/archive/cleanup. No cargo t/tv/tt/tb/tu/taa/tf whole tier or docs_gen was run; two precisely selected tests passed. All original R2 examples are closed, but that does not close the full cross-consumer invariant.
