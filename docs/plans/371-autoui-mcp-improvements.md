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

### 4.1 Edit 流程验证（已通过 ✅）

用 MCP 执行完整的 Edit 流程自动化验证（零手动 UI 交互）：

1. `autoui_snapshot` → 找到 Edit 按钮的 `vnode_N`
2. `autoui_action(element_id="vnode_N", action="press")` → 点击 Edit
3. `autoui_snapshot` → 验证 EditorPanel 切换到编辑模式（Save/Cancel 按钮出现）
4. `autoui_state(fields=["editing"])` → 验证 `editing=true`
5. `autoui_snapshot` → 找到标题输入框的 `vnode_N`
6. `autoui_action(element_id="vnode_N", action="type_text", value="Modified Title")` → 输入文字
7. `autoui_state(fields=["edit_title"])` → 验证 `edit_title="Modified Title"`

### 4.2 全按钮点击验证（已通过 ✅）

系统性测试了 UI 中所有 24 个按钮，覆盖全部组件层级：

| # | 按钮类型 | 按钮 | handler | 状态变化 | 结果 |
|---|---------|------|---------|---------|------|
| 1 | **根组件** (App) | New | `.NewNote` | notes +1 | ✅ |
| 2 | **NavTree 标签** | All/Pinned/Recent | `.SelectAll/Pinned/Recent` | active_folder 变化 | ✅ |
| 3 | **NavTree 笔记项** | Welcome/Quick Ideas/... | `.SelectNote` | active_id 变化 | ✅ |
| 4 | **NavTree 文件夹** | + (NewNoteInFolder) | `.NewNote` | notes +1 | ✅ |
| 5 | **NavTree 主题色块** | 5 个色块 | `.SetAccent` | accent_color 变化 | ✅ |
| 6 | **NavTree 暗黑模式** | 🌙 Dark / ☀ Light | `.ToggleDarkMode` | dark_mode 翻转 | ✅ |
| 7 | **EditorPanel** | Edit | `.Edit` | editing→true, edit_title 填充 | ✅ |
| 8 | **EditorPanel** | Delete | `.Delete` | 触发 on_delete 回调 | ✅ |

**结论：通过 MCP vnode_N 可以点击任何按钮**，无论位于根组件还是子组件内部。

### 4.3 已知注意事项

- 点击后视图重建会导致 vnode_N 变化（路径结构变了）。连续操作需要在视图变化后重新 snapshot。
- 同一元素的连续点击（视图不变时）vnode_N 是稳定的。

### 不在本次范围

- 不改渲染器、不改 InspectorCache、不改 view builder
- 不删除 aura_N 支持（向后兼容）
- aura_N 的已知限制（不覆盖子组件）不修复——vnode_N 是推荐路径

---

## 5. 后续改进

### Task 7: VTree 全量展示 + 变化检测（高优先级）✅ 已完成

**动机**：当前验证按钮点击效果需要反复 `autoui_snapshot` + `autoui_state`，效率低。更快的办法是直接通过 `autoui_vtree` 获取**完整渲染节点树**（VTree），根据节点树的变化判断页面是否变化。

**验证结果**：
- `autoui_vtree` 已完整展示所有 73 个渲染节点（无截断、无折叠）
- 所有 `use`/`import` 的子组件（EditorPanel、NavTree）内部元素全部展开
- VTree 节点级 diff 清晰展示变化（如 Edit 点击后：Save/Cancel 新增、📌/标题消失）

**新增 `autoui_find` 工具（Task 7 实现）**：

比 diff 整棵树更精准——Agent 按条件搜索特定组件，直接验证"某个元素是否存在"。

搜索条件（全部可选，AND 组合）：
- `kind`：节点类型（button, input, text, textarea, col, row, checkbox...）
- `label`：对 label/content/value/placeholder 做大小写不敏感子串匹配
- `limit`：最大返回数（默认 20）

返回匹配节点的 Atom 格式（含 vnode_N ID 和路径深度缩进）。

验证场景：
- `autoui_find(kind=button, label=Edit)` → 精确找到 Edit 按钮
- `autoui_find(kind=input)` → 找到搜索输入框
- `autoui_find(label=Welcome)` → 找到按钮 + 文本节点
- 编辑模式验证：点击 Edit 前 `autoui_find(label=Save)` → 不存在；点击后 → 存在 ✓

### Task 8: InspectorCache 合并 events 到 ComputedNodeLite

让 `autoui_vtree` 的输出也能显示每个元素的事件信息（当前 events 字段为空）。需要在渲染器的 probe→cache 合并中复制 events 数据。

### Task 9: autoui_wait 支持 vnode_N 级别的等待

等待特定元素出现/消失，而不是只等待 state 字段变化。

### Task 10: 截图 + 视觉对比（回归测试）

`autoui_screenshot` 已存在。增强为支持截图 diff（对比基线截图），用于回归测试。

---

## 6. Rust 模式（a2r + iced）MCP 支持

### 6.1 背景

当前 MCP 仅在 VM 模式（`auto run -r vm`）中可用。Rust 模式（`auto run -r rust`，a2r 转译的 iced 原生窗口）没有 MCP 支持，无法用同一套测试套件验证。

**架构差异**：

