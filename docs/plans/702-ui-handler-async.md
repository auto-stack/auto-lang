---
plan_id: PLAN-702
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: ui-handler-async
author: [zhaop, agent]
created_at: 2026-09-25
updated_at: 2026-09-25

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm, auto-lang/ui]
current_step: 0
total_steps: 8
---

# [PLAN-702] ui-handler-async

## 变更摘要

VM/iced UI 的 handler 派发当前走 `engine.rs::call_fn_by_name` 同步驱动器：
handler 内一旦发起异步 HTTP（`#[api]` 调用 Yield），驱动器以 5ms 轮询**忙等**
（30s deadline），期间整个 iced 事件循环冻结、窗口死画。2026-09-24 实锤事故：
musk 桌面版 workspace 选择器"打开文件夹"→ 后端 rfd 模态对话框（人在环，
时长不可界）→ 30s 超时崩溃 + 静默失败（`vm` 日志 `[VM-HANDLER]
WorkspaceSelector.PickFolder failed: RuntimeError("async http request timed
out in call_fn_by_name")`）。

本计划把 UI handler 驱动切换为**可挂起/可恢复的任务段执行**（仓内
`scheduler.rs::execute_handler_with_vm` 已有同语义驱动器先例）：handler 遇
Yield 即交还控制权、iced 立刻重渲染；结果就绪后经恢复泵续跑 handler、model
落账触发刷新。同步忙等从 **UI 驱动路径**退役（engine 旧 API 为非 UI 调用方
保留）。直接风格 .at 语法不变；从不 yield 的 handler 行为零变化。
CPU-bound handler 抢占与严格 actor 纪律（store 所有权裁定）不在本计划实现，
以有界调查产出决策工件（T-07），为后续计划铺路。

## 目标

1. **UI 永不因 handler 等待而冻结**：handler 内任意时长的异步等待
   （含人在环的模态端点，如 rfd 对话框）期间，窗口保持可交互、可重绘。
2. **结果正确回填**：parked handler 在结果到达后恢复执行，model 落账、
   view 刷新；错误可被 .at `try/catch` 捕获。
3. **兼容零回归**：从不 yield 的 handler 在新旧驱动器下行为逐字节相同
   （现有全量测试绿即为回归证明）；scheduler actor 路径与 PLAN-083 消息桥
   不受影响。
4. **沉淀契约**：vm/ui 两册 module spec 增补 handler 驱动契约；产出
   store 所有权与 actor 纪律的决策工件。

**非目标**（本计划不做，防蔓延）：
- CPU-bound handler 的抢占式时间片（BudgetExhausted 段切）——后续计划；
- 严格 actor 化（handler 禁阻塞调用的类型/lint 强制、store 归属裁定落地）
  ——本计划只做决策工件（T-07）；
- engine `call_fn_by_name` 忙等段的整体删除（back_proxy/http_server 等非 UI
  调用方仍在使用且各自线程内阻塞可接受）——本计划只退役 **UI 驱动路径**；
- ag（TS）轨任何改动（其原生 async 语义本就正确）。

## 架构方案

改造分层与数据流（目标态）：

```
iced 事件循环（UI 线程）
  │ window event / 恢复泵 Message
  ▼
iced update ──► vm_bridge dispatch（改造点①）
  │               │  首选：段执行驱动（改造点② engine 新入口）
  │               │  FrameResult::Return          → 本 update 内跑完（现状不变）
  │               │  FrameResult::Yielded+Waiting → 登记 parked task，**立即返回**
  ▼               ▼
view 渲染（model 旧值/中途值）        parked task 注册表（改造点① vm_bridge）
                                       │
async worker 线程（spawn_async_http_handle / future）
  │ ASYNC_RESULTS[req_id] 就绪
  ▼
恢复泵（改造点③ ui/iced/renderer.rs AppTick 订阅，同 http_msg_subscription 族）
  │ 取一条就绪 parked task → dispatch 续跑段 → 落账/再 park
  ▼
model 变更 → iced 重渲染
```

关键设计裁断：

1. **单 VM 串行一致性**：所有 VM 执行仍经 iced update 串行化——一次 update
   最多跑一个"连续段"（某 parked task 的续跑或一个新 handler 的首段）。
   parked 中途 model 的中途态对 view 可见（与现状 handler 顺序写 model 的
   可见性同级），不引入锁或快照；重入互斥保证同一 handler 不会并发两段。
