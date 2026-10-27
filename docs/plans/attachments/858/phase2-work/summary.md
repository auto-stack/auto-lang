# PLAN-858 Phase 2 — scoped work evidence

Code commit `c20c08a0cb5f45bb64f4feb3e2a1a41d65775fc8`; base `243da2c5ebb0a2f657a89a117885a91b17a696ed`; independent review pending.

auto-down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`; auto-ui `b4fe0000e56cddaf7d3cf70e348431f2258aacde`; both detached sibling worktrees clean.

Cargo executed from plan worktree; CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0, CARGO_BUILD_JOBS=4. After D: filled, independent `CARGO_TARGET_DIR` and raw logs use `C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2`; default positive raw log remains in plan-artifacts on D:. No links were created.

## Commands and results

- Default `cargo test -p auto-lang --lib --no-run`: exit 0, 4m33s. Book `cargo test -p auto-lang --lib --features test-book --no-run`: exit 0, 3m44s. `cargo check -p auto-lang`: exit 0. Existing compiler warnings remain in raw logs (default core146/UI compiler40/lang tests41; book lang tests43; check lang3).
- Book executable `--list`: no 821/822/827 IDs without test-cases. With `--features test-cases -- --list`, 2187 tests total; all 14 IDs equal the pre-fix source declarations and source files are unchanged.
- Controlled negative: remove only 821 cfg, then only 827 cfg; each book no-run exits 101 with matching module E0432/E0433; source restored byte-exact. See `cfg-negative-results.json.txt`.
- Worker built from this tree with `cargo build --locked --release --manifest-path experimental/ac-av-probe/Cargo.toml -p auto-av-probe --bin av_probe_worker`; AC_AV_CASE_WORKER points to resulting executable. SHA-256 `2f5c3df14101fff3f5902a7fe5364b3e2113992e5fc765d97dbe76dafd4cc501`.
- `cargo test -p auto-lang --lib --features test-cases <group> -- --test-threads=1 --nocapture`, once per group:

  - `av_owned_821_tests`: 3 passed, 0 failed, 0 ignored; all expected IDs present.
  - `av_collections_822_tests`: 3 passed, 0 failed, 0 ignored; all expected IDs present.
  - `av_source_827_tests`: 8 passed, 0 failed, 0 ignored; all expected IDs present.

Original test files, corpus, dependencies, lockfiles and test configurations unchanged by this fix; Phase 1 review/evidence remains historical. Unified regression `cargo t` and applicable surface tiers remain merge-owned, followed by the due main-only full batch. No whole-tier gate was run in this work stage.

## Raw log identities

- `default_no_run`: SHA-256 `12a1d4b4c5c8b02239984318b549c3c620078b2ff9ee52df7d91c764e3eeadfa`; `D:/autostack/plan-artifacts/plan-858/phase2-work/default-no-run-r2.log`
- `book_no_run`: SHA-256 `f0c4555eed8956a404b2d8b7af225da8e152b5dbac9d0e15d6a638d68df82e8a`; `C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/book-no-run.log`
- `cargo_check`: SHA-256 `de4487639f14470c75f6f96721ab6fc25138bdea212d3b168a2a6fab3310ae64`; `C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/cargo-check.log`
- `enabled_list`: SHA-256 `43fe8ba15762e24a43938d1ec93d40a76489568aed257f46385fb03f37969fb6`; `C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/enabled-list.log`
- `worker_build`: SHA-256 `38a08d38c829e7be4e73fb075529bae224df4fc956c47b4c8bf708ab784dd5fc`; `C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/worker-build.log`
- `av_owned_821_tests`: SHA-256 `cd15ec9479a8d38211bac6a746a61757fe3768a517bd96b81462f6b8dbc9b1f6`; `C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/av_owned_821_tests-run.log`
- `av_owned_821_tests_negative`: SHA-256 `2c85683a342c2fd6d588fe6bb5246f16c7e14d214c942e3f71492302a9b6152b`; `C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/av_owned_821_tests-negative.log`
- `av_collections_822_tests`: SHA-256 `375650ceab06697ff524cb98ac84e0f95fa19de8c1872ae475fb5e1dbb214911`; `C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/av_collections_822_tests-run.log`
- `book_feature_disabled_list`: SHA-256 `8ae6d93c8b2eeac46d8d80876cf994bd8f1a2ab61a70249fd5664150da7d465a`; `C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/book-list.log`
- `av_source_827_tests`: SHA-256 `9c2a4fd06933db37eeb9bb49c8153f335bb1e5dd986c8194706bc5e907c07421`; `C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/av_source_827_tests-run.log`
- `av_source_827_tests_negative`: SHA-256 `e626c7e38bc075985d9f9f7452e7acaefef38ed4e55e3d4396749870071673a6`; `C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan-858-phase2/av_source_827_tests-negative.log`
