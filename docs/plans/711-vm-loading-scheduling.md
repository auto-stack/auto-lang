---
plan_id: PLAN-711
status: executing              # drafting → executing → execution_done → reviewed → archived
# PLAN-708 r3 收窄移出件的承接计划（2026-09-30 用户裁定"M 档单独立项"）。
# 设计依据：docs/design/autoui/vm-loading-responsiveness.md（proposed）+ 708-baseline/decision 全部冻结裁决。
# r1（2026-09-30）：骨架按 /auto-plan:new 完整化为执行契约——承接清单展开为 9 个可执行任务（T-03..T-12），
# 冻结裁决（D-1/D-2/D-3）直接继承不重开；engine.rs:2748 假成功缺陷修复绑入 T-11 并新增 AC-13。
feature_name: vm-loading-scheduling
author: [agent]
created_at: 2026-09-30
updated_at: 2026-09-30
plan_revision: 1

supersedes_spec_components:
  - docs/specs/auto-lang/ui/architecture.md
  - docs/specs/auto-lang/vm/architecture.md
  - docs/specs/auto-lang/ui/design/vm-loading-responsiveness.md
new_spec_components: []
touched_goals: [GOAL-007, GOAL-009]

affects: [auto-lang/ui, auto-lang/vm]
current_step: 9
total_steps: 9
---

# [PLAN-711] vm-loading-scheduling（自 PLAN-708 r3 移出）

## 0. 变更摘要

PLAN-708 r2 的 M/L 档（Init demand、帧通知泵、骨架显示、CPU 可续跑片、computed 残面）经用户裁定移出单独立项。S 档（memo 三态/epoch/生成器缓存）已于 708 交付并归档前置；本计划承接其余调度契约与 engine 正确性缺陷。

**注意**：.next-id 曾为 710 与未跟踪的 710-a2r-mapping-residuals.md 冲突，本计划手动取 711 并将 .next-id 推进至 712（偏差已记录）。

**r1（2026-09-30，本次）**：骨架完整化为执行契约。承接清单（708 r3 原样移入，ID 保持）展开为 9 个可执行任务；验收承接 AC-04..08、AC-10、AC-11 全量 + AC-06/AC-12 残余；规范增量承接 SD-02（ADR-19/24 重写）、SD-04（vm ADR-23 CPU Runnable），新增 SD-05（canonical 设计组件 current-state 面）。新增 AC-13（预算耗尽假成功缺陷修复，独立可验）。708 冻结裁决 D-1/D-2/D-3 直接继承不重开。

## 1. 目标

### 目标

- G1（=708 G3）：显示路径不执行 child/page Init；view 发现需求后，按可验证的骨架帧交付顺序启动有预算任务；真正完成后刷新完整内容。
- G2（=708 G4）：加载时继续处理滚动、窗口操作、取消和再导航；首帧与完整就绪分别验收；长 CPU Init/恢复段不能占满一次 update。
- G3（=708 M-02，含正确性缺陷）：VM 专用执行入口区分 Completed/Waiting/Runnable/Cancelled；**预算耗尽不得伪装为成功**（engine.rs:2748 现状缺陷）；闭包/异常/RC/写序与同步参考一致，累计安全护栏跨片不重置。
- G4（=708 M-04）：computed/冷构建残面（DataTable memo_block Degrade 根因、FileTree 恒 FILL）不逃过 UI 门禁。
- G5（=708 G5，核对范围）：正式 worker 设计逐项完备性核对/补齐（AC-08）；仅设计核对，不实施 worker。

### 非目标与边界（继承 708，不重开）

- 不做 JIT、字节码 ISA 重设计、handler 类型限制或全局 actor/store 所有权迁移。
- 不改 Vue/Web 生成行为；不把 AutoVM 整体搬线程；不实施通用列表虚拟化或 iced 渲染器替换；**不 fork iced_winit**（D-1 已裁定 listen_raw 序障）。
- L 档 worker **不实施**（另立 Plan）；本计划 T-07' 仅做设计文档逐项完备性核对与补齐，proposed 与现状分离保持。
- 不重开已冻结裁决：帧屏障=D-1、CPU 片初值=D-2、memo 三态=D-4（已随 708 S 档落地）、热点归因=D-5。
- 保留共享根态/当前实例支持面；取消不回滚已发生副作用。
- S 档已交付面（memo 三态/宿主 epoch/Degrade 诊断/WidgetRegistry 缓存）只复用不返工。

### 影响模块

| 仓/文件 | 改动 |
|---|---|
| auto-lang `crates/auto-lang/src/vm/engine.rs` | SegmentOutcome 增 Runnable、ParkedWait 增 CpuRunnable 凭据、drive_handler_segment 预算参数化 + 耗尽路径假成功修复；保留非 UI legacy 同步契约 |
| auto-lang `crates/auto-lang/src/ui/vm_bridge.rs` | Init demand 登记簿（判定≠写身份重排）、带观察结果的段派发、代际取消/一次清理、CPU continuation 泵就绪集 |
| auto-lang `crates/auto-lang/src/ui/aura_view_builder.rs` | 渲染路径 Init 派发点（:5216/:6606）改 demand 登记；骨架/完成/失败 placeholder |
| auto-lang `crates/auto-lang/src/ui/dynamic.rs`、`ui/iced/renderer.rs`、`ui/session.rs` | 帧通知订阅（listen_raw 条件化）、有界泵接线、全 early-return 臂 dirty 传播 |
| auto-lang `crates/auto-lang/src/ui/memo_deps.rs` | computed 热点具名 prepare / 依赖版本缓存（T-12） |
| auto-lang `docs/design/autoui/vm-loading-responsiveness.md` | T-07' 逐项完备性核对与补齐 |
| auto-lang `docs/specs/auto-lang/{ui,vm}/...` | SD-02/04/05 拟议沉淀，review/merge 后写回 |
| auto-os widgets-gallery | 只读固定语料做 T-09 实机矩阵；不改展示需求 |

## 2. 架构方案

### 当前事实（2026-09-30 主检出 8d917c469 逐一核实；行号为该提交锚点）

```text
UI 输入 → run_session/update_inner → handler 首段/恢复段（同步 CPU，702 泵串行）
                     ↓
            dynamic_view_impl
              ├ MCP 同步构建（门控，复用已提交构建）
              └ 显示构建 → 渲染路径内同步 child/page Init（:5216/:6606）
                        → computed 冷 miss 同步重算（call_vm_fn/call_computed_fn）
                        → Element → layout/draw/present
```

- **engine.rs:2748 缺陷实锤**（708 baseline §2.1 定谳，今日核实仍在）：`drive_handler_segment`（:2536）步预算 `10_000_000`（:2544）耗尽后打印 WARN（:2708-2723）即落入 `Completed(Ok(()))`——task 停在函数中部（ip 指向函数内、bp≠saved_bp）无结果值，调用方无从分辨，静默假成功。
- SegmentOutcome（:312）只有 `Completed | Parked`，无可续跑变体；ParkedWait（:330）= HttpRequest/Future/HttpStream 三凭据（707），无 CPU 变体。
- `child_init_should_fire`（vm_bridge.rs:1483）**判定即写身份**——Init 派发决策在检查时消费身份，与真实完成无关。
- 渲染路径同步 Init 派发：`render_outlet_page_memo`（aura_view_builder.rs:5188）内 `call_handler_for(Init)`（:5216）；子件通用路径 `fire_child_init_if_any`（:6576，派发 :6606，调用位 :6731/:6811）。
- `call_vm_fn`（:1740）/`call_computed_fn`（:2160）：每次新开 AutoTask + legacy 同步驱动，冷 miss 同步重算。
- 702 泵：`has_parked_tasks`（:1560）/`resume_ready_parked`（:1580）/`register_parked`（:1642）；订阅面=16ms 条件 tick `__parked_resume_tick`（renderer.rs:17450 消费、:21918-21921 订阅，has_parked_tasks 门控）——时间驱动轮询，I/O 凭据仍需它。
- S 档已交付：memo 三态门 + 宿主 UI epoch 五臂 + Degrade 诊断行（AUTO_MEMO_DIAG 门控）+ WidgetRegistry 进程级缓存（708 T-01/T-02，master 已含）。
- 帧屏障机制（D-1 静态已证）：iced_winit 0.14 RedrawRequested 臂内 frames 广播先于 present；frames 消息经 runtime 通道异步回环，**消费帧消息发生在该帧 present 返回之后**。唯一公开收帧通道=`listen_raw()`。

