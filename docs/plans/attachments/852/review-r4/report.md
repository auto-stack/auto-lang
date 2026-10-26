# PLAN-852 r2 — T-07 final re-review R4

**outcome: needs_fix**. Same independent reviewer context as R1–R3; no implementation edits or subagents.

Baseline: `0cc4e2f7e808ed4f3396db954546cc4e05d682b7`, `codex/plan-852-dev`, `D:/autostack/.wt/lang-852/auto-lang`. Previous baseline `545b727dd17b6c19232a744b01e231ad5f83f8ef`; substantive repair `87ec33640`. Dependencies unchanged: auto-ui `b4fe0000e56cddaf7d3cf70e348431f2258aacde`, auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`. Tracked implementation clean; existing untracked probe-t01 retained. Source/oracle/runner/stream/identity implementations unchanged except header comments and additional in-memory negative cases; unrelated merged Plan822/853 documents excluded from repair scope.

## Closure / remaining invariant

R3's exact produced_by.unknown_field counterexample now rejects in both consumers (r3-replay.txt, certify_exit1/snapshot_exit1). The expanded Rust in-memory schema test selects1/pass1 (schema-test.txt). Changed Rust formatting passes (fmt.txt); implementation diff --check clean. Key spelling, lowercase hash and negative created_at guards added to daily consumer were inspected. The R2 actual compiler/cleanup and earlier stream/implicit-certification/oracle evidence remain valid and are not rerun.

### F-852-R4-01 [P2] Daily case/module fields still lack scalar string validation

Continuation of R2-B / F-852-R3-01, not a new scope promise. Affected AC-04/06/08, T-03/T-06. Existing rule: exact typed receipt schema, with the same invalid inputs rejected by both consumers.

Input: copy committed B01 certificate and change only `case` from `"B01"` to `["B01"]` and `module` from `"m1_b01"` to `["m1_b01"]`. Keep source/golden/reference/context and all other receipt fields unchanged.

Command: frozen ace-certify --corpus <copy> --only b01; then documented Windows PowerShell5.1 snapshot with that copied RepoRoot and frozen AUTO_CASE_BIN. Reusable full reproduction: replay-array.ps1 (isolated fresh temp corpus only). Actual raw result: case-module-array.txt.

Expected: wrong JSON field types reject in both validators.

Observed: Rust reports `REJECTED ... case is not B01`, exit1. Snapshot reports PASS,26 a2r checks/10 STRICTLY valid certificates/97 full-suite checks,exit0. Legitimate scalar receipt snapshot control is recorded separately in valid-snapshot.txt; original produced_by rejection remains intact.

Established cause: script line `if ($cert.case -cne $row.id -or $cert.module -cne $module)` has no `-is [string]` guards. For a collection left operand, PowerShell -cne filters elements instead of producing scalar typed equality; a single matching string leaves an empty collection, false in the if. Changing -ne to -cne fixes case sensitivity but not JSON type enforcement. Rust's as_str rejects the arrays.

Closing oracle: independently reject array/object/null/number/bool for each declared scalar string field, accept exact valid strings, and compare the same corpus of malformed receipts across both validators. Avoid patching only these two example values; inventory every declared field's type predicate and exact-key/value rules. This is the previously requested full typed-schema invariant. No global suites needed; use a direct production-validator seam for the matrix where possible, with one real snapshot negative and valid control. Preserve previous produced_by/flags/deps/key-case/hash/date negatives and explicit-only certification behavior.

## Still-unfinished SD text correction

The work handoff claims SD-02's old identity description was fixed, but the implementation diff contains **no change** to docs/specs/auto-test-case/a2r-golden-certification.md. Its resource paragraph still says default tool identity comes from build.rs, only --rustc overrides are measured. This contradicts actual behavior and the same draft's earlier actual-compiler paragraph. The ace_certify.rs top header was corrected, but BUILD_TOOLCHAIN_IDENTITY's doc comment still claims it supplies the default certificate identity and avoids certifier identity spawning. Fix these remaining exact locations; do not change the repaired implementation to match stale prose.

Frozen prepared drafts copied here; hashes in spec-hashes.txt. SD-01 unchanged from R3 (`9f668f89f179ee63979d40b11eea0c106f124302d0a1ea5808248f6bc2f405b6`); SD-02 unchanged (`a70e18b9bc7e6eb2b905da23d0a3665e515898f108bad4224c196ea052d65255`); decision hash `c0f6fe811c57adfe28fddd7d481440c88aa0865a6414d5af5df49496d186b75ea`. No canonical Specs/ledger publication by review. Existing SD-01/04/05 and touched_goals rationale retain prior acceptance; SD-02/03 strict-daily claims remain blocked.

## Acceptance / routing

Retain R3 AC-01/02/03/05/07 pass, including complete fixed oracle/source audit and supported baseline. AC-04 partial,AC-06 fail,AC-08 partial due to F-R4-01 and stale delta; AC-09 pending merge. Reopen only T-03/T-06; completed T-01/T-02/T-04/T-05 remain completed,current_step4,plan_revision2 unchanged. Next: work in same tree, diagnose why the type audit stopped at case sensitivity/selected fields, repair the full shared invariant, commit and re-review only affected controls. No merge/archive/cleanup.

Bare cargo t and actual applicable surface gates remain merge-owned pending after final review pass; cargo tf batch-only main. No whole-tier tests or docs_gen ran this round. Final pass cannot be granted merely because each historical example now rejects while a same-contract wrong-type sibling remains green.
