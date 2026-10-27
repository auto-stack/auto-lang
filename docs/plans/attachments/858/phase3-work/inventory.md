# PLAN-858 r4 ACE latency policy inventory

Baseline `97ad76691578957b9c35f62dfbc72eb4bda83660`, main `e5f1ccf331adc99b6b37e6717f50577c9b4f803a`; original final review `f098e8d707c4774e0c5a2f7dc459c9065e515488` applies to Phase 2 only. AutoDown `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4` and AutoUI `b4fe0000e56cddaf7d3cf70e348431f2258aacde`; searches of those sibling trees found no P95/1000 ACE contract. Their state must be rechecked before handoff.

| Family | Producer → current consumer/gate | Planned/implemented adjustment |
|---|---|---|
| Main AC/MIR-native | `tests/ac_mn_probe_tests.rs`: parent window/raw and formal session; `verify-ac-mir-native-818.ps1` | P90 on the same raw windows, 2000ms, retain P95/P99 and non-time axes. |
| AV | `formal_worker`, `formal_828`, `formal_campaign`, `bench`, `av_probe_worker`, 822/827/828 tests; `tests/ac_av_probe_tests.rs`; 798/827 scripts | P90 from all raw windows, retained P95, report P90, reject invalid windows; correctness/structure/memory and zero-gate negative retained. |
| CTE | `ac-cte-probe/probe/src/bench.rs` → `cte_probe_worker` → `cte_turnaround_tests.rs` and `cte_snapshot_tests.rs` | P90/2000 release (existing debug smoke bound retained); injection now 4 of 30 at 2500ms so P90 gate actually fails. |
| Semantics | `ac-semantics/probe/src/turnaround.rs`, `sema_turnaround_scenarios.rs`, parent `probe_adapter_turnaround_ipc.rs`, `verify-ac-semantics-796.ps1` | P90/2000 from raw producer and parent; keep P95 diagnostic, admission/tiling, invalid samples and status gating. |
| ABI | `auto-abi-probe/tests/t05_cache_turnaround.rs`, `t06_binding.rs`; parent `tests/probe_adapter_tests.rs` | Existing real C/worker paths, 30+ samples, all outcomes and P95 retained; P90/2000 is gate. |
| AutoDown native/syntax | `crates/auto-lang/tests/plan788_r3_.rs`; `verify-autodown-ac-788.ps1` | Same integration test IDs and real paths; P90/2000 gate, P95 retained as historical diagnostic. |
| Other subsystems | HTTP/UI/graphics independent performance profiles | No changes under this user decision. |

Historical spec-plan summaries remain history, while current contracts and forward-looking design sections require supersession. `docs/design/39`, `40`, `43`, `44`, `strategy/auto-native-backend-evolution.md`, `strategy/av-source-programs-and-avm-replacement-roadmap.md` carry existing unrelated WIP on main; do not overwrite or silently include it. `plan-823-dev` overlaps `av_probe_worker.rs` and `av_collections_822_incremental.rs`; `plan-824-dev` overlaps `tests.rs` and `av_probe_worker.rs`. Reconcile actual tips and affected evidence before integration; do not modify those worktrees.

Review checklist and exact target baseline hashes: [spec delta draft](spec-delta-draft.md). This is an implementation inventory, not a claim that final review, full performance runs, cargo t, or main batch passed.

Revision r4 preserves the r3 user decision and corrects the invented blanket zero rejection: the 788 path records integer milliseconds, so a real sub-millisecond value may quantize to zero. Existing 796 strict positive-sample checks remain.
