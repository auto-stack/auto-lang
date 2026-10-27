# PLAN-858 r2 merge attempt R1 — gate failed, not landed

Frozen tested tree: `4d88b1336df00e8919aa77db9df7965a9ebf04ca`; after unrelated main bookkeeping, equivalent rebased tip `ee91830f9aadb6434af84509d6ae343a75b8f55f`. `git range-diff` showed equality for all three commits. Reviewed commit `f098e8d707c4774e0c5a2f7dc459c9065e515488`. No source, test, configuration or dependency change. Auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`, auto-ui `b4fe0000e56cddaf7d3cf70e348431f2258aacde`.

- `cargo t`, attempt 1: exit 100; 83 passed, 20 timed out, 3090 not run. At the time 20 test processes were concurrently waiting for two cold worker builds; 120-second limits were not changed.
- Attempt 2 after original worker builds: exit 100; 895 passed, 1 failed, 4 timed out, 2293 not run. `mir_native_rounds_failure_liveness_and_caps` got session wall rather than expected stdout cap while additional workers built; after worker prebuild an isolated rerun passed (exit 0), and the same test passed in attempt 3.
- Attempt 3 after AV, AC, MN, CTE, sema, ABI release workers prebuilt via original source commands: exit 100; 904 passed, 1 failed, 2288 not run. `mir_native_turnaround_signature_rebuild_gate`: P95 1245.5ms >= 1000ms. Other sessions built concurrently; resource contention is plausible but not established. No waiver or test threshold change.

Raw unmodified logs, external project evidence directory `C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/`:

| File | SHA256 |
|---|---|
| merge-cargo-t.log | `67761596767a57dfbcddaa1667421edc4f6d957840a5e1a2ef5b2a61d94a8320` |
| merge-cargo-t-r2.log | `fab5d75dc40dff04b9b68cd8be0a39075252c939d34c6be45585ff69ba634b42` |
| merge-cargo-t-r3.log | `2b377cc755f572e792bfda99d2f01f56a091766ded9f9725102f2b156e644b86` |
| merge-failure-recheck.log | `034bf16e6810f50a499644004aedcacb81339cb97137b35b54ad96c6529be2cf` |

Generated tracked Plan 818 reports saved byte-identically under `plan-858-phase2/r3-generated-reports/`, then restored in the plan tree. Ledger v99→100 is committed only on the plan branch, not published on main. Restore path: check clean dependency/worktree and current master, rebase with range-diff as needed; when machine is idle recheck the performance case and run full bare `cargo t` on final frozen baseline. Only on pass, guard and ff land; then official main-only complete batch, archive/link preservation and cleanup.