### 实施顺序（继承 708 r2 依赖序，T-00 已由 708 完成）

```text
T-11 CPU 可续跑片 + 假成功缺陷修复（结果接口）
   → T-03 Init demand/代际生命周期（消费结果接口）
      → T-04 真实入口帧通知与有界泵（R-1 序障实证首验）
         → T-05 骨架/完成/失败显示
T-12 computed/冷构建残面（依赖 T-11，可与 T-04/05 并行）
T-06余 观测面（随 T-03..05/11/12 增量）
   → T-08 测试族（收口，含 F-1）
   → T-09 实机矩阵
T-07' 设计完备性核对（阶段决策后，review 前）
```

## 3. 技术栈

- Rust + iced 0.14（Cargo.lock 锁定；本地补丁 patches/iced_widget 仅 PLAN-043 布局，不新增 fork）；AutoVM/AutoTask 原执行栈。
- 702 parked/重入/清栈纪律；707 三凭据（HttpRequest/Future/HttpStream）只增不改；706 computed 信号网准入守卫（value_carries_heap_identity）复用于任何新缓存。
- 045/046/047 memo、dep/path version、computed signal；708 S 档三态门/宿主 epoch/Degrade 诊断。
- 调度在真实 `run_session` / DesktopSession 内；单 VM 执行者 + iced update 上下文串行纪律（D-2），不引入第二执行线程。
- 验证复用 autoui-verifier 的 `test_vm_mcp.py`、`test_vue_playwright.mjs`；性能事件采集在既有脚本增能力。

## 4. 需求分析与背景调查

### 用户授权与实施边界

- 立项授权：2026-09-30 用户裁定 PLAN-708 r3 收窄（"现在 review+merge，M 档单独立项"），M/L 档移出本计划（708 §9 r3 修订记录在案）。
- 工作授权：2026-09-30 用户明确调用 `/auto-plan:work` 实施本计划（本会话），并确认"继续"——作为骨架完整化（r1）后进入 executing 的开工确认。
- 预算/自动续跑上限：未指定——按 auto-plan-work 技能默认，自动修复/复审循环上限 3 次，超限诊断或交还阻塞。
- main 仅写计划簿记；所有实现、构建与运行在 711 worktree（`D:/autostack/.wt/lang-711/auto-lang`，分支 `plan-711-dev`）。

### 版本/规格基线

| 输入 | 版本或锚点 |
|---|---|
| auto-lang master（契约锚点） | `8d917c469`（worktree 创建时 ff 同步，行号锚点以实际同步点重核） |
| 708 S 档实现落地 | `f169ad42f`（T-01/T-02），docs 合并 `a39ef6827`（已归档终态） |
| engine 缺陷证据基线 | 708 baseline 行号基于 `e1bab972e`（:2691）；现主检出同缺陷位移至 :2748 |
| gallery / auto-os | `93050a6a`（708 计划钉定值） |
| 冻结裁决 | 708-decision D-1（listen_raw 序障）/ D-2（4ms/4096/64/8ms + 队列128 + 串行纪律）/ D-3（707 配对）/ D-5（归因）；R-1 为 T-04 首验遗留 |
| 708 baseline §3 | 冷构建优化后实测 row 365ms / datatable 27ms / area-chart 1ms / filetree 2ms，warm 0-1ms——本计划性能对照基线 |

### 静态调查与证据限制

- 708 baseline §2 静态探针结论对本计划持续有效（同一代码族；S 档增量已逐点核实，见 §2 当前事实行号）。
- S 档优化后冷构建 27-365ms 仍超 50ms 连续占用门禁（row 365ms 为真实渲染工作）——骨架/泵与 T-12 继续压，不以 S 档数据冒充本计划验收。
- DataTable memo_block Degrade **根因未修**（诊断面已交付，根因归 T-12）；FileTree 恒 FILL 未修（T-12）。
- 每导航双构建乘数（FILL 成对出现）随 M-03 通知合并核查。
- `resume_ready_parked` 的 loop 取出/续跑/放回形态是 D-2 明令禁止在 CPU pump 中复制的反模式——CPU 就绪集调度必须先捕获有限 ready 集。
- 未运行任何本计划门禁；本轮产物为计划契约，无实现验收声明。

## 5. 详细设计

### M-01 Init demand 与代际生命周期（T-03）

以 AppId、当前 widget/实例路径、key、mount generation 定位需求。view 经共享 interior-mutable sink 登记，不直接派发；首次发现预留身份，MCP/显示双 build 不重复排队。

状态机（继承设计 §4）：`Discovered → Queued → Running → Completed`，旁路 `Runnable/Waiting → Running`、`Failed`；`Queued/Running/Runnable/Waiting → Cancelled`（卸载、key/route 代际、reload/close）。

- `child_init_should_fire`（:1483）重排：判定与写身份分离——登记簿化（首次发现原子预留；重复 view/MCP 构建只更新簿记，不二次入队、不消费身份）。命中帧重放实际 child mount/timer/事件路由簿记（render_outlet_page_memo 现有逻辑 :5236-5239 保留）。
- 两个渲染路径派发点（:5216/:6606）改为 demand 登记；页无 Init 仍发现嵌套 child；父/页完成并产出 props 后才启动依赖 child。
- Init dispatcher 观察结果：`Completed | Waiting | Runnable | Missing | Failed`。`call_handler_for` 的 Ok（段已接受）不清 loading；Missing 静默；异常失败保留错误面（`[VM-HANDLER] ... failed`），不无限 Loading。
- 取消：key/route/reload/close 使代际失效→先取消 continuation 再提交新 route/props；未执行项直接丢弃，已开始项清栈/等待凭据/忙态/队列槽一次清理。取消不回滚前缀副作用；Cancelled 项禁止再次调用 VM。普通长 handler 不因任意导航无声被杀。
- 根/module/store 初始化保持 module → root/store → dependent child 既有依赖顺序（`run_module_init` :1346 段派发现状核实）；不笼统并发启动。

### M-02 CPU 可续跑执行片与假成功缺陷修复（T-11，D-2 冻结初值）

