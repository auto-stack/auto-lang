# AutoUI VM 加载响应性：执行预算、挂载调度与 worker 边界

> 状态：**proposed，尚未实施**（2026-09-29，PLAN-708 r2）。
> 这是专题架构设计；当前实现仍以 `docs/specs/auto-lang/{ui,vm}/` 为准。
> 关联：[PLAN-708](../../plans/708-vm-render-responsiveness.md)、[帧预算设计](vm-frame-budget.md)、
> [Design 34](../34-vm-store-ownership-and-handler-discipline.md)、[组件时间源](component-time-and-events.md)。

## 1. 问题与设计范围

用户观察到 widgets-gallery VM 切页时卡顿。当前代码能证明：handler 与恢复段在 UI 线程运行，view 构建还会派发 child Init；但尚无分段性能证据，不能断言 10–30s 全由 Init 引起。gallery 已显式开启侧栏和 outlet memo；Row/DataTable 无 Init，FileTree 的拍平在 computed。

PLAN-708 实施短期与中期：建立测量、修正缓存、让初始化离开 view、在 UI 段执行中加入合作式预算。长期 worker 在本设计中提出，另立计划实施。本设计不裁定 Design 34 中 actor/store 的全局所有权选择，也不引入 handler 受限语法。

**目标**：加载中的滚动、窗口操作、取消与再次导航继续推进；首帧与完整就绪分别测量。通用原生 FFI、编译加载及任意大布局不能仅由字节码预算给出硬实时保证；它们在基线与端到端门禁中单独计时，超限不得判通过。

## 2. 阶段拆分

| 阶段 | 实施内容 | 交付边界 |
|---|---|---|
| S | 分段基线；memo 失效/诊断关闭；重复构建和示例代码生成的定向减负 | 优化由实测决定，不把已有 memo 标志算成新增收益 |
| M | 可续跑 CPU 段；初始化挂载状态机；真正的显示路径调度；就绪后刷新 | 单个长 Init 与恢复段不能继续整段占 UI；覆盖测得的 gallery 热点 |
| L | VM/视图准备由独占 worker 负责，UI 消费不可变展示快照 | 本文仅设计，不在 PLAN-708 发布为已交付行为 |

实施分为可独立复核的测量/缓存阶段、预算与挂载阶段、端到端收口阶段；共用 708 worktree，不在测量阶段先开启全局缺省或改变 VM 执行语义。

## 3. 可续跑的段预算

### 3.1 执行结果

UI 专用执行入口需要区分：

```text
Completed(value/error)     逻辑调用结束；才清栈、解 busy、提交完成通知
Waiting(wait, continuation) I/O/定时等尚未就绪；已有 702/707 凭据语义
Runnable(continuation)      CPU 预算用尽；栈完整，下一轮继续
Cancelled(reason)          挂载代际已失效；清理一次，不再恢复
```

具体类型名称由探针冻结，现有公开 legacy 同步入口保持其返回契约；不能在它们的 `Result<Value>` 中塞入未完成结果。CPU Runnable 不伪装成 I/O Waiting，也不通过 HTTP readiness 热轮询。

保存 `AutoTask` 的 ip/bp/ram、调用/异常/闭包帧、原始调用参数和返回槽所有权。首次派发与 resume 共用预算策略；到片末时不执行 completion 清栈、不重进 prologue、不重发 native 请求。保留同 handler 在途重入门与错误面。指令数/堆增长安全护栏跨 CPU 片累计，不能每让出一次就重置以绕过原 runaway 保护；I/O 等待不计 CPU 占用。

### 3.2 调度与公平性

提议初值：每片 4ms、指令上限 4096、每至多 64 指令查时钟；一个会话宿主轮次总 CPU pump 8ms。数值为待验证配置，不是实测结果。只在安全指令边界让出；额外记录最慢单条 native/FFI 指令。若一条指令本身超过预算，必须缩小/分块/异步化该真实热点；不能称检查间隔提供了硬上界。

就绪任务轮转；每次先捕获有限的 ready 集，预算耗尽立即返回事件循环，不能在 `resume_ready_parked` 的 loop 中取出、续跑、重新放回后又本轮取出。AppId 间轮转；统计首次派发与所有恢复的总占用。繁忙消息批次也受共享轮次预算约束，不能每条输入都重新领满额。

