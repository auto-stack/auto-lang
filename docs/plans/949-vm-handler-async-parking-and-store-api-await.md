---
plan_id: PLAN-949
status: drafting
feature_name: VM handler 异步挂起扩展（store/方法内 api 调用）
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-06
plan_revision: 1
current_step: 0
total_steps: 4
base_branch: master
depends_on: []
affects: [auto-lang/vm-engine, auto-ui-runtime/vm-bridge, auto-os/PLAN-054]
---

# PLAN-949：VM handler 异步挂起扩展（store/方法内 api 调用）

## 0. 变更摘要

iced 桌面（auto-os/PLAN-054 T-04）的 VM 应用在真实用户任务上失败：todo 的
`StoreInput("x") + AddTodo` 能落库（HTTP 请求确实发出）但 UI 不清输入、不刷
列表。日志显示 **只有 boot `Init` 会 `[VM-PARKED] parked → resumed`**，后续
handler 里的 api 调用（`store.AddTodo()` → `create_todo(...)`）都**同步返回
默认值**（`MutationResult::default()`，ok=false），HTTP 响应只会旁路写库。

本计划让「任意 handler（根组件 handler、store/伴生方法、嵌套调用）里的异步
api 调用」走同一套挂起-续跑（park/resume）驱动，使注入/事件驱动的用户任务
与 boot Init 行为一致。

## 1. 目标

- G1：非 Init 的 handler 在遇到异步 api（HTTP/future/stream）时同样
  `SegmentOutcome::Parked` 并入注册表；`resume_ready_parked` 就绪后续跑，
  完成后的状态变更驱动 UI 重渲染。
- G2：store/伴生方法内的 api 调用不被降级为同步默认值（与 root handler 同
  语义）。
- G3：零回归——boot Init 挂起、既有 `call_handler_for`/`call_handler_for_cpu_slice`
  语义与测试不变。

## 2. 架构方案

auto-lang VM 引擎侧（本仓）+ auto-ui 桥（消费方，不改语义）：

- 定位「handler → store 方法」的编译/派发链：store 方法是否被编译为独立
  段（segment）还是内联调用；内联时 api await 是否被降级为同步调用。
- 若 store 方法是独立段：让 `vm_bridge` 的 store 派发复用
  `call_fn_by_name_segment` 的挂起路径（当前疑似同步入口）。
- 若 store 方法被内联：把 api 调用统一编译为可挂起的段边界，或让内联路径
  在遇到异步 api 时冒泡 `Parked` 到外层 handler 段。
- 续跑后置 `view_dirty`（auto-ui 侧已有 `resume_ready_parked` 订阅位）。

## 3. 技术栈

Rust（auto-lang vm engine / trans）、auto-ui-runtime vm_bridge、iced 桌面实测。

## 4. 需求分析与背景调查

证据（2026-11-06，auto-os/PLAN-054 T-04）：

| 观察 | 内容 |
|---|---|
| 日志 | `[VM-PARKED] handler_App_Init parked (event Init, wait HttpRequest(1))` → `resumed to completion (3.3s)` |
| 非 Init | 注入 `AddTodo` 仅 `[CALL_HANDLER] … handler_App_AddTodo args=[]`，无 parked |
| 数据 | `todos.db` 新增 `('epsilon',0)`（HTTP 请求确实完成） |
| UI | 输入框仍显示、列表不刷新、无失败横幅 |

推断：`store.AddTodo()` 内的 `create_todo` 走了同步默认值分支（VM 未挂起），
HTTP 响应仅写库；因此 `r.ok=false`，`.input` 不清空、列表不重载。

## 5. 详细设计

- T-00 定位：`call_handler_for`（vm_bridge.rs:3741）已支持 Parked；差异在
  store 方法派发路径。读 `prepare_handler_dispatch`、store 编译产物与
  `call_fn_by_name_segment` 的 await 指令判定，确认为何 store 内 api 不 yield。
- T-01 实现：按定位结论选择「store 段化」或「内联冒泡 Parked」；补单测
  （store 方法内 api 调用 → Parked；resume 后状态变更可见）。
- T-02 验证：iced 桌面 + todo 后端（8080）实测 `StoreInput + AddTodo` →
  列表出现新任务、输入清空；`[VM-PARKED]` 出现在 AddTodo 段。
- T-03 回归：boot Init 挂起/续跑、`has_parked_io_tasks` 门、cpu-slice 入口
  既有测试。

## 6. 测试设计

- 单测：store 方法内 api 调用返回 Parked；resume 完成后 `view_dirty`。
- 端到端：ico 桌面 todo 用户任务（AC-08 型）。
- 回归：VM bridge 既有 parked 测试组。

## 7. 验收标准

- AC-01：非 Init handler 的异步 api 调用产生 `[VM-PARKED]` 并成功 resume。
- AC-02：todo 在 iced 桌面 `StoreInput + AddTodo` 后列表出现该条、输入清空。
- AC-03：boot Init 与 cpu-slice 语义零回归。

## 8. 执行步骤

- [ ] T-00 定位 store/内联 api await 降级点。
- [ ] T-01 实现挂起扩展 + 单测。
- [ ] T-02 iced 桌面 todo 实测。
- [ ] T-03 回归与回执。

## 9. 复审记录

- stage: new | plan_id: PLAN-949 | plan_revision: 1 | outcome: pass（用户 2026-11-06 授权：继续补 VM 异步事件循环） | next: work/T-00

## 10. 待澄清事项

- 若 store 方法内 api 调用在设计上刻意同步（如 boot 只有 Init 允许异步），
  需在 T-00 给出证据并改为「事件驱动消息走段派发」的替代方案。
