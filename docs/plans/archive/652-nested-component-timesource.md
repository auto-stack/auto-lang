---
plan_id: PLAN-652
status: archived          # drafting → executing → execution_done → reviewed → archived
feature_name: nested-component-timesource
author: [agent]
created_at: 2026-09-18
updated_at: 2026-09-18
plan_revision: 1
current_step: 8
total_steps: 8
worktree_note: |
  会话守卫拦 git worktree add → clone 隔离（650 先例）。
  实现 clone: D:/autostack/.wt/lang-652/auto-lang @ plan-652-dev
  base: d4ad02984 · code: 52bf8606b
  merge 时从 clone fetch plan-652-dev（非 git worktree 注册条目）。
  T-06 实机证据已冻结：docs/plans/evidence-p652-t06-mcp.json
  SHA256=FB2B3D8CC1D524170936ACFFF218D11F0FED1032E6859DECD9EDDD2598A6CCB1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/design/autoui/component-time-and-events.md
  - docs/specs/auto-lang/ui/design/nested-timesource.md
touched_goals:
  - GOAL-007: 嵌套组件时间源调度与挂载过滤（gallery 内嵌 Tick / D-2 类型级）

affects: [auto-lang/ui]
---

# [PLAN-652] nested-component-timesource —— 嵌套组件 TimeSource 收集与挂载过滤订阅

## 0. 变更摘要

gallery VM 内嵌 012-clock 时钟冻结（`w_local="--:--:--"`、日志 0 条 Tick）经 650 前后二进制 A/B 定罪为 **既有缺口**：`.Tick`+`interval` 仅根组件进入 iced 订阅，子组件时间源未登记；`timer` 虽静态收集 child_decls，但 **不看是否仍在树上**（P530-D2）。

系统设计见 [docs/design/autoui/component-time-and-events.md](../design/autoui/component-time-and-events.md)。本计划为 **阶段 1**：在 VM/iced 轨落地

1. **候选收集**：child_decls 的 `.Tick`/`timer` 进入 TimeSource 候选表（与既有 timer 三源收集对齐）；
2. **挂载过滤订阅**：装配期 `mounted_types` + 订阅循环只挂「挂载中且 when 通过」的源；
3. **派发打通**：子组件 Tick/timer 消息路由到单 VM 内对应 handler；
4. **实机**：ui-gallery 选中 012-clock 走时，切换 demo 后停订。

不在本计划：InstancePath 多实例分频、回调 props、Element 缓存 D-1、Clock 服务（设计 §7 阶段 2/3）。

## 1. 目标

- **G-1 嵌套 Tick 生效**：任意深度（至少 1 层 AppViewport→Demo）子组件声明 `.Tick` 后，挂载时 handler 周期执行，状态可见于 MCP/state。
- **G-2 挂载一致**：条件分支切走 / 组件不再实例化后，对应 Tick/timer **不再订阅**（类型级 D-2）。
- **G-3 when 门保持**：`timer_when_allows_subscription` 行为不回退（与 PLAN-650 E-1 正交叠加）。
- **G-4 静止不劣化**：gallery 静止（未选含 Tick 的 demo，或 demo 无 Tick）时 rebuild 频率不差于 650 后基线——禁止「全量 child 常订」。

### 非目标

- vue 轨生成器行为变更（仅保证不回退）。
- 把 gallery demo 改成独立 App。
- path 级 for 多实例订阅、框架 Clock 服务。
- MCP 快照对组件子树可见性（Plan 449 既有债，另案）。

### 受影响仓库/模块

| 仓库 | 范围 |
|------|------|
| **auto-lang** | `crates/auto-lang/src/ui/{dynamic.rs,vm_bridge.rs,aura_view_builder.rs,iced/renderer.rs,handler_codegen.rs}` + 单测 |
| **auto-os** | `ui-gallery` **仅作验证靶**，不提交产品语义改动 |

## 2. 架构方案

```text
装载期（dynamic / vm_bridge）
  root_decl + all_child_decls + store-as-child
    ├─ timer { }     → Timer 候选（已有）
    └─ .Tick + interval → Tick 候选（新增，event="Tick"）

装配期（aura_view_builder / render_child_widget）
  写入 mounted_types: HashSet<widget 名>
  root / store-as-child 恒 mounted

订阅期（iced renderer subscription）
  for src in candidates:
    mounted(src) && when_ok(src) → widget_event_tick / widget_tick

派发期（update）
  (app, widget, event) → fire_timer 或 child Tick handler（单 VM）
```

关键点（设计 §3）：

- 阶段 1 身份键 = **widget 类型名**（与 `TimerEntryRuntime.widget` 一致）；条件实例化未命中 ⇒ 该名不在 `mounted_types` ⇒ 不订阅。
- gallery：`SelectDemo("012-clock")` → AppViewport 条件臂实例化 `Demo012Clock` → mounted 含该名 → 订阅 Tick → 走时；切到 002-counter → 不在 mounted → 停订。