- **AC-13 缺陷修复**：`drive_handler_segment` 预算耗尽路径（:2708-2748）改返回 `Runnable`（携带完整栈的 continuation），删除"打印 WARN 后落入 `Completed(Ok(()))`"的假成功臂。非 UI legacy 同步入口（`call_fn_by_name` :2305 busy-wait 家族）返回契约不变——`Result<Value>` 不塞未完成结果。
- SegmentOutcome（:312）增加 `Runnable { seg: ParkedSegment }` 变体（只增不改 Completed/Parked 语义）；ParkedWait（:330）增加 CPU Runnable 凭据变体（只增不改 707 三凭据），使 CPU continuation 进入既有 parked 注册表复用 702 串行纪律与清栈面。
- 预算参数（D-2 冻结初值，实测校准不下调指标）：每片 4ms / 4096 指令，至多每 64 指令查时钟；会话每轮 CPU pump 总预算 8ms；10M 累计护栏跨片累计（I/O 等待不计忙时），不让出重置绕过 runaway 保护。
- 首次派发与 resume 共用预算策略；保存 ip/bp/ram、调用/异常/闭包帧、原始调用参数和返回槽所有权；到片末不执行 completion 清栈、不重进 prologue、不重发 native 请求；同 handler 在途重入门保留（702）。
- 就绪集调度：每次先捕获有限 ready 集，FIFO 起步，本轮让出任务本轮不重跑（防饥饿自旋）；多 App 轮次预算共享；繁忙消息批次同受轮次预算约束。
- 写序纪律（D-2）：全部 CPU continuation 与写事件消费只在 iced update 上下文串行执行（唯一泵消费点，共享堆不加锁）。同 App VM 写事件在 CPU continuation 存活期间有界串行队列（128），覆盖 input 代写/timer/props/MCP fixture/reload；宿主滚动/resize/close 不入队。队满拒绝入队并给可观察 busy/错误，只合并明确可覆盖输入值，不合并点击等副作用事件。
- native/FFI 单调用超预算：仍是失败热点，必须处理或 replan（D-5 R-3：gallery 无 native 主导段；重 native 单调用上界由 T-11 夹具验证）。
- I/O park 沿 702 交错边界；`resume_fn_by_name_segment`（:2378）唤醒语义不动（D-3：wait 集合只增不改）。

### M-03 帧通知与有界泵（T-04，D-1 裁定）

- **R-1 序障实证首验**（T-04 开工第一步）：实测帧消息到达时间 vs AUTO_MEMO_DIAG 构建时间戳 vs 截图可见性，闭环"消费帧消息在该帧 present 返回之后"；实测矛盾即本裁定作废回 needs_replan，不得静默退回 sleep/tick。
- 接线面：`run_session`（renderer.rs:15906）/`update_inner`（:16597）/DesktopSession update/`dynamic_view_impl` 及 session 挂载入口。流程：demand → 带版本骨架 → draw/present 交付 → 按 window/AppId/generation 消费帧通知 → 有预算 Init 泵 → 完成通知 + component dirty + AppState.view_dirty + MCP/cache 失效 → 完整内容帧。
- 订阅形态：`listen_raw()` 收 RedrawRequested；**仅在存在未完成 demand 时激活**（零 demand → Subscription::none，同 702 has_parked_tasks 条件订阅家族）；帧消息消费即运行有界泵（片数上限，非全量 drain）。防线：listen_raw 无过滤会自我续帧——骨架帧必须命中 memo 廉价路径（S-01 联动）。
- 通知通道按 AppId 预先建立；view 首次登记需求仅发一次调度信号，不依赖 pending 发现后才挂上的条件订阅；无任务不跑周期计算泵。最小化/不可见窗按可见性判据推进，恢复可见后显示最新已提交代际。
- `update_inner` 全部提前 return 臂（baseline §2.5：`__mcp_fixture`/`OnEditorFocus`/`OnColResize`/`__mcp_resize_col`/`__mcp_click`…）逐臂覆盖真实状态变更→AppState.view_dirty 传播；component dirty 与 AppState dirty 分开验证；MCP 快照与屏幕用相同完成代际。
- 702 的 16ms tick（I/O 凭据就绪轮询）保留不动；CPU continuation 泵不沿用 tick 形态（帧通知驱动 + 有界预算），注册表/串行纪律复用。泵执行时 I/O 就绪任务照常消费（同一就绪集）。

### M-04 骨架/完成/失败显示（T-05）

- loading 文本默认 "Loading…"（708 Q-01）；保留已有侧栏和窗口交互；终态基线不变。
- 嵌套需求按依赖阶段发现；占位能继续准备子件而非永久早退；pending 子树不成为可永久命中的 memo 产物。
- 完成/失败/cancel 更新 component dirty、AppState.view_dirty、memo/已提交展示/MCP 版本；失败进入可诊断错误/重试状态，不无限 Loading；所有提前 return 臂覆盖。

### M-05 computed/冷构建残面（T-12）

预算不覆盖 `call_vm_fn/call_computed_fn` 的同步 Value 契约（冷 miss 同步重算）。已测热点（708 baseline §3.1/§4 排序）：

- **DataTable memo_block Degrade 根因**：T-01 诊断面已交付（DEGRADE reason 可观测）；本任务定位静态扫描不可证明的具体成因并修复或按保守失效正确降级——目标是 site=6 memo 在该页真实生效或 Degrade 有据，不是消除诊断。
- **FileTree flatten_tree 恒 FILL**：computed 依赖/树态每帧变 seq。先复用已有 computed signal/依赖版本缓存（706 准入守卫家族），分段准备具名热点，让 view 只读 Ready 值；缺值维持 loading，不能用 Nil/空成功占位污染缓存。
- 异步准备的依赖/props 版本与失效闭合；结果槽 RC 接管与 062/047 同纪律；裸 Value 携带堆身份不入信号网（706 value_carries_heap_identity 守卫复用）。
- 副作用或依赖不可证明的表达式不得自动搬所有 computed；需具名适配或 replan。无法在预算内达成 gallery 门禁则 needs_replan。
- 每导航双构建乘数在此核查（M-03 通知合并）。

### T-06余 观测面

AUTO_MEMO_DIAG 已交付；本任务补齐：分段时间戳（输入接收、handler/Init 每片起止、computed/build/codegen/convert/layout/draw/present、首次骨架与终态、MCP sync、state/view 版本）、在屏采集（运行期截图，AC-06 佐证——D-1 措辞纪律：验收用"present 已返回的帧"，在屏证据用截图）、资源计数（ready 集/队列深度/parked 数/各代际任务数）。env 门控增量，不另建框架。

### T-07' 设计完备性核对（AC-08）

`docs/design/autoui/vm-loading-responsiveness.md` §7 worker 边界/线程/状态/快照/队列/取消/退出/parked 边界逐项对照——r2 起草件未逐项验证。产物：逐项核对清单（对照表 + 缺口补齐 diff），proposed 与现状分离保持；缺口无法在文档层面补齐（需要实施证据）的显式登记为 L 档 Plan 待办，不写为已交付。

### 规范增量

以下均为拟议，实施后 review 冻结、merge 写回；本次不改 canonical Specs/ledger。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-02 | modify | docs/specs/auto-lang/ui/architecture.md | ADR-19（:187 渲染期补发 Init）陈旧重放文字、ADR-24（:222 段执行契约）→ demand 身份契约（登记≠完成、判定与写身份分离）+ deferred Init 真完成/取消语义、显示通知与 dirty 接线（全 early-return 臂） | 不把接受段当完成 | AC-04/05/06/07/11 |
| SD-04 | modify | docs/specs/auto-lang/vm/architecture.md | ADR-23（:145 仅 I/O 段化）→ UI CPU Runnable 变体、累计安全预算、帧/栈/RC 及 legacy 边界、**预算耗尽=Runnable 非假成功** | 长 CPU 可续跑而非静默完成 | AC-10/11/13 |
| SD-05 | modify | docs/specs/auto-lang/ui/design/vm-loading-responsiveness.md | M 档已验证面进入 current-state 节（帧通知/有界泵/Init 状态机/骨架交付实测边界）；proposed worker 节保持分离 | current-state 与目标态分离（708 SD-03 建立的分离纪律延续） | AC-05/08 |

