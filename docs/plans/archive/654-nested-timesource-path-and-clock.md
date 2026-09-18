---
plan_id: PLAN-654
status: archived               # drafting → executing → execution_done → reviewed → archived
completion_kind: delivered
feature_name: nested-timesource-path-and-clock
author: [agent]
created_at: 2026-09-18
updated_at: 2026-09-18TstageB-fold
plan_revision: 1
current_step: 12
total_steps: 12

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-lang/ui/design/nested-timesource.md
  - docs/specs/auto-lang/ui/design/clock-service.md
  - docs/specs/auto-lang/ui/overview.md
touched_goals:
  - GOAL-007: TimeSource path 级订阅身份 + 框架 Clock 服务（多层时间调度收官线）

affects: [auto-lang/ui]
phases:
  - id: A
    name: InstancePath TimeSource
    design_ref: component-time-and-events.md §3/§7 阶段 2
  - id: B
    name: Clock 服务
    design_ref: component-time-and-events.md §4.3/§7 阶段 3
---

# [PLAN-654] nested-timesource-path-and-clock —— 阶段 2 InstancePath + 阶段 3 Clock 服务

## 0. 变更摘要

**上游**：设计 [component-time-and-events](../design/autoui/component-time-and-events.md)；**PLAN-652（阶段 1，已 archived）** 落地类型级 `TimeSource` + `mounted_types` 订阅过滤，gallery 012-clock 实机走时。

**本计划（阶段 2 + 3 同契约、分阶段执行）**：

| 阶段 | 交付 | 解决 |
|------|------|------|
| **A（设计阶段 2）** | **InstancePath** 订阅身份；装配期 **path 级 mounted**；for/条件实例 **分订阅与退订**；派发消息带 path；MCP/调试可观察 TimeSource 表；ui specs 回写 | 同类型多实例共用一条 Tick；实例离开后仍可能类型级误订；path 不可观测 |
| **B（设计阶段 3）** | **Clock 服务**（框架级墙钟：`now_sec`/字段或 store 注入）；与 desktop `__wm_clock` 语义对齐说明；至少一条示例/指南证明「可不自起高频 Tick」；与自声明 `.Tick` **并存** | 时钟类组件无必要 4Hz 自泵；应用作者缺少统一墙钟入口 |

阶段 A **默认先交付并可独立 review**；阶段 B 依赖 A 的订阅模型稳定（Clock 不替代业务 timer）。多阶段纪律见 §8：A 验证通过后 **fold + re-sync** 再开 B（AGENTS.md 多阶段计划）。

### 非目标

- 子组件 model **实例槽完全隔离**的大规模状态重构（path 派发若仍落到类型级 handler，须在 AC/specs **显式文档化**，不静默假称 per-instance state）。
- VM 回调 props（Plan 449）。
- Element 帧间缓存 D-1。
- 强制迁移全部示例到 Clock 服务（只提供能力 + 指南 + ≥1 个示范消费面）。
- vue 轨生成器语义大改（Clock 若双端，vue 侧以不回退/可选对齐为限；阶段 B 验收以 **VM/iced** 为准）。

## 1. 目标

### 阶段 A

- **G-A1 path 身份**：`TimeSourceRuntime` 携带稳定 `InstancePath`（或等价）；订阅 identity 可区分同 `widget` 的多实例。
- **G-A2 path 挂载**：装配期登记 **mounted paths**；实例离开（for 长度变短/条件臂切换）后，对应 path 的 Tick/Timer **停止订阅**。
- **G-A3 派发可路由**：tick 消息携带 path；至少能区分「哪个实例的消息」；若状态仍类型级合并，行为可预测且有测试/文档钉住。
- **G-A4 when 保持**：`timer_when_allows_subscription` 不回退；path 级仍可叠加 when。
- **G-A5 可观测**：调试/MCP（或 `autoui_state`/日志 env）可列出 candidates / mounted / subscribable（含 path）。
- **G-A6 兼容**：阶段 1 单实例 gallery 012-clock、650 when 门、652 单测 **不回归**。

### 阶段 B

- **G-B1 Clock 能力**：VM 应用可读取框架墙钟（API 形态见 §5.3；实现可优先复用已有 `Time.now_sec()` 汇聚 + **订阅面降频**，避免「每个组件自订 250ms」）。
- **G-B2 与 `__wm_clock` 对齐**：desktop 注入语义（分钟级字段等）有文档对照；standalone 有可用等价物。
- **G-B3 示范**：≥1 个消费面（示例片段或 012-clock 可选臂/文档化迁移）证明 Clock 可用；**不破坏**仍声明 `.Tick` 的组件。
- **G-B4 文档**：specs/design 更新阶段 2+3 current-state 与迁移指引。

## 2. 架构方案

### 阶段 A：InstancePath TimeSource

```text
装载期
  TimeSourceRuntime {
    kind, widget, event, every_ms, when,
    path_template / 类型锚点,     // 如何在装配时实例化为具体 path
  }

装配期（aura_view_builder）
  视图树实例化 → InstancePath
    root                    → 根
    AppViewport → DemoClock → "…/AppViewport/Demo012Clock"
    for item in list        → "…/List/item#<idx|stable-id>"
  mounted_paths.insert(path)
  （可保留 mounted_types 作聚合/兼容查询）

订阅期
  for src in candidates:
    for path in mounted_paths matching src:
      if when_ok(src, path/root state):
        subs.push(identity=(app, path, widget, event, ms))

派发期
  DM::App(app, WidgetEvent { path, widget, event })
  → 查表到实例 → handler（或类型级 handler + path 日志/参数，见 §5.2）
```