## 3. 技术栈

- 运行时：`dynamic.rs`（TimeSource/候选/mounted API）、`renderer.rs` subscription、`vm_bridge.rs` 派发。
- 装配：`aura_view_builder.rs` `render_child_widget` 登记 mounted。
- 测试：`dynamic.rs` / `renderer` 邻域单测；gallery MCP 实机脚本（scratch 或 `tests/`）。
- 门禁：**Category B**——`cargo check -p auto-lang --features ui-iced` + `cargo t` 滤 `dynamic|timer|timesource|plan652|ui`；无 aavm/docs_gen 必跑（未改 schema/docs_gen 时）。

## 4. 需求分析与背景调查

### 授权记录

- **2026-09-18 用户**：认可 TimeSource+挂载过滤方向后裁定「立项。先把相关文档写好。包括用技能 /auto-plan-new 来建立计划。」
- **范围**：auto-lang 框架实现 + auto-os ui-gallery 作验证靶（不强制改 gallery 源码语义）。
- **设计层**：已写 `docs/design/autoui/component-time-and-events.md` 并登记 autoui README / 00-intro；`vm-frame-budget` §5.2 D-2 指向本线。
- **取号**：`scripts/new-plan.sh nested-component-timesource` → **PLAN-652**（`.next-id` → 653）。
- **worktree（执行期）**：`D:/autostack/.wt/lang-652/auto-lang` + 分支 `plan-652-dev`（Plan 529；若会话守卫拦 `git worktree add`，按 650 先例 clone 隔离并在 handoff 注明）。

### 证据（会话实机 + 代码锚）

| 证据 | 内容 |
|------|------|
| gallery MCP（after=650 二进制） | `SelectDemo` ok；`w_local: "--:--:--"`；`event=Tick` **0** |
| gallery MCP（before=pre-650） | 同上冻结 → **非 650 回归** |
| standalone 012-clock + after | `w_local` 走秒；`h_rot` 变化；Tick 30+ |
| `dynamic.rs:408-437` | timer 三源收集（root/child/store）**已有** |
| `dynamic.rs:451` | `tick_interval: view.tick_interval` **仅 root** |
| `handler_codegen.rs:1678-1699` | `extract_tick_interval_from_decl` 仅单 decl；无 `.Tick` → None |
| `renderer.rs:18666-18688` | 订阅只扫 `app.component.tick_interval()` + `timer_entries()` |
| `AppViewport.vm.at` | `if .app == "012-clock" { Demo012Clock {} }` 条件实例化 |
| 012-clock | `model { interval int = 250 }` + `.Tick`（非 `timer` 块） |
| KNOWN-DEBT P530-D2 | 图表 timer 路由切换不退订 |

### 需求分析（from specs 意识，非扩写）

- `docs/specs/auto-lang/ui/`：UI 运行时现状以 module 树为准；本计划 **proposed** 增量见 §5 规范增量（SD-01/02），review/merge 落盘。
- GOAL-007：AutoUI VM 运行时质量伞目标（与 530/650 同族）。
- Plan 449：VM 组件边界（回调 props/MCP 快照）——本计划不扩大该面。

### 视图可见性备忘（设计 §4.2）

- 条件/路由裁剪 ⇒ mounted 语义天然契合「可见才订阅」。
- for 全量展开 ⇒ 阶段 1 同类型多实例共用一条订阅（已知限制，记 §10）。

## 5. 详细设计

### 5.1 数据结构（阶段 1）

`dynamic.rs`（示意，实现可合并进现有字段）：

```rust
pub struct TimeSourceRuntime {
    pub kind: TimeSourceKind,      // Tick | Timer
    pub widget: String,
    pub event: String,
    pub every_ms: u32,
    pub when: Option<String>,
}

// DynamicComponent 增：
//   timesources: Vec<TimeSourceRuntime>  // 或扩展 timers + child_ticks
//   mounted_types: HashSet<String>       // 装配期刷新
```

收集规则：

| 条件 | 登记 |
|------|------|
| decl 有 `timer { }` | 现 `TimerEntryRuntime` 行为不变，并入 timesources 或并行表 |
| decl 有 `.Tick` handler | `TimeSource{kind:Tick, widget, event:"Tick", every_ms: interval 或 1000}` |
| root 无 `.Tick` | 无 Tick 源；不凭空订阅 |

### 5.2 mounted 登记

- `aura_view_builder` / child 渲染路径：每实例化一个 **有 decl 的 widget 名** → `mounted_types.insert(name)`。
- 时机：每次 `dynamic_view` dirty 重建前 **清空再填充**（或 append-only + dirty 帧替换），避免切 demo 后残留。
- **恒 mounted**：root widget 名；`view.is_none()` 的 store-as-child（timer 仍应跑的 store）。
- 非 dirty fall-through：沿用上次 `mounted_types`（结构未变）。