2. **恢复触发走现有 AppTick 轮询**（renderer.rs 已有 AppTickKind::Poll 泵，
   19ms），备选 iced `Command::perform` + channel（更精准、改动更大）——
   取最小差分，先轮询后优化。
3. **engine 增量而不是替换**：新增段执行入口（返回
   `Completed / Parked`），`call_fn_by_name` 原样保留给 back_proxy、
   http_server 等非 UI 调用方；忙等段在本计划结束时对 **vm_bridge 路径**
   不可达（grep 断言）。
4. **兼容性论证**：parked 驱动对不 yield 的 handler 与同步驱动行为相同
   （都在本次 update 内跑完）——迁移爆炸半径 = 今天已坏的路径（会 yield 的
   handler），这正是缺陷面本身。

## 需求分析与背景调查

### 授权记录

用户 2026-09-25 会话明确指令："把问题描述清楚，然后建立一个 auto-lang 的
平台计划"。范围 = 本仓（auto-lang）；预算/自动续跑限制未设定（无）。计划
簿记落 master 主检出（AGENTS.md L1 流程 + `scripts/new-plan.sh` 取号 702）。

### 证据链（2026-09-24/25 实测与源码勘察）

| # | 事实 | 出处 |
|---|---|---|
| E1 | UI handler 派发契约 = 同步跑完 | `crates/auto-lang/src/ui/vm_bridge.rs:18`（模块注释 "Dispatches handlers via call_fn_by_name"）、`:1114` 调用点 |
| E2 | 忙等 + 30s deadline | `crates/auto-lang/src/vm/engine.rs:2192-2213`：Yield + `waiting_http_request_id` → 5ms sleep 轮询 `async_http_result_ready`，超时 `RuntimeError("async http request timed out in call_fn_by_name")`（`:2207`）并回收结果槽（PLAN-027 缺陷 A 注） |
| E3 | 忙等占 UI 线程、实测冻结 | engine.rs 同段位于 iced update 内；PLAN-083 T-01 注（stdlib.rs ~7420）："ui 侧 call_fn_by_name 对 Waiting 任务是忙等——UI 线程整个 handler 期间被占（切 auto-edit 冻结 20.9s 即此）" |
| E4 | 可挂起驱动器先例已存在 | `crates/auto-lang/src/vm/scheduler.rs:249-262` `execute_handler_with_vm`：`FrameResult::Yielded` + `TaskStatus::Waiting` → **返回状态而非阻塞** |
| E5 | mid-await 挂起/恢复机制已存在 | `engine.rs:10444` `handle_await_future`："Phase B: body suspended mid-await — keep synthetic closure so resume can still LOAD_CAPTURED"；`task.rs` `TaskStatus::Waiting(String)` |
| E6 | worker→UI 消息泵先例已存在 | `crates/auto-lang/src/ui/iced/renderer.rs`（AppTickKind::Poll / http_msg_subscription，PLAN-083 C2 形态：发起段即返回、回填段事件泵） |
| E7 | 事故实例 | 2026-09-24 musk 桌面版 PickFolder：rfd 对话框（`auto-musk backend/crates/musk/src/workspace.rs:640` spawn_blocking 代弹，时长=人的操作）>30s → E2 超时 → .at 层 catch 接不住 VM 级 RuntimeError → 契约"取消/失败静默不动"→ 无提示失败 |
| E8 | web 轨无此缺陷 | 同一份 .at 编译为 TS 后 `Http.post` 是真 async fetch，handler 先返回、续体稍后执行，UI 线程不冻结——本计划使 VM 轨语义向 web 轨收敛 |

### 相关既有机制（只引用，不复制）

- `scheduler.rs::task_loop`（actor 任务：mailbox + pattern handler）与
  `TaskStatus::Waiting`（sleep/SSE/message_loop 均在用）——本计划不把 widget
  handler 改造成 actor task（差分过大），只借用其"段执行 + 状态返回"驱动语义。
- PLAN-083 `Http.get_msg/post_msg` 消息桥——保留可用，作为手工两段式逃生舱；
  本计划落地后两者长期并存，收敛为后续决策。

## 详细设计

### T-01 engine 段执行入口

新增 `call_fn_by_name_segment(task, fn_name, args) -> SegmentOutcome`（命名
可定夺）：

