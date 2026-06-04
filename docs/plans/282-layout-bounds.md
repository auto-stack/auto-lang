# Plan 282: MCP Snapshot 添加实际 Layout Bounds（Phase 2）

## Context

Plan 281（Phase 1）已实现：从 Tailwind class 字符串提取 **期望的** 布局属性（padding、margin、width、max-w 等）并显示在 MCP snapshot 中。但 AI 只能看到"期望值"，无法看到 iced 实际渲染的布局矩形。

**核心问题**：014-weather 的 `center` 容器应该让内容居中，`col` 设置了 `w-full max-w-md`，期望宽度 448px 并水平居中。但实际渲染时元素挤在左侧。MCP 只显示 `[w=full(1600) max-w=448]`（期望值），AI 无法发现 iced 实际给了 col 多少宽度、是否居中。

**目标**：在 MCP snapshot 中为每个组件显示 iced **实际渲染的** layout 矩形 `[x, y, width, height]`，让 AI 能对比期望值 vs 实际值，发现布局问题。

## 技术方案

### 方案选择

| 方案 | 优点 | 缺点 |
|------|------|------|
| A: 按遍历顺序收集 bounds | 零 renderer 改动 | 顺序对齐不可靠（iced 树 ≠ AuraNode 树） |
| B: 给 container 设 ID + Operation 收集 | ID 可靠关联 | 只有 container 支持 `.id()`，col/row 不支持 |
| C: **给所有 widget 包 container 设 ID** | 所有 widget 都有 ID | 多一层 container 可能影响布局 |
| **D: 给 container 设 ID + 非容器 widget 用顺序** | 平衡可靠性和改动量 | 混合策略稍复杂 |

**选定方案 B**：给 `container` widget（包括 "center"）设置 iced `Id`，通过自定义 Operation 收集其 bounds。col/row 等非容器 widget 暂不收集 bounds——**container 级别的 bounds 已经足够诊断大部分布局问题**（如 014-weather 的居中问题）。后续需要时再扩展。

### 工作原理

1. `render_dynamic_view` 中的 `AbstractView::Container` 分支，在创建 iced `container` 时调用 `.id("aura_N")`
2. 自定义 `LayoutCollector` 实现 `iced::widget::Operation`，在 `container(id, bounds)` 方法中记录 ID → bounds 映射
3. update 函数中返回 `iced::Task::widget(collector)`，runtime 在 layout 计算后执行 operation
4. 收集结果通过特殊的 `IcedMessage` 回传给 update，存入 MCP `SharedState`
5. `AuraSnapshotBuilder` 读取 bounds 数据，在 snapshot 中输出 `@rect(x, y, w, h)`

### 数据流

```
dynamic_view() → render_dynamic_view()
  └→ container(child).id("aura_0").center_x(Fill)  // 设置 iced ID
  └→ wrap_debug() 中 mouse_area 包裹

update() 收到 __collect_bounds 消息
  └→ 返回 Task::widget(LayoutCollector)
  └→ iced runtime 执行 operation → 遍历 widget 树
  └→ container("aura_0", bounds=0,0,1600,900) 被记录
  └→ Operation::finish() 返回 HashMap<Id, Rect>
  └→ 通过 channel 发送结果 → 生成 __bounds_collected 消息

update() 收到 __bounds_collected
  └→ 存入 mcp_shared.layout_bounds

MCP tool_snapshot()
  └→ AuraSnapshotBuilder 读取 layout_bounds
  └→ 输出: center #aura_0 @rect(0, 0, 1600, 900) { ... }
```

## 实现步骤

### Step 1: 给 container widget 设置 iced ID

**修改**: `crates/auto-lang/src/ui/iced/renderer.rs`

在 `render_dynamic_view` 的 `AbstractView::Container` 分支中，当 debug_ctx 存在且有 AuraNodeId 时，调用 `c = c.id(format!("aura_{}", aura_id))` 给 iced container 设置 ID。