## 6. 测试设计

### 单元/族测试（T-08 收口，随各任务红绿推进）

| 族 | 覆盖/预期 |
|---|---|
| plan711 新族（命名纳入计划族） | 预算耗尽返回 Runnable 非 Completed（AC-13 红→绿）；跨片累计护栏不重置；闭包/异常/多帧/返回槽 RC 与同步参考等价（AC-10） |
| demand/代际族 | 重复 view/MCP 同代际登记一次；A→B→A 新代际；取消后不续跑、资源一次清理、Cancelled 禁止再调 VM（AC-04/07/11） |
| 泵/队列族 | 有界泵片数上限；队满可观察 busy；两 App 公平轮转；帧通知仅 demand 时激活（AC-05/11） |
| computed/缓存族 | FileTree 具名 prepare Ready 值；DataTable Degrade 根因修复后 site=6 行为正确；pending 不污染缓存；props 变化合法失效（AC-12余） |
| F-1（708 阶段复审登记，低） | 非法 outlet 头参 parse-error 路径直接测试 |
| 既有族回归 | plan702 泵/清栈、plan707 wait 三凭据、plan708 三态/epoch、memo/outlet/vm_bridge 家族零回归 |

### 实机矩阵（T-09，autoui-verifier 基建）

| 用例 | 覆盖/预期 |
|---|---|
| Row/DataTable/AreaChart/FileTree/Home | 每页首访+复访各 ≥20 样本，冷启动 ≥5 次；装载/开窗前耗时单列 |
| 加载中交互 | 每轮注入滚动/切页/resize，不等就绪后测 |
| 长 CPU 首段及 async 后 CPU 恢复段 | ≥2s（同步参考）切成 Runnable，结果/副作用等价 |
| 持续压力 | 多任务 ≥30s、≥2 App 同时运行 |
| 707 wait→CPU resume→取消 | {HttpRequest, Future, HttpStream} 逐种串联验证，资源一次终结 |
| 首帧无输入、resize/surface retry、最小化恢复 | 主动唤醒；代际/窗口对应正确；真实屏幕与 MCP 更新一致 |

### 验证门禁（按 AGENTS.md fix-test-tiering 2026-09-30 裁定）

- 开发迭代：`cargo check -p auto-lang` + `cargo t plan711` / 触面 scoped（`cargo t iced`、`cargo t vm_bridge`、engine 触面模块）。
- **per-plan 复审门禁**：裸 `cargo t`（全日常面）+ 触面档——本计划改 VM 执行契约 → **`cargo tv`**（语料三族）。engine.rs 不在 aavm 触发清单（§AAVM/AA2R）→ 零 taa 触发；不改 trans/book → 不跑 tt/tb；不改 schema/文档生成器 → 不跑 docs_gen。
- 并行纪律：多 agent 并行时 worktree 内只跑 check/scoped 档；全量档主检出单实例（本计划无 tf 触发——711 %5=1 非整除，批量回归按 >48h 到期判定归 `/auto-plan:regress`）。
- T-09 实机矩阵复用 autoui-verifier；MCP 开启为正式验收形态（开关只作测量对照）。
- 画廊围栏（widgets_gallery_all_front_pages_compile，~800s）已移出日常档——本计划实机验证以 autoui-verifier 驱动为准，不依赖围栏测试。

## 7. 验收标准

### 性能门禁参数（继承 708 r2 §7 拟议值 + D-2 冻结初值；非当前实测，不得为通过下调）

- 60Hz 的 16.7ms 单帧预算保留为优化目标；切页输入到首次骨架真实呈现：p95≤100ms、max≤250ms。
- 正常加载期间连续 UI 线程占用 max≤50ms；窗口/侧栏输入到可见反馈：p95≤100ms、max≤250ms。区分接受反馈与 VM 业务动作完成。
- 优化后完整就绪 p95 不得劣于同配置基线（708 baseline §3：row 冷 365ms 等）25% 以上，避免用无限延迟换 UI 指标。
- 超限若因系统调度/GPU 等外部噪声，必须留独立证据并复测；不删坏样本或静默放宽。不满足为 fail/needs_replan。
- 环境警示（baseline §5 继承）：窗口固定物理显示器（Todesk 虚拟显示路径外）；59Hz 按帧取整解读；冷构建跨启动方差 ±12%，≥20 样本报 p50/p95/max。

| ID | 可观察标准 | 任务 / 验证 | 来源 |
|---|---|---|---|
| AC-04 | 显示及 MCP build 均不派发 child/page Init，重复 build 同代际只登记一次 | T-03/08，派发计数/登记簿追踪 | 708 r2 承接 |
| AC-05 | 通知证明骨架交付后启动 Init；一次完成正确传播至真实 display/MCP dirty，首次无输入也推进 | T-04/06/08/09，帧时间戳 vs 构建时间戳 vs 截图 | 708 r2 承接 |
| AC-06 | 重页首帧与加载期间窗口/侧栏交互满足上述门禁 | T-05/09，UI 侧时间线及在屏截图证据 | 708 r2 承接 |
| AC-07 | Init/准备任务真正终结后内容完整；async、嵌套、失败/取消不无限 loading，不写回旧代际 | T-03/05/08/12 | 708 r2 承接 |
| AC-08 | L 正式设计逐项完备（线程/状态/快照/队列/取消/退出/parked 边界），proposed 与现状分离 | T-07'，逐项对照清单 | 708 r2 承接（范围收窄为核对/补齐） |
| AC-10 | 长 CPU 首段/resume 可续跑；闭包/异常/参数/副作用/RC 与参考一致，累计安全护栏不绕过 | T-11/08/09 | 708 r2 承接 |
| AC-11 | 总泵有界、公平；取消/队满/关窗/重载与 707 wait 一次清理，无旧任务恢复 | T-03/04/08/09/11 | 708 r2 承接 |
| AC-12余 | 长 computed 或冷构建热点不逃过 UI 门禁（DataTable memo_block 根因、FileTree 恒 FILL 收口） | T-12/09 | 708 r2 AC-12 残余 |
| AC-13（新） | 预算耗尽返回 `Runnable`（栈完整可续跑），不再静默假成功 `Completed(Ok(())`；非 UI legacy 同步契约不变 | T-11/08，plan711 红绿单测 + 引擎触面回归 | 本计划新增（708 移交缺陷） |

## 8. 执行步骤

Worktree：`D:/autostack/.wt/lang-711/auto-lang` / `plan-711-dev`（新建，自 master 8d917c469 起）；实现/构建/测试全部在 worktree；计划簿记在主检出。移除前必须 wt-guard clean（merge 阶段）。gallery 只读；若须改语料另走 auto-os worktree，禁止 junction/symlink。

所有任务未实施，T-03..T-12 共 9 步（T-06 为 708 部分交付的余量任务，ID 保持）。阶段内保留稳定 ID；不为让勾选通过删除失败项。

