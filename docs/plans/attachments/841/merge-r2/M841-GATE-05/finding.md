# M841-GATE-05 — parked Init 段 resume 失败不回写需求相位，滞留 InFlight 被静默翻 Done

发现：2026-10-27，merge r2 统一门 Phase A `ui-t`（auto-ui `cargo t`，UI `bfad5149`）。

## 症状

```
auto-ui-runtime ui_tests::plan492_tests::m1_pkg_fstr::pkg_canary_undefined_var_kills_bar_init
panicked at plan492_tests.rs:482: negative control must reach Init and fail there
  left: Some(Done)  right: Some(Failed)
```

全档 2203/2430：2202 过 / 1 败 / 227 未跑（nextest fail-fast 取消）。同一冻结提交上
**隔离重放通过**（`cargo t pkg_canary_undefined_var_kills_bar_init` 1/1 绿，0.97s）——
失败仅在整档并行负载下出现。捕获输出证明 BarChart Init 确实执行且失败：

```
[VM-CPU] handler_BarChart_Init slice-resume FAILED: DivisionByZero
```

## 根因（代码级，冻结源 UI bfad5149）

1. `vm_bridge.rs dispatch_pending_inits` 派发循环：`call_handler_for_cpu_slice` 返回
   Ok 且任务 parked → 记录置 `InFlight` 并 break（依赖序）。
2. 负载下 CPU 片预算耗尽先 park（隔离跑时同步完成，走 Err 臂正确记 `Failed`——
   这解释了定向重放绿/整档红的分叉）。
3. `resume_cpu_slices` 的 `SegmentOutcome::Completed(Err(e))` 臂：释放栈、记
   `report.failed`、eprintln——**不回写 `init_demand_records`**，记录滞留 `InFlight`。
4. 下一轮 `dispatch_pending_inits` 第 1 步：InFlight 探测发现 `is_handler_parked`
   已出册 → 按"段已收敛=成功"契约**静默翻 `Done`**（silent_done_transitions，
   PLAN-712 r2 T-16 占位符路径）。

同类缺口：`resume_ready_parked`（I/O 等待 tick 泵，vm_bridge.rs:2036）的
`Completed(Err)` 臂同样只进 report 不回写——I/O park 后 resume 失败的 Init 同样会被
静默翻 Done。

## 影响面

- 观察面：`init_demand_phase` 消费者（渲染占位符「Loading…」清除、MCP/诊断）会把
  resume 失败的 Init 当作完成；错误本身仍经 `[VM-CPU]`/`report.failed` 通道可见。
- 触发条件：Init 段先 park（CPU 片耗尽或 I/O 等待）后 resume 失败——全档并行负载
  或真实 I/O 故障场景；同步失败路径不受影响。
- 计划492 金丝雀（`negative control must reach Init and fail there`）在整档下确定性
  捕获此缺口；M841-GATE-04 前轮修复（准入/快照所有权）与本缺口正交。

## 关闭 oracle

1. 回写修复：两处 resume 失败臂将对应 InFlight 需求记录置 `Failed`
   （按 `namespaced_handler_fn_name(widget,"Init")` 反查 widget）。
2. 强制 park 回归测试（确定性，不依赖负载）：小 `max_steps` 派发 → park/InFlight →
   `resume_cpu_slices` 失败 → 断言相位 `Failed` 且随后 `dispatch_pending_inits` 不再
   静默翻 Done（silent_done_transitions==0）。
3. 受影响定向：plan841 快照族、plan492 金丝雀族、plan711 CPU 片族、plan712 占位符族、
   plan437 child-init 族；`cargo check -p auto-ui-runtime`。
4. 新冻结提交上整档重跑（ui t/tu/ti + lang t/tv/tt/tu + Man/发行/792/777）。

## 原始日志

- 整档失败：`../ui-t.log.txt`（本目录上一级）
- 隔离重放绿：`isolated-replay.log.txt`