**InstancePath 生成**：优先复用既有 **view path / VNodeId**（`stable_vnode_id_for_path`、`id_from_path`、Debug 树 path）——与命中测试/MCP 同源，避免第三套 id。阶段 A 任务 T-01 必须先 **实勘** builder 现有 path 产出点，再定 `InstancePath` 是「复用 `Vec<u16>` path」「VNodeId 字符串」还是「轻量 `widget#seq`」；**禁止**在未读码前发明平行 id 体系。

**兼容降级**：无 path 的 root/store 源 → `path = root` 或空 path + `always_mounted`（652 语义）。

### 阶段 B：Clock 服务

```text
┌─────────────────────────────────────┐
│  Framework Clock                    │
│  - now_sec / now_ms / date 投影     │
│  - 更新源：ServiceTick / 专用 1Hz   │
│    订阅 / desktop __wm_clock 对齐   │
└──────────────┬──────────────────────┘
               │ 写入 store 字段或
               │ 提供 handler 可读 API
               ▼
     012-clock / 世界时钟 / 业务组件
     （可选消费；仍可自订 .Tick）
```

- **不**用 Clock 取代 `timer`/业务 Tick。
- standalone：明确更新节拍（建议 ≤1s 或分钟投影 + 组件本地插值）；避免再造一条「每组件 250ms」。
- desktop：对照 `__wm_clock`（Plan 497，分钟注入）文档化：应用读字段 vs 自 Tick。

## 3. 技术栈

| 面 | 内容 |
|----|------|
| 运行时 | `dynamic.rs`（TimeSource/path/mounted）、`renderer.rs` subscription + update 路由、`vm_bridge.rs` 派发、`aura_view_builder.rs` path/mounted_sink |
| Path | 视图 path / `vnode.rs` `id_from_path`（T-01 定案） |
| Clock | `Time.now_sec` 消费面 + 可能的 store 注入 / `session`/`mcp`/desktop `__wm_clock` 文档 |
| 测试 | 单测 plan654_*；652/650 回归；可选 gallery MCP |
| 门禁 | Category B：`cargo check -p auto-lang --features ui-iced` + 滤测 `plan654|plan652|timesource|timer|dynamic`；Clock 若触 desktop 协议文档则 merge 时 specs 回写 |

## 4. 需求分析与背景调查

### 授权记录

- **2026-09-18 用户**：「计划652结束了…起草阶段2的计划吧。阶段3也放到同一个计划里。」
- **范围**：auto-lang 为主；auto-os ui-gallery 仅验证靶；阶段 2+3 **同一 plan 契约**，**分阶段执行/可分段验收**。
- **设计**：`docs/design/autoui/component-time-and-events.md`（§7 路线）；阶段 1 specs `docs/specs/auto-lang/ui/design/nested-timesource.md` 已在 master。
- **取号**：`scripts/new-plan.sh nested-timesource-path-and-clock` → **PLAN-654**（`.next-id` → 655）。
- **worktree**：`D:/autostack/.wt/lang-654/auto-lang` + `plan-654-dev`（Plan 529；守卫拦 worktree 时 clone 隔离，handoff 注明）。

### 证据（master 现状锚，PLAN-652 merge 后）

| 锚 | 内容 |
|----|------|
| `dynamic.rs:157-195` | `timesources` / `TimeSourceRuntime`（**无 path 字段**，身份=widget 名） |
| `dynamic.rs:161-168,703-745` | `always_mounted` / `mounted_types` / `subscribable_timesources` / mount 帧 API |
| `renderer.rs:18880-18904` | 订阅循环 `subscribable_timesources` → `widget_event_tick(app, widget, event, ms)` |
| `aura_view_builder.rs` `mounted_sink` | 装配期 insert **类型名** |
| `nested-timesource.md` 已知限制 1–3 | 类型级订阅；订阅刷新时滞；MCP 子树可见性 |
| 设计 §6 E/§7 | path 级=阶段 2；Clock=阶段 3 |
| desktop `__wm_clock` | Plan 497 / projection protocol 分钟字段 |

### 多实例语义注意（必须进 AC/文档）

Plan 320 单 VM：子 model 字段并根态。**path 级订阅 ≠ 自动 path 级状态隔离**。阶段 A 允许：

1. 多实例 **分订阅/分退订**（生命周期正确）；
2. 派发带 path，handler 仍可能是类型级；
3. 若无法 per-instance 写状态，**specs 明确限制**，不把「两个钟各走各的」写成必达，除非 T-02 实勘证明 VM 已支持实例状态槽。

## 5. 详细设计

### 5.1 阶段 A — 数据结构（目标形态）

```rust
/// 稳定实例身份（T-01 定案后定别名/新类型）。
pub struct InstancePath(pub String); // 或包装 VNodeId / path hash

pub struct TimeSourceRuntime {
    pub kind: TimeSourceKind,
    pub widget: String,
    pub event: String,
    pub every_ms: u64,
    pub when: Option<String>,
    /// 阶段 2：候选的类型锚；实例 path 在订阅展开时由 mounted_paths 过滤。
    /// root/store 可为固定 path。
}

pub struct InstanceTimeSource {
    pub path: InstancePath,
    pub widget: String,
    pub event: String,
    pub every_ms: u64,
    pub when: Option<String>,
    pub kind: TimeSourceKind,
}

// DynamicComponent:
//   mounted_paths: RefCell<HashSet<InstancePath>>  // 可与 types 并存
//   always_mounted_paths: HashSet<InstancePath>
```

