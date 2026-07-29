# Plan 371: AutoUI MCP 功能大改进 — Agent 驱动的 UI 自动化验证

> **目标**: 让 Agent 通过 MCP 服务直接操纵 AutoUI 界面（类似 playwright-mcp），实现"开发→构建→启动→自动验证"的闭环。
>
> **核心挑战**: 当前 MCP 的 action/inspect 工具只覆盖根组件元素（`aura_N`），子组件内部元素（EditorPanel 的按钮、NavTree 的列表项）无法操作。需要统一 ID 系统为 `vnode_N`（覆盖全部渲染元素）。

---

## 1. 背景与问题

### 1.1 两套 ID 系统

| | `aura_N` | `vnode_N` |
|---|---|---|
| **类型** | `AuraNodeId(u32)` — 源码提取时静态计数器 | `VNodeId(u64)` — 渲染时路径 FNV-1a 哈希 |
| **覆盖范围** | 仅根组件直接子元素（~14个） | **全部**渲染元素（含子组件内部） |
| **用于** | action / inspect | vtree / snapshot |
| **子组件可见** | ❌ 不可见 | ✓ 可见 |

### 1.2 根因

`render_child_widget`（`aura_view_builder.rs:1350`）调用 `child_builder.build()`（untracked），不经过 `DebugIdMap`。子组件内部元素永远拿不到 aura ID。

### 1.3 具体表现

- `autoui_action(element_id="aura_N", action="press")` 只能点击根组件的按钮（如顶部 "New"）
- EditorPanel 的 Edit/Delete 按钮、NavTree 的 note 列表项没有 aura_N → 无法通过 MCP 操作
- `autoui_snapshot` 和 `autoui_vtree` 返回 `vnode_N`（覆盖全部），但 action/inspect 不认 vnode_N

---

## 2. 方案：action/inspect 支持 vnode_N（向后兼容 aura_N）

### 2.1 核心思路

View 树（`View<DynamicMessage>`）已经携带了完整的事件信息——每个 Button 的 `onclick` 是一个 `DynamicMessage { widget_name, event_name }`。VNode 的 `path` 字段（子索引序列）与 View 树结构一一对应。

**链路**: `vnode_N → VNode.path → 遍历 View 树找到对应 View → 提取 DynamicMessage（widget_name + event_name）→ 构建 ActionMessage`

不需要改渲染器、不需要改数据管道、不需要改 InspectorCache。

### 2.2 关键改进

vnode_N 路径直接从 DynamicMessage 获取 `widget_name`（如 "EditorPanel"），不再硬编码 `shared.widget_name`（"App"）。这修复了子组件按钮事件路由问题——之前即使能找到按钮，事件也会路由到 App 而非 EditorPanel。

---

## 3. 实施步骤

### 改动文件：`crates/auto-lang/src/ui/mcp_server.rs`

### Task 1: 新增 `ElementId` enum 和 `parse_element_id`

```rust
enum ElementId {
    Aura(AuraNodeId),
    Vnode(VNodeId),
}

fn parse_element_id(s: &str) -> Option<ElementId> {
    if let Some(n) = s.strip_prefix("aura_") {
        return n.parse::<u32>().ok().map(|n| ElementId::Aura(AuraNodeId(n)));
    }
    if let Some(n) = s.strip_prefix("vnode_") {
        return n.parse::<u64>().ok().map(|n| ElementId::Vnode(VNodeId::new(n)));
    }
    None
}
```

### Task 2: 新增 `find_view_by_vnode_path` — 按 VNode path 遍历 View 树

从 `styled_vtree.vtree` 中找到 VNodeId 对应的 VNode，取其 `path`（`Vec<u16>`），然后用 `extract_children`（从 `vnode_converter.rs`，已有）逐级深入 View 树，返回对应的 `&View<DynamicMessage>`。

```rust
fn find_view_by_vnode_path<'a>(
    view: &'a View<DynamicMessage>,
    path: &[u16],
) -> Option<&'a View<DynamicMessage>> {
    let mut current = view;
    for &idx in path {
        let children = extract_children(current);  // 复用 vnode_converter 的函数
        current = children.get(idx as usize)?;
    }
    Some(current)
}
```

