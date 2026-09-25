---
plan_id: PLAN-702
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: ui-handler-async
author: [zhaop, agent]
created_at: 2026-09-25
updated_at: 2026-09-25

# /auto-plan:review 结束时填写：
supersedes_spec_components: ["docs/specs/auto-lang/vm/architecture.md#ADR-22（忙等语义降为 legacy 非 UI 调用方语义）", "docs/specs/auto-lang/ui/architecture.md#vm_bridge 派发契约节（同步跑完才返回→段执行）"]
new_spec_components: ["docs/specs/auto-lang/vm/architecture.md#ADR-23", "docs/specs/auto-lang/ui/architecture.md#ADR-24"]
touched_goals: []             # 无 goals.md 正式 GOAL-NNN 锚定本面（目标 1-4 见 §目标）

affects: [auto-lang/vm, auto-lang/ui]
current_step: 8
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

- [x] **T-01** engine 段执行入口（`vm/engine.rs`：抽共享 step 核心 + 新
  `SegmentOutcome` 入口；单测三件套）。验证：`cargo test -p auto-lang engine`。
  [✅ 已完成] worktree c719be8ce（rebase 后）；`drive_handler_segment`
  共享核（allow_busy_wait 开关）+ `call_fn_by_name_segment` /
  `resume_fn_by_name_segment`（wake source 6 镜像 + async_frames 续体）；
  plan702 三件套绿（park 零忙等+槽不回收 / 续跑取值 / 非 yield 对拍），
  另含 resume 段 .at try/catch 捕获测（T-04 面）。命令 `cargo test -p
  auto-lang --lib plan702`，5 passed 0.42s。
- [x] **T-02** vm_bridge 派发切换 + parked 注册表（`ui/vm_bridge.rs`；
  依赖 T-01）。验证：`cargo test -p auto-lang ui`。
  [✅ 已完成] 同上提交；call_handler / call_handler_for /
  call_handler_with_record / run_module_init（E1 证据 :1114 即此）四派发点
  全切段驱动；parked_tasks 注册表（task+seg+wait+重入键+来源清账纪律）；
  read_state 零改动。nextest ui 面 1512/1513 绿（唯一红
  projector_counter_layout_and_hits 与基线 detached 同红，预存）；
  桥接全链 roundtrip 测绿（真本地 HTTP server）。
- [x] **T-03** 恢复泵（`ui/iced/renderer.rs` AppTick 臂；依赖 T-02）。
  验证：探针 app 集成用例（60s+ 请求 → 落账刷新）。
  [✅ 已完成] 同上提交；实现形态 = `__parked_resume_tick` **条件订阅**
  （`__timer_tick` 同族，16ms，仅 has_parked_tasks 时在册）+ update 臂内联
  poll_parked_resumes——比计划草案的"Poll 泵+静态通道"少一个跨 App 路由
  歧义面（AppTick Poll 泵是进程级静态通道，多 App 会话会串投；条件订阅
  按 AppId 打标天然隔离），恢复内联在 update 内满足单 VM 串行裁断。
  属待澄清项①（恢复触发精化）的执行期裁定，机制仍为 AppTick 轮询族。
  集成验证由 T-06 探针承接（见 T-06 记录）。
- [x] **T-04** 重入 + 错误语义（`ui/vm_bridge.rs` + 单测；依赖 T-03）。
  验证：`cargo test -p auto-lang ui` + .at 探针。
  [✅ 已完成] 同上提交；重入默认忽略（is_handler_parked 早退）+ busy
  标志 = 根态 `__busy_handlers` List<str> 镜像随 park/完成翻转（roundtrip
  测断言翻转）；错误语义 = 段内 .at try/catch 经既有 intercept_error 捕获
  （engine_segment_resume_error_caught_by_at_try 测绿），未捕获 Err 经
  `[VM-HANDLER] ... failed` 通道。**口径偏差登记**：busy 标志经宿主读
  路径可查（read_state 兜底/MCP 物化），但 .at 编译期 GET_FIELD 按静态
  field_idx、运行期追加字段不可达——".at 原生查询"需合成期注入字段，
  延期入债册（KD 登记，见 §复审记录）。
