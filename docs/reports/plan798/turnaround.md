# PLAN-798 T-05 周转基准报告（r2，六场景 × 30 样本，release，硬门强制执行）

2026-10-16（r2 修复轮，闭合复审 F798-R1-03/04/05）。负载：`load256::generate()`
冻结件（100 caller + 14 leaf + main，≈267 KiB，独立闭式期望）；编辑场景每样本
请求**当轮新修订**（fn_037 首乘子 +1..+30，各有独立期望）。原始样本
`turnaround-raw.jsonl`（180 行，逐样本 wall/输入 hash/计数/失败态）；汇总
`turnaround-summary.json` 由 worker 生成（真实结构，含 gate 断言结果）。

## 硬门（§5.5 r2：由正式入口强制执行）

**PASS（exit 0）**。单函数 body 编辑：请求→当前正确 AV 结果
P95 = **506.5 ms < 1000 ms**（30/30，每样本独立输入 hash 与独立期望）；签名冲突
编辑：请求→目标诊断 P95 = **21.3 ms < 1000 ms**（30/30，全部命中
`frontend:source.argument`，不接受任意诊断）。**反控**：`--gate-ms 0` 故障注入
实测 exit 5（失败原因逐条列于汇总）——门真实接线。

| 场景 | P50 (ms) | P95 (ms) | P99 (ms) | 失败 | 场景条件（r2 强制） |
|---|---|---|---|---|---|
| fresh | 3752.6 | 4424.5 | 4523.0 | 0 | 每样本独立冷 store |
| same-process-hot | 435.2 | 582.2 | 587.4 | 0 | 同进程同 store |
| restart-disk-hot | 515.3 | 743.6 | 791.9 | 0 | 每样本新进程（跨工程交替） |
| **single-body-edit** | **418.1** | **506.5** | 543.0 | **0** | 每样本当轮新修订 |
| **signature-conflict** | **18.6** | **21.3** | 23.1 | **0** | 仅目标 frontend 诊断计 ok |
| gc-concurrent | 431.2 | 814.8 | 816.8 | 0 | GC 进程与请求真并发 |

worker private-bytes 增量 = **8.6 MiB ≤ 128 MiB**（bench 全程实测，计入 gate）。

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
