---
plan_id: PLAN-623
status: reviewed                # drafting → executing → execution_done → reviewed → archived
feature_name: autoui-mcp-test-fixture
author: [codex]
created_at: 2026-09-14
updated_at: 2026-09-14
plan_revision: 2

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [auto-lang/mcp/autoui-test-fixture]
touched_goals: []             # 无 GOAL-NNN 变更：本计划只增加测试夹具与验收基础设施，不改变产品目标。

affects: [auto-lang/ui, auto-lang/mcp]
current_step: 7
total_steps: 7
---

# [PLAN-623] autoui-mcp-test-fixture

## 0. 变更摘要

为 AutoUI MCP 增加**测试专用的运行时状态夹具接口**，让 VM 轨可以从 MCP
注入确定性的棋盘/状态，再经真实 VM handler 或指定事件完成规则验证。接口
属于 auto-lang 框架能力，不在任何游戏里增加调试按钮、隐藏菜单或生产 UI。

第一版面向 VM 轨，必须同时覆盖：

- merged：VM handler 与宿主在同一进程；
- no-merge：VM UI 通过 HTTP 调用后端；
- 默认生产运行：夹具关闭，调用被拒绝，现有 MCP 行为不变；
- Rust 轨：明确返回“不支持 VM fixture”，不修改 Rust 组件状态。

Tetris 的字段名、golden 场景和 Rust 对拍仍归属 auto-os Plan 005；本计划只
提供框架通道、协议和自测证据。

## 1. 目标

- G1：提供一个名为 `autoui_fixture` 的 AutoUI MCP 工具，支持按状态字段
  注入标量、数组和对象值，并返回“已应用”而非仅“已排队”。
- G2：夹具写入走 VM 的真实 `DynamicComponent::write_state` /
  `write_state_vec` 路径，写入后标记视图 dirty；可选地在同一个 VM update
  周期内派发一个现有事件（例如 Tetris 的 Tick），避免时间竞态。
- G3：以显式环境门控和请求校验保护生产面：`AUTOUI_TEST_FIXTURES=1`
  才允许执行；未开启、字段未知、类型不兼容、大小超限都返回结构化错误。
- G4：merged/no-merge 两种 VM 启动方式都能由同一 MCP 请求得到同一状态结果；
  Rust MCP 对该工具保持拒绝且不影响既有 `autoui_action`。
- G5：补齐 auto-lang 单测/VM MCP 集成探针和文档，使 Plan 005 能复用同一组
  1/2/3/4 行消除与 7 种方块旋转 golden。

**非目标**

- 不在 app.at 中加入 debug button、debug handler、测试分支或编译期开关。
- 不让框架知道 Tetris 的 `board`、`piece` 等业务字段；字段由调用方按当前
  `autoui_state` 声明面提供。
- 不把 fixture 工具用于 Rust 组件或 Vue HTTP 前端；Vue 仍使用 HTTP 业务链路。
- 不改变普通 MCP 动作、键盘、截图、等待、状态读取的既有语义。
- 不以固定 sleep 作为“应用成功”的判据。

## 2. 架构方案

### 2.1 数据流

```
MCP HTTP /mcp
  -> tool_definitions + dispatch_tool_static
  -> autoui_fixture 校验/分配 request_id
  -> ActionMessage::Fixture(payload)
  -> VM poll_mcp_actions()
  -> update_inner()
  -> DynamicComponent.write_state / write_state_vec
  -> 可选 trigger 事件（同一 update）
  -> view_dirty + SharedState fixture ack
  -> MCP 返回 applied / error
```

当前代码锚点：

- `crates/auto-lang/src/ui/mcp_server.rs`
  的 `SharedState`、`ActionMessage`、`ActionTarget`、
  `tool_definitions`、`dispatch_tool_static`、`tool_state`、
  `tool_wait`。
- `crates/auto-lang/src/ui/iced/renderer.rs`
  的 `poll_mcp_actions`、`update_inner`、`json_to_auto_val`；
  夹具拦截必须发生在普通 `on_with_input_for` 之前。
- `crates/auto-lang/src/ui/dynamic.rs`
  的 `write_state`、`write_state_vec`、
  `read_state_as_vec` 和 dirty 标志。
- `crates/auto-lang/src/ui/session.rs`
  的 per-App `mcp_shared`，用于把应用线程的应用结果回传 MCP。

### 2.2 MCP 请求/响应契约（v1）

请求：

