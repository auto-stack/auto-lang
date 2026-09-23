# PLAN-696 T-08: implementation verification audit

## Plan-scoped gates

| Gate | Result | Evidence |
|---|---|---|
| `cargo check -p auto-lang` | Pass | Exit 0; compiled in 8.44s and reported 349 library warnings. The warning locations in edited `engine.rs`/`native.rs` are in existing code outside the changed blocks; no warnings point at the new HTTP/SSE code. |
| `cargo th` | Pass | 51/51 HTTP E2E tests passed, including `http_e2e_back_proxy_real_031_native_ns_session` (the earlier T-02 run's `names` empty failure did not recur). |
| `cargo tv` | Fail with unrelated P-053 tests | Exit 100 after 821.979s: 1,521/5,620 run, 1,518 passed, 3 failed, 508 skipped, 4,099 not run after nextest cancellation. The 3 failing assertions are listed below. The 70-page `widgets-gallery` compile test completed as the single slow test. |
| `cargo tf` | Fail with the same P-053 tests | Exit 100 after 830.402s: 1,417/5,473 run, 1,414 passed, 3 failed, 112 skipped, 4,056 not run after cancellation. |
| `cargo fmt --all -- --check` | Blocked by existing formatting drift | Exit 101; emitted file diffs beginning in the detached `auto-down` dependency worktree and many unrelated files. No formatter was run in write mode. `git diff --check` passes. |

The three full-gate failures reproduce individually in this worktree:

1. `tests::musk_vm_track_tests::musk_vm_track_p053_1_widget_computed::widget_computed_passthrough_survives_reeval` — computed value returned 0 rows instead of 2.
2. `tests::musk_vm_track_tests::musk_vm_track_p053_1_widget_computed::widget_computed_store_arg_helper_chain` — computed helper chain returned 0 rows instead of 1.
3. `tests::musk_vm_track_tests::musk_vm_track_p053_4_merged_api_warning::merged_mode_api_call_emits_warn_opcode` — expected merged `CALL_NAT 3142` was absent.

All three tests live in `crates/auto-lang/src/tests/musk_vm_track_tests.rs`, which is unchanged by Plan 696. The first two also failed in the isolated targeted rerun; the third failed in its isolated targeted rerun. The HTTP changes' LocalSet/cooperative SSE tests, API generation tests, and all `cargo th` tests pass. Treat these failures as review blockers to triage separately; this execution did not alter their unrelated UI/compiler behavior.

## Static health audit

- `git diff --check`: pass.
- New HTTP request parsing tests cover split TCP writes, malformed/short/oversize lengths; existing multipart and body binding coverage passed during T-03.
- Search of the edited HTTP entry paths found no `Arc/Rc::{into_raw,from_raw}` or `AutoVM` pointer-to-`usize` transfer. Remaining `as usize` uses in `http_server.rs` are numeric indexing/length conversions.
- Diff scan found no `dbg!` or ad-hoc print debugging. The SSE generator error diagnostic and the existing generator instruction-budget diagnostic are intentional failure-path messages.
- Existing compiler warning volume remains high: 349 for the library check, 477–479 in library-test builds, and 30 in the `auto-man` library-test build. None points at the new HTTP/SSE implementation; the changed `api_gen.rs` helpers also introduce no test-build warning.

## Spec and review handoff

T-01 inventory, T-04 owner decision, T-07 parity, and this verification audit are available in `docs/plans/reports/`. SD-01..03 are still proposed deltas in PLAN-696; independent review must check the source claims and decide the Spec edits before the merge workflow deposits them. This work phase ends at `execution_done`; it does not perform review, Spec deposit, archive, merge, or worktree cleanup.
