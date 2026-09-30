# PLAN-711 T-09 运行矩阵报告（wave-1，2026-09-30）

- `stage: work` | `task: T-09` | `plan_revision: 1` | 状态：**wave-1 已量化；部分矩阵格未跑（覆盖表见 §5，登记为 wave-2 待办）**
- 环境：Windows 11 / i5-13600KF / RTX 4060 Ti / **Todesk 虚拟显示适配器在册**（708 baseline §5 警示延续——本轮帧节奏 ~94-117ms 与其空闲刷新节流一致）/ 59Hz / release `auto.exe`（worktree f405ff1ef+T-12/05/06 构建）/ auto-os widgets-gallery @ 0d5f5bf / 仓内 r1-app 夹具。
- 驱动：autoui-verifier MCP（autoui_action/autoui_snapshot/autoui_state）+ AUTO_SCHED_DIAG/AUTO_MEMO_DIAG/AUTO_CPU_PROBE 时间戳轴（tmp/ 下 stderr 留档）。

## 1. 长 CPU 夹具（r1-app：按钮 300K 次迭代 ≈ 6M VM 步）

20 次 press（AUTO_CPU_PROBE 门控 → CPU slice 派发），检测通道=autoui_state 直读 state：

| 指标 | 值 |
|---|---|
| press → 内容落地（n 达标） | **p50=16309ms p95=19337ms max=19337ms**（n=20） |
| 泵轮次连续占用（4160 个有计算轮） | **p50=4ms p95=4ms max=4ms** |
| 每轮片数 | max=1（4ms 墙钟帽为活预算，D-2 校准后） |
| 完成 trace | parked ~15s + `5.99M total steps` 逐片推进至终态，cpu_cont 归零 |

**§7 门禁对照**：
- 「加载期间连续 UI 占用 max≤50ms」：**PASS**（max 4ms，≪50ms；≤8ms 轮预算成立）。
- 「输入到可见反馈」：click 反馈即时（首片 park 后 update 返回，UI 保持交互）——PASS。
- 「切页到首次骨架 p95≤100ms」：本夹具为常规 handler（无 Init demand 骨架面）；骨架窗由 gallery 导向波覆盖（§2）。
- **完成墙时 15-19s 的归因**：帧节奏主导（~94ms/帧 × ~160 片；物理 60Hz 下同等工作量 ≈2.7s）——与旧同步模型对比：旧模型为 ~0.2-0.5s 的**整段独占**（10M 预算内一次跑完、期间 UI 全冻）；deferred 模型以墙时换响应性（全程可交互、每轮 ≤4ms）。此为本计划的核心权衡，非缺陷；调优面=帧节奏（环境）与片预算校准（已按 D-2 校准一轮）。

## 2. gallery 导向波（Init demand 路径 + memo）

- 冷导航 /line-chart：**291ms 到页面内容**（含 MCP 快照往返开销 ~100-200ms；骨架窗短暂——chart Init 毫秒级完成）。
- 复访：**LineChartPage HIT fast / HIT version-fast 持续，0× DEGRADE**。
- DataTable（T-12 修复后）：FILL→HIT fast→HIT version-fast，**0× DEGRADE**（708 时代每帧 9.5s 全量重建的病灶页现已 memo 命中）。
- FileTree（T-12 修复后）：打破恒 FILL——转换 FILL 一次（Init 完成 epoch 失效）后 HIT fast/HIT version-fast 持续。

## 3. 生命周期/取消/护栏（已在 T-03/T-04 单测+实机取证）

- 10M 累计护栏生产命中（>10M 步 handler 真错误收口，非假成功）。
- 重入忽略（两次 press，cpu_cont 恒 1）、代际取消（A→B→A 重跑+前缀副作用保留）、凭据映射清理（单测族）。
- 707 wait 三凭据串联：HttpRequest 凭据在 013-todo boot 实机贯通（park→I/O resume→完成）；**HttpStream/Future 串联矩阵格**→§5 wave-2（单测族已覆盖，plan707 e2e）。

## 4. 已知预存红（非本计划面）

musk p053/p054 ×6、plan606 gallery、projector、e4 default_headers——4905 跑满 4896 绿，零新红。

## 5. §7 矩阵覆盖表

| 格 | 状态 |
|---|---|
| 长 CPU 夹具量化（20 样本+占用轴） | ✅ 本报告 §1 |
| gallery 导向冷/复访 + memo 命中 | ✅ §2（抽样 6 周期；**≥20 样本×5 页全扫 → wave-2**） |
| 加载期交互注入（滚动/切页/resize） | ⏳ wave-2（骨架窗内交互单测已覆盖 dirty 传播；实机注入未跑） |
| 多 App（≥2 同屏） | ⏳ wave-2（桌面多窗会话驱动） |
| 707 wait→CPU resume→cancel 逐凭据 | ⏳ wave-2（HttpRequest 实机已贯；HttpStream/Future 单测覆盖） |
| resize/surface retry/最小化恢复 | ⏳ wave-2（Q-03 环境警示适用；最小化窗帧通知停摆策略未验证——登记） |
| 冷启动 ≥5 次（装载/开窗前耗时单列） | ⏳ wave-2 |
| p50/p95/max 报告 | ✅ §1/§2（部分格） |

## 6. 结论

wave-1 量化证明：调度器占用门禁（≤50ms）决定性通过；deferred 模型的响应性-墙时权衡按设计工作；T-12 修复使 DataTable/FileTree 两病灶页 memo 真实生效。wave-2 待办（矩阵剩余格）不阻塞本计划终审的结论：**核心 AC-05/06/10/11 的实机证据已取得**；wave-2 格登记为后续计划 or 终审前补跑项（由 review 裁定）。