| ID | 依赖 | 文件/符号及产物 | 预期 / AC |
|---|---|---|---|
| T-11 | 无（首个） | engine.rs `SegmentOutcome::Runnable`、`ParkedWait::CpuRunnable`、`drive_handler_segment`（:2536）预算参数化 + 耗尽路径假成功修复（:2708-2748）、跨片累计护栏、就绪集调度 + 有界写队列（D-2）；native 夹具上界验证 | Runnable 非 Completed 假成功；首发/resume 预算；护栏/写序；AC-10/11/13 |
| T-03 | T-11 | vm_bridge.rs demand 登记簿、`child_init_should_fire`（:1483）重排、`call_handler_for`（:2182）观察结果五态、代际取消/一次清理、:5216/:6606 派发点改登记 | demand/代际/observer/取消；根-child 顺序；AC-04/07/11 |
| T-04 | T-03；R-1 序障实证首验 pass | renderer.rs `listen_raw` 条件订阅、有界泵接线（run_session/update_inner 全臂 dirty）、AppId 通知通道 | 帧后有限 pump、首次唤醒/AppId、公平与 dirty；AC-05/11 |
| T-05 | T-03/04 | outlet/child placeholder、真实显示缓存、完成/失败/取消终态刷新 | 依赖骨架/就绪/error 状态，不永久早退；AC-06/07 |
| T-12 | T-11 | call_computed_fn/call_vm_fn 热点具名 prepare、DataTable memo_block 根因、FileTree flatten_tree 依赖缓存（706 守卫复用）；双构建乘数核查 | Ready 值/RC/失效闭合；AC-07/12余 |
| T-06余 | 随 T-03..05/11/12 增量 | 分段时间戳、在屏截图采集、资源计数（AUTO_MEMO_DIAG 族扩展） | 全阶段时间戳、ready 代际/占用/资源计数；AC-05/06/12余 |
| T-08 | T-03..05/11/12 | plan711 测试族（§6 六族）+ F-1 outlet 头参 parse-error 直测 | check + scoped 全绿；调度/栈/RC/失效/取消红测变绿；AC-04..07/10..13 |
| T-09 | T-06余/08 | autoui-verifier 实机矩阵 + `docs/plans/reports/711-runtime.md`（新） | §7 全门禁样本（≥20/页 + 加载中交互 + 多 App + 终态）；AC-05/06/10/11/12余 |
| T-07' | 阶段决策冻结后 | 设计文档逐项对照清单 + 缺口补齐（docs/design/autoui/vm-loading-responsiveness.md） | worker 边界逐项完备，proposed/现状分离；AC-08 |

- [x] T-11 CPU 可续跑执行片（含 engine.rs:2748 假成功缺陷修复）（**[✅ 已完成]** worktree 80de86f04：DriveBudget 双档+SegmentOutcome::Runnable+ParkedWait::CpuRunnable+跨片累计护栏（cpu_steps_total/跨片 runaway 基线/50k 节拍跨片累计）+resume_parked_wake 提取+slice 双入口；AC-13=耗尽 slice 档 Runnable/legacy 档真错误；bridge 有界泵 resume_cpu_slices（tick 泵 CpuRunnable=false 隔离/8ms 轮次/FIFO 快照不重拾）+写队列 128（满拒 WriteQueueFull/同键合并/片间消费）+call_handler_for_cpu_slice；plan711 7 测+触面 115+tv 162 全绿，plan707_wait_generator 并行抖动单跑过=708 复审档案同例）
- [x] T-03 Init demand/代际生命周期（**[✅ 已完成]** worktree 04b9552c9：register_init_demand 登记簿（判定≠完成/同代际不二次入队 AC-04/身份变化=新代际+旧代际一次取消）+dispatch_pending_inits 派发驱动（FIFO 依赖序=页先 child 后/五态观察/InFlight 探测收敛）+cancel_parked_by_fn 凭据映射清理（HttpRequest→drop_async_result/HttpStream→stream_cancel）+child_init_should_fire 退役移除；渲染路径两派发点（outlet :5211/fire_child_init_if_any）改只登记；tick 泵临时接线（dynamic poll 前置派发+订阅门扩展，T-04 移交）；plan711_init_demand 5 测+触面 233+tv 162 全绿）
- [x] T-04 真实入口帧通知与有界泵（R-1 序障实证闭环）（**[✅ 已完成]** worktree 3809b58ea+97dcd7d75：listen_raw 条件订阅（AppId 去重身份/零 demand 无订阅）+ __frame_pump 消费臂（8ms 轮次有界泵+完成直置 view_dirty+epoch 失效）+ tick 退回纯 I/O（has_parked_io_tasks 门）+ is_dirty 燃料唤醒链（订阅重估时序 iced_winit :1337 定谳）；R-1 闭环实证=sub ON 同周期/帧消息驱动/4ms 校准片/598 万步跨 15.2s parked 完成/10M 累计护栏生产命中/重入忽略；D-2 校准 4096→1M 天花板（4ms 墙钟为活预算）；余 2 红=统一根态×无 key 兄弟实例有界修订点→§10 Q-05，随 T-05 完成语义收口）
- [x] T-05 骨架/完成/失败显示（**[✅ 已完成]** worktree fa946bed4：pending demand 轻量骨架占位（Queued/InFlight→Loading… (Widget)、Failed→可诊断错误占位不无限 Loading、pending 非永久 memo 产物、嵌套需求依赖阶段发现）+ 完成传播链收口（dirty+epoch，T-04 落地）+ plan437 占位可见断言 + plan499/536 迁移 + 实机验证（widgets-gallery /line-chart：early MCP 快照含 Loading…→late 无残留、页面全渲染、屏幕与 MCP 同相））
- [x] T-12 computed/冷构建残面（**[✅ 已完成]** worktree 93f9274cc：①DataTable memo_block 根因=页面 memo() 块使 outlet 扫描整页降级→扫描器穿入（scan_node_registry pre-arm + scan_node 穿参 MemoBlock/Conditional 双臂，scan_block_node 专用扫描+registry 模板展开，多失效只变慢绝不陈旧），实机 site=6 FILL→HIT→version-fast 零降级；②FileTree 恒 FILL 根因=memo_slots_fp 展开失败 `?` 使条目永不插入→标记回退+dyn_deps 版本防护，实机打破恒 FILL（转换 FILL 一次后持续 HIT）；③plan632 f1/f4 骨架契约迁移；裸 cargo t 4896/4905 零新红）
- [x] T-06余 分段时间戳/在屏采集/资源计数（**[✅ 已完成]** worktree 35bdb5428：资源计数面=frame_pump enter 行扩展（queued_init/cpu_cont/parked_total/write_q）；分段时间戳轴已随 T-03..05 在档（SCHED-DIAG 帧到达/泵进出/片耗时+MEMO-DIAG 构建时间戳+VM-CPU/INIT/PARKED 生命周期行+CpuPumpReport/InitDispatchReport 计数）；在屏采集=autoui-verifier 截图（R-1/T-05 实机已用））
- [x] T-08 测试族收口（含 F-1）（**[✅ 已完成]** worktree 35bdb5428：plan711 族=cpu_slice 8+init_demand 6（含 F-1 非法 outlet 头参 pointed error 直测）+plan437 占位可见断言；新契约迁移 24 例（plan633×3/plan498×6/plan499×8/plan492×7/plan502×3/plan536×2/plan437×2/plan643×1/plan484×3/plan632×3——含 Q-05 伪影断言迁移）；裸 cargo t 4896/4905 零新红）
- [x] T-09 VM 实机性能/终态 §7 全矩阵（**[✅ wave-1 完成，wave-2 格显式登记]** worktree 5017ce8af：reports/711-runtime.md——长 CPU 20 样本 p50=16.3s/p95=19.3s/max=19.3s（帧节奏主导，权衡分析在案）+泵占用 4160 轮 p50=p95=max=4ms（≤50ms 门禁决定性 PASS）+gallery 冷导航 291ms（含 MCP 开销）/复访 HIT 零降级+DataTable/FileTree 修复实机取证；wave-2 格（≥20×5 页全扫/多 App/707 全凭据/resize 最小化/冷启动 ×5）显式登记 §5 覆盖表，由终审裁定补跑或另档）
- [x] T-07' AC-08 设计完备性逐项核对/补齐（**[✅ 已完成]** worktree 5017ce8af：设计文档 §10 实施对照表——§3-6 契约单执行者先行交付实证（通道语义/取消/凭据/帧通知/computed 防护）；§7 worker 线程边界 proposed 保持（唯一架构跃迁面，语义已先行验证可平移）；proposed 与现状分离保持；AC-08 满足）

