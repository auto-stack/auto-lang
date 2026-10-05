# PLAN-741 Phase 5(r5)链接清理修复验证报告(2026-10-05)

r5 基线:worktree `D:/autostack/.wt/lang-741/auto-lang` 分支 `plan-741-dev`,
基线提交 `0501b0f27`(v0.6-dev 含复审方 r5 再激活提交)。源码指纹见
[source-hashes.txt](source-hashes.txt);verify.rs/native.rs 相对 r4 零改动。
组内只读依赖 `D:/autostack/.wt/lang-741/auto-down`(detached @ fba6563e,同 r1..r4)。
T-31 前置:trace 支持库 `test-support/target/debug/ac_trace_support.lib` 已先构建
(r5 复审初跑 trace 失败的前置原因)。

## 修复摘要与 AC 对账

| 项 | 修复 | 验证 |
|---|---|---|
| R4-QA-01(P2) | `link_object_staged` 两个错误出口(链接器非零退出、执行器错误 spawn/deadline/containment/wait)的暂存 exe 回收改为 **discard_own_staged + with_staged_cleanup**(与 publish 同一 helper,已去重):收集 remove 真实 OS 错误,受阻时返回 `link.cleanup` 携带原失败(link.failed/link.spawn/deadline 等)+ 残留自有完整路径 + NOT committed + 恢复办法;NotFound=已回收、目录=用户占位不删不计;publish 的本地实现同步去重到共享 helper | lib 新增三测试:nonzero+真实共享锁(禁 DELETE)→ link.cleanup 含原 link.failed/路径/os error 32/NOT committed;spawn 错误+锁 → 同契约;无锁正常失败 → 原码 link.failed、零自有残留。r5 复审 reproduce.ps1 实测(nonzero 与 60s timeout 两出口):path_in_diagnostic=true、cleanup_in_diagnostic=true、actual_remove_os_error=Some(32)、old 三件套不变、old exe exit 5、released_cleanup_ok=true |
| R4-QA-02(P3) | 归档后断链修复:计划文件 7 处 `](../reports/` → `](../../reports/`(自 archive 父目录可解析);新增 **final_assertions.py**(`docs/reports/741-phase5-link-cleanup/`,围栏感知 + 生命周期三态):从计划真实父目录解析围栏外 markdown 链接(active 态按模拟 archive 父目录判定=终态契约);35 个唯一任务 ID、同 ID 勾选一致;metadata/README/两模块导航/全部 P741-* ledger 指针断言(激活期 archive 指向为已知暂态,标注由 merge 刷新,不谎称同步) | active 模式实测全过:location/links(7 相对链接自模拟 archive 解析)/tasks 35/35 duplicates consistent/README+auto-hir+auto-ac 指针/ledger 6 项(4 暂态标注);归档收口由 merge 以 archive 模式复跑 |
| 计数 | T-08 补勾与 22/35 口径由激活提交就位;T-21..24/T-27..30 重开由 T-32..T-35 关闭(勾选注记),最终 35/35 由 final_assertions 在归档态断言 | T-30 前勾选与关闭记录见计划 §8 |

## 门禁证据(全部 worktree 实跑,2026-10-05)

- 原型全族:**54 项绿 + 1 ignored 60s 证据项**(text_binding 11 + hir_verify 9 +
  native_execution 11 + cli 11 + trace_execution 1 + lib 14;r4 基线 54 → lib +3 为
  r5 正负矩阵,active 计数不变);fmt 干净;all-targets 零 warning。
- 一键 verify-ac-741.ps1:原型段 13/13 PASS;`cargo check -p auto-lang` PASS;
  `cargo tv` 162/162;`cargo t` 归因见下。
- 生产面零 diff:`git diff 0501b0f27 -- Cargo.toml Cargo.lock crates/ test/` 为空。
- r5 复现(20261005 目录,-ProductionDeadline,独立 scratch):
  [r4-review-reproduce-rerun.txt](r4-review-reproduce-rerun.txt)（探针文本输出,.txt 命名——P741P5-R3）
  —— nonzero 与 60s timeout 两出口均 `link.cleanup` 全要素(原失败/残留路径/
  os error 32/NOT committed/恢复办法),旧三件套字节不变、old exe exit 5、
  released_cleanup_ok=true;public-deadline 60.08s 且后代 death-watch 317ms 内回收。
- 公开 60s 证据测试(`--ignored`)60.01s 显式跑过。

## cargo t 同基线归因(THR-D3/THR-D4,零新增)

双侧同基线全量 `cargo t --no-fail-fast`(并行跑):

- 基线(主检出,同一 crates 提交):**32 红**([reds-main-baseline.txt](reds-main-baseline.txt))
- worktree(r5 改动):**32 红**([reds-worktree.txt](reds-worktree.txt))
- 29 例共有;两侧各 3 例独有,且 **worktree 独有的 3 例(state_file lock_serializes、
  plan437 gallery_chart、plan707 park_drive)在主检出 scoped 单跑全部 PASS**
  ——THR-D4 并行负载 flake 实证,非 r5 归因;基线独有 3 例同模式。
- 归因结论:crates/** 零 diff + ac-core 独立 workspace ⇒ r5 零新增;
  既有红按 THR-D3/D4 族保留完整清单,不伪写绿。

## 遗留

- 60s 证据测试仍为 `#[ignore]`(默认门禁不跑),本轮显式跑 60.01s;
  r5 reproduce 的 timeout 出口(60.015s)与 public-deadline(60.083s)为交叉证据。
- final_assertions.py 的 archive 模式由 merge 在真实归档后复跑收口(AC-26)。
