# PLAN-827 final review R1 — needs_fix (2026-10-25)

Independent review context: this chat did not implement PLAN-827. No delegated reviewer, model identity claim, implementation mutation, merge or broad regression. This is an initial readiness/acceptance audit; concrete work-owned gaps below prevent final pass. Unexamined criteria are pending, not implicitly passed.

Baseline: plan_revision=3; main HEAD=153d1dff5c1b54ef40b5ada91e10c0330d3390c2; implementation HEAD=bd396d732cc82883b1e30e6be7e1f80a5df853d1; base=369e1c349ea9587907f073aaecd3082e2bb486bd. Worktree D:/autostack/.wt/lang-827/auto-lang, branch plan-827-dev verified via git worktree list --porcelain, clean before and after targeted check. D798/D828 consumed from committed baseline; no new cross-repo implementation. Review did not validate every external tool/dependency identity; those remain pending with the acceptance evidence.

## Findings

### F-827-R1-01 / P1 / AC-05,06 / T-06 / V-09,10

Existing rule §5.5 and contract-decision §5: acceptance covers all required subcases; skipped/missing required cases cannot pass. scripts/verify-av-source-827.ps1:144-175 dispatches first-case, semantics, admission, cache, autodown, inventory, compatibility, incremental, but never resources. It still prints `PASS: acceptance (all scopes + SEM material)`. Thus a resource gate failure is outside the purported aggregate. Assert-Selected at lines 23-35 only requires one passing/selected test and ignores skipped population or exact expected IDs.

Independent reproduction (without running aggregate): parse the committed script using System.Management.Automation.Language.Parser; select FunctionDefinitionAst Name=Assert-Selected; Invoke-Expression its Extent.Text; invoke with `test result: ok. 1 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out`. Actual output `selected: 1`, successful return/exit 0. Expected: required skipped case causes nonzero. This is a validator unit counterexample, not a product runtime defect.

Close: connect every required scope or validate revision-bound reusable evidence explicitly, and enforce required case identity/population/no skips. Replay valid, skipped, missing, duplicate and resource-failure controls against the real aggregator. Minimal scope: validator and scope integration; no global cargo tier required.

### F-827-R1-02 / P1 / AC-09 / T-09 / V-13

Existing §5 incremental contract and contract-decision §6 require unchanged HIR/MIR/code object identity/generation, production and IO counters, plus forced whole-frontend/lower/reload or hash-preserving object replacement causing the formal gate to fail. av_source_827_incremental.rs only inspects install sets and selected stage counters; it never observes resident_inventory/phase_io or retained object generations. Test `av_source_827_incremental_degradation_caught` (line 240) performs a normal body edit then an outside-body edit that is refused. It does not force any prohibited degradation and cannot establish the required detector's negative oracle. Conflict on republishing different bytes in T-05 is a different invariant from replacing resident objects with the same hash.

Independent command in experimental/ac-av-probe: `cargo test -p auto-av-probe --test av_source_827_incremental av_source_827_incremental_degradation_caught -- --exact --nocapture`. Actual 1 passed, 0 failed, 3 filtered; 0.03s. This proves the existing normal/refusal control passes, not the absent mutation requirement. Source inspection establishes missing axes; no claim of an observed runtime identity regression.

Close: bind before/after identities, generations and all agreed IO/stage facts on unchanged units; add actual forced degradation and hash-preserving replacement controls that make the production acceptance validator nonzero, with legitimate edits remaining green. Retain raw bound observations for required revisions. Minimal replay V-13 affected controls, no plan-wide regression.

### F-827-R1-03 / P1 / AC-01,06 / T-01,T-06 / V-11

Existing G0 requirement is full replayable item inventory, including owner/gap for failed scans/no oracle; contract-decision §4 explicitly promises JSON items ID/path/SHA256/nature/owner/status. crates/auto-lang/src/tests/av_source_827_tests.rs inventory test instead emits only aggregate counts. It tests 13 entry *file existence* checks, stdlib provider JSON existence/file count, and C/EP substring presence. No provider-entry enumeration, actual call-site/symbol discovery or per-program missing-expected/gap rows are produced. read_dir flatten/filter_map silently suppress enumeration errors. Handwritten 23 seeds and summary counts do not satisfy full inventory, as the Plan explicitly warns.

Close: implement durable item-level discovery output with source/call-site/stdlib consumers and owner/status/gap, retain failures as rows, reconcile denominator drift. Add missing directory/parse/no oracle negative controls and replay inventory only. Existing catalog/oracle materials remain useful but do not close this gap.

## Acceptance and Spec disposition

AC-01 fail (F03); AC-02/03 partial (source implementation/tests inspected, not every runtime subcase independently replayed); AC-04 partial (admission/cache assertions inspected, complete V04-08 not certified); AC-05 partial/fail aggregation coverage (F01; no resource promises waived); AC-06 fail (F01,F03); AC-09 fail (F02). AC-07 review conducted, no final pass; AC-08 merge-owned pending. Remaining V01-12 axes and SD01-06 detailed normative finalization are pending after readiness repair. No deferred in-contract item is accepted as debt closure.

Frozen proposed SD01-06: spec-delta-frozen.md SHA256 FF071C925FB768404C0A09435B0AF5B8DEC838E183858FBDEB0CC02753A35130. Impact metadata paths and GOAL-002 retained. No canonical Spec/ledger writes. Source Spec hashes at review: resident-function-install 80A81F301AF98C0408E009F839C57AAF9C76A2DD4DCD2CED2038F190B94568A1; project C3B14094911FEAE1E69F571741C4C7106299E3ED8AF8E6A5BEC298B8F9326A6C; cache-and-conformance 8CE4C88FAE062629C3FC60D3B5C6488C0EE5F67C0F8F6E478C9F6CB0FDDA7C1B; auto-test-case architecture CB27DCCD643F616B6B79C2C39D1C90EFEBF3D51A6D701D5992EA69A06D90B987.

Next: work repairs in the same tree; reopen T01/T06/T09 (4/7 work tasks retain completion, total current_step=4/9). Preserve historical work claims and other progress. Final frozen bare cargo t and applicable surface/workspace gates remain merge-owned pending after successful final acceptance review; tf batch-only. No implementation fix/merge/archive/cleanup performed.