CPU 片期间默认延续当前共享堆体系，不增加跨线程写者。为避免新的 CPU 交错破坏原纯 CPU handler 的语句顺序，同 App 的其他 VM 写事件在 CPU continuation 存活期间有界串行排队；同 handler 重入仍遵守 702，I/O park 处恢复现有交错边界。滚动/resize/close 与宿主局部交互不等待 VM。输入代写、timer、props 播种、热重载和 MCP fixture 均在此纪律内，不能绕过后门改同一根态。

提议每 App 非合并事件队列最多 128 项；仅可合并明确可覆盖的输入值，不合并点击等有副作用事件。满队列拒绝入队并给可观察 busy/错误，不无限增容或静默丢动作。导航若使 Init 作废，先取消其 continuation，再提交新的 route/props；普通长 handler 不因任意导航无声被杀。取消不回滚已经发生的副作用和前缀写入；不得承诺事务原子性。

CPU continuation 未结束时展示最后一次已提交的展示树或加载骨架；避免重进 builder 再播种 props/执行 computed。MCP 标明任务在途与已提交快照版本，不把中间态宣布为最终就绪。可见中间态仅在明确的 I/O park 边界沿 702 保留，具体观测矩阵在 decision 中冻结。

## 4. 初始化状态机

```text
Discovered → Queued → Running → Completed
                        ├→ Runnable → Running
                        ├→ Waiting  → Running
                        ├→ Failed
Queued/Running/Runnable/Waiting → Cancelled（卸载、key/route 代际、reload/close）
```

身份包含 AppId、widget/既有实例路径、key 与 mount generation。沿用当前实例限制；不能为了加队列重做全局多实例状态系统。判定是否需要 Init 与确认 Init 完成分开记账；首次发现时原子预留，重复 view/MCP 构建只更新发现簿记，不二次入队。A→B→A 的第二个 A 是新代际。

view 只登记需求、构建轻量骨架；不运行 Init。父/页完成后才准备依赖其 props 的 child；页无 Init 时仍发现其嵌套 child。pending 子树不成为可永久命中的 memo 产物，挂载/timer/回调身份必须重放正确。根/module/store 初始化保持 module → root/store → dependent child 的既有依赖顺序，真实调用路径须调查；不能将它们笼统并发启动。

Init dispatcher 需返回带任务身份的观察结果：Completed、Waiting、Runnable、Missing 或 Failed。`call_handler_for` 的 Ok 只说明段已接受的情况不能清 loading。Cancelled 项禁止再次调用 VM；其 wait/堆份额/busy/队列槽随来源纪律一次清理。与 707 共同消费的等待凭据必须逐种映射清理能力，不能只删除桥登记项后放任生产者永久存活。

错误沿 `[VM-HANDLER] ... failed`，界面进入可诊断错误/重试状态；不会无限 Loading。正常程序中 Missing Init 静默；声明了却导出缺失的异常不能伪装成完成。

## 5. 帧通知与真实接线

当前 VM 轨入口是 `run_session` 的 `update_inner`、DesktopSession update 与 `dynamic_view_impl`；通用 ComponentIced 的 trait 方法不足以覆盖它。

期望顺序：

```text
route/mount generation 确立
 → view 登记 init demand、生成带版本的骨架
 → layout/draw、帧提交
 → 该 window/AppId/代际的帧通知
 → 有预算的 Init pump
 → 完成通知 + component dirty + AppState.view_dirty + MCP/cache 失效
 → 完整内容帧
```

iced 0.14 的 `window::frames()` 由 RedrawRequested 转换，其广播点位于 compositor.present 之前；它不是成功 present 回执。设计不把 update 末尾、sleep、普通 tick、单独 frames 回调当成已上屏证据。

T-00 探针在当前锁定 iced 版本验证是否能用公开帧事件加带代际的 draw 标记构成下一事件循环轮次屏障，并以平台呈现时间线/在屏采样证明目标顺序。需覆盖 surface 重试、resize、首次 view 无输入、多 App、多窗。若不能提供足够屏障，本阶段只准推进基线/缓存准备，回到 needs_replan 选择明确的 renderer 回执适配；不能悄悄用 tick 代替 AC。

