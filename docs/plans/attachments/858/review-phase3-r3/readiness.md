# PLAN-858 Phase 3 R2 repair readiness

Baseline `d177040f0a69d8b9f544a62312a6c46d2d5bee90` in existing `D:/autostack/.wt/lang-858/auto-lang`, branch `plan-858-dev`; dependencies auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`, auto-ui `4903bef64532750d1597627b1197f3d2cac1effc`, clean. No Spec contract revision. Earlier R1 output-report fix and actual raw samples remain unchanged.

## Findings → shared invariants → closing checks

- **F-858-P3-R2-01**: release `cte_turnaround_resident_degrade_gate_and_control` passed `1000` to the real bench on normal, forced-rebuild and detector-disabled paths. Set its release threshold to `2000`; debug's existing `5000` smoke limit and all sample counts and structural assertions remain. Independent R2 captured controlled 1100ms-input worker replays under the old 1000 and intended 2000 thresholds; the raw/summary files copied here preserve those historical results, not production performance claims. Added `cte_turnaround_release_gate_2000_boundary_control` using the actual worker: 10 real body-edit samples with 1100ms injected into the first 2 produce P90 in (1000,2000] and must pass with all values right; 2200ms injected into the first 2 must fail P90>2000 and retain all correct results. `cargo test --offline --release -p auto-cte-probe --test cte_turnaround_tests cte_turnaround_release_gate_2000_boundary_control -- --exact --nocapture` passed 1/1; the *original* release structure control test, same command with its name, passed 1/1 in 110.21s, including normal, forced degradation and detector-disabled assertions.
- **F-858-P3-R2-02**: four MIR formal receipt consumers (fresh, restart, signature rebuild, signature conflict) previously coerced missing/non-numeric `request_wall_ms` to zero; replace all four with one shared strict finite-positive admission. This preserves their existing positive-window rule and does not change 788's valid integer-ms zero. Body-edit and formal-series sibling paths already assert a positive worker window and parent window coverage; no fallback zero may pass those checks. Prior R2 mutated-receipt script is retained as a reproduction of the *old* access/default expression, not represented as a post-fix passing test. New permanent `mn_formal_receipt_wall_admission_controls` invokes the same helper used at all four call sites: 29 valid peers plus each missing, null, string, zero, negative, NaN/Infinity-encoded null must return an error before percentile; 30×2000 equality and positive 0.1ms pass. `cargo test -p auto-lang --features test-cases --lib mn_formal_receipt_wall_admission_controls -- --nocapture` passed 1/1. Both actual 30-sample signature rebuild and conflict tests passed 1/1 each using their existing windows and constraints; the copied reports each have 30 finite positive raw values and nearest-rank recomputations 1419.0248ms and 235.9753ms respectively; prior tracked reports were restored byte-identically. Existing `cargo test -p auto-lang --features test-cases --test plan788_r3_ ace_p90_nearest_rank_boundaries -- --exact --nocapture` passed 1/1 with its legal zero and 10/30 boundary assertions.

The CTE test commands ran in `experimental/ac-cte-probe` with jobs=4; main test commands ran with `CARGO_TARGET_DIR=C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/target`, `CARGO_INCREMENTAL=0`, dev/test debug=0, jobs=4, `MN818_FORMAL_SAMPLES=30`, `MN818_FORMAL_GATE_MS=2000`. Broader ACE gate search across changed consumers found no additional 1000ms gate; historical diagnostics and independent targets are not reclassified. Logs retain baseline compiler warnings. No whole-tier/batch gates ran. No known open work-owned findings; independent review must verify closure, actual sample admissions, both structural control states and unchanged evidence before phase pass. Merge still owns final frozen bare cargo t, applicable tiers, main-only full batch, conflicts, knowledge and guarded cleanup.

## Full local log hashes

- `repair-r2-mn-admission.log`: `c6957677c5bc24e485e77d51714af10c7d879b91c5cc49f969db9310cc6116a0`
- `repair-r2-cte-boundary.log`: `80b2bd8d58178977147f09852429667ec805c7abdcc2602f267e503f431bd3d3`
- `repair-r2-cte-structure.log`: `b07ea98b6909dde3b365ef307dba2f912f48392925d27a34d76482969c1b2e05`
- `repair-r2-mn-signature-rebuild.log`: `17ecd6515bb381ada70900b4d2d9cddc85bd592448a3abcf49d2eb36236f3d84`
- `repair-r2-mn-signature-conflict.log`: `03a4ce4fd3e15544d4470079fe924b6c678aedd0e88884203a535940fd646110`
- `repair-r2-788-zero.log`: `99e79b6f1c7307b05cb18b3227e9c5870cbbabdc410238c470eb150168db57ac`

## Retained evidence SHA256

- `mn-signature-conflict.json.txt`: `945f379fcce7461727a394d53a0b1fc3739759003b021b2ab47bf3e561e9cb4a`
- `mn-signature-rebuild.json.txt`: `1f12d4343b62b22fe7405e4ba60c63d9e3d65bf0d82dae95ee04db326fe6d410`
- `r2-review-cte-1000-raw.jsonl`: `8505105e7229a50e36a1c8507bbc80f780d7466e67656d4b3c8c207b453c11bb`
- `r2-review-cte-1000-stdout.json.txt`: `aeabf9d4bbcc844f1f0f8532ae324cafcc34d417abdb8e3b449e5b3ead2972c3`
- `r2-review-cte-2000-raw.jsonl`: `f69ac1d78725f1ae07f6d1401dabaf0eb2e6ea1090c0815bf4b3a8f7aa7004b3`
- `r2-review-cte-2000-stdout.json.txt`: `7f13cf1e0e1a8d5404561d529dc6bd6ede59df69a032a4153db1f7b9340805ce`
- `r2-review-replay-missing-receipt.py`: `56ef78549316c2b3d6419991ae817fbdfc8ae1aa17f7ec87815d7826c16c5e1b`
- `repair-r2-788-zero-excerpt.txt`: `691be974efe15393a4b63ec0e49d5b920e112967f6577947099d3e300778435b`
- `repair-r2-cte-boundary-excerpt.txt`: `9f02bfe6e76b5d647b2c707319a91135127502065fcd22d526df0c18d179c9b1`
- `repair-r2-cte-structure-excerpt.txt`: `0854484daab7aaced119f07231245eff3dbd01990a3964917943aecb010cdf31`
- `repair-r2-mn-admission-excerpt.txt`: `be4df63010fd045868aafd2f53b0cb0a093ff67effdb6477721cb7811f1ada29`
- `repair-r2-mn-signature-conflict-excerpt.txt`: `3925d03e9e08aae316bdf94011366dacb50358a074c180db5713c9cc318b540f`
- `repair-r2-mn-signature-rebuild-excerpt.txt`: `c1ecc115ea8e4fa834711bccb863afd14b6d18f9ff0e7d1cd0e189f1b6e8aea0`