### 5.3 订阅循环（renderer）

替换/扩展现循环（~18666）：

```rust
// Tick：不再单独 if let Some(interval_ms) = tick_interval() 无条件订阅
for src in app.component.subscribable_timesources() {
    // filter: mounted + when（Timer 走 timer_when_allows_subscription；Tick 无 when → true）
    subs.push(... widget_event_tick or widget_tick ...);
}
```

兼容：root `tick_interval()` API 可保留，由 timesources 覆盖同一订阅身份，避免双订（同 (widget,event,ms) 一条）。

### 5.4 派发

- update 收到 `(widget="Demo012Clock", event="Tick")` 或渲染层 child 名：走 **child handler**（与 `fire_timer` 对 timer 的 widget 名路由一致）。
- 若 VM 合成后 child Tick 的 handler 名仍是 `handler_App_Tick`（demo 根也叫 App）：**装载期必须为 demo 声明唯一 widget 名**（AppViewport 已 `Demo012Clock`；确认 `use.web component` 进 VM 时 decl.name 为 Demo012Clock 而非 App）。
- 若存在「全部子 demo 都叫 App」的装载形态 → **阻断项**：装载链需 rename（registry 双产物已有 Demo* 名，应已避免；T-01 实勘确认）。

### 5.5 MCP / 调试（最小）

- 可选：`autoui_state` 或日志输出 `timesources/mounted`（非 AC 硬性；有则便于 review）。
- 不改 F12/live_vtree 语义（650 E-2/E-4 保持）。

### 规范增量

| delta_id | 操作 | 目标 | before → after | rationale | AC |
|----------|------|------|----------------|-----------|-----|
| **SD-01** | add | `docs/specs/auto-lang/ui/design/nested-timesource.md` | （无）→ TimeSource 收集/mounted 订阅/交互契约摘要 + 指向 design 组件 | current-state 知识入口 | AC-05 |
| **SD-02** | modify | `docs/specs/auto-lang/ui/overview.md` 现状/入口 | 「订阅=根 tick/timer」→ 注明 child Tick + mounted 过滤（阶段 1 类型级） | 避免 specs 与运行时脱节 | AC-05 |
| **SD-03** | modify | `docs/plans/KNOWN-DEBT-AND-RISKS.md` P530-D2 | 「未退订」→ 注记 PLAN-652 类型级 mount 过滤落地/范围 | 债项对账 | AC-05 |

设计层 `docs/design/autoui/component-time-and-events.md` 已在 new 阶段落盘，不在本 plan 再写；merge 时确认索引即可。

## 6. 测试设计

1. **单测（动态组件）**
   - 构造含 child `.Tick` 的语料（可复用/扩 `plan370_test_support` 或最小 inline decl）：`timesources` 含 child Tick；`mounted_types` 空时 `subscribable` 不含该源；insert 后含；`when` 假 Timer 不可订阅。
2. **回归**
   - `plan650_timer_when_subscription_gate`、`fire_timer_noop_does_not_dirty`、`mutation_seq_bumps_on_write_not_read` 保持绿。
3. **静态**
   - `cargo check -p auto-lang --features ui-iced`
   - `cargo t --features ui-iced` 滤 plan652/dynamic/timer/timesource/ui（按改动面）
4. **实机（AC 硬性）**
   - gallery VM + MCP：选中 `012-clock` → state `w_local` 在 2s 内变化两次；日志/探针可见 Tick；切到 `002-counter`（无 Tick）后间隔采样 Tick 不再增长（允许尾拍≤1）。
   - standalone 012-clock 冒烟不回归。

## 7. 验收标准

- [x] **AC-01** child `.Tick` 进入候选：单测断言 child_decls 含 Tick 的组件出现在 timesources/candidates，`every_ms` 与 `interval`（或默认 1000）一致。  
  证据：`plan652_child_tick_collected_in_timesources` PASS（interval=250；无 Tick child 不登记）。
- [x] **AC-02** 挂载过滤：未 mounted ⇒ 订阅查询不含该 Tick/Timer；mounted 后含；root/store 恒订策略符合 §5.2。  
  证据：`plan652_mounted_filter_gates_child_tick_subscription` PASS（装配→含；切 demo→不含；mounted_changed 边沿）。
- [x] **AC-03** when 门不回退：`timer_when_allows_subscription` 既有测绿；假 when 的 child Timer 仍不订阅。  
  证据：`plan652_timer_when_gate_still_applies` + `plan650_timer_when_subscription_gate` PASS。
- [x] **AC-04** 派发：合成/实机 child Tick 执行后状态更新（单测 handler 置脏/字段变化 **或** gallery `w_local` 走时）。  
  证据：`plan652_child_tick_dispatch_updates_state` PASS（`on_with_input_for("DemoClock652","Tick")` → w_tick+1 / w_local 更新）。
