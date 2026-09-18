# nested-timesource（PLAN-652 阶段 1 + PLAN-654 阶段 2 current-state）

> 设计层：[component-time-and-events](../../../../design/autoui/component-time-and-events.md)
> 计划：阶段 1 `docs/plans/archive/652-nested-component-timesource.md`；
> **阶段 2+3** `docs/plans/archive/654-nested-timesource-path-and-clock.md`（archived）
> 状态：阶段 1+2 已 fold master；path 级订阅 current-state 见下；Clock 见 [clock-service.md](clock-service.md)

## 问题（已定罪）

gallery VM 内嵌 012-clock 时钟冻结：`.Tick`+`interval` **仅根组件**进入 iced 订阅；
子组件时间源未登记。timer 虽静态收集 child_decls，但不看是否仍在树上（P530-D2）。

## 阶段 1 契约

### TimeSource 收集（装载期）

| 来源 | 登记 |
|------|------|
| decl 有 `.Tick` handler | `TimeSource{kind:Tick, widget, event:"Tick", every_ms}`；`interval` 缺省 1000 |
| decl 有 `timer { }` | 与既有 `TimerEntryRuntime` 对齐，并入 timesources |
| root / store-as-child | `always_mounted`（恒挂载） |

API（`crates/auto-lang/src/ui/dynamic.rs`）：

- `timesources()` — 候选表
- `subscribable_timesources()` — mounted + when 过滤后的可订阅源（**类型级**，阶段 1 语义；同 widget 多实例聚合为一条）
- `is_timesource(widget, event)` — path 装饰名自动剥 `@seq`
- `begin_mount_frame` / `end_mount_frame` / `mounted_types` / `take_mounted_changed`

### 挂载过滤（装配期）

- `AuraViewBuilder.mounted_sink`：`render_child_widget*` 实例化 registry child 时 insert 名。
- 视图重建前 `begin_mount_frame` 清空动态 mounted（保留 always）；结束后比较变更。
- 条件分支切走 ⇒ 该 widget 名不在 mounted ⇒ **不订阅**（类型级 D-2）。

### 订阅（调度期，iced renderer）

```text
for src in component.subscribable_timesources():
  Tick  → widget_event_tick(app, widget, "Tick", ms)  // 仍过 dashboard_hatched_tick_allowed
  Timer → widget_event_tick(app, widget, event, ms)   // when 已在 API 层过滤
```

- 根 `tick_interval()` **不再单独订阅**（避免与 root Tick 源双订）。
- 订阅身份 = `(widget, event, ms)`（`AppTickKind::WidgetEvent`）。

### 派发（update）

- Tick 非 timer 条目 → `on_with_input_for(widget, "Tick")` → 单 VM `handler_<W>_Tick`。
- 子 model 字段已并入根态（Plan 320）；MCP/state 可读 `w_local` 等。

### 已知限制（阶段 1）

1. **类型级订阅**：同 widget 名 for 多实例共用一条 Tick（→ **阶段 2 已解除**）。
2. **订阅刷新时滞**：mounted 变化发生在 view 之后；下一次 update/heartbeat/hot_reload 重算订阅。gallery MCP/热重载场景通常 <2s。
3. MCP 快照对组件子树可见性仍受 Plan 449 限制；AC 以 state 字段为准。

## 阶段 2 契约（PLAN-654 阶段 A，plan-654-dev）

### InstancePath

- 形态：`InstancePath(String)`
  - root / store / always_mounted：`{widget}`（与阶段 1 类型键同形）
  - 子组件实例：`{widget}@{mount_seq}`，`mount_seq` = 装配帧内按 widget 类型的 0-based 实例序号
- **不**另发明平行 id 体系；debug/MCP 可叠加既有 `id_from_path(view_path)`。
- API：`InstancePath::of_type` / `instance` / `widget_type` / `mount_seq` / `is_instance`

### path 级 mounted

- `DynamicComponent`：`mounted_paths` / `always_mounted_paths` / `mount_type_seq`
- `mount_path_sink() -> MountPathSinkRef`：builder 装配期 `register(widget) -> InstancePath`
- `AuraViewBuilder.mount_path_sink`：`render_child_widget*` 与类型级 sink **并行**写入
- `begin_mount_frame` 同时重置 path 集合与 per-type seq

