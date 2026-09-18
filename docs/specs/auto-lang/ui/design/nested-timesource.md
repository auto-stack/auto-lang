# nested-timesource（PLAN-652 阶段 1 current-state）

> 设计层：[component-time-and-events](../../../../design/autoui/component-time-and-events.md)
> 计划：阶段 1 `docs/plans/archive/652-nested-component-timesource.md`；
> **阶段 2+3** `docs/plans/654-nested-timesource-path-and-clock.md`（drafting）
> 状态：阶段 1 已 merge；path 级订阅与 Clock 服务见 PLAN-654

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
- `subscribable_timesources()` — mounted + when 过滤后的可订阅源
- `is_timesource(widget, event)`
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

1. **类型级订阅**：同 widget 名 for 多实例共用一条 Tick（阶段 2 path 级）。
2. **订阅刷新时滞**：mounted 变化发生在 view 之后；下一次 update/heartbeat/hot_reload 重算订阅。gallery MCP/热重载场景通常 <2s。
3. MCP 快照对组件子树可见性仍受 Plan 449 限制；AC 以 state 字段为准。

## 验收映射

| AC | 证据 |
|----|------|
| AC-01 child Tick 进候选 | `plan652_child_tick_collected_in_timesources` |
| AC-02 挂载过滤 | `plan652_mounted_filter_gates_child_tick_subscription` |
| AC-03 when 门 | `plan652_timer_when_gate_still_applies` + `plan650_timer_when_subscription_gate` |
| AC-04 派发 | `plan652_child_tick_dispatch_updates_state` |
| AC-06 实机 gallery | `docs/plans/evidence-p652-t06-mcp.json`（gallery idle→选中走时→切 demo 停订；standalone 不回归；`handler_Demo012Clock_Tick`） |
| AC-08 静止不劣化 | `plan652_idle_gallery_no_child_tick_subscription` |

## 债项对账

- P530-D2：类型级 mount 过滤落地（PLAN-652 阶段 1）；**path 级/精确实例退订 → PLAN-654 阶段 A**。
- PLAN-650 E-1：`timer_when_allows_subscription` 保持，与 mount 过滤正交叠加。
- 阶段 3 Clock / `__wm_clock` 对齐：PLAN-654 阶段 B。
