# PLAN-798 T-05 周转基准报告（六场景 × 30 样本，release）

2026-10-16，参考 Windows 机（本机）。负载：`load256::generate()` 冻结件
（100 caller + 14 leaf + main，267 KiB，独立闭式期望 2665）；编辑位 fn_037 首
乘子；签名冲突 = fn_leaf_1 增参。原始样本：`turnaround-raw.jsonl`（180 行，
逐样本 wall/命中/重算/执行计数/失败态，可复验）。

| 场景 | P50 (ms) | P95 (ms) | P99 (ms) | max | 失败 | 平均 ABC 命中 | 平均 mir_build | 平均 av_execute |
|---|---|---|---|---|---|---|---|---|
| fresh | 782.0 | 841.6 | 1295.7 | 1295.7 | 0 | 111.2 | 3.83 | 1 |
| same-process-hot | 774.9 | 831.5 | 962.0 | 962.0 | 0 | 115 | 0 | 1 |
| restart-disk-hot（含跨工程） | 781.0 | 821.8 | 886.7 | 886.7 | 0 | 115 | 0 | 1 |
| **single-body-edit** | **753.2** | **829.4** | 888.4 | 888.4 | **0** | 115.0 | 0.03 | **1** |
| **signature-conflict** | **37.9** | **41.4** | 41.5 | 41.5 | **0** | — | 0 | 0（正确诊断态） |
| gc-concurrent | 940.0 | 1159.2 | 1182.8 | 1182.8 | 0 | 115 | 0 | 1 |

## 硬门（§5.5 冻结语义）

**PASS**。warm 会话单函数 body 编辑：请求→当前正确 AV 结果
P95 = **829.4 ms < 1000 ms**（30/30 样本值与独立期望一致，失败 0）；
签名冲突编辑：请求→当前正确诊断 P95 = **41.4 ms < 1000 ms**（30/30 正确
frontend 诊断，失败 0）。

含当次必要前端（整模块重查，如实计量——267 KiB 词法/解析/resolve/typecheck/
verify 约占每请求墙钟的主要部分）/verify/依赖/cache 读取解码/MIR/ABC/AV/比较；
未扣任何 IPC/等待（本 tier 为 Rust 定向 runner；正式 AutoDown adapter 端到端
待 T-07 后按同负载重测）。

## 命中/重算事实

- 命中请求：mir_build=0、abc_emit=0、**av_execute=1**（命中仍执行并重新比较，
  不缓存 PASS/运行结果）。
- body 编辑：仅 fn_037 单元重算（首轮 1 miss，其后该变体亦命中），
  其余 114 单元复用——"未读 body 的 caller 代码可复用"成立。
- 签名冲突：frontend 诊断态（stage=frontend），非崩溃、非 trap。
- fresh 场景内 30 样本共享 store 根，故样例 2+ 为命中态；真正的全冷构建 =
  每场景首样本（P99 1295.7ms 反映首个全 miss 样本）。

## 边界与如实计量说明

- gc-concurrent P95 1159ms 超过 1000ms：该场景不在硬门定义内（硬门仅 body
  编辑与签名冲突两路径）；GC 竞争开销按 §5.5 单列报告。
- p99/max 反映冷启动与 GC 交叠样本；未删除任何失败样本（本报告失败数为 0）。
- native 冷构建/全冷/广泛失效按 §5.5 另报（native 参考单次构建见
  `native_trace_parity`/vertical 测试；非本硬门组成）。
- debug 档冒烟（3 样本/场景）在 `av_turnaround_six_scenarios_smoke`。

## 复验

```
powershell -File scripts/verify-ac-av-798.ps1 -Scope turnaround -Samples 30
```

脚本构建 release worker、跑六场景、断言硬门并落盘 raw/summary；退出码非 0 即
门失败。