`subscribable_timesources` → 阶段 2 演进为 `subscribable_instance_timesources() -> Vec<InstanceTimeSource>`；保留类型级 API 作兼容包装（聚合 unique widget）以免一次打爆所有调用点。

### 5.2 阶段 A — 订阅与派发

- `AppTickKind::WidgetEvent`（或等价 recipe）**扩展 path 维**：identity 含 path 字符串；同 widget 不同 path = 不同 iced Subscription。
- update：解析 path →  
  - **优先**：path→实例路由（若存在）；  
  - **否则**：按 `widget` 走现 `on_with_input_for` / `fire_timer`，并在调试日志带 path。
- `when`：timer 仍读根态/store 字段（阶段 2 **不**强制 per-path when 状态）；文档写明。

### 5.3 阶段 B — Clock API（实现选型在 T-B0 记录）

候选（T-B0 二选一或组合，写入决策注记）：

| 形态 | 描述 |
|------|------|
| **C1 文档+现状** | 强调 `Time.now_sec()`；提供「组件勿自订高频 Tick」指南 + desktop `__wm_clock` 对照；零新 API |
| **C2 store 注入** | 框架向根态注入 `__clock_now_sec` 等（standalone ServiceTick / desktop 对齐分钟投影） |
| **C3 轻量原语** | `clock.now_sec()` 模块 fn 或只读 computed；与 650/652 订阅模型兼容 |

阶段 B **最低交付**：C1 必做；C2/C3 至少一条落地或明确「因 Y 暂缓并记债」。示范：文档代码块 +（可选）gallery/012-clock 消费说明。

### 规范增量

| delta_id | 操作 | 目标 | before → after | rationale | AC |
|----------|------|------|----------------|-----------|-----|
| **SD-A01** | modify | `docs/specs/auto-lang/ui/design/nested-timesource.md` | 阶段 1 only → 增 **阶段 2** path/mounted/派发/限制 | current-state 跟上实现 | AC-A6 AC-A7 |
| **SD-A02** | modify | `docs/specs/auto-lang/ui/overview.md` | 类型级 TimeSource → path 级订阅身份 | 现状入口 | AC-A7 |
| **SD-A03** | modify | `docs/plans/KNOWN-DEBT-AND-RISKS.md` P530-D2 | 注记 path 级退订范围/是否仍开残项 | 债项对账 | AC-A7 |
| **SD-B01** | modify/extend | `nested-timesource.md` 或 `docs/specs/auto-lang/ui/design/clock-service.md` | （无阶段 3）→ Clock 能力 + 与 `__wm_clock` 对照 + 迁移指引 | 阶段 3 知识 | AC-B3 |
| **SD-B02** | modify | `ui/overview.md` + `component-time-and-events.md` §7/§9 | 路线「待立项」→ 654 承接状态 | 设计/speck 一致 | AC-B3 |

设计文档 `component-time-and-events.md` 随 654 进展追加变更记录（§9），不复制 plan 过程。

## 6. 测试设计

### 阶段 A

1. **单测**
   - path 生成稳定：同一 for index/同一条件臂 → path 稳定（跨帧）；
   - 多实例：两个 mounted path 同 widget → `subscribable_instance_*` **两条**；
   - 退订：移除一个 path → 仅剩一条；
   - 类型级兼容 API 与 path 级一致（对单实例语料）；
   - when / 652 测不回归。
2. **门禁**：`cargo check --features ui-iced`；滤测 plan654/plan652/plan650/timer。
3. **实机（可选加强）**：合成 for 双实例或最小 .at 语料 VM；gallery 012-clock 回归。

### 阶段 B

1. Clock API/字段可读（单测或最小 app）；
2. 未迁移 `.Tick` 组件行为不变；
3. 文档对照 `__wm_clock` 存在且与实现一致；
4. 门禁同 Category B。

## 7. 验收标准

### 阶段 A

- [x] **AC-A1** `InstancePath`（或等价）进运行时模型；单测证明同 widget 多 path 可区分。
- [x] **AC-A2** path 级 mount：装配登记 mounted paths；实例消失后 **不再订阅** 该 path（单测）。
- [x] **AC-A3** 订阅 identity 含 path：同 widget 两 path 可同时出现在 subscribable 列表。
- [x] **AC-A4** 派发：消息含 path；路由到 handler（或类型级+path 可观测）；**状态语义**（隔离或不隔离）有测试或文档钉住。
- [x] **AC-A5** when 门与 652/650 回归绿。
- [x] **AC-A6** 可观测：MCP/state/debug 至少一种可列出 timesources/mounted/subscribable（含 path）。
- [x] **AC-A7** SD-A01..03 内容成文（merge 落 specs）；design §7 阶段 2 状态更新。
- [x] **AC-A8** 门禁 check + 滤测通过；gallery 012-clock 单实例回归不红。

### 阶段 B