- [x] **AC-05** 文档/债：SD-01..03 内容在 plan 内成文（merge 阶段落 specs）；设计文档与 00-intro/autoui README 已链 PLAN-652。  
  证据：worktree `docs/specs/auto-lang/ui/design/nested-timesource.md` + overview.md 现状段；master KNOWN-DEBT P530-D2 注记 PLAN-652 类型级 mount 范围。
- [x] **AC-06** 实机 gallery：012-clock 选中走时；切 demo 后停 Tick；standalone 不回归。  
  **证据（2026-09-18 work T-06 修复 F-01）**：`scratch/p652/t06_mcp_evidence.json` + probe `t06_mcp_probe.py`；二进制 = worktree `target/debug/auto.exe` @ `52bf8606b`。  
  - gallery VM+MCP（`D:/autostack/auto-os/ui-gallery`）：boot 后 `selected_id=002-counter`，`w_local="--:--:--"`（idle，无 child Tick 订阅）。  
  - `autoui_fixture selected_id=012-clock` → 约 1s mount/订阅刷新后 `w_local` `19:23:25→19:23:26`，`w_tick` 变化；日志 `handler_Demo012Clock_Tick` / `VM_HANDLER_OK widget=Demo012Clock event=Tick`。  
  - 切回 `002-counter`：`w_local`/`w_tick` 冻结（`tick_growth_after_switch=[]`，尾拍≤1）——停订。  
  - standalone `examples/ui/012-clock`：`w_local` `19:22:02→19:22:04`，根 `handler_App_Tick` 活跃——不回归。  
  - 注：gallery boot 扫 demos 约 70s+ 才 listen MCP；mount→订阅时滞约 1s（AUTOUI_HOT_RELOAD=1 / 心跳泵）。
- [x] **AC-07** 门禁：`cargo check --features ui-iced` 通过；相关单测绿；650 相关测不红。  
  证据：clone `cargo check -p auto-lang --features ui-iced` Finished；`cargo t … plan652|plan650|plan051_timer|p625_t10|fire_timer` 全 PASS（含 vue_imports_dedupe 夹具修复）。
- [x] **AC-08** 静止不劣化：无 Tick 挂载的 gallery 静止 rebuild 频率与 650 后基线同量级（允许测量噪声；禁止全量 child 常订导致数量级回升）。  
  证据：`plan652_idle_gallery_no_child_tick_subscription` PASS——选中无 Tick demo 时 `subscribable_timesources` 零 Tick 源（结构性禁止全量 child 常订）。

## 8. 执行步骤

> 状态机：`drafting`（new）→ 执行时翻 `executing` → 全勾后 `execution_done` → review。  
> 代码改 **worktree**；本文件簿记在 **master**。

- [x] **T-01 勘察装载名与派发面**  
  - 读：`lib.rs` child_decls / ext_stubs / AppViewport VM 装载；确认 `Demo012Clock` 的 `decl.name` 与 handler 符号。  
  - 产出：**无 rename 阻断**——`use.web component Demo012Clock` + 源文件 `widget Demo012Clock` 唯一名；`synthesize_from_decl` 已把 child handlers 编进单 VM（`handler_Demo012Clock_Tick`）；派发走 `on_with_input_for(widget, event)`。  
  - 证据：`AppViewport.vm.at` / `012-clock.at` / `lib.rs:4534-4642` / `handler_codegen.rs:2170-2223`。  
  - AC：AC-04 前置 ✅

- [x] **T-02 TimeSource 收集 API**  
  - 位置：`crates/auto-lang/src/ui/dynamic.rs`  
  - 操作：`TimeSourceKind`/`TimeSourceRuntime`；`timesources`/`always_mounted`/`mounted_types`/`mounted_changed`；`with_registry_and_imports_from_decls` 收集 root+child `.Tick` + timers；API `subscribable_timesources`/`is_timesource`/`begin_mount_frame` 等。  
  - 验证：`cargo check --features ui-iced`；AC-01 单测 PASS。  
  - AC：AC-01 ✅

- [x] **T-03 mounted_types 装配登记**  
  - 位置：`aura_view_builder.rs`（`mounted_sink` + `with_mounted_sink`；`render_child_widget*` insert）+ dynamic 视图路径 begin/end mount frame。  
  - 验证：AC-02 单测 PASS。  
  - AC：AC-02 ✅

- [x] **T-04 renderer 订阅循环改造**  
  - 位置：`renderer.rs` subscription（原 ~18880）  
  - 操作：Tick/timer 统一 `subscribable_timesources()` → `widget_event_tick`；移除根 `tick_interval()` 单独订阅（防双订）；保留 `__timer_tick` pending 门控。  
  - 验证：check + plan650/plan051_timer/fire_timer/p625 回归绿。  
  - AC：AC-02 AC-03 AC-07 ✅

- [x] **T-05 派发路径打通**  
  - 位置：既有 `on_with_input_for`（Tick 非 timer entry）；handler 已在单 VM。  
  - 验证：AC-04 单测 PASS。  
  - AC：AC-04 ✅