### path 级订阅

```text
for src in component.subscribable_instance_timesources():
  // path = {widget} 或 {widget}@{seq}
  widget_event_tick(app, path.as_str(), event, ms)
```

- API：`subscribable_instance_timesources() -> Vec<InstanceTimeSource>`
- 同 widget 多 mounted path → 多条订阅（iced hash 区分）
- 类型级 `subscribable_timesources()` **保留**作兼容包装（聚合 unique widget）
- when 门仍读根态（阶段 A **不** per-path when）
- 兼容回退：仅类型 mounted、path 表未登记 → 类型级 path 一条

### path 级派发

- `IcedMessage.widget` 携带 InstancePath 串（`Demo@0`）
- `on_with_input_for` / `fire_timer` / `is_timer_entry` / `is_timesource` 解析 `@seq` → 类型名
- handler 仍为 `handler_<Widget>_<Event>`，**单 VM 根态**（Plan 320）

### 状态语义（显式限制）

> **path 级订阅 ≠ path 级状态隔离**。
> `ensure_child_state` 恒返回 root_id；child handler 路由 ROOT state。
> 同类型多实例共享模型字段；「双钟各走各的」**不在**阶段 A AC。
> 债项：path 级实例状态槽（单 VM 大题，另开计划）。

### 可观测

- `timesource_debug_rows()`：每行 `(path, widget, event, kind, every_ms, when, mounted, subscribable)`
- 测试：`plan654_*`（identity / multi-instance / unmount / when / type-fallback / debug-rows / dispatch / view-assembly）

## 验收映射

### 阶段 1

| AC | 证据 |
|----|------|
| AC-01 child Tick 进候选 | `plan652_child_tick_collected_in_timesources` |
| AC-02 挂载过滤 | `plan652_mounted_filter_gates_child_tick_subscription` |
| AC-03 when 门 | `plan652_timer_when_gate_still_applies` + `plan650_timer_when_subscription_gate` |
| AC-04 派发 | `plan652_child_tick_dispatch_updates_state` |
| AC-06 实机 gallery | `docs/plans/evidence-p652-t06-mcp.json` |
| AC-08 静止不劣化 | `plan652_idle_gallery_no_child_tick_subscription` |

### 阶段 2（PLAN-654 阶段 A）

| AC | 证据（plan-654-dev） |
|----|---------------------|
| AC-A1 path 身份 | `plan654_instance_path_identity_forms` + `plan654_multi_instance_paths_expand_subscription` |
| AC-A2 path 挂载/退订 | `plan654_view_assembly_registers_paths` + `plan654_unmount_drops_instance_path` |
| AC-A3 订阅 identity 含 path | multi-instance expand + renderer loop 使用 `path.as_str()` |
| AC-A4 派发 + 状态语义 | `plan654_dispatch_is_type_level_state_with_path_identity`（钉住类型级状态限制） |
| AC-A5 when/652/650 回归 | `plan654_when_gate_survives_path_expansion` + plan652 5/5 + plan650 1/1 |
| AC-A6 可观测 | `timesource_debug_rows` + `plan654_debug_rows_include_path` |
| AC-A7 specs | 本文阶段 2 段（worktree 草稿；merge 时 deposit） |
| AC-A8 门禁 | `cargo check --features ui-iced` Finished；滤测 plan654/652/650/fire_timer 全绿 |

## 债项对账

- P530-D2：类型级 mount 过滤落地（PLAN-652 阶段 1）；**path 级实例退订已落地（PLAN-654 阶段 A）**。
- PLAN-650 E-1：`timer_when_allows_subscription` 保持，与 mount 过滤正交叠加。
- **path 级实例状态槽**：单 VM 统一根态，阶段 A 不承诺 per-instance state；债项另开。
- 订阅刷新时滞（652 F-02）：阶段 A 默认不顺手做即时重订阅；保持文档化时滞。
- 阶段 3 Clock / `__wm_clock` 对齐：见 [clock-service.md](clock-service.md)（PLAN-654 阶段 B）。
