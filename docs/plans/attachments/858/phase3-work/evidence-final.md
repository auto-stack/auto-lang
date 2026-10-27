# PLAN-858 r4 Phase 3 — work handoff

- Code baseline `1690d1313f779f12f457f08a16acda0209a26ae8`, main `e263a71dd735b86233e75e38d524b6492eb81bc9`, worktree `D:/autostack/.wt/lang-858/auto-lang`, branch `plan-858-dev`; auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`, auto-ui `4903bef64532750d1597627b1197f3d2cac1effc`, both clean. Worktree clean before adding this evidence.
- Spec draft `spec-delta-draft.md` SHA256 `b77d1dd28af9eae239c38513d3dff1c8080f376f371655a46a042cec0571523f`; refreshed input hashes after 841. Phase 2 independent review was performed on its prior baseline; 841 changed shared dependencies and test configuration, so its affected book and three test groups were independently rerun during this work phase but r4 final reviewer must evaluate reuse and inspect code.
- Rebase from pre-841 `df2860c98`: all implementation and evidence patches `=` in range-diff; the single prior offline ledger-projection commit changed intentionally because main ledger v101 won the conflict. The v100 offline projection was never landed. Recompute current reviewed ledger at merge; do not overwrite 841 entries.

## Targeted verification

- `cargo check -p auto-lang` exit 0. Main P90 nearest-rank boundary test exit 0; book `--no-run` exit 0; 821/822/827 original tests 3+3+8 passed, zero ignored. Development work did not run whole-tier `cargo t`; the single-name `cargo t p90_nearest_rank_boundaries` was scoped.
- Independent AV, CTE, sema and ABI percentile controls pass; 788 valid 0ms quantization, 10-sample 9/1 and 8/2, 30-sample 27/3 and 26/4 boundaries pass. AV gate tests reject forged P90, wrong result and structure; real IPC controls reject missing samples and zero gate; resource two-line controls pass. CTE real four-outlier negative passes by producing the required red gate. Semantics and AV verification-script selftests pass.
- MIR-native: 30 samples each, P90 body edit 254.56ms, signature rebuild 724.77ms, signature conflict 112.65ms, parent-observed series 284.73ms; all pass and all had zero >2000ms. Original tracked report files restored byte-identically after each test.
- AV: 30 parent windows per path, P90 body 668.9ms / signature 256.1ms; 30-revision persistent formal P90 53.30ms, correctness, structure and memory pass; 822 formal baseline and 827 source 30 revisions P90 55.81ms pass. Independent six-scenario CLI 30 per scenario body P90 426.87ms / conflict 18.68ms, memory lines pass.
- CTE: real worker 30 per scenario body P90 170.29ms / meta 410.21ms / resident 123.62ms, all values and target pass; complete raw file retained. Semantics 1MiB producer and real parent IPC 30 samples pass, parent P90 259ms.
- ABI: actual worker body P90 661.9ms / reject 174.4ms, 30 each; parent C-consumer body P90 537ms / reject 65ms, 31/30 samples; six-scenario body P90 177ms / reject <0.1ms, 30 each; all pass and raw records retained.
- 788: 30-sample native hot P90 526ms / body edit 578ms, six-scenario correct; 30-sample syntax P90 6ms and 50-request concurrent revision stream with latest revision published, both pass. Raw reports retained.

Machine conditions were not asserted as isolated; numbers are observed on this run and are not long-term guarantees. Initial failures and pre-841 evidence remain in `evidence-initial.md`. These are targeted work checks, not the pending post-review `cargo t` and main-only complete batch.

## Remaining

Independent final review must audit full ACE surface, P90 boundary and unchanged gates, the draft Spec delta, r4 zero-precision correction, review baseline and all original IDs. Merge then recomputes ledger from current v101, runs bare `cargo t` and any triggered surface tiers on the final frozen revision, ff lands, runs complete main-only `cargo tf --batch`, preserves Phase 1 archive history, archives and guards/cleans. Do not claim any of these pending effects. Six main design documents carry unrelated WIP; 823/824 overlap files; reconcile current state before touching them.

## Original evidence hashes

Directory `C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/`:

| File | SHA256 |
|---|---|
| `p90-post841-cargo-check.log` | `6b4ebbdb8670c7b962ac518251c6536919bc123174ff7c721ab1db8bafc31aaa` |
| `p90-post841-core-boundaries.log` | `695eacfe45d62c267ec14f09133fe35e14d2c85c2f7e0f5e298550fcc66081e6` |
| `p90-post841-book-norun.log` | `e04181820f678ee718812b498b11eca7215c2698658003abd9d5babcb5a20f5c` |
| `p90-post841-av_owned_821_tests.log` | `3ee4ef6a80bbdb8edd79f3927ea7c684abeeeb680d3eb58d6ba7f728e1819ee3` |
| `p90-post841-av_collections_822_tests.log` | `f60cda08108088d6a724419f55ec6ccf2cd568ba74e9e49a423ec8f8756ba22c` |
| `p90-post841-av_source_827_tests.log` | `a3b983cc0cccc12c35d06fac6ae6bdb5b33dc0262ebb5f8eccbfeb228026017d` |
| `p90-post841-av-boundaries.log` | `02671dffb780651d0a634489249f3d0d5959ca416bc196ae1592128cb2262507` |
| `p90-post841-av-gate-controls.log` | `ea68983706a165bf41986470cc658caaff60f9659096ec3b91ecb17080ff68e0` |
| `p90-post841-av-memory.log` | `636e2d8540111b7be8493d3487e5b86c13dce1bf7c8c1928058a031385a84124` |
| `p90-post841-cte-boundaries.log` | `9fcd3769ede4c491433f0064575536fdc7a5cc798d5b624c7206fe5cbac11b0c` |
| `p90-post841-sema-boundaries.log` | `844beff421cd2235e2602801d519ca71abb046dc5d9273fc864c83f0be305580` |
| `p90-post841-abi-boundaries.log` | `b5fe68e185b572796717f0525c49e0746aae60ab05e1e16eb7fad1197f0986a3` |
| `p90-post841-mn-signature-rebuild.log` | `047847963671d4988687757e01d7443760b29102b8b8345d47a1fd9af116ff13` |
| `p90-post841-mn-signature-rebuild-after.json` | `5dddf1a8d526e407fd83e219f66f2a1ba2c2624ea0f01fee107486d9d46abf8c` |
| `p90-post841-mn-body-edit.log` | `e75c468ee86f338d991a8de97842c0b4aaee753fec59da298c53864a14d51275` |
| `p90-post841-mn-body-edit-after.json` | `7221825a1dd694eb3b5344503bffedd08b4609a9972aec6e2a32a3e70d40e970` |
| `p90-post841-mn-signature-conflict.log` | `43e94ab0674c94256a231eed56fea03f0fc106a5252a3514215f41071d650bc9` |
| `p90-post841-mn-signature-conflict-after.json` | `ad0c94c23c7ffae91ea3b2fe25b0ea6f7b9cf3233c2888351fec6ae6457134e2` |
| `p90-post841-mn-incremental-series.log` | `acc0b819b2396e6847161152e1496e6418295e6d4896bba25aa4afe25cdf638c` |
| `p90-post841-mn-incremental-series-after.json` | `643adf961d6009d6cc4c2cdb7bad273c9f52daa9a1c5d4a49219b677605aeee2` |
| `p90-post841-av-parent-live.log` | `3d57a0e06e0b6bccea5e0df7b0d7be2a5e279adc8d17546b80af74b46b57962b` |
| `p90-post841-av-judge.log` | `cb1db2cc9ef606e8139d66280abf809731c5ebc780767f2c0ade146da6ca9fd2` |
| `p90-post841-av-formal.log` | `62b59fb10c17181e044977d6cd222dfee0766086c84759b3ecb5c1f9cb62d1d0` |
| `p90-post841-av-formal-evidence/formal-report.json` | `d9c9f5ade4b849f103edcf8760601796ad0a3e656bc09ba12fc0492503bc3e16` |
| `p90-post841-av-formal-evidence/formal-samples.jsonl` | `116634af8ab6667cb747003e76060336964817f88d5dada331a8e41f6a771862` |
| `p90-post841-av822.log` | `d41fd46c6e50aa6810666ffc92577daff1cb6fb1e2a16f700c34ce54297476da` |
| `p90-post841-av827.log` | `6a26408af3fc84ac1894c49ee794e2ddf95235b864083ab3be34019724d85c87` |
| `p90-post841-av827-raw.json` | `096310c156ba3dc5f639aa241dd97d286337e6a368f7ceda7a5d550e26ff0dfa` |
| `p90-post841-cte-negative.log` | `eeac62b9b92ce8fb043f3e60b5d70d8203526b62fa9e72c83dddd7957c429dd9` |
| `p90-post841-cte-live.log` | `582f9e309950cec4826af94ae1545bac52d1ec9c4e13e3497ac385503f169688` |
| `p90-post841-cte-raw.jsonl` | `8c2ba2f8117fcf7f735b3fa3b15451eee7f1659b0277d304760db8fb5bf66e7f` |
| `p90-post841-av-bench.log` | `3dafe218f6ba88eff59d681a88ced4786c11530abd7527e4c955ebb3b4f0c0a9` |
| `p90-post841-av-bench-raw.jsonl` | `0462b4562660f8dc7db078deed2963496ef940d8fb55924d066397d55ab6f78a` |
| `p90-post841-sema-formal.log` | `1f4c502f1edc311c0d2f57166075929f2a80fcd63ce53e07dddd72414e42857f` |
| `p90-post841-sema-ipc.log` | `cb452bb9f64e44d3f1611a9d421fcb0fd8370ede9a3d315f2b12ee6ae435d2c2` |
| `p90-post841-sema-ipc-after.json` | `615c04dcf6bda49a4f2d04f72eb010fdf4113c6f9f94d1ed4d4ca2a75b4beee3` |
| `p90-post841-abi-formal.log` | `cd086fec09838d6022bed37f4e2551053a8d4a7c9f976205282312d5d39fdc9f` |
| `p90-post841-abi-six.log` | `ae15232e6498419a874f6716c0c42ca41920e00bc45e192236ecda3094476c3f` |
| `p90-post841-abi-parent.log` | `0e69b5730802cb429a771e5d05bb5b429b057a38e8ded8e6d4d95cdbb4a39f26` |
| `p90-post841-abi-parent-raw.jsonl` | `7f51c7843b487e00621f0cd9183e4f29260f37cf24a1c13afcf6ae3282243c7b` |
| `p90-post841-788-boundary.log` | `f4098cba74f44ccd0e98495d810464a83be109963c4b5e005273b35cc3eb90b1` |
| `p90-post841-788-native.log` | `b6987285061044dc86794fc64db53fa41dc90c189493c5ebdccac72222528604` |
| `p90-post841-788-native-raw.jsonl` | `a50f323c2d9cd16d2e02605d12c62da764f2994c07ea94d5fa7f83a621b36582` |
| `p90-post841-788-syntax.log` | `0a497ea197fcda3873d69e7bd5d2f761386102378eb0b9813d6e5bb89db874d9` |
| `p90-post841-788-syntax-raw.jsonl` | `fb748acba9ed9870ee05d3d53efeffe48950ec20764f177104dea41059d8ffa3` |
| `p90-post841-sema-script-selftest.log` | `59d6d5e7e4b0ef50c17950ceb087796ddd61be9497bbd53ef6ffcaee2c2d9971` |
| `p90-post841-av-script-selftest.log` | `2cce7d47f4895af51b7bd9280a50bc93154cf0f86623eea0f06f2f24d51e0860` |
