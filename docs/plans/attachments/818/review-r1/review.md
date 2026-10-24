# PLAN-818 final review R1 — needs_fix (2026-10-24)

Baseline: plan_revision=3; independent review in this chat, no implementation performed here. Worktree D:/autostack/.wt/lang-818/auto-lang; branch plan-818-dev; reviewed_commit=4e9e03cb077fcb4832ed2a1228cb89cf6d650b59; base_commit=828237e56b288b1770d93c5a96969ff72a2379fb. Implementation tree clean before/after checks. Review does not merge or repair.

## Findings (existing contract, all open)

### F-818-R1-01 / P1 — AC-11/V-12.1/T-11: reported edit latency excludes compile stages
src/incremental.rs:168 starts Instant inside run_current, line 215 returns elapsed. edit (line 225) already rebuilt source, prepared the edit, verified MIR, emitted/persisted COFF and committed before entering run_current. tests/incremental_tier.rs uses obs.wall_ms as its edit→PE P95. Thus 715.5ms is link/run-tail evidence, not the contracted continuous edit→correct PE comparison interval. This is a proved measurement gap, not proof that real P95 exceeds 1s. Close with a continuous outer timer including request preparation, edit compilation, admission and independent result comparison; retain >=30 raw samples and all structural assertions. No whole-tier rerun needed for repair.

### F-818-R1-02 / P1 — AC-11/V-12.4/T-11: ordinary history hit forces whole-module work
Replayed: cargo test --manifest-path experimental/ac-mir-native/Cargo.toml --test incremental_tier mir_native_incremental_degenerate_modes_gate_nonzero -- --nocapture (1 passed, real link/run, 1.21s).
Input in that test: site(v)=v+100 → v+200 → v+100, unchanged main calls site(1), no signature/config/capability changes. Expected: unchanged functions have zero production stages, historical changed unit can be reused. Observed: None unit_mir branch calls whole_module_units on the full new source (incremental.rs:310), and test explicitly asserts fallback_whole_lower=1. This is a normal edit sequence, not injected degradation. Recording the violation and asserting a local gate would reject it does not satisfy AC-11. Preserve that sequence as a positive control with zero full-module work; provide a separate deliberate fault injection whose actual formal gate fails. Trace-capable sibling also matters: fallback currently passes [] caps, losing the warmup grants. Close the mechanism, not just this arithmetic example.

### F-818-R1-03 / P1 — AC-11/V-12/T-11: formal recipe does not consume incremental session
Search of crates and experimental/ac-mir-native/src finds MnIncrementalSession only at its declaration/impl. mn-worker case-run creates MnDriver and calls request at src/bin/mn-worker.rs:175–185. Driver request recompiles the full source/HIR/MIR and splits all functions. AutoDown adapter materializes leading edits before submitting that whole-source request. T-11 tests directly invoke the separate library session; adding that test binary to acceptance does not connect the formal recipe. Existing V-12 requires actual consumer/formal recipe observation of zero unchanged production work. Wire the supported formal session/edit path and prove it through the real runner with parent-side expected result, identity/stage/IO counters, restart separation and failure controls. Keep the old whole-request path if needed for cold/restored scenarios; do not label its warm edits incremental.

### F-818-R1-04 / P2 — AC-09/T-08 health check: formatting fails
cargo fmt --manifest-path experimental/ac-mir-native/Cargo.toml -- --check exits 1, including mn-worker.rs, driver.rs, incremental.rs, resources.rs and test files. git diff master...HEAD --check passes. Apply formatting in work, commit, recheck; no global regression for formatting-only closure.

## Evidence / acceptance map
- Independently replayed cache_tier 7/7 (8.33s), codegen_tier 6/6 (5.05s), parity_tier 2/2 (5.46s), at frozen HEAD. Supports AC-02/03 and scoped AC-04 paths; does not prove all resource/parent/transaction requirements.
- AC-01 partial: first-case code and submitted evidence inspected, not independently replayed here.
- AC-02/03 scoped pass for the replayed semantics. Public emit_unit_coff takes MirModule with only profile/shape/import admission; complete trust-boundary audit remains pending, not certified by this result.
- AC-04 partial: cache replay passes; complete malicious restored-object matrix remains pending.
- AC-05 partial: source audit of ledger and sampler, historical resource/transaction evidence read; no independent scale replay. Sampler counts only direct children; full request-tree coverage not certified.
- AC-06 partial: pre-T11 raw/runner design inspected; end-to-end T11 time claim invalid per 01. No new performance pass issued.
- AC-07 partial: adapter inspected, no independent runner replay here; formal incremental gap per 03.
- AC-08 partial: SD proposal read, not approved for deposition. SD-06 must describe repaired actual formal path, history-hit behavior and valid timing. Existing metadata targets preserved; touched_goals=[] needs written rationale at closeout.
- AC-09 fail; AC-10 pending merge; AC-11 fail (01/02/03).

All high-risk areas above have explicit coverage limits. This receipt is a non-pass and does not certify unexamined acceptance criteria. Historical work evidence remains as submitted claims, not erased. No whole cargo t/tv/tt/tb/tu/taa, aggregate acceptance, scale run or docs_gen executed. Pending merge-owned gates after final pass: bare cargo t, applicable surface tiers plus dedicated workspace/consumer closeout on frozen revision; tf batch-only on main. Spec delta frozen separately with input hashes. No canonical Spec/ledger changes.

Next: work in the same tree; repair all findings with targeted tests and formal-path evidence; update T-08 report and T-11 evidence; commit and resubmit final review. Preserve unaffected completed tasks. T-08/T-11 reopened; current_step=7/11, status=executing.
