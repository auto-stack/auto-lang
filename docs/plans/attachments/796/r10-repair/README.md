# PLAN-796 R10 修复轮证据（work，2026-10-19）

修复 commit：worktree `D:/autostack/.wt/lang-796/auto-lang` 分支 `plan-796-dev`
`074464f51`（R10 复审基线 `a09456532` → 本修复提交）。冻结决策 D20 见
[contract-decision.md](../contract-decision.md)（在 worktree 提交内）。

## F-796-R10-01 完整窗口计时

- `sema.rs::run_check` session 墙钟改为单一完整窗口：请求准备
  （steps/nonce/1MiB base64/JSON/hash）之前起表，父侧准入完成（schema/
  nonce/hash/recipe/worker+core 版本/restart 声明/source 镜像/query 收集，
  当前结果可用）后停表；`call_worker` 返回只是窗口内分段边界。
- 连续分段计数器 `session_{prep,worker,admit}_us_N` + `session_wall_us_N`（µs）
  与 `session_wall_ms_N` 并存；分段铺满窗口（见 window-tiling-stats.txt：
  wall−sum 差 1–78µs，≤100µs 断言内；prep 54.7–183.6ms、admit 2.1–5.0ms
  均为正且入窗——正是旧窄窗口漏掉的量）。
- 反控：`probe_adapter_edit_session_window_tiles_prep_worker_admit`（单真实
  样本，早/迟阶段都在窗内+分段无缝铺满+ms/µs 一致）+ 硬门测试逐样本同断言；
  V-08c Required 按名绑定两项。
- 新口径硬门结果（30 原始样本，1MiB，阈值 1000ms 不变）：
  edit-session（完整窗口）P50/P95/P99 = 362/430/449ms，PASS；
  冷 warm-up 单列 total_p95=1048.09ms。报告
  `docs/reports/plan796/turnaround-ipc.json`（样本含四项分段字段）。

## F-796-R10-02 V-07 raw 数值准入

- `Get-AdmittedTimes`：逐样本字段存在/非 null/数值/有限/正值，违规记
  rejection、该行 FAIL，绝不默认 0ms；lib `total_ns` 与 IPC
  `edit_session_ms` 双源同准。
- `Get-IpcReportOutcome` 纯函数与 acceptance 共用；六类变异控制 +
  合法正控入 `-Scope selftest`（selftest.log.txt：12/12 PASS——全缺失/全
  null/全 0/全 -1 均 bad=30 拒绝，单缺失 bad=1 拒绝，1001ms 慢控 p95=1001
  FAIL，合法 p95=274 PASS）。

## 统一验收（acceptance.log.txt）

13 行全 PASS、exit 0：V-01/02a/02b/03/03b/06/04/04b/05/09/08/08c/07。
V-06 12 例 281.69s；V-08 16 例；V-08c 2 例（含新反控）；V-07 重算
lib P95 8.07ms + IPC 完整窗口 P95 430.00ms。共 113 个不同定向 Rust 测试
（较 R10 +1：新窗口反控）。

未跑整档 t/tv/tt/tb/tu/taa/tf（merge-owned，2026-10-14 裁定）。