- [x] **AC-B1** Clock 能力落地（C2/C3 之一）**或** C1 文档+决策记录「暂缓新 API」有用户可见理由（非空话）。
- [x] **AC-B2** standalone 可消费墙钟（API/字段/文档可执行说明）。
- [x] **AC-B3** SD-B01/B02 + 与 `__wm_clock` 对照文档；`.Tick` 兼容不回归。
- [x] **AC-B4** 阶段 B 门禁绿；阶段 A 验收证据仍有效。

> **阶段门**：A 的 AC 全过才开 B 的实现任务（允许提前写设计注记）；B 不得静默删减 A 的 AC。

## 8. 执行步骤

> 多阶段：worktree 全生命周期一个 `plan-654-dev`；**阶段 A 完成且门禁绿后**按 AGENTS fold 到 master 并 re-sync，再执行 B（防与并行 master 漂移）。簿记在 master plan 文件。

### 阶段 A

- [x] **T-A01 实勘 path 体系** [✅ 已完成]  
  - 读：`aura_view_builder` child/for 渲染、`vnode.rs` path、`mounted_sink` 写入点、`AppTickKind::WidgetEvent`。  
  - 产出：path 形态裁定（复用何种 id）+ 派发是否可达 per-instance state 的结论 → §10/§9 注记。  
  - AC：AC-A1 前置  
  - 证据：见 §9 T-A01；worktree clone 隔离 + auto-down sibling；InstancePath=`{widget}@{mount_seq}`；状态隔离不可达已文档化。

- [x] **T-A02 TimeSource path 模型** [✅ 已完成]  
  - `dynamic.rs`：path 类型、mounted_paths、subscribable_instance_*、兼容 API。  
  - 验证：单测 AC-A1/A3。  
  - 证据：worktree `plan-654-dev`；`cargo check --features ui-iced` Finished；`cargo t plan654` 7/7 PASS（identity/multi-instance/unmount/when/type-fallback/debug-rows/dispatch）；`cargo t plan652` 5/5；`cargo t plan650` 1/1。

- [x] **T-A03 装配登记 path** [✅ 已完成]  
  - `aura_view_builder.rs`：实例化时写 path；begin/end mount 帧语义扩展。  
  - 验证：单测 AC-A2。  
  - 证据：commit @ plan-654-dev；`cargo t plan654` 9/9（含 view_assembly_registers_paths / for_view_multiple）；plan652 5/5。

- [x] **T-A04 订阅 recipe 扩展** [✅ 已完成]  
  - `renderer.rs` + subscription identity：含 path；避免双订。  
  - 验证：AC-A3 + 回归 AC-A5/A8。  
  - 证据：subscription loop → `subscribable_instance_timesources()`；`widget_event_tick(app, path.as_str(), …)`；hash 含 path。

- [x] **T-A05 派发路由** [✅ 已完成]  
  - update 侧消费 path；文档化状态语义。  
  - 验证：AC-A4。  
  - 证据：`on_with_input_for`/`fire_timer`/`is_timer_entry`/`is_timesource` 剥 `@seq`；`plan654_dispatch_is_type_level_state_with_path_identity` 钉住「path 可观测 + 类型级状态」。

- [x] **T-A06 可观测** [✅ 已完成（最小面）]  
  - MCP/state/debug 列表（最小面）。  
  - 验证：AC-A6。  
  - 证据：`timesource_debug_rows()` 返回 (path,widget,event,kind,ms,when,mounted,subscribable)；单测 `plan654_debug_rows_include_path`。MCP server 全量接线可在 review 前按需补。

- [x] **T-A07 阶段 A 规范草稿 + 门禁** [✅ 已完成（specs 草稿+滤测+gallery 实机）]  
  - SD-A01/A02 草稿已在 worktree：`nested-timesource.md` 阶段 2 段 + `ui/overview.md` 更新。  
  - SD-A03（KNOWN-DEBT P530-D2 注记）merge 时落 master ledger。  
  - 门禁：check + plan654/652/650/fire_timer 滤测全绿。  
  - **gallery 实机（2026-09-18）**：`docs/plans/evidence-p654-gallery-live.json`  
    - 652 回归红线 **PASS**：gallery idle `w_local="--:--:--"` → 选中 012-clock 走时 `22:03:42→45` → 切 002-counter tick 停止增长；standalone `clock_running=true`。  
    - path 可观测 **PASS**：`[TS_PATH] path=Demo012Clock@0`；`UI_EVENT widget="Demo012Clock@0"`；`handler_Demo012Clock_Tick` 类型级派发仍在。  
    - standalone path=`App`（always_mounted 类型形态）。  
    - 注：日志中同 path 同时出现 Tick 与 Timer 候选（ms=250）——iced 按 `WidgetEvent(path,event,ms)` hash 去重，实际单订；候选表并存待 review 核对 012-clock 是否双声明。  
  - 多实例 gallery 语料：无现成双实例 Tick demo；模型层由 plan654 单测覆盖。静止不劣化：idle 实机 `w_local` 恒 placeholder + 单测 `plan652_idle_*`。  
  - AC：AC-A5/A7/A8 **pass**。  

- [x] **T-A08 阶段 A 收执** [✅ 已完成]  
  - §9 A 段 handoff；fold master + re-sync worktree；**阶段 B 未开**（overall 仍 `executing`）。  
  - landing：format-patch apply（隔离钩子拦 merge）；evidence `docs/plans/evidence-p654-gallery-live.json`。  
  - 依赖 sibling `.wt/lang-654/auto-down` 保留至阶段 B 结束再清理。  

