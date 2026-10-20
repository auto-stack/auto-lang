# 批量回归续办收尾（2026-10-20，/auto-plan:regress，续 815 批）

续 [batch-status.md](../batch-status.md) 记录的未完项：tt 3 失败/1 超时归因、tb 补完、新合并窗口（815-A `e609caf1a`、797 `1799f86ca`、813/816/817 等）覆盖。到期依据=815 落地（%5 规则）。主检出单实例串行；预检无并发 cargo/nextest/cl。taa 窗口 `2772c8012..5594158a4` 零 aavm 触发。

## 执行与结果

| 腿 | 命令 | 结果 | 证据 |
|---|---|---|---|
| tt（首 fail-fast 轮） | `cargo tt` | 2237/5685 即止（1 红=plan484_024 charts，已知 flake 族；solo 3× 全绿） | [tt.log.txt](tt.log.txt) |
| tt（完整轮） | `cargo tt --no-fail-fast` | **5685/5685 全绿**（156.0s；815 批遗留 plan498_bar_group_emphasis/probe_adapter×2/plan776_r7 全部通过——归因闭合：并发会话负载 flake） | [tt-full.log.txt](tt-full.log.txt) |
| tb | `cargo tb` | **5296/5296 全绿**（110.6s；815 批中断腿补完） | [tb.log.txt](tb.log.txt) |
| tf（完整轮） | `cargo tf --no-fail-fast` | **6267/6270**（111.3s）：3 红=state_file 并发×2（solo 3 轮绿、`state_file.rs` 窗口零提交、PLAN-044 远古无归属→新登记观察名单）+ ffi_dual_019（REG-2/THR-D4 在案，同签名复现）；`abi_recipe_formal_turnaround_hard_gates` 池内通过；1M churn/语料族/画廊围栏全过 | [tf.log.txt](tf.log.txt) |

## 判定

- **new_reds = 空**：无可归属到覆盖窗口内任何已合计划的回归。
- known_reds_seen 3 项（ffi_dual_019 / plan484 charts / state_file 对——最后一项本批新登记入 KNOWN-DEBT 2026-10-20 批量登记节）。
- 收据 `docs/plans/.last-batch-regression.json` 推进 `last_covered_plan_id` 812→**817**（covered_commit `5594158a4`；批量期间 PLAN-814 会话落 3 个 docs-only 簿记提交，crates 零漂移）。
- 815 批未完项就此闭合：本文件目录四份日志为完整轮原始输出；[batch-status.md](../batch-status.md) 状态行已注记"已由本批续办闭合"。