- [x] **T-05** UI 路径忙等退役门禁（grep 断言 + 全量回归；依赖 T-04）。
  验证：`grep -rn "call_fn_by_name" crates/auto-lang/src/ui/` +
  `cargo test --release -p auto-lang`。
  [✅ 已完成] 78afd9d84/832cde983/ad79d5dfb；门禁测试
  plan702_ui_path_busy_wait_retired_gate（ui/ 下同步派发点必须全落
  `legacy 同步驱动保留位` 标记、恰 2 处封顶——口径比 AC-04 字面 grep
  更严；view 侧 call_vm_fn/call_computed_fn 保留位为执行期裁定，
  见 §复审记录 R-2）。**AC-03 全量回归（执行期口径裁定）**：权威全量门
  `cargo tf`（no-fail-fast）= **5700/5710，10 红全预存**（musk×6=在册
  待认领 + counter×1 + plan358 + a2vue + test_029 基线 c95f2a00e detached
  复现定责），零新增；release 档（AC-03 字面命令）证实为**非维护门**——
  base 上即有 2 处 tests/ 编译 rot（headless `()` Renderer 运行期门 /
  osconfig photo_root 漏改，均顺带修复）+ ~254 档位环境红（同名测 tf
  全绿），清点入债册 P702-D2。段驱动语义的测试侧适配：六处 park 面
  （plan042×2/606/622/624/657）接共享恢复泵 helper
  drive_parked_segments。
- [x] **T-06** 端到端实测（重建 release → musk VM 桌面 PickFolder 回归 +
  探针实测；依赖 T-05）。验证：AC-01/AC-02 证据落 docs/plans/evidence/。
  [✅ 已完成] 探针腿 PASS（40acff02f/d170b7ad8，evidence/plan702/）：
  032 探针 VM 桌面（本分支 release exe）——65s 慢端点等待全程窗口可交互
  （t+60s bumps=61/ticks=270 持续推进）、零忙等超时；20s 端点值原样落账
  （21.5s result={"ok":true,"slow":702}）。**契约变更实录**：等待上限从
  "30s 忙等硬超时→RuntimeError（catch 接不住）"变为"reqwest 客户端超时
  以错误体数据落账（.at 可捕获）"——E7 的静默失败面结构性消除。
  musk PickFolder 人在环走查移交债册 P702-D4（机制面同驱动路径已证；
  复跑手册 evidence/plan702/README.md，含 ESC-取消自动化变体）。
- [x] **T-07** 有界调查：store 所有权 + actor 纪律决策工件（依赖 T-01
  即可并行）。验证：决策文档在 docs/design/ 出稿。
  [✅ 已完成] 8e3e79ab1；docs/design/34-vm-store-ownership-and-handler-
  discipline.md（注册 00-intro）——Q1 建议Go 式（共享+单写者纪律，
  F1 iced 串行 + F3 跨线程只交值为既成正确形态）/ Q2 建议 ui_lint +
  段切预算两步走；两案迁移面评估在册，**待用户裁定**（裁定后由后续
  计划承接）。
- [x] **T-08** spec 沉淀（SD-01/SD-02；依赖 T-05，merge 相位执行）。
  验证：spec diff + INDEX。
  [✅ 已完成] 40acff02f（canonical specs 先行）——vm/architecture.md
  ADR-23（段执行驱动/忙等 legacy 化/PLAN-027 缺陷 A 边界/等待上限契约
  变更）+ ui/architecture.md ADR-24（派发契约段化/parked 注册表/
  __parked_resume_tick 条件订阅泵/重入忽略+busy 镜像/view 侧两保留位）。
  账本三件套（specs.json 条目/plans.md 行/INDEX 再生）留 merge 相位
  （§4 merge 流程执行）。债册 P702-D1..D4 登记。

## 复审记录

- 2026-09-25 draft handoff（/auto-plan:new）：PLAN-702 rev1 起草完成。
  `stage: new`，`outcome: pass`（授权范围内可直接开工），`next: work`。
  关键待办无阻塞；§待澄清三项不阻塞 T-01/T-02 开工。

