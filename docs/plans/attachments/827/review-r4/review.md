# PLAN-827 final repair review R4 — needs_fix (2026-10-25)

Independent reviewer: existing review chat, not implementation context. This round closes R3's last validator finding and continues the previously explicitly pending initial acceptance audit. No global suites, implementation edits or merge. Newly found native-oracle gap predates the repair; it is NOT a regression caused by exact-ID matching.

Baseline: Plan r3, main aac01dad0014ad479fb12693e41455c47912f435; Plan pre-review SHA256 4EC00240B2C99D9DC93DADFDA1FDEF8A9559DC04EC08C260BD85E3B910788A48. Unique implementation tree D:/autostack/.wt/lang-827/auto-lang, plan-827-dev @438a700d0af6dbb4744b0b9c6e6ae727e1d358ca; repair base 25c2ac9fb7905b1d102a97ea8ba23db2906aae3d; original base 369e1c349ea9587907f073aaecd3082e2bb486bd. Clean before and after. Repair diff is only verification script exact-ID logic/selftests; runtime, dependencies, fixtures, test configuration unchanged. Do not consume stale Plan copy in implementation tree.

## Prior findings closed

F-827-R1-01 now closed: exact line-end ID and one-match binding; duplicate required declarations rejected. Independent selftest passes prefix/suffix/version collision matrix, exact positives, duplicate PASS/list, real skip/ignored/missing and child-failure controls. Original R3 AST replay yields legitimate=True, absent=False, renamed=False. Real `powershell -File scripts/verify-av-source-827.ps1 -Scope inventory` exits 0; 2 passed, 6926 filtered/skipped; required 2 all passed. Durable raw inventory.log.txt. Resources remains in aggregate. This closes the identified R1/R2/R3 validator defect; it cannot prove runtime test bodies contain no silent success exits.

R1-02, R1-03, R2-01 remain closed within R2/R3 recorded scope. Only script changed since R3; prior 818-row inventory/IO failure controls, native actual source/identity and incremental P95=401.8ms evidence is reused where unchanged. No unnecessary performance matrix replay.

## F-827-R4-01 / P1 / original acceptance defect / AC02,03,06 / T03,T04,T06 / V01,V03,V10

Existing requirement: Plan §5.5/AC06 explicitly says missing required tools cannot pass; V01/V03 require independent native channels, including AP23 native exit70 vs guest trap separation. `av_source_827_first_case.rs:188-190` and `av_source_827_semantics.rs:99-101` check `find_rust_lld().is_err()`, print infra-blocked/skipped, then **return normally**. Libtest marks these REQUIRED tests passed with zero ignored. The aggregate only sees passing IDs, so its exact-ID guard cannot detect this missing oracle.

### Bound reproduction

Durable replay: `powershell -File docs/plans/attachments/827/review-r4/replay-native-missing.ps1` from main. It changes only the child process environment, restores it in finally, and never renames/removes a real tool. Input override AC741_RUST_LLD points at a nonexistent file in the Plan tree. `experimental/ac-core/src/link.rs:648-657` takes this supported override and returns an error for a missing file; it does not silently search a fallback tool. Both test inputs still read the frozen AP source before tool lookup.

Commands in experimental/ac-av-probe:

```
cargo test -p auto-av-probe --test av_source_827_first_case av_source_827_first_case_native_cross_check -- --exact --nocapture
cargo test -p auto-av-probe --test av_source_827_semantics av_source_827_ap23_native_exit70_channel -- --exact --nocapture
```

Actual in native-missing.log.txt: both print `infra-blocked: rust-lld unavailable ... skipped`; both report `1 passed; 0 failed; 0 ignored`, **exit 0**. Expected both required validation runs nonzero/explicit blocked, never acceptance green. This is actual tool-discovery failure along the production test path, not an artificial summary string.

Independent legitimate controls after restoring environment: same commands each pass with actual native builds/runs (AP01 0.77s, AP23 1.21s), no warnings. Therefore the finding is a false-success branch for unavailable required oracles, not an inability to execute the native reference on this machine.

### Closure oracle and minimal replay

Make missing mandatory native tools fail/block required validation; do not convert the run to an ignored or success-returning case. Keep independent native actual results separate from guest trap outcomes. Add missing-tool controls covering BOTH native branches and retain legitimate AP01=0/AP23=70/false-only=0 controls. Ensure the actual selected-test/scope caller propagates the missing-tool failure to a nonzero overall verdict. Scope is native channels + caller propagation; no broad suite, complete benchmark or unrelated incremental replay needed.

## Coverage, knowledge delta, routing

This is continuation of the pending initial audit, not a reopening of closed inventory or exact-ID repairs. AC06 fail on F-R4-01, AC02/03 native-oracle negative readiness partial. Preserve all prior scoped positive evidence. V04-09 complete source/store malformed-admission and resource/lifecycle evidence reconciliation, V13 full remaining dependency/cleanup raw, and final SD01-06 normative-text audit remain pending, not implicitly passed or moved to merge. Implementation work should reconcile these named pre-existing acceptance rows and bind inherited D798/D828 evidence explicitly; an assertion that all scopes passed does not itself close them. No global run is requested to repair the known blocker.

Frozen SD01-06 proposal unchanged, spec-delta-frozen.md SHA256 FF071C925FB768404C0A09435B0AF5B8DEC838E183858FBDEB0CC02753A35130. Existing impact metadata/GOAL-002 retained; no canonical Spec/ledger writes. Final pass not issued.

Reopen T03/T04/T06 for the native tests and aggregate evidence; preserve T01/T02/T05/T09. status executing, current_step=4/9 (work 4/7). After repair commit, review affected native/caller controls and complete outstanding audit. Frozen bare cargo t and applicable workspace/surface gates remain merge-owned pending after acceptance pass; tf batch-only. No merge/archive/worktree cleanup.