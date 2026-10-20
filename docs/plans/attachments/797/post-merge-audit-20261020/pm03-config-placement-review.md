# F-797-PM-03 补充独立审查：合入期 nextest default-filter 放置修正（2026-10-20）

**结论：ACCEPT——该变更是纯"验证配置放置"决定，无断言弱化、无覆盖丢失、串行参考入口完整；缺失的是冻结提交上的执行绑定证据，由同日 closeout 重跑收据补齐（见本目录 `pm03-closeout-rerun/`）。**

审查对象：`1799f86ca` 相对 R17 受审基线 `5a54f588d` 的唯一非 docs 变更 `.config/nextest.toml`（diff 全文 = 本目录 [post-review-config-diff.txt](post-review-config-diff.txt)：8 行注释 + default-filter 行追加 `and not test(abi_recipe_formal_turnaround_hard_gates)`；同提交内 `experimental/auto-abi-probe/src/cache.rs` + `tests/t05_cache_turnaround.rs` 为 R16→R17 修复轮已审代码随 rebase 进入交付，R17 已完整复审）。

逐项核验（本轮实测命令留存于本文件，可复跑）：

| # | 审查点 | 证据 | 判定 |
|---|---|---|---|
| 1 | 排除仅作用于日常档 default-filter，单 token 追加，无其它 profile 变化 | diff 仅 1 个 filter 行 + 8 行注释；`[profile.default.slow-timeout]` 等未动 | 通过 |
| 2 | 计时门断言零改动（reviewed → HEAD） | `git diff 5a54f588d..HEAD -- crates/auto-lang/src/tests/probe_adapter_tests.rs` = **空**（本轮复跑实测） | 通过 |
| 3 | 批量回归档仍可见 | `.config/nextest-full.toml` default-filter 仅排除 `aavm2_t3_tower_milestone`，不含本门（本轮实测 grep） | 通过 |
| 4 | 串行参考执行入口完整 | `scripts/verify-autoabi-797.ps1` 父侧步 = `cargo test -p auto-lang --features test-cases --lib abi_recipe -- --test-threads=1`（libtest 串行）；probe t06 正式门在 probe 套件内 SCEN_LOCK/GATE_LOCK 串行（R4 起在案） | 通过 |
| 5 | tv 语料档不受影响 | tv profile 为语料三族正向筛选，本门原本就不属于 tv | 通过 |
| 6 | 先例一致性 | 与 churn/里程碑/画廊围栏/ui_gen 的既有排除待遇同构（nextest.toml 注释链在案）；归因实录（并行池争载、9% 空载仍红、同提交单跑 31.4s PASS）已在计划 §9 merge 收据 | 通过 |

审查者边界：本审查未实施该变更（变更属 merge 阶段放置决定）；不恢复并行计时主张；"排除后的日常全绿 ≠ 被排除硬门通过"由本目录 closeout 重跑收据直接回应。
