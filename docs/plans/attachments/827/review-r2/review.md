# PLAN-827 final repair review R2 — needs_fix (2026-10-25)

Independent reviewer: same review chat as R1, separate from implementation. No agents or code mutation. Scope: R1 closure diff plus affected controls/invariants; incomplete readiness prevents final certification of the pending Plan-wide acceptance/SD review.

Baseline: main bookkeeping d76b2691def0029f82d777f8775cf0cc39022dc6; Plan r3 SHA256 before this receipt CEE91CBCD1A530B5A856BB439951450E59502D4B728B23E0D3736AC25BF50E4E. Existing unique worktree D:/autostack/.wt/lang-827/auto-lang, branch plan-827-dev, reviewed commit 4531ece17e20712b1b87692c8d21d897bb99d897, repair base bd396d732cc82883b1e30e6be7e1f80a5df853d1; original base 369e1c349ea9587907f073aaecd3082e2bb486bd. Worktree clean before/after. D798/D828/public implementations unchanged by repair; diff only tests, script and inventory artifact. No global cargo tiers run.

## Closure decisions and evidence

### F-827-R1-01 remains open / P1 / AC05,06 / T06

Resources is now present in acceptance and selftest correctly detects its five synthetic bad shapes (independent `powershell -File scripts/verify-av-source-827.ps1 -Scope selftest`: exit 0). However, the validator now mistakes nextest's filtered-out global population for required cases skipped. R1's rule was about REQUIRED cases; its skipped-summary counterexample was not a requirement to run every unrelated test. Contract-decision r2's blanket skipped>0 rule is an invalid interpretation of that requirement.

Production replay in unique tree: `powershell -File scripts/verify-av-source-827.ps1 -Scope inventory` exits **1**, despite both required tests passing. Raw output retained in inventory.log.txt:

```
Starting 2 tests across 8 binaries (6926 tests skipped)
PASS ... av_source_827_inventory_negative_controls
PASS ... av_source_827_inventory_denominators_replayable
Summary [0.085s] 2 tests run: 2 passed, 6926 skipped
FAILED: inventory has 6926 skipped/ignored cases (skips are never green)
```

Expected: legitimate required inventory population passes; dropping/ignoring one of those required tests fails. Cause established: Test-Selected matches `(\d+) skipped` globally without consulting selection membership. This affects all scoped Run-Runner-Tests filters. Still no required test-name manifest/count binding; deleting a required test while retaining one passing test is not detected by this summary-only check. Corpus duplicate-check rejection is useful but covers a different layer.

Close with per-scope required test IDs/selection evidence that distinguishes excluded unrelated tests from ignored/missing required cases. Replay real inventory positive through the actual script, required ignored/missing/duplicate controls, and failed scope propagation. Preserve resources aggregation. No global suite required.

### F-827-R1-02 core detector gap closed within reviewed repair boundary

Independent targeted commands, cwd experimental/ac-av-probe:

- `cargo test -p auto-av-probe --test av_source_827_incremental av_source_827_incremental_detector_negative_controls -- --exact --nocapture`: 1 pass, 4 filtered, 5.82s. Actual fresh warmup creates correct-valued but identity-changing snapshots; identity detector rejects >=100 unchanged object replacements. Same-content dual warmups publish equal content versions/different pointer identities. Legitimate edit accepted.
- `cargo test -p auto-av-probe --test av_source_827_incremental av_source_827_incremental_d828_baseline_30_structure_and_p95 -- --exact --nocapture`: 1 pass, 4 filtered, 9.78s. P50=210.6ms, P95=401.8ms. New assertions check retained code ptr/generation and rescan/store-write facts across 30 edits.

These close the specific R1 absent detector/identity observations. They do not independently certify all still-pending V13 lifecycle/dependency/raw evidence or pending full Plan acceptance. No widening to global campaign.

### F-827-R1-03 remains open / P1 / AC01,06 / T01,T06

Repair materially improves the inventory: 714 durable rows and negative controls, both inventory tests independently pass (script failure above is separate). Yet original contract-decision §4 and R1 explicitly require stdlib/auto file inventory in addition to providers, and program identity records. Independent JSON grouping: 23 ap-seed, 14 ac-fixture, 47 vm-corpus-category, 517 vm-corpus-case, 37 stdlib-provider, 43 capability, 10 entry-point, 10 gate, 13 entry-call-site. There are **zero stdlib/auto member rows**, while `Get-ChildItem stdlib/auto -Filter *.at` finds **102** files. Repair source never scans that directory. G0's full stdlib consumer inventory cannot be replaced by 37 hand-declared provider rows.

All 517 vm-corpus-case rows have directory paths with empty sha256, including 01_basics/001_hello; scanner only tests source/sidecar presence, not bytes/identity. Thus editing a source keeps that row identical; read/parse failures of an existing source/expected file cannot be surfaced as promised gap rows. Entry discovery remains 13 fixed substring anchors and omits previously named service/man anchors. A total row count does not prove the full denominator.

Close the existing R1 invariant: enumerate stdlib files/actual consumers with origin/status/owner, bind discovered program source and expected identities, preserve IO/parse/no-oracle gaps, reconcile actual entry/SKU coverage to named EP categories. Add affected missing/unreadable/source-change controls. Do not merely update the claimed denominator to the current narrower artifact.

### F-827-R2-01 new health regression / P2 / AC07 / T09

Both independent probe commands emit new `unused_mut` warnings at av_source_827_incremental.rs:459,482,483 (s2, sa, sb), introduced by repair. These are not baseline root workspace warnings. Repository independent review health gate requires no unhandled new compiler warnings. Remove unnecessary mutability in work, replay the affected detector check; no broad tier needed.

## Acceptance / next

AC01 fail F03; AC06 fail F01/F03; AC05 remains partial with invalid aggregate; AC09 targeted detector repair passes, full pending coverage retained from R1; AC02/03/04 prior inspected evidence unchanged, not promoted to full certification. AC07 needs_fix; AC08 pending merge. Reopen T01/T06/T09 (latter only new warnings; detector core finding closed), executing current_step=4/9; T02..T05 retained. No in-contract defect is accepted as deferred debt.

Frozen SD01-06 identical to R1, SHA256 FF071C925FB768404C0A09435B0AF5B8DEC838E183858FBDEB0CC02753A35130; proposed paths/GOAL-002 retained, final normative delta audit still pending. No canonical Spec/ledger mutation. Final frozen bare cargo t and applicable workspace/surface gates remain merge-owned pending after acceptance review; tf batch-only. No merge/archive/cleanup.