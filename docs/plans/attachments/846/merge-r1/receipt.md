# PLAN-846:r2 merge:r1 — blocked before landing

Reviewed source: `bf22db3465203f3babd649494f4b37aa2a012027`. Actual root receiver: `44480cf6aff20f3b1e2a80df783976b866f5920f`, parent master `9a6c2dace33e65de8db68e7655118bc823eab30a`. The pure SDK fix replay is range-diff equivalent; see [mapping](sdk-root-equivalence.json.txt).

## F-846-M1-ROOT-MN-P95 — required root performance gate failed

Original unfiltered `cargo t`, after six current-source worker release preparations: 880/5368 run, 879 passed, 1 failed, 4488 not run, 1552 skipped; exit 100. `tests::ac_mn_probe_tests::mir_native_turnaround_signature_rebuild_gate` failed at `crates/auto-lang/src/tests/ac_mn_probe_tests.rs:1161`: signature-rebuild P95 **1032.8293ms >= 1000ms**, 30 samples. No thresholds, assertions, sample counts or selection changed. [Raw full gate](root-main-final-t.txt), [30 samples](root-receiver-generated-reports/docs/reports/plan818/turnaround-mn-signature-rebuild.json.txt).

The MN worker and performance gate are unchanged by PLAN-846; its worker has no auto-val dependency. This is a root receiver performance failure with attribution pending, not proof of a PLAN-846 product regression or of a pre-existing failure. Earlier cold-build timeouts and old graphics-root failures/replays remain diagnostic history; none constitutes a full gate pass. No further automatic full-gate retry is justified in this run.

Actual receiver `cargo tv` passed **163/163** in 3.841s; [raw](root-main-final-tv.txt), [exit receipt](root-main-final-tv-result.json.txt). Producer and fresh own-SDK/own-target consumer full driver each passed **178 top-level tests** (raw logs include one nested child); both native builds passed. SDK pure-module 17 passed on the actual receiver. Frozen source 345 inputs/SDK 681 files verified. AGP 23 inherited slots have explicit unchanged-input provenance in [reuse receipt](agp-reuse.json.txt). Remaining original native/owner/project/Step matrices and spatial per-side 14/owner invocations are **pending**, not passed; stopped after the required root failure. Draft audit/canonical/ledger scripts were not executed and do not authorize publication.

## Recovery state and closing oracle

The unique implementation worktree is restored to `plan-846-dev` at reviewed bf22, with approved raw source bytes restored from the frozen packet, source verification passed and Git clean; [restoration](worktree-restored.json.txt). The unlanded root SDK mapping branch `codex/plan-846-sdk-merge` at 44480 remains available. Permanent SDK tag `autographics-sdk-846-nan` is retained. Generated root reports were saved and only the worktree reports restored; unrelated main-checkout changes are preserved.

Plan remains **reviewed 10/11**. Checkpoints: preparation performed; **landed=false, ledger_refreshed=false, archived=false, cleaned=false**. No master/autographics-dev delivery ref advanced, no canonical Specs or live ledger published, no worktree removed. No validated-pass receipt exists.

Next responsibility: investigate the root signature-rebuild performance gate on the actual receiving baseline. Any code repair requires scoped work and affected review; then reconcile baseline/dependency equivalence and run full root t+tv and all remaining merge gates before landing. Do not lower the threshold, skip the test, reduce samples, or combine isolated replay successes into a full pass.

Skill rule: [auto-plan-merge](D:/autostack/auto-musk/.agents/skills/auto-plan-merge/SKILL.md), Gate and recovery baseline §3: “Only explicitly merge-owned operations/gates may remain pending; perform them before claiming delivered pass.” This required gate failure prevents landing, publication, archival and cleanup.