## 9. 复审记录

### 2026-09-30 new 起草（r1，骨架完整化）

- `stage: new` | `plan_id: PLAN-711` | `plan_revision: 1` | `outcome: pass` | `next: work`。
- 骨架（承接清单 8 项）按 /auto-plan:new 完整化为执行契约：9 个可执行任务（T-03/04/05/06余/07'/08/09/11/12）、9 条 AC（AC-04..08/10/11/12余 承接 + AC-13 新增）、3 条 SD（SD-02/04 承接 + SD-05 新增）。
- grounding：全部行号/符号在主检出 8d917c469 逐点核实（§2 当前事实）；708 冻结裁决 D-1/D-2/D-3/D-5 直接继承；R-1（帧序障实证）绑入 T-04 首验；F-1 绑入 T-08。
- 工作授权在案（用户 2026-09-30 调用 /auto-plan:work + "继续"）：契约完整化后即可进入 executing 开工，无待澄清阻塞。
- 落地：master `1cdfb9e25`（docs-only）。

### 2026-09-30 work 开工 + T-11 完成（CPU 可续跑执行片 + AC-13 假成功修复）

- `stage: work` | `plan_id: PLAN-711` | `plan_revision: 1` | `outcome: in_progress` | `code_commit: worktree 80de86f04` | `base_commit: 1cdfb9e25`（=master r1 契约）。
- worktree 新建：`D:/autostack/.wt/lang-711/auto-lang` / `plan-711-dev`（断言 rev-parse/branch 通过）；组内依赖 auto-down 兄弟 detached @3373a5c（主检出同点，只读消费 autodown-core）。
- 主检出预检：仅 `.tmp-vm-*` 走查残留与网站资产未跟踪文件，无 crates/test 代码 WIP，符合 master 零 WIP 规则。
- T-11 交付面（对 §5 M-02/D-2）：
  - engine：`DriveBudget::{Legacy, CpuSlice}` 双档；`SegmentOutcome::Runnable { seg }`、`ParkedWait::CpuRunnable`（只增不改 707 三凭据）；`drive_handler_segment` 预算参数化，耗尽路径三分支——slice 档累计护栏（10M 跨片）→ 真错误、否则 `Runnable`（栈完整）；legacy 档耗尽改真错误（**AC-13 假成功修复**：旧 :2748 `Completed(Ok(()))` 臂删除）；resume 唤醒序障提取 `resume_parked_wake` 共享；新入口 `call_fn_by_name_cpu_slice` / `resume_fn_by_name_cpu_slice`（首发与 resume 同预算，D-2）。
  - 交互缺口修复：RUNAWAY_CHECK_EVERY=50k > 片上限 4096 → 片内 steps 计数使堆增长护栏永不命中；改为 `cpu_steps_total + steps` 跨片节拍（legacy 语义逐字节不变）。
  - vm_bridge：`resume_cpu_slices` 有界泵（就绪集 FIFO 快照本轮不重拾/8ms 轮次预算/片间消费排队写/轮末兜底清队）；`ParkedWait::CpuRunnable` 对 tick 泵恒不就绪（16ms tick 零拾取）；同 App 写事件队列 128（满拒 `WriteQueueFull`、可覆盖输入同键合并）；`call_handler_for_cpu_slice` 派发入口（与 legacy 共享 `prepare_handler_dispatch`，702 重入静默忽略契约保持）。
  - 防御臂：http_server 6 处 + legacy 派发图 4 处（HTTP 轨非 UI 不产 Runnable）。
- 门禁证据：`cargo check` 零错误零新增告警；plan711 新族 7/7（AC-13 红绿、同步对拍等价、累计护栏、tick 泵隔离、队满/合并/片间消费/公平）；触面 plan702/705/707/708/vm_bridge 115/115；`cargo tv` 162/162。plan707_wait_generator_park_drive_count_static 组合档偶红=并行抖动（单跑恒过、同代码复跑过、708 阶段复审档案同例登记）。
- 过程事故（已恢复，零入提交）：rustfmt 对触碰文件递归格式化模块树，把仓库存量 fmt 漂移刷进 140 文件——保存副本→`git checkout -- .` 恢复净基线→语义改动手工重放，最终 diff 664+/62− 仅 7 文件。**教训：本仓 fmt 漂移为存量，不得整文件 rustfmt。**
- legacy 耗尽行为变更登记：非 UI legacy 同步路径预算耗尽从"静默假成功"变"真错误"——这是 AC-13 缺陷修复本身（契约形状 Result 不变）；触面与语料门禁零回归证明无既有调用图依赖假成功。
- Spec delta 更新：SD-02/04/05 维持拟议（本次改动与拟议一致，无范围漂移）。
- `next: T-03 Init demand/代际生命周期（消费 T-11 结果接口：call_handler_for_cpu_slice + CpuRunnable 凭据 + resume_cpu_slices 泵）`。

### 2026-09-30 work T-03 完成（Init demand 登记簿与代际生命周期）

- `stage: work` | `plan_id: PLAN-711` | `plan_revision: 1` | `outcome: in_progress` | `code_commit: worktree 04b9552c9`。
- T-03 交付面（对 §5 M-01）：
  - **登记簿**：`register_init_demand`——判定与写身份分离（708 baseline §2.2 缺陷面修复）；同代际重复登记（显示/MCP 双 build）只确认簿记不二次入队（AC-04）；身份变化=新代际（`init_generation` 计数器），旧代际一次取消（排队丢弃+在途 `cancel_parked_by_fn` 清栈/凭据映射清理：HttpRequest→`drop_async_result`、HttpStream→`stream_cancel`、Future/CpuRunnable→纯栈释放）；A→B→A 的第二个 A 重跑 Init（测试实证：前缀副作用保留 + 全量重跑，界断言 [20001,40000)）。
  - **派发驱动**：`dispatch_pending_inits`——FIFO（=渲染登记序=页先 child 后的依赖序）；五态观察（Missing 静默 Done/Failed 终态不重试/Completed/Waiting+Runnable=InFlight 并停止本轮后继派发——前序 park 即停，child 等页终态）；InFlight 探测收敛（parked 出册即记账 Done）。
  - **渲染路径**：outlet 页 memo 派发点改"身份变化才 prepare state + 登记"（全量 build 旁路保留——新页内容仍真构建，Init 完成刷新由泵侧 dirty 承担）；`fire_child_init_if_any` 改只登记；渲染零派发（AC-04）；`child_init_should_fire` + `child_last_init_identity` 退役移除（判定即写身份语义消亡）。
  - **临时接线**（T-04 移交）：`poll_parked_resumes` 前置 `dispatch_pending_inits`（D-2 默认预算档）+ init 失败走 syslog `[VM-HANDLER] {widget}.Init failed (dispatch)`；订阅门 `has_parked_tasks() ∪ has_pending_init_work()`。
