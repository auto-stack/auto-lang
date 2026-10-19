# PLAN-798 周转基准报告（r2.1 + T-07 正式 AutoDown 硬门）

2026-10-16（r2.1 修复轮：闭合 F798-R1-03/04/05 与 F798-R2-01/02）。负载：`load256::generate()`
冻结件（100 caller + 14 leaf + main，≈267 KiB，独立闭式期望）；编辑场景每样本
请求**当轮新修订**（fn_037 首乘子 +1..+30，各有独立期望）。原始样本
`turnaround-raw.jsonl`（180 行，逐样本 wall/输入 hash/计数/失败态）；汇总
`turnaround-summary.json` 由 worker 生成（真实结构，含 gate 断言结果）。

## 硬门（§5.5 r2：由正式入口强制执行）

**PASS（exit 0）**。单函数 body 编辑：请求→当前正确 AV 结果
P95 = **422.1 ms < 1000 ms**（30/30，每样本独立输入 hash 与独立期望）；签名冲突
编辑：请求→目标诊断 P95 = **20.9 ms < 1000 ms**（30/30，全部命中
`frontend:source.argument`，不接受任意诊断）。**反控**：`--gate-ms 0` 故障注入
实测 exit 5（失败原因逐条列于汇总）——门真实接线。

| 场景 | P50 (ms) | P95 (ms) | P99 (ms) | 失败 | 场景条件（r2 强制） |
|---|---|---|---|---|---|
| fresh | 3752.6 | 4424.5 | 4523.0 | 0 | 每样本独立冷 store |
| same-process-hot | 435.2 | 582.2 | 587.4 | 0 | 同进程同 store |
| restart-disk-hot | 515.3 | 743.6 | 791.9 | 0 | 每样本新进程（跨工程交替） |
| **single-body-edit** | — | **422.1** | — | **0** | 每样本当轮新修订 |
| **signature-conflict** | — | **20.9** | — | **0** | 仅目标 frontend 诊断计 ok |
| gc-concurrent | 431.2 | 814.8 | 816.8 | 0 | GC 进程与请求真并发 |

## 内存峰值证据与归因（F798-R2-02 闭合）

- **峰值采样**（非端点差值）：worker 全程 5ms 轮询 PrivateUsage 峰值 =
  **215.7 MB**；request 子进程各自采样上报，折入最大 **211.6 MB**。
- **归因测量**：隔离运行已着陆 scalar 前端（compile_scalar_source，冻结负载）
  单相峰值 = **211.8 MB**——占 worker 峰值 ~98%。探针自身各层（MIR/ABC/AV/
  cache/装配）合计个位数 MB；分区实测：staging 峰值 2.4 MB（≤16 MiB 硬门）、
  hot 闭包 1.1 MB（≤8 MiB）、AV 活槽 0（≤8 MiB）。
- **§3 128 MiB worker 目标：未达（target_met=false，如实上报于汇总）**。根因
  为已着陆前端在 267 KiB 冻结负载上的瞬态峰值（~180-211 MB），ac-core 是本
  计划**只读面**（并行合同），不能在本计划内修改——已记入
  KNOWN-DEBT-AND-RISKS 并路由前端 owner。硬内存守卫 = 受管 32 MiB 分区
  （16/8/8，逐样本入 validate_and_gate）+ P95 门；目标 miss 不静默、不误归于
  AV 管线。
- **反控**：`av_resources_peak_sampler_distinguishes_transient_spike`——~96 MiB
  瞬态分配被采样器捕获（峰值），而端点差值保持低位：证明峰值证据能区分
  "端点低而过程峰值高"。

## 命中/重算事实（r2 计数分离）

- 编辑场景每样本：`mir_lower_modules=1`（整模块 lowering 恰一次，如实）、
  `mir_units_persisted=1`、`abc_units_emitted=1`（仅被改单元链重键）、其余 114
  单元命中复用；`closure_encodes=1`（闭包 codec 如实计入请求墙）。
- 命中请求：lowering/持久化/发射全部为 0，`av_execute=1`（仍执行并重新比较）。
- fresh（真冷）P95 4.4s 如实另报：冷 store 下 115 单元全 miss + 首次前端；
  不在硬门定义内（§5.5 硬门仅 body 编辑/签名冲突两路径）。

## 边界与如实计量

- gc-concurrent P95 814.8ms：真并发 GC（独立 `gc-loop` 进程，20ms 间隔）与请求
  重叠的开销，不在硬门内，如实报告。
- 本 tier 为 Rust 定向 runner；正式 AutoDown adapter 端到端（T-07 后按同负载
  重测）仍 pending，不以本报告冒充。
- debug 档冒烟（3 样本/场景，INF 门只验场景/样本/诊断校验）：
  `av_turnaround_six_scenarios_smoke`。

## 复验

```
powershell -File scripts/verify-ac-av-798.ps1 -Scope turnaround -Samples 30
# 反控（必须失败，exit 5）：
experimental/ac-av-probe/target/release/av_probe_worker.exe bench-turnaround --samples 1 --gate-ms 0 --out -
```

## 正式 AutoDown 端到端硬门（T-07，D-788/D-796 合入后）

recipe `ac-av/core-run-v1` 经统一 probe/run 接缝（796 T-06 接缝 × 788 TestCase
v2 runner）正式执行；worker `av_probe_worker case-serve`（envelope `avworker/1`，
Job 收容，父侧判定）。**warm-session 编辑会话**步骤模型：`[run(热身), edit-source,
run]` 共享一个会话 store——终步即"单函数 body 编辑→当前正确 AV 结果"请求。

- 30 样本（冻结负载，逐样本新修订 fn_037 乘子）：**编辑窗 P50/P95/P99 =
  556/569/574 ms < 1000 ms**（窗口 = 终步驱动墙 ~412ms + spawn/IPC/admit 开销
  ~148ms；整个会话另含热身 run ~2.3s，如实分列）。
- 签名冲突路径（正式诊断负例，30 样本）：**P95 = 221 ms < 1000 ms**。
- 负控：错误 exit 期望必红；decode 准入负例（未知 recipe/跨域步骤词表/跨域
  绑定/缺绑定）全拒；worker 从不读 expected。
- 复验：`powershell -File scripts/verify-ac-av-798.ps1 -Scope autodown`
  （含正式硬门断言，退出码透传）。
