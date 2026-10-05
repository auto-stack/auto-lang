# PLAN-741 Phase 4(r4)资源边界修复验证报告(2026-10-05)

r4 基线:worktree `D:/autostack/.wt/lang-741/auto-lang` 分支 `plan-741-dev`,
基线提交 `c865adf66`(v0.6-dev 含复审方 r4 再激活提交)。源码指纹见
[source-hashes.txt](source-hashes.txt);verify.rs/native.rs 相对 r3 零改动。
组内只读依赖 `D:/autostack/.wt/lang-741/auto-down`(detached @ fba6563e,同 r1/r3)。

## 修复摘要与 AC 对账

| 项 | 修复 | 验证 |
|---|---|---|
| R3-QA-01(P2) | **受控启动消除约束前窗口**:子进程以 CREATE_SUSPENDED 起跑,入 Job(KILL_ON_JOB_CLOSE)后才经 ToolHelp32 快照恢复主线程——首个指令执行前必被管控,后代无法逃逸;**约束失败=受控拒绝**(ContainmentUnavailable → `link.containment` 诊断):安全收口(挂起态 kill+wait,零泄漏),彻底删除 r3 的静默降级/detach 分支——reap 无条件 join reader;工厂函数注入使失败分支可确定性测试(无产品故障注入开关) | lib 新增 containment_failure_is_controlled_rejection(挂起子进程被杀、ping 未曾启动、tasklist 证实零泄漏、3s 内返回);collect-deadline 测试三连跑确定性通过(r3 时序敏感);复审 reproduce.ps1 外层 UI 限制 Job 下 `public-60s`:60s 准时拒绝且**持管道孙进程 alive=false**(r3 反例存活);`--ignored` 60s 证据测试 60.01s |
| R3-QA-02(P2) | **清理错误全量汇总**:discard_staged 逐文件收集真实 OS 错误(NotFound=已回收;目录=用户占位,不删不计);新增 `link.cleanup` 诊断携带原失败+事务提交状态(NOT committed / COMMITTED)+每个自有残留路径;发布成功段备份回收受阻不再伪称普通成功;main.rs 暂存清理(staged obj)错误上屏 | lib 新增 cleanup_failure_reports_residual_staged_exe(真实 PE 基线+暂存 exe 禁 DELETE 共享锁+收据目录占位 → link.cleanup 含原 link.receipt、残留 exe 路径、os error 32、NOT committed;旧三件套字节不变;释放锁后可清理);复审 helper 实测 `staged_exe_left=true;staged_exe_path=...os error 32;old_exe/obj/receipt_unchanged=true;user_directory_preserved=true` |
| R3-QA-03(P3) | 簿记:T-08 已据 r1 收据补勾(复审方激活提交);current_step 19/30 口径=已完成 ID 数;README/plans.md 导航回写 active(复审方);ledger 暂态由 merge 刷新 | 本轮核对一致性;最终归档断言留 merge |

## 门禁证据(全部 worktree 实跑,2026-10-05)

- 原型全族:**54 项绿 + 1 ignored 证据项**(text_binding 11 + hir_verify 9 +
  native_execution 11 + cli 11 + trace_execution 1 + lib 11;r3 基线 52 → 新增 2 项
  为 r4 反例);fmt 干净;all-targets 零 warning。
- 一键 verify-ac-741.ps1:原型段 13/13 PASS;`cargo check -p auto-lang` PASS
  (382 既有 lib warning 为基线,非本轮);`cargo tv` 162/162;`cargo t` 归因见下。
- 生产面零 diff:`git diff c865adf66 -- Cargo.toml Cargo.lock crates/ test/` 为空。
- 复现脚本:
  - r4(20261005 目录,-ProductionDeadline 全量):
    [r3-review-reproduce-rerun.json](r3-review-reproduce-rerun.json)
    —— `locked-cleanup`:link.cleanup 携带原 link.receipt + 残留 exe 路径 +
    os error 32 + NOT committed,旧三件套不变、用户目录幸存(R3-QA-02 闭环);
    `public-60s`:真实 run_exe 60.08s 准时 link.deadline 且**持管道孙进程
    alive=false**(R3-QA-01 闭环;r3 反例该后代存活);`process-0..2` 为复审
    冻结的 r3 执行器**副本**(有意演示 r3 缺陷,非当前库;其输出不因本轮改变)。
  - r3(20261005 无 ProductionDeadline):块图两例 exit 1、receipt-prepare
    `staged_exe_left=false`(r3 常规路径无回归)
    [prior-20261005-reproduce.json](prior-20261005-reproduce.json)。
  - r2(20261004):六反例保持翻转、发布失败保旧制品、旧 exe exit 5
    [prior-20261004-reproduce.json](prior-20261004-reproduce.json)。

## cargo t 同基线归因(THR-D3/THR-D4,worktree 红集为基线真子集)

双侧同基线全量 `cargo t --no-fail-fast`(并行跑):

- 基线(主检出,同一 crates 提交):**36 红**
  ([reds-main-baseline.txt](reds-main-baseline.txt))
- worktree(r4 改动):**34 红**([reds-worktree.txt](reds-worktree.txt))
- 34 例完全共有;worktree **零独有红**;基线多出的 2 例
  (plan437 gallery_chart、plan716 diff_window_census)为 THR-D4 负载 flake 族。
- 归因结论:crates/** 零 diff + ac-core 独立 workspace ⇒ **r4 零新增归因**,
  且不伪写绿——基线既有红按 r1 裁定的 THR-D3/D4 族保留完整清单。

## 遗留

- 60s 证据测试仍为 `#[ignore]`(默认门禁不跑),本轮显式跑 60.01s;
  复审 reproduce 的 -ProductionDeadline 路径(60.08s,孙进程回收)为第二证据。
- process-0..2 的 r3 执行器副本属复审冻结资料,随当前库演进不再更新;
  其 r3 行为演示与本轮无关。