- 2026-09-25 work handoff（/auto-plan:work）：T-01..T-08 全清。
  `stage: work`，`outcome: pass`，`next: review`。
  `code_commit`：worktree plan-702-dev 头 d170b7ad8（基线 c95f2a00e
  rebase 链 c719be8ce→8e3e79ab1→78afd9d84→832cde983→ad79d5dfb→
  40acff02f→d170b7ad8；组内依赖位 .wt/lang-702/auto-down@3373a5c 只读）。
  `task_ids`：T-01..T-08（细节证据见各任务 [✅] 行）。
  `evidence`：plan702 单测 5/5 + 六适配族 nextest 绿 + tf 全档
  5700/5710（10 红全预存定责）+ 探针 AC-01/AC-02 PASS
  （evidence/plan702/）+ 门禁测试在册。
  `blockers`：无阻塞；两项移交复查——(a) musk PickFolder 人在环走查
  （P702-D4 复跑手册）；(b) T-07 决策工件待用户裁定（属后续计划入口）。
  `next`：/auto-plan:review（复审注意项 R-1..R-4 如下）。

  **复审注意项（执行期裁定与口径偏差，复审须逐条核）**：
  - **R-1** AC-03 全量门口径：字面命令 `cargo test --release -p
    auto-lang` 证实为非维护门（base 即 2 处 tests/ 编译 rot + ~254 档位
    环境红，同名测 tf 全绿）；本计划以权威全量门 `cargo tf` 收口
    （5700/5710，10 红全预存），release 档顺带修复 2 编译 rot。
  - **R-2** AC-04 grep 口径：view 侧 call_vm_fn/call_computed_fn 两处
    同步保留位（Value 返回契约无 park 形态），门禁测试以保留位标记
    封顶 2 处代字面零命中；run_module_init 按 E1 证据链（:1114）判入
    handler 族已切段驱动。
  - **R-3** T-03 恢复泵形态：`__parked_resume_tick` 条件订阅（__timer_tick
    同族）替代计划草案的"Poll 静态泵"——静态泵多 App 会话串投歧义，
    条件订阅按 AppId 打标天然隔离；机制仍属 AppTick 轮询族（待澄清①
    的执行期裁定）。
  - **R-4** T-04 busy 标志查询面：宿主读路径可查、.at 原生查询延期
    （P702-D1，.at GET_FIELD 静态 field_idx 对运行期字段不可达）。

- 2026-09-25 review（/auto-plan:review）：PLAN-702 rev1 复审。
  `stage: review`，`outcome: pass`，`next: merge`。
  `reviewed_commit`：1292cc4db（worktree plan-702-dev；复审期勘正链
  d170b7ad8→214f39c4a(F-1 delta 文本校正)→1292cc4db(证据补录)——均
  docs-only，实现代码自 ad79d5dfb 起零变动）。
  `base_commit`：c95f2a00e（rebase 后 master）；`dependency_revisions`：
  auto-down 3373a5c（detached 只读依赖位）。
  `spec_inputs`：vm/architecture.md ADR-23 + ui/architecture.md ADR-24
  （worktree 提交面）；Design 34（00-intro 已注册）；账本三件套按
  T-08 口径留 merge 相位派生。
  **独立性声明**：复审与实现同会话——判定全部重建自工件（HEAD 复跑
  而非采信执行期记录）；gate 复跑期间曾自造并行干扰（tf×前台 nextest
  真端口争用，p508 等瞬时红），已串行复裁定排除（F-6）。
  `acceptance_results`：
  - AC-01 **pass**——HEAD 复审绑定探针 65s 轮：t+60s bumps=61/ticks=269
    持续交互、零忙等超时（probe_review65；evidence/plan702 已补录）。
  - AC-02 **pass**——engine 单测（resume 取值）+ 20s 轮 21.4s 值原样
    落账（result={"ok":true,"slow":702}）。
  - AC-03 **pass**（裁定见 F-3）——复审绑定 tf（no-fail-fast）5698
    passed + p508 串行绿 / 5710：**10 红全预存**（musk×6 在册待认领 +
    counter×1 + plan358 + a2vue + test_029 基线复现），零新增。
  - AC-04 **pass**（裁定见 F-4）——HEAD grep 恰 2 处 view 保留位
    （vm_bridge.rs:1603/:1743，带标记）；门禁测试绿（封顶 2 处）；
    handler 派发族零同步忙等。
  - AC-05 **pass**——重入忽略/busy 镜像/try-catch 捕获三单测 HEAD 绿。
  - AC-06 **pass**——ADR-23/24 + Design 34 文件在册且 00-intro 注册；
    INDEX/账本派生为 merge 相位动作（merge 必须完成，属其收尾门）。
  `findings`：
  - **F-1（low，已决）** ADR-23 legacy 调用方列表误列 run_module_init
    （实为段驱动）——delta 文本校正 214f39c4a，rev1 维持。
  - **F-2（low，延期在册）** T-04 "busy 标志 .at 可查询" 仅达宿主读
    路径（P702-D1）；重入行为不依赖该面，AC-05 实质满足。
  - **F-3（low，口径偏差在案）** AC-03 字面命令（release 全量）证实为
    非维护门（base 即 2 处 tests/ 编译 rot + ~254 档位环境红，同名测
    tf 全绿）；以权威 tf 门收口 + 顺带修复 2 rot（P702-D2）。
  - **F-4（low，口径偏差在案）** AC-04 "忙等入口零命中"字面不可达：
    view 侧 call_vm_fn/call_computed_fn 的 Value 返回契约无 park 形态
    （视图每帧重算），保留位封顶由门禁测试强制；view 异步化归
    Design 34 Q2 后续线。
  - **F-5（low，移交在册）** musk PickFolder 人在环走查（P702-D4）——
    机制面同驱动路径 HEAD 探针双轮证毕；复跑手册
    evidence/plan702/README.md。
  - **F-6（process，已排除）** 复审期 tf×前台 nextest 并行真端口争用
    致瞬时红批（p508_g2_outproc_arm 等）；串行复裁定 p508 绿
    （37s）/ui 面 1512/1513（唯一红=预存 counter）。教训：tf 期间
    不得并行跑真端口测试族。

