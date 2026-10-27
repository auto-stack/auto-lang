# PLAN-858 Phase 3 R1 repair handoff

Finding F-858-P3-R1-01 / AC-12 / T-13 / V-10. Baseline `447fb53c08516f6fe1c4af31bd83639bacb20561`; same worktree `D:/autostack/.wt/lang-858/auto-lang`, branch `plan-858-dev`, dependencies auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4` and auto-ui `4903bef64532750d1597627b1197f3d2cac1effc`, both clean. Contract r4 and frozen SD SHA256 `b77d1dd28af9eae239c38513d3dff1c8080f376f371655a46a042cec0571523f` unchanged. The issue was independently established by the R1 old successful logs lacking P90; actual old rows independently recomputed to 526/578/6 ms. No timing window, count, result, assertion, gate, or historical P95 calculation was changed.

## Closure and sibling check

- 788 hot, body-edit and syntax success outputs now include the measured P90 and existing P95; observational 788 fresh/restart/cross/GC siblings also include both. The two original real tests ran with `cargo test -p auto-lang --features test-cases --test plan788_r3_ <exact test name> -- --exact --nocapture`, one each, both 1 passed/0 failed/0 ignored, native 140.64s, syntax 6.01s. The same 30 raw records for each of the seven emitted scenarios are committed here. `python docs/plans/attachments/858/phase3-repair-r1/verify_samples.py` recomputes P90 and the existing path-specific P95 and compares the actual captured success text, including all seven scenarios. The 50-request syntax stream remained enabled and completed. Hot P90/P95 1137/1270 ms; body 650/663; syntax 5/5. Observe performance is machine-dependent.
- ABI parent C-leg `cargo test -p auto-lang --features test-cases --lib abi_recipe_formal_turnaround_hard_gates -- --nocapture` passed 1/1, 31 body + 30 reject actual parent requests. `abi-parent-summary.txt` now contains P90/P95 for both; retained raw rows and `verify_samples.py` recalculate both. Body 600/619ms, reject 73/80ms. ABI standalone `cargo test --manifest-path experimental/auto-abi-probe/Cargo.toml --test t05_cache_turnaround abi_turnaround_six_scenarios_30_samples -- --exact --nocapture` passed 1/1; seven reported scenarios retain 30 actual samples each and P90/P95, independently recalculated by that same replay.
- MIR parent `cargo test -p auto-lang --features test-cases --lib mir_native_incremental_series_formal -- --nocapture`, `MN818_FORMAL_SAMPLES=30`, `MN818_FORMAL_GATE_MS=2000`, passed 1/1; 30 raw windows and report retained, prior tracked report restored byte-for-byte. `verify_samples.py` matches the report and success line (P90 509.4385ms, P95 528.3502ms). Debug mirror `cargo test --manifest-path experimental/ac-mir-native/Cargo.toml --test incremental_tier mir_native_incremental_d828_baseline_30_structure_and_p95 -- --exact --nocapture` passed 1/1; real 30 revisions printed P90 636.0 / P95 647.1 ms. Its existing producer does not persist individual debug observations; output/actual test evidence are preserved, not represented as independently recomputed raw.
- AV 821 first-case evidence is an 8-sample resource observation, not an ACE latency gate. Its report now includes nearest-rank P90 and existing P95. Extracted the actual `wall_ms_p90` expression from `scripts/verify-av-owned-821.ps1`, executed it on retained `docs/reports/plan821/first-case-evidence-20261026-104651.json` (SHA256 `574b85d2aab3daefa7f2bd9ccce6c0b0318365816400e96d2c6bbd2b7db9f39f`), and independently matched 8-sample P90=26.144ms; 10-sample one/two-outlier controls passed. PowerShell parser accepted both changed scripts. Did not run 821 first-case aggregate (out of repair test scope); no claim of its full execution.
- Semantics 796 success summary now includes the P95 values it already computed. `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/verify-ac-semantics-796.ps1 -Scope selftest` passed all existing valid/invalid controls (including 27/3 vs 26/4). This is selftest, not the acceptance aggregate. AV parent already emitted P90 and P95 separately; inspected without change. 788/MIR/ABI correctness, structural and resource tests remain unchanged. The prior R1 independent n=11/19/31/39, NaN and zero boundary checks and unaffected work evidence are reusable only where input/dependencies/config unchanged.

All code checks used `CARGO_TARGET_DIR=C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/target`, `CARGO_INCREMENTAL=0`, dev/test debug=0, jobs=4. Historical compiler warnings remain in the complete logs, with no claim that they were introduced by this repair. No bare cargo t or surface tier/batch was run; those are merge-owned. There are no known open work-owned findings; independent review must close R1 and evaluate previously partial full Phase 3 and SD coverage, goal-impact rationale, in-flight changes and clean baseline. The design WIP/in-flight conflicts remain merge reconciliation, not implicitly overwritten.

## Complete local logs (full file SHA256)

- `repair-r1-788-native.log`: `e3a2182cc554e7f2967afa7f139a21663166e26dfbc00adbab23044f9dac0120`
- `repair-r1-788-syntax.log`: `7be4ac9203585511c595a913da4f2dad6b7918c591792c9452d80cc8d440348b`
- `repair-r1-abi-parent.log`: `f8e8b252dc1b8c0a5aaf0b43b8e0a76609f96243589d3092dea55825fb5a514b`
- `repair-r1-abi-six.log`: `398fb909cd347d364266794e7ead867e09ba79602bb6f68d7206c15bf542bb46`
- `repair-r1-mn-parent.log`: `330e27fabff5c2ab589215ea0c205a14ed27b1f38985425e6c2685d88f3ce43a`
- `repair-r1-mn-debug.log`: `7d9854c7f83c1a9605405fa7da71173bf6a7a15d0086e3ed44db12fa41cffeab`
- `repair-r1-sema-selftest.log`: `097e042b63d8b43fe4cc8aa917707a91a224f9354922f90c879c737b9eddabf3`

## Durable evidence file SHA256

- `788-native-log-excerpt.txt`: `f68c03e4ec3b9b06c2ca7137e88da77208db91a3addc76c1ff0eb93736d33fda`
- `788-native-raw.jsonl`: `39d96584bf25ea35619e64676e951d6f86238f5929911ea4e116c1030d29b2f3`
- `788-native-verification.txt`: `57f10e5ea0b69399b6b6b33487f7292bf233ffeedcae3540d4e6abae5e3ae9f4`
- `788-syntax-log-excerpt.txt`: `febe64e8406b047e65b0b972ee1c727b40ec9f3b34b70df7bef35e3252aa550c`
- `788-syntax-raw.jsonl`: `bad63de7f80a00b7aa6c516000b04d85bf92da1575fdb3e286511385db926c29`
- `788-syntax-verification.txt`: `5b332f6d53040eb6d824d7302708404ed9e526091c916e08450f64146b6f2de6`
- `abi-parent-log-excerpt.txt`: `b71751c6b6e35df2808726a95d6c1c8a247183fbbbdd45a1dbcc9323984b5d05`
- `abi-parent-raw.jsonl`: `4206eb062299ae95e7cdd3b7f2c08a517df67ba36484d0c2e478b349142d02fa`
- `abi-parent-summary.txt`: `528c3b2109262b33ca1ffb7f046b5590c6f3796efa3bdcea1ef5d884b8b84983`
- `abi-six-abi-layout-edit.jsonl`: `9602b06737113d3f2d45ebd75e944de020c047ca9b5283309b371252ca3dfd6f`
- `abi-six-body-edit.jsonl`: `dccc1a02742b05b818b665b6d36e07ab1875bdf98eef73f4706085ce5eab4aad`
- `abi-six-descriptor-reject.jsonl`: `5349e1cf7f9a0dbfdbb369494a7a3bf16f713735dc435f63ee78a19d1a5155c4`
- `abi-six-fresh.jsonl`: `874111d78078ae3cccb3dcc33a917dc35df259925d5e67a5853eb53a7facd1ca`
- `abi-six-layout-hot.jsonl`: `f9e5f7848478c6441caf1a1fc0ba4027680e8c332053fe1523e22a5c42a17357`
- `abi-six-object-hot.jsonl`: `10f64c80ad4c75379ed0cc407f5198227f733be7454893260405ea8e0266204f`
- `abi-six-restart-disk-hot.jsonl`: `27f465f49177965aaae5885525f74edfac0925abbd1ab7832f01dba84e8f87b9`
- `abi-six-summary.md`: `90de78d92de54ee8864bf796627b9bc9ec0d024f099389fe5f0ef52229458cbd`
- `mn-parent-log-excerpt.txt`: `dde3b1884dedadde9d64f6619cdc03f14f5934704d5607c0541d45408d16e55e`
- `mn-parent-raw.json`: `7bf52521a6ba6e6462898b39eae7ada2291fe94c913ae18fdba79a5fec6bdd5c`
- `verify_samples.py`: `282cf01852df39612f06040587c973f45a76c200ed81bbc0f6889a981c3a7d21`