唤醒不能依赖 pending 发现后才挂上的条件订阅：预先建立按 AppId 路由的轻量通知通道，view 首次登记需求时仅发一次调度信号；无任务时不跑周期计算泵。不可见/最小化窗无需等待永不来的成功呈现，可按可见性判据推进预算执行；恢复可见后必须呈现最新已完成代际。初值与信号方式由探针冻结。

所有提前 return 臂都必须传播真实状态变更到 AppState.view_dirty。component dirty 与 AppState dirty 分开验；MCP 快照和屏幕用相同完成代际，不能只让测试看见状态而屏幕保持缓存。

## 6. view 计算与缓存

memo 开关优先级：env=0 强制关；env=1 兼容强制开；env 未设/其他值时读取显式 prop（true/false），prop 缺省才开。必须保留“未设”和 false 的区别，不能先解析成 bool 丢失它。改变 046 的旧并集规则需明确记录并更新测试。

宿主局部状态版本（preview show/tab/copied、nav 等）、route/params、theme/popover、模板及 mount generation 都进入适用缓存键或明确保守失效。探针/probe 配置分键，所有缓存仍受 ScanVerdict::Degrade 约束。完整生命周期簿记不能因缓存命中被省略。

布局/Element 成本与 VM 成本分别计时；未改变状态的诊断查询不得再次生成整页示例代码或重跑 Init。代码示例文本可按模板/生成器/选项键缓存；MCP 与显示共享一次已提交构建结果时保持 probe、bounds、状态采集顺序。禁止关闭 MCP 作为正式性能修复。

computed 仍有 `Result<Value>` 同步契约。其已有 computed signal 可以减少重复计算，但冷 miss 仍可能长：T-00 必须测 `flatten_tree` 等。若超过端到端预算，本计划需用明确的依赖版本缓存/准备任务使该热点离开 view，准备完只返回 Ready 值；pending 不写成 Nil/空成功、不污染 memo。缓存复用不能覆盖合法 props 变化或依赖变化。副作用/无安全依赖面的表达式不能自动改成异步 computed；需具名适配或 replan。任意通用 view 求值并不由 handler quantum 自动解决。

## 7. 长期 worker（仅设计）

提议独占所有权模型：一个 App 的 AutoVM、任务栈、model/store、Init/timer/parked 与视图准备均由一个 worker 拥有；UI 保留不可变已提交展示快照，执行 iced layout/draw 和窗口交互。UI 不等待持有 VM 的 Mutex。

| 通道 | 候选消息 / 约束 |
|---|---|
| UI → worker | 带 AppId/generation/seq 的输入、route、resize hint、取消、shutdown；有界（候选 128），只有可覆盖输入允许合并 |
| worker → UI | SnapshotReady(version)、task terminal、错误；展示快照只保留最新一份，生命周期完成通知不能静默覆盖 |
| I/O → worker | 702/707 就绪与取消回执；保持原等待凭据，VM 栈不跨线程 |
| MCP | 已提交快照只读；动作经同一个排序入口，标记接受/完成区别 |

需提供复制/增量成本、图像/资源句柄可传输边界，拒绝把 Rc/裸堆引用/iced Element 直接跨线程。可以在线程内部构造非 Send 的 VM 对象；不能从 RefCell 一词推导“全部改 Mutex”。关闭/热重载撤销旧 generation、停止接受旧消息，释放等待资源并返回退出回执；UI 退出不无限 join。

worker 不自动解决 UI layout/draw 的超大树成本；虚拟化、Element 复用按帧预算设计另立实际需求。长期方案应在后续实现证据通过后才进入 current-state Spec。

## 8. 决策门与性能验收

PLAN-708 T-00 输出 `docs/plans/reports/708-decision.md`：实际二进制/依赖/设备、归因、帧屏障、quanta 安全、native 最大成本、部分状态可见性、队列与 snapshot 内存上界、707 接口配对。每个未知项有通过/失败与后继动作，失败不能以债务替代。