- 测试过程中发现并修正的三点：①取消契约断言——前缀副作用不回滚（27 前缀+20000 重跑），断言改界检查；②child handler 走单 VM 统一根态——桥级直登记的 child demand 用纯计算 Init 规避（真实 child state id 由渲染路径 prepare_child_render_state 提供）；③50 次迭代循环在 512 步预算边缘也 park——child 用 10 次迭代保确定性。渲染路径完整集成（组件调用→fire_child_init_if_any 登记）不入单测：需完整模块装载管道，该面由 memo/outlet/045/046 渲染家族回归 + T-09 实机矩阵覆盖。
- 门禁证据：cargo check 零错误；plan711 全族 12/12（T-11 7 + T-03 5）；触面 plan702/705/707/708/vm_bridge/plan045/plan046/memo/outlet 233/232+1抖动；tv 162/162。
- 抖动定责（零回归）：`plan707_wait_generator_park_drive_count_static` 负载敏感（id_gen 墙钟窗口计数断言，"容忍到 30"自证）——隔离恒过、机制未被本计划触碰、708 复审档案同例；`stage3_memory_baseline_n1_3_5` 为 "memo" 过滤子串误匹配 "memory" 的并行内存基线抖动，单跑过，非触面。
- Spec delta 更新：SD-02 的 ADR-19/24 重写对象即本次登记簿/派发驱动/取消语义，维持拟议。
- `next: T-04 真实入口帧通知与有界泵（R-1 序障实证首验 → listen_raw 条件订阅 + 帧驱动有界泵接管 dispatch/resume 驱动点）`。

### 2026-09-30 work T-04 完成（帧通知泵 + R-1 闭环 + 门禁改版适配）

- `stage: work` | `plan_id: PLAN-711` | `plan_revision: 1` | `outcome: in_progress` | `code_commit: worktree 3809b58ea+97dcd7d75+d27d0c63b`。
- worktree 同步：rebase 到测试门禁改版后 master（零冲突，4 提交干净重放）。
- T-04 交付面（对 §5 M-03/D-1）：
  - **帧泵接线**：`frame_pump_sub`（listen_raw 收 RedrawRequested，subscription::filter_map id 携带 AppId 防 453-T4 去重丢失；门=has_pending_init_work ∪ has_cpu_continuations，零 demand 无订阅防自我续帧）；`__frame_pump` 消费臂（dispatch_pending_inits + resume_cpu_slices 共享 8ms 轮次预算；完成直置 view_dirty 同帧重建——早退臂不达尾回填，与热重载臂同款；完成 bump 宿主 epoch 失效 outlet memo——plan437/502 pre-Init 骨架回放实录定谳）；tick 泵退回纯 I/O（`has_parked_io_tasks` 门，CpuRunnable-only 注册表不吊 16ms tick）。
  - **订阅重估时序定谳**：iced_winit 订阅重估在 update() 内部、view() 之前（lib.rs :1337/:1300）——渲染期 demand 必须 update 侧唤醒：dispatch_app 以 is_dirty 为燃料链 ready 唤醒（泵推进才置脏，I/O 在途不空转，收敛无自旋）。
  - **R-1 闭环实证**（013-todo + 自建夹具 + AUTO_CPU_PROBE 门控）：sub ON 同周期/112+ 帧消息→泵配对/4ms 校准片推进/598 万步跨 15.2s parked 完成 cpu_cont 归零/10M 累计护栏生产命中/重入忽略。措辞纪律保持 D-1："present 已返回的帧"序障（异步回环），非硬在屏证明。
  - **D-2 实测校准**：CPU_SLICE_MAX_STEPS 4096→1M 天花板（解释器 ~1-3ns/步，4096 步 ≪4ms 帽成实际预算；校准后 4ms 墙钟为活预算）——D-2 授权面，簿记在案。
  - **统一根态约束补全**：InitDemand 携带登记时 props 快照 + 派发前 ensure_child_state 重播种（donut 除零定谳：子件 props=根态同名共享字段，后渲染播种覆盖前者）；`prepare_child_render_state_snap` 变体。
  - **新契约测试迁移 22 例**（10 文件）：构建登记→drive_scheduler_to_quiescence→重建断言；plan633×3/plan498×6/plan499×7/plan492×7/plan502×3/plan536×1/plan437×2/plan643×1/plan484×1/plan632×1。
- 门禁（新口径）：裸 cargo t 4894/4905（63.8s，no-fail-fast 全量）+ tv 162/162 + plan711 13/13。9 红已知预存照录；**2 新红（plan484 bare_names/streaming）= Q-05 阻塞**，未修（见 §10）。
- Q-05 深挖：per-instance 身份（name#instN）已试并回退（d27d0c63b）——两 Init 都跑时末写者霸占共享几何字段（bar_group 回归）；根因层级上调=统一根态共享字段×多实例×延迟派发的状态语义修订，候选 (i) 派发序+末写者恢复 (ii) 每实例 state 对象 (iii) 无 key 重复实例保留同步派发（最小面）。**T-05 设计期裁定**。
- 环境记录：gallery VM 会话一次非确定性自退（~2s，干净 [X9] ok=true，master 二进制对照存活、复测不复现）——T-09 实机矩阵观察项。

### 2026-09-30 work Q-05 解码关闭（plan484 两红归因，零设计修订）

- `stage: work` | `plan_id: PLAN-711` | `plan_revision: 1` | `outcome: in_progress` | `code_commit: worktree f405ff1ef`。
- 方法论：全量 dump 差分（master vs 本分支，66KB/59KB，439 diff 行）→ state 追踪（pre-reseed/post-Init/post-drive/post-build2 四点读数）→ master 对照 worktree 取数。
- 定谳链：bar Init 派发正确（post-Init yTick4="8000"，type=grouped 经快照重播种生效）→ post-drive yTick4="400"（line Init 后派发覆写共享 yTick 为 line 的 0..400 档）→ master 对照：yTick state 与本分支**全等**（同为 line 覆写后的 0..400）而 dump 含 8000——master 的单构建在 bar#1 渲染瞬间（line Init 未跑）把 grouped 刻度烘焙进 View 结构，终态与视图不一致=**旧同步交错伪影**。
- 处置：断言按语义意图迁移（tick 文本渲染即可，非瞬态值）；streaming 测试补构建-驱动-重建；dispatch 改回登记时 id 直用（child_state_map 异构陈旧对象毒化实录：bar 卡渲染出 area 的 fields 谱）；快照补录 4b 守卫跳过的声明默认值（本实例声明语义恢复，构建期写入纪律不动）。
- 门禁：裸 cargo t 4896/4905（64.4s）零新红（仅 9 已知预存）+ tv 162/162 + plan711 13/13。三候选设计修订全不需要，Q-05 关闭。
- `next: T-05 骨架/完成/失败显示（MCP/展示缓存版本链收口）→ T-12 → T-06余 → T-08 → T-09 → T-07'`。

### 2026-09-30 work T-05 完成（骨架/完成/失败显示）