| | VM 模式 | Rust 模式 |
|---|---|---|
| 组件类型 | `DynamicComponent`（运行时 VM） | `C: Component`（编译时 Rust struct） |
| 视图输出 | `View<DynamicMessage>` | `View<C::Msg>` |
| 消息类型 | `DynamicMessage { widget_name, event_name, args }` | 强类型 `C::Msg`（如 `AppMsg::AddTodo`） |
| 状态访问 | `read_state(field)` / `write_state(field, val)` | `Component` trait 无泛型字段访问 |
| MCP | ✅ `start_mcp_server` + `SharedState` | ❌ 无 |

### 6.2 好消息：80% 基础已具备

Rust 模式的 `DevToolsWrapper`（`renderer.rs:6619`）已经：

- **每帧构建 VTree**：`view_element()` 调用 `view_to_vtree_with_paths(self.inner.view())`，生成 VNodeId 索引的完整渲染树，存入 `DevToolsState.live_vtree`
- **支持消息注入**：`WrapperMsg<C>` 有 `Inner(C::Msg)` 和 `Debug(String)` 两个变体，`devtools_update` 分别路由到 `inner.on(msg)` 和 `apply_debug_event`
- **F12 DevTools 面板**：已有 hover/select/inspector 功能

### 6.3 实施步骤

#### Task 11: Rust 模式 MCP 服务器启动

在 `run_app_devtools` / `run_app_with_task_devtools` 中加入 `start_mcp_server` 调用（复用 VM 模式的 MCP 服务器代码）。

**挑战**：MCP 的 `SharedState` 需要 `View<DynamicMessage>` + state 字典，但 Rust 模式是 `View<C::Msg>` + 强类型 struct。需要一个**桥接层**。

**方案**：创建 `RustMcpBridge<C>` 适配器，将 Rust 模式的 `DevToolsWrapper` 包装为 MCP 可用的形式：
- `snapshot()`：从 `DevToolsState.live_vtree` 构建 Atom 文本（已有 `build_aura_from_styled_vtree`，只需适配数据源）
- `action(vnode_id, action_type)`：从 VTree 的 VNode path 遍历 `inner.view()` 找到对应 View，提取其 `C::Msg`（onclick/on_change），包装为 `WrapperMsg::Inner(msg)` 注入
- `state()`：从 VTree 的 props（label/value/content）提取可见状态（不依赖强类型 struct 字段）

#### Task 12: Rust 模式 action 分发

创建 Rust 版的 `find_view_by_path` + `extract_action_from_view`（泛型版本）：
- `find_view_by_path_generic<M>(view: &View<M>, path: &[u16]) -> Option<&View<M>>`：复用 `extract_children` 逻辑
- 从 `View<C::Msg>` 的 Button/Input/Textarea 提取 `onclick`/`on_change` 消息（类型为 `C::Msg`）
- 包装为 `WrapperMsg::Inner(msg)` 并通过 MCP channel 注入 iced 事件循环

**关键约束**：`C::Msg` 必须满足 `Send + 'static`（iced 消息要求）。MCP channel 使用 `mpsc::Sender<WrapperMsg<C>>`。由于 `C` 是泛型的，channel 类型需要泛型化或使用 trait object。

#### Task 13: Rust 模式 MCP subscription

在 `devtools_subscription` 中新增一个 subscription，轮询 MCP channel 并将 action 转为 `WrapperMsg`：

```rust
fn devtools_subscription<C: Component + 'static>(w: &DevToolsWrapper<C>)
    -> iced::Subscription<WrapperMsg<C>>
{
    let inner = w.inner.subscription().map(WrapperMsg::Inner);
    let f12 = /* ... */;
    let win = /* ... */;
    let mcp = mcp_action_subscription_rust::<C>();  // 新增
    iced::Subscription::batch(vec![inner, f12, win, mcp])
}
```

#### Task 14: 验证 Rust 模式 MCP

用与 VM 模式相同的测试套件验证 Rust 模式：
1. `auto run -r rust` 启动 015-notes（a2r iced 窗口）
2. MCP `autoui_snapshot` → 获取完整渲染树
3. MCP `autoui_find(button, Edit)` → 找到 Edit 按钮
4. MCP `autoui_action(vnode_N, press)` → 点击 Edit
5. MCP `autoui_exists(button, Save)` → 验证编辑模式

**预期结果**：VM 模式和 Rust 模式的 MCP 行为一致，同一套测试用例可以通过。

### 6.4 技术风险

| 风险 | 影响 | 缓解 |
|------|------|------|
| `C::Msg` 不满足 `Send` | MCP channel 无法跨线程 | `Component` trait 已要求 `'static`；可能需要加 `Send` bound |
| state 字段无法泛型读取 | `autoui_state` 工具不可用 | 通过 VTree props 间接读取可见状态；或给 Component trait 加 `fn serialize_state() -> HashMap<String, Value>` |
| 泛型 channel 类型 | `start_mcp_server` 需要具体类型 | 使用 `dyn Fn(WrapperMsg)` trait object 或宏泛型化 |
| Rust 编译时间 | a2r 生成 + cargo build 较慢 | 与 VM 模式并行测试，不阻塞 |

### 6.5 不在本次范围

- 不修改 `Component` trait 核心签名（除非必须加 `Send`）
- 不支持 Vue+Rust 模式的 MCP（那是浏览器端，用 Playwright）
- 不实现 Rust 模式的 state 字段读写（首期只支持 action + snapshot + find）