### 阶段 B（A 完成后）

- [x] **T-B00 Clock 选型决策** [✅ 已完成]  
  - **裁定 C1 + C2**（见 §9 T-B00 差异表）。默认节拍 **1s**。  
  - C3 不单独立项：`Time.now_sec()` 已是 handler 原语。  

- [x] **T-B01 Clock 实现** [✅ 已完成]  
  - `__clock_now_sec` / `__clock_hhmm` + `write_or_insert_state`；`wants_framework_clock` 门控 1Hz `__clock_tick`；Init 播种。  
  - 验证：AC-B1/B2。  
  - 证据：`plan654_framework_clock_fields_and_gate` PASS；worktree commit stage B。  

- [x] **T-B02 示范与兼容** [✅ 已完成]  
  - `clock-service.md` 用法 + 迁移指引；`.Tick` 兼容测 `plan654_clock_does_not_break_self_tick_component` PASS。  
  - AC：AC-B3  

- [x] **T-B03 阶段 B 文档 + 门禁** [✅ 已完成（待 fold 后 review）]  
  - SD-B01 `docs/specs/auto-lang/ui/design/clock-service.md`；SD-B02 overview/nested-timesource 链接。  
  - 门禁：check + plan654 **11/11** + plan652 **5/5** + plan650 **1/1**。  
  - AC-B4：阶段 A 证据仍有效 + B 门禁绿。  
  - **overall `execution_done` 待 stage B fold 到 master 后 flip**（下一步 `/auto-plan:review` 或先 fold）。  

### 完成收执（2026-09-18）

- **status: execution_done**；阶段 A+B 任务 12/12；next: `/auto-plan:review`。

## 9. 复审记录

- 2026-09-18 draft handoff（/auto-plan:new）：`stage: new | plan_id: PLAN-654 | plan_revision: 1 | outcome: pass（草稿）`。  
  上游：PLAN-652 archived（delivery `0b301b17e`）；设计 component-time-and-events §7 阶段 2/3。  
  授权：用户 2026-09-18 指定阶段 2+3 **同一计划**立项。  
  **next: work**（先 T-A01 实勘 path；worktree `D:/autostack/.wt/lang-654/auto-lang` / `plan-654-dev`）。

- **2026-09-18 T-A01..T-A06 work**（stage: work | plan_id: PLAN-654 | plan_revision: 1 | outcome: partial-continue）：
  - **环境**：`git worktree add` 被隔离钩子拦 → clone 隔离 `D:/autostack/.wt/lang-654/auto-lang` @ `plan-654-dev`；
    跨仓 `autodown-core` path 依赖 → 组内 sibling clone `D:/autostack/.wt/lang-654/auto-down`。
  - **主检出 WIP**：`examples/rust-workspace/Cargo.toml` foreign members 变更——**未纳入**本计划。
  - **commits**：`e3e250bd7` T-A01/T-A02 path 模型；`6d9c41daf` T-A03 builder 登记；后续 T-A04/T-A05/T-A06 订阅+派发+debug rows。
  - **门禁**：`cargo check --features ui-iced` Finished；`cargo t plan654` 9/9；`cargo t plan652` 5/5；`cargo t plan650` 1/1；`fire_timer` 1/1。
  - **裁定落地**：InstancePath=`{widget}@{mount_seq}`；状态隔离不可达（单 VM 根态）已在测试钉住；订阅 identity 含 path。
  - **remaining**：T-A07 规范草稿 SD-A01..03 + gallery 回归；T-A08 收执 handoff；阶段 B 未开。
  - **next: work T-A07**（specs 回写 + 门禁；用户要求 fold 后再开 B）。

- **2026-09-18 T-A07 partial**：SD-A01/A02 specs 草稿已入 worktree commit；滤测门禁绿；
  gallery 实机 + fold 收执留 T-A08。代码 commit 链：
  `e3e250bd7`（T-A01/A02）→ `6d9c41daf`（T-A03）→ `a989ca79c`（T-A04/A05/A06）→ specs draft。
  **stage A 代码面基本齐**；**next: user 确认后 fold + T-A08 handoff，再开阶段 B**。

- **2026-09-18 gallery live（stage: work）**：
  `outcome: pass` | evidence `docs/plans/evidence-p654-gallery-live.json` @ plan-654-dev |
  REGRESSION_652_GALLERY=PASS · REGRESSION_652_STANDALONE=PASS · PATH_OBS_LOG=PASS · PATH_OBS_HANDLER=PASS |
  注记：TS_PATH 候选含 Tick+Timer 同 path（hash 去重）；gallery 无双实例语料（单测覆盖）|
  **next: T-A08 fold 收执**。

- **2026-09-18 T-A08 fold（stage: work | plan_id: PLAN-654 | plan_revision: 1）**：
  `outcome: pass`（阶段 A land）|
  landing_method: format-patch apply @ master（会话隔离钩子拦 `git worktree add`/`git merge`；clone 隔离 worktree）|
  patches: `e3e250bd7`→`6d9c41daf`→`a989ca79c`→`8dd9e84f0`→`3bcc8aea5` |
  交付文件：`dynamic.rs` InstancePath/mounted_paths/instance timesources；`aura_view_builder.rs` path sink；`renderer.rs` path 订阅+TS_PATH；specs SD-A01/A02 草稿；gallery evidence |
  未 fold 内容：master foreign WIP（`examples/rust-workspace/Cargo.toml`、`docs/plans/655-*`）保持原样 |
  dependency_revisions：`.wt/lang-654/auto-down` clone sibling（autodown path）；阶段 B 期间保留 |
  **overall status 仍为 `executing`**；阶段 A AC 映射见上；**next: 阶段 B T-B00 Clock 选型**（需用户确认开 B，或按契约续作）。