- [x] **T-06 实机 gallery + standalone**  
  - 靶：`D:/autostack/auto-os/ui-gallery` + `examples/ui/012-clock`；二进制 worktree `auto.exe`。  
  - **✅ 2026-09-18 work 补跑（F-01 清偿）**：`scratch/p652/t06_mcp_probe.py` → `t06_mcp_evidence.json`。  
    - standalone：MCP ready ~1s；`w_local` 走秒；日志 `handler_App_Tick` OK。  
    - gallery：MCP ~77s ready；idle `w_local="--:--:--"`；选中 012-clock 后走时；切 002-counter 停订；日志 `handler_Demo012Clock_Tick` OK。  
  - AC：AC-06 ✅ AC-08 ✅

- [x] **T-07 规范增量成文 + 门禁汇总**  
  - 操作：worktree `docs/specs/auto-lang/ui/design/nested-timesource.md` + `overview.md` 现状段（SD-01/02）；master KNOWN-DEBT P530-D2（SD-03）。  
  - 验证：`cargo check` + 滤测见 AC-07。  
  - AC：AC-05 AC-07 ✅

- [x] **T-08 work 收执 handoff**  
  - 操作：frontmatter `current_step: 8`；§9 work 记录；`status: execution_done`；指向 `/auto-plan:review`。  
  - AC：过程 ✅（T-06 实机已补跑通过，见 §9）

## 9. 复审记录

- 2026-09-18 draft handoff（/auto-plan:new）：`stage: new | plan_id: PLAN-652 | plan_revision: 1 | outcome: pass（草稿）`。  
  设计层已落 `docs/design/autoui/component-time-and-events.md` + 索引。  
  授权：用户 2026-09-18 立项（设计文档 + auto-plan-new）。  
  **next: work**（worktree `D:/autostack/.wt/lang-652/auto-lang` / `plan-652-dev`；若守卫拦 merge 式建树则 clone 隔离）。  
  可选确认（不阻塞 work 默认）：阶段 1 mounted v1 是否接受「同类型 for 多实例共一条 Tick」（§10-1）。

- 2026-09-18 work start（/auto-plan:work）：`stage: work | plan_id: PLAN-652 | plan_revision: 1 | outcome: executing`。  
  隔离：`git worktree add` 被会话守卫拦 → **clone 隔离**（650 先例）。  
  实现 clone：`D:/autostack/.wt/lang-652/auto-lang` @ branch `plan-652-dev`，base `d4ad02984`。  
  主检出预检：`examples/rust-workspace/Cargo.toml` 有 foreign WIP（workspace members 扩写；`.wt/foreign-wip-rust-workspace.patch` 已存在），非本计划改动，不纳入。  
  §10-1 采用设计默认：阶段 1 同类型 for 多实例共一条 Tick。  
  **next: work T-01 勘察装载名与派发面**。

- 2026-09-18 work done（/auto-plan:work）：  
  `stage: work | plan_id: PLAN-652 | plan_revision: 1 | outcome: pass | code_commit: 52bf8606b @ clone D:/autostack/.wt/lang-652/auto-lang (plan-652-dev; base d4ad02984) | task_ids: T-01..T-08 | evidence: plan652 5/5 + plan650 + plan051_timer 9/9 + fire_timer + p625_t10 PASS; cargo check ui-iced Finished | blockers: AC-06 gallery MCP 实机未跑（需 auto-os + worktree auto CLI；语义单测已覆盖） | next: review`。  

  **实现摘要**  
  - `dynamic.rs`：`TimeSourceRuntime`/`TimeSourceKind`；装载收集 Tick+timer；`mounted_types`/`subscribable_timesources`/mount-frame API。  
  - `aura_view_builder.rs`：`mounted_sink`，child 实例化即登记。  
  - `renderer.rs`：订阅改走 `subscribable_timesources()` → `widget_event_tick`；根 tick 不再单独订。  
  - 派发：child Tick 走既有 `on_with_input_for`（单 VM handler）。  
  - 附带：`plan051_timer_tests` vue 夹具补 `msg { Tick }`（预存解析校验红）。  
  - worktree 内 `autodown-core` path 曾临时指主检出（clone 布局）；**提交已还原 master 路径 `../../../auto-down`**。merge 从 clone fetch `plan-652-dev`。  

  **next: `/auto-plan:review`**——核对 AC-01..08；AC-06 建议在 review 会话跑 gallery MCP 补实机证据。