具体位置：在 `let el: iced::Element<...> = c.into();` 之前插入：
```rust
// Set iced widget ID for layout bounds collection (Plan 282)
if let Some(ctx) = debug_ctx {
    if let Some(aura_id) = ctx.debug_id_map.get(path) {
        c = c.id(iced::widget::Id::new(format!("aura_{}", aura_id.0)));
    }
}
```

同样在 `AbstractView::Scrollable` 分支中设置 ID（scrollable 也支持 `.id()`）。

### Step 2: 实现 LayoutCollector Operation

**新建文件**: `crates/auto-lang/src/ui/iced/layout_collector.rs`

```rust
use std::collections::HashMap;

/// 收集 iced container widget 的实际 layout bounds
pub struct LayoutCollector {
    bounds_map: HashMap<String, (f32, f32, f32, f32)>,
}

impl LayoutCollector {
    pub fn new() -> Self {
        Self { bounds_map: HashMap::new() }
    }
}

impl iced::widget::Operation<HashMap<String, (f32, f32, f32, f32)>> for LayoutCollector {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<HashMap<String, (f32, f32, f32, f32)>>)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&iced::widget::Id>, bounds: iced::Rectangle) {
        if let Some(id) = id {
            // 只收集以 "aura_" 开头的 ID
            if id.as_str().starts_with("aura_") {
                self.bounds_map.insert(
                    id.as_str().to_string(),
                    (bounds.x, bounds.y, bounds.width, bounds.height),
                );
            }
        }
    }

    fn finish(&self) -> iced::widget::Operation::Outcome<HashMap<String, (f32, f32, f32, f32)>> {
        iced::widget::Operation::Outcome::Some(self.bounds_map.clone())
    }
}
```

### Step 3: 在 update 函数中处理 bounds 收集

**修改**: `crates/auto-lang/src/ui/iced/renderer.rs`

在 update 闭包开头添加 needs_bounds 检查（详见 Step 5），以及在消息分发中添加 `__bounds_collected` 处理：
```rust
if msg.event == "__bounds_collected" {
    if let Some(json) = &msg.input_value {
        if let Ok(bounds_map) = serde_json::from_str::<HashMap<String,(f32,f32,f32,f32)>>(json) {
            if let Some(ref mcp) = state.mcp_shared {
                mcp.lock().unwrap().set_layout_bounds(bounds_map);
            }
        }
    }
    return iced::Task::none();
}
```

### Step 4: SharedState 添加 layout_bounds 存储

**修改**: `crates/auto-lang/src/ui/mcp_server.rs`

`SharedState` 添加字段和方法：
```rust
/// Actual layout bounds from iced renderer (Plan 282)
/// Key: widget ID like "aura_0", Value: (x, y, width, height)
layout_bounds: HashMap<String, (f32, f32, f32, f32)>,

pub fn set_layout_bounds(&mut self, bounds: HashMap<String, (f32, f32, f32, f32)>) {
    self.layout_bounds = bounds;
}

pub fn get_layout_bounds(&self) -> &HashMap<String, (f32, f32, f32, f32)> {
    &self.layout_bounds
}
```

### Step 5: 触发 bounds 收集的时机

**iced 执行循环**：`process_events → update() → view() → layout → process_actions(operate) → draw`

`Task::widget(op)` 产生 `Action::Widget`，在 action 处理阶段执行（当帧 layout 之后）。

**方案：自动收集 + 标志位**

1. `DynamicState` 添加 `needs_bounds: RefCell<bool>` 标志
2. `dynamic_view()` 中，当 view 重建后设置 `needs_bounds = true`
3. `update()` 函数**开头**检查并触发：
   ```rust
   if *state.needs_bounds.borrow() {
       *state.needs_bounds.borrow_mut() = false;
       return iced::Task::widget(LayoutCollector::new())
           .map(|bounds_map| IcedMessage {
               widget: String::new(),
               event: "__bounds_collected".to_string(),
               input_value: Some(serde_json::to_string(&bounds_map).unwrap_or_default()),
           });
   }
   ```