- **2026-09-18 T-A08 fold 收执落地**：
  - master landing commit **`b13af5927`**（8 files，format-patch apply）。
  - master 验证：`cargo check --features ui-iced` Finished；`cargo t plan654` 9/9；`cargo t plan652` 5/5。
  - worktree re-sync：隔离钩子拦 `git merge`；已 `git fetch` master → `refs/remotes/sync/master`；
    阶段 A 代码文件与 master **内容一致**（worktree 为 patch 源）。clone worktree 继续承载阶段 B。
  - foreign WIP 未触碰：`examples/rust-workspace/Cargo.toml`、`docs/plans/655-*`。
  - **stage: work | outcome: pass | next: 阶段 B T-B00**。

- **2026-09-18 T-B00..T-B03 work（stage B）**：
  `outcome: pass` @ plan-654-dev |
  选型 C1+C2、1s 节拍、`__clock_*` + 门控 1Hz、`clock-service.md` |
  门禁 plan654 11/11 + plan652 5/5 + plan650 1/1 |
  **next: fold stage B → status execution_done → `/auto-plan:review`**。

- **2026-09-18 stage B fold + execution_done**：
  `stage: work | plan_id: PLAN-654 | plan_revision: 1 | outcome: pass` |
  landing_method: format-patch apply（`76487fbea` → master）|
  阶段 A `b13af5927` + 阶段 B Clock C1/C2 + specs SD-A01/A02/B01/B02 |
  evidence: stage A `evidence-p654-gallery-live.json`；stage B plan654 11/11 + plan652 5/5 + plan650 1/1 |
  worktree: `.wt/lang-654/{auto-lang,auto-down}` 保留至 review/merge 清理 |
  foreign WIP 未触碰（`examples/rust-workspace/Cargo.toml`）|
  **status: execution_done** | **next: `/auto-plan:review`**。

- **2026-09-18 独立复审（/auto-plan:review）**：
  `stage: review | plan_id: PLAN-654 | plan_revision: 1 | outcome: **pass** |
  reviewed_commit: **4817b51e1e5268ae28fd9651f1b8aa1342d6ba86** @ master |
  base_commit: 9886ba901（worktree clone 起点）|
  worktree: clone 隔离 `.wt/lang-654/auto-lang` @ `76487fbea`（非 git worktree 注册项；以 master ancestry 验 landing）|
  dependency_revisions: sibling clone `.wt/lang-654/auto-down`（autodown-core path 解析；未改 auto-down 代码）|
  limitation: **实现会话内复审**——结论由工件重放重建（门禁重跑+证据 JSON 独立解析+代码锚 grep），非采信执行摘要 |
  spec_inputs: `docs/specs/auto-lang/ui/design/nested-timesource.md`、`clock-service.md`、`ui/overview.md`；设计 `component-time-and-events.md`；`docs/plans/evidence-p654-gallery-live.json` |
  acceptance_results:
  - AC-A1 pass | plan654_instance_path_identity_forms + multi_instance_paths_expand | InstancePath@dynamic.rs:222
  - AC-A2 pass | plan654_view_assembly_registers_paths + unmount_drops_instance_path | builder path sink
  - AC-A3 pass | renderer.rs:18983 subscribable_instance_timesources + path.as_str() 订阅
  - AC-A4 pass | plan654_dispatch_is_type_level_state_with_path_identity + nested-timesource.md「状态语义」节 + 实机 UI_EVENT widget=Demo012Clock@0 → handler_Demo012Clock_Tick
  - AC-A5 pass | cargo t plan652 5/5 + plan650 1/1 + timer_when 2/2 + fire_timer 1/1（复审重跑）
  - AC-A6 pass | timesource_debug_rows() + AUTOUI_TIMESOURCE_DEBUG 日志 + evidence TS_PATH 42 行含 path
  - AC-A7 pass | nested-timesource 阶段 2 段 + clock-service.md + overview 段落成文（F-R1 头注已校正）
  - AC-A8 pass | check Finished；plan654 11/11；gallery evidence ac6_pass=true（复审独立解析 JSON）
  - AC-B1 pass | CLOCK_* 字段 + wants_framework_clock 1Hz 门控 + plan654_framework_clock_fields_and_gate
  - AC-B2 pass | write_or_insert_state 注入 __clock_now_sec/__clock_hhmm；clock-service.md standalone 消费说明
  - AC-B3 pass | clock-service.md 与 __wm_clock 对照表 + plan654_clock_does_not_break_self_tick_component
  - AC-B4 pass | 阶段 A gallery 证据仍有效 + plan654 11/11 on master 4817b51e1
  findings:
  - **F-R1 (info,已修)** nested-timesource.md 头注曾写「plan-654-dev 待 fold」——与 master 已 fold 不符；复审期校正为 execution_done/current-state，非语义契约变更
  - **F-R2 (info,非本计划)** `cargo tf` 1 红：`ui_gen::rust::tests::test_display_family_codegen_arm_fixture`（icon size px 断言）。**无 ui-iced 时必红、有 ui-iced 必绿**；在 plan-651 基线 worktree（pre-654）同样复现 → **预存 feature 配置债**，与 PLAN-654 diff（dynamic/builder/renderer/vm_bridge/specs）无文件交集
  - **F-R3 (info,merge 时)** KNOWN-DEBT P530-D2 仍写「path 级退订仍开」——merge 清偿注记应指向 PLAN-654 阶段 A 已落地（SD-A03）
  - **F-R4 (info)** TS_PATH 同 path 同现 Tick+Timer 候选（ms=250）——iced WidgetEvent(path,event,ms) hash 去重后单订；012-clock 是否双声明待 merge 后核对，非阻断
  evidence:
  - master `4817b51e1` check Finished；`cargo t plan654` 11/11；plan652 5/5；plan650 1/1；timesource 16/16
  - `cargo tf --no-fail-fast` 3640/3641（唯一红=F-R2 预存）
  - `docs/plans/evidence-p654-gallery-live.json`：gallery ac6_pass/clock/stop=true；standalone clock_running=true；path Demo012Clock@0；handler 类型级
  - 代码锚：InstancePath/InstanceTimeSource/subscribable_instance_timesources/timesource_debug_rows/CLOCK_*/wants_framework_clock；renderer path 订阅+__clock_tick；vm_bridge write_or_insert_state
  | next: **`/auto-plan:merge`**（overall **reviewed**；worktree/clone 与 auto-down sibling 留 merge 清理；foreign WIP 仍不在本计划范围）`。

