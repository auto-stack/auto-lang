# VM 渲染加载响应性（PLAN-708 S 档 + PLAN-711 已交付面 + 目标态链接）

> 状态：current-state 为已交付行为（PLAN-708 S 档 + PLAN-711，2026-09-30）；目标态（§2 worker 线程边界）为 **proposed**（另立 Plan，未实施）。两态严格分离，proposed 面不构成现行契约。

## 1. 已交付面（PLAN-708 S 档，实测锚定）

### 1.1 热点归因方法学与结论（可复现）

- 归因协议：`AUTO_MEMO_DIAG=1` 时间戳 stderr 采集 + autoui-verifier MCP 驱动逐页导航；判别实验用最小 scratch 语料隔离单因子（tmp bench 四组：同内容带/不带 preview-card、显式代码 prop 变体）。
- 结论（gallery 93050a6a，release 980304eb）：页构建秒级成本 ~100% 归一于 **preview-card 每实例 `VueGenerator::new()`**（~1.1–1.3s/卡，内容无关、跳过代码生成字符串化仍等量——成本在 `WidgetRegistry::with_defaults()` 的 register_defaults + 全 schema 折叠）。
- 处置（已交付）：`with_defaults` 进程级 OnceLock 单次构建 + Clone 深拷贝出借；`run_session` 启动后台预热线程把一次性首建移出 UI 输入路径（竞争态最坏=一次性内联，语义等价）。
- 实测：冷构建 row 8.1s→365ms / datatable 9.6s→27ms / area-chart 1.2s→1ms / filetree 1.4s→2ms；复访 memo 命中 0–1ms；原始 10–30s 卡死消除（突发输入窗口实测 26.4s 连续占用 → 输入排队即时返回）。

### 1.2 memo 失效边界（已交付契约，ADR-26）

- `AUTO_OUTLET_MEMO` 三态门：`0` 诊断强制关（覆盖显式 `memo: true`）> `1` 强制开 > 未设/其他值回落 prop；outlet 页 prop 未设=**缺省 on**，菜单/nav 族字面量缺省关（PLAN-045 语义保留）。
- 解析面：`outlet` / `outlet (memo: true)` / `outlet (memo: false)` 三形态（`Option<bool>` 贯通 parser→ast→extract→aura）。
- 宿主 UI epoch：组件局部 UI 态（preview toggle/tab/copy、nav 组开合、热重载 reload）不 bump VM seq——epoch 通道（`MemoKey.ui_epoch`）废止在册产物，堵 memo 回放陈旧面缺口。
- Degrade 不再静默：`AUTO_MEMO_DIAG=1` 输出 `[MEMO-DIAG] site=6 DEGRADE page=… reason=…`（DataTable 每帧全量重建 9.5s 的可观测性缺口实证修复）。
- 已知未闭面（PLAN-711）：DataTable 静态扫描 Degrade 根因（memo_block reason）与 FileTree 恒 FILL 的 computed 依赖缓存。

### 1.3 观测边界

- `AUTO_MEMO_DIAG=1`：`[VM-VIEW] build_ms` / `[VM-VIEW-OUTLET] outlet_ms` / `[MEMO-DIAG] site=N HIT fast|version-fast|slow|MISS|FILL|DEGRADE`。
- MCP `press` 往返 ≈ UI 线程 update+render 轮时长（action 等待渲染轮）——可作占用代理，不可作上屏证明（在屏证据用截图）。

## 1.5 已交付面（PLAN-711，2026-09-30 实证锚定）

- **CPU 可续跑片**：`DriveBudget::{Legacy, CpuSlice}` 双档（片 4ms 墙钟/指令天花板 1M/64 步查钟；10M 累计护栏跨片）；预算耗尽 slice 档=Runnable（栈完整可续跑）、legacy 档=真错误——静默假成功缺陷已除。
- **Init demand 登记簿**：渲染期只登记（判定≠完成）；同代际去重、身份变化=新代际+旧代际一次取消（凭据映射清理）；props 快照重播种（统一根态交错等价）；派发泵 FIFO 依赖序（页先 child 后）、五态观察。
- **帧通知泵**：`listen_raw` 条件订阅（零 demand 无订阅）+ 8ms 轮次有界泵；tick 泵退回纯 I/O；完成 bump 宿主 epoch（memo 失效）+ dirty 链。
- **骨架/失败占位**：pending→`Loading… (Widget)`、Failed→可诊断错误占位（不无限 Loading）。
- **computed/冷构建残面**：outlet 扫描穿入 MemoBlock/Conditional（DataTable memo 生效）；memo_slots_fp 不可展开槽标记回退+dyn_deps 防护（FileTree 破恒 FILL）——两病灶页实机 memo 命中。
- 实测：`docs/plans/reports/711-runtime.md`（长 CPU 20 样本+泵占用 4ms 恒+gallery HIT 序列）；单测族 plan711_cpu_slice（8）+ plan711_init_demand（6）。

## 2. 目标态（proposed — worker 线程边界，另立 Plan 未实施）

以下为 PLAN-708 r2 设计并经 T-00 冻结裁决、随 r3 移出单独立项的调度契约；完整设计见 [proposed 设计文档](../../../../../design/autoui/vm-loading-responsiveness.md)：

- **Init demand/挂载代际**：view 经 interior-mutable sink 登记需求（AppId/路径/key/mount generation），渲染不派发 Init；queued/running/runnable/waiting/completed/failed/cancelled 状态机；`child_init_should_fire` 判定即写身份的重排。
- **帧通知泵**：`listen_raw` 收 RedrawRequested（iced_winit 0.14.0 frames 广播在 present 之前的异步回环序障——消费帧消息时该帧 present 已返回）；条件订阅仅在有未完成 demand 时激活；有界 CPU 泵（初值 4ms/4096 指令/64 步查时钟/轮次 8ms，D-2 冻结）。
- **CPU 可续跑片**：`ParkedWait` 增加 CPU 凭据（保留 707 三凭据只增不改）；engine 预算耗尽静默假成功 `Completed(Ok(()))`（engine.rs 段驱动耗尽臂）的正确性修复。
- **骨架帧**：loading 占位 + 完成通知真实刷新（AC-04..08/10/11 承接）。
- **computed 残面**：DataTable 静态扫描 Degrade 根因、FileTree flatten_tree 依赖缓存。

## 3. 历史与证据

- 归档计划：`docs/plans/archive/708-vm-render-responsiveness.md`（PLAN-708:r3）。
- 实测报告：`docs/plans/reports/708-baseline.md`（冻结值 §3/§4）、`docs/plans/reports/708-decision.md`（D-1..D-6 裁决）。
- ledger：P708-1（designs）/ P708-2（reviews）。