4. `update()` 中处理 `__bounds_collected`：反序列化 JSON → 存入 `mcp_shared.layout_bounds`

**时序**：view() 标记 → 下一帧 update() 触发 Task::widget → 当帧 layout 后 operation 执行 → channel 回传 → 下下一帧 update() 存入 SharedState。滞后 2 帧但对 MCP 完全可接受。

### Step 6: AuraSnapshotBuilder 集成 bounds 输出

**修改**: `crates/auto-lang/src/ui/aura_snapshot_builder.rs`

1. 添加 `layout_bounds: HashMap<String, (f32, f32, f32, f32)>` 字段
2. 添加 `with_layout_bounds()` builder 方法
3. 在 `traverse()` 的 `Element` 分支中，查找 `debug_id` 对应的 bounds：
   ```rust
   // 查找实际 layout bounds (Plan 282)
   let bounds_str = debug_id.as_ref().and_then(|id| {
       let aura_id = format!("aura_{}", id.trim_start_matches("aura_"));
       // 或者直接用 debug_id 作为 key
       layout_bounds.get(id).map(|(x,y,w,h)| format!("@rect({:.0},{:.0},{:.0},{:.0})", x, y, w, h))
   });
   ```
4. 插入到标签行：`center #aura_0 @rect(0,0,1600,900) {`

### Step 7: MCP tool_snapshot 传递 bounds

**修改**: `crates/auto-lang/src/ui/mcp_server.rs` 的 `tool_snapshot()`

从 `shared.layout_bounds` 读取，传给 `AuraSnapshotBuilder` 的 `with_layout_bounds()`。

## Snapshot 输出效果

```
AURA Snapshot v2
widget: "App"
viewport: 1600x900

state:
  ...

tree:
center #aura_0 @rect(0, 0, 1600, 900) {
  col #aura_1 @rect(0, 0, 1600, 900) [pad=24 w=full(1600) max-w=448 min-h=screen] {
    style: "w-full max-w-md p-6 bg-gray-50 min-h-screen"
    row #aura_2 [w=full(1600)] {
      ...
    }
  }
}
```

AI 可以立即看到：`center` 的 rect 是 `(0,0,1600,900)`，`col` 的 rect 也是 `(0,0,1600,900)`——宽度 1600 远超 max-w=448，说明 `max_width` 或 `center` 没有正确生效。

注意：`col` 和 `row` **不会**有 `@rect`，因为它们不是 iced container。只有 `center`（container）和 `scrollable` 会有 bounds。这对于诊断布局问题已经足够。

## 关键文件清单

| 文件 | 操作 | 说明 |
|------|------|------|
| `crates/auto-lang/src/ui/iced/layout_collector.rs` | **新建** | LayoutCollector Operation 实现 |
| `crates/auto-lang/src/ui/iced/renderer.rs` | 修改 | Container 设置 ID + update 触发 bounds 收集 |
| `crates/auto-lang/src/ui/mcp_server.rs` | 修改 | SharedState 添加 layout_bounds |
| `crates/auto-lang/src/ui/aura_snapshot_builder.rs` | 修改 | 输出 @rect() 信息 |

**参考文件（只读）**：
- `iced_core-0.14.0/src/widget/operation.rs` — Operation trait 定义
- `iced_widget-0.14.2/src/container.rs:280` — container 的 operate 实现（有 ID 支持）
- `iced_widget-0.14.2/src/column.rs:247` — column 的 operate（ID 硬编码 None）
- `iced_widget-0.14.2/src/mouse_area.rs:204` — mouse_area 透传 operate

## 验证方案

1. **编译**: `cargo build --bin auto`
2. **运行 014-weather**: `auto ./examples/ui/014-weather/src/front/app.at`
3. **MCP snapshot**: 确认输出包含 `@rect(...)` 信息
4. **诊断验证**: AI 能从 snapshot 中看出 center 的 bounds 不正确（宽度=窗口宽度而非受限宽度）
5. **对比验证**: 运行正常布局的示例（如 016-calendar），确认 bounds 数据合理