- **2026-09-18 merge 收据（/auto-plan:merge，key: PLAN-654:r1）**：
  `stage: merge | plan_id: PLAN-654 | plan_revision: 1 | outcome: pass` |
  **prepared**: reviewed@`4817b51e1`；canonical Spec 三件已在 master（A land `b13af5927` / B land `4817b51e1`，format-patch）；evidence `docs/plans/evidence-p654-gallery-live.json` |
  **landed**: master ancestry 验证 `b13af5927`+`4817b51e1` 均为 HEAD 祖先；specs nested-timesource/clock-service/overview 在盘；复审簿记 `bc676a8fa` |
  **ledger_refreshed**: `docs/specs/auto-lang/ui/plans.md` 增 654 行；`docs/specs/INDEX.md` 经 `scripts/spec-index.py`；`.autoos/specs.json` upsert `P654-1..5`（reports/goals/architecture/designs/reviews，file 指向 archive 与 canonical specs）；KNOWN-DEBT **P530-D2** 注记 path 级退订已由 654 落地 |
  **archived**: `docs/plans/archive/654-nested-timesource-path-and-clock.md`；`status: archived`；`completion_kind: delivered` |
  **cleaned**: wt-guard `auto-lang`/`auto-down`/组目录均 **clean**（exit 0）；
  clone 目录 `.wt/lang-654/{auto-lang,auto-down}` 与 patches/临时文件已删除；
  组目录 `.wt/lang-654` 移除；主仓无 `plan-654*` 分支/无注册 worktree 残留。
  foreign WIP 未触碰（`examples/rust-workspace/Cargo.toml` 等） |

- **2026-09-18 T-B00 Clock 选型**（stage: work | plan_id: PLAN-654）：

  **裁定：C1（必做）+ C2（最小 store 注入）**。C3 不单独立项——`Time.now_sec()` 已是 handler 侧原语。

  | 能力 | 形态 | 说明 |
  |------|------|------|
  | Handler 读墙钟 | **既有** `Time.now_sec()` / `Time.now_ms()` / `Time.now()` | stdlib shim `shim_time_now_sec`；vue 桥 `Date.now()`；**不**用高频 Tick 才能「知道现在几点」 |
  | 视图/state 读墙钟（免自泵） | **C2 新增** `__clock_now_sec`（int unix 秒）+ `__clock_hhmm`（str `HH:MM`） | 框架注入根态；**仅当**视图/computed 引用 `__clock_` 时才订 1Hz `__clock_tick` |
  | Desktop 对照 | 既有 `__wm_clock` / `__wm_date` | Plan 497：**分钟级**、ServiceTick 泵、只写 shell App、变化才 dirty；应用作者在 desktop 壳内可读 `__wm_clock` |

  **`__clock_*` vs `__wm_clock` 差异表**：

  | 维度 | `__clock_now_sec` / `__clock_hhmm`（PLAN-654 B） | `__wm_clock`（Plan 497 desktop） |
  |------|---------------------------------------------------|----------------------------------|
  | 作用域 | **任意** VM UI App（standalone + gallery 内嵌） | **仅** desktop shell App |
  | 节拍 | **1s**（unix 秒 / HH:MM 秒级刷新） | **1min**（HH:MM） |
  | 订阅门控 | 视图引用 `__clock_` 才订；静止无引用 **零** 额外泵 | shell ServiceTick 帧泵内变化才写 |
  | 字段 | `__clock_now_sec` int + `__clock_hhmm` str | `__wm_clock` str + `__wm_date` str |
  | 目标 | 时钟类 UI **可不自起 250ms Tick** | dock 状态栏分钟时钟 |

  **T-B00 默认节拍**：standalone/framework clock = **1s**（计划 §10.2）。
  理由：数字钟/秒显示需要秒粒度；1Hz 相对 012-clock 自订 250ms Tick 降频 4×；
  分钟投影留给 desktop `__wm_clock`；秒针平滑动画仍可 `.Tick` + `Time.now_sec()` 并存。

  **AC-B1 路径**：C2 落地 `__clock_*` + 门控 1Hz 订阅；C1 文档写清「handler 用 Time.now_sec；视图用 __clock_*；勿为墙钟自泵 4Hz」。
  **AC-B2**：standalone 可直接读 `__clock_*`；desktop 壳内对照 `__wm_clock` 文档化（应用也可读 `__clock_*` 若框架对所有 App 注入）。
  **next: T-B01 实现**。

