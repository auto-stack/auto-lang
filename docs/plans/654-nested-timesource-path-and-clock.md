---
plan_id: PLAN-654
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: nested-timesource-path-and-clock
author: [agent]
created_at: 2026-09-18
updated_at: 2026-09-18
plan_revision: 1
current_step: 0
total_steps: 12

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-lang/ui/design/nested-timesource.md  # 阶段 2/3 增量（modify existing）
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

- [ ] **AC-A1** `InstancePath`（或等价）进运行时模型；单测证明同 widget 多 path 可区分。
- [ ] **AC-A2** path 级 mount：装配登记 mounted paths；实例消失后 **不再订阅** 该 path（单测）。
- [ ] **AC-A3** 订阅 identity 含 path：同 widget 两 path 可同时出现在 subscribable 列表。
- [ ] **AC-A4** 派发：消息含 path；路由到 handler（或类型级+path 可观测）；**状态语义**（隔离或不隔离）有测试或文档钉住。
- [ ] **AC-A5** when 门与 652/650 回归绿。
- [ ] **AC-A6** 可观测：MCP/state/debug 至少一种可列出 timesources/mounted/subscribable（含 path）。
- [ ] **AC-A7** SD-A01..03 内容成文（merge 落 specs）；design §7 阶段 2 状态更新。
- [ ] **AC-A8** 门禁 check + 滤测通过；gallery 012-clock 单实例回归不红。

### 阶段 B

- [ ] **AC-B1** Clock 能力落地（C2/C3 之一）**或** C1 文档+决策记录「暂缓新 API」有用户可见理由（非空话）。
- [ ] **AC-B2** standalone 可消费墙钟（API/字段/文档可执行说明）。
- [ ] **AC-B3** SD-B01/B02 + 与 `__wm_clock` 对照文档；`.Tick` 兼容不回归。
- [ ] **AC-B4** 阶段 B 门禁绿；阶段 A 验收证据仍有效。

> **阶段门**：A 的 AC 全过才开 B 的实现任务（允许提前写设计注记）；B 不得静默删减 A 的 AC。

## 8. 执行步骤

> 多阶段：worktree 全生命周期一个 `plan-654-dev`；**阶段 A 完成且门禁绿后**按 AGENTS fold 到 master 并 re-sync，再执行 B（防与并行 master 漂移）。簿记在 master plan 文件。

### 阶段 A

- [ ] **T-A01 实勘 path 体系**  
  - 读：`aura_view_builder` child/for 渲染、`vnode.rs` path、`mounted_sink` 写入点、`AppTickKind::WidgetEvent`。  
  - 产出：path 形态裁定（复用何种 id）+ 派发是否可达 per-instance state 的结论 → §10/§9 注记。  
  - AC：AC-A1 前置  

- [ ] **T-A02 TimeSource path 模型**  
  - `dynamic.rs`：path 类型、mounted_paths、subscribable_instance_*、兼容 API。  
  - 验证：单测 AC-A1/A3。  

- [ ] **T-A03 装配登记 path**  
  - `aura_view_builder.rs`：实例化时写 path；begin/end mount 帧语义扩展。  
  - 验证：单测 AC-A2。  

- [ ] **T-A04 订阅 recipe 扩展**  
  - `renderer.rs` + subscription identity：含 path；避免双订。  
  - 验证：AC-A3 + 回归 AC-A5/A8。  

- [ ] **T-A05 派发路由**  
  - update 侧消费 path；文档化状态语义。  
  - 验证：AC-A4。  

- [ ] **T-A06 可观测**  
  - MCP/state/debug 列表（最小面）。  
  - 验证：AC-A6。  

- [ ] **T-A07 阶段 A 规范草稿 + 门禁**  
  - SD-A01..03；check+滤测；gallery 回归。  
  - AC：AC-A5/A7/A8  

- [ ] **T-A08 阶段 A 收执**  
  - §9 A 段 handoff；**建议** fold master + re-sync 后开 B（若用户要求一气呵成到 review，须在 handoff 写明未 fold 风险）。  

### 阶段 B（A 完成后）

- [ ] **T-B00 Clock 选型决策**  
  - C1/C2/C3 裁定 + 与 `__wm_clock` 差异表；写入 §9/§10。  

- [ ] **T-B01 Clock 实现**  
  - 按 T-B00 落地最小 API/字段/文档链。  
  - 验证：AC-B1/B2。  

- [ ] **T-B02 示范与兼容**  
  - 消费说明/片段；`.Tick` 回归。  
  - AC：AC-B3  

- [ ] **T-B03 阶段 B 文档 + 门禁 + 全量收执**  
  - SD-B01/B02；AC-B4；status→execution_done；next review。  

## 9. 复审记录

- 2026-09-18 draft handoff（/auto-plan:new）：`stage: new | plan_id: PLAN-654 | plan_revision: 1 | outcome: pass（草稿）`。  
  上游：PLAN-652 archived（delivery `0b301b17e`）；设计 component-time-and-events §7 阶段 2/3。  
  授权：用户 2026-09-18 指定阶段 2+3 **同一计划**立项。  
  **next: work**（先 T-A01 实勘 path；worktree `D:/autostack/.wt/lang-654/auto-lang` / `plan-654-dev`）。

## 10. 待澄清事项

1. **实例状态隔离**：若 VM 仅类型级 handler，阶段 A 是否仍算「多实例完成」？  
   - **默认**：生命周期（分订阅/退订）达标即可；**per-instance state** 若不可达则 AC-A4 以「可观测 path + 文档化限制」通过，并开债项。用户若要求「双钟各走各的」须在 work 前明示（可能扩大范围）。  
2. **Clock 默认节拍**：standalone 墙钟刷新 1s 还是分钟投影？T-B00 给默认并记理由。  
3. **订阅刷新时滞**（652 F-02）：阶段 A 是否顺手做 mount 变更即时重订阅？  
   - **默认**：不强制；若 T-A04 成本低可做，否则保持文档化时滞。  
4. **vue 双端 Clock**：阶段 B 默认 VM 优先；vue 对齐不阻塞 AC-B*。

## 11. Handoff

- 2026-09-18 /auto-plan:new：见 §9。阶段 A/B 任务与 AC 已挂设计节号；待 `/auto-plan:work`。