```json
{
  "schema_version": 1,
  "state": {
    "board": [0, 0, 1],
    "piece": 0,
    "rotation": 1,
    "px": 7,
    "py": 16,
    "pending_lock": true,
    "phase": "playing"
  },
  "trigger": {
    "widget": "TetrisStore",
    "event": "Tick",
    "input": null
  }
}
```

- `schema_version` 必须为 1。
- `state` 必须是非空对象；字段名必须已经出现在当前
  `autoui_state` 声明面中。
- 值递归支持 null/bool/int/float/string/array/object；数组写回时必须保留
  VM 数组字段的现有存储形态（`write_state_vec`），不能把数组 ID 当普通
  整数覆盖。
- `trigger` 可选。存在时，夹具字段全部写入成功后，在同一个 VM update
  周期内按普通 handler 入口派发；不允许执行任意代码或新增 handler。
- 请求体、字段数、递归深度、数组元素数有固定上限，超限返回可读错误。

响应：

```json
{
  "status": "applied",
  "request_id": 17,
  "changed": ["board", "piece", "rotation", "px", "py", "pending_lock", "phase"],
  "trigger": "TetrisStore.Tick"
}
```

应用线程在写入和可选 trigger 完成后写入 ack；MCP 工具等待有界超时。
超时、未知字段、写入失败或 trigger 失败均返回 `isError=true)，不得伪报
`queued` 为成功。

### 2.3 安全与后端边界

- `AUTOUI_TEST_FIXTURES=1` 是必要条件；默认关闭。关闭时工具返回明确错误，
  不写队列、不改状态。
- 工具确认当前运行时是 VM；Rust 轨调用返回明确“不支持”，不进入
  `DevToolsState` 的普通 action 路径。
- HTTP MCP 继续只监听本机既有地址；本计划不新增公网监听。
- 夹具接口不出现在任何 app 的可见 UI 或业务 action 列表中。
- 夹具请求不得写入持久化存储，Tetris 的 high score 等持久化字段由真实
  handler 自己决定，测试脚本负责在场景间显式重置。

## 3. 技术栈

- Rust：`serde_json`、现有 AutoValue/VM bridge、iced update/subscription。
- AutoUI MCP：现有 HTTP JSON-RPC `tools/list` / `tools/call`。
- Python：扩展 `.agents/skills/autoui-verifier/scripts/test_vm_mcp.py`
  的客户端，新增可复用 `fixture()`、`state()`、`wait_state()` 辅助。
- Tetris：Plan 005 现有 VM/Rust golden，不在本计划新增业务实现。

## 4. 需求分析与背景调查

**授权记录（2026-09-14）**：用户明确要求为 AutoUI MCP 增加测试用棋盘注入
接口；明确禁止在游戏 UI 中添加临时调试按钮；要求先在 auto-lang 单独立项，
完成后再用同一套 golden 验证 VM 与 Rust。此前已确认 Vue 使用 HTTP，VM 需
支持 merged/no-merge。

**当前实现事实**

| 事实 | 代码证据 | 对本计划的影响 |
| --- | --- | --- |
| MCP 工具集中注册/分发 | `ui/mcp_server.rs:tool_definitions`、`dispatch_tool_static` | 新工具不需要新增 server |
| 动作通过 channel 进入 iced | `SharedState::send_action`、`poll_mcp_actions` | 夹具应复用同一线程边界 |
| VM 状态可写且有数组专用 API | `DynamicComponent::write_state` / `write_state_vec` | 需要按现有字段类型选择写入路径 |
| renderer 已有递归 JSON 转换 | `renderer.rs:json_to_auto_val` | 可抽成共享转换器，避免 MCP scalar-only helper 漂移 |
| Rust MCP 使用独立 DevTools 路径 | `DevToolsState::mcp_shared`、`devtools_subscription` | Fixture 必须明确拒绝 Rust |
| 现有 `autoui_state`/等待可观察结果 | `tool_state`、`tool_wait` | golden 以状态和 ack 判定，不依赖截图或 sleep |

**依赖与约束**

- 这是 AutoUI MCP/VM runtime 的架构级小协议扩展，执行前须先落架构设计
  文档，再建 `D:/autostack/.wt/lang-623/auto-lang` worktree。
- auto-lang 变更按 Category B 至少运行 `cargo check -p auto-lang` 和
  相关 ui/mcp scoped tests；若触及核心消息协议，再在 review/fold 前按规约
  运行对应全量门禁。
- Plan 005 不得在本计划未落地前把 VM fixture golden 标成通过。

## 5. 详细设计

### 5.1 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
| --- | --- | --- | --- | --- | --- |
| SD-01 | add | `docs/specs/auto-lang/mcp/overview.md` | before：AutoUI MCP 只有用户动作/观测工具；after：增加 gated `autoui_fixture` 的 VM-only v1 契约、ack 和限制 | 让测试夹具成为可复用框架能力且默认关闭 | AC-01, AC-02, AC-03 |
| SD-02 | add | `docs/specs/auto-lang/mcp/architecture.md` | before：ActionMessage 只有 Event/Path；after：Fixture 作为独立目标通过同一 MCP→iced 队列传递，Rust 路径拒绝 | 避免把测试语义伪装成普通点击并冻结边界 | AC-02, AC-04 |
| SD-03 | add | `docs/design/autoui/autoui-mcp-test-fixture.md` | 记录请求 schema、限制、ack 时序、merged/no-merge 约束与 Tetris 移交 | L2 级协议变更需要单独设计依据 | AC-01..AC-06 |

### 5.2 协议与执行

- 新增 `ActionTarget::Fixture`（或等价的不可与 Event 混淆的目标类型），
  携带 request id；普通 `ActionTarget::Event/Path` 的既有匹配行为保持不变。
- `tool_fixture` 在 server 线程完成 schema、大小、字段存在性和 VM/gate
  检查，然后向动作队列提交序列化 payload。
- renderer 在 `update_inner` 入口解码 payload，逐字段写入：
  - 当前字段为数组时用 `read_state_as_vec`/ `write_state_vec`；
  - 其他字段用 `write_state`；
  - 任一字段失败则整次请求失败，不提交 trigger。
- 写入成功后设置 `view_dirty`，再按请求的 `trigger` 调用现有
  `on_with_input_for`/timer 入口；trigger 失败不得吞掉。
- 通过 `SharedState` 的 request-id ack 表或等价一次性 reply 通道回传
  changed 字段与错误；MCP 端有界等待后返回结构化结果。
- 提取一个共享的递归 JSON→AutoValue 函数，补齐数组/对象和整数范围校验；
  不改变现有 command-result 转换语义。

### 5.3 Tetris 移交格式

Plan 005 在本计划完成后增加 VM golden runner，使用同一份 case 数据驱动：

- opening/lock；
- 7 种方块的 4 种旋转；
- 1、2、3、4 行消除及得分；
- 场景前显式重置 board/piece/rotation/px/py/pending_lock/phase；
- fixture 应用 ack 后再通过 `trigger: Tick` 或现有键盘/动作入口推进；
- 用 `autoui_state` 读取 board/score/lines/phase/feedback，与
  `rust-workspace/036-tetris/tests/rules_golden.rs` 逐 case 比较。

## 6. 测试设计

- **单测**：tool schema/gate、递归 JSON 转换、字段/大小校验、request-id
  ack、Rust 拒绝；使用 `dispatch_tool_static` 和 mock action channel，
  不创建窗口。
- **VM 集成**：最小 AutoUI fixture app（一个标量字段、一个数组字段、一个
  trigger handler），通过真实 MCP HTTP 调用验证：
  1. gate off 拒绝且 state 不变；
  2. gate on 注入并收到 applied ack；
  3. unknown/type/oversize 拒绝；
  4. trigger 在写入后执行；
  5. merged 与 no-merge 结果相同。
- **协议回归**：现有 `autoui_action`、`autoui_keyboard`、
  `autoui_state`、`autoui_wait`、`tools/list` 既有测试全绿。
- **Tetris 移交验收**：Plan 005 的 runner 以相同 case JSON 分别驱动 VM
  merged、VM no-merge、Rust，输出逐 case diff；该项在本计划完成后由
  Plan 005 执行，不把业务结果伪装成本计划单测。

## 7. 验收标准

| ID | 标准 | 验证方法 | 期望 |
| --- | --- | --- | --- |
| AC-01 | `autoui_fixture` 工具契约可发现且请求可校验 | `cargo test` scoped MCP tests + `tools/list` | schema/version/required 字段稳定；错误可读 |
| AC-02 | 默认安全门 | gate off 集成测试 | 返回 `isError=true`；无队列消息、无状态变化 |
| AC-03 | VM 状态写入与 ack | gate on fixture app HTTP 测试 | 标量/数组/对象写入正确；收到同 request id 的 applied/error ack |
| AC-04 | trigger 时序 | fixture app + trigger handler | 先观察到字段写入，再观察到 handler 结果；失败整次可诊断 |
| AC-05 | merged/no-merge parity | 同一 fixture suite 启动两种 VM | 每个 case 的 `autoui_state` 结果字节/值一致 |
| AC-06 | Rust 边界与普通 MCP 回归 | Rust MCP 调用 + 既有 ui/mcp tests | fixture 明确拒绝；既有动作/键盘/状态工具零回归 |
| AC-07 | Tetris 可复用移交 | Plan 005 runner（后续） | 同一 rules case 数据可驱动 VM 与 Rust，阻断项 R-001 中“fixture 不可用”关闭 |

## 8. 执行步骤

| ID | 任务 | 文件/操作 | 验证 | 关联 |
| --- | --- | --- | --- | --- |
| T-00 | 架构设计先行 | 新建 `docs/design/autoui/autoui-mcp-test-fixture.md`，注册 `docs/design/00-intro.md`；记录 API、门控、ack、VM/Rust 边界 | 设计文档自洽、无未决协议分歧 | SD-03 |
| T-01 | 夹具协议建模 | `ui/mcp_server.rs` / `ui/mcp_types.rs`：Fixture target/request id、schema、错误结构；更新所有 Event/Path match | `cargo check -p auto-lang`；scoped MCP tests | AC-01, AC-06 |
| T-02 | MCP 工具入口 | `tool_definitions`、`dispatch_tool_static`、`tool_fixture`；实现 env/backend gate、字段和大小校验、有界 ack 等待 | gate/schema/unknown/type 单测 | AC-01, AC-02 |
| T-03 | VM 消费与写回 | `iced/renderer.rs`：poll、fixture decode、标量/数组写入、dirty、trigger、ack；共享递归 JSON→AutoValue | fixture app merged/no-merge integration | AC-03, AC-04, AC-05 |
| T-04 | Rust 明确拒绝 + 普通回归 | `devtools_subscription`/MCP 路径加拒绝或不可达守卫；补普通 action/key/state 回归 | Rust MCP fixture error；既有测试全绿 | AC-06 |
| T-05 | 验证器客户端 | 扩展 `.agents/skills/autoui-verifier/scripts/test_vm_mcp.py` 或新增同目录 fixture client；输出 request/ack/state 证据 | 可复用脚本，无固定 sleep 判定 | AC-03, AC-05 |
| T-06 | 规范沉淀与 Plan 005 移交 | 更新 `docs/specs/auto-lang/mcp/overview.md`、`architecture.md`、`plans.md`；在 auto-os Plan 005 写入调用契约与命令 | spec-index/lint；移交文档可执行 | SD-01, SD-02, AC-07 |
| T-07 | 复审门 | 在 lang-623 worktree 独立重跑验收，检查未门控路径、debug UI、协议回归和 worktree 红线 | review 记录 outcome pass/blocked | AC-01..AC-07 |

### 执行进度（2026-09-14）

- [✅ 已完成] T-00：在 `D:/autostack/.wt/lang-623/auto-lang` 完成架构设计文档 `docs/design/autoui/autoui-mcp-test-fixture.md`，并登记 `docs/design/00-intro.md` 与 `docs/design/autoui/README.md`；提交 `1f62ca0df`。协议已裁定使用独立 `ActionTarget::Fixture` + request-id ack，VM/Rust capability 边界与门控已写入设计。
- [✅ 已完成] T-01：`ActionTarget::Fixture`、`BackendKind`、`FixtureAck` 和有界 request-id 回执表已落到 `ui/mcp_server.rs`；所有 Event/Path 消费 match 已补齐。
- [✅ 已完成] T-02：`autoui_fixture` 已注册到 `tools/list` 和 dispatch，固定 schema v1、递归转换、字段/大小/深度/数组上限、gate/backend/type 校验及 2 秒 ack 超时已实现；4 个 scoped MCP 单测通过。
- [✅ 已完成] T-03：VM renderer 在普通 handler 入口前消费 fixture，按标量/数组写回并设置 dirty，可触发既有 handler/timer；Tetris merged 与 no-merge HTTP 实例均完成 1–4 行 golden 注入，回执和 100/300/500/800 分结果一致。
- [✅ 已完成] T-04：Rust DevTools 路径明确丢弃 Fixture；Rust 原生实例以扩大主线程栈运行后，`autoui_fixture` 返回 `backend_unsupported`，`autoui_state` 仍可用。
- [✅ 已完成] T-05：`test_vm_mcp.py` 新增 `fixture()`、`state()`、`wait_state()` 客户端辅助；已用 HTTP 调用验证 applied/error 回执。
- [✅ 已完成] T-06：mcp overview/architecture/plans 规格已沉淀 ADR-07 和 Plan 623 索引；Plan 005 已写入 fixture 移交格式、merged/no-merge 正确命令及 Rust 边界，提交 `9d40544`。
- [✅ 已完成] T-07：独立验收完成。`cargo check -p auto-lang` 通过；`cargo test -p auto-lang --features ui-iced tests_plan623 --lib` 为 4/4；真实 Tetris VM merged/no-merge MCP 均完成 fixture applied、trigger 和 1–4 行分数 golden，门控关闭返回 `fixtures_disabled` 且 score 保持 0；Rust 原生 MCP 返回 `backend_unsupported`。工作树无未提交改动，提交 `432a53b2c`。

## 9. 复审记录

- 2026-09-14 stage:new（/auto-plan:new，rev 1；路径修正后 rev 2）：基于
  `ui/mcp_server.rs`、`ui/iced/renderer.rs`、`ui/dynamic.rs`、
  `ui/session.rs` 与 mcp/ui specs 实勘，确认现有动作队列和状态写回能力，
  确认缺口是“测试夹具协议 + VM 消费 + 完成 ack”，而非 Tetris UI 设计。
  `outcome: pass`，`next: work`；下一步为 T-00 设计文档后建立
  `D:/autostack/.wt/lang-623/auto-lang` 执行 worktree。

- 2026-09-14 stage:review | plan_id: PLAN-623 | plan_revision: 2 |
  outcome: pass | reviewed_commit: `b2a5bee7ae486f50cded6e9d2a88a09d27a1ec5e` |
  base_commit: `6084ac2e90886d76d2ed390bfaeaefcc5657dd96` |
  dependency_revisions: `auto-down@67bb508`（组内 detached 基线） |
  spec_inputs: `docs/design/autoui/autoui-mcp-test-fixture.md`（提交
  `1f62ca0df`）、`docs/specs/auto-lang/mcp/{overview,architecture,plans}.md`
  （实现提交 `58b87541d`）；`supersedes_spec_components: []`，
  `new_spec_components: [auto-lang/mcp/autoui-test-fixture]`，
  `touched_goals: []`（无产品目标变更）。
  acceptance_results: AC-01 pass（4 个 `tests_plan623` 单测、真实
  `tools/list` schema/校验）；AC-02 pass（fixture gate off 返回
  `fixtures_disabled` 且 score 保持 0）；AC-03 pass（VM HTTP fixture
  标量/数组写回、request-id applied/error）；AC-04 pass（`App.Tick`
  写入后触发并返回 trigger）；AC-05 pass（VM merged :9257 与 no-merge
  :9258 同一 1–4 行 golden，分数 100/300/500/800）；AC-06 pass（Rust
  :9259 明确返回 `backend_unsupported`，普通 `autoui_state` 可用）；
  AC-07 pass（Plan 005 已提交可执行移交契约，后续 runner 按该契约复用
  同一 rules case 数据）。
  findings: 计划范围内无遗漏、未门控生产路径或临时 UI；`cargo tf` 与
  `cargo tv` 的完整档在本机均被既有 `ffi::tests::test_rust_ffi_*` 失败
  提前终止（auto-cache 无法解析 home directory），失败路径不在本次 diff；
  因此以 scoped MCP/UI 测试和运行时 HTTP golden 作为本计划回归证据。
  evidence: `cargo check -p auto-lang --features ui-iced` 通过且新增夹具
  行无编译告警；`cargo test -p auto-lang --features ui-iced
  tests_plan623 --lib` 4/4；真实 VM merged/no-merge 与 Rust MCP 结果见
  本节 acceptance_results；工作树在复审前干净。复审在执行上下文中完成，
  已按提交、基线、规格和运行时结果重建结论，未声称由独立 agent 完成。
  next: merge after explicit approval；本复审不执行合并。

## 10. 待澄清事项

- T-01 的实现需按 T-00 设计落地独立 `ActionTarget::Fixture`；若代码勘验发现现有消息边界无法承载 ack，须先记录 needs_replan，不得退化为固定等待。
- 多 App 桌面场景下是否要求请求携带 app/session id；单 App Tetris 可先用当前
  MCP 会话，但协议应为后续多 App 留出字段。
- VM no-merge 集成测试需要可用的本地 HTTP backend 启动方式；若环境缺失，
  记录为环境阻断，不以 merged 结果代替。
- 本计划完成前，Plan 005 的 VM fixture golden 保持 blocked。