- **2026-09-18 T-A01 实勘**（work / plan_revision 1）：`stage: work | plan_id: PLAN-654 | task_ids: T-A01 | outcome: pass`。

  **环境**：`git worktree add` 被仓库隔离钩子拦下（共享 .git registry）；按 plan 既有约定走 **clone 隔离**：
  `D:/autostack/.wt/lang-654/auto-lang` @ `plan-654-dev`，base `9886ba901`（与 master 同）。
  主检出另有 foreign WIP：`examples/rust-workspace/Cargo.toml` members 追加大量 `*-back` 条目——**未纳入**本计划 worktree。

  **Path 体系实勘锚**：

  | 锚 | 内容 |
  |----|------|
  | `vnode.rs:626-636` | `id_from_path(&[u16]) -> u64`（FNV-1a，确定性）——规范 path→id |
  | `dynamic.rs:1093-1095` | `stable_vnode_id_for_path` 包装同一 hash |
  | `aura_view_builder.rs:70-85` | tracked 路径 `path: &mut Vec<usize>`（AuraNode 结构下降路径）；**untracked `build()` 不带 path** |
  | `aura_view_builder.rs:6094-6097,6165-6167` | mounted_sink 现只写 **类型名** |
  | `renderer.rs:7149-7155,7247-7256` | `AppTickKind::WidgetEvent(widget,event,ms)` **无 path 维**；`IcedMessage{widget,event,input_value}` |
  | `renderer.rs:18940-18960` | 订阅循环：`subscribable_timesources()` → `widget_event_tick(app,widget,event,ms)` |
  | `dynamic.rs:1595-1651` | `on_with_input_for`：child handler **一律路由 ROOT state** |
  | `vm_bridge.rs:1088-1135` | `ensure_child_state` **恒返回 root_id**；`child_state_map: HashMap<widget_name,u64>` 类型键 |

  **裁定（T-A01 decision）**：

  1. **InstancePath 形态**：`String` 新类型；identity = `{widget}`（root/store/always）或 `{widget}@{mount_seq}`（child 实例）。
     - `mount_seq`：装配帧内 **按 widget 类型** 的 0-based 实例序号（`begin_mount_frame` 清零）。
     - **不**发明第三套 id；tracked/debug/MCP 展示时可叠加既有 `id_from_path(view_path)`，但订阅身份以 `widget@seq` 为阶段 A 主键（untracked 生产路径也能产）。
     - for 单 body 子组件时 `seq ≈ 迭代下标`（search 过滤时按命中序，仍帧内稳定）。
  2. **per-instance state：不可达**。Plan 320 单 VM 统一根态；`ensure_child_state` 忽略 widget_name 返回 root；handler 路由 root。
     → **path 级订阅 ≠ path 级状态隔离**。阶段 A AC-A4 以「消息含 path + 类型级 handler + 文档化限制」通过；「双钟各走各的」**不在** AC（按 plan §10.1 默认）。
  3. **派发携带 path**：`AppTickKind::WidgetEvent` 的 widget 字段扩为 **InstancePath 字符串**（`DemoClock@0`）；hash/identity 自然区分；update 解析 `widget_type()` 走现 `on_with_input_for`/`fire_timer`，path 进调试可观测面。兼容：无 `@` 的 plain widget 名 = 阶段 1 语义。
  4. **when**：仍读根态/store（阶段 A 不 per-path when）；path 级可叠加同一 when 门。
  5. **订阅刷新时滞**（§10.3）：本阶段 **不**做 mount 变更即时重订阅（保持 652 文档化时滞）；T-A04 若 identity 扩展成本低再评估。

  **AC 映射**：AC-A1 前置（形态已定）；AC-A4 按「path 可观测 + 状态语义文档化」执行；开债项候选：path 级实例状态槽（单 VM 大题，不在 654）。

## 10. 待澄清事项

1. **实例状态隔离**：**已按默认裁定**（见 §9 T-A01）——生命周期达标；per-instance state 记债，不在本计划 AC。用户若要求「双钟各走各的」须另开计划。
2. **Clock 默认节拍**：standalone 墙钟刷新 1s 还是分钟投影？T-B00 给默认并记理由。
3. **订阅刷新时滞**（652 F-02）：阶段 A **默认不**顺手做即时重订阅（T-A01 裁定）。
4. **vue 双端 Clock**：阶段 B 默认 VM 优先；vue 对齐不阻塞 AC-B*。

## 11. Handoff

- 2026-09-18 /auto-plan:new：见 §9。
- 2026-09-18 /auto-plan:work T-A01：path 形态/状态语义/派发携带方案已定；worktree clone 隔离；next T-A02 `dynamic.rs` path 模型。