- `SegmentOutcome::Completed(result)`：handler 跑到 RET（等价现行为）；
- `SegmentOutcome::Parked { wait }`：遇 Yield 且 `waiting_http_request_id`
  或 `waiting_future_id` 就位时，**不进入忙等循环**，直接返回（task 持有
  ip/栈/闭包上下文，天然可续）。

实现：抽取 `call_fn_by_name` 现有 step 循环为共享核心，忙等分支仅在旧入口
编译进来（`#[cfg]`/参数开关均可，review 定夺）；`drop_async_result` 回收
逻辑（PLAN-027 缺陷 A）保持只有"真正放弃等待"的路径触发——parked 路径
**不回收**，恢复时按 req_id 取用。

### T-02 vm_bridge 派发切换 + parked 注册表

- `call_handler`（vm_bridge.rs:1114 一族）改走 T-01 段入口；
- VM 级 `parked_tasks: Vec<ParkedTask>`（登记 task、来源 widget/handler、
  wait 凭据、重入键）；单 VM 串行（见架构裁断 1）；
- `read_state()`（view 取数）路径零改动。

### T-03 恢复泵

`ui/iced/renderer.rs` AppTick 轮询臂（与 http_msg_subscription 同族）增加：
扫描 `parked_tasks` 中 wait 已就绪者 → 以 iced Message 投递"resume 事件" →
下一次 update 内续跑该 task 段 → Completed 则落账出清，仍 Waiting 则继续
park。恢复执行的错误经既有 `[VM-HANDLER] ... failed` 通道上日志。

### T-04 重入与错误语义

- 重入默认策略：同（widget, handler）键已有 parked 段时，后续触发**忽略**
  并置 model 可见 busy 标志（.at 可查询；命名随实现定）——队列/合并策略
  留待使用反馈；
- 错误传播：resume 后段内 Err 走既有 handler 失败路径，且验证 .at
  `try/catch` 能捕获（E7 中"catch 接不住 VM 级超时"的同族问题在 parked
  路径不得复现）。

### T-05 UI 路径忙等退役（门禁任务）

vm_bridge 全部派发点迁移后：grep 断言 `ui/` 下无 `call_fn_by_name` 同步
派发调用；engine 忙等段注释标记 `legacy: 非 UI 调用方专用`。不删代码。

### T-06 端到端实测（原始事故回归）

auto-lang 重建 release → musk 主检出消费（canvas/spawn 链走
`AUTO_EXE`/兄弟位解析）→ VM 桌面实测：workspace 选择器"打开文件夹"，对话框
滞留 >60s 后选择——workspace 正常注册并切换、全程窗口可交互；辅以探针 .at
app（handler 内 60s+ post_json）断言 AC-01。

### T-07 有界调查（决策工件，无代码）

store 所有权 + actor 纪律裁定：Erlang 式（store 归 UI task、worker 消息
交账）vs Go 式（共享+同步）；handler 禁阻塞调用的类型化 vs ui_lint 化。
产出决策文档（docs/design/ 下新编号），含两案迁移面评估。时间盒：1 个
worktree 会话内出稿。

### T-08 spec 沉淀

按下方规范增量更新两册 module spec + INDEX（merge 相位执行，canonical
specs 先行）。

### 规范增量

| delta_id | 类型 | 目标 | before/after 规则 | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/architecture.md | before：UI 侧 call_fn_by_name 对 Waiting 任务忙等（5ms 轮询、30s deadline）；after：UI 驱动路径使用段执行驱动，Yield+Waiting 即交还控制权（Parked），恢复泵续跑；忙等仅为 legacy 非 UI 调用方语义 | UI 线程不可被 handler 等待占用（E3/E7 实证） | AC-01/04 |
| SD-02 | modify | docs/specs/auto-lang/ui/architecture.md | before：handler 派发契约=同步跑完才返回（vm_bridge 注释自述）；after：派发契约=交还任务段即返回；parked 注册表+AppTick 恢复泵；同 handler 重入默认忽略+busy 可见；错误经 handler 失败通道且 .at 可捕获 | AutoUI 桥接层契约变更的事实沉淀（本计划主改造面） | AC-01/02/05 |

## 测试设计

- **单元（T-01）**：engine 新增测试——(a) handler 内 post_json：首调用返回
  Parked 且结果槽未被回收；(b) 注入 ASYNC_RESULTS 后续跑返回值正确；
  (c) 从不 yield 的 handler 走段入口结果与旧入口一致（对拍）。
  命令：`cargo test -p auto-lang engine`。
