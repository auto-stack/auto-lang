# PLAN-827 final repair review R3 — needs_fix (2026-10-25)

Reviewer: existing independent review chat, not implementation context. Repair scope is R2's remaining findings and affected mechanisms. No broad suite, code changes, merge or Spec publication.

Baseline: main bookkeeping 585fdc3e4fbb4a4182dd3bd989c562c1feb6352e; Plan r3 pre-review SHA256 B777C05E32BF27F158A953CFE59D155CCA4F1D96DE89952A38A9F8721F85F181; contract-decision r3 SHA256 DE97DD2F58E518B2522D32D3B4561C040C1D16238AD6888C550E0C027016F6EA. Implementation 25c2ac9fb7905b1d102a97ea8ba23db2906aae3d, repair base 4531ece17e20712b1b87692c8d21d897bb99d897, original base 369e1c349ea9587907f073aaecd3082e2bb486bd. Same unique tree D:/autostack/.wt/lang-827/auto-lang on plan-827-dev, clean before/after. Dependency implementations and test configuration unchanged except four repaired artifact/script/test files. Prior unaffected observations retain their recorded limits.

## F-827-R1-01 partially repaired, still open / P2 / AC06 / T06

Positive original replay now works: `powershell -File scripts/verify-av-source-827.ps1 -Scope inventory` exits 0; 2 tests passed, 6926 filtered/skipped; script reports required 2 all passed. See inventory.log.txt. Selftest independently exits 0, rejecting real SKIP/ignored/absent controls and accepting filtered population. Resources remains included in aggregate.

The new required-ID check, however, uses `PASS.*$esc` without end-of-test-name boundary (script line ~53). It therefore does not bind the exact required ID. Existing contract requires missing/renamed required cases to fail, which its own error text also says.

Independent replay uses the ACTUAL committed Test-Selected function extracted through PowerShell AST; no implementation mutation. Durable command: `powershell -File docs/plans/attachments/827/review-r3/replay-required-id.ps1` from main. It supplies three nextest-shaped passing result outputs, differing only in the reported test name:

- actual exact required name: True (correct)
- unrelated name: False (correct)
- required name suffixed `_disabled`: **True (incorrect)**

Expected third False: the required `av_source_827_inventory_negative_controls` is absent. The suffix case would still match the cargo scope filter, so renaming a removed required test with this prefix defeats the intended guard. This is a validator bug, not a claim that current product execution fails. No requirement for new runtime semantics is introduced.

Close: parse exact selected result IDs (including binary/module identity where needed), bind the full required member name, reject missing/renamed/duplicate required IDs. Add exact positive, unrelated negative, prefix/suffix collision negative and selected skip controls against the same validator used by actual scope invocation. Retain the real inventory filtered positive. Scope: script/selftest plus actual inventory only; no whole acceptance/resource/performance replay needed for this string-matching repair.

## F-827-R1-03 closed for R2 identified inventory repair axes

Source inspection confirms stdlib/auto enumeration and hashes, VM source SHA and oracle SHA, service/man entry anchors; source edit and exclusive-read-lock failure controls now run within the inventory negative test. Independent real scope above runs both inventory tests successfully. Generated artifact has 818 rows: AP23, fixtures14, categories47, corpus517, providers37, stdlib-auto102, C43, EP10, gates10, anchors15. No artifact drift after regeneration. This closes the R2 concrete missing file-member/source-identity/entry anchors gap. It is not a claim of complete future AV support or full old-language replacement. Remaining initial Plan acceptance/Spec review retains its prior pending boundary rather than being implied by this closure.

## F-827-R2-01 closed

Independent command in experimental/ac-av-probe: `cargo test -p auto-av-probe --test av_source_827_incremental av_source_827_incremental_detector_negative_controls -- --exact --nocapture`: 1 pass, 4 filtered, 5.37s. No compiler warnings. Diff only removes the three unnecessary mut bindings; prior R2 30-revision P95/identity evidence remains applicable (401.8ms; no behavior/dependency/config/input changes). R1-02 remains closed within its documented detector scope.

## Disposition

Reopen T06 only; preserve T01/T02/T03/T04/T05/T09 completion. status executing, current_step 6/9 (work 6/7). AC06 partial/fail exact required identity; AC01 inventory repair axes pass; AC09 affected warning repair passes/reuses R2 bounded evidence. AC02..05 and final comprehensive acceptance/SD audit not promoted beyond previously documented coverage; no final pass. After two unsuccessful final reviews, work should verify the closure strategy with the invariant/collision matrix, not just the old example, before resubmission.

SD01-06 frozen proposal unchanged (spec-delta-frozen.md SHA256 FF071C925FB768404C0A09435B0AF5B8DEC838E183858FBDEB0CC02753A35130); impact paths and GOAL-002 retained. Canonical Specs and ledger unchanged; final delta review remains pending. Frozen bare cargo t and applicable workspace/surface gates are merge-owned pending after acceptance pass; tf batch-only. No merge/archive/cleanup.