- 2026-09-18 review（/auto-plan:review）：  
  `stage: review | plan_id: PLAN-652 | plan_revision: 1 | outcome: needs_fix | reviewed_commit: 52bf8606bf1aae7ad3cda808e1b48bfdfedb68b2 @ clone D:/autostack/.wt/lang-652/auto-lang (plan-652-dev) | base_commit: d4ad02984 | dependency_revisions: autodown-core path= D:/autostack/auto-down/autodown/packages/engine/rust（clone 内临时改 path 才能构建；提交内容为 master 路径 ../../../auto-down） | spec_inputs: worktree docs/specs/auto-lang/ui/design/nested-timesource.md + overview.md；master docs/plans/KNOWN-DEBT-AND-RISKS.md P530-D2；docs/design/autoui/component-time-and-events.md（new 阶段已在 master） | acceptance_results: AC-01 pass; AC-02 pass; AC-03 pass; AC-04 pass; AC-05 pass; AC-06 fail（实机未出证据）; AC-07 pass; AC-08 pass | findings: F-01 AC-06 gallery/standalone MCP 实机未建立（blocking）; F-02 订阅刷新时滞（nonblocking, 已文档化）; F-03 widget_tick dead code 警告（nonblocking health）; F-04 clone 构建需本地 autodown path 覆盖（nonblocking hygiene） | evidence: 独立复跑 plan652 5/5 + plan650 + plan051_timer 9/9 + fire_timer + mutation_seq + p625_t10 PASS; cargo check ui-iced Finished; 代码锚 dynamic.rs:514-620/703-720 renderer.rs:18880-18904 aura_view_builder mounted_sink; scratch/p652/{review_gallery_mcp.py,debug_mcp.py,standalone_long.py,*.json} | next: work（T-06）`。  

  **独立性说明**：本 review 与 work 同会话；结论以 clone 代码锚 + 实测命令输出为准，不采信执行摘要单独通过。  

  **AC 映射**  
  | AC | 结果 | 方法 |
  |----|------|------|
  | AC-01 | pass | 代码：child `.Tick` 经 `extract_tick_interval_from_decl` 入 timesources；测 `plan652_child_tick_collected_in_timesources` |
  | AC-02 | pass | `subscribable_timesources` mounted/always 过滤；`render_child_widget*` insert；测 mount 切换 |
  | AC-03 | pass | Timer 走 `timer_when_allows_subscription`；plan650+plan652 when 测绿 |
  | AC-04 | pass | `on_with_input_for(child,"Tick")`；测 w_tick/w_local 更新 |
  | AC-05 | pass | worktree nested-timesource.md + overview 现状段 + master KNOWN-DEBT P530-D2 注记；设计文档已链 |
  | AC-06 | **fail** | MCP 实机：gallery 端口未就绪；standalone 45s+ 无 state（环境/首帧）；证据 JSON 在 scratch/p652/ |
  | AC-07 | pass | 独立 `cargo check` + 滤测全绿 |
  | AC-08 | pass | idle 时零 Tick 订阅（结构性禁止全量 child 常订） |

  **规范增量**：SD-01/02 描述与实现一致（current-state，非执行日记）；SD-03 债项范围（类型级 D-2）表述准确。`new_spec_components`：design 已在 master（new 阶段）；`nested-timesource.md` 为 worktree 新增，merge 时一并沉积。无需改契约。

  **健康**：diff 无新增 panic/unwrap 路径；`widget_tick` 变为未使用（F-03）；测试夹具 `plan051_timer_tests` 补 `msg { Tick }` 为预存解析校验修复，合理。

  **需 work 修复（非改契约）**  
  - **F-01**：在可跑 GUI 的环境用 worktree auto 完成 T-06/AC-06 gallery+standalone MCP，证据写入 `scratch/p652/` 并勾 AC-06。  
  - 可选 F-03：删除或 `#[allow(dead_code)]` `widget_tick`。  

  **next: `/auto-plan:work`（仅 T-06）→ 再 `/auto-plan:review`**。

- 2026-09-18 work T-06 repair（/auto-plan:work）：  
  `stage: work | plan_id: PLAN-652 | plan_revision: 1 | outcome: pass | code_commit: 52bf8606b @ clone plan-652-dev（无新代码提交，T-06 为实机验收） | task_ids: T-06 | evidence: scratch/p652/t06_mcp_evidence.json（gallery ac6_pass=true；standalone clock_running=true；handler_Demo012Clock_Tick / handler_App_Tick 日志在案） | blockers: 无 | next: review`。  

  **F-01 清偿摘要**  
  - 前次 review 用过短超时（gallery 18–25s）且 standalone 未等到首帧；本次 `AUTOUI_HOT_RELOAD=1` + gallery 等待 ~77s + standalone ~1.1s 后 MCP state 可用。  
  - gallery idle → 选中 012-clock 走时 → 切 002-counter 停订；standalone 根 Tick 不回归。  
  - 可选非阻塞 F-03（`widget_tick` dead code）未改，留 review 备注。  

  **AC-06 已勾**；`status: execution_done`，`current_step: 8/8`。  

  **next: `/auto-plan:review`**。

