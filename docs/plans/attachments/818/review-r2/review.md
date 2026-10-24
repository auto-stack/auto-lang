# PLAN-818 R2 final review — needs_fix (2026-10-24)

Independent repair review, same chat as R1, no implementation writes. Plan r3. Worktree D:/autostack/.wt/lang-818/auto-lang, branch plan-818-dev, HEAD a7440f0c7211e2f69f427ec5799011366388bffa (80ca3e5e1 repair plus acceptance-doc commit); base 828237e56b288b1770d93c5a96969ff72a2379fb. Clean before and after. Documentation-only followup included explicitly rather than reviewing a stale named HEAD.

## Prior finding closure
- R1-01 PARTIAL: edit timer now wraps compile/persist/link/run, closing the original missing-compilation mechanism. However formal series test still takes each worker's obs.wall_ms, not parent admission/comparison latency: literal source copy/search/insert preparation occurs before session.edit, worker response serialization/IPC/parent admission and expected comparison occur after this timer. The agreed §5.5/AC11 interval includes request preparation through parent comparison. The 514ms is not yet evidence of that whole interval. Retain continuous worker evidence but add the contracted parent-observed gate; don't sum unrelated/discontinuous timestamps or silently weaken the interval. Debug report-only versus release hard gate is acceptable in principle; build mode does not resolve timing boundaries.
- R1-02 CLOSED for normal mapped history-hit mechanism: independent test mir_native_incremental_degenerate_modes_gate_nonzero passes 1/1, 1.44s, preserving 100→200→100→200 expected results with zero whole fallback and history reuse, followed by explicit drop_unit_history injection. No claim that this local closure covers the new formal trace/fallback regressions below. The local injected gate is not yet independently exercised as a failing actual formal-parent gate.
- R1-03 wiring gap CLOSED: actual worker creates MnIncrementalSession, adapter dispatches edit sequences, and independent real runner test mir_native_incremental_formal_runner_gate passes 1/1 (8.365s). Integration behavior is not accepted due to new findings below.
- R1-04 CLOSED: cargo fmt --manifest-path experimental/ac-mir-native/Cargo.toml -- --check and git diff 4e9e03cb0..HEAD --check pass. Separate compiler-warning regression remains.

## New repair regressions

### F-818-R2-01 / P1 / AC03, AC07, AC11 / T11: incremental trace cannot link and trace bytes are discarded
Durable replay: replay-trace.ps1, trace-output.txt. Legal scalar source helper(v) calls authorized mark_a, then returns v+a; main returns helper(10); steps edit return v+a to return v+a+1, then run. caps=[hir.test.trace.v1]. Expected final exit=12 and stderr byte 61. Observed warmup fails mn.worker.incremental, rust-lld undefined symbol hir.test.mark_a.
Control: same source and caps, steps=[run], exit=11 and stderr_hex=61 (trace-control-output.txt). Thus source, SDK and support library are valid; failure reaches new incremental link path. Root cause established: worker obtains tracelib but discards it with let _=&tracelib; incremental run_current calls link_and_run(..., None,...). Additional same-invariant defect: EditObservation contains no raw stdout/stderr, and incremental worker hardcodes empty output strings. Merely passing the library will still lose observable trace. Close with authorized trace through real runner including cold/edit/history-hit and exact byte-order parent comparisons, unsupported/unauthorized controls unchanged.

### F-818-R2-02 / P1 / AC03, AC07, AC11 / T11: later syntax fallback loses already accepted edits
Durable replay: replay-prefix.ps1, prefix-output.txt. Source:
```
fn helper(v int) int {
    return v + 100
}

fn main() int { return helper(1) }
```
Steps: edit `return v + 100`→`return v + 200`; edit `helper(1)`→`helper(2)`; run. Expected 202; observed success warm_edit_fallback=true, exit=102. First-step-only control (prefix-control-output.txt) proves actual incremental engagement and correct result 201. The second edit is legal; NoConvergence triggers new fallback. Established cause: serve_incremental invokes serve_incremental_whole_fallback with original &source and only steps[st_idx..], dropping committed prefix; function reconstructs from original source. This is a result defect, not merely incrementality labeling. Close by replaying whole sequence correctly (including dependent from literals introduced by prior edits), first-step fallback and no-fallback controls, plus failure/diagnostic propagation; preserve accepted prefix exactly once.

### F-818-R2-03 / P2 / AC09 / T08: new unhandled compiler warnings
Independent targeted test emits new warnings in incremental.rs:340 (initial minted_fp assignment never read), mn-worker.rs:247 (unneeded mut), incremental_tier.rs:74 (unused gate_ms). Separate these from inherited main-workspace warnings. Required health gate remains zero newly introduced warnings. Close using targeted check/test build, no whole suite.

## Commands and evidence scope
- cargo test --manifest-path experimental/ac-mir-native/Cargo.toml --test incremental_tier mir_native_incremental_degenerate_modes_gate_nonzero -- --nocapture: 1/1 pass; above warnings.
- cargo t -E 'test(mir_native_incremental_formal_runner_gate)': 1 selected / 6899 skipped, pass. This is a targeted test, not bare cargo t/full tier.
- Native worker replays and controls above used release worker rebuilt by the real runner's always-build path. Worker identity/hash saved separately. Scripts only create test artifacts under implementation target; implementation remains clean.
- No aggregate acceptance, full standalone suite, resources/churn/performance matrix or whole cargo tier run after known blockers. No new performance pass or claim that actual full P95 exceeds threshold.
- R1 independently established codegen6/parity2/cache7 results retained to their unchanged semantic coverage; no replay of unrelated baseline families. New wrapper/link/session failures invalidate affected integration assertions.

Acceptance: AC02/03/04 prior scoped evidence preserved; AC03/07 now fail on new integration cases; AC11 fail/partial (timing residual plus integration); AC01/05/06/08 retain R1 stated boundaries and submitted work evidence, not blanket certification. AC09 fail, AC10 pending. SD-06 proposed formal-path/trace/history/timing statements require correction after fixes; no canonical Specs or ledger modified. Current Spec-input hashes and frozen delta saved alongside this report; touched_goals empty rationale remains closeout requirement.

Next: same-tree work repair all open findings and timing/actual-formal-negative-control requirements, commit, resubmit. Reopen T08/T11, status executing 7/11; preserve all history and unaffected tasks. Final pass would still leave bare cargo t + applicable surface/dedicated workspace closeout as merge-owned pending gates; batch tf on main only. Review does not merge/archive/clean.