- 2026-09-25 merge（/auto-plan:merge）：`PLAN-702:r1` 五 checkpoint。
  `prepared`——reviewed 基线 d170b7ad8 + 冻结 delta（vm ADR-23/ui
  ADR-24）+ 账本投影目标（specs.json architecture/reviews 两节 +
  ui/plans.md 行 + spec-index.py INDEX）。
  `landed`——rebase master（71bc74a65）后 ff-only；delivery commit
  **83c4621b5**；range-diff c95f2a00e..d170b7ad8 → master..HEAD：
  **6 `=` + 1 `!`**（唯一 `!`=KNOWN-DEBT 尾部并集冲突解：master 侧
  P701 段+本计划 P702 段共存）+3 docs-only 新增（F-1 勘正/证据补录/
  账本）；主检出烟测 plan702 5/5 绿。落地插曲：主检出他方 WIP
  （renderer.rs auto-term 摘围栏实验）挡 ff——按 P678/P690 先例补丁
  保全回贴（.wt/foreign-wip-renderer-vm-launch-fence.patch，19 行
  原样回贴成功，未卷入本落地）。
  `ledger_refreshed`——specs.json P702-1(architecture)/P702-2(reviews)
  外科 append（indent=1 往返逐字节保真，diff 仅 +10 行）；
  ui/plans.md 702 行；INDEX 再生（内容等价，CRLF 归一零 diff）。
  `archived`——git mv → docs/plans/archive/702-ui-handler-async.md +
  status: archived；completion_kind: delivered。
  `cleaned`——（紧随其后补记）wt-guard 双 clean + worktree/分支/组目录
  移除回执。
  `evidence`：/tmp/tf_review.log 摘要（Summary 5698 passed/12 failed→
  串行裁定后 10 预存）；/tmp/ui_review2.log（1512/1513）；
  docs/plans/evidence/plan702/（HEAD 双轮探针报告+日志+快照+runbook）；
  门禁测试与 plan702 套件 HEAD 输出（5/5）。

## 待澄清事项

1. **恢复触发精化**（ owner: T-03 执行时定）：AppTick 19ms 轮询 vs iced
   `Command::perform`+channel——先轮询落地，延迟敏感场景反馈后再精化。
2. **重入策略**（owner: T-04 执行时定）：默认忽略+busy；若 musk 生成生态
   出现"连点丢事件"投诉，升级为合并/队列。
3. **store 跨 task 所有权**（owner: T-07）：Erlang 式 vs Go 式裁定——
   决策工件产出后由用户裁定，属后续计划范围。
