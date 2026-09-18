# clock-service（PLAN-654 阶段 B current-state）

> 设计层：[component-time-and-events](../../../../design/autoui/component-time-and-events.md) §4.3
> 计划：`docs/plans/654-nested-timesource-path-and-clock.md` 阶段 B
> 状态：C1 文档 + C2 store 注入已实现（plan-654-dev / 待 fold）

## 能力总览

| 入口 | 适用 | 语义 |
|------|------|------|
| **`Time.now_sec()` / `Time.now_ms()` / `Time.now()`** | 任意 VM handler / Init | 拉取当前墙钟（stdlib `shim_time_now_sec`）；**不**触发视图重建 |
| **`__clock_now_sec` / `__clock_hhmm`** | 视图 / computed / 需要「随钟刷新 UI」的组件 | 框架注入根态；**仅当**视图或 computed 引用 `__clock_` 时才订 **1Hz** `__clock_tick` |
| **`.Tick` + `interval`** | 动画 / 业务节奏 / 自定频 | 阶段 2 path 级订阅；**与 Clock 服务并存**，互不替代 |
| **desktop `__wm_clock` / `__wm_date`** | desktop shell App | Plan 497：**分钟级** ServiceTick 注入，dock 状态栏 |

## 推荐用法（应用作者）

### Handler 里要「现在几点」

```text
fn show_now() {
    .now_sec = Time.now_sec()
    // 或本地展示串自行格式化
}
```

不要为了「知道时间」去订 4Hz `.Tick`。

### 视图里要显示墙钟且不自泵

```text
widget StatusClock {
    view {
        col {
            text .__clock_hhmm          // 框架 1Hz 刷新
            // text "${.__clock_now_sec}"
        }
    }
}
```

- 视图引用 `__clock_` → 框架自动订 `__clock_tick`（1000ms）并写字段。
- **未引用** `__clock_` 的组件 **零** 额外时钟泵（静止不劣化）。
- Init 前框架会 `seed_framework_clock()`，首帧即可读。

### 仍要平滑秒针 / 业务定时

继续用 `.Tick` + `Time.now_sec()`，或 `timer { … }`。Clock 服务 **不**取代业务 timer。

## 与 `__wm_clock` 对照

| 维度 | `__clock_*`（PLAN-654 B） | `__wm_clock`（Plan 497） |
|------|---------------------------|---------------------------|
| 作用域 | 任意 VM UI App | 仅 desktop shell |
| 节拍 | 1s | 1min（HH:MM） |
| 订阅门控 | 视图引用 `__clock_` | shell ServiceTick 帧泵内变化才写 |
| 字段 | `__clock_now_sec` int + `__clock_hhmm` str | `__wm_clock` str + `__wm_date` str |
| 目标 | 应用 UI 免自泵读秒/分 | dock 分钟时钟 |

Standalone **可用等价物**：直接读 `__clock_*`（本设计）或 handler 内 `Time.now_sec()`。

## 实现锚（`crates/auto-lang/src/ui/`）

| 符号 | 位置 |
|------|------|
| `DynamicComponent::CLOCK_*` 常量 | `dynamic.rs` |
| `wants_framework_clock` / `write_framework_clock` / `handle_clock_tick` / `seed_framework_clock` | `dynamic.rs` |
| `VmBridge::write_or_insert_state` | `vm_bridge.rs`（缺字段追加） |
| 订阅门控 1Hz `__clock_tick` | `iced/renderer.rs` subscription 循环 |
| update 臂刷新字段 + dirty | `iced/renderer.rs` `__clock_tick` |

## 迁移指引（从自泵 Tick → Clock）

1. **只显示时间、无动画**：视图改读 `. __clock_hhmm` / `. __clock_now_sec`，删掉该组件的 `.Tick`。
2. **handler 计算**：改调 `Time.now_sec()`，不必依赖 Tick 才能取时间。
3. **秒针动画 / 倒计时节奏**：保留 `.Tick`；可与 `__clock_*` 并用。
4. **desktop dock**：继续读 `__wm_clock`（分钟）；应用窗内用 `__clock_*`。

## 测试

- `plan654_framework_clock_fields_and_gate` — 字段可写可读 + wants 门控
- `plan654_clock_does_not_break_self_tick_component` — `.Tick` 组件兼容
- plan652 / plan650 回归 — 阶段 A/B 不破坏既有订阅

## 非目标

- 不强制迁移全部示例到 Clock（只提供能力 + 指南）。
- vue 轨：`Time.now_sec` 已桥 `Date.now()`；`__clock_*` store 注入 vue 对齐不在阶段 B 验收（VM/iced 为准）。
- 不替代 timer/业务调度。