### Task 3: 新增 `extract_action_from_view` — 从 View 提取 handler

```rust
/// 从 View 提取事件 handler，返回 (widget_name, event_name)。
/// action_name: "press" / "type" / "toggle" / "select" / "set_value"
fn extract_action_from_view(
    view: &View<DynamicMessage>,
    action_name: &str,
) -> Option<(String, String)> {
    match view {
        View::Button { onclick, .. } if action_name == "press" => {
            extract_dyn_msg(onclick)
        }
        View::Input { on_change, .. } | View::Textarea { on_change, .. }
            if action_name == "type" =>
        {
            on_change.as_ref().and_then(extract_dyn_msg)
        }
        View::Checkbox { on_toggle, .. } if action_name == "toggle" => {
            on_toggle.as_ref().and_then(extract_dyn_msg)
        }
        _ => None,
    }
}

fn extract_dyn_msg(msg: &DynamicMessage) -> Option<(String, String)> {
    match msg {
        DynamicMessage::Typed { widget_name, event_name } =>
            Some((widget_name.clone(), event_name.clone())),
        DynamicMessage::String(name) => Some(("App".into(), name.clone())),
    }
}
```

### Task 4: 修改 `tool_action` — 双路径分发

```rust
// 解析 element_id（支持 aura_N 和 vnode_N）
let element_id = match parse_element_id(element_id_str) { ... };

match element_id {
    ElementId::Aura(aura_id) => {
        // 现有路径：SnapshotBuilder + find_node + execute_action_on_shared（不变）
    }
    ElementId::Vnode(vnode_id) => {
        // 新路径：
        // 1. 从 styled_vtree.vtree.get(vnode_id) 获取 VNode（path + kind）
        // 2. 用 VNode.path 遍历 view 树（find_view_by_vnode_path）
        // 3. 从 View 提取 DynamicMessage（extract_action_from_view）
        // 4. 构建 ActionMessage { widget: widget_name, event: event_name, input_value }
        //    ↑ widget_name 来自 DynamicMessage（如 "EditorPanel"），不再硬编码 "App"
        // 5. send_action(msg)
    }
}
```

### Task 5: 修改 `tool_inspect` — 支持 vnode_N

vnode_N 路径：从 styled_vtree 查 VNode，返回其 kind/label/props。对于事件信息，同样从 View 树提取。

### Task 6: `autoui_snapshot` 已支持 vnode_N（v2 styled_vtree 路径），无需改动

---

## 4. 验证标准

用 MCP 执行完整的 Edit 流程自动化验证：

1. `autoui_snapshot` → 找到 Edit 按钮的 `vnode_N`
2. `autoui_action(element_id="vnode_N", action="press")` → 点击 Edit
3. `autoui_snapshot` → 验证 EditorPanel 切换到编辑模式（Save/Cancel 按钮出现）
4. `autoui_state(fields=["editing"])` → 验证 `editing=true`
5. `autoui_snapshot` → 找到标题输入框的 `vnode_N`
6. `autoui_action(element_id="vnode_N", action="type_text", value="Modified Title")` → 输入文字
7. `autoui_state(fields=["edit_title"])` → 验证 `edit_title="Modified Title"`

### 不在本次范围

- 不改渲染器、不改 InspectorCache、不改 view builder
- 不删除 aura_N 支持（向后兼容）
- aura_N 的已知限制（不覆盖子组件）不修复——vnode_N 是推荐路径

---

## 5. 未来扩展（后续 Task）

- **Task 7+**: snapshot 工具统一输出 vnode_N（当前 v2 已是 vnode_N，但 v1 fallback 仍用 aura_N）
- **Task 8+**: InspectorCache 合并 events 到 ComputedNodeLite（让 vtree 也能显示事件信息）
- **Task 9+**: autoui_wait 支持 vnode_N 级别的等待（等特定元素出现）
- **Task 10+**: 截图 + 视觉对比（回归测试）