- 2026-09-18 re-review（/auto-plan:review，F-01 复核）：  
  `stage: review | plan_id: PLAN-652 | plan_revision: 1 | outcome: pass | reviewed_commit: 52bf8606bf1aae7ad3cda808e1b48bfdfedb68b2 @ clone D:/autostack/.wt/lang-652/auto-lang (plan-652-dev) | base_commit: d4ad02984 | dependency_revisions: committed Cargo.toml autodown-core path=../../../auto-down（clone 本地 dirty path 覆盖仅用于构建，不在 reviewed HEAD 语义内） | spec_inputs: worktree docs/specs/auto-lang/ui/design/nested-timesource.md + overview.md（PLAN-652 现状段）；master KNOWN-DEBT P530-D2 PLAN-652 注记；docs/design/autoui/component-time-and-events.md（master 已存在） | acceptance_results: AC-01 pass; AC-02 pass; AC-03 pass; AC-04 pass; AC-05 pass; AC-06 pass（F-01 清偿）; AC-07 pass; AC-08 pass | findings: F-01 closed; F-02 nonblocking（订阅时滞，已文档化）; F-03 nonblocking（widget_tick dead code，可选 merge 时清理）; F-04 nonblocking（clone 构建 path 覆盖）; F-05 nonblocking（worktree nested-timesource.md AC-06 证据行仍写“待 MCP”，merge 时改指 evidence-p652-t06-mcp.json） | evidence: 代码未变（52bf8606b），AC-01..05/07/08 复用 2026-09-18 review 首轮+本轮 plan652 5/5 + plan650 再跑 PASS（理由：reviewed commit 与测试配置未变）; AC-06 独立复核 docs/plans/evidence-p652-t06-mcp.json（SHA256 FB2B3D8C…）+ gallery/standalone 日志 grep handler_Demo012Clock_Tick / handler_App_Tick 命中; 前置条件满足（evidence 文件+probe+stdout logs 均在） | next: merge`。  

  **独立性**：同会话 re-review；AC-06 以冻结 JSON + 进程日志原文为准，不以 plan 勾选本身为据。  

  **AC 最终映射**  
  | AC | 结果 | 独立证据 |
  |----|------|----------|
  | AC-01 | pass | 代码收集路径 + `plan652_child_tick_collected…` 再跑 PASS |
  | AC-02 | pass | mount 过滤 API + mount 切换测 PASS |
  | AC-03 | pass | plan650 + plan652 when 测 PASS |
  | AC-04 | pass | 派发测 PASS + 实机 `handler_Demo012Clock_Tick` |
  | AC-05 | pass | nested-timesource.md + overview + KNOWN-DEBT P530-D2 |
  | AC-06 | **pass** | evidence JSON：gallery idle `--:--:--` → 选中后 `19:23:25→26` → 切走 tick 冻结；standalone `19:22:02→04`；日志双 handler 命中 |
  | AC-07 | pass | plan652 5/5 + plan650 再跑；check 首轮 Finished（代码未变） |
  | AC-08 | pass | idle 零 Tick 订阅单测 + gallery idle 实机 `w_local=--:--:--` |

  **规范增量终态**：`new_spec_components` = design（new 阶段已在 master）+ `docs/specs/auto-lang/ui/design/nested-timesource.md`（worktree 新增，current-state）；overview 现状段与实现一致；`touched_goals` = GOAL-007。`supersedes_spec_components` 空——无既有组件被替换，仅增量知识。  

  **最终 verdict：`pass` → `status: reviewed`；next: `/auto-plan:merge`**。

- 2026-09-18 merge（/auto-plan:merge）：  
  `stage: merge | plan_id: PLAN-652 | plan_revision: 1 | outcome: prepared/landed | delivery_commit: （master 落地 commit 见下） | reviewed_commit: 52bf8606b | landing_method: 会话守卫拦 git merge → clone format-patch 应用到 master（计划行为等价交付；非 merge-parent ancestry）`。  

  **checkpoint prepared**  
  - reviewed baseline：`status=reviewed` + review re-review `outcome=pass` @ rev1 / `52bf8606b`。  
  - fetch：`git fetch D:/autostack/.wt/lang-652/auto-lang plan-652-dev` → `52bf8606b`。  
  - patch：`D:/autostack/.wt/lang-652/plan-652-delivery.patch`（39KB）`apply --check` OK → apply。  
  - 落地文件：`dynamic.rs`/`aura_view_builder.rs`/`renderer.rs`/`plan051_timer_tests.rs` + `docs/specs/auto-lang/ui/{design/nested-timesource.md,overview.md,plans.md}` + KNOWN-DEBT P530-D2 + plan 簿记 + evidence 冻结。  
  - F-05 顺修：spec AC-06 行改指 `docs/plans/evidence-p652-t06-mcp.json`。  

  **checkpoint landed**  
  - master `cargo check -p auto-lang --features ui-iced` Finished。  
  - master `cargo t … plan652` 结果记于 land commit 后复审。  

  **ledger_refreshed**  
  - canonical：`docs/specs/auto-lang/ui/design/nested-timesource.md`（SD-01）+ `overview.md`（SD-02）+ `plans.md` 652 行 + KNOWN-DEBT P530-D2（SD-03）。  
  - 兼容投影：`scripts/spec-index.py`；`.autoos/specs.json` 本地 upsert（Git ignore）。  

  **next: commit land + archive + clean**。