- **单元（T-04）**：重入忽略 + busy 标志；resume Err 经 handler 失败路径且
  .at try/catch 捕获（.at 全链模拟器探针，PLAN-083 U-3 同法）。
- **集成（T-02/T-03）**：探针 app——handler 发起 60s+ 请求期间窗口事件持续
  响应（X9/日志断言事件流不断流），结果到达后 model 落账、view 刷新。
- **回归（T-05 前置）**：`cargo test --release -p auto-lang` 全量绿 +
  musk 侧 parity（vm 轨冒烟脚本族）。
- **端到端（T-06）**：musk VM 桌面 PickFolder 实测（E7 原始事故回归），
  留证据：vm/serve 日志 + 截图。

## 验收标准

- **AC-01** UI handler 内异步等待 ≥60s 期间，窗口可交互（事件响应、重绘不
  停），无忙等轮询发生。验证：T-06 实测 + 探针日志断言（预期：等待期间
  AppTick/渲染事件持续，`[VM-HANDLER] ... timed out` 零条）。
- **AC-02** 结果到达后 handler 正确恢复：model 值正确、view 刷新、且执行
  顺序（挂起点后续）与直接风格语义一致。验证：T-01(b)/T-03 集成测试
  （预期：返回值与同步对拍一致）。
- **AC-03** 零回归：现有全量测试绿。验证：`cargo test --release -p
  auto-lang`（预期：与 PLAN-702 取号时 master 基线同绿）。
- **AC-04** UI 驱动路径忙等退役。验证：`grep -rn "call_fn_by_name"
  crates/auto-lang/src/ui/` 仅命中段执行入口（预期：忙等入口零命中）。
- **AC-05** 重入与错误语义生效。验证：T-04 单测（预期：parked 期间重复
  触发不叠执行；Err 可捕获）。
- **AC-06** 沉淀完成：SD-01/SD-02 落册 + T-07 决策工件产出。验证：文件
  存在且 INDEX 引用（预期：两处 spec diff + 一份决策文档）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] **T-01** engine 段执行入口（`vm/engine.rs`：抽共享 step 核心 + 新
  `SegmentOutcome` 入口；单测三件套）。验证：`cargo test -p auto-lang engine`。
- [ ] **T-02** vm_bridge 派发切换 + parked 注册表（`ui/vm_bridge.rs`；
  依赖 T-01）。验证：`cargo test -p auto-lang ui`。
- [ ] **T-03** 恢复泵（`ui/iced/renderer.rs` AppTick 臂；依赖 T-02）。
  验证：探针 app 集成用例（60s+ 请求 → 落账刷新）。
- [ ] **T-04** 重入 + 错误语义（`ui/vm_bridge.rs` + 单测；依赖 T-03）。
  验证：`cargo test -p auto-lang ui` + .at 探针。
- [ ] **T-05** UI 路径忙等退役门禁（grep 断言 + 全量回归；依赖 T-04）。
  验证：`grep -rn "call_fn_by_name" crates/auto-lang/src/ui/` +
  `cargo test --release -p auto-lang`。
- [ ] **T-06** 端到端实测（重建 release → musk VM 桌面 PickFolder 回归 +
  探针实测；依赖 T-05）。验证：AC-01/AC-02 证据落 docs/plans/evidence/。
- [ ] **T-07** 有界调查：store 所有权 + actor 纪律决策工件（依赖 T-01
  即可并行）。验证：决策文档在 docs/design/ 出稿。
- [ ] **T-08** spec 沉淀（SD-01/SD-02；依赖 T-05，merge 相位执行）。
  验证：spec diff + INDEX。

## 复审记录

- 2026-09-25 draft handoff（/auto-plan:new）：PLAN-702 rev1 起草完成。
  `stage: new`，`outcome: pass`（授权范围内可直接开工），`next: work`。
  关键待办无阻塞；§待澄清三项不阻塞 T-01/T-02 开工。

## 待澄清事项

1. **恢复触发精化**（ owner: T-03 执行时定）：AppTick 19ms 轮询 vs iced
   `Command::perform`+channel——先轮询落地，延迟敏感场景反馈后再精化。
2. **重入策略**（owner: T-04 执行时定）：默认忽略+busy；若 musk 生成生态
   出现"连点丢事件"投诉，升级为合并/队列。
3. **store 跨 task 所有权**（owner: T-07）：Erlang 式 vs Go 式裁定——
   决策工件产出后由用户裁定，属后续计划范围。
