# PLAN-858 r4 Phase 3 — work evidence on initial baseline

Implementation and test tree `D:/autostack/.wt/lang-858/auto-lang`, branch `plan-858-dev`, code HEAD `87a4cb17f088f03242c93c6506d539b6c6429ce3`, auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`, auto-ui `b4fe0000e56cddaf7d3cf70e348431f2258aacde`. Existing Phase 2 review remains bound to its earlier code, not to this phase.

Implemented nearest-rank P90 and 2000ms inclusive threshold for AC/MIR-native, AV, CTE, semantics, ABI, 788 native/syntax and verification scripts. Formal 30+ samples, full paths, independent values/diagnostics, structural counters, raw records and memory gates remain. P95/P99 remain diagnostic. Zero duration follows path-specific rules; 788 integer-millisecond quantization can produce valid zero, while existing 796 admission still rejects zero.

Targeted observed results on the initial baseline:

- Boundary and negative controls: main, AV, CTE, semantics, ABI and 788 tests pass 10/30-sample one/two or three/four outlier boundaries, equality and relevant invalid samples; AV formal judge actual IPC pass and wrong result/missing sample/zero gate red; AV structural/correctness gate mutation controls and two-line memory controls pass; CTE 30-sample four-spike actual negative returned red and its test passed; 796/827 script selftests pass.
- `cargo check -p auto-lang` and book `--no-run` pass; original 821/822/827 families pass 3+3+8 (0 ignored).
- MIR-native 30-sample P90: body 307.78ms, signature rebuild 999.92ms, conflict 206.35ms, parent series 423.49ms; all pass. Signature rebuild had 0/30 over 2s and P95 1211.50ms; concurrent build existed and is not a claimed quiet-machine run.
- AV parent 30+30 samples: edit P90 900.7ms, diagnosed signature P90 287.3ms, 0 over 2s; AV persistent 30-revision formal P90 53.37ms, all-correct/structure and both memory lines green; 822 baseline and 827 source 30-revision P90 69.39ms pass; six-scenario CLI body P90 503.88ms, signature P90 24.52ms and both memory lines pass.
- CTE real 30-sample CLI body/meta/resident P90 295.93/761.84/224.61ms; all-values and target pass, full raw preserved. Semantics 1MiB producer and 30-sample parent IPC pass (parent P90 401ms).
- ABI worker two 30-sample gates pass (body P90 987.8ms, reject 160.6ms); parent body/reject P90 767/86ms; 30-sample six-scenario body/reject P90 276.8/0.1ms. First six-scenario attempt failed before sampling at `cl -?` prewarm deadline while other builds ran; retry after load changed passed. Causation is not proven.
- 788 native six scenarios ×30 pass, syntax 30 plus 50-revision concurrent stream pass; original raw copies kept.

No whole-tier regression was run during work. The merge-owned bare cargo t, applicable surface tiers and main-only full batch remain pending. These results do not imply final independent review or delivery.

Since measurements, master advanced to `e263a71dd735b86233e75e38d524b6492eb81bc9` through PLAN-841, changed shared language dependencies and four experimental Cargo.lock files, and updated ledger v99→v101; main auto-ui advanced b4fe0000e→4903bef64532750d1597627b1197f3d2cac1effc. Before final review, reconcile the changed main and dependencies, preserve the 841 ledger, regenerate affected evidence and Spec target hashes. Previously measured data above remains historical, not proof on the new baseline.

Six existing main design WIP files and in-flight 823/824 overlap must be preserved and reconciled before landing.

## External evidence SHA-256

Directory `C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/`:

| File | SHA256 |
|---|---|
| `p90-core-boundary-final.log` | `f564751ae2266f499c5c825e91b6822593dc576aac2e036d55d489c97430794d` |
| `p90-av-boundary-final.log` | `0445fb3d6a0a96e6939fc5901a0f70ecec8dd3e0f78c575c3b6bb85bedd81769` |
| `p90-cte-boundary-final.log` | `6e4656682bdd16ed4d885288da84cc1e3df7cb3357aa56b047eb388a35e2b607` |
| `p90-sema-test.log` | `31d2c8b5c0dc1ef6ca9d242ee7b2dc3e233db63af205e6575b410c025fe45afa` |
| `p90-abi-boundary-final.log` | `708c703310731a2708dfdc5c89decc15054dbb419435727e44ae7647a0577967` |
| `p90-788-zero-test.log` | `e5d4740804e4359132923ba364d49e462de6f7fbf5ae2b3e0d0c905479936f3c` |
| `p90-mn-policy.log` | `d5626535865da3cde1ecf7cf2d6b2c2401259269d8fff1c22ab05422af8d0a20` |
| `p90-av-gate-controls.log` | `e42ad30c6328bc82793454109bef19e57304b9b92c619b6de3b72c1101a8400b` |
| `p90-av-judge-controls.log` | `d3b0f12eb6404379b218ee8a866ae2534bf326bdbe060c3bee79c66a18049d64` |
| `p90-av-memory-controls.log` | `096e4f014682111f405deaa3450bc1095621948cb0fdd9d05487986724532e6e` |
| `p90-cte-control.log` | `e7de5bd65868d45d7a7a88bf47c31a6cc74568c6439dee19db582dfa25899bdf` |
| `p90-script-selftest-final.log` | `59d6d5e7e4b0ef50c17950ceb087796ddd61be9497bbd53ef6ffcaee2c2d9971` |
| `p90-script-selftests.log` | `c94d678918ba10122afdea847dd1733390ba5a5aa3f64b3934eb510f77fd32b4` |
| `p90-cargo-check.log` | `2e94647f5f7b352b19d7094d46d9c220018203345cc0dc8bede3b79576c94bc2` |
| `p90-book-norun.log` | `069ef507e7b0b309c5ca89703f569af40f9e8def5e3368d76c46b0d9f527e100` |
| `p90-av_owned_821_tests.log` | `50ca0d6e237ecada7ab5b8e873de991f2f17c9f97db29a97866424dccd345211` |
| `p90-av_collections_822_tests.log` | `cc42710aa2a3fea343a5b79a4069b47aa056b56fcb2cc74743c16bd7fd01026c` |
| `p90-av_source_827_tests.log` | `15d3348363a1769fa347701d1723c2911933d4874cf52f4ea5bb7d07dc9a4140` |
| `p90-mn-signature-live.log` | `342a58958cbec2ba0df0b1f5f657a28cd105458b3fd71ca2f93aa83c6121ce23` |
| `p90-mn-body-edit-live.log` | `c19fdf31f840b3c17b298c13277cb934e95acffa641d319cb8069902f59499dc` |
| `p90-mn-signature-conflict-live.log` | `7ce5d7a650d1e8b46e6093bdc36d9abb1818ac760468c390c316872541e24139` |
| `p90-mn-incremental-series-live.log` | `b8210e73cd3d7da1e9f0b764dd9e4d48792cdea1042d4ad70f9cecba696bdff2` |
| `p90-av-parent-live.log` | `e89bc91c937d1b83ef86aaf39aa31828b63e20405f3c4e78c2464ba03dc707d5` |
| `p90-av-formal-live.log` | `cbe030b9d3c81d22b51de2d4d284b6efa38cda9d5f75643080b550545f91626a` |
| `p90-av822-baseline-live.log` | `27930b371cb19339ee21aa034d496686cffa1015a2060feb7b5ca232f740ea2a` |
| `p90-av827-incremental-live.log` | `e6a32803f565c0510c689838ca96b2d37002ce594bda8f564382310b7a74fec5` |
| `p90-av-bench-live.log` | `b14102750e73acff3ba6cb9d5277667b391b0c0c262683aa3419c6e8dc09274b` |
| `p90-cte-formal-live.log` | `81075c0388ce0fc7cd8f0e08c1cc381d50324960f92383605e2d7f1b90a4dc1c` |
| `p90-cte-worker-live.log` | `40dd32d4f44849f3222e12fe2e8f4cbb4e99907e4799e19f79df5b38bdfaba65` |
| `p90-sema-formal-live.log` | `8af1bd3976cf1ff2125ac975cb1b23119efaae04a054997ce10b6c2b3002a24c` |
| `p90-sema-ipc-live.log` | `bd16f74ab6ad5caa327c5d58dcba9e23de23a4d804b1ac93171499c2bcff6835` |
| `p90-abi-formal-live.log` | `2149e1322d00840f0de805881702bd569e570b9f171095102bdd08a2e3c1f980` |
| `p90-abi-parent-live.log` | `04373bb0abcb75f08585bf9c80993c68a1136c20cdd10e0c8c10ed20cb8085ff` |
| `p90-abi-six-live.log` | `df432b743b1a13caa73676210372494f1fbd3a85505d489becbf78e58bc19ea2` |
| `p90-abi-six-live-r2.log` | `ba96f5d685e4ec1e72800a804051b1833545532eccf96af2b7a60e7bea52071e` |
| `p90-788-native-live.log` | `f742b3d47507b9ea3156f1fb626e605922f0d5f2c73905002b77b11477a2f933` |
| `p90-788-syntax-live.log` | `33686c593b315489cb6cbac209e9358e2fc4a5913960c899b5d243715a186c47` |
| `p90-av827-negative-test.log` | `5d032c8ee33d8c1294ad704bc381e589c2f8c284ccdfb2599cccae789a9f6b1c` |
| `p90-788-native-raw.jsonl` | `dd144808dd4402f77be22a4293255dd22a529d9ea0ebd2127c87163aebd50dae` |
| `p90-788-syntax-raw.jsonl` | `a46178f0de377405493b9ea5245c857ccb4347a8b0b87ab256e03e730aef78ce` |
| `p90-cte-worker-raw.jsonl` | `a2db8a228976b57338a4c0d27ea545c9c2fe1847f7b958ac39bc2036578cf17f` |
| `p90-av-bench-raw.jsonl` | `be88f92d93d3d29cc417652f97d5c739e5d997925159647b0f73e1578cc7d0eb` |
| `p90-av827-incremental-raw.json` | `d551ca3ae88b433390abd59e59c1a630957b8a7e0250ce1eff14bd70b6331bc7` |
| `p90-abi-parent-raw.jsonl` | `b155fc25fea4677d76eca05fd51be9fe27b70e2d08619e9b026adc287248d148` |
| `p90-av-formal-evidence/formal-report.json` | `b1dd8451a6cde4f354aba0d38279b73edd94d1a6fa2cc3d39c5e1a90a3172706` |
| `p90-av-formal-evidence/formal-samples.jsonl` | `877b66839f805410aaf27940a69b7291851022f0b01fcc97a7268cf333e8f985` |