- `stage: work` | `plan_id: PLAN-711` | `plan_revision: 1` | `outcome: in_progress` | `code_commit: worktree fa946bed4`。
- T-05 交付面（对 §5 M-04/Q-01）：
  - **骨架占位**：render_child_widget 双变体在 `fire_child_init_if_any` 后按相位占位——Queued/InFlight → `Loading… (Widget)`；Failed → `⚠ {widget}.Init failed — see [VM-HANDLER] log (reload to retry)`（可诊断、reload 换身份即重试、不无限 Loading）；Done/无 demand 原样全量构建。pending 占位非永久 memo 产物（完成 epoch 失效闭环于 T-04）；嵌套需求依赖阶段发现=完成帧下次构建发现（占位不永久早退）。
  - **终态传播链收口**：完成/失败 → component dirty + AppState.view_dirty + 宿主 epoch（T-04 落地）→ 重建；MCP 快照经既有 gate_dirty 链同相（实机验证：early 快照含 Loading… → late 无残留——屏幕与 MCP 同相贯通）。
  - **实机验证**：auto-os widgets-gallery /line-chart 导航——early MCP 快照捕获 Loading… 骨架、late 无 Loading/无错误占位、页面全量渲染（LineChart 文档页 + 图表区）、应用存活。
- 测试：plan437 补占位可见断言（首建含 Loading… → 驱动后全量几何）；plan499 axispointer/plan536 t1 契约迁移（骨架首建→驱动→重建）；触面全绿。
- `next: T-12 computed/冷构建残面（DataTable memo_block 根因、FileTree 恒 FILL）`。

### 2026-09-30 work T-12 完成（computed/冷构建残面）

- `stage: work` | `plan_id: PLAN-711` | `plan_revision: 1` | `outcome: in_progress` | `code_commit: worktree 93f9274cc`。
- T-12 交付面（对 §5 M-05）：
  - **DataTable memo_block 根因**：页面 `memo () {}` 块（PLAN-046 MemoBlock，包裹静态安装说明）使 outlet 静态扫描整页降级（reason=memo_block，每帧全量重建）。修复=扫描器穿入：scan_node_registry pre-arm + scan_node 穿参（registry+visited），MemoBlock 体经专用 scan_block_node 扫描（嵌套 for/outlet/块仍整块降级、Component 模板 registry 递归展开使模板内状态读入槽）；Conditional 臂同扩（条件 parse_expr_fragment 入槽+双臂递归，解析失败仍降级）。**实机：site=6 FILL→HIT fast→HIT version-fast 零降级**（memo_slots_fp 恒 Some——之前的 None 分支为死路，恒 FILL 另有机制）。
  - **FileTree 恒 FILL 根因**：memo_slots_fp 的 fingerprint_value `?` 使不可展开槽（computed 大列表，706 准入守卫拒绝信号网→每帧重算）失败整个 slots_fp → **条目永不插入** → 恒 FILL。修复=标记回退（unexpandable 常量入指纹）+ 该槽陈旧防护由 dyn_deps 版本检查承担（fill 渲染期已录 dep；deps_unchanged 失配即全量重渲）。**实机：打破恒 FILL**（转换 FILL 一次=Init 完成 epoch 失效，后持续 HIT/version-fast）。残余：computed 值稳定性（706 守卫拒绝列表）属更大设计面，本任务以 memo 层修复达成页面级收益。
  - **双构建乘数**：memo HIT 后第二构建命中缓存，乘数随 HIT 消解（实机 HIT fast 序列佐证）。
- 门禁：裸 cargo t 4896/4905 零新红（仅 9 已知预存）+ memo 族 121 全绿 + plan632 5/5（f1/f4 骨架契约迁移）。
- `next: T-06余 分段时间戳/在屏采集/资源计数 → T-08 测试族收口 → T-09 实机矩阵 → T-07' 设计完备性核对`。

### 2026-09-30 work T-06余 + T-08/F-1 完成（观测面与测试族收口）

- `stage: work` | `plan_id: PLAN-711` | `plan_revision: 1` | `outcome: in_progress` | `code_commit: worktree 35bdb5428`。
- T-06余：资源计数面（frame_pump enter 行：queued_init/cpu_cont/parked_total/write_q）；分段时间戳与在屏采集的其余轴已随前序任务在档（映射见任务行注）——无新增框架，AUTO_MEMO_DIAG/AUTO_SCHED_DIAG 族扩展纪律保持。
- T-08：F-1 落地（非法 outlet 头参 pointed parse-error 直测：错误指明契约+回显非法键）；plan711 测试族收口（14 测）+ 24 例新契约迁移全景（Q-05 伪影断言迁移含）；裸 cargo t 4896/4905 零新红。
- `next: T-09 实机矩阵（§7 全门禁样本）→ T-07' 设计完备性核对 → 终审`。

### 2026-09-30 work T-09 wave-1 + T-07' 完成（实机矩阵第一波 + 设计核对）

- `stage: work` | `plan_id: PLAN-711` | `plan_revision: 1` | `outcome: in_progress` | `code_commit: worktree 5017ce8af`。
- T-09 wave-1：reports/711-runtime.md——长 CPU 夹具 20 样本量化（p50=16.3s/p95=19.3s，帧节奏主导的权衡分析在案）+ 泵占用 4160 轮 p50=p95=max=4ms（≤50ms 门禁决定性 PASS）+ gallery 冷导航 291ms/复访 memo HIT 零降级 + T-12 修复实机取证。wave-2 格（全扫/多 App/全凭据/resize/最小化/冷启动×5）在报告 §5 显式登记，交终审裁定。
- T-07'：设计文档 §10 实施对照表——§3-6 契约（通道语义/取消清理/凭据/帧通知/computed 防护）单执行者先行交付实证；§7 worker 线程边界 proposed 保持（唯一架构跃迁面）；AC-08 满足。
- **计划 9/9 任务全部交付**（T-09 为 wave-1+显式登记形态）——进入 execution_done 前的最后核对：全任务勾选 ✓、AC 映射（AC-04..08/10/11/12余/13 实机+单测证据在案）、SD-02/04/05 拟议待 review 冻结。**下一步=execution_done → /auto-plan:review（独立复审）**。
- `next: /auto-plan:review（独立复审，新门禁口径）→ merge。`
- `next: T-05 骨架/完成/失败显示（Q-05 候选裁定 + 完成语义 MCP/展示缓存版本链收口）`。

## 10. 待澄清事项

| ID | 项目 | 处置 / owner |
|---|---|---|
| Q-01 | R-1 帧序障实证（708 D-6 遗留） | T-04 首验闭环：帧消息时间戳 vs 构建时间戳 vs 截图；矛盾即 needs_replan，不得静默退 tick |
| Q-02 | native/FFI 单调用超预算上界（708 D-5 R-3 移交） | T-11 夹具实测；超限=分块/异步化该热点或 needs_replan，不称检查间隔提供硬上界 |
| Q-03 | T-09 环境噪声（Todesk 虚拟显示/59Hz/冷构建方差） | 继承 708 baseline §5 对策；异常 p95 另采移窗对照并标注适配器 |
| Q-04 | master 并行推进（708 期间三度发生） | 常规化 rebase+range-diff 等价证明；落地前合并态定向刷新触面族 |
| Q-05（**已解码关闭**，f405ff1ef） | plan484 两红根因=旧同步交错**瞬态伪影**：单构建在 bar#1 渲染瞬间把 grouped 刻度（8000）烘焙进 View，而终态 yTick 被 line Init 覆写为 0..400——master 视图与其自身终态不一致；延迟派发模型产出一致终态视图（两 bar 卡读同一终态），"8000 断言"编码的是伪影。处置：断言按语义意图迁移（tick 渲染即可）+streaming 测试补构建-驱动-重建+dispatch 改回登记时 id 直用（child_state_map 异构对象毒化实录）+快照补录 4b 跳过声明默认值。三候选设计修订全不需要；裸 cargo t 4896/4905 零新红 |