r2 提议的初始门禁：60Hz 单帧 16.7ms 是优化目标；加载期 UI 连续占用 max≤50ms、输入到反馈 p95≤100ms/max≤250ms、切页首次骨架呈现 p95≤100ms/max≤250ms。这些是拟议验收参数，需要在进入行为实施前确认 r2；T-00 不得自行放宽。冷编译时间单列，不计为已开窗会话的导航指标。

实际 gallery、无 Init 页、计算型 child、long CPU 首段/恢复段、async Init、长 computed、切页取消、多 App 和 surface 恢复必须全部测；真实 first present 与 MCP ACK 不混用。脚本复用 autoui-verifier 基础设施，输入和时间戳在 UI 侧采集，不以 MCP 网络往返作交互时延。

## 9. 实施与知识沉淀

设计先行登记；PLAN-708 保留 T/AC 稳定 ID并补新项，按阶段在同一 worktree 实施。最终修改 ui/VM Spec 的只有已验证的开关、挂载、预算与同步边界；不先占 ADR-27 发布 worker 目标态。707 并行接口变化必须按实际落地 commit 重新验证。r2 自检不是独立实现复审。

## 10. PLAN-711 实施对照（T-07' 完备性核对，2026-09-30）

> 逐项对照 §7 worker 边界表与 §3-§5 契约在本计划（单执行者形态）的落地状态。
> **proposed 与现状分离保持**：worker 线程边界未实施，仍为 proposed（另立 Plan）；
> 其语义等价物已在单执行者上先行交付。

| §7/§3-5 项 | 设计态 | 711 落地态 | 判定 |
|---|---|---|---|
| UI→worker：AppId/generation 输入、有界 128、可覆盖合并、满拒 busy | worker 通道 | `cpu_write_queue`（128 有界/同键合并/`WriteQueueFull` 可观察 busy）——单线程同构 | 语义先行 ✓ |
| worker→UI：SnapshotReady/task terminal/错误、完成不静默覆盖 | worker 通道 | 完成通知→epoch 失效+component/view dirty+`[VM-HANDLER]` syslog 面（MCP/屏幕同相经 gate_dirty 链，T-05/T-09 实机） | 语义先行 ✓ |
| I/O→worker：702/707 凭据原样、栈不跨线程 | worker 通道 | 702 tick 泵+三凭据原样+`CpuRunnable` 只增（D-3）；全部单线程 | 原样 ✓ |
| MCP：已提交快照只读、同一排序入口、接受/完成区分 | 通道契约 | 既有 shared-state 快照+完成代际同相（T-09 §2 取证） | 语义先行 ✓ |
| 取消/退出：generation 撤销、资源一次清理、退出回执 | worker 通道 | `register_init_demand` 代际取消+`cancel_parked_by_fn` 凭据映射清理（drop_async_result/stream_cancel/纯栈） | 语义先行 ✓ |
| parked 边界：CpuRunnable 凭据、帧驱动、轮次预算 | §3 契约 | `ParkedWait::CpuRunnable`+`resume_cpu_slices`（FIFO 快照/8ms 轮次/4ms 片，D-2 校准） | 已交付 ✓ |
| Init 状态机（判定≠完成、五态观察、依赖序） | §4 契约 | demand 登记簿+`dispatch_pending_inits`+骨架/失败占位 | 已交付 ✓ |
| 帧通知序障（listen_raw 异步回环、消费在 present 后） | §5/D-1 | 条件订阅+消费臂+R-1 实证（§711-runtime） | 已交付 ✓ |
| **worker 线程边界（独占所有权、快照跨线程、不跨线程 Rc/Element、退出 join）** | **proposed** | **未实施**——单执行者不涉及；worker 化 Plan 的核心面 | **proposed 保持** |
| computed 值稳定性（列表入信号网） | §6 保守面 | 706 准入守卫拒绝维持；memo 层标记回退+dyn_deps 防护达成页面级收益；值稳定性属后续设计面 | 部分交付（登记） |

核对结论：§3-§6 的调度/状态机/帧通知/computed 契约已按"单执行者先行"交付并实证；
§7 的线程边界（本设计唯一的架构跃迁面）保持 proposed——其全部通道语义已在单线程
形态先行验证，worker 化 Plan 可平移。无缺口需要回补本设计文档正文。
