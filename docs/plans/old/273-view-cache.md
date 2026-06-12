# Plan 273: AutoUI View Cache — 跳过无变化帧的重建

> **Status: ✅ 已完成** — `view_dirty` 标记 + cached AbstractView 复用已实现在 `crates/auto-lang/src/ui/iced/renderer.rs` 中。

## Context

AutoUI 的 `dynamic_view()` 每帧都无条件重建整个 AbstractView 树：
```
component.view() → inject_todo_list → patch_input_values → convert_view_messages → render_dynamic_view
```
即使没有任何状态变化（如 500ms 热重载 tick 检测文件未变时），也会执行完整的 O(n) 遍历。

**已有但未使用的基础设施**：`DynamicComponent.dirty` 标志在 `on()`、`write_state()`、`reload()` 后设为 true，但 `is_dirty()` 从未被调用，dirty 从未清除。

**目标**：当状态未变化时，跳过 AbstractView 重建（节省约 30-40% CPU），只保留 `render_dynamic_view()`（iced Element 创建，60-70% 开销，无法缓存因为 `iced::Element` 不是 Clone）。

## 方案：DynamicState 级别的 view_dirty + cached_converted_view

在 `DynamicState` 上加两个 `RefCell` 字段，利用已有的内部可变性模式：

- `view_dirty: RefCell<bool>` — 状态是否变化，需要重建 View
- `cached_converted_view: RefCell<Option<AbstractView<IcedMessage>>>` — 缓存已转换的 View

### 流程

```
update() 中状态变化 → view_dirty = true
                    ↓
dynamic_view():
  view_dirty == true?  → 重建 pipeline → 存入 cache → view_dirty = false → render
  view_dirty == false? → 用 cached_converted_view → patch input_values → render
```

### 500ms tick 的节省

每 500ms 触发 `HOT_RELOAD_EVENT`：
- 文件未变 → `check_file_changed()` 返回 None → 无状态变化 → `view_dirty` 保持 false
- `dynamic_view()` 用缓存 → **跳过 AuraViewBuilder::build + convert_view_messages**

## 实施步骤

### Step 1: DynamicState 添加缓存字段

文件：`crates/auto-lang/src/ui/iced/renderer.rs` — `DynamicState` 结构体

```rust
/// Whether the AbstractView needs rebuilding (set in update, cleared in dynamic_view).
view_dirty: std::cell::RefCell<bool>,
/// Cached converted view tree, reused when view_dirty is false.
cached_converted_view: std::cell::RefCell<Option<crate::ui::view::View<IcedMessage>>>,
```

初始化：`view_dirty: true`, `cached_converted_view: None`

### Step 2: update() 中标记 view_dirty

在所有修改状态的地方添加 `*state.view_dirty.borrow_mut() = true;`：

- `component.on_with_input()` 之后（用户事件）
- `component.reload()` 之后（热重载）
- `component.on_with_input("Tick", None)` 之后（定时器 tick）
- `todos` 变更后

**不标记 view_dirty 的情况**（无需重建 View）：
- DevTools tab 切换
- DevTools 面板打开/关闭
- Hover 事件
- F12 切换（叠加层在 render_dynamic_view 中独立应用）
- 500ms 热重载 tick 但文件未变

### Step 3: 重写 dynamic_view() 使用缓存

```rust
fn dynamic_view(state: &DynamicState) -> iced::Element<'_, IcedMessage> {
    // ... hover resolution (不变) ...

    let converted = {
        let mut dirty = state.view_dirty.borrow_mut();
        if *dirty {
            // 状态变化 → 重建
            let mut view = state.component.view();
            inject_todo_list(&mut view, &state.todos, state.component.widget_name());
            if !state.input_values.is_empty() {
                patch_input_values(&mut view, &state.input_values);
            }
            let converted = convert_view_messages(view);
            *state.cached_converted_view.borrow_mut() = Some(converted.clone());
            *dirty = false;
            converted
        } else {
            drop(dirty);
            // 无变化 → 用缓存（但仍需 patch input 值）
            let mut cached = state.cached_converted_view.borrow_mut();
            if let Some(ref mut view) = *cached {
                if !state.input_values.is_empty() {
                    patch_input_values_iced(view, &state.input_values);
                }
                view.clone()
            } else {
                // 首帧兜底
                let mut view = state.component.view();
                inject_todo_list(&mut view, &state.todos, state.component.widget_name());
                let converted = convert_view_messages(view);
                *cached = Some(converted.clone());
                converted
            }
        }
    };

    // ... console sync, debug_ctx, render_dynamic_view (不变) ...
}
```

### Step 4: 添加 patch_input_values_iced() 辅助函数

与 `patch_input_values` 结构相同，但操作 `AbstractView<IcedMessage>`。
因为缓存的 View 是 `IcedMessage` 类型，需要匹配 IcedMessage 的事件名提取。

### Step 5: 热重载后清除缓存

`apply_edit()` 和 `HOT_RELOAD_EVENT` 中 `reload()` 后：
```rust
*state.cached_converted_view.borrow_mut() = None;
*state.view_dirty.borrow_mut() = true;
```

## 文件修改清单

| 文件 | 改动 |
|------|------|
| `crates/auto-lang/src/ui/iced/renderer.rs` | 添加 `view_dirty` + `cached_converted_view` 字段、初始化、update 中标记、dynamic_view 缓存逻辑、`patch_input_values_iced` |

**不修改的文件**：`dynamic.rs`（dirty 标志保持不变）、`view.rs`、`aura_view_builder.rs`、`component.rs`

## 边缘情况

1. **首帧**：`view_dirty=true`, `cache=None` → 正常重建并缓存
2. **输入框打字**：`on_input` → update 设置 `view_dirty=true` → 重建（必须，因为 input 值变了）
3. **秒表 tick**：tick 事件确实改变状态 → `view_dirty=true` → 正确重建
4. **F12 切换**：不设 `view_dirty` → 用缓存的 AbstractView → `render_dynamic_view` 根据 debug_mode 独立决定是否包装 debug overlay
5. **热重载文件变更**：`reload()` → `view_dirty=true` + 清缓存 → 正确重建

## 验证步骤

1. `cargo build --bin auto --features ui-iced`
2. 运行 UI 示例 → 正常显示
3. F12 打开 → 点击元素 → Inspector 正常
4. 不操作时观察 CPU 占用（应比之前低，因为 500ms tick 不再触发完整重建）
5. 拖拽窗口流畅度验证