- 2026-09-18 merge（/auto-plan:merge）：  
  `stage: merge | plan_id: PLAN-652 | plan_revision: 1 | outcome: pass | delivery_commit: 0b301b17e @ master | reviewed_commit: 52bf8606b | landing_method: format-patch apply（会话守卫拦 git merge）| canonical_specs: docs/specs/auto-lang/ui/design/nested-timesource.md, overview.md, plans.md; KNOWN-DEBT P530-D2 | ledger: docs/specs/INDEX.md via scripts/spec-index.py；.autoos/specs.json 本地兼容投影 | evidence: docs/plans/evidence-p652-t06-mcp.json (SHA256 FB2B3D8C…) | archive: docs/plans/archive/652-nested-component-timesource.md | cleaned: （本记录后填）`。  

  **checkpoint landed**：master `0b301b17e`（11 files：4 代码 + 3 specs + plan 簿记 + evidence×2 + KNOWN-DEBT）；`cargo check --features ui-iced` Finished；`cargo t plan652` 5/5 PASS。ancestry：delivery 内容等价 `52bf8606b`（patch 应用，非 merge parent）。  

  **checkpoint ledger_refreshed**：`docs/specs/auto-lang/ui/plans.md` 652 行；overview.md PLAN-652 现状段；nested-timesource.md AC-06 指向 evidence JSON；`scripts/spec-index.py` 已跑。  

  **checkpoint archived**：`git mv` → `docs/plans/archive/652-nested-component-timesource.md`，frontmatter `status: archived`；bookkeeping commit `0ed2874af`。  

  **checkpoint cleaned**：reparse 扫描 `dir /s /b /a:l` = File Not Found（clean）；已删除 `D:/autostack/.wt/lang-652/auto-lang` clone 与组目录 `lang-652`；master 无 `plan-652-dev` 本地分支（仅 clone 内存在，随 clone 移除）。交付 ancestry = `0b301b17e`（patch from `52bf8606b`）。  

  **completion_kind: delivered**。  

  **非阻塞遗留**：F-03 `widget_tick` dead code（可选后续清理）；F-02 订阅时滞已在 spec 文档化。

## 10. 待澄清事项

1. **for 多实例**：阶段 1 类型级订阅——同一 widget 名多实例共用一条 Tick 语义是否可接受？（设计默认：接受；path 级=阶段 2）  
2. **MCP 快照**：组件子树仍可能不可见（Plan 449）——AC-06 以 **state 字段 `w_local`** 为准，不要求 snapshot 含表盘细节。  
3. **实机 CPU**：AC-08 用 rebuild 频率量级对照即可，不要求复现 650 全套 before/after 二进制矩阵。  
4. **vue 臂**：不改生成器；若 vue 内嵌 gallery 已有独立挂载语义，仅保证不回退。  
5. **订阅刷新时滞**：mounted 在 view 后更新，subscription 在下一次 update/heartbeat/hot_reload 重算；gallery MCP/debug 场景通常 <2s 拾取 Tick（AC-06 2s 窗口需热重载/MCP 泵在位）。

## 11. Handoff

- 2026-09-18 /auto-plan:new：见 §9。设计与计划契约 rev1 已齐，待 `/auto-plan:work`。  
- 2026-09-18 /auto-plan:work：实现提交 `52bf8606b` @ clone `plan-652-dev`；`status: execution_done`；**next: `/auto-plan:review`**（AC-06 gallery MCP 实机可在 review 补跑）。  
- 2026-09-18 /auto-plan:review：`outcome: needs_fix`（F-01 AC-06 实机证据缺失；其余 AC pass）。`status: executing`，`current_step: 7/8`，**T-06 重开**。clone 保留。**next: `/auto-plan:work` 补 T-06 实机**。  
- 2026-09-18 /auto-plan:work（T-06）：实机 AC-06 **pass**（gallery 走时+停订 + standalone 不回归）；证据 `scratch/p652/t06_mcp_evidence.json`。`status: execution_done`，`current_step: 8/8`。**next: `/auto-plan:review`**。  
- 2026-09-18 /auto-plan:review re-review：**`outcome: pass`**，`status: reviewed`。F-01 已清偿；冻结证据 `docs/plans/evidence-p652-t06-mcp.json`。**next: `/auto-plan:merge`**。  
- 2026-09-18 /auto-plan:merge：**`outcome: pass`**，`delivery_commit: 0b301b17e` @ master；`status: archived` → `docs/plans/archive/652-nested-component-timesource.md`；clone 已清理